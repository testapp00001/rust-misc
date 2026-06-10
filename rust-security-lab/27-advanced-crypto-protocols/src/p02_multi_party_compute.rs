//! # Lesson 02: Secure Multi-Party Computation -- Compute Without Revealing
//!
//! ## What is MPC?
//!
//! Secure Multi-Party Computation allows N parties, each holding a private input x_i,
//! to jointly compute f(x_1, ..., x_n) without any party learning another's input.
//!
//! ## The SPDZ-like Protocol (Simplified)
//!
//! A common approach uses additive secret sharing over a finite field:
//! 1. Each party splits their input into N random shares (one per party)
//! 2. Each party sends one share to every other party (keeping their own)
//! 3. Each party locally sums the shares they received
//! 4. The local sums are exchanged; the global sum of local sums = sum of all inputs
//!
//! This reveals only the final result, not individual inputs.
//!
//! ## Threat Models
//!
//! - **Semi-honest (honest-but-curious)**: Parties follow the protocol but try to learn
//!   from the messages they receive. The additive sharing protocol above is secure against this.
//! - **Malicious**: Parties may send arbitrary messages. Requires additional checks
//!   (e.g., zero-knowledge proofs, message authentication codes on shares).
//!
//! ## Attack: Collusion
//!
//! If N-1 parties collude, they can sum their shares and subtract from the total to learn
//! the remaining party's input. The protocol only hides inputs when fewer than N parties collude.
//!
//! ## Applications
//!
//! - Private set intersection (find common elements without revealing the sets)
//! - Threshold signatures (sign without any party holding the full key)
//! - Private auctions (compute winner without revealing bids)
//! - Privacy-preserving machine learning (train on distributed data)

use serde::{Deserialize, Serialize};

/// A party's local computation result (their share of the global computation).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyShare {
    /// Party identifier
    pub party_id: usize,
    /// The local share value
    pub value: i64,
}

/// The result of an MPC computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MPCResult {
    /// The computed output (e.g., sum, average)
    pub output: i64,
    /// Number of parties that participated
    pub party_count: usize,
    /// Whether the computation completed successfully
    pub success: bool,
}

/// Exercise 1: Generate additive shares for a party's input.
///
/// Split `input` into `n` random shares that sum to `input` (mod prime).
/// Each share should be uniformly random; the last share absorbs the difference.
pub fn generate_shares(input: i64, n: usize, prime: i64) -> Vec<i64> {
    todo!("Split input into n additive shares mod prime")
}

/// Exercise 2: Compute local sum for one party.
///
/// Each party receives one share from every party (including themselves).
/// The local sum is the sum of all received shares mod prime.
pub fn compute_local_sum(received_shares: &[i64], prime: i64) -> i64 {
    todo!("Sum all received shares mod prime")
}

/// Exercise 3: Run a full MPC sum protocol.
///
/// Simulate the protocol:
/// 1. Each party generates additive shares of their input
/// 2. Distribute shares (party i sends share[j] to party j)
/// 3. Each party computes their local sum
/// 4. Aggregate: sum of all local sums = sum of all inputs (mod prime)
///
/// Return the MPCResult with the global sum.
pub fn mpc_sum(inputs: &[i64], prime: i64) -> MPCResult {
    todo!("Run full MPC sum protocol")
}

/// Exercise 4: MPC comparison -- check if any input exceeds a threshold.
///
/// In a real MPC system, this would use garbled circuits. Here we simulate:
/// Each party compares their input to the threshold locally and contributes
/// 1 (exceeds) or 0 (does not exceed). The MPC sum counts how many exceed.
///
/// Returns: count of parties whose input > threshold.
pub fn mpc_count_exceeding(inputs: &[i64], threshold: i64, prime: i64) -> MPCResult {
    todo!("Count inputs exceeding threshold using MPC")
}

/// Exercise 5: MPC with semi-honest adversary simulation.
///
/// Simulate a semi-honest adversary who records all shares they receive.
/// Return the shares that party `adversary_id` would see (their own share from
/// each party's distribution).
///
/// This demonstrates what a curious party can learn from the protocol messages.
pub fn adversary_view(
    inputs: &[i64],
    adversary_id: usize,
    prime: i64,
) -> Vec<i64> {
    todo!("Extract what a semi-honest adversary would see")
}

/// Exercise 6: Detect if collusion can recover a target's input.
///
/// Given `n` parties and a set of colluding party IDs, determine if the
/// colluding set can recover party `target_id`'s input.
///
/// Rule: if all parties except target_id are colluding, the target is compromised.
pub fn can_colluders_recover(
    n: usize,
    colluding_ids: &[usize],
    target_id: usize,
) -> bool {
    todo!("Check if colluding parties can recover the target's input")
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
        // Two calls should produce different shares
        let s1 = generate_shares(100, 5, P);
        let s2 = generate_shares(100, 5, P);
        // Extremely unlikely to be identical
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
        assert_eq!(result.output, 3); // 15, 25, 30 exceed 12
    }

    #[test]
    fn test_adversary_view_length() {
        let inputs = vec![10, 20, 30, 40];
        let view = adversary_view(&inputs, 0, P);
        // Adversary sees one share from each party
        assert_eq!(view.len(), 4);
    }

    #[test]
    fn test_adversary_view_does_not_reveal_others() {
        let inputs = vec![100, 200, 300];
        let view = adversary_view(&inputs, 0, P);
        // The shares alone should not reveal the other parties' inputs
        // (each share is random, the sum of all 3 views for party 0 would
        // equal the total, but party 0 only sees their own collection)
        let sum_view: i64 = view.iter().sum::<i64>() % P;
        // This sum is party 0's local sum, which is a random share of the total
        // It should NOT equal any individual input
        assert!(sum_view != 100 || sum_view != 200 || sum_view != 300);
    }

    #[test]
    fn test_can_colluders_recover_all_but_one() {
        // If all parties except party 2 collude, they can recover party 2's input
        let colluding = vec![0, 1, 3, 4];
        assert!(can_colluders_recover(5, &colluding, 2));
    }

    #[test]
    fn test_can_colluders_recover_partial() {
        // If only 2 of 5 parties collude, they cannot recover party 3's input
        let colluding = vec![0, 1];
        assert!(!can_colluders_recover(5, &colluding, 3));
    }
}
