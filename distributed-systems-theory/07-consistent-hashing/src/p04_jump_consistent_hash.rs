//! # Exercise: Jump Consistent Hash
//!
//! ## Theory
//!
//! Jump Consistent Hash, proposed by John Lamping and Eric Veach at Google (2014),
//! provides a perfectly uniform hash distribution with O(1) memory and O(ln(n)) time.
//!
//! The algorithm uses a mathematical formula to determine which bucket a key should
//! be assigned to. It simulates a random walk on a ring, "jumping" forward when the
//! random value falls in a certain range.
//!
//! ## Proof / Intuition
//!
//! The algorithm works by maintaining a counter starting at -1 and repeatedly:
//! 1. Increment the counter
//! 2. Generate a random number r in [0, 1)
//! 3. While r < (counter+1)/buckets: set r = r * buckets/(counter+1), increment
//!
//! The result is that for n buckets, each key has a 1/n probability of landing in
//! any bucket, and adding a new bucket only moves ~1/(n+1) of keys from bucket n
//! to bucket n+1.
//!
//! ## Implementation Task
//!
//! Implement `jump_consistent_hash(key: u64, num_buckets: usize) -> usize`.
//! The key is used to seed the random number generator for reproducibility.
//!
//! ## Verification
//!
//! Verify uniform distribution and minimal key movement when adding buckets.
//!
/// Jump Consistent Hash: assigns a key to a bucket in [0, num_buckets).
///
/// - O(1) memory
/// - O(ln(num_buckets)) time
/// - Perfectly uniform distribution
/// - Only supports bucket ranges [0, n)
///
/// Based on the algorithm from "A Fast, Minimal Memory, Consistent Hash Algorithm"
/// by Lamping & Veach (Google, 2014).
pub fn jump_consistent_hash(key: u64, num_buckets: usize) -> usize {
    if num_buckets <= 1 {
        return 0;
    }

    let mut key = key;
    let mut b: i64 = -1;
    let mut j: i64 = 0;

    while (j as usize) < num_buckets {
        b = j;
        // Linear congruential generator
        key = key.wrapping_mul(2862933555777941757).wrapping_add(1);
        // Compute next j using the formula from the paper
        let mut k = key >> 33;
        k = k.wrapping_add(1);
        let num_buckets_f = num_buckets as f64;
        let k_f = k as f64;
        let b_plus_1_f = (b + 1) as f64;
        j = (b_plus_1_f * (2147483648.0 / k_f)) as i64;
    }

    b as usize
}

/// Measure key movement when adding a bucket.
pub fn measure_key_movement(
    keys: &[u64],
    old_buckets: usize,
    new_buckets: usize,
) -> (usize, usize) {
    let moved = keys
        .iter()
        .filter(|&&key| {
            jump_consistent_hash(key, old_buckets) != jump_consistent_hash(key, new_buckets)
        })
        .count();
    (moved, keys.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_uniform_distribution() {
        let num_buckets = 10;
        let num_keys = 100_000;

        let mut counts = HashMap::new();
        for key in 0..num_keys {
            let bucket = jump_consistent_hash(key as u64, num_buckets);
            *counts.entry(bucket).or_insert(0) += 1;
        }

        // Each bucket should get roughly 1/num_buckets of the keys
        let expected = num_keys / num_buckets;
        for bucket in 0..num_buckets {
            let count = counts.get(&bucket).copied().unwrap_or(0);
            let deviation = (count as f64 - expected as f64).abs() / expected as f64;
            assert!(
                deviation < 0.05,
                "Bucket {} got {} keys, expected ~{} (deviation: {:.1}%)",
                bucket,
                count,
                expected,
                deviation * 100.0
            );
        }
    }

    #[test]
    fn test_minimal_key_movement_on_add() {
        let keys: Vec<u64> = (0..100_000).collect();
        let (moved, total) = measure_key_movement(&keys, 10, 11);

        let move_ratio = moved as f64 / total as f64;
        // Adding a bucket should move ~1/N keys
        assert!(
            move_ratio < 0.15,
            "Adding a bucket should move ~1/10 of keys, but moved {:.1}% ({}/{})",
            move_ratio * 100.0,
            moved,
            total
        );
        assert!(
            move_ratio > 0.05,
            "Adding a bucket should move ~1/10 of keys, but only moved {:.1}%",
            move_ratio * 100.0
        );
    }

    #[test]
    fn test_deterministic_results() {
        let key = 12345;
        let b1 = jump_consistent_hash(key, 10);
        let b2 = jump_consistent_hash(key, 10);
        assert_eq!(b1, b2, "Same key must produce same bucket");
    }

    #[test]
    fn test_output_in_valid_range() {
        for num_buckets in 1..=100 {
            for key in 0..1000 {
                let bucket = jump_consistent_hash(key, num_buckets);
                assert!(
                    bucket < num_buckets,
                    "Bucket {} out of range [0, {}) for key {}",
                    bucket,
                    num_buckets,
                    key
                );
            }
        }
    }

    #[test]
    fn test_single_bucket() {
        for key in 0..100 {
            assert_eq!(jump_consistent_hash(key, 1), 0);
        }
    }
}
