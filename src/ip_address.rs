use std::collections::HashMap;
use std::net::IpAddr;

#[derive(PartialEq, Debug)]
pub enum MyIpProvider {
    Ipify,
    IpifyV6,
    Httpbin,
    IdentMe,
    IdentMeV6,
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

async fn execute_identme(
    v6: bool,
) -> Result<IpAddr, Box<dyn std::error::Error>> {
    let url = if v6 {
        "https://v6.ident.me/"
    } else {
        "https://v4.ident.me/"
    };

    let resp = reqwest::get(url).await?.text().await?;

    let ip_parsed: Result<IpAddr, _> = resp.parse();

    match ip_parsed {
        Ok(addr) => Ok(addr),
        Err(err) => Err(Box::new(err)),
    }
}

pub async fn current(
    provider: &MyIpProvider,
) -> Result<IpAddr, Box<dyn std::error::Error>> {
    match provider {
        MyIpProvider::Ipify => execute_ipify(false).await,
        MyIpProvider::IpifyV6 => execute_ipify(true).await,
        MyIpProvider::Httpbin => execute_httpbin().await,
        MyIpProvider::IdentMe => execute_identme(false).await,
        MyIpProvider::IdentMeV6 => execute_identme(true).await,
    }
}

#[cfg(test)]
mod ip_tests {
    #[tokio::test]
    async fn ipify_test() {
        let ipv4 = super::execute_ipify(false).await.unwrap();
        println!("IPv4: {}", ipv4);

        let ipv6 = super::execute_ipify(true).await.unwrap();
        println!("IPv6: {}", ipv6);
    }

    #[tokio::test]
    async fn httpbin_test() {
        let ipv4 = super::execute_httpbin().await.unwrap();
        println!("IPv4: {}", ipv4);
    }

    #[tokio::test]
    async fn identme_test() {
        let ipv4 = super::execute_identme(false).await.unwrap();
        println!("IPv4: {}", ipv4);

        let ipv6 = super::execute_identme(true).await.unwrap();
        println!("IPv6: {}", ipv6);
    }
}
