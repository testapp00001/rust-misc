//! # Exercise: Hash Function Comparison
//!
//! ## Theory
//!
//! The choice of hash function affects the quality of consistent hashing.
//! A good hash function for consistent hashing should:
//! - Produce uniformly distributed values
//! - Have low collision rates
//! - Be fast to compute
//!
//! Common hash functions used in distributed systems:
//! - SipHash: Cryptographically strong, used in Rust's HashMap
//! - FNV-1a: Simple, fast, good distribution
//! - xxHash: Very fast, non-cryptographic
//! - MurmurHash3: Good distribution, fast, widely used
//!
//! ## Proof / Intuition
//!
//! For consistent hashing, we need the hash ring to be uniformly populated.
//! If the hash function has biases (some positions more likely than others),
//! the ring will have uneven segment sizes, leading to load imbalance.
//!
//! ## Implementation Task
//!
//! Implement multiple hash functions and compare their:
//! - Distribution uniformity
//! - Collision rates
//! - Speed
//!
//! ## Verification
//!
//! Verify all hash functions produce reasonable distribution and compare
//! their performance characteristics.
//!
use std::time::Instant;

/// Simple FNV-1a hash (64-bit).
pub fn fnv1a_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
    const FNV_PRIME: u64 = 0x100000001b3; // FNV prime
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Simple xxHash (64-bit, simplified version).
pub fn xxhash(data: &[u8]) -> u64 {
    const PRIME1: u64 = 0x9e3779b185ebca77;
    const PRIME2: u64 = 0xc2b2ae3d27d4eb4f;
    const PRIME3: u64 = 0x165667b19e3779f9;
    const PRIME4: u64 = 0x85ebca77c2b2ae63;
    const PRIME5: u64 = 0x27d4eb2f165667c5;

    let mut hash = PRIME5.wrapping_add((data.len() as u64).wrapping_mul(PRIME2));

    for chunk in data.chunks(8) {
        let mut val = [0u8; 8];
        val[..chunk.len()].copy_from_slice(chunk);
        let k = u64::from_le_bytes(val);
        let k = k.wrapping_mul(PRIME2);
        let k = k.rotate_left(31);
        let k = k.wrapping_mul(PRIME1);
        hash ^= k;
        hash = hash.rotate_left(27);
        hash = hash.wrapping_mul(PRIME1).wrapping_add(PRIME4);
    }

    // Process remaining bytes
    let mut val = [0u8; 8];
    val[..data.len() % 8].copy_from_slice(&data[data.len() - (data.len() % 8)..]);
    let k = u64::from_le_bytes(val);
    hash ^= k.wrapping_mul(PRIME3);
    hash = hash.rotate_left(31);
    hash = hash.wrapping_mul(PRIME4);

    // Finalize
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(PRIME2);
    hash ^= hash >> 29;
    hash = hash.wrapping_mul(PRIME3);
    hash ^= hash >> 32;
    hash
}

/// Simple MurmurHash3 (64-bit, simplified).
pub fn murmurhash3(data: &[u8]) -> u64 {
    const C1: u64 = 0xff51afd7ed558ccd;
    const C2: u64 = 0xc4ceb9fe1a85ec53;
    const SEED: u64 = 0xdeadbeef;

    let mut hash = SEED;
    let len = data.len();

    for chunk in data.chunks(8) {
        let mut val = [0u8; 8];
        val[..chunk.len()].copy_from_slice(chunk);
        let mut k = u64::from_le_bytes(val);

        if chunk.len() < 8 {
            // Partial block - just use what we have
            k = u64::from_le_bytes(val);
        }

        k = k.wrapping_mul(C1);
        k = k.rotate_left(31);
        k = k.wrapping_mul(C2);
        hash ^= k;
        hash = hash.rotate_left(27);
        hash = hash.wrapping_mul(5).wrapping_add(0xe6546b64);
    }

    // Finalization
    hash ^= len as u64;
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(C2);
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(C1);
    hash ^= hash >> 33;

    hash
}

