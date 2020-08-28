use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub zone_id: String,
    pub record_set: String,
    pub record_set_v6: Option<String>,
    pub update_ipv4: bool,
    pub update_ipv6: bool,
    pub check_before_updating: Option<bool>,

    pub provider_v4: Option<String>,

    pub aws_access_key: Option<String>,
    pub aws_secret_access_key: Option<String>,
}

impl AppConfig {
    pub async fn parse(
        config_file: &str,
        is_in_quiet_mode: bool,
    ) -> Option<Self> {
        // Opening and parsing the configuration file
        let f = std::fs::File::open(config_file);
        if let Err(io_err) = f {
            use std::io::ErrorKind;

            return match io_err.kind() {
                ErrorKind::NotFound => {
                    if is_in_quiet_mode {
                        panic!("Configuration file not found!");
                    }

                    println!(
                        "Config file not found! Starting configuration wizard\n"
                    );
                    AppConfig::run_config_wizard(config_file).await
                }
                _ => None,
            };
        }

        let f = f.unwrap();
        let config: AppConfig = serde_yaml::from_reader(f).unwrap();
        Some(config)
    }

    /// Starts a wizard to generate a valid configuration file
    async fn run_config_wizard(config_file: &str) -> Option<Self> {
        loop {
            // AWS Access Key
            print!("AWS Access Key [Blank for env / system credentials]: ");
            let aws_access_key = read_line();
            let aws_access_key = String::from(aws_access_key.trim());

            let (aws_access_key, aws_secret_key) = if !aws_access_key.is_empty()
            {
                print!("AWS Secret Key: ");
                let secret_key = read_line();
                let secret_key = String::from(secret_key.trim());

                (Some(aws_access_key), Some(secret_key))
            } else {
                (None, None)
            };

            // Fetch hosted zones
            let hosted_zone_id: &str;
            let hosted_zone_name: &str;
            let credentials = if aws_access_key.is_none() {
                None
            } else {
                let access_key = String::from(aws_access_key.as_ref().unwrap());
                let secret_key = String::from(aws_secret_key.as_ref().unwrap());

                Some(crate::aws_credentials::AppAwsCredentials {
                    access_key,
                    secret_access_key: secret_key,
                })
            };

            let client = crate::route53_client::Route53Client::new(credentials);
            let hosted_zones = client.list_hosted_zones().await;

            if hosted_zones.is_none() {
                // It wasn't possible to fetch hosted zones. Restart the wizard
                println!("\nThere was an error fetching Route53 hosted zones. Check your credentials\n");
                continue;
            }

            let hosted_zones = hosted_zones.unwrap();
            if hosted_zones.is_empty() {
                panic!("There are no configured Hosted Zones on this Route 53 account. Please create a hosted zone and run this application again.");
            }

            // Ask for zone_id
            loop {
                println!("\nSelect the desired Hosted Zone:");
                hosted_zones.iter().enumerate().for_each(|entry| {
                    println!(
                        "{}. {} ({})",
                        entry.0 + 1,
                        (entry.1).1,
                        (entry.1).0
                    )
                });

                let hosted_zone_idx = read_int("Hosted Zone: ");
                if hosted_zone_idx > 0 {
                    let hosted_zone_idx = hosted_zone_idx as usize;
                    if hosted_zone_idx <= hosted_zones.len() {
                        let hosted_zone = &hosted_zones[hosted_zone_idx - 1];
                        hosted_zone_id = &hosted_zone.0;
                        hosted_zone_name = &hosted_zone.1;

                        break;
                    }
                }
            }

            // Ask for record_sets
            let update_ipv4 = read_int("Update IPv4 (0 - No): ") != 0;
            let update_ipv6 = read_int("Update IPv6 (0 - No): ") != 0;

            let prompt =
                format!("IPv4 record set prefix (xxx.{}): ", hosted_zone_name);
            let record_set_v4 = read_non_blank_line(&prompt).to_lowercase();
            let record_set_v4 =
                format!("{}.{}", record_set_v4, hosted_zone_name);

            let record_set_v6 = if update_ipv6 {
                print!("IPv6 record set prefix (xxx.{}): ", hosted_zone_name);
                let record_set_v6_str = read_line().trim().to_lowercase();

                if record_set_v6_str.is_empty() {
                    None
                } else {
                    Some(format!("{}.{}", record_set_v6_str, hosted_zone_name))
                }
            } else {
                None
            };

            // Write configuration out
            let config = AppConfig {
                zone_id: String::from(hosted_zone_id),
                record_set: record_set_v4,
                record_set_v6,
                update_ipv4,
                update_ipv6,
                check_before_updating: Some(true),

                provider_v4: None,

                aws_access_key,
                aws_secret_access_key: aws_secret_key,
            };

            let file = std::fs::File::create(config_file).unwrap();
            serde_yaml::to_writer(file, &config).unwrap();

            return Some(config);
        }
    }
}

fn read_line() -> String {
    use std::io::{self, BufRead, Write};

    let mut stdout = io::stdout();
    stdout.flush().unwrap();

    let mut line = String::new();
    let stdin = io::stdin();

    stdin.lock().read_line(&mut line).unwrap();
    line
}

fn read_int(prompt: &str) -> i32 {
    loop {
        print!("{}", prompt);

        let line = read_line();
        let line = line.trim();

        let n = line.parse::<i32>();
        if let Ok(n) = n {
            return n;
        }
    }
}

fn read_non_blank_line(prompt: &str) -> String {
    loop {
        print!("{}", prompt);

        let line = read_line();
        let line = line.trim();

        if !line.is_empty() {
            return String::from(line);
        }
    }
}
