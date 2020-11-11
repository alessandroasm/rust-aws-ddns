use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::net::IpAddr;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum MyIpProvider {
    Ipify,
    IpifyV6,
    Httpbin,
    IdentMe,
    IdentMeV6,
}

#[derive(Debug)]
pub struct IpAddressResolutionError {
    message: String,
}
impl fmt::Display for IpAddressResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error on IP Address resolution: {}", self.message)
    }
}
impl Error for IpAddressResolutionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

/// Information about a provider
struct ProviderInfo {
    pub provider: MyIpProvider,
    pub name: &'static str,
    pub is_v6: bool,
}

lazy_static! {
    static ref PROVIDERS_INFO: Vec<ProviderInfo> = vec![
        ProviderInfo {
            provider: MyIpProvider::Ipify,
            name: "ipify",
            is_v6: false,
        },
        ProviderInfo {
            provider: MyIpProvider::IpifyV6,
            name: "ipify",
            is_v6: true,
        },
        ProviderInfo {
            provider: MyIpProvider::Httpbin,
            name: "httpbin",
            is_v6: false,
        },
        ProviderInfo {
            provider: MyIpProvider::IdentMe,
            name: "identme",
            is_v6: false,
        },
        ProviderInfo {
            provider: MyIpProvider::IdentMeV6,
            name: "identme",
            is_v6: true,
        },
    ];
}

/// Returns true if provider is IPv6
fn is_provider_v6(provider: &MyIpProvider) -> bool {
    let provider_info = PROVIDERS_INFO
        .iter()
        .find(|&info| info.provider == *provider)
        .expect("Unknown provider type");
    provider_info.is_v6
}

/// Returns an interator of providers
/// The first item will be the specified provider, followed by alternative ones.
fn find_provider_with_alternatives(
    provider: &MyIpProvider,
) -> Vec<MyIpProvider> {
    let is_v6 = is_provider_v6(provider);

    let mut providers = vec![*provider];
    let mut alternatives: Vec<MyIpProvider> = PROVIDERS_INFO
        .iter()
        .filter_map(|candidate| {
            if candidate.is_v6 == is_v6 && &candidate.provider != provider {
                Some(candidate.provider)
            } else {
                None
            }
        })
        .collect();

    providers.append(&mut alternatives);
    providers
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

/// Returns the current public ip address
/// This function will try alternatives if the specified provider isn't available.
pub async fn current(
    provider: &MyIpProvider,
) -> Result<IpAddr, Box<dyn std::error::Error>> {
    let providers_to_try = find_provider_with_alternatives(provider);
    for provider in providers_to_try.iter() {
        let res = match provider {
            MyIpProvider::Ipify => execute_ipify(false).await,
            MyIpProvider::IpifyV6 => execute_ipify(true).await,
            MyIpProvider::Httpbin => execute_httpbin().await,
            MyIpProvider::IdentMe => execute_identme(false).await,
            MyIpProvider::IdentMeV6 => execute_identme(true).await,
        };

        if res.is_ok() {
            return res;
        }
    }

    let providers_tried: Vec<String> = providers_to_try
        .into_iter()
        .map(|p| format!("{:?}", p))
        .collect();
    let ex = IpAddressResolutionError {
        message: format!("providers tried: {}", providers_tried.join(", ")),
    };
    Err(Box::new(ex))
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

    #[test]
    fn provider_alternatives() {
        use super::{ find_provider_with_alternatives, MyIpProvider };

        let alts = find_provider_with_alternatives(&MyIpProvider::Httpbin);
        assert_eq!(MyIpProvider::Httpbin, alts[0]);
        assert!(alts.contains(&MyIpProvider::Ipify));
        assert!(!alts.contains(&MyIpProvider::IpifyV6));

        let alts = find_provider_with_alternatives(&MyIpProvider::IdentMeV6);
        assert_eq!(MyIpProvider::IdentMeV6, alts[0]);
        assert!(alts.contains(&MyIpProvider::IpifyV6));
        assert!(!alts.contains(&MyIpProvider::IdentMe));
    }
}
