//! # Lesson 10: Continuous Fuzzing and CI Integration
//!
//! ## What is Continuous Fuzzing?
//!
//! Continuous fuzzing runs fuzz tests as part of the CI/CD pipeline, so every commit
//! is tested against known and newly-generated adversarial inputs. This catches
//! regressions early and accumulates a growing corpus over time.
//!
//! ## CI Fuzzing Strategies
//!
//! 1. **Smoke fuzzing** (CI): Run each fuzz target for 30-60 seconds per PR
//! 2. **Nightly fuzzing**: Run all targets for hours overnight
//! 3. **Continuous fuzzing** (OSS-Fuzz): Run 24/7 on dedicated infrastructure
//! 4. **Corpus management**: Store and share corpora across runs
//!
//! ## Corpus Management
//!
//! The corpus is the collection of inputs that explore different code paths.
//! - **Seed corpus**: Hand-crafted inputs for known edge cases
//! - **Generated corpus**: Inputs discovered by the fuzzer
//! - **Regression corpus**: Inputs that triggered past bugs
//!
//! ## Security Perspective
//!
//! ### Attack: Time-of-Check to Time-of-Use (TOCTOU)
//! If fuzzing only runs before release, bugs introduced after the fuzz run but before
//! shipping are missed. Continuous fuzzing closes this window.
//!
//! ### Defense: Fuzz Every Commit
//! - Add fuzz smoke tests to CI (short runs)
//! - Run longer fuzz campaigns nightly
//! - Share corpora between CI runs for cumulative coverage

use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

/// Configuration for a fuzzing campaign.
#[derive(Debug, Clone)]
pub struct FuzzConfig {
    /// Maximum time to run the fuzzer.
    pub max_duration: Duration,
    /// Maximum number of iterations (0 = unlimited).
    pub max_iterations: usize,
    /// Maximum input size in bytes.
    pub max_input_size: usize,
    /// Path to the corpus directory (simulated).
    pub corpus_dir: String,
    /// Whether to minimize the corpus after fuzzing.
    pub minimize_corpus: bool,
}

impl FuzzConfig {
    /// Create a "smoke test" config for CI (short run).
    pub fn smoke_test() -> Self {
        todo!("Create smoke test fuzz config (30 seconds, 1000 iterations)")
    }

    /// Create a "nightly" config for longer runs.
    pub fn nightly() -> Self {
        todo!("Create nightly fuzz config (8 hours)")
    }

    /// Create a "thorough" config for pre-release fuzzing.
    pub fn thorough() -> Self {
        todo!("Create thorough fuzz config (24 hours)")
    }
}

/// A corpus of test inputs, tracking which inputs explore which "coverage paths".
pub struct Corpus {
    inputs: Vec<Vec<u8>>,
    coverage: BTreeMap<usize, BTreeSet<usize>>, // input_index -> set of branch IDs
}

impl Corpus {
    pub fn new() -> Self {
        todo!("Initialize empty corpus")
    }

    /// Add an input and its observed coverage. Returns true if it added new coverage.
    pub fn add(&mut self, input: Vec<u8>, branches: BTreeSet<usize>) -> bool {
        todo!("Add input to corpus if it provides new coverage")
    }

    /// Return the total number of unique branches covered by the entire corpus.
    pub fn total_coverage(&self) -> usize {
        todo!("Count total unique branches across all inputs")
    }

    /// Return the number of inputs in the corpus.
    pub fn len(&self) -> usize {
        self.inputs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inputs.is_empty()
    }

    /// Minimize the corpus: remove inputs that don't contribute unique coverage.
    ///
    /// An input is redundant if every branch it covers is also covered by some
    /// combination of other inputs.
    ///
    /// Hints:
    /// - Start with an empty set of "covered branches"
    /// - Iterate inputs, keeping only those that add new coverage
    /// - This is a greedy set cover approximation
    pub fn minimize(&mut self) {
        todo!("Minimize corpus by removing redundant inputs")
    }

    /// Serialize the corpus to a list of (hex_input, branch_count) tuples.
    pub fn serialize(&self) -> Vec<(String, usize)> {
        todo!("Serialize corpus entries")
    }
}

/// A fuzzing campaign result.
#[derive(Debug)]
pub struct FuzzResult {
    /// Total iterations performed.
    pub iterations: usize,
    /// Number of unique branches discovered.
    pub branches_found: usize,
    /// Number of inputs in the final corpus.
    pub corpus_size: usize,
    /// Time elapsed.
    pub elapsed: Duration,
    /// Whether any crashes were found.
    pub crashes_found: Vec<Vec<u8>>,
}

