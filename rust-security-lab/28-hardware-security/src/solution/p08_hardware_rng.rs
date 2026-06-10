//! # Lesson 08: Hardware RNG — TRNG vs CSPRNG (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::digest;
use ring::rand::SecureRandom;

/// Entropy source abstraction.
pub trait EntropySource {
    fn fill(&self, buf: &mut [u8]);
    fn name(&self) -> &str;
}

/// Simulated TRNG using OS entropy (which may use hardware RNG).
pub struct HardwareRng;

impl EntropySource for HardwareRng {
    fn fill(&self, buf: &mut [u8]) {
        let rng = ring::rand::SystemRandom::new();
        rng.fill(buf).unwrap();
    }

    fn name(&self) -> &str {
        "HardwareRng (OsRng)"
    }
}

/// Weak deterministic PRNG (LCG) — demonstrates poor entropy.
pub struct WeakRng {
    state: u64,
}

impl WeakRng {
    /// Create a weak RNG with a given seed.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Generate the next pseudo-random byte using LCG.
    pub fn next_byte(&mut self) -> u8 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.state >> 33) & 0xFF) as u8
    }
}

impl EntropySource for WeakRng {
    fn fill(&self, _buf: &mut [u8]) {
        // Intentionally unimplemented through trait — WeakRng is for demonstration only
    }

    fn name(&self) -> &str {
        "WeakRng (DO NOT USE)"
    }
}

/// Entropy pool that mixes multiple entropy sources.
#[derive(Debug)]
pub struct EntropyPool {
    pool: Vec<u8>,
    contribution_count: u32,
}

impl EntropyPool {
    /// Create a new empty entropy pool.
    pub fn new() -> Self {
        Self {
            pool: vec![0u8; 32], // Initialize with zeros
            contribution_count: 0,
        }
    }

    /// Mix entropy into the pool: pool = SHA-256(pool || new_entropy).
    pub fn add_entropy(&mut self, data: &[u8]) {
        let mut combined = self.pool.clone();
        combined.extend_from_slice(data);
        self.pool = digest::digest(&digest::SHA256, &combined).as_ref().to_vec();
        self.contribution_count += 1;
    }

    /// Derive random bytes from the pool using SHA-256(pool || counter).
    pub fn extract(&self, num_bytes: usize) -> Vec<u8> {
        let mut output = Vec::with_capacity(num_bytes);
        let mut counter: u64 = 0;

        while output.len() < num_bytes {
            let mut input = self.pool.clone();
            input.extend_from_slice(&counter.to_le_bytes());
            let block = digest::digest(&digest::SHA256, &input).as_ref().to_vec();
            let remaining = num_bytes - output.len();
            let take = remaining.min(32);
            output.extend_from_slice(&block[..take]);
            counter += 1;
        }

        output
    }

    pub fn contribution_count(&self) -> u32 {
        self.contribution_count
    }
}

/// Analyze entropy quality: byte frequency distribution and chi-squared statistic.
pub fn analyze_entropy(data: &[u8]) -> (Vec<u32>, f64) {
    let mut freq = vec![0u32; 256];
    for &byte in data {
        freq[byte as usize] += 1;
    }

    let expected = data.len() as f64 / 256.0;
    let chi_sq: f64 = freq
        .iter()
        .map(|&obs| {
            let diff = obs as f64 - expected;
            diff * diff / expected
        })
        .sum();

    (freq, chi_sq)
}

/// Detect potential seed reuse between two sequences.
pub fn detect_seed_reuse(seq1: &[u8], seq2: &[u8], check_len: usize) -> bool {
    let len = check_len.min(seq1.len()).min(seq2.len());
    if len == 0 {
        return false;
    }
    seq1[..len] == seq2[..len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_rng_produces_output() {
        let rng = HardwareRng;
        let mut buf = vec![0u8; 32];
        rng.fill(&mut buf);
        assert!(!buf.iter().all(|&b| b == 0), "Hardware RNG should produce non-zero output");
    }

    #[test]
    fn test_weak_rng_deterministic() {
        let mut rng1 = WeakRng::new(42);
        let mut rng2 = WeakRng::new(42);
        for _ in 0..100 {
            assert_eq!(rng1.next_byte(), rng2.next_byte(), "Same seed should produce same output");
        }
    }

    #[test]
    fn test_weak_rng_different_seeds() {
        let mut rng1 = WeakRng::new(1);
        let mut rng2 = WeakRng::new(2);
        let b1: Vec<u8> = (0..32).map(|_| rng1.next_byte()).collect();
        let b2: Vec<u8> = (0..32).map(|_| rng2.next_byte()).collect();
        assert_ne!(b1, b2, "Different seeds should produce different output");
    }

    #[test]
    fn test_entropy_pool_mixed() {
        let mut pool = EntropyPool::new();
        pool.add_entropy(b"first contribution");
        pool.add_entropy(b"second contribution");
        assert_eq!(pool.contribution_count(), 2);
    }

    #[test]
    fn test_entropy_pool_extract_length() {
        let mut pool = EntropyPool::new();
        pool.add_entropy(b"seed data here");
        let output = pool.extract(100);
        assert_eq!(output.len(), 100);
    }

    #[test]
    fn test_entropy_pool_deterministic() {
        let mut pool = EntropyPool::new();
        pool.add_entropy(b"same seed");
        let out1 = pool.extract(32);
        let out2 = pool.extract(32);
        assert_eq!(out1, out2, "Same pool state should produce same output");
    }

    #[test]
    fn test_entropy_analysis_uniform() {
        let rng = HardwareRng;
        let mut data = vec![0u8; 10000];
        rng.fill(&mut data);
        let (_, chi_sq) = analyze_entropy(&data);
        assert!(chi_sq < 500.0, "Hardware RNG should produce near-uniform data, chi-squared: {}", chi_sq);
    }

    #[test]
    fn test_seed_reuse_detection() {
        let mut rng1 = WeakRng::new(999);
        let mut rng2 = WeakRng::new(999);
        let seq1: Vec<u8> = (0..32).map(|_| rng1.next_byte()).collect();
        let seq2: Vec<u8> = (0..32).map(|_| rng2.next_byte()).collect();
        assert!(detect_seed_reuse(&seq1, &seq2, 16));
    }

    #[test]
    fn test_no_seed_reuse_different_seeds() {
        let mut rng1 = WeakRng::new(111);
        let mut rng2 = WeakRng::new(222);
        let seq1: Vec<u8> = (0..32).map(|_| rng1.next_byte()).collect();
        let seq2: Vec<u8> = (0..32).map(|_| rng2.next_byte()).collect();
        assert!(!detect_seed_reuse(&seq1, &seq2, 16));
    }
}
