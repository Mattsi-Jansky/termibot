use crate::plugins::mastodon::api::FetchError;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use url::{Host, Url};

/// Links and media URLs can point anywhere, including the bot's own network
pub async fn ensure_public(url: &Url) -> Result<(), FetchError> {
    let domain = public_domain(url)?;
    let port = url.port_or_known_default().unwrap_or(443);
    let addresses: Vec<_> = tokio::net::lookup_host((domain, port)).await?.collect();

    if addresses.is_empty() || !addresses.iter().all(|address| is_public(address.ip())) {
        return Err(FetchError(format!("{domain} isn't on the public internet")));
    }
    Ok(())
}

fn public_domain(url: &Url) -> Result<&str, FetchError> {
    if url.scheme() != "https" {
        return Err(FetchError(format!("{url} doesn't use https")));
    }
    match url.host() {
        Some(Host::Domain(domain)) if domain != "localhost" && !domain.ends_with(".localhost") => {
            Ok(domain)
        }
        _ => Err(FetchError(format!(
            "{url} doesn't use a public domain name"
        ))),
    }
}

fn is_public(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => is_public_v4(ip),
        IpAddr::V6(ip) => match ip.to_ipv4_mapped() {
            Some(ip) => is_public_v4(ip),
            None => is_public_v6(ip),
        },
    }
}

fn is_public_v4(ip: Ipv4Addr) -> bool {
    let [first, second, ..] = ip.octets();
    let is_shared_address_space = first == 100 && (64..128).contains(&second);
    let is_reserved = first == 0 || first >= 240;

    !(ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_broadcast()
        || ip.is_documentation()
        || ip.is_multicast()
        || ip.is_unspecified()
        || is_shared_address_space
        || is_reserved)
}

fn is_public_v6(ip: Ipv6Addr) -> bool {
    let first_segment = ip.segments()[0];
    let is_unique_local = first_segment & 0xfe00 == 0xfc00;
    let is_link_local = first_segment & 0xffc0 == 0xfe80;

    !(ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_multicast()
        || is_unique_local
        || is_link_local)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(url: &str) -> Url {
        Url::parse(url).unwrap()
    }

    #[test]
    fn should_accept_https_urls_with_domain_names() {
        assert_eq!(
            public_domain(&parse("https://mastodon.social/api/v1/statuses/1")).unwrap(),
            "mastodon.social"
        );
    }

    #[test]
    fn should_refuse_urls_without_https() {
        assert!(public_domain(&parse("http://mastodon.social/")).is_err());
    }

    #[test]
    fn should_refuse_urls_with_ip_addresses_or_localhost() {
        for url in [
            "https://127.0.0.1/",
            "https://192.168.1.1/",
            "https://[::1]/",
            "https://localhost/",
            "https://bot.localhost/",
        ] {
            assert!(public_domain(&parse(url)).is_err(), "{url}");
        }
    }

    #[test]
    fn should_treat_internet_addresses_as_public() {
        for ip in ["1.1.1.1", "151.101.1.140", "2a04:4e42::396"] {
            assert!(is_public(ip.parse().unwrap()), "{ip}");
        }
    }

    #[test]
    fn should_not_treat_local_or_reserved_addresses_as_public() {
        for ip in [
            "127.0.0.1",
            "10.0.0.1",
            "172.16.0.1",
            "192.168.1.1",
            "169.254.169.254",
            "100.64.0.1",
            "0.0.0.0",
            "255.255.255.255",
            "::1",
            "::",
            "fd00::1",
            "fe80::1",
            "::ffff:192.168.1.1",
        ] {
            assert!(!is_public(ip.parse().unwrap()), "{ip}");
        }
    }
}
