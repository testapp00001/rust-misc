//! # Lesson 06: Coverage-Guided Fuzzing Concepts (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use proptest::prelude::*;
use std::collections::HashSet;

/// A function with multiple branches for coverage tracking.
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

/// A coverage tracker that records which branches have been hit.
pub struct CoverageTracker {
    hit_branches: HashSet<u32>,
}

impl CoverageTracker {
    pub fn new() -> Self {
        Self {
            hit_branches: HashSet::new(),
        }
    }

    pub fn record(&mut self, branch_id: u32) {
        self.hit_branches.insert(branch_id);
    }

    pub fn coverage_count(&self) -> usize {
        self.hit_branches.len()
    }

    pub fn is_new_branch(&self, branch_id: u32) -> bool {
        !self.hit_branches.contains(&branch_id)
    }

    pub fn hit_branches(&self) -> Vec<u32> {
        let mut branches: Vec<u32> = self.hit_branches.iter().cloned().collect();
        branches.sort();
        branches
    }

    pub fn merge(&mut self, other: &CoverageTracker) {
        for &branch in &other.hit_branches {
            self.hit_branches.insert(branch);
        }
    }
}

/// Execute categorize_input with coverage tracking.
pub fn run_with_coverage<'a>(
    tracker: &mut CoverageTracker,
    data: &[u8],
) -> &'static str {
    let category = categorize_input(data);
    let branch_id = match category {
        "empty" => 0,
        "magic" => 1,
        "null-byte" => 2,
        "ascii" => 3,
        "binary" => 4,
        _ => 5,
    };
    tracker.record(branch_id);
    category
}

/// Coverage-guided selection: keep only inputs that discovered new coverage.
pub fn coverage_guided_select(
    corpus: &[Vec<u8>],
) -> Vec<Vec<u8>> {
    let mut tracker = CoverageTracker::new();
    let mut selected = Vec::new();

    for input in corpus {
        let before = tracker.coverage_count();
        run_with_coverage(&mut tracker, input);
        if tracker.coverage_count() > before {
            selected.push(input.clone());
        }
    }

    selected
}

/// Score a set of inputs by total coverage.
pub fn coverage_score(inputs: &[Vec<u8>]) -> usize {
    let mut tracker = CoverageTracker::new();
    for input in inputs {
        run_with_coverage(&mut tracker, input);
    }
    tracker.coverage_count()
}

/// Deterministic input mutation.
pub fn mutate_input(data: &[u8], seed: u64) -> Vec<u8> {
    if data.is_empty() {
        return vec![(seed & 0xFF) as u8];
    }

    let mut result = data.to_vec();
    let mutation_type = seed % 4;
    let pos = (seed as usize) % result.len();

    match mutation_type {
        0 => {
            // Bit flip
            let bit = ((seed >> 8) & 7) as u8;
            result[pos] ^= 1 << bit;
        }
        1 => {
            // Byte replace
            result[pos] = ((seed >> 16) & 0xFF) as u8;
        }
        2 => {
            // Byte insert
            let new_byte = ((seed >> 24) & 0xFF) as u8;
            result.insert(pos.min(result.len()), new_byte);
        }
        3 => {
            // Byte delete
            if result.len() > 1 {
                result.remove(pos);
            }
        }
        _ => unreachable!(),
    }

    result
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
        assert_eq!(t1.coverage_count(), 3);
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
            b"world".to_vec(),
        ];
        let selected = coverage_guided_select(&corpus);
        assert!(selected.len() <= corpus.len());
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
        assert_eq!(score, 5);
    }

    // --- mutate_input ---

    #[test]
    fn test_mutate_deterministic() {
        let data = b"hello world";
        let m1 = mutate_input(data, 42);
        let m2 = mutate_input(data, 42);
        assert_eq!(m1, m2);
    }

    #[test]
    fn test_mutate_different_seeds() {
        let data = b"hello world";
        let mutations: Vec<Vec<u8>> = (0..100).map(|s| mutate_input(data, s)).collect();
        let all_same = mutations.windows(2).all(|w| w[0] == w[1]);
        assert!(!all_same);
    }

    #[test]
    fn test_mutate_empty_input() {
        let result = mutate_input(b"", 0);
        assert!(!result.is_empty());
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
