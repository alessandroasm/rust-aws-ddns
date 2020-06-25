use serde::Deserialize;

pub struct AppAwsCredentials {
    pub access_key: String,
    pub secret_access_key: String,
}

#[derive(Debug, Deserialize)]
struct AwsCsvEntry {
    #[serde(rename = "User name")]
    pub user_name: String,

    #[serde(rename = "Password")]
    pub password: Option<String>,

    #[serde(rename = "Access key ID")]
    pub access_key: String,

    #[serde(rename = "Secret access key")]
    pub secret_access_key: String,

    #[serde(rename = "Console login link")]
    pub console_login_link: Option<String>,
}

pub fn from_csv(csv_file: &str) -> Option<AppAwsCredentials> {
    let csv_file = std::fs::File::open(csv_file).unwrap();

    let mut csv_rdr = csv::Reader::from_reader(csv_file);
    for row in csv_rdr.deserialize::<AwsCsvEntry>() {
        if let Ok(record) = row {
            println!("Record: {:?}", record);
            return Some(AppAwsCredentials {
                access_key: record.access_key,
                secret_access_key: record.secret_access_key,
            });
        }
    }

    None
}
