//! # Lesson 10: CI Security Gate
//!
//! ## The Problem
//!
//! Individual tools (Clippy, cargo-audit, Miri) each catch a class of bugs.
//! But if running them is optional, developers will skip them. The CI security
//! gate **blocks merges** when any security check fails.
//!
//! ```text
//! Pull Request
//!     |
//!     v
//! ┌─────────────────────────────┐
//! │     CI Security Gate        │
//! │                             │
//! │  [x] Clippy security lints  │──► PASS
//! │  [x] cargo-audit            │──► PASS
//! │  [x] cargo-deny             │──► PASS
//! │  [ ] Semgrep                │──► FAIL ──► BLOCK MERGE
//! │  [ ] Test coverage >= 80%   │──► (skipped)
//! └─────────────────────────────┘
//! ```
//!
//! ## Gate Configuration
//!
//! A CI security gate defines:
//! 1. Which checks to run
//! 2. Which are blocking vs. advisory
//! 3. Thresholds (e.g., max critical findings = 0)
//!
//! ## Exercise
//!
//! Implement a security gate that aggregates results from multiple checks
//! and makes a pass/fail decision.

/// Represents the result of a single security check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckResult {
    /// Check passed
    Pass,
    /// Check found issues (severity, count)
    Findings { critical: u32, high: u32, medium: u32, low: u32 },
    /// Check failed to run (tool error)
    Error(String),
}

/// Represents whether a check is blocking or advisory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckPolicy {
    /// Failure blocks the merge
    Blocking,
    /// Failure is reported but does not block
    Advisory,
}

/// A named security check with its policy and result.
#[derive(Debug, Clone)]
pub struct SecurityCheck {
    pub name: String,
    pub policy: CheckPolicy,
    pub result: CheckResult,
}

/// Exercise 1: Determine if a single check passes.
///
/// Requirements:
/// - A check passes if `result` is `CheckResult::Pass`
/// - A check passes if `result` is `CheckResult::Findings` and `critical == 0`
/// - A check fails if `result` is `CheckResult::Error`
/// - A check fails if `result` is `CheckResult::Findings` and `critical > 0`
pub fn check_passes(result: &CheckResult) -> bool {
    todo!("Determine if a security check passes")
}

/// Exercise 2: Determine if the overall gate passes.
///
/// Requirements:
/// - The gate passes ONLY if ALL blocking checks pass
/// - Advisory checks do not affect the gate decision
/// - If there are no blocking checks, the gate passes
pub fn gate_passes(checks: &[SecurityCheck]) -> bool {
    todo!("Determine if the CI security gate passes")
}

/// Exercise 3: Generate a gate report.
///
/// Requirements:
/// - Format: one line per check
/// - Format per line: "[PASS/FAIL] name (blocking/advisory)"
/// - Final line: "GATE: PASS" or "GATE: FAIL"
/// - Sort checks: blocking first, then advisory
pub fn generate_gate_report(checks: &[SecurityCheck]) -> String {
    todo!("Generate CI security gate report")
}

/// Exercise 4: Count findings by severity across all checks.
///
/// Requirements:
/// - Sum up critical, high, medium, low findings across all checks
/// - Return (critical, high, medium, low)
/// - Error results contribute 0 findings
pub fn count_total_findings(checks: &[SecurityCheck]) -> (u32, u32, u32, u32) {
    todo!("Count total findings across all checks")
}

/// Exercise 5: Filter checks that failed.
///
/// Requirements:
/// - Return references to all checks that do NOT pass (per `check_passes` logic)
/// - Include both blocking and advisory failures
pub fn failed_checks(checks: &[SecurityCheck]) -> Vec<&SecurityCheck> {
    todo!("Return all failed checks")
}

/// Exercise 6: Check if any critical findings exist.
///
/// Requirements:
/// - Return `true` if any check has critical findings > 0
/// - Return `false` otherwise
pub fn has_critical_findings(checks: &[SecurityCheck]) -> bool {
    todo!("Check for critical findings")
}

