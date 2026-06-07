//! # Lesson 8: Snapshot Testing
//!
//! Snapshot testing compares output against a stored reference.
//! This lesson covers snapshot concepts, inline snapshots, redaction,
//! and review workflows.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Snapshot testing framework concepts
// ---------------------------------------------------------------------------

/// A snapshot stored for comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub name: String,
    pub content: String,
    pub metadata: SnapshotMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub file: String,
    pub line: u32,
    pub version: u32,
    pub created_at: String,
}

impl Snapshot {
    pub fn new(name: &str, content: &str, file: &str, line: u32) -> Self {
        Self {
            name: name.to_string(),
            content: content.to_string(),
            metadata: SnapshotMetadata {
                file: file.to_string(),
                line,
                version: 1,
                created_at: chrono::Utc::now().to_rfc3339(),
            },
        }
    }
}

/// The result of comparing actual output to a snapshot.
#[derive(Debug, PartialEq)]
pub enum SnapshotResult {
    /// Output matches the snapshot.
    Matched,
    /// Output differs from the snapshot.
    Diff {
        expected: String,
        actual: String,
        diff: String,
    },
    /// No snapshot exists; this is a new snapshot.
    NewSnapshot { actual: String },
}

/// A simple snapshot store (in-memory, for teaching).
pub struct SnapshotStore {
    snapshots: BTreeMap<String, Snapshot>,
    update_mode: bool,
}

impl SnapshotStore {
    pub fn new() -> Self {
        Self {
            snapshots: BTreeMap::new(),
            update_mode: false,
        }
    }

    pub fn set_update_mode(&mut self, update: bool) {
        self.update_mode = update;
    }

    /// Assert that the actual output matches the stored snapshot.
    pub fn assert_snapshot(&mut self, name: &str, actual: &str) -> SnapshotResult {
        if let Some(snapshot) = self.snapshots.get(name) {
            if snapshot.content == actual {
                SnapshotResult::Matched
            } else {
                if self.update_mode {
                    // Update the snapshot
                    self.snapshots.insert(
                        name.to_string(),
                        Snapshot::new(name, actual, "test.rs", 0),
                    );
                    return SnapshotResult::Matched;
                }
                SnapshotResult::Diff {
                    expected: snapshot.content.clone(),
                    actual: actual.to_string(),
                    diff: compute_diff(&snapshot.content, actual),
                }
            }
        } else {
            // New snapshot
            self.snapshots.insert(
                name.to_string(),
                Snapshot::new(name, actual, "test.rs", 0),
            );
            SnapshotResult::NewSnapshot {
                actual: actual.to_string(),
            }
        }
    }

    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    pub fn get_snapshot(&self, name: &str) -> Option<&Snapshot> {
        self.snapshots.get(name)
    }
}

impl Default for SnapshotStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute a simple diff between two strings.
pub fn compute_diff(expected: &str, actual: &str) -> String {
    let mut diff = String::new();
    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();

    let max_len = expected_lines.len().max(actual_lines.len());
    for i in 0..max_len {
        match (expected_lines.get(i), actual_lines.get(i)) {
            (Some(e), Some(a)) if e == a => {
                diff.push_str(&format!("  {}\n", e));
            }
            (Some(e), Some(a)) => {
                diff.push_str(&format!("- {}\n", e));
                diff.push_str(&format!("+ {}\n", a));
            }
            (Some(e), None) => {
                diff.push_str(&format!("- {}\n", e));
            }
            (None, Some(a)) => {
                diff.push_str(&format!("+ {}\n", a));
            }
            (None, None) => {}
        }
    }
    diff
}

// ---------------------------------------------------------------------------
// Inline snapshot macro simulation
// ---------------------------------------------------------------------------

/// Simulates the inline_snapshot! macro behavior.
/// In real code, you'd use insta::assert_snapshot! or similar.
macro_rules! inline_snapshot {
    ($actual:expr, $expected:expr) => {
        let actual = $actual;
        let expected = $expected;
        assert_eq!(
            actual, expected,
            "\nInline snapshot mismatch:\n  actual:   {:?}\n  expected: {:?}",
            actual, expected
        );
    };
}

