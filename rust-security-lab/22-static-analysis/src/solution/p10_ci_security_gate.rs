//! # Lesson 10: CI Security Gate (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

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

/// Determine if a single check passes.
pub fn check_passes(result: &CheckResult) -> bool {
    match result {
        CheckResult::Pass => true,
        CheckResult::Findings { critical, .. } => *critical == 0,
        CheckResult::Error(_) => false,
    }
}

/// Determine if the overall CI security gate passes.
pub fn gate_passes(checks: &[SecurityCheck]) -> bool {
    checks
        .iter()
        .filter(|c| c.policy == CheckPolicy::Blocking)
        .all(|c| check_passes(&c.result))
}

/// Generate CI security gate report.
pub fn generate_gate_report(checks: &[SecurityCheck]) -> String {
    let mut report = String::new();

    // Sort: blocking first, then advisory
    let mut sorted: Vec<&SecurityCheck> = checks.iter().collect();
    sorted.sort_by(|a, b| {
        let ord = (a.policy as u8).cmp(&(b.policy as u8));
        ord.then(a.name.cmp(&b.name))
    });

    for check in &sorted {
        let status = if check_passes(&check.result) {
            "PASS"
        } else {
            "FAIL"
        };
        let policy = match check.policy {
            CheckPolicy::Blocking => "blocking",
            CheckPolicy::Advisory => "advisory",
        };
        report.push_str(&format!("[{}] {} ({})\n", status, check.name, policy));
    }

    let gate_status = if gate_passes(checks) { "PASS" } else { "FAIL" };
    report.push_str(&format!("GATE: {}", gate_status));

    report
}

/// Count total findings across all checks.
pub fn count_total_findings(checks: &[SecurityCheck]) -> (u32, u32, u32, u32) {
    let mut total = (0u32, 0u32, 0u32, 0u32);
    for check in checks {
        if let CheckResult::Findings { critical, high, medium, low } = &check.result {
            total.0 += critical;
            total.1 += high;
            total.2 += medium;
            total.3 += low;
        }
    }
    total
}

/// Return all failed checks.
pub fn failed_checks(checks: &[SecurityCheck]) -> Vec<&SecurityCheck> {
    checks.iter().filter(|c| !check_passes(&c.result)).collect()
}

/// Check for critical findings.
pub fn has_critical_findings(checks: &[SecurityCheck]) -> bool {
    checks.iter().any(|c| {
        matches!(&c.result, CheckResult::Findings { critical, .. } if *critical > 0)
    })
}

/// Validate CI security gate configuration.
pub fn validate_ci_config(checks: &[SecurityCheck], min_checks: usize) -> Result<(), String> {
    if checks.len() < min_checks {
        return Err(format!(
            "Only {} checks defined, minimum is {}",
            checks.len(),
            min_checks
        ));
    }

    let has_blocking = checks.iter().any(|c| c.policy == CheckPolicy::Blocking);
    if !has_blocking {
        return Err("No blocking checks defined".to_string());
    }

    Ok(())
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