/// Run a simulated fuzzing campaign.
///
/// Given a function to test and a configuration, runs the function with
/// mutated inputs and tracks results.
///
/// The function under test should return Ok(branch_ids) or Err(crash_input).
///
/// Hints:
/// - Create a Corpus
/// - Loop up to max_iterations
/// - Generate mutated input (use simple counter-based mutation)
/// - Call the function
/// - Track coverage and crashes
/// - Stop if max_duration exceeded
pub fn run_fuzz_campaign<F>(
    config: &FuzzConfig,
    mut test_fn: F,
) -> FuzzResult
where
    F: FnMut(&[u8]) -> Result<BTreeSet<usize>, ()>,
{
    todo!("Run fuzzing campaign")
}

/// Generate a CI fuzzing report in a simple text format.
///
/// Format:
/// ```text
/// === Fuzzing Report ===
/// Iterations: N
/// Branches found: N
/// Corpus size: N
/// Crashes: N
/// Duration: Ns
/// Status: PASS/FAIL
/// ```
pub fn generate_report(result: &FuzzResult) -> String {
    todo!("Generate fuzzing report")
}

/// Validate that a fuzzing configuration is sane.
///
/// Checks:
/// - max_duration > 0
/// - max_input_size > 0 and <= 1MB
/// - corpus_dir is not empty
///
/// Returns Ok(()) or Err with description.
pub fn validate_config(config: &FuzzConfig) -> Result<(), String> {
    todo!("Validate fuzz configuration")
}

