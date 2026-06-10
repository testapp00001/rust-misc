//! # Lesson 10: Secure API Versioning
//!
//! ## Why Version APIs?
//!
//! APIs evolve over time. Versioning lets you:
//! - Add features without breaking existing clients
//! - Fix security issues in old versions
//! - Deprecate and remove dangerous endpoints
//!
//! ## Versioning Strategies
//!
//! ### URL Path Versioning
//! ```text
//! GET /v1/users
//! GET /v2/users
//! ```
//!
//! ### Header Versioning
//! ```text
//! GET /users
//! Accept: application/vnd.myapi.v2+json
//! ```
//!
//! ### Query Parameter Versioning
//! ```text
//! GET /users?version=2
//! ```
//!
//! ## Security Implications
//!
//! ### 1. Old Versions May Have Vulnerabilities
//!
//! ```text
//! v1: No input validation → SQL injection
//! v2: Input validation added → safe
//!
//! If v1 is still accessible, attackers target it.
//! ```
//!
//! ### 2. Version Confusion Attacks
//!
//! If version negotiation is ambiguous, attackers may trigger old code paths:
//! ```text
//! GET /v1/../v2/users  →  Which version handles this?
//! GET /users (no version)  →  Fallback to oldest version?
//! ```
//!
//! ### 3. Information Disclosure
//!
//! Different versions may return different amounts of data:
//! ```text
//! v1: Returns email (over-sharing)
//! v2: Returns masked email
//!
//! Attacker requests v1 to get full emails.
//! ```
//!
//! ## Defense
//!
//! 1. Deprecate and remove old versions on a schedule
//! 2. Apply security patches to ALL active versions
//! 3. Default to the LATEST version when none is specified
//! 4. Validate version format strictly
//! 5. Log which version was used for each request
//! 6. Return security headers regardless of version

use std::collections::HashMap;

/// Exercise 1: Parse an API version from a URL path.
///
/// Extract the version from a URL path like "/v1/users" or "/v2/posts/123".
///
/// The version is the "v{N}" segment immediately after the first "/".
///
/// Returns:
/// - `Ok(version_number)` — the parsed version number
/// - `Err("no_version")` — if the path doesn't start with /v{N}
/// - `Err("invalid_version")` — if "v" is not followed by a valid number
///
/// Examples:
/// - "/v1/users" → Ok(1)
/// - "/v23/posts/123" → Ok(23)
/// - "/users" → Err("no_version")
/// - "/vabc/users" → Err("invalid_version")
///
/// Hints:
/// - Check if path starts with "/v"
/// - Extract the segment between "/v" and the next "/" (or end of string)
/// - Parse the segment as u32
pub fn parse_version_from_path(path: &str) -> Result<u32, &'static str> {
    todo!("Parse API version from URL path")
}

/// Exercise 2: Parse an API version from an Accept header.
///
/// Extract the version from a header like:
/// `application/vnd.myapi.v2+json`
///
/// The format is: `application/vnd.{name}.v{version}+json`
///
/// Returns:
/// - `Ok(version)` if the header matches the pattern
/// - `Err("no_version")` if the header doesn't contain version info
/// - `Err("invalid_version")` if the version number can't be parsed
///
/// Hints:
/// - Look for ".v" in the header value
/// - Extract the number after ".v" and before "+"
/// - Parse as u32
pub fn parse_version_from_accept(accept: &str) -> Result<u32, &'static str> {
    todo!("Parse API version from Accept header")
}

/// Exercise 3: Negotiate the API version from multiple sources.
///
/// Determine the API version using this priority:
/// 1. URL path version (highest priority)
/// 2. Accept header version
/// 3. Query parameter `version`
/// 4. Default version (fallback)
///
/// Returns the negotiated version number.
///
/// Parameters:
/// - `path`: the URL path
/// - `accept_header`: the Accept header value (Option)
/// - `query_params`: the query parameters as a HashMap
/// - `default_version`: the default version if none is specified
///
/// Hints:
/// - Try `parse_version_from_path` first
/// - If that fails, try `parse_version_from_accept`
/// - If that fails, look for "version" in query_params
/// - If all fail, return default_version
pub fn negotiate_version(
    path: &str,
    accept_header: Option<&str>,
    query_params: &HashMap<String, String>,
    default_version: u32,
) -> u32 {
    todo!("Negotiate API version from multiple sources")
}

/// Exercise 4: Check if an API version is still supported.
///
/// Given a list of supported versions and a list of deprecated versions,
/// determine the status of a requested version.
///
/// Returns:
/// - `Ok(())` if the version is supported and not deprecated
/// - `Err("version_deprecated")` if the version is deprecated (but still works)
/// - `Err("version_removed")` if the version is not in the supported list
///
/// A deprecated version is one that's in the supported list but also in the
/// deprecated list. It still works but should return a deprecation warning header.
///
/// Hints:
/// - Check if version is in supported_versions first
/// - If not supported → removed
/// - If supported and in deprecated_versions → deprecated
/// - If supported and not deprecated → ok
pub fn check_version_status(
    version: u32,
    supported_versions: &[u32],
    deprecated_versions: &[u32],
) -> Result<(), &'static str> {
    todo!("Check if an API version is supported or deprecated")
}

/// Exercise 5: Build deprecation warning headers.
///
/// When a client uses a deprecated version, include warning headers in the
/// response. Build a HashMap of deprecation-related headers.
///
/// For a deprecated version, return:
/// - `Deprecation: true`
/// - `Sunset: {sunset_date}` (the date the version will be removed)
/// - `Link: </v{latest}>; rel="successor-version"` (the latest version)
///
/// For a removed version, return an empty HashMap (the request should be rejected).
///
/// Hints:
/// - Check version status first
/// - Build the header map for deprecated versions
pub fn build_deprecation_headers(
    version: u32,
    supported_versions: &[u32],
    deprecated_versions: &[u32],
    latest_version: u32,
    sunset_date: &str,
) -> HashMap<String, String> {
    todo!("Build deprecation warning headers")
}

