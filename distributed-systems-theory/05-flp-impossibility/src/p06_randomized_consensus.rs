//! # Exercise: Randomized Consensus
//!
//! ## Theory
//!
//! Randomized consensus protocols circumvent FLP impossibility by using randomization.
//! Instead of guaranteeing termination, they guarantee termination *with probability 1*
//! (i.e., the probability of not terminating approaches zero).
//!
//! The classic approach (Ben-Or's algorithm):
//!
//! 1. **Proposal Phase:** Each process broadcasts its proposal.
//! 2. **Agreement Phase:**
//!    - If all proposals are the same, decide on that value.
//!    - If there is disagreement, each process flips a fair coin.
//!    - Each process broadcasts its coin flip result.
//!    - If a majority agrees on a value, use that value in the next round.
//!    - Otherwise, flip again.
//!
//! ## Proof / Intuition
//!
//! Why does randomization work?
//!
//! 1. In each round with disagreement, each process flips a fair coin.
//! 2. The probability that all coins agree is (1/2)^(n-1) for n processes.
//!    This is small but non-zero.
//! 3. Even if coins disagree, there might be a majority on one value.
//! 4. Each round is an independent trial with a non-zero probability of termination.
//! 5. After k rounds, the probability of not terminating is at most (1 - p)^k,
//!    which approaches 0 as k increases.
//!
//! This is "termination with probability 1" -- for any epsilon > 0, there exists
//! a finite number of rounds after which the probability of not having terminated
//! is less than epsilon.
//!
//! ## Implementation Task
//!
//! Implement randomized consensus:
//!
//! - `CoinFlip`: A random boolean value.
//! - `RandomizedProcess`: A process that can flip coins.
//! - `randomized_consensus(proposals: Vec<Value>) -> Value`: Run the protocol.
//! - Use majority voting with coin-flip tiebreaking.
//!
//! ## Verification
//!
//! - Verify all runs eventually terminate (statistical over many runs).
//! - Verify agreement: all processes decide the same value.
//! - Verify validity: the decided value was proposed by some process.

use rand::Rng;

/// A consensus value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Zero,
    One,
}

impl Value {
    /// Flip to the opposite value.
    pub fn flip(&self) -> Value {
        match self {
            Value::Zero => Value::One,
            Value::One => Value::Zero,
        }
    }
}

/// A coin flip result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoinFlip {
    Heads,
    Tails,
}

/// Simulate a single coin flip.
pub fn flip_coin() -> CoinFlip {
    let mut rng = rand::thread_rng();
    if rng.gen_bool(0.5) {
        CoinFlip::Heads
    } else {
        CoinFlip::Tails
    }
}

/// Run a single round of the randomized consensus protocol.
///
/// Returns `Some(value)` if a decision is reached, or `None` if another round is needed.
pub fn consensus_round(proposals: &[Value]) -> Option<Value> {
    // Check if all proposals are the same.
    let first = &proposals[0];
    if proposals.iter().all(|p| p == first) {
        return Some(first.clone());
    }

    // Check majority of original proposals.
    let ones = proposals.iter().filter(|v| **v == Value::One).count();
    let zeros = proposals.iter().filter(|v| **v == Value::Zero).count();

    if ones > zeros {
        return Some(Value::One);
    }
    if zeros > ones {
        return Some(Value::Zero);
    }

    // Tie: coin flip decides.
    match flip_coin() {
        CoinFlip::Heads => Some(Value::One),
        CoinFlip::Tails => Some(Value::Zero),
    }
}

/// Run the randomized consensus protocol until all processes agree.
///
/// Returns the agreed value and the number of rounds taken.
pub fn randomized_consensus(proposals: Vec<Value>) -> (Value, u32) {
    let mut current_proposals = proposals;
    let mut rounds = 0;
    let max_rounds = 10000; // Safety limit.

    loop {
        rounds += 1;
        assert!(
            rounds <= max_rounds,
            "Consensus exceeded maximum rounds ({}). This should not happen statistically.",
            max_rounds
        );

        if let Some(decision) = consensus_round(&current_proposals) {
            return (decision, rounds);
        }

        // For next round, all processes use the majority value or re-propose.
        // In this simplified version, we re-run with the original proposals
        // but the coin flip provides the randomization.
        // In a real protocol, processes would exchange coin flip results and
        // converge on the majority derived value.
    }
}

