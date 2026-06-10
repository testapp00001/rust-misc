//! # Lesson 06: Coverage-Guided Fuzzing Concepts
//!
//! ## What is Coverage-Guided Fuzzing?
//!
//! Coverage-guided fuzzers instrument the target code to track which branches are
//! executed. When a new input reaches a previously-unexplored branch, it's added to
//! the "corpus" and used as a seed for further mutation.
//!
//! ```
//! Seed input → Mutate → Execute → Measure coverage
//!                                  ↓
//!                         New branch reached?
//!                         Yes → Add to corpus, mutate more
//!                         No  → Try another mutation
//! ```
//!
//! ## Coverage Metrics
//!
//! - **Edge coverage**: Which pairs of adjacent basic blocks executed
//! - **Block coverage**: Which basic blocks executed
//! - **Path coverage**: Which full execution paths taken
//!
//! libFuzzer uses edge coverage (like AFL), which is more informative than block
//! coverage alone.
//!
//! ## Simulating Coverage in Rust
//!
//! We can't use libFuzzer's real instrumentation without `cargo-fuzz`, but we can
//! simulate the concept: track which "branches" an input reaches and use that to
//! guide our input generation.
//!
//! ## Security Perspective
//!
//! ### Attack: Uncovered Code Paths
//! Security-critical error-handling paths (e.g., invalid auth token) are often only
//! reached by specific inputs. If they're never tested, they may contain exploitable bugs.
//!
//! ### Defense: Maximize Coverage of Security Code
//! - Aim for 100% branch coverage on authentication, authorization, and crypto code
//! - Use coverage reports to find untested paths
//! - Write targeted tests for branches the fuzzer can't reach

use std::collections::HashSet;

/// A simple function with multiple branches to demonstrate coverage tracking.
///
/// Returns a description of the input category.
///
/// Categories:
/// - "empty" for empty input
/// - "magic" for input starting with 0xDEAD
/// - "null-byte" for input containing a zero byte
/// - "ascii" for input with all bytes in 0x20..=0x7E
/// - "binary" for everything else
pub fn categorize_input(data: &[u8]) -> &'static str {
    if data.is_empty() {
        "empty"
    } else if data.len() >= 2 && data[0] == 0xDE && data[1] == 0xAD {
        "magic"
    } else if data.contains(&0) {
        "null-byte"
    } else if data.iter().all(|&b| b >= 0x20 && b <= 0x7E) {
        "ascii"
    } else {
        "binary"
    }
}

/// A coverage tracker that records which "branches" have been hit.
///
/// Each branch is identified by a u32 ID. The tracker maintains a set of
/// all branch IDs that have been observed.
pub struct CoverageTracker {
    hit_branches: HashSet<u32>,
}

impl CoverageTracker {
    pub fn new() -> Self {
        todo!("Initialize empty coverage tracker")
    }

    /// Record that a branch was hit.
    pub fn record(&mut self, branch_id: u32) {
        todo!("Record branch hit")
    }

    /// Return the total number of unique branches hit.
    pub fn coverage_count(&self) -> usize {
        todo!("Return count of unique branches hit")
    }

    /// Return true if this branch_id has never been seen before.
    pub fn is_new_branch(&self, branch_id: u32) -> bool {
        todo!("Check if branch is new")
    }

    /// Return all hit branch IDs sorted.
    pub fn hit_branches(&self) -> Vec<u32> {
        todo!("Return sorted list of hit branches")
    }

    /// Merge another tracker's coverage into this one.
    pub fn merge(&mut self, other: &CoverageTracker) {
        todo!("Merge coverage from another tracker")
    }
}

/// Execute `categorize_input` with coverage tracking.
///
/// Records which category (branch) the input falls into, using the tracker.
/// Returns the category string.
///
/// Hints:
/// - Call categorize_input to get the category
/// - Map each category to a unique branch ID (e.g., 0=empty, 1=magic, etc.)
/// - Record the branch ID in the tracker
pub fn run_with_coverage<'a>(
    tracker: &mut CoverageTracker,
    data: &[u8],
) -> &'static str {
    todo!("Run categorize_input with coverage tracking")
}

/// A coverage-guided fuzzer simulator.
///
/// Given a corpus of seed inputs, runs each one, tracks coverage, and returns
/// only the inputs that discovered NEW coverage (new branches).
///
/// This simulates what libFuzzer does: keep inputs that explore new code paths.
///
/// Hints:
/// - Create a CoverageTracker
/// - For each seed, run_with_coverage and check if any new branches were found
/// - Return the seeds that added new coverage
pub fn coverage_guided_select(
    corpus: &[Vec<u8>],
) -> Vec<Vec<u8>> {
    todo!("Simulate coverage-guided corpus selection")
}

/// Score a set of inputs by how much coverage they collectively provide.
///
/// Returns the total number of unique branches covered by the inputs.
///
/// Hints:
/// - Create a CoverageTracker
/// - Run each input through run_with_coverage
/// - Return the total coverage count
pub fn coverage_score(inputs: &[Vec<u8>]) -> usize {
    todo!("Calculate total coverage score")
}

