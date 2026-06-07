//! # Lesson 6: Debug Techniques with Tracing
//!
//! Tracing is invaluable for debugging. This lesson covers conditional
//! tracing, trace-level debugging, inspection techniques, and debugging
//! patterns that go beyond simple println.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Conditional tracing
// ---------------------------------------------------------------------------

/// A debug flag system for enabling/disabling trace categories.
#[derive(Debug, Clone)]
pub struct DebugFlags {
    flags: BTreeMap<String, bool>,
}

impl DebugFlags {
    pub fn new() -> Self {
        Self {
            flags: BTreeMap::new(),
        }
    }

    pub fn enable(&mut self, category: &str) {
        self.flags.insert(category.to_string(), true);
    }

    pub fn disable(&mut self, category: &str) {
        self.flags.insert(category.to_string(), false);
    }

    pub fn is_enabled(&self, category: &str) -> bool {
        self.flags.get(category).copied().unwrap_or(false)
    }

    pub fn enabled_categories(&self) -> Vec<&str> {
        self.flags
            .iter()
            .filter(|(_, v)| **v)
            .map(|(k, _)| k.as_str())
            .collect()
    }
}

impl Default for DebugFlags {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Debug inspection
// ---------------------------------------------------------------------------

/// A debug inspector that captures state snapshots.
#[derive(Debug)]
pub struct StateInspector {
    snapshots: Vec<StateSnapshot>,
}

#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub name: String,
    pub timestamp_relative_ms: u64,
    pub state: BTreeMap<String, String>,
}

impl StateInspector {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
        }
    }

    /// Take a named snapshot of the current state.
    pub fn snapshot(&mut self, name: &str, state: BTreeMap<String, String>) {
        self.snapshots.push(StateSnapshot {
            name: name.to_string(),
            timestamp_relative_ms: self.snapshots.len() as u64 * 100, // simulated
            state,
        });
    }

    /// Get all snapshots.
    pub fn snapshots(&self) -> &[StateSnapshot] {
        &self.snapshots
    }

    /// Compare two snapshots and find differences.
    pub fn diff(&self, idx_a: usize, idx_b: usize) -> Vec<StateDiff> {
        if idx_a >= self.snapshots.len() || idx_b >= self.snapshots.len() {
            return vec![];
        }

        let a = &self.snapshots[idx_a];
        let b = &self.snapshots[idx_b];
        let mut diffs = Vec::new();

        for (key, val_a) in &a.state {
            match b.state.get(key) {
                Some(val_b) if val_a != val_b => {
                    diffs.push(StateDiff::Changed {
                        key: key.clone(),
                        old: val_a.clone(),
                        new: val_b.clone(),
                    });
                }
                None => {
                    diffs.push(StateDiff::Removed {
                        key: key.clone(),
                        value: val_a.clone(),
                    });
                }
                _ => {}
            }
        }

        for (key, val_b) in &b.state {
            if !a.state.contains_key(key) {
                diffs.push(StateDiff::Added {
                    key: key.clone(),
                    value: val_b.clone(),
                });
            }
        }

        diffs
    }
}

impl Default for StateInspector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq)]
pub enum StateDiff {
    Added { key: String, value: String },
    Removed { key: String, value: String },
    Changed { key: String, old: String, new: String },
}

// ---------------------------------------------------------------------------
// Performance tracing
// ---------------------------------------------------------------------------

/// A simple timer for measuring operation durations.
pub struct PerfTimer {
    name: String,
    start: Instant,
}

impl PerfTimer {
    pub fn start(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
        }
    }

    /// Stop the timer and return the elapsed duration.
    pub fn stop(self) -> TimedOperation {
        TimedOperation {
            name: self.name,
            duration: self.start.elapsed(),
        }
    }
}

#[derive(Debug)]
pub struct TimedOperation {
    pub name: String,
    pub duration: Duration,
}

impl TimedOperation {
    pub fn duration_ms(&self) -> f64 {
        self.duration.as_secs_f64() * 1000.0
    }
}

/// A performance trace that records multiple timed operations.
pub struct PerfTrace {
    operations: Vec<TimedOperation>,
}

impl PerfTrace {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    pub fn time<F, T>(&mut self, name: &str, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        let timer = PerfTimer::start(name);
        let result = f();
        self.operations.push(timer.stop());
        result
    }

    pub fn operations(&self) -> &[TimedOperation] {
        &self.operations
    }

    pub fn total_duration(&self) -> Duration {
        self.operations.iter().map(|op| op.duration).sum()
    }

    pub fn summary(&self) -> Vec<(&str, f64)> {
        self.operations
            .iter()
            .map(|op| (op.name.as_str(), op.duration_ms()))
            .collect()
    }
}

impl Default for PerfTrace {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Debug output formatting
// ---------------------------------------------------------------------------

/// Format a debug tree for nested data structures.
pub struct DebugTree {
    lines: Vec<(usize, String)>, // (indent, text)
}

impl DebugTree {
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    pub fn add(&mut self, indent: usize, text: impl Into<String>) {
        self.lines.push((indent, text.into()));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        for (indent, text) in &self.lines {
            let prefix = "  ".repeat(*indent);
            output.push_str(&format!("{}{}\n", prefix, text));
        }
        output
    }
}

impl Default for DebugTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_flags() {
        let mut flags = DebugFlags::new();
        assert!(!flags.is_enabled("database"));

