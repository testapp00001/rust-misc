//! # Lesson 05: Secure Aggregation (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use rand::Rng;
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskedValue {
    pub party_id: usize,
    pub masked_value: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggParams {
    pub num_parties: usize,
    pub modulus: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairwiseSeed {
    pub party_i: usize,
    pub party_j: usize,
    pub seed: i64,
}

/// Generate pairwise random seeds for all party pairs (i < j).
pub fn generate_pairwise_seeds(params: &AggParams) -> Vec<PairwiseSeed> {
    let mut rng = rand::thread_rng();
    let mut seeds = Vec::new();

    for i in 0..params.num_parties {
        for j in (i + 1)..params.num_parties {
            seeds.push(PairwiseSeed {
                party_i: i,
                party_j: j,
                seed: rng.gen_range(1..params.modulus),
            });
        }
    }

    seeds
}

/// Compute the combined mask for a party from their pairwise seeds.
///
/// For party i: mask = sum of s_ij where i is party_i (+seed)
///                          + sum of (-s_ji) where i is party_j
pub fn compute_party_mask(
    party_id: usize,
    seeds: &[PairwiseSeed],
    modulus: i64,
) -> i64 {
    let mut mask = 0i64;
    for seed in seeds {
        if seed.party_i == party_id {
            mask = (mask + seed.seed) % modulus;
        } else if seed.party_j == party_id {
            mask = (mask - seed.seed) % modulus;
        }
    }
    ((mask % modulus) + modulus) % modulus
}

/// Add mask to input value.
pub fn mask_value(input: i64, mask: i64, modulus: i64) -> i64 {
    ((input % modulus + mask) % modulus + modulus) % modulus
}

/// Sum all masked values to get the true aggregate.
pub fn aggregate(
    masked_values: &[MaskedValue],
    params: &AggParams,
) -> Result<i64, String> {
    if masked_values.len() != params.num_parties {
        return Err(format!(
            "Expected {} masked values, got {}",
            params.num_parties,
            masked_values.len()
        ));
    }

    // Verify all party IDs are present
    let mut ids: Vec<usize> = masked_values.iter().map(|v| v.party_id).collect();
    ids.sort();
    ids.dedup();
    if ids.len() != params.num_parties {
        return Err("Duplicate or missing party IDs".to_string());
    }

    let sum: i64 = masked_values
        .iter()
        .map(|v| v.masked_value)
        .sum::<i64>()
        % params.modulus;

    Ok(((sum % params.modulus) + params.modulus) % params.modulus)
}

/// Full secure aggregation protocol.
pub fn secure_sum(inputs: &[i64], modulus: i64) -> Result<i64, String> {
    let n = inputs.len();
    if n == 0 {
        return Ok(0);
    }

    let params = AggParams {
        num_parties: n,
        modulus,
    };

    // Step 1: Generate pairwise seeds
    let seeds = generate_pairwise_seeds(&params);

    // Step 2: Each party computes their mask and masks their value
    let masked_values: Vec<MaskedValue> = inputs
        .iter()
        .enumerate()
        .map(|(i, &input)| {
            let mask = compute_party_mask(i, &seeds, modulus);
            MaskedValue {
                party_id: i,
                masked_value: mask_value(input, mask, modulus),
            }
        })
        .collect();

    // Step 3: Aggregate
    aggregate(&masked_values, &params)
}

/// Compute average using secure aggregation.
pub fn secure_average(inputs: &[i64], modulus: i64) -> Result<f64, String> {
    if inputs.is_empty() {
        return Ok(0.0);
    }
    let total = secure_sum(inputs, modulus)?;
    Ok(total as f64 / inputs.len() as f64)
}

/// Derive a deterministic seed from party IDs and a master secret.
pub fn derive_seed(party_i: usize, party_j: usize, master: i64) -> i64 {
    let mut hasher = Sha256::new();
    hasher.update(party_i.to_le_bytes());
    hasher.update(party_j.to_le_bytes());
    hasher.update(master.to_le_bytes());
    let hash = hasher.finalize();
    let bytes = &hash[..8];
    i64::from_le_bytes(bytes.try_into().unwrap()).abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_params() -> AggParams {
        AggParams {
            num_parties: 5,
            modulus: 1_000_007,
        }
    }

    #[test]
    fn test_generate_pairwise_seeds_count() {
        let params = test_params();
        let seeds = generate_pairwise_seeds(&params);
        assert_eq!(seeds.len(), 10);
    }

    #[test]
    fn test_pairwise_seeds_unique() {
        let params = test_params();
        let seeds = generate_pairwise_seeds(&params);
        let mut seeds_sorted: Vec<(usize, usize)> = seeds
            .iter()
            .map(|s| (s.party_i, s.party_j))
            .collect();
        seeds_sorted.sort();
        seeds_sorted.dedup();
        assert_eq!(seeds_sorted.len(), 10);
    }

    #[test]
    fn test_masks_cancel() {
        let params = test_params();
        let seeds = generate_pairwise_seeds(&params);
        let total_mask: i64 = (0..params.num_parties)
            .map(|i| compute_party_mask(i, &seeds, params.modulus))
            .sum::<i64>()
            % params.modulus;
        assert_eq!(total_mask, 0);
    }

    #[test]
    fn test_secure_sum_basic() {
        let inputs = vec![10, 20, 30, 40, 50];
        let result = secure_sum(&inputs, 1_000_007).unwrap();
        assert_eq!(result, 150);
    }

    #[test]
    fn test_secure_sum_single_party() {
        let inputs = vec![42];
        let result = secure_sum(&inputs, 1_000_007).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_secure_sum_negative_values() {
        let inputs = vec![100, -30, 50];
        let result = secure_sum(&inputs, 1_000_007).unwrap();
        assert_eq!(result, 120);
    }

    #[test]
    fn test_secure_average() {
        let inputs = vec![10, 20, 30, 40];
        let avg = secure_average(&inputs, 1_000_007).unwrap();
        assert!((avg - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_mask_value() {
        let mask = 100;
        let input = 50;
        let masked = mask_value(input, mask, 1_000_007);
        assert_eq!(masked, 150);
    }

    #[test]
    fn test_aggregate_checks_party_count() {
        let params = test_params();
        let masked = vec![
            MaskedValue { party_id: 0, masked_value: 10 },
            MaskedValue { party_id: 1, masked_value: 20 },
            MaskedValue { party_id: 2, masked_value: 30 },
        ];
        let result = aggregate(&masked, &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_secure_sum_random_values() {
        let inputs = vec![123, 456, 789, 1011, 1213];
        let expected: i64 = inputs.iter().sum();
        let result = secure_sum(&inputs, 1_000_007).unwrap();
        assert_eq!(result, expected);
    }
}
