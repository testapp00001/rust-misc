//! # Lesson 08: Hardware Random Number Generators — TRNG vs CSPRNG
//!
//! ## TRNG vs CSPRNG
//!
//! A **True Random Number Generator (TRNG)** derives randomness from physical phenomena:
//! - Thermal noise in resistors
//! - Clock jitter between oscillators
//! - Radioactive decay
//! - Quantum effects (photon detection)
//!
//! A **Cryptographically Secure PRNG (CSPRNG)** is a deterministic algorithm seeded
//! with entropy from a TRNG:
//! - ChaCha20-based (Linux /dev/urandom, Rust's OsRNG)
//! - AES-CTR based (NIST SP 800-90A)
//! - HMAC-DRBG (NIST SP 800-90A)
//!
//! | Property | TRNG | CSPRNG |
//! |----------|------|--------|
//! | Source | Physical | Algorithm |
//! | Speed | Slow | Fast |
//! | Deterministic | No | Yes |
//! | Requires seed | No | Yes |
//! | Use | Seed CSPRNG, key generation | Bulk random data |
//!
//! ## Entropy Quality
//!
//! Entropy quality is measured by:
//! - **Min-entropy**: Worst-case predictability
//! - **Shannon entropy**: Average information per symbol
//! - **Bias**: Deviation from uniform distribution
//!
//! ## Attack: Weak Entropy
//!
//! If the TRNG has low entropy or is predictable, all keys derived from it are weak.
//! Examples:
//! - Debian OpenSSL bug (2008): PRNG seeded with PID only → 32,768 possible keys
//! - Embedded devices with no hardware RNG, using boot time as seed
//!
//! ## Attack: Entropy Starvation
//!
//! On VMs or containers, multiple instances may share the same entropy pool,
//! leading to identical "random" values. Defense: use `virtio-rng` for VMs,
//! or wait for sufficient entropy before generating keys.

use ring::digest;
use serde::{Deserialize, Serialize};

/// Entropy source abstraction.
pub trait EntropySource {
    /// Fill the buffer with random bytes.
    fn fill(&self, buf: &mut [u8]);

    /// Return the name of this entropy source.
    fn name(&self) -> &str;
}

/// Simulated TRNG — uses ring's SystemRandom (which uses OS entropy, potentially hardware).
pub struct HardwareRng;

impl EntropySource for HardwareRng {
    fn fill(&self, buf: &mut [u8]) {
        use ring::rand::SecureRandom;
        let rng = ring::rand::SystemRandom::new();
        rng.fill(buf).unwrap();
    }

    fn name(&self) -> &str {
        "HardwareRng (OsRng)"
    }
}

/// Simulated weak RNG — deterministic PRNG with a fixed seed.
/// This represents what happens when you DON'T use hardware entropy.
pub struct WeakRng {
    state: u64,
}

impl WeakRng {
    /// Exercise 1: Create a weak RNG with a given seed.
    ///
    /// This is intentionally weak to demonstrate the danger of poor seeding.
    pub fn new(seed: u64) -> Self {
        todo!("Create weak PRNG with fixed seed")
    }

    /// Exercise 2: Generate the next pseudo-random byte.
    ///
    /// Use a simple linear congruential generator (LCG):
    /// state = state * 6364136223846793005 + 1442695040888963407
    /// Return (state >> 33) as u8
    fn next_byte(&mut self) -> u8 {
        todo!("Generate next pseudo-random byte using LCG")
    }
}

impl EntropySource for WeakRng {
    fn fill(&self, _buf: &mut [u8]) {
        // Note: we can't mutate self through the trait without interior mutability.
        // This is intentional — WeakRng should not be used through the trait for real.
        // In tests, use next_byte() directly.
    }

    fn name(&self) -> &str {
        "WeakRng (DO NOT USE)"
    }
}

/// Entropy pool that mixes multiple entropy sources.
#[derive(Debug)]
pub struct EntropyPool {
    /// Accumulated entropy (running hash).
    pool: Vec<u8>,
    /// Number of entropy contributions.
    contribution_count: u32,
}

impl EntropyPool {
    /// Exercise 3: Create a new empty entropy pool.
    pub fn new() -> Self {
        todo!("Create empty entropy pool")
    }

    /// Exercise 4: Add entropy to the pool.
    ///
    /// Mix new entropy into the pool using: pool = SHA-256(pool || new_entropy)
    /// This ensures that even if new_entropy is weak, the pool doesn't get worse
    /// (as long as the pool previously had good entropy).
    pub fn add_entropy(&mut self, data: &[u8]) {
        todo!("Mix entropy into the pool")
    }

    /// Exercise 5: Extract random bytes from the pool.
    ///
    /// Derive output bytes by hashing: SHA-256(pool || counter)
    /// Increment counter for each 32-byte block needed.
    ///
    /// This does NOT deplete the pool — it's a derivation, not extraction.
    pub fn extract(&self, num_bytes: usize) -> Vec<u8> {
        todo!("Derive random bytes from entropy pool")
    }

    /// Get the number of entropy contributions.
    pub fn contribution_count(&self) -> u32 {
        self.contribution_count
    }
}

/// Exercise 6: Analyze the quality of random data.
///
/// Compute basic entropy metrics:
/// 1. Byte frequency distribution (count of each byte value 0-255)
/// 2. Chi-squared statistic (measure of deviation from uniform)
///
/// Returns (byte_frequencies, chi_squared).
/// A chi-squared value close to 0 indicates near-uniform distribution.
/// Values above 300 suggest non-random data.
///
/// Hints:
/// - Create a 256-element array of counts
/// - For each byte in data, increment counts[byte as usize]
/// - Expected count per bucket = data.len() / 256
/// - Chi-squared = sum((observed - expected)^2 / expected) for each bucket
pub fn analyze_entropy(data: &[u8]) -> (Vec<u32>, f64) {
    todo!("Compute byte frequency distribution and chi-squared statistic")
}

/// Exercise 7: Detect if two random sequences likely share the same seed.
///
/// If two sequences from the same weak RNG are correlated, they may share
/// the same initial bytes. Check if the first `check_len` bytes match.
pub fn detect_seed_reuse(seq1: &[u8], seq2: &[u8], check_len: usize) -> bool {
    todo!("Detect potential seed reuse between two sequences")
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
        // Generate 10000 bytes from hardware RNG — should be near-uniform
        let rng = HardwareRng;
        let mut data = vec![0u8; 10000];
        rng.fill(&mut data);
        let (_, chi_sq) = analyze_entropy(&data);
        // Chi-squared for 255 degrees of freedom at p=0.01 is ~310
        // But with random data, we sometimes exceed this, so use a generous bound
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
