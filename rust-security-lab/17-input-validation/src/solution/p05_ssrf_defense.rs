//! # Lesson 05: SSRF Defense (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use url::Url;

#[derive(Debug, Clone)]
pub struct SafeUrl {
    url: Url,
    original: String,
}

impl SafeUrl {
    pub fn as_str(&self) -> &str {
        self.original.as_str()
    }

    pub fn host(&self) -> Option<&str> {
        self.url.host_str()
    }

    pub fn scheme(&self) -> &str {
        self.url.scheme()
    }
}

const BLOCKED_PORTS: &[u16] = &[6379, 27017, 9200, 11211];

pub fn is_private_ip(ip: &str) -> bool {
    // IPv4 checks
    if let Some(octets) = parse_ipv4(ip) {
        let [a, b, _c, _d] = octets;
        // 127.0.0.0/8 (loopback)
        if a == 127 {
            return true;
        }
        // 10.0.0.0/8 (private class A)
        if a == 10 {
            return true;
        }
        // 172.16.0.0/12 (private class B)
        if a == 172 && (16..=31).contains(&b) {
            return true;
        }
        // 192.168.0.0/16 (private class C)
        if a == 192 && b == 168 {
            return true;
        }
        // 169.254.0.0/16 (link-local / cloud metadata)
        if a == 169 && b == 254 {
            return true;
        }
        // 0.0.0.0
        if a == 0 && b == 0 {
            return true;
        }
        return false;
    }

    // IPv6 checks
    let lower = ip.to_lowercase();
    if lower == "::1" || lower.starts_with("fc00::") || lower.starts_with("fe80::") {
        return true;
    }
    if lower == "0:0:0:0:0:0:0:1" {
        return true;
    }

    false
}

fn parse_ipv4(ip: &str) -> Option<[u8; 4]> {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return None;
    }
    let mut octets = [0u8; 4];
    for (i, part) in parts.iter().enumerate() {
        octets[i] = part.parse().ok()?;
    }
    Some(octets)
}

pub fn validate_url(input: &str) -> Result<SafeUrl, String> {
    // Reject URLs with empty authority (e.g., "http:///path")
    let after_scheme = input
        .find("://")
        .map(|i| &input[i + 3..])
        .unwrap_or("");
    if after_scheme.starts_with('/') {
        return Err("URL must have a host".to_string());
    }

    let url = Url::parse(input).map_err(|e| format!("Invalid URL: {}", e))?;

    let scheme = url.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(format!("Scheme '{}' not allowed; use http or https", scheme));
    }

    let host = url
        .host_str()
        .filter(|h| !h.is_empty())
        .ok_or_else(|| "URL must have a host".to_string())?;

    let lower_host = host.to_lowercase();
    if lower_host == "localhost" || lower_host.ends_with(".localhost") {
        return Err("localhost is not allowed".to_string());
    }

    // Check for IPv6 in brackets
    if lower_host.starts_with('[') && lower_host.ends_with(']') {
        let inner = &lower_host[1..lower_host.len() - 1];
        if inner == "::1" {
            return Err("IPv6 localhost is not allowed".to_string());
        }
    }

    if is_private_ip(host) {
        return Err(format!("Private/internal IP '{}' is not allowed", host));
    }

    if let Some(port) = url.port() {
        if BLOCKED_PORTS.contains(&port) {
            return Err(format!("Port {} is blocked", port));
        }
    }

    Ok(SafeUrl {
        url,
        original: input.to_string(),
    })
}

pub struct DomainAllowlist {
    allowed_domains: Vec<String>,
}

impl DomainAllowlist {
    pub fn new(domains: Vec<String>) -> Self {
        Self {
            allowed_domains: domains,
        }
    }

    pub fn is_allowed(&self, url_str: &str) -> bool {
        let safe_url = match validate_url(url_str) {
            Ok(u) => u,
            Err(_) => return false,
        };

        let host = match safe_url.host() {
            Some(h) => h.to_lowercase(),
            None => return false,
        };

        for allowed in &self.allowed_domains {
            let allowed_lower = allowed.to_lowercase();
            if host == allowed_lower || host.ends_with(&format!(".{}", allowed_lower)) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_public_url() {
        let result = validate_url("https://example.com/path");
        assert!(result.is_ok());
    }

    #[test]
    fn test_reject_ftp_scheme() {
        assert!(validate_url("ftp://example.com/file").is_err());
    }

    #[test]
    fn test_reject_localhost() {
        assert!(validate_url("http://localhost/admin").is_err());
    }

    #[test]
    fn test_reject_127_loopback() {
        assert!(validate_url("http://127.0.0.1:8080/api").is_err());
    }

    #[test]
    fn test_reject_cloud_metadata() {
        assert!(validate_url("http://169.254.169.254/latest/meta-data/").is_err());
    }

    #[test]
    fn test_reject_private_10() {
        assert!(validate_url("http://10.0.0.1/internal").is_err());
    }

    #[test]
    fn test_reject_private_192_168() {
        assert!(validate_url("http://192.168.1.1/admin").is_err());
    }

    #[test]
    fn test_reject_blocked_port() {
        assert!(validate_url("http://example.com:6379/").is_err());
    }

    #[test]
    fn test_reject_no_host() {
        assert!(validate_url("http:///path").is_err());
    }

    #[test]
    fn test_is_private_ip_loopback() {
        assert!(is_private_ip("127.0.0.1"));
        assert!(is_private_ip("127.0.0.2"));
    }

    #[test]
    fn test_is_private_ip_cloud_metadata() {
        assert!(is_private_ip("169.254.169.254"));
    }

    #[test]
    fn test_is_private_ip_public() {
        assert!(!is_private_ip("8.8.8.8"));
        assert!(!is_private_ip("142.250.80.46"));
    }

    #[test]
    fn test_allowlist_permits_allowed_domain() {
        let allowlist = DomainAllowlist::new(vec!["example.com".to_string()]);
        assert!(allowlist.is_allowed("https://example.com/path"));
    }

    #[test]
    fn test_allowlist_permits_subdomain() {
        let allowlist = DomainAllowlist::new(vec!["example.com".to_string()]);
        assert!(allowlist.is_allowed("https://api.example.com/data"));
    }

    #[test]
    fn test_allowlist_blocks_unknown() {
        let allowlist = DomainAllowlist::new(vec!["example.com".to_string()]);
        assert!(!allowlist.is_allowed("https://evil.com/steal"));
    }
}