// ---------------------------------------------------------------------------
// Redaction: replacing dynamic values in snapshots
// ---------------------------------------------------------------------------

/// A redactor that replaces dynamic values with placeholders.
pub struct Redactor {
    patterns: Vec<(String, String)>,
}

impl Redactor {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    /// Add a redaction pattern: replace `pattern` with `replacement`.
    pub fn redact(mut self, pattern: &str, replacement: &str) -> Self {
        self.patterns
            .push((pattern.to_string(), replacement.to_string()));
        self
    }

    /// Apply all redactions to a string.
    pub fn apply(&self, input: &str) -> String {
        let mut result = input.to_string();
        for (pattern, replacement) in &self.patterns {
            result = result.replace(pattern, replacement);
        }
        result
    }
}

impl Default for Redactor {
    fn default() -> Self {
        Self::new()
    }
}

/// Redact common dynamic values from output.
pub fn redact_common(input: &str) -> String {
    let redactor = Redactor::new()
        .redact("0x[0-9a-f]+", "[HEX_ADDR]")
        .redact("[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}", "[TIMESTAMP]");

    redactor.apply(input)
}

// ---------------------------------------------------------------------------
// Snapshot review workflow
// ---------------------------------------------------------------------------

/// Represents a pending snapshot review.
#[derive(Debug, Clone)]
pub struct PendingReview {
    pub snapshot_name: String,
    pub old_content: String,
    pub new_content: String,
    pub status: ReviewStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReviewStatus {
    Pending,
    Accepted,
    Rejected,
}

impl PendingReview {
    pub fn accept(&mut self) {
        self.status = ReviewStatus::Accepted;
    }

    pub fn reject(&mut self) {
        self.status = ReviewStatus::Rejected;
    }
}

/// A collection of pending snapshot reviews.
pub struct ReviewQueue {
    reviews: Vec<PendingReview>,
}

impl ReviewQueue {
    pub fn new() -> Self {
        Self {
            reviews: Vec::new(),
        }
    }

    pub fn add(&mut self, review: PendingReview) {
        self.reviews.push(review);
    }

    pub fn pending_count(&self) -> usize {
        self.reviews
            .iter()
            .filter(|r| r.status == ReviewStatus::Pending)
            .count()
    }

    pub fn accept_all(&mut self) {
        for review in &mut self.reviews {
            if review.status == ReviewStatus::Pending {
                review.accept();
            }
        }
    }

    pub fn reviews(&self) -> &[PendingReview] {
        &self.reviews
    }
}

impl Default for ReviewQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_store_new_snapshot() {
        let mut store = SnapshotStore::new();
        let result = store.assert_snapshot("test", "hello world");
        assert!(matches!(result, SnapshotResult::NewSnapshot { .. }));
        assert_eq!(store.snapshot_count(), 1);
    }

    #[test]
    fn test_snapshot_store_match() {
        let mut store = SnapshotStore::new();
        store.assert_snapshot("test", "hello world");
        let result = store.assert_snapshot("test", "hello world");
        assert_eq!(result, SnapshotResult::Matched);
    }

    #[test]
    fn test_snapshot_store_diff() {
        let mut store = SnapshotStore::new();
        store.assert_snapshot("test", "hello");
        let result = store.assert_snapshot("test", "world");
        assert!(matches!(result, SnapshotResult::Diff { .. }));
    }

    #[test]
    fn test_snapshot_store_update_mode() {
        let mut store = SnapshotStore::new();
        store.assert_snapshot("test", "old");
        store.set_update_mode(true);
        let result = store.assert_snapshot("test", "new");
        assert_eq!(result, SnapshotResult::Matched);

        // Verify the snapshot was updated
        let snapshot = store.get_snapshot("test").unwrap();
        assert_eq!(snapshot.content, "new");
    }

    #[test]
    fn test_compute_diff_identical() {
        let diff = compute_diff("hello", "hello");
        assert!(diff.contains("  hello"));
    }

