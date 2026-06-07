// ===========================================================================
// Supply Chain Security
//
// Modern software is assembled from hundreds of dependencies. A compromised
// crate can introduce backdoors, exfiltrate secrets, or corrupt data.
// Supply chain security covers the full lifecycle: auditing known
// vulnerabilities, verifying provenance, generating Software Bills of
// Materials (SBOMs), and producing reproducible builds.
//
// Key Concepts:
//   - cargo audit: check dependencies against the RustSec Advisory Database
//   - Dependency verification: pin versions, review changes before upgrading
//   - SBOM (Software Bill of Materials): list every component and version
//   - Reproducible builds: deterministic output from identical sources
//   - Feature-gating to minimize attack surface
// ===========================================================================

use std::collections::BTreeMap;
use std::fmt;

// ---------------------------------------------------------------------------
// 1. Vulnerability Advisory Model
// ---------------------------------------------------------------------------

/// Severity levels for security advisories (mirrors CVSS ranges).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// A security advisory for a specific crate version.
#[derive(Debug, Clone)]
pub struct Advisory {
    pub id: String,
    pub crate_name: String,
    pub affected_versions: String,
    pub severity: Severity,
    pub description: String,
    pub patched_versions: Vec<String>,
}

impl Advisory {
    pub fn new(
        id: impl Into<String>,
        crate_name: impl Into<String>,
        affected_versions: impl Into<String>,
        severity: Severity,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            crate_name: crate_name.into(),
            affected_versions: affected_versions.into(),
            severity,
            description: description.into(),
            patched_versions: Vec::new(),
        }
    }

    pub fn with_patch(mut self, version: impl Into<String>) -> Self {
        self.patched_versions.push(version.into());
        self
    }
}

// ---------------------------------------------------------------------------
// 2. Audit Report (cargo audit equivalent)
// ---------------------------------------------------------------------------

/// Result of auditing a dependency graph against known advisories.
#[derive(Debug)]
pub struct AuditReport {
    pub advisories: Vec<AuditFinding>,
    pub dependencies_scanned: usize,
}

#[derive(Debug)]
pub struct AuditFinding {
    pub advisory: Advisory,
    pub installed_version: String,
}

impl AuditReport {
    pub fn is_clean(&self) -> bool {
        self.advisories.is_empty()
    }

    pub fn critical_count(&self) -> usize {
        self.advisories
            .iter()
            .filter(|f| f.advisory.severity >= Severity::Critical)
            .count()
    }

    pub fn high_or_above_count(&self) -> usize {
        self.advisories
            .iter()
            .filter(|f| f.advisory.severity >= Severity::High)
            .count()
    }
}

impl fmt::Display for AuditReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Audit: {} dependencies scanned, {} vulnerabilities found",
            self.dependencies_scanned,
            self.advisories.len()
        )?;
        for finding in &self.advisories {
            writeln!(
                f,
                "  [{}] {} {} (installed: {})",
                finding.advisory.severity,
                finding.advisory.id,
                finding.advisory.crate_name,
                finding.installed_version,
            )?;
        }
        Ok(())
    }
}

/// Audits a list of dependency name/version pairs against known advisories.
pub fn audit_dependencies(
    dependencies: &[(&str, &str)],
    advisory_db: &[Advisory],
) -> AuditReport {
    let mut findings = Vec::new();

    for (dep_name, dep_version) in dependencies {
        for advisory in advisory_db {
            if advisory.crate_name == *dep_name
                && version_matches(dep_version, &advisory.affected_versions)
            {
                findings.push(AuditFinding {
                    advisory: advisory.clone(),
                    installed_version: dep_version.to_string(),
                });
            }
        }
    }

    AuditReport {
        advisories: findings,
        dependencies_scanned: dependencies.len(),
    }
}

/// Simplified version range check. Supports exact match and "<X.Y.Z" prefix.
fn version_matches(installed: &str, range: &str) -> bool {
    if range.starts_with('<') {
        let threshold = &range[1..];
        compare_versions(installed, threshold) == std::cmp::Ordering::Less
    } else {
        installed == range
    }
}

/// Compare two semver-ish version strings segment by segment.
fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let a_parts: Vec<u32> = a.split('.').filter_map(|s| s.parse().ok()).collect();
    let b_parts: Vec<u32> = b.split('.').filter_map(|s| s.parse().ok()).collect();

    for (ai, bi) in a_parts.iter().zip(b_parts.iter()) {
        match ai.cmp(bi) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }
    a_parts.len().cmp(&b_parts.len())
}

