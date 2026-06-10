//! # Lesson 09: Taint Analysis (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

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

/// Mark data as tainted.
pub fn mark_tainted() -> TaintStatus {
    TaintStatus::Tainted
}

/// Mark data as clean.
pub fn mark_clean() -> TaintStatus {
    TaintStatus::Clean
}

/// Propagate taint: tainted + anything = tainted.
pub fn propagate_taint(a: TaintStatus, b: TaintStatus) -> TaintStatus {
    match (a, b) {
        (TaintStatus::Clean, TaintStatus::Clean) => TaintStatus::Clean,
        _ => TaintStatus::Tainted,
    }
}

/// Sanitize data if validation passes.
pub fn sanitize(input: TaintStatus, is_valid: bool) -> TaintStatus {
    match input {
        TaintStatus::Clean => TaintStatus::Clean,
        TaintStatus::Tainted => {
            if is_valid {
                TaintStatus::Clean
            } else {
                TaintStatus::Tainted
            }
        }
    }
}

/// Check if data is safe for the given sink.
pub fn check_sink(status: TaintStatus, sink: SinkType) -> Result<(), String> {
    match status {
        TaintStatus::Clean => Ok(()),
        TaintStatus::Tainted => {
            let sink_name = match sink {
                SinkType::SqlQuery => "SQL query",
                SinkType::HtmlOutput => "HTML output",
                SinkType::ShellCommand => "shell command",
                SinkType::FilePath => "file path",
            };
            Err(format!("Tainted data used in {}", sink_name))
        }
    }
}

/// Analyze data flow from source to sink.
pub fn analyze_flow(
    source_status: TaintStatus,
    sanitized: bool,
    sink: SinkType,
) -> Result<(), String> {
    let status = sanitize(source_status, sanitized);
    check_sink(status, sink)
}

/// Find all data flows where tainted data reaches a sink.
pub fn find_tainted_sinks(flows: &[(&str, TaintStatus, SinkType)]) -> Vec<String> {
    flows
        .iter()
        .filter(|(_, status, sink)| check_sink(*status, *sink).is_err())
        .map(|(label, _, _)| label.to_string())
        .collect()
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
