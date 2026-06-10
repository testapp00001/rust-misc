//! # Lesson 10: Continuous Fuzzing and CI Integration (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use proptest::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

/// Configuration for a fuzzing campaign.
#[derive(Debug, Clone)]
pub struct FuzzConfig {
    pub max_duration: Duration,
    pub max_iterations: usize,
    pub max_input_size: usize,
    pub corpus_dir: String,
    pub minimize_corpus: bool,
}

impl FuzzConfig {
    pub fn smoke_test() -> Self {
        Self {
            max_duration: Duration::from_secs(30),
            max_iterations: 1000,
            max_input_size: 4096,
            corpus_dir: "fuzz-corpus/smoke".to_string(),
            minimize_corpus: true,
        }
    }

    pub fn nightly() -> Self {
        Self {
            max_duration: Duration::from_secs(8 * 3600),
            max_iterations: 0, // unlimited
            max_input_size: 65536,
            corpus_dir: "fuzz-corpus/nightly".to_string(),
            minimize_corpus: true,
        }
    }

    pub fn thorough() -> Self {
        Self {
            max_duration: Duration::from_secs(24 * 3600),
            max_iterations: 0,
            max_input_size: 1024 * 1024,
            corpus_dir: "fuzz-corpus/thorough".to_string(),
            minimize_corpus: true,
        }
    }
}

/// A corpus of test inputs with coverage tracking.
pub struct Corpus {
    inputs: Vec<Vec<u8>>,
    coverage: BTreeMap<usize, BTreeSet<usize>>,
}

impl Corpus {
    pub fn new() -> Self {
        Self {
            inputs: Vec::new(),
            coverage: BTreeMap::new(),
        }
    }

    /// Add an input and its observed coverage. Returns true if it added new coverage.
    pub fn add(&mut self, input: Vec<u8>, branches: BTreeSet<usize>) -> bool {
        // Check if this input provides any new coverage
        let existing: BTreeSet<usize> = self
            .coverage
            .values()
            .flat_map(|s| s.iter().cloned())
            .collect();

        let new_branches: BTreeSet<usize> = branches.difference(&existing).cloned().collect();

        if new_branches.is_empty() {
            return false;
        }

        let index = self.inputs.len();
        self.inputs.push(input);
        self.coverage.insert(index, branches);
        true
    }

    pub fn total_coverage(&self) -> usize {
        let all: BTreeSet<usize> = self
            .coverage
            .values()
            .flat_map(|s| s.iter().cloned())
            .collect();
        all.len()
    }

    pub fn len(&self) -> usize {
        self.inputs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inputs.is_empty()
    }

    /// Minimize the corpus using greedy set cover.
    pub fn minimize(&mut self) {
        let mut covered: BTreeSet<usize> = BTreeSet::new();
        let mut keep: Vec<bool> = vec![false; self.inputs.len()];

        // Greedy: pick inputs that cover the most new branches first
        let mut remaining: Vec<usize> = (0..self.inputs.len()).collect();

        loop {
            let mut best_idx = None;
            let mut best_new = 0;

            for &idx in &remaining {
                if let Some(branches) = self.coverage.get(&idx) {
                    let new_count = branches.difference(&covered).count();
                    if new_count > best_new {
                        best_new = new_count;
                        best_idx = Some(idx);
                    }
                }
            }

            if let Some(idx) = best_idx {
                if best_new == 0 {
                    break;
                }
                keep[idx] = true;
                if let Some(branches) = self.coverage.get(&idx) {
                    covered.extend(branches.iter().cloned());
                }
                remaining.retain(|&i| i != idx);
            } else {
                break;
            }
        }

        // Rebuild with only kept inputs
        let old_inputs = std::mem::take(&mut self.inputs);
        let old_coverage = std::mem::take(&mut self.coverage);

        for (i, (input, branches)) in old_inputs
            .into_iter()
            .zip(old_coverage.into_iter())
            .enumerate()
        {
            if keep[i] {
                let new_idx = self.inputs.len();
                self.inputs.push(input);
                self.coverage.insert(new_idx, branches.1);
            }
        }
    }

    pub fn serialize(&self) -> Vec<(String, usize)> {
        self.inputs
            .iter()
            .enumerate()
            .map(|(i, input)| {
                let branch_count = self
                    .coverage
                    .get(&i)
                    .map(|s| s.len())
                    .unwrap_or(0);
                (hex::encode(input), branch_count)
            })
            .collect()
    }
}

/// Fuzzing campaign result.
#[derive(Debug)]
pub struct FuzzResult {
    pub iterations: usize,
    pub branches_found: usize,
    pub corpus_size: usize,
    pub elapsed: Duration,
    pub crashes_found: Vec<Vec<u8>>,
}

/// Run a simulated fuzzing campaign.
pub fn run_fuzz_campaign<F>(
    config: &FuzzConfig,
    mut test_fn: F,
) -> FuzzResult
where
    F: FnMut(&[u8]) -> Result<BTreeSet<usize>, ()>,
{
    let start = Instant::now();
    let mut corpus = Corpus::new();
    let mut crashes = Vec::new();
    let mut iterations = 0usize;
    let mut seed = 12345u64;

    loop {
        // Check termination conditions
        if config.max_iterations > 0 && iterations >= config.max_iterations {
            break;
        }
        if start.elapsed() >= config.max_duration {
            break;
        }

        // Generate input: simple seed-based mutation
        let input = generate_input(seed, config.max_input_size);
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);

        // Run the test function
        match test_fn(&input) {
            Ok(branches) => {
                corpus.add(input, branches);
            }
            Err(()) => {
                crashes.push(input);
            }
        }

        iterations += 1;
    }

    let branches_found = corpus.total_coverage();
    let corpus_size = corpus.len();

    FuzzResult {
        iterations,
        branches_found,
        corpus_size,
        elapsed: start.elapsed(),
        crashes_found: crashes,
    }
}