// ---------------------------------------------------------------------------
// 3. Dependency Lock and Verification
// ---------------------------------------------------------------------------

/// A pinned dependency with its integrity hash (SHA-256).
#[derive(Debug, Clone)]
pub struct PinnedDependency {
    pub name: String,
    pub version: String,
    pub sha256: String,
    pub source: String,
}

/// Verifies that actual dependency metadata matches the expected lock.
pub fn verify_dependencies(
    expected: &[PinnedDependency],
    actual: &[PinnedDependency],
) -> Vec<DependencyMismatch> {
    let mut mismatches = Vec::new();

    let actual_map: BTreeMap<(&str, &str), &PinnedDependency> = actual
        .iter()
        .map(|d| ((d.name.as_str(), d.version.as_str()), d))
        .collect();

    for dep in expected {
        let key = (dep.name.as_str(), dep.version.as_str());
        match actual_map.get(&key) {
            None => {
                mismatches.push(DependencyMismatch::Missing(dep.name.clone()));
            }
            Some(found) => {
                if found.sha256 != dep.sha256 {
                    mismatches.push(DependencyMismatch::HashMismatch {
                        name: dep.name.clone(),
                        expected_hash: dep.sha256.clone(),
                        actual_hash: found.sha256.clone(),
                    });
                }
            }
        }
    }

    mismatches
}

#[derive(Debug, PartialEq)]
pub enum DependencyMismatch {
    Missing(String),
    HashMismatch {
        name: String,
        expected_hash: String,
        actual_hash: String,
    },
}

// ---------------------------------------------------------------------------
// 4. SBOM (Software Bill of Materials)
// ---------------------------------------------------------------------------

/// A lightweight SBOM entry.
#[derive(Debug, Clone)]
pub struct SbomEntry {
    pub name: String,
    pub version: String,
    pub license: String,
    pub supplier: String,
    pub hash: String,
}

/// Generates an SBOM from a list of pinned dependencies.
pub fn generate_sbom(deps: &[PinnedDependency]) -> Vec<SbomEntry> {
    deps.iter()
        .map(|d| SbomEntry {
            name: d.name.clone(),
            version: d.version.clone(),
            license: "UNKNOWN".into(),
            supplier: "crates.io".into(),
            hash: format!("sha256:{}", d.sha256),
        })
        .collect()
}

/// Serializes an SBOM to a JSON string (CycloneDX-lite format).
pub fn sbom_to_json(entries: &[SbomEntry]) -> String {
    let components: Vec<String> = entries
        .iter()
        .map(|e| {
            format!(
                r#"    {{"name":"{}","version":"{}","license":"{}","supplier":"{}","hash":"{}"}}"#,
                e.name, e.version, e.license, e.supplier, e.hash,
            )
        })
        .collect();

    format!(
        "{{\n  \"bomFormat\": \"CycloneDX\",\n  \"version\": \"1.5\",\n  \"components\": [\n{}\n  ]\n}}",
        components.join(",\n")
    )
}

// ---------------------------------------------------------------------------
// 5. Reproducible Build Fingerprint
// ---------------------------------------------------------------------------

/// Computes a deterministic fingerprint over source files and build metadata.
/// In production this would hash every source file, the toolchain version,
/// and all environment variables that affect codegen.
pub fn build_fingerprint(sources: &[(&str, &[u8])], toolchain: &str) -> [u8; 32] {
    use ring::digest;

    let mut ctx = digest::Context::new(&digest::SHA256);

    // Deterministic ordering: sources are assumed pre-sorted by path.
    for (path, contents) in sources {
        ctx.update(path.as_bytes());
        ctx.update(b"\0");
        ctx.update(contents);
        ctx.update(b"\0");
    }

    ctx.update(toolchain.as_bytes());

    let digest = ctx.finish();
    let mut out = [0u8; 32];
    out.copy_from_slice(digest.as_ref());
    out
}

/// Checks whether two fingerprints match (reproducible build verification).
pub fn fingerprints_match(a: &[u8; 32], b: &[u8; 32]) -> bool {
    constant_time_eq(a, b)
}

