use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub zone_id: String,
    pub record_set: String,
    pub record_set_v6: Option<String>,
    pub update_ipv4: bool,
    pub update_ipv6: bool,

    pub provider_v4: Option<String>,
}

impl AppConfig {
    pub fn parse(config_file: &str) -> Option<Self> {
        // Opening and parsing the configuration file
        let f = std::fs::File::open(config_file);
        if f.is_err() {
            return None;
        }

        let f = f.unwrap();
        let config: AppConfig = serde_yaml::from_reader(f).unwrap();
        Some(config)
    }
}
