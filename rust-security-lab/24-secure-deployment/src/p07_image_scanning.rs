//! # Lesson 07: Image Scanning
//!
//! ## Attack: Known CVE Exploitation
//!
//! Your container image uses a base image with OpenSSL 1.1.1, which has known CVEs.
//! An attacker scans the internet for vulnerable containers, finds yours, and
//! exploits the vulnerability to gain remote code execution inside your container.
//!
//! The vulnerability was known for 6 months. You never scanned your image.
//!
//! ## Defend: Automated Image Scanning
//!
//! Scan every image in CI before pushing to a registry:
//! - Use tools like Trivy, Grype, or Snyk
//! - Fail the build on critical/high CVEs
//! - Block deployment of unscanned images
//! - Continuously monitor running containers for new CVEs
//!
//! ## Audit: Scanning Policy
//!
//! - [ ] All images scanned before push to registry
//! - [ ] Critical CVEs block deployment
//! - [ ] High CVEs require explicit approval
//! - [ ] Base images updated at least monthly
//! - [ ] Running containers monitored for new CVEs

use serde::{Deserialize, Serialize};

/// Severity of a CVE.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Informational, no action needed
    Negligible,
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity, should fix
    High,
    /// Critical, must fix immediately
    Critical,
}

/// A known vulnerability (CVE).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Cve {
    /// CVE identifier (e.g., "CVE-2024-0001")
    pub id: String,
    /// Package name
    pub package: String,
    /// Installed version
    pub installed_version: String,
    /// Fixed version (empty if not yet fixed)
    pub fixed_version: String,
    /// Severity
    pub severity: Severity,
    /// Brief description
    pub description: String,
}

/// An image scan result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScanResult {
    /// Image name and tag
    pub image: String,
    /// List of CVEs found
    pub cves: Vec<Cve>,
    /// Scan timestamp (ISO 8601)
    pub scan_timestamp: String,
    /// Scanner tool name
    pub scanner: String,
}

/// Deployment policy: which severity levels block deployment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScanPolicy {
    /// If true, Critical CVEs block deployment
    pub block_on_critical: bool,
    /// If true, High CVEs block deployment
    pub block_on_high: bool,
    /// If true, Medium CVEs block deployment
    pub block_on_medium: bool,
    /// Maximum age of scan result in hours (0 = no limit)
    pub max_scan_age_hours: u32,
}

/// Exercise 1: Count CVEs by severity.
///
/// Return a tuple: (critical_count, high_count, medium_count, low_count, negligible_count)
pub fn count_by_severity(scan: &ScanResult) -> (usize, usize, usize, usize, usize) {
    todo!("Count CVEs grouped by severity")
}

/// Exercise 2: Check if an image passes the scan policy.
///
/// Return Ok(()) if the image can be deployed, Err(description) if blocked.
///
/// Checks:
/// 1. If `block_on_critical` and there are Critical CVEs, block
/// 2. If `block_on_high` and there are High CVEs, block
/// 3. If `block_on_medium` and there are Medium CVEs, block
/// 4. If `max_scan_age_hours > 0` and the scan is older than that, block
///
/// For age check, parse the scan_timestamp as ISO 8601. If parsing fails,
/// assume the scan is too old.
pub fn check_policy(scan: &ScanResult, policy: &ScanPolicy, current_timestamp: &str) -> Result<(), String> {
    todo!("Check if scan result passes deployment policy")
}

/// Exercise 3: Find fixable CVEs.
///
/// A CVE is fixable if `fixed_version` is not empty.
/// Return a list of fixable CVEs sorted by severity (Critical first).
pub fn find_fixable_cves(scan: &ScanResult) -> Vec<&Cve> {
    todo!("Find fixable CVEs sorted by severity")
}

/// Exercise 4: Generate a remediation report.
///
/// Return a multi-line string summarizing the scan results:
///
/// ```text
/// Image Scan Report: {image}
/// Scanner: {scanner}
/// Scanned: {timestamp}
///
/// Critical: {count}
/// High:     {count}
/// Medium:   {count}
/// Low:      {count}
///
/// Fixable CVEs: {fixable_count}
/// Top fix: {highest_severity_fixable.package} {installed_version} -> {fixed_version}
/// ```
///
/// If there are no fixable CVEs, show "Top fix: N/A" instead.
pub fn remediation_report(scan: &ScanResult) -> String {
    todo!("Generate a remediation report")
}

/// Exercise 5: Filter CVEs by package name.
///
/// Return all CVEs that affect the given package.
/// The comparison should be case-insensitive.
pub fn filter_by_package<'a>(scan: &'a ScanResult, package: &str) -> Vec<&'a Cve> {
    todo!("Filter CVEs by package name")
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
                    fixed_version: String::new(), // Not yet fixed
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
        assert!(result.unwrap_err().contains("Critical") || result.unwrap_err().contains("critical"));
    }

    #[test]
    fn test_check_policy_scan_too_old() {
        let mut scan = sample_scan();
        scan.cves.clear(); // No CVEs
        let policy = ScanPolicy {
            block_on_critical: false,
            block_on_high: false,
            block_on_medium: false,
            max_scan_age_hours: 24,
        };
        // Scan was at 10:00, current time is 48 hours later
        let result = check_policy(&scan, &policy, "2024-01-17T10:00:01Z");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("age") || result.unwrap_err().contains("old") || result.unwrap_err().contains("stale"));
    }

    #[test]
    fn test_find_fixable_cves() {
        let scan = sample_scan();
        let fixable = find_fixable_cves(&scan);
        assert_eq!(fixable.len(), 3); // openssl, curl, zlib are fixable
        assert_eq!(fixable[0].severity, Severity::Critical); // Sorted by severity
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
