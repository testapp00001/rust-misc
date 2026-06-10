//! # Lesson 07: Image Scanning (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Negligible,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Cve {
    pub id: String,
    pub package: String,
    pub installed_version: String,
    pub fixed_version: String,
    pub severity: Severity,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScanResult {
    pub image: String,
    pub cves: Vec<Cve>,
    pub scan_timestamp: String,
    pub scanner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScanPolicy {
    pub block_on_critical: bool,
    pub block_on_high: bool,
    pub block_on_medium: bool,
    pub max_scan_age_hours: u32,
}

/// Count CVEs grouped by severity.
pub fn count_by_severity(scan: &ScanResult) -> (usize, usize, usize, usize, usize) {
    let critical = scan.cves.iter().filter(|c| c.severity == Severity::Critical).count();
    let high = scan.cves.iter().filter(|c| c.severity == Severity::High).count();
    let medium = scan.cves.iter().filter(|c| c.severity == Severity::Medium).count();
    let low = scan.cves.iter().filter(|c| c.severity == Severity::Low).count();
    let negligible = scan.cves.iter().filter(|c| c.severity == Severity::Negligible).count();
    (critical, high, medium, low, negligible)
}

/// Simple timestamp parser for ISO 8601 "YYYY-MM-DDTHH:MM:SSZ" format.
fn parse_timestamp(ts: &str) -> Option<(u32, u32, u32, u32, u32, u32)> {
    let parts: Vec<&str> = ts.split('T').collect();
    if parts.len() != 2 { return None; }
    let date: Vec<&str> = parts[0].split('-').collect();
    let time: Vec<&str> = parts[1].trim_end_matches('Z').split(':').collect();
    if date.len() != 3 || time.len() != 3 { return None; }
    Some((
        date[0].parse().ok()?,
        date[1].parse().ok()?,
        date[2].parse().ok()?,
        time[0].parse().ok()?,
        time[1].parse().ok()?,
        time[2].parse().ok()?,
    ))
}

/// Convert a timestamp to hours for age calculation.
fn to_hours(ts: &str) -> Option<u64> {
    let (y, m, d, h, min, s) = parse_timestamp(ts)?;
    let days = (y as u64 - 2000) * 365 + (m as u64 - 1) * 30 + (d as u64 - 1);
    Some(days * 24 + h as u64 + min as u64 / 60 + s as u64 / 3600)
}

/// Check if scan result passes deployment policy.
pub fn check_policy(scan: &ScanResult, policy: &ScanPolicy, current_timestamp: &str) -> Result<(), String> {
    let (critical, high, medium, _, _) = count_by_severity(scan);

    if policy.block_on_critical && critical > 0 {
        return Err(format!("Blocked: {} Critical CVEs found", critical));
    }

    if policy.block_on_high && high > 0 {
        return Err(format!("Blocked: {} High CVEs found", high));
    }

    if policy.block_on_medium && medium > 0 {
        return Err(format!("Blocked: {} Medium CVEs found", medium));
    }

    if policy.max_scan_age_hours > 0 {
        let scan_hours = to_hours(&scan.scan_timestamp);
        let current_hours = to_hours(current_timestamp);

        match (scan_hours, current_hours) {
            (Some(scan_h), Some(curr_h)) => {
                if curr_h > scan_h && (curr_h - scan_h) > policy.max_scan_age_hours as u64 {
                    return Err(format!(
                        "Blocked: scan is {} hours old, max allowed is {}",
                        curr_h - scan_h, policy.max_scan_age_hours
                    ));
                }
            }
            _ => {
                return Err("Blocked: unable to parse scan timestamp, assuming too old".to_string());
            }
        }
    }

    Ok(())
}

/// Find fixable CVEs sorted by severity (Critical first).
pub fn find_fixable_cves(scan: &ScanResult) -> Vec<&Cve> {
    let mut fixable: Vec<&Cve> = scan.cves.iter()
        .filter(|c| !c.fixed_version.is_empty())
        .collect();
    fixable.sort_by(|a, b| b.severity.cmp(&a.severity));
    fixable
}

/// Generate a remediation report.
pub fn remediation_report(scan: &ScanResult) -> String {
    let (critical, high, medium, low, _) = count_by_severity(scan);
    let fixable = find_fixable_cves(scan);
    let fixable_count = fixable.len();

    let top_fix = if let Some(top) = fixable.first() {
        format!("{} {} -> {}", top.package, top.installed_version, top.fixed_version)
    } else {
        "N/A".to_string()
    };

    format!(
        "Image Scan Report: {}\n\
         Scanner: {}\n\
         Scanned: {}\n\n\
         Critical: {}\n\
         High:     {}\n\
         Medium:   {}\n\
         Low:      {}\n\n\
         Fixable CVEs: {}\n\
         Top fix: {}",
        scan.image, scan.scanner, scan.scan_timestamp,
        critical, high, medium, low,
        fixable_count, top_fix
    )
}

/// Filter CVEs by package name (case-insensitive).
pub fn filter_by_package<'a>(scan: &'a ScanResult, package: &str) -> Vec<&'a Cve> {
    let package_lower = package.to_lowercase();
    scan.cves.iter()
        .filter(|c| c.package.to_lowercase() == package_lower)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_scan() -> ScanResult {
        ScanResult {
            image: "myapp:1.0.0".to_string(),
            cves: vec![
                Cve {
                    id: "CVE-2024-0001".to_string(),
                    package: "openssl".to_string(),
                    installed_version: "1.1.1k".to_string(),
                    fixed_version: "1.1.1w".to_string(),
                    severity: Severity::Critical,
                    description: "Buffer overflow in OpenSSL".to_string(),
                },
                Cve {
                    id: "CVE-2024-0002".to_string(),
                    package: "curl".to_string(),
                    installed_version: "7.74.0".to_string(),
                    fixed_version: "7.88.1".to_string(),
                    severity: Severity::High,
                    description: "HTTP request smuggling".to_string(),
                },
                Cve {
                    id: "CVE-2024-0003".to_string(),
                    package: "libxml2".to_string(),
                    installed_version: "2.9.10".to_string(),
                    fixed_version: String::new(),
                    severity: Severity::Medium,
                    description: "XML parsing issue".to_string(),
                },
                Cve {
                    id: "CVE-2024-0004".to_string(),
                    package: "zlib".to_string(),
                    installed_version: "1.2.11".to_string(),
                    fixed_version: "1.2.13".to_string(),
                    severity: Severity::Low,
                    description: "Minor compression issue".to_string(),
                },
            ],
            scan_timestamp: "2024-01-15T10:00:00Z".to_string(),
            scanner: "trivy".to_string(),
        }
    }

    fn default_policy() -> ScanPolicy {
        ScanPolicy {
            block_on_critical: true,
            block_on_high: true,
            block_on_medium: false,
            max_scan_age_hours: 24,
        }
    }

    #[test]
    fn test_count_by_severity() {
        let scan = sample_scan();
        let (critical, high, medium, low, _negligible) = count_by_severity(&scan);
        assert_eq!(critical, 1);
        assert_eq!(high, 1);
        assert_eq!(medium, 1);
        assert_eq!(low, 1);
    }

    #[test]
    fn test_check_policy_passes() {
        let mut scan = sample_scan();
        scan.cves.retain(|c| c.severity <= Severity::Medium);
        let policy = default_policy();
        assert!(check_policy(&scan, &policy, "2024-01-15T11:00:00Z").is_ok());
    }

    #[test]
    fn test_check_policy_blocks_critical() {
        let scan = sample_scan();
        let policy = default_policy();
        let result = check_policy(&scan, &policy, "2024-01-15T11:00:00Z");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Critical") || err.contains("critical"));
    }

    #[test]
    fn test_check_policy_scan_too_old() {
        let mut scan = sample_scan();
        scan.cves.clear();
        let policy = ScanPolicy {
            block_on_critical: false,
            block_on_high: false,
            block_on_medium: false,
            max_scan_age_hours: 24,
        };
        let result = check_policy(&scan, &policy, "2024-01-17T10:00:01Z");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("age") || err.contains("old") || err.contains("stale"));
    }

    #[test]
    fn test_find_fixable_cves() {
        let scan = sample_scan();
        let fixable = find_fixable_cves(&scan);
        assert_eq!(fixable.len(), 3);
        assert_eq!(fixable[0].severity, Severity::Critical);
    }

    #[test]
    fn test_remediation_report() {
        let scan = sample_scan();
        let report = remediation_report(&scan);
        assert!(report.contains("myapp:1.0.0"));
        assert!(report.contains("Critical") || report.contains("critical"));
        assert!(report.contains("openssl") || report.contains("1.1.1w"));
    }

    #[test]
    fn test_filter_by_package() {
        let scan = sample_scan();
        let openssl_cves = filter_by_package(&scan, "openssl");
        assert_eq!(openssl_cves.len(), 1);
        assert_eq!(openssl_cves[0].id, "CVE-2024-0001");
    }

    #[test]
    fn test_filter_by_package_case_insensitive() {
        let scan = sample_scan();
        let results = filter_by_package(&scan, "OpenSSL");
        assert_eq!(results.len(), 1);
    }
}