/// Constant-time comparison to prevent timing side channels.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_advisory_db() -> Vec<Advisory> {
        vec![
            Advisory::new("RUSTSEC-2024-0001", "serde_json", "<1.0.110", Severity::High, "Use-after-free in parser")
                .with_patch("1.0.110"),
            Advisory::new("RUSTSEC-2024-0002", "hyper", "<1.2.0", Severity::Critical, "HTTP request smuggling"),
            Advisory::new("RUSTSEC-2024-0003", "smallvec", "<1.12.0", Severity::Medium, "Buffer overflow"),
        ]
    }

    #[test]
    fn test_audit_finds_known_vulnerabilities() {
        let deps = vec![
            ("serde_json", "1.0.99"),
            ("hyper", "1.1.4"),
            ("tokio", "1.35.0"),
        ];
        let report = audit_dependencies(&deps, &sample_advisory_db());

        assert_eq!(report.dependencies_scanned, 3);
        assert_eq!(report.advisories.len(), 2); // serde_json + hyper
        assert_eq!(report.critical_count(), 1);
        assert!(!report.is_clean());
    }

    #[test]
    fn test_audit_clean_when_patched() {
        let deps = vec![
            ("serde_json", "1.0.110"),
            ("hyper", "1.2.0"),
            ("smallvec", "1.13.0"),
        ];
        let report = audit_dependencies(&deps, &sample_advisory_db());
        assert!(report.is_clean());
    }

    #[test]
    fn test_audit_report_display() {
        let deps = vec![("hyper", "1.1.4")];
        let report = audit_dependencies(&deps, &sample_advisory_db());
        let display = format!("{report}");
        assert!(display.contains("1 vulnerabilities"));
        assert!(display.contains("RUSTSEC-2024-0002"));
    }

    #[test]
    fn test_dependency_verification_pass() {
        let pinned = vec![PinnedDependency {
            name: "ring".into(),
            version: "0.17.8".into(),
            sha256: "abc123".into(),
            source: "crates.io".into(),
        }];
        let mismatches = verify_dependencies(&pinned, &pinned);
        assert!(mismatches.is_empty());
    }

    #[test]
    fn test_dependency_verification_detects_mismatch() {
        let expected = vec![PinnedDependency {
            name: "ring".into(),
            version: "0.17.8".into(),
            sha256: "abc123".into(),
            source: "crates.io".into(),
        }];
        let actual = vec![PinnedDependency {
            name: "ring".into(),
            version: "0.17.8".into(),
            sha256: "TAMPERED".into(),
            source: "crates.io".into(),
        }];
        let mismatches = verify_dependencies(&expected, &actual);
        assert_eq!(mismatches.len(), 1);
        assert!(matches!(
            mismatches[0],
            DependencyMismatch::HashMismatch { .. }
        ));
    }

    #[test]
    fn test_sbom_generation() {
        let deps = vec![PinnedDependency {
            name: "ring".into(),
            version: "0.17.8".into(),
            sha256: "abcdef".into(),
            source: "crates.io".into(),
        }];
        let sbom = generate_sbom(&deps);
        assert_eq!(sbom.len(), 1);
        assert_eq!(sbom[0].name, "ring");
        assert!(sbom[0].hash.starts_with("sha256:"));
    }

    #[test]
    fn test_sbom_json_roundtrip() {
        let deps = vec![PinnedDependency {
            name: "serde".into(),
            version: "1.0.200".into(),
            sha256: "deadbeef".into(),
            source: "crates.io".into(),
        }];
        let sbom = generate_sbom(&deps);
        let json = sbom_to_json(&sbom);
        assert!(json.contains("CycloneDX"));
        assert!(json.contains("serde"));
    }

    #[test]
    fn test_build_fingerprint_deterministic() {
        let sources = vec![
            ("src/main.rs", b"fn main() {}" as &[u8]),
            ("src/lib.rs", b"pub fn hello() {}" as &[u8]),
        ];
        let fp1 = build_fingerprint(&sources, "rustc 1.77.0");
        let fp2 = build_fingerprint(&sources, "rustc 1.77.0");
        assert_eq!(fp1, fp2);
        assert!(fingerprints_match(&fp1, &fp2));
    }

    #[test]
    fn test_build_fingerprint_differs_with_changes() {
        let sources_a = vec![("src/main.rs", b"v1" as &[u8])];
        let sources_b = vec![("src/main.rs", b"v2" as &[u8])];
        let fp_a = build_fingerprint(&sources_a, "rustc 1.77.0");
        let fp_b = build_fingerprint(&sources_b, "rustc 1.77.0");
        assert!(!fingerprints_match(&fp_a, &fp_b));
    }
}
