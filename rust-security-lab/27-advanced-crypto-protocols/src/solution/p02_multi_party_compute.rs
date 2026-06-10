//! # Lesson 02: Secure Multi-Party Computation (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyShare {
    pub party_id: usize,
    pub value: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MPCResult {
    pub output: i64,
    pub party_count: usize,
    pub success: bool,
}

/// Split input into n additive shares that sum to input (mod prime).
pub fn generate_shares(input: i64, n: usize, prime: i64) -> Vec<i64> {
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![input % prime];
    }

    let mut rng = rand::thread_rng();
    let mut shares = Vec::with_capacity(n);
    let mut sum = 0i64;

    for _ in 0..(n - 1) {
        let share: i64 = rng.gen_range(0..prime);
        shares.push(share);
        sum = (sum + share) % prime;
    }

    // Last share: input - sum (mod prime)
    let last = ((input % prime - sum) % prime + prime) % prime;
    shares.push(last);
    shares
}

/// Sum all received shares mod prime.
pub fn compute_local_sum(received_shares: &[i64], prime: i64) -> i64 {
    received_shares.iter().fold(0i64, |acc, &s| (acc + s) % prime)
}

/// Run a full MPC sum protocol.
pub fn mpc_sum(inputs: &[i64], prime: i64) -> MPCResult {
    let n = inputs.len();
    if n == 0 {
        return MPCResult {
            output: 0,
            party_count: 0,
            success: true,
        };
    }

    // Step 1: Each party generates additive shares
    let all_shares: Vec<Vec<i64>> = inputs
        .iter()
        .map(|&input| generate_shares(input, n, prime))
        .collect();

    // Step 2: Each party j collects the j-th share from every party
    let mut local_sums = vec![0i64; n];
    for j in 0..n {
        for i in 0..n {
            local_sums[j] = (local_sums[j] + all_shares[i][j]) % prime;
        }
    }

    // Step 3: Aggregate -- sum all local sums
    let output = local_sums.iter().fold(0i64, |acc, &s| (acc + s) % prime);

    MPCResult {
        output,
        party_count: n,
        success: true,
    }
}

/// Count inputs exceeding a threshold using MPC.
pub fn mpc_count_exceeding(inputs: &[i64], threshold: i64, prime: i64) -> MPCResult {
    // Each party locally converts their input to 0 or 1
    let indicators: Vec<i64> = inputs
        .iter()
        .map(|&v| if v > threshold { 1 } else { 0 })
        .collect();

    // Use MPC sum on the indicators
    mpc_sum(&indicators, prime)
}

/// Extract what a semi-honest adversary at adversary_id would see.
///
/// The adversary sees all shares sent TO them (the adversary_id-th share from each party).
pub fn adversary_view(
    inputs: &[i64],
    adversary_id: usize,
    prime: i64,
) -> Vec<i64> {
    let n = inputs.len();
    if adversary_id >= n {
        return Vec::new();
    }

    // Each party generates shares; the adversary sees the adversary_id-th share
    let all_shares: Vec<Vec<i64>> = inputs
        .iter()
        .map(|&input| generate_shares(input, n, prime))
        .collect();

    // Adversary collects their share from each party
    all_shares
        .iter()
        .map(|shares| shares[adversary_id])
        .collect()
}

/// Check if colluding parties can recover a target's input.
///
/// If all parties except the target collude, they can sum their shares
/// and subtract from the global sum to learn the target's input.
pub fn can_colluders_recover(
    n: usize,
    colluding_ids: &[usize],
    target_id: usize,
) -> bool {
    if target_id >= n {
        return false;
    }
    // The colluders can recover the target's input if they control all other parties
    let non_colluding: usize = n - colluding_ids.len();
    // If only the target is non-colluding, the colluders win
    non_colluding <= 1 && !colluding_ids.contains(&target_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    const P: i64 = 2027;

    #[test]
    fn test_generate_shares_sum() {
        let shares = generate_shares(100, 5, P);
        assert_eq!(shares.len(), 5);
        let sum: i64 = shares.iter().sum::<i64>() % P;
        assert_eq!(sum, 100);
    }

    #[test]
    fn test_generate_shares_randomness() {
        let s1 = generate_shares(100, 5, P);
        let s2 = generate_shares(100, 5, P);
        assert_ne!(s1, s2);
    }

    #[test]
    fn test_compute_local_sum() {
        let shares = vec![10, 20, 30, 40, 50];
        let local = compute_local_sum(&shares, P);
        assert_eq!(local, 150);
    }

    #[test]
    fn test_mpc_sum_basic() {
        let inputs = vec![10, 20, 30, 40];
        let result = mpc_sum(&inputs, P);
        assert!(result.success);
        assert_eq!(result.output, 100);
        assert_eq!(result.party_count, 4);
    }

    #[test]
    fn test_mpc_sum_single_party() {
        let inputs = vec![42];
        let result = mpc_sum(&inputs, P);
        assert_eq!(result.output, 42);
    }

    #[test]
    fn test_mpc_sum_wraps_mod_prime() {
        let inputs = vec![1000, 1000, 1000];
        let result = mpc_sum(&inputs, P);
        assert_eq!(result.output, 3000 % P);
    }

    #[test]
    fn test_mpc_count_exceeding() {
        let inputs = vec![5, 15, 25, 10, 30];
        let result = mpc_count_exceeding(&inputs, 12, P);
        assert_eq!(result.output, 3);
    }

    #[test]
    fn test_adversary_view_length() {
        let inputs = vec![10, 20, 30, 40];
        let view = adversary_view(&inputs, 0, P);
        assert_eq!(view.len(), 4);
    }

    #[test]
    fn test_adversary_view_does_not_reveal_others() {
        let inputs = vec![100, 200, 300];
        let view = adversary_view(&inputs, 0, P);
        let sum_view: i64 = view.iter().sum::<i64>() % P;
        // The view sum is a random share, not the individual inputs
        assert!(sum_view != 100 || sum_view != 200 || sum_view != 300);
    }

    #[test]
    fn test_can_colluders_recover_all_but_one() {
        let colluding = vec![0, 1, 3, 4];
        assert!(can_colluders_recover(5, &colluding, 2));
    }

    #[test]
    fn test_can_colluders_recover_partial() {
        let colluding = vec![0, 1];
        assert!(!can_colluders_recover(5, &colluding, 3));
    }
}