fn generate_input(seed: u64, max_size: usize) -> Vec<u8> {
    let size = if max_size == 0 {
        1
    } else {
        ((seed as usize) % max_size).max(1)
    };
    let mut data = Vec::with_capacity(size);
    let mut s = seed;
    for _ in 0..size {
        data.push((s & 0xFF) as u8);
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
    }
    data
}

/// Generate a CI fuzzing report.
pub fn generate_report(result: &FuzzResult) -> String {
    let status = if result.crashes_found.is_empty() {
        "PASS"
    } else {
        "FAIL"
    };
    format!(
        "=== Fuzzing Report ===\n\
         Iterations: {}\n\
         Branches found: {}\n\
         Corpus size: {}\n\
         Crashes: {}\n\
         Duration: {}s\n\
         Status: {}",
        result.iterations,
        result.branches_found,
        result.corpus_size,
        result.crashes_found.len(),
        result.elapsed.as_secs(),
        status
    )
}

/// Validate fuzzing configuration.
pub fn validate_config(config: &FuzzConfig) -> Result<(), String> {
    if config.max_duration.is_zero() {
        return Err("max_duration must be > 0".to_string());
    }
    if config.max_input_size == 0 {
        return Err("max_input_size must be > 0".to_string());
    }
    if config.max_input_size > 1024 * 1024 {
        return Err("max_input_size must be <= 1MB".to_string());
    }
    if config.corpus_dir.is_empty() {
        return Err("corpus_dir must not be empty".to_string());
    }
    Ok(())
}

/// Parse a fuzzing report string.
pub fn parse_report(report: &str) -> Result<(usize, usize, usize, usize, u64, bool), String> {
    let mut iterations = 0;
    let mut branches = 0;
    let mut corpus_size = 0;
    let mut crashes = 0;
    let mut duration_secs = 0;
    let mut passed = false;

    for line in report.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("Iterations:") {
            iterations = val.trim().parse().map_err(|_| "bad iterations")?;
        } else if let Some(val) = line.strip_prefix("Branches found:") {
            branches = val.trim().parse().map_err(|_| "bad branches")?;
        } else if let Some(val) = line.strip_prefix("Corpus size:") {
            corpus_size = val.trim().parse().map_err(|_| "bad corpus size")?;
        } else if let Some(val) = line.strip_prefix("Crashes:") {
            crashes = val.trim().parse().map_err(|_| "bad crashes")?;
        } else if let Some(val) = line.strip_prefix("Duration:") {
            let s = val.trim().trim_end_matches('s');
            duration_secs = s.parse().map_err(|_| "bad duration")?;
        } else if let Some(val) = line.strip_prefix("Status:") {
            passed = val.trim() == "PASS";
        }
    }

    Ok((iterations, branches, corpus_size, crashes, duration_secs, passed))
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
        assert!(config.max_duration >= Duration::from_secs(3600));
    }

    #[test]
    fn test_thorough_config() {
        let config = FuzzConfig::thorough();
        assert!(config.max_duration >= Duration::from_secs(12 * 3600));
    }

    // --- Corpus ---

    #[test]
    fn test_corpus_add_new_coverage() {
        let mut corpus = Corpus::new();
        let mut branches = BTreeSet::new();
        branches.insert(1);
        branches.insert(2);

        assert!(corpus.add(vec![0x01], branches));
        assert_eq!(corpus.total_coverage(), 2);
    }

    #[test]
    fn test_corpus_add_redundant() {
        let mut corpus = Corpus::new();
        let mut branches = BTreeSet::new();
        branches.insert(1);
        corpus.add(vec![0x01], branches.clone());

        assert!(!corpus.add(vec![0x02], branches));
    }

    #[test]
    fn test_corpus_add_partial_new() {
        let mut corpus = Corpus::new();
        let mut b1 = BTreeSet::new();
        b1.insert(1);
        b1.insert(2);
        corpus.add(vec![0x01], b1);

        let mut b2 = BTreeSet::new();
        b2.insert(1);
        assert!(!corpus.add(vec![0x02], b2));
    }

    #[test]
    fn test_corpus_minimize() {
        let mut corpus = Corpus::new();

        let mut b0 = BTreeSet::new();
        b0.extend([1, 2, 3]);
        corpus.add(vec![0], b0);

        let mut b1 = BTreeSet::new();
        b1.extend([1, 2]);
        corpus.add(vec![1], b1);

        let mut b2 = BTreeSet::new();
        b2.extend([3, 4]);
        corpus.add(vec![2], b2);

        let before = corpus.len();
        corpus.minimize();
        let after = corpus.len();

        assert!(after <= before);
        assert_eq!(corpus.total_coverage(), 4);
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

        assert!(result.branches_found >= 2);
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