        flags.enable("database");
        assert!(flags.is_enabled("database"));
        assert!(!flags.is_enabled("network"));

        flags.disable("database");
        assert!(!flags.is_enabled("database"));
    }

    #[test]
    fn test_debug_flags_enabled_categories() {
        let mut flags = DebugFlags::new();
        flags.enable("database");
        flags.enable("cache");
        flags.disable("network");

        let enabled = flags.enabled_categories();
        assert_eq!(enabled.len(), 2);
        assert!(enabled.contains(&"database"));
        assert!(enabled.contains(&"cache"));
    }

    #[test]
    fn test_state_inspector_snapshot() {
        let mut inspector = StateInspector::new();
        let mut state = BTreeMap::new();
        state.insert("count".to_string(), "0".to_string());
        state.insert("status".to_string(), "idle".to_string());

        inspector.snapshot("initial", state);
        assert_eq!(inspector.snapshots().len(), 1);
    }

    #[test]
    fn test_state_inspector_diff() {
        let mut inspector = StateInspector::new();

        let mut state1 = BTreeMap::new();
        state1.insert("count".to_string(), "0".to_string());
        state1.insert("status".to_string(), "idle".to_string());
        state1.insert("old_field".to_string(), "removed".to_string());
        inspector.snapshot("before", state1);

        let mut state2 = BTreeMap::new();
        state2.insert("count".to_string(), "5".to_string());
        state2.insert("status".to_string(), "active".to_string());
        state2.insert("new_field".to_string(), "added".to_string());
        inspector.snapshot("after", state2);

        let diffs = inspector.diff(0, 1);
        assert!(diffs.contains(&StateDiff::Changed {
            key: "count".to_string(),
            old: "0".to_string(),
            new: "5".to_string(),
        }));
        assert!(diffs.contains(&StateDiff::Changed {
            key: "status".to_string(),
            old: "idle".to_string(),
            new: "active".to_string(),
        }));
        assert!(diffs.contains(&StateDiff::Removed {
            key: "old_field".to_string(),
            value: "removed".to_string(),
        }));
        assert!(diffs.contains(&StateDiff::Added {
            key: "new_field".to_string(),
            value: "added".to_string(),
        }));
    }

    #[test]
    fn test_state_inspector_diff_no_changes() {
        let mut inspector = StateInspector::new();
        let state = BTreeMap::new();
        inspector.snapshot("a", state.clone());
        inspector.snapshot("b", state);

        let diffs = inspector.diff(0, 1);
        assert!(diffs.is_empty());
    }

    #[test]
    fn test_state_inspector_diff_out_of_bounds() {
        let inspector = StateInspector::new();
        let diffs = inspector.diff(0, 1);
        assert!(diffs.is_empty());
    }

    #[test]
    fn test_perf_timer() {
        let timer = PerfTimer::start("test");
        std::thread::sleep(Duration::from_millis(1));
        let op = timer.stop();
        assert!(op.duration_ms() >= 0.0);
        assert_eq!(op.name, "test");
    }

    #[test]
    fn test_perf_trace() {
        let mut trace = PerfTrace::new();

        trace.time("step1", || {
            std::thread::sleep(Duration::from_millis(1));
        });
        trace.time("step2", || {
            std::thread::sleep(Duration::from_millis(1));
        });

        assert_eq!(trace.operations().len(), 2);
        let summary = trace.summary();
        assert_eq!(summary.len(), 2);
        assert_eq!(summary[0].0, "step1");
        assert_eq!(summary[1].0, "step2");
    }

    #[test]
    fn test_perf_trace_total() {
        let mut trace = PerfTrace::new();
        trace.time("a", || std::thread::sleep(Duration::from_millis(1)));
        trace.time("b", || std::thread::sleep(Duration::from_millis(1)));

        let total = trace.total_duration();
        assert!(total >= Duration::from_millis(1));
    }

    #[test]
    fn test_perf_trace_return_value() {
        let mut trace = PerfTrace::new();
        let result = trace.time("compute", || 42);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_debug_tree() {
        let mut tree = DebugTree::new();
        tree.add(0, "root");
        tree.add(1, "child1");
        tree.add(2, "grandchild");
        tree.add(1, "child2");

        let rendered = tree.render();
        assert!(rendered.contains("root"));
        assert!(rendered.contains("  child1"));
        assert!(rendered.contains("    grandchild"));
        assert!(rendered.contains("  child2"));
    }

    #[test]
    fn test_debug_tree_empty() {
        let tree = DebugTree::new();
        assert_eq!(tree.render(), "");
    }

    #[test]
    fn test_debug_flags_default() {
        let flags = DebugFlags::default();
        assert!(flags.enabled_categories().is_empty());
    }

    #[test]
    fn test_timed_operation_duration() {
        let op = TimedOperation {
            name: "test".to_string(),
            duration: Duration::from_millis(1500),
        };
        assert!((op.duration_ms() - 1500.0).abs() < 0.1);
    }
}