/// Exercise 7: Validate a CI configuration for completeness.
///
/// Requirements:
/// - A valid CI config must have at least `min_checks` checks defined
/// - A valid CI config must have at least one blocking check
/// - Return `Ok(())` if valid, `Err(String)` describing what is missing
pub fn validate_ci_config(checks: &[SecurityCheck], min_checks: usize) -> Result<(), String> {
    todo!("Validate CI security gate configuration")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pass(name: &str, policy: CheckPolicy) -> SecurityCheck {
        SecurityCheck {
            name: name.to_string(),
            policy,
            result: CheckResult::Pass,
        }
    }

    fn make_fail(name: &str, policy: CheckPolicy, critical: u32) -> SecurityCheck {
        SecurityCheck {
            name: name.to_string(),
            policy,
            result: CheckResult::Findings { critical, high: 0, medium: 0, low: 0 },
        }
    }

    fn make_error(name: &str, policy: CheckPolicy) -> SecurityCheck {
        SecurityCheck {
            name: name.to_string(),
            policy,
            result: CheckResult::Error("tool crashed".to_string()),
        }
    }

    #[test]
    fn test_check_passes_pass() {
        assert!(check_passes(&CheckResult::Pass));
    }

    #[test]
    fn test_check_passes_no_critical() {
        assert!(check_passes(&CheckResult::Findings {
            critical: 0, high: 2, medium: 5, low: 10,
        }));
    }

    #[test]
    fn test_check_passes_with_critical() {
        assert!(!check_passes(&CheckResult::Findings {
            critical: 1, high: 0, medium: 0, low: 0,
        }));
    }

    #[test]
    fn test_check_passes_error() {
        assert!(!check_passes(&CheckResult::Error("crash".to_string())));
    }

    #[test]
    fn test_gate_passes_all_pass() {
        let checks = vec![
            make_pass("clippy", CheckPolicy::Blocking),
            make_pass("audit", CheckPolicy::Blocking),
        ];
        assert!(gate_passes(&checks));
    }

    #[test]
    fn test_gate_fails_blocking() {
        let checks = vec![
            make_pass("clippy", CheckPolicy::Blocking),
            make_fail("audit", CheckPolicy::Blocking, 1),
        ];
        assert!(!gate_passes(&checks));
    }

    #[test]
    fn test_gate_passes_advisory_fail() {
        let checks = vec![
            make_pass("clippy", CheckPolicy::Blocking),
            make_fail("semgrep", CheckPolicy::Advisory, 1),
        ];
        assert!(gate_passes(&checks), "Advisory failures should not block");
    }

    #[test]
    fn test_gate_passes_no_blocking() {
        let checks = vec![
            make_fail("semgrep", CheckPolicy::Advisory, 1),
        ];
        assert!(gate_passes(&checks), "No blocking checks = pass");
    }

    #[test]
    fn test_count_total_findings() {
        let checks = vec![
            SecurityCheck {
                name: "a".into(),
                policy: CheckPolicy::Blocking,
                result: CheckResult::Findings { critical: 1, high: 2, medium: 3, low: 4 },
            },
            SecurityCheck {
                name: "b".into(),
                policy: CheckPolicy::Blocking,
                result: CheckResult::Findings { critical: 0, high: 1, medium: 0, low: 5 },
            },
        ];
        let (c, h, m, l) = count_total_findings(&checks);
        assert_eq!(c, 1);
        assert_eq!(h, 3);
        assert_eq!(m, 3);
        assert_eq!(l, 9);
    }

    #[test]
    fn test_count_total_findings_clean() {
        let checks = vec![make_pass("a", CheckPolicy::Blocking)];
        let (c, h, m, l) = count_total_findings(&checks);
        assert_eq!((c, h, m, l), (0, 0, 0, 0));
    }

    #[test]
    fn test_failed_checks() {
        let checks = vec![
            make_pass("clippy", CheckPolicy::Blocking),
            make_fail("audit", CheckPolicy::Blocking, 1),
            make_error("miri", CheckPolicy::Advisory),
        ];
        let failed = failed_checks(&checks);
        assert_eq!(failed.len(), 2);
        assert_eq!(failed[0].name, "audit");
        assert_eq!(failed[1].name, "miri");
    }

    #[test]
    fn test_has_critical_findings_true() {
        let checks = vec![make_fail("audit", CheckPolicy::Blocking, 1)];
        assert!(has_critical_findings(&checks));
    }

    #[test]
    fn test_has_critical_findings_false() {
        let checks = vec![make_pass("audit", CheckPolicy::Blocking)];
        assert!(!has_critical_findings(&checks));
    }

    #[test]
    fn test_validate_ci_config_valid() {
        let checks = vec![
            make_pass("clippy", CheckPolicy::Blocking),
            make_pass("audit", CheckPolicy::Advisory),
        ];
        assert!(validate_ci_config(&checks, 2).is_ok());
    }

    #[test]
    fn test_validate_ci_config_too_few() {
        let checks = vec![make_pass("clippy", CheckPolicy::Blocking)];
        assert!(validate_ci_config(&checks, 3).is_err());
    }

    #[test]
    fn test_validate_ci_config_no_blocking() {
        let checks = vec![
            make_pass("semgrep", CheckPolicy::Advisory),
            make_pass("coverage", CheckPolicy::Advisory),
        ];
        assert!(validate_ci_config(&checks, 2).is_err());
    }

    #[test]
    fn test_gate_report_format() {
        let checks = vec![
            make_pass("clippy", CheckPolicy::Blocking),
            make_fail("audit", CheckPolicy::Advisory, 1),
        ];
        let report = generate_gate_report(&checks);
        assert!(report.contains("[PASS]"));
        assert!(report.contains("[FAIL]"));
        assert!(report.contains("GATE:"));
    }
}
