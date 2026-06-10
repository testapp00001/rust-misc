//! # Lesson 09: Taint Analysis
//!
//! ## The Problem
//!
//! **Taint analysis** tracks untrusted data ("tainted") as it flows through a
//! program. Data from external sources (HTTP requests, user input, file reads)
//! is "tainted" until it is validated ("sanitized"). Using tainted data in
//! security-sensitive operations (SQL queries, shell commands, HTML output)
//! is a vulnerability.
//!
//! ```text
//! Tainted Source         Sanitization           Safe Sink
//! ─────────────         ─────────────           ─────────
//! HTTP body             validate_length()       SQL query
//! User input            escape_html()           HTML output
//! File contents         parse_and_check()       Shell command
//! ```
//!
//! ## How Taint Analysis Works
//!
//! 1. **Sources**: Mark where untrusted data enters the program
//! 2. **Propagation**: Track how data flows through assignments, function calls
//! 3. **Sinks**: Mark where data is used in security-sensitive operations
//! 4. **Sanitizers**: Mark where data is validated/escaped
//!
//! If tainted data reaches a sink without passing through a sanitizer, that is
//! a finding.
//!
//! ## Exercise
//!
//! Implement a taint tracking system that models data flow through a program.

/// Represents the taint status of a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaintStatus {
    /// Data from a trusted source or already sanitized
    Clean,
    /// Data from an untrusted source
    Tainted,
}

/// Represents the type of security-sensitive operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SinkType {
    /// SQL query execution
    SqlQuery,
    /// HTML output rendering
    HtmlOutput,
    /// Shell command execution
    ShellCommand,
    /// File path construction
    FilePath,
}

/// Exercise 1: Create a taint source.
///
/// Requirements:
/// - Return `TaintStatus::Tainted` for any input from an untrusted source
/// - This represents marking data as coming from user input
pub fn mark_tainted() -> TaintStatus {
    todo!("Return Tainted status")
}

/// Exercise 2: Create a clean source.
///
/// Requirements:
/// - Return `TaintStatus::Clean` for trusted/validated data
pub fn mark_clean() -> TaintStatus {
    todo!("Return Clean status")
}

/// Exercise 3: Propagate taint through an operation.
///
/// When you combine tainted and clean data, the result is tainted.
///
/// Requirements:
/// - If either `a` or `b` is `Tainted`, return `Tainted`
/// - Only return `Clean` if BOTH inputs are `Clean`
pub fn propagate_taint(a: TaintStatus, b: TaintStatus) -> TaintStatus {
    todo!("Propagate taint: tainted + anything = tainted")
}

/// Exercise 4: Sanitize tainted data.
///
/// Requirements:
/// - If input is `Tainted` AND `is_valid` is `true`, return `Clean`
/// - If input is `Tainted` AND `is_valid` is `false`, return `Tainted`
/// - If input is `Clean`, return `Clean` regardless of `is_valid`
pub fn sanitize(input: TaintStatus, is_valid: bool) -> TaintStatus {
    todo!("Sanitize data if validation passes")
}

/// Exercise 5: Check if data is safe to use in a security-sensitive sink.
///
/// Requirements:
/// - Return `Ok(())` if `status` is `Clean`
/// - Return `Err(String)` with a message like "Tainted data used in SQL query"
///   if `status` is `Tainted`, including the sink type in the message
pub fn check_sink(status: TaintStatus, sink: SinkType) -> Result<(), String> {
    todo!("Check if data is safe for the given sink")
}

/// Exercise 6: Track taint through a simulated data flow.
///
/// Requirements:
/// - Simulate a data flow pipeline: source -> operations -> sink
/// - `source_status`: taint status of the initial data
/// - `sanitized`: whether the data was sanitized
/// - `sink`: the sink type
/// - Return `Ok(())` if the flow is safe, `Err(String)` if tainted data reaches the sink
pub fn analyze_flow(
    source_status: TaintStatus,
    sanitized: bool,
    sink: SinkType,
) -> Result<(), String> {
    todo!("Analyze data flow from source to sink")
}

/// Exercise 7: Find all tainted sinks in a list of data flows.
///
/// Requirements:
/// - Given a list of `(label, taint_status, sink_type)` tuples
/// - Return the labels of all flows where tainted data reaches a sink
/// - Flows with clean data are ignored
pub fn find_tainted_sinks(flows: &[(&str, TaintStatus, SinkType)]) -> Vec<String> {
    todo!("Find all data flows where tainted data reaches a sink")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mark_tainted() {
        assert_eq!(mark_tainted(), TaintStatus::Tainted);
    }

    #[test]
    fn test_mark_clean() {
        assert_eq!(mark_clean(), TaintStatus::Clean);
    }

    #[test]
    fn test_propagate_taint_both_clean() {
        assert_eq!(propagate_taint(TaintStatus::Clean, TaintStatus::Clean), TaintStatus::Clean);
    }

    #[test]
    fn test_propagate_taint_one_tainted() {
        assert_eq!(propagate_taint(TaintStatus::Tainted, TaintStatus::Clean), TaintStatus::Tainted);
        assert_eq!(propagate_taint(TaintStatus::Clean, TaintStatus::Tainted), TaintStatus::Tainted);
    }

    #[test]
    fn test_propagate_taint_both_tainted() {
        assert_eq!(propagate_taint(TaintStatus::Tainted, TaintStatus::Tainted), TaintStatus::Tainted);
    }

    #[test]
    fn test_sanitize_tainted_valid() {
        assert_eq!(sanitize(TaintStatus::Tainted, true), TaintStatus::Clean);
    }

    #[test]
    fn test_sanitize_tainted_invalid() {
        assert_eq!(sanitize(TaintStatus::Tainted, false), TaintStatus::Tainted);
    }

    #[test]
    fn test_sanitize_clean() {
        assert_eq!(sanitize(TaintStatus::Clean, false), TaintStatus::Clean);
    }

    #[test]
    fn test_check_sink_clean() {
        assert!(check_sink(TaintStatus::Clean, SinkType::SqlQuery).is_ok());
    }

    #[test]
    fn test_check_sink_tainted_sql() {
        let result = check_sink(TaintStatus::Tainted, SinkType::SqlQuery);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("SQL"));
    }

    #[test]
    fn test_check_sink_tainted_html() {
        let result = check_sink(TaintStatus::Tainted, SinkType::HtmlOutput);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("HTML"));
    }

    #[test]
    fn test_analyze_flow_safe() {
        assert!(analyze_flow(TaintStatus::Tainted, true, SinkType::SqlQuery).is_ok());
    }

    #[test]
    fn test_analyze_flow_unsafe() {
        assert!(analyze_flow(TaintStatus::Tainted, false, SinkType::SqlQuery).is_err());
    }

    #[test]
    fn test_analyze_flow_clean() {
        assert!(analyze_flow(TaintStatus::Clean, false, SinkType::ShellCommand).is_ok());
    }

    #[test]
    fn test_find_tainted_sinks() {
        let flows = vec![
            ("login", TaintStatus::Tainted, SinkType::SqlQuery),
            ("config", TaintStatus::Clean, SinkType::SqlQuery),
            ("user_bio", TaintStatus::Tainted, SinkType::HtmlOutput),
        ];
        let tainted = find_tainted_sinks(&flows);
        assert_eq!(tainted, vec!["login", "user_bio"]);
    }

    #[test]
    fn test_find_tainted_sinks_clean() {
        let flows = vec![
            ("config", TaintStatus::Clean, SinkType::SqlQuery),
        ];
        assert!(find_tainted_sinks(&flows).is_empty());
    }
}