    #[test]
    fn test_compute_diff_different() {
        let diff = compute_diff("hello", "world");
        assert!(diff.contains("- hello"));
        assert!(diff.contains("+ world"));
    }

    #[test]
    fn test_compute_diff_multiline() {
        let expected = "line 1\nline 2\nline 3";
        let actual = "line 1\nmodified\nline 3";
        let diff = compute_diff(expected, actual);
        assert!(diff.contains("  line 1"));
        assert!(diff.contains("- line 2"));
        assert!(diff.contains("+ modified"));
    }

    #[test]
    fn test_inline_snapshot_macro() {
        inline_snapshot!("hello", "hello");
    }

    #[test]
    fn test_redactor() {
        let redactor = Redactor::new()
            .redact("Alice", "[NAME]")
            .redact("42", "[ID]");

        let result = redactor.apply("User Alice has ID 42");
        assert_eq!(result, "User [NAME] has ID [ID]");
    }

    #[test]
    fn test_redactor_no_match() {
        let redactor = Redactor::new().redact("xyz", "[REDACTED]");
        let result = redactor.apply("hello world");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_snapshot_metadata() {
        let snapshot = Snapshot::new("test", "content", "test.rs", 42);
        assert_eq!(snapshot.name, "test");
        assert_eq!(snapshot.metadata.file, "test.rs");
        assert_eq!(snapshot.metadata.line, 42);
        assert_eq!(snapshot.metadata.version, 1);
    }

    #[test]
    fn test_pending_review() {
        let mut review = PendingReview {
            snapshot_name: "test".into(),
            old_content: "old".into(),
            new_content: "new".into(),
            status: ReviewStatus::Pending,
        };
        assert_eq!(review.status, ReviewStatus::Pending);

        review.accept();
        assert_eq!(review.status, ReviewStatus::Accepted);
    }

    #[test]
    fn test_pending_review_reject() {
        let mut review = PendingReview {
            snapshot_name: "test".into(),
            old_content: "old".into(),
            new_content: "new".into(),
            status: ReviewStatus::Pending,
        };
        review.reject();
        assert_eq!(review.status, ReviewStatus::Rejected);
    }

    #[test]
    fn test_review_queue() {
        let mut queue = ReviewQueue::new();
        queue.add(PendingReview {
            snapshot_name: "a".into(),
            old_content: "old".into(),
            new_content: "new".into(),
            status: ReviewStatus::Pending,
        });
        queue.add(PendingReview {
            snapshot_name: "b".into(),
            old_content: "old".into(),
            new_content: "new".into(),
            status: ReviewStatus::Pending,
        });

        assert_eq!(queue.pending_count(), 2);
        queue.accept_all();
        assert_eq!(queue.pending_count(), 0);
        assert!(queue
            .reviews()
            .iter()
            .all(|r| r.status == ReviewStatus::Accepted));
    }

    #[test]
    fn test_snapshot_result_diff_fields() {
        let mut store = SnapshotStore::new();
        store.assert_snapshot("test", "expected");
        let result = store.assert_snapshot("test", "actual");

        match result {
            SnapshotResult::Diff {
                expected,
                actual,
                diff,
            } => {
                assert_eq!(expected, "expected");
                assert_eq!(actual, "actual");
                assert!(!diff.is_empty());
            }
            _ => panic!("expected Diff"),
        }
    }

    #[test]
    fn test_snapshot_multiple() {
        let mut store = SnapshotStore::new();
        store.assert_snapshot("first", "content 1");
        store.assert_snapshot("second", "content 2");
        store.assert_snapshot("third", "content 3");
        assert_eq!(store.snapshot_count(), 3);
    }

    #[test]
    fn test_snapshot_get() {
        let mut store = SnapshotStore::new();
        store.assert_snapshot("test", "content");
        let snapshot = store.get_snapshot("test").unwrap();
        assert_eq!(snapshot.content, "content");
        assert_eq!(snapshot.name, "test");
    }

    #[test]
    fn test_snapshot_get_missing() {
        let store = SnapshotStore::new();
        assert!(store.get_snapshot("missing").is_none());
    }
}
