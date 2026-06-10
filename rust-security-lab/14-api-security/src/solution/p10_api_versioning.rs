//! # Lesson 10: Secure API Versioning (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::{HashMap, HashSet};

pub fn parse_version_from_path(path: &str) -> Result<u32, &'static str> {
    let path = path.strip_prefix('/').unwrap_or(path);

    if !path.starts_with('v') {
        return Err("no_version");
    }

    let after_v = &path[1..];
    let version_str: String = after_v.chars().take_while(|c| c.is_ascii_digit()).collect();

    if version_str.is_empty() {
        return Err("invalid_version");
    }

    version_str.parse().map_err(|_| "invalid_version")
}

pub fn parse_version_from_accept(accept: &str) -> Result<u32, &'static str> {
    let v_pos = accept.find(".v").ok_or("no_version")?;
    let after_v = &accept[v_pos + 2..];

    let version_str: String = after_v.chars().take_while(|c| c.is_ascii_digit()).collect();

    if version_str.is_empty() {
        return Err("invalid_version");
    }

    version_str.parse().map_err(|_| "invalid_version")
}

pub fn negotiate_version(
    path: &str,
    accept_header: Option<&str>,
    query_params: &HashMap<String, String>,
    default_version: u32,
) -> u32 {
    // Priority 1: URL path
    if let Ok(v) = parse_version_from_path(path) {
        return v;
    }

    // Priority 2: Accept header
    if let Some(accept) = accept_header {
        if let Ok(v) = parse_version_from_accept(accept) {
            return v;
        }
    }

    // Priority 3: Query parameter
    if let Some(v_str) = query_params.get("version") {
        if let Ok(v) = v_str.parse::<u32>() {
            return v;
        }
    }

    // Priority 4: Default
    default_version
}

pub fn check_version_status(
    version: u32,
    supported_versions: &[u32],
    deprecated_versions: &[u32],
) -> Result<(), &'static str> {
    if !supported_versions.contains(&version) {
        return Err("version_removed");
    }

    if deprecated_versions.contains(&version) {
        return Err("version_deprecated");
    }

    Ok(())
}

pub fn build_deprecation_headers(
    version: u32,
    supported_versions: &[u32],
    deprecated_versions: &[u32],
    latest_version: u32,
    sunset_date: &str,
) -> HashMap<String, String> {
    let mut headers = HashMap::new();

    match check_version_status(version, supported_versions, deprecated_versions) {
        Err("version_deprecated") => {
            headers.insert("Deprecation".to_string(), "true".to_string());
            headers.insert("Sunset".to_string(), sunset_date.to_string());
            headers.insert(
                "Link".to_string(),
                format!("</v{}>; rel=\"successor-version\"", latest_version),
            );
        }
        _ => {
            // No headers for current or removed versions
        }
    }

    headers
}

pub fn validate_version_upgrade(
    old_version: u32,
    new_version: u32,
    version_features: &HashMap<u32, Vec<String>>,
) -> Result<Vec<String>, Vec<String>> {
    let old_features: HashSet<&String> = version_features
        .get(&old_version)
        .map(|v| v.iter().collect())
        .unwrap_or_default();

    let new_features: HashSet<&String> = version_features
        .get(&new_version)
        .map(|v| v.iter().collect())
        .unwrap_or_default();

    // Check for missing features (regression)
    let missing: Vec<String> = old_features
        .difference(&new_features)
        .map(|s| (*s).clone())
        .collect();

    if !missing.is_empty() {
        return Err(missing);
    }

    // Return new features (additions)
    let added: Vec<String> = new_features
        .difference(&old_features)
        .map(|s| (*s).clone())
        .collect();

    Ok(added)
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
        features.insert(2, vec!["auth".to_string()]);
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
