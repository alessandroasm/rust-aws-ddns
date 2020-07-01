use clap::App;

mod aws_credentials;

mod ip_address;
use ip_address::MyIpProvider;

mod config;
mod route53_client;

const VERSION: &str = env!("CARGO_PKG_VERSION");

static mut QUIET_MODE: bool = false;
fn println(message: &str) {
    unsafe {
        if !QUIET_MODE {
            println!("{}", message);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let clap_matches = App::new("rust-aws-ddns")
        .version(VERSION)
        .author("Alessandro Menezes <alessandroasm@gmail.com>")
        .about("This application implements DDNS backed by AWS Route 53")
        .args_from_usage(
            "-c, --config=[FILE] 'Sets a custom config file'
            --csv=[FILE]         'Sets a custom credentials file'
            -q                   'Quiet mode'",
        )
        .get_matches();

    // Load configuration
    let quiet_mode = clap_matches.is_present("q");
    unsafe {
        QUIET_MODE = quiet_mode;
    }

    let config_file = clap_matches
        .value_of("config")
        .unwrap_or("rust-aws-ddns.yml");
    let app_config = config::AppConfig::parse(config_file, quiet_mode);
    let app_config = app_config.await.unwrap();

    // Get API credentials
    let credentials_file = clap_matches
        .value_of("csv")
        .unwrap_or("aws_user_credentials.csv");
    let mut credentials = aws_credentials::from_csv(credentials_file);

    if credentials.is_none()
        && app_config.aws_access_key.is_some()
        && app_config.aws_secret_access_key.is_some()
    {
        let access_key = app_config.aws_access_key.as_ref().unwrap();
        let secret_access_key =
            app_config.aws_secret_access_key.as_ref().unwrap();

        credentials = Some(aws_credentials::AppAwsCredentials {
            access_key: String::from(access_key),
            secret_access_key: String::from(secret_access_key),
        });
    }

    // Checking and updating IPs
    let route53_client = route53_client::Route53Client::new(credentials);

    // IPv4 First
    if app_config.update_ipv4 {
        let provider_str = if app_config.provider_v4.is_none() {
            ""
        } else {
            app_config.provider_v4.as_ref().unwrap()
        };
        let provider = match provider_str {
            "httpbin" => MyIpProvider::Httpbin,
            _ => MyIpProvider::Ipify,
        };

        let record_set = app_config.record_set.as_ref();
        update_record_set(&app_config, &route53_client, provider, record_set)
            .await?;
    }

    // And then IPv6
    if app_config.update_ipv6 {
        let provider = MyIpProvider::IpifyV6;
        let record_set: &str = app_config
            .record_set_v6
            .as_ref()
            .unwrap_or(&app_config.record_set);

        update_record_set(&app_config, &route53_client, provider, record_set)
            .await?;
    }

    Ok(())
}

async fn update_record_set(
    config: &config::AppConfig,
    client: &route53_client::Route53Client,
    ip_provider: MyIpProvider,
    record_set: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get current IP Address
    let my_ipaddr = ip_address::current(ip_provider).await?;

    client
        .set_ip_address(&config.zone_id, record_set, &my_ipaddr)
        .await;

    Ok(())
}
