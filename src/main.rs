use clap::App;

mod aws_credentials;
use aws_credentials::AppAwsCredentials;

mod ip_address;

type AppResult<T> = Result<T, std::boxed::Box<dyn std::error::Error>>;

fn main() -> AppResult<()> {
    let clap_matches = App::new("rust-server-stats-gatherer")
        .version("0.1")
        .author("Alessandro Menezes <alessandroasm@gmail.com>")
        .about("This application gathers stats from several hosts using SSH")
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
    ip_address::current();

    println!("matches: {:?}", &clap_matches);
    Ok(())
}