/// Parse a fuzzing report string back into structured data.
///
/// Expected format is the output of `generate_report`.
///
/// Returns (iterations, branches, corpus_size, crashes, duration_secs, passed).
pub fn parse_report(report: &str) -> Result<(usize, usize, usize, usize, u64, bool), String> {
    todo!("Parse fuzzing report")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    // --- FuzzConfig ---

    #[test]
    fn test_smoke_config() {
        let config = FuzzConfig::smoke_test();
        assert!(config.max_duration >= Duration::from_secs(10));
        assert!(config.max_iterations > 0);
    }

    #[test]
    fn test_nightly_config() {
        let config = FuzzConfig::nightly();
        assert!(config.max_duration >= Duration::from_hours(1));
    }

    #[test]
    fn test_thorough_config() {
        let config = FuzzConfig::thorough();
        assert!(config.max_duration >= Duration::from_hours(12));
    }

    // --- Corpus ---

    #[test]
    fn test_corpus_add_new_coverage() {
        let mut corpus = Corpus::new();
        let mut branches = BTreeSet::new();
        branches.insert(1);
        branches.insert(2);

        assert!(corpus.add(vec![0x01], branches), "Should add new coverage");
        assert_eq!(corpus.total_coverage(), 2);
    }

    #[test]
    fn test_corpus_add_redundant() {
        let mut corpus = Corpus::new();
        let mut branches = BTreeSet::new();
        branches.insert(1);
        corpus.add(vec![0x01], branches.clone());

        // Same branches = redundant
        assert!(!corpus.add(vec![0x02], branches), "Should reject redundant input");
    }

    #[test]
    fn test_corpus_add_partial_new() {
        let mut corpus = Corpus::new();
        let mut b1 = BTreeSet::new();
        b1.insert(1);
        b1.insert(2);
        corpus.add(vec![0x01], b1);

        // Subset of existing coverage = redundant
        let mut b2 = BTreeSet::new();
        b2.insert(1);
        assert!(!corpus.add(vec![0x02], b2));
    }

    #[test]
    fn test_corpus_minimize() {
        let mut corpus = Corpus::new();

        // Input 0 covers branches {1, 2, 3}
        let mut b0 = BTreeSet::new();
        b0.extend([1, 2, 3]);
        corpus.add(vec![0], b0);

        // Input 1 covers branches {1, 2} (subset -- redundant)
        let mut b1 = BTreeSet::new();
        b1.extend([1, 2]);
        corpus.add(vec![1], b1);

        // Input 2 covers branches {3, 4} (partially new)
        let mut b2 = BTreeSet::new();
        b2.extend([3, 4]);
        corpus.add(vec![2], b2);

        let before = corpus.len();
        corpus.minimize();
        let after = corpus.len();

        assert!(after <= before, "Minimization should not increase corpus size");
        assert_eq!(corpus.total_coverage(), 4, "All branches should still be covered");
    }

    #[test]
    fn test_corpus_serialize() {
        let mut corpus = Corpus::new();
        let mut branches = BTreeSet::new();
        branches.insert(5);
        corpus.add(vec![0xDE, 0xAD], branches);

        let serialized = corpus.serialize();
        assert_eq!(serialized.len(), 1);
        assert!(serialized[0].0.contains("dead"));
        assert_eq!(serialized[0].1, 1);
    }

    // --- run_fuzz_campaign ---

    #[test]
    fn test_fuzz_campaign_basic() {
        let config = FuzzConfig {
            max_duration: Duration::from_secs(5),
            max_iterations: 100,
            max_input_size: 256,
            corpus_dir: "/tmp/test-corpus".to_string(),
            minimize_corpus: false,
        };

        let result = run_fuzz_campaign(&config, |data| {
            // Simple function: branch on first byte
            let mut branches = BTreeSet::new();
            if data.is_empty() {
                branches.insert(0);
            } else if data[0] < 128 {
                branches.insert(1);
            } else {
                branches.insert(2);
            }
            Ok(branches)
        });

        assert!(result.iterations > 0);
        assert!(result.branches_found > 0);
    }

    #[test]
    fn test_fuzz_campaign_finds_all_branches() {
        let config = FuzzConfig {
            max_duration: Duration::from_secs(10),
            max_iterations: 500,
            max_input_size: 64,
            corpus_dir: "/tmp/test-corpus".to_string(),
            minimize_corpus: true,
        };

        let result = run_fuzz_campaign(&config, |data| {
            let mut branches = BTreeSet::new();
            if data.is_empty() {
                branches.insert(0);
            } else if data.len() == 1 {
                branches.insert(1);
            } else {
                branches.insert(2);
            }
            Ok(branches)
        });

        assert!(result.branches_found >= 2, "Should find at least 2 branches");
    }

    // --- generate_report / parse_report ---

    #[test]
    fn test_report_roundtrip() {
        let result = FuzzResult {
            iterations: 1000,
            branches_found: 42,
            corpus_size: 15,
            elapsed: Duration::from_secs(60),
            crashes_found: vec![],
        };

        let report = generate_report(&result);
        assert!(report.contains("1000"));
        assert!(report.contains("PASS"));

        let (iterations, branches, corpus, crashes, _duration, passed) =
            parse_report(&report).unwrap();
        assert_eq!(iterations, 1000);
        assert_eq!(branches, 42);
        assert_eq!(corpus, 15);
        assert_eq!(crashes, 0);
        assert!(passed);
    }

    #[test]
    fn test_report_with_crash() {
        let result = FuzzResult {
            iterations: 500,
            branches_found: 10,
            corpus_size: 5,
            elapsed: Duration::from_secs(30),
            crashes_found: vec![vec![0xFF, 0x00]],
        };

        let report = generate_report(&result);
        assert!(report.contains("FAIL"));

        let (_, _, _, crashes, _, passed) = parse_report(&report).unwrap();
        assert_eq!(crashes, 1);
        assert!(!passed);
    }

    // --- validate_config ---

    #[test]
    fn test_validate_config_valid() {
        let config = FuzzConfig::smoke_test();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_zero_duration() {
        let config = FuzzConfig {
            max_duration: Duration::ZERO,
            max_iterations: 100,
            max_input_size: 256,
            corpus_dir: "/tmp".to_string(),
            minimize_corpus: false,
        };
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_validate_config_empty_corpus_dir() {
        let config = FuzzConfig {
            max_duration: Duration::from_secs(10),
            max_iterations: 100,
            max_input_size: 256,
            corpus_dir: String::new(),
            minimize_corpus: false,
        };
        assert!(validate_config(&config).is_err());
    }

    proptest::proptest! {
        #[test]
        fn test_corpus_minimize_preserves_coverage(
            inputs in prop::collection::vec(
                prop::collection::vec(prop::num::usize::ANY, 1..5),
                1..20
            )
        ) {
            let mut corpus = Corpus::new();
            for input in inputs {
                let branches: BTreeSet<usize> = input.into_iter().collect();
                corpus.add(vec![0u8], branches);
            }
            let total_before = corpus.total_coverage();
            corpus.minimize();
            let total_after = corpus.total_coverage();
            prop_assert_eq!(total_before, total_after, "Minimization must preserve total coverage");
        }
    }
}
