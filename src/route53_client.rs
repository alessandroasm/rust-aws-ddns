use std::net::IpAddr;

use crate::aws_credentials::AppAwsCredentials;

use rusoto_core::Region;
use rusoto_route53::{Route53, Route53Client as AwsRoute53Client};

pub struct Route53Client {
    credentials: Option<AppAwsCredentials>,
}

impl Route53Client {
    pub fn new(credentials: Option<AppAwsCredentials>) -> Self {
        Route53Client { credentials }
    }

    fn new_client(&self) -> AwsRoute53Client {
        let region = Region::UsEast1;
        match &self.credentials {
            Some(cred) => {
                let dispatcher = rusoto_core::HttpClient::new()
                    .expect("Failed to create Rusoto HTTP Client");
                let provider = rusoto_core::credential::StaticProvider::new(
                    String::from(&cred.access_key),
                    String::from(&cred.secret_access_key),
                    None,
                    None,
                );
                AwsRoute53Client::new_with(dispatcher, provider, region)
            }
            _ => AwsRoute53Client::new(region),
        }
    }

    pub async fn list_hosted_zones(&self) -> Option<Vec<(String, String)>> {
        let client = self.new_client();

        let request = rusoto_route53::ListHostedZonesRequest {
            delegation_set_id: None,
            marker: None,
            max_items: None,
        };

        let result = client.list_hosted_zones(request).await;
        match result {
            Err(_) => None,
            Ok(res) => {
                let v = res
                    .hosted_zones
                    .iter()
                    .map(|zone| {
                        (String::from(&zone.id), String::from(&zone.name))
                    })
                    .collect();
                Some(v)
            }
        }
    }

    pub async fn set_ip_address(
        &self,
        zone_id: &str,
        record_set: &str,
        ip: &IpAddr,
    ) {
        println!("Updating \"{}\" to {}", record_set, ip);

        let client = self.new_client();
        let (is_recordset_present, is_recordset_uptodate) =
            check_record_set(&client, zone_id, record_set, ip).await;

        // Already up to date, nothing to do
        if is_recordset_present && is_recordset_uptodate {
            println!("   {} is already up to date.", record_set);
            return;
        }

        // We need to update / create the recordset
        update_record_set(&client, zone_id, record_set, ip).await;
        println!("   {} was updated.", record_set);
    }

    // pub fn get_hosted_zone(&self) {
    //     // GET /2013-04-01/hostedzone/Id HTTP/1.1
    // }
}

async fn check_record_set(
    client: &AwsRoute53Client,
    zone_id: &str,
    record_set: &str,
    ip: &IpAddr,
) -> (bool, bool) {
    use rusoto_route53::{
        ListResourceRecordSetsRequest, ListResourceRecordSetsResponse,
    };

    let mut is_recordset_present = false;
    let mut is_recordset_uptodate = false;

    // Fetching the recordSets for the specified zone_id
    let mut prev_response: Option<ListResourceRecordSetsResponse> = None;
    'outer: loop {
        let mut request = ListResourceRecordSetsRequest {
            hosted_zone_id: zone_id.to_string(),
            max_items: None,
            start_record_identifier: None,
            start_record_name: None,
            start_record_type: None,
        };

        if let Some(ref_response) = prev_response {
            request.start_record_identifier =
                ref_response.next_record_identifier;
            request.start_record_name = ref_response.next_record_name;
            request.start_record_type = ref_response.next_record_type;
        }

        let response = client.list_resource_record_sets(request).await.unwrap();

        // Looking for the desired record_set
        for record_set_entry in response.resource_record_sets.iter() {
            if record_set_entry.name == record_set {
                let wanted_record_type =
                    if ip.is_ipv4() { "A" } else { "AAAA" };

                if record_set_entry.type_ == wanted_record_type {
                    is_recordset_present = true;

                    if let Some(records) = &record_set_entry.resource_records {
                        for record in records.iter() {
                            let record_ip: Result<IpAddr, _> =
                                record.value.parse();

                            if record_ip.is_ok() {
                                let record_ip = record_ip.unwrap();
                                if record_ip == *ip {
                                    is_recordset_uptodate = true;
                                    break 'outer;
                                }
                            }
                        }
                    }

                    break 'outer;
                }
            }
        }

        // Fetching next recordSets, if necessary
        if !response.is_truncated {
            break;
        }
        prev_response = Some(response);
    }

    (is_recordset_present, is_recordset_uptodate)
}

async fn update_record_set(
    client: &AwsRoute53Client,
    zone_id: &str,
    record_set: &str,
    ip: &IpAddr,
) {
    use rusoto_route53::{
        Change, ChangeBatch, ChangeResourceRecordSetsRequest, ResourceRecord,
        ResourceRecordSet,
    };

    let new_record_set = ResourceRecordSet {
        alias_target: None,
        failover: None,
        geo_location: None,
        health_check_id: None,
        multi_value_answer: None,
        name: record_set.to_string(),
        region: None,
        set_identifier: None,
        ttl: Some(120),
        traffic_policy_instance_id: None,
        type_: String::from(if ip.is_ipv4() { "A" } else { "AAAA" }),
        weight: None,

        resource_records: Some(vec![ResourceRecord {
            value: ip.to_string(),
        }]),
    };

    let request = ChangeResourceRecordSetsRequest {
        hosted_zone_id: zone_id.to_string(),
        change_batch: ChangeBatch {
            comment: Some("changed by rust-aws-ddns".to_string()),
            changes: vec![Change {
                action: "UPSERT".to_string(),
                resource_record_set: new_record_set,
            }],
        },
    };

    client.change_resource_record_sets(request).await.unwrap();
}