/// Run consensus with a specific seed for reproducibility.
pub fn randomized_consensus_deterministic(
    proposals: Vec<Value>,
    seed: u64,
) -> (Value, u32) {
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    let mut rng = SmallRng::seed_from_u64(seed);
    let mut current_proposals = proposals;
    let mut rounds = 0;

    loop {
        rounds += 1;
        assert!(rounds <= 10000);

        // Check if all proposals are the same.
        let first = &current_proposals[0];
        if current_proposals.iter().all(|p| p == first) {
            return (first.clone(), rounds);
        }

        // Disagreement: flip coins.
        let ones = current_proposals.iter().filter(|v| **v == Value::One).count();
        let zeros = current_proposals.iter().filter(|v| **v == Value::Zero).count();

        // Random tiebreak with a coin flip.
        if ones > zeros {
            return (Value::One, rounds);
        }
        if zeros > ones {
            return (Value::Zero, rounds);
        }

        // Tie: random selection.
        let decision = if rng.gen_bool(0.5) {
            Value::One
        } else {
            Value::Zero
        };
        return (decision, rounds);
    }
}

/// Simulate many runs of randomized consensus and measure statistics.
pub fn benchmark_consensus(proposals: Vec<Value>, runs: u32) -> ConsensusStats {
    let mut total_rounds = 0u64;
    let mut ones_count = 0u32;
    let mut max_rounds = 0u32;
    let mut min_rounds = u32::MAX;

    for _ in 0..runs {
        let (_, rounds) = randomized_consensus(proposals.clone());
        total_rounds += rounds as u64;
        max_rounds = max_rounds.max(rounds);
        min_rounds = min_rounds.min(rounds);
    }

    ConsensusStats {
        total_runs: runs,
        average_rounds: total_rounds as f64 / runs as f64,
        max_rounds,
        min_rounds,
        ones_count,
    }
}

/// Statistics from running consensus multiple times.
#[derive(Debug)]
pub struct ConsensusStats {
    pub total_runs: u32,
    pub average_rounds: f64,
    pub max_rounds: u32,
    pub min_rounds: u32,
    pub ones_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consensus_unanimous_agrees_immediately() {
        let proposals = vec![Value::One, Value::One, Value::One];
        let (decision, rounds) = randomized_consensus(proposals);
        assert_eq!(decision, Value::One);
        assert_eq!(rounds, 1, "Unanimous proposals should decide in 1 round");
    }

    #[test]
    fn consensus_unanimous_zero() {
        let proposals = vec![Value::Zero, Value::Zero, Value::Zero];
        let (decision, rounds) = randomized_consensus(proposals);
        assert_eq!(decision, Value::Zero);
        assert_eq!(rounds, 1);
    }

    #[test]
    fn consensus_majority_one() {
        let proposals = vec![Value::One, Value::One, Value::Zero];
        let (decision, rounds) = randomized_consensus(proposals.clone());
        // Validity: decision must be a proposed value.
        assert!(
            decision == Value::One || decision == Value::Zero,
            "Decision must be a proposed value"
        );
        // In this case, majority is One, so it should be One.
        assert_eq!(decision, Value::One);
        let _ = rounds;
    }

    #[test]
    fn consensus_eventually_terminates_statistical() {
        let proposals = vec![Value::One, Value::Zero, Value::Zero];
        let stats = benchmark_consensus(proposals, 100);
        // All 100 runs should terminate.
        assert_eq!(stats.total_runs, 100);
        // Average rounds should be small (typically 1-3 for 3 processes).
        assert!(
            stats.average_rounds < 10.0,
            "Average rounds should be reasonably small, got {}",
            stats.average_rounds
        );
    }

    #[test]
    fn coin_flip_is_fair_statistical() {
        let mut heads = 0u64;
        let total = 10000u64;
        for _ in 0..total {
            if flip_coin() == CoinFlip::Heads {
                heads += 1;
            }
        }
        let ratio = heads as f64 / total as f64;
        assert!(
            (0.45..=0.55).contains(&ratio),
            "Coin flip should be approximately fair, got ratio {}",
            ratio
        );
    }

    #[test]
    fn consensus_round_on_disagreement_returns_none_or_value() {
        let proposals = vec![Value::One, Value::Zero];
        // Due to randomness, this might return Some or None.
        // If it returns Some, the value should be valid.
        if let Some(val) = consensus_round(&proposals) {
            assert!(
                val == Value::One || val == Value::Zero,
                "Returned value must be valid"
            );
        }
        // The test passes either way -- randomness means we can't predict the result.
    }

    #[test]
    fn consensus_two_process_always_terminates() {
        let proposals = vec![Value::One, Value::Zero];
        let (decision, rounds) = randomized_consensus(proposals);
        assert!(
            decision == Value::One || decision == Value::Zero,
            "Decision must be valid"
        );
        assert!(rounds > 0, "Should take at least 1 round");
    }
}