/// Compute SipHash using Rust's standard library hasher.
pub fn siphash(data: &[u8]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

/// Analyze hash function distribution.
pub struct HashAnalyzer {
    name: String,
}

impl HashAnalyzer {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    /// Compute distribution of hash values into bins.
    pub fn distribution(&self, keys: &[String], num_bins: usize) -> Vec<usize> {
        let hash_fn = self.get_hash_fn();
        let mut bins = vec![0usize; num_bins];
        for key in keys {
            let hash = hash_fn(key.as_bytes());
            let bin = (hash as usize) % num_bins;
            bins[bin] += 1;
        }
        bins
    }

    /// Compute standard deviation of bin counts.
    pub fn std_dev(&self, keys: &[String], num_bins: usize) -> f64 {
        let bins = self.distribution(keys, num_bins);
        let mean = bins.iter().map(|b| *b as f64).sum::<f64>() / bins.len() as f64;
        let variance = bins.iter().map(|b| (*b as f64 - mean).powi(2)).sum::<f64>() / bins.len() as f64;
        variance.sqrt()
    }

    /// Compute coefficient of variation.
    pub fn coefficient_of_variation(&self, keys: &[String], num_bins: usize) -> f64 {
        let bins = self.distribution(keys, num_bins);
        let mean = bins.iter().map(|b| *b as f64).sum::<f64>() / bins.len() as f64;
        if mean == 0.0 {
            return 0.0;
        }
        let variance = bins.iter().map(|b| (*b as f64 - mean).powi(2)).sum::<f64>() / bins.len() as f64;
        variance.sqrt() / mean
    }

    /// Count collisions (bins with >1 entry).
    pub fn collision_rate(&self, keys: &[String], num_bins: usize) -> f64 {
        let bins = self.distribution(keys, num_bins);
        let occupied = bins.iter().filter(|&&b| b > 1).count();
        occupied as f64 / num_bins as f64
    }

    /// Measure hash speed (operations per second).
    pub fn measure_speed(&self, keys: &[String], iterations: usize) -> f64 {
        let hash_fn = self.get_hash_fn();
        let start = Instant::now();
        for _ in 0..iterations {
            for key in keys {
                hash_fn(key.as_bytes());
            }
        }
        let elapsed = start.elapsed();
        let total_ops = keys.len() * iterations;
        total_ops as f64 / elapsed.as_secs_f64()
    }

    fn get_hash_fn(&self) -> fn(&[u8]) -> u64 {
        match self.name.as_str() {
            "fnv1a" => fnv1a_hash,
            "xxhash" => xxhash,
            "murmurhash3" => murmurhash3,
            "siphash" => siphash,
            _ => fnv1a_hash,
        }
    }
}

/// Generate test keys.
pub fn generate_keys(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("key_{}", i)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_hash_functions_produce_distribution() {
        let keys = generate_keys(10_000);
        let num_bins = 100;

        for name in ["fnv1a", "xxhash", "murmurhash3", "siphash"] {
            let analyzer = HashAnalyzer::new(name);
            let cv = analyzer.coefficient_of_variation(&keys, num_bins);
            assert!(
                cv < 0.15,
                "{} should produce reasonable distribution (CV={:.4})",
                name,
                cv
            );
        }
    }

    #[test]
    fn test_hash_functions_deterministic() {
        let key = "test_key";
        assert_eq!(fnv1a_hash(key.as_bytes()), fnv1a_hash(key.as_bytes()));
        assert_eq!(xxhash(key.as_bytes()), xxhash(key.as_bytes()));
        assert_eq!(murmurhash3(key.as_bytes()), murmurhash3(key.as_bytes()));
        assert_eq!(siphash(key.as_bytes()), siphash(key.as_bytes()));
    }

    #[test]
    fn test_different_keys_produce_different_hashes() {
        let keys = generate_keys(1000);
        let hashes_fnv: Vec<u64> = keys.iter().map(|k| fnv1a_hash(k.as_bytes())).collect();
        let unique: std::collections::HashSet<u64> = hashes_fnv.into_iter().collect();
        assert!(
            unique.len() > 990,
            "FNV-1a should produce mostly unique hashes for unique keys ({} unique out of 1000)",
            unique.len()
        );
    }

    #[test]
    fn test_speed_measurement_runs() {
        let keys = generate_keys(1000);
        let analyzer = HashAnalyzer::new("fnv1a");
        let ops_per_sec = analyzer.measure_speed(&keys, 100);
        assert!(
            ops_per_sec > 0.0,
            "Speed measurement should produce positive value"
        );
    }
}
