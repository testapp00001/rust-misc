//! # Lesson 05: SSRF Defense
//!
//! ## The Problem
//!
//! Server-Side Request Forgery (SSRF) occurs when an attacker tricks your
//! server into making requests to internal services that should not be
//! accessible from the internet.
//!
//! ```ignore
//! // VULNERABLE: Fetch any URL the user provides
//! fn fetch_avatar(url: &str) -> Result<Image, Error> {
//!     let response = reqwest::get(url).await?;
//!     // Attacker passes: http://169.254.169.254/latest/meta-data/iam/
//!     // Server fetches AWS metadata, leaking credentials!
//! }
//! ```
//!
//! ## Attack Targets
//!
//! | Target                   | URL Example                                | Impact                  |
//! |--------------------------|--------------------------------------------|-------------------------|
//! | Cloud metadata           | `http://169.254.169.254/latest/meta-data/` | Steal IAM credentials   |
//! | Internal services        | `http://internal-db:5432/admin`            | Access internal APIs    |
//! | Localhost services       | `http://127.0.0.1:6379/`                   | Redis, databases, etc.  |
//! | File protocol            | `file:///etc/passwd`                        | Read local files        |
//! | DNS rebinding            | `http://attacker-controlled.com`           | Bypass IP checks        |
//!
//! ## Defense: URL Validation
//!
//! 1. **Protocol allowlist**: Only allow `http` and `https`
//! 2. **Hostname blocklist**: Reject `localhost`, `127.0.0.1`, `0.0.0.0`,
//!    `169.254.169.254`, `::1`, and private IP ranges
//! 3. **DNS resolution check**: Resolve the hostname and verify the IP is
//!    not in a private range (defense against DNS rebinding)
//! 4. **Allowlist approach**: Only allow requests to known, trusted domains

use url::Url;

/// A validated URL that has passed SSRF checks.
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

/// Validate a URL for safe server-side fetching.
///
/// Checks:
/// 1. URL must parse correctly
/// 2. Scheme must be `http` or `https` only
/// 3. Host must be present
/// 4. Host must not be a local/internal address:
///    - `localhost`
///    - `127.0.0.1`, `127.x.x.x`
///    - `0.0.0.0`
///    - `10.x.x.x` (private class A)
///    - `172.16-31.x.x` (private class B)
///    - `192.168.x.x` (private class C)
///    - `169.254.x.x` (link-local / cloud metadata)
///    - `::1`, `fc00::`, `fe80::` (IPv6 private/local)
///    - `[::1]` (IPv6 localhost in brackets)
/// 5. Port must not be on the blocked list: [6379, 27017, 9200, 11211]
///    (Redis, MongoDB, Elasticsearch, Memcached)
///
/// Returns Ok(SafeUrl) if the URL passes all checks.
pub fn validate_url(input: &str) -> Result<SafeUrl, String> {
    todo!("Implement SSRF-safe URL validation")
}

/// Check if an IP address string represents a private/internal address.
///
/// Handles both IPv4 and IPv6.
pub fn is_private_ip(ip: &str) -> bool {
    todo!("Check if IP is in a private/internal range")
}

/// A URL allowlist that restricts requests to known-good domains.
///
/// This is the strongest SSRF defense: only allow specific domains.
pub struct DomainAllowlist {
    allowed_domains: Vec<String>,
}

impl DomainAllowlist {
    pub fn new(domains: Vec<String>) -> Self {
        Self {
            allowed_domains: domains,
        }
    }

    /// Check if a URL is allowed by this allowlist.
    ///
    /// The URL must:
    /// 1. Pass basic SSRF validation (via `validate_url`)
    /// 2. Have a hostname that matches or is a subdomain of an allowed domain
    ///
    /// For example, if "example.com" is allowed, then "api.example.com"
    /// and "sub.api.example.com" are also allowed.
    pub fn is_allowed(&self, url_str: &str) -> bool {
        todo!("Check URL against allowlist")
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
