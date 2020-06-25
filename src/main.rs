use clap::App;

mod aws_credentials;
use aws_credentials::AppAwsCredentials;

mod ip_address;
use ip_address::MyIpProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let clap_matches = App::new("rust-aws-ddns")
        .version("0.1")
        .author("Alessandro Menezes <alessandroasm@gmail.com>")
        .about("This application configures the current IP Address on AWS Route 53")
        .args_from_usage(
            "-c, --config=[FILE] 'Sets a custom config file'
            --csv=[FILE]         'Sets a custom config file'
            -v                   'Verbose mode'",
        )
        .get_matches();

    // Get API credentials
    let mut credentials: Option<AppAwsCredentials> = None;
    credentials = aws_credentials::from_csv("aws_user_credentials.csv");

    if credentials.is_none() {
        panic!("No AWS credentials found");
    }

    // Get current IP Address
    let my_ipaddr = ip_address::current(MyIpProvider::Ipify).await?;
    if !my_ipaddr.is_ipv4() {
        panic!("The application only supports IPv4");
    }

    println!("matches: {:?}", &clap_matches);
    println!("ip: {:?}", &my_ipaddr);
    Ok(())
}