/// Simulate input mutation for fuzzing.
///
/// Applies random mutations to an input:
/// - Bit flip: flip a random bit
/// - Byte insert: insert a random byte
/// - Byte delete: remove a random byte
/// - Byte replace: replace a byte with a random value
///
/// Uses a deterministic seed for reproducibility.
///
/// Hints:
/// - Use the seed to pick a mutation type (seed % 4)
/// - Use seed to pick position (seed % data.len())
/// - Apply the mutation
/// - For empty input, always insert
pub fn mutate_input(data: &[u8], seed: u64) -> Vec<u8> {
    todo!("Implement deterministic input mutation")
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- categorize_input ---

    #[test]
    fn test_categorize_empty() {
        assert_eq!(categorize_input(b""), "empty");
    }

    #[test]
    fn test_categorize_magic() {
        assert_eq!(categorize_input(&[0xDE, 0xAD, 0x01]), "magic");
    }

    #[test]
    fn test_categorize_null_byte() {
        assert_eq!(categorize_input(&[1, 2, 0, 3]), "null-byte");
    }

    #[test]
    fn test_categorize_ascii() {
        assert_eq!(categorize_input(b"hello world"), "ascii");
    }

    #[test]
    fn test_categorize_binary() {
        assert_eq!(categorize_input(&[0x01, 0x80, 0xFF]), "binary");
    }

    // --- CoverageTracker ---

    #[test]
    fn test_tracker_basic() {
        let mut tracker = CoverageTracker::new();
        assert_eq!(tracker.coverage_count(), 0);

        tracker.record(1);
        assert_eq!(tracker.coverage_count(), 1);

        tracker.record(2);
        assert_eq!(tracker.coverage_count(), 2);

        // Duplicate should not increase count
        tracker.record(1);
        assert_eq!(tracker.coverage_count(), 2);
    }

    #[test]
    fn test_tracker_new_branch() {
        let mut tracker = CoverageTracker::new();
        assert!(tracker.is_new_branch(42));
        tracker.record(42);
        assert!(!tracker.is_new_branch(42));
    }

    #[test]
    fn test_tracker_merge() {
        let mut t1 = CoverageTracker::new();
        t1.record(1);
        t1.record(2);

        let mut t2 = CoverageTracker::new();
        t2.record(2);
        t2.record(3);

        t1.merge(&t2);
        assert_eq!(t1.coverage_count(), 3); // 1, 2, 3
    }

    // --- run_with_coverage ---

    #[test]
    fn test_run_with_coverage() {
        let mut tracker = CoverageTracker::new();
        let cat = run_with_coverage(&mut tracker, b"hello");
        assert_eq!(cat, "ascii");
        assert!(tracker.coverage_count() > 0);
    }

    // --- coverage_guided_select ---

    #[test]
    fn test_coverage_guided_select() {
        let corpus = vec![
            b"".to_vec(),
            b"hello".to_vec(),
            vec![0xDE, 0xAD],
            vec![1, 0, 3],
            b"world".to_vec(), // duplicate category with "hello"
        ];
        let selected = coverage_guided_select(&corpus);
        // "world" should NOT be selected (same category as "hello")
        assert!(selected.len() <= corpus.len());
        // At least the unique categories should be selected
        assert!(selected.len() >= 3, "Expected at least 3 unique categories");
    }

    // --- coverage_score ---

    #[test]
    fn test_coverage_score_empty() {
        assert_eq!(coverage_score(&[]), 0);
    }

    #[test]
    fn test_coverage_score_diverse() {
        let inputs = vec![
            b"".to_vec(),
            b"hello".to_vec(),
            vec![0xDE, 0xAD],
            vec![1, 0, 3],
            vec![0x01, 0x80],
        ];
        let score = coverage_score(&inputs);
        assert_eq!(score, 5, "All 5 categories should be covered");
    }

    // --- mutate_input ---

    #[test]
    fn test_mutate_deterministic() {
        let data = b"hello world";
        let m1 = mutate_input(data, 42);
        let m2 = mutate_input(data, 42);
        assert_eq!(m1, m2, "Same seed must produce same mutation");
    }

    #[test]
    fn test_mutate_different_seeds() {
        let data = b"hello world";
        // With enough different seeds, at least some mutations should differ
        let mutations: Vec<Vec<u8>> = (0..100).map(|s| mutate_input(data, s)).collect();
        let all_same = mutations.windows(2).all(|w| w[0] == w[1]);
        assert!(!all_same, "Different seeds should produce different mutations");
    }

    #[test]
    fn test_mutate_empty_input() {
        // Mutating empty input should produce non-empty output
        let result = mutate_input(b"", 0);
        assert!(!result.is_empty(), "Mutating empty input should add bytes");
    }

    proptest::proptest! {
        #[test]
        fn test_mutate_never_panics(data in prop::collection::vec(prop::num::u8::ANY, 0..100), seed in 0u64..10000) {
            let _ = mutate_input(&data, seed);
        }

        #[test]
        fn test_coverage_tracker_never_panics(
            ops in prop::collection::vec(prop::num::u32::ANY, 0..100)
        ) {
            let mut tracker = CoverageTracker::new();
            for id in ops {
                tracker.record(id);
            }
        }
    }
}
