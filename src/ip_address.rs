use std::collections::HashMap;
use std::net::IpAddr;

pub enum MyIpProvider {
    Ipify,
    IpifyV6,
    Httpbin,
}

async fn execute_ipify(v6: bool) -> Result<IpAddr, Box<dyn std::error::Error>> {
    let url = if v6 {
        "https://api6.ipify.org?format=json"
    } else {
        "https://api.ipify.org?format=json"
    };
    let resp = reqwest::get(url)
        .await?
        .json::<HashMap<String, String>>()
        .await?;

    let ip = &resp["ip"];
    let ip_parsed: Result<IpAddr, _> = ip.parse();

    match ip_parsed {
        Ok(addr) => Ok(addr),
        Err(err) => Err(Box::new(err)),
    }
}

async fn execute_httpbin() -> Result<IpAddr, Box<dyn std::error::Error>> {
    let resp = reqwest::get("https://httpbin.org/ip")
        .await?
        .json::<HashMap<String, String>>()
        .await?;

    let ip = &resp["origin"];
    let ip_parsed: Result<IpAddr, _> = ip.parse();

    match ip_parsed {
        Ok(addr) => Ok(addr),
        Err(err) => Err(Box::new(err)),
    }
}

pub async fn current(
    provider: MyIpProvider,
) -> Result<IpAddr, Box<dyn std::error::Error>> {
    match provider {
        MyIpProvider::Ipify => execute_ipify(false).await,
        MyIpProvider::IpifyV6 => execute_ipify(true).await,
        MyIpProvider::Httpbin => execute_httpbin().await,
    }
}