/// Exercise 6: Validate that a version upgrade is secure.
///
/// When upgrading from one API version to another, check that the upgrade
/// doesn't introduce security regressions. Given a list of security features
/// per version, ensure the new version has all the security features of the old one.
///
/// Each version has a list of required security features (e.g., "auth", "rate_limit").
///
/// Returns:
/// - `Ok(new_features)` — list of features added in the new version
/// - `Err(missing_features)` — list of features present in old version but missing in new
///
/// Hints:
/// - Get features for old version and new version
/// - Check that all old features are present in new version
/// - Compute the set difference (features in new but not in old)
pub fn validate_version_upgrade(
    old_version: u32,
    new_version: u32,
    version_features: &HashMap<u32, Vec<String>>,
) -> Result<Vec<String>, Vec<String>> {
    todo!("Validate that a version upgrade doesn't regress security")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_basic() {
        assert_eq!(parse_version_from_path("/v1/users").unwrap(), 1);
    }

    #[test]
    fn test_parse_version_multi_digit() {
        assert_eq!(parse_version_from_path("/v23/posts/123").unwrap(), 23);
    }

    #[test]
    fn test_parse_version_no_version() {
        assert_eq!(parse_version_from_path("/users").unwrap_err(), "no_version");
    }

    #[test]
    fn test_parse_version_invalid() {
        assert_eq!(parse_version_from_path("/vabc/users").unwrap_err(), "invalid_version");
    }

    #[test]
    fn test_parse_accept_version() {
        assert_eq!(parse_version_from_accept("application/vnd.myapi.v2+json").unwrap(), 2);
    }

    #[test]
    fn test_parse_accept_no_version() {
        assert_eq!(parse_version_from_accept("application/json").unwrap_err(), "no_version");
    }

    #[test]
    fn test_parse_accept_invalid() {
        assert_eq!(parse_version_from_accept("application/vnd.myapi.vXYZ+json").unwrap_err(), "invalid_version");
    }

    #[test]
    fn test_negotiate_from_path() {
        let params = HashMap::new();
        assert_eq!(negotiate_version("/v2/users", None, &params, 1), 2);
    }

    #[test]
    fn test_negotiate_from_accept() {
        let params = HashMap::new();
        assert_eq!(negotiate_version("/users", Some("application/vnd.api.v3+json"), &params, 1), 3);
    }

    #[test]
    fn test_negotiate_from_query() {
        let mut params = HashMap::new();
        params.insert("version".to_string(), "5".to_string());
        assert_eq!(negotiate_version("/users", None, &params, 1), 5);
    }

    #[test]
    fn test_negotiate_default() {
        let params = HashMap::new();
        assert_eq!(negotiate_version("/users", None, &params, 1), 1);
    }

    #[test]
    fn test_negotiate_path_highest_priority() {
        let mut params = HashMap::new();
        params.insert("version".to_string(), "5".to_string());
        assert_eq!(negotiate_version("/v2/users", Some("application/vnd.api.v3+json"), &params, 1), 2);
    }

    #[test]
    fn test_version_status_supported() {
        assert!(check_version_status(2, &[1, 2, 3], &[1]).is_ok());
    }

    #[test]
    fn test_version_status_deprecated() {
        assert_eq!(check_version_status(1, &[1, 2, 3], &[1]).unwrap_err(), "version_deprecated");
    }

    #[test]
    fn test_version_status_removed() {
        assert_eq!(check_version_status(0, &[1, 2, 3], &[1]).unwrap_err(), "version_removed");
    }

    #[test]
    fn test_deprecation_headers_deprecated() {
        let headers = build_deprecation_headers(1, &[1, 2, 3], &[1], 3, "2025-01-01");
        assert_eq!(headers.get("Deprecation").unwrap(), "true");
        assert_eq!(headers.get("Sunset").unwrap(), "2025-01-01");
        assert!(headers.get("Link").unwrap().contains("/v3"));
    }

    #[test]
    fn test_deprecation_headers_current() {
        let headers = build_deprecation_headers(3, &[1, 2, 3], &[1], 3, "2025-01-01");
        assert!(headers.is_empty());
    }

    #[test]
    fn test_upgrade_valid() {
        let mut features = HashMap::new();
        features.insert(1, vec!["auth".to_string(), "rate_limit".to_string()]);
        features.insert(2, vec!["auth".to_string(), "rate_limit".to_string(), "csrf".to_string()]);
        let added = validate_version_upgrade(1, 2, &features).unwrap();
        assert!(added.contains(&"csrf".to_string()));
    }

    #[test]
    fn test_upgrade_regression() {
        let mut features = HashMap::new();
        features.insert(1, vec!["auth".to_string(), "rate_limit".to_string()]);
        features.insert(2, vec!["auth".to_string()]); // Missing rate_limit!
        let missing = validate_version_upgrade(1, 2, &features).unwrap_err();
        assert!(missing.contains(&"rate_limit".to_string()));
    }

    #[test]
    fn test_upgrade_same_features() {
        let mut features = HashMap::new();
        features.insert(1, vec!["auth".to_string()]);
        features.insert(2, vec!["auth".to_string()]);
        let added = validate_version_upgrade(1, 2, &features).unwrap();
        assert!(added.is_empty());
    }
}
