//! # Exercise: Raft vs PBFT Benchmark
//!
//! ## Theory
//!
//! **Raft** and **PBFT** are consensus protocols designed for different fault models:
//!
//! - **Raft** tolerates **crash faults** only. It requires n >= 2f + 1 nodes and
//!   uses a simple leader-based protocol with log replication. Messages are:
//!   RequestVote and AppendEntries. The message complexity per round is O(n).
//!
//! - **PBFT** tolerates **Byzantine faults**. It requires n >= 3f + 1 nodes and
//!   uses a three-phase protocol (Pre-prepare, Prepare, Commit). The message
//!   complexity per round is O(n^2).
//!
//! The key trade-off is safety vs efficiency:
//! - Raft is simpler and faster but cannot handle malicious nodes.
//! - PBFT is more resilient but requires more messages and nodes.
//!
//! ## Proof / Intuition
//!
//! The message complexity difference is fundamental:
//!
//! - In Raft, the leader sends one AppendEntries to each follower, and each
//!   follower responds to the leader. Total: O(n) messages per round.
//!
//! - In PBFT, after the leader sends Pre-prepare (n-1 messages), every node
//!   broadcasts Prepare (n * (n-1) messages), and then every node broadcasts
//!   Commit (n * (n-1) messages). Total: O(n^2) messages per round.
//!
//! For small clusters (n <= 10), the overhead is manageable. For large clusters
//! (n > 100), PBFT's quadratic cost becomes prohibitive, which is why protocols
//! like HotStuff (linear message complexity) were developed.
//!
//! ## Implementation Task
//!
//! Implement:
//!
//! - `RaftRound` struct tracking message counts for one Raft consensus round
//! - `PBFTRound` struct tracking message counts for one PBFT consensus round
//! - `BenchmarkResult` struct with comparison metrics
//! - `benchmark_consensus(n, f)` function that computes message counts for both
//!   protocols and returns a comparison
//! - Methods to calculate total messages, per-node messages, and ratio
//!
//! ## Verification
//!
//! - Verify Raft uses O(n) messages per round
//! - Verify PBFT uses O(n^2) messages per round
//! - Verify the crossover point where PBFT overhead becomes significant
//! - Verify both protocols satisfy their respective fault tolerance bounds

/// The result of benchmarking one consensus round.
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Number of nodes in the system.
    pub n: usize,
    /// Maximum number of Byzantine faults tolerated.
    pub f: usize,
    /// Total messages in one Raft round.
    pub raft_messages: usize,
    /// Total messages in one PBFT round (Pre-prepare + Prepare + Commit).
    pub pbft_messages: usize,
    /// Ratio of PBFT messages to Raft messages.
    pub pbft_raft_ratio: f64,
    /// Raft minimum nodes for f crash faults.
    pub raft_min_nodes: usize,
    /// PBFT minimum nodes for f Byzantine faults.
    pub pbft_min_nodes: usize,
}

/// Count messages in one Raft consensus round.
///
/// Raft uses a leader-based protocol:
/// - Leader sends AppendEntries to all n-1 followers: n-1 messages
/// - Each follower responds to the leader: n-1 messages
/// - Total: 2 * (n - 1) messages
pub fn raft_round_messages(n: usize) -> usize {
    if n < 2 {
        return 0;
    }
    // Leader sends to n-1 followers, each responds
    2 * (n - 1)
}

/// Count messages in one PBFT consensus round.
///
/// PBFT three-phase protocol:
/// - Pre-prepare: leader broadcasts to n-1 replicas: n-1 messages
/// - Prepare: each node broadcasts to n-1 others: n * (n-1) messages
/// - Commit: each node broadcasts to n-1 others: n * (n-1) messages
/// - Total: (n-1) + 2 * n * (n-1) = (n-1) * (2n + 1) messages
pub fn pbft_round_messages(n: usize) -> usize {
    if n < 2 {
        return 0;
    }
    let pre_prepare = n - 1;
    let prepare = n * (n - 1);
    let commit = n * (n - 1);
    pre_prepare + prepare + commit
}

/// Benchmark both consensus protocols for a given system configuration.
///
/// Returns a `BenchmarkResult` comparing the message complexity of Raft and PBFT.
pub fn benchmark_consensus(n: usize, f: usize) -> BenchmarkResult {
    assert!(n >= 2, "need at least 2 nodes");

    let raft_min = 2 * f + 1;
    let pbft_min = 3 * f + 1;

    let raft_msgs = raft_round_messages(n);
    let pbft_msgs = pbft_round_messages(n);
    let ratio = if raft_msgs > 0 {
        pbft_msgs as f64 / raft_msgs as f64
    } else {
        0.0
    };

    BenchmarkResult {
        n,
        f,
        raft_messages: raft_msgs,
        pbft_messages: pbft_msgs,
        pbft_raft_ratio: ratio,
        raft_min_nodes: raft_min,
        pbft_min_nodes: pbft_min,
    }
}

/// Calculate per-node outgoing message count for Raft.
///
/// In Raft, the leader sends n-1 AppendEntries messages.
/// Each follower sends 1 response message.
pub fn raft_per_node_messages(n: usize) -> (usize, usize) {
    // (leader_outgoing, follower_outgoing)
    if n < 2 {
        return (0, 0);
    }
    (n - 1, 1)
}

/// Calculate per-node message count for PBFT.
///
/// In PBFT, each node broadcasts to all others in Prepare and Commit phases.
/// The leader also sends Pre-prepare.
pub fn pbft_per_node_messages(n: usize) -> (usize, usize) {
    // (leader_count, replica_count)
    if n < 2 {
        return (0, 0);
    }
    let leader = (n - 1) + 2 * (n - 1); // pre_prepare + prepare + commit
    let replica = 2 * (n - 1); // prepare + commit (no pre_prepare for replicas)
    (leader, replica)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raft_uses_linear_messages() {
        // Raft: 2 * (n - 1) messages per round
        assert_eq!(raft_round_messages(2), 2);
        assert_eq!(raft_round_messages(4), 6);
        assert_eq!(raft_round_messages(10), 18);
        assert_eq!(raft_round_messages(100), 198);
    }

    #[test]
    fn pbft_uses_quadratic_messages() {
        // PBFT: (n-1) + n*(n-1) + n*(n-1) messages per round
        assert_eq!(pbft_round_messages(4), 3 + 12 + 12, "4 nodes: 3+12+12=27");
        assert_eq!(pbft_round_messages(7), 6 + 42 + 42, "7 nodes: 6+42+42=90");
    }

    #[test]
    fn pbft_overhead_grows_quadratically() {
        let result4 = benchmark_consensus(4, 1);
        let result10 = benchmark_consensus(10, 3);
        let result20 = benchmark_consensus(20, 6);

        // The ratio should increase as n grows
        assert!(
            result10.pbft_raft_ratio > result4.pbft_raft_ratio,
            "PBFT overhead ratio should increase with n: {} > {}",
            result10.pbft_raft_ratio,
            result4.pbft_raft_ratio
        );
        assert!(
            result20.pbft_raft_ratio > result10.pbft_raft_ratio,
            "PBFT overhead ratio should increase further: {} > {}",
            result20.pbft_raft_ratio,
            result10.pbft_raft_ratio
        );
    }

    #[test]
    fn fault_tolerance_bounds() {
        let result = benchmark_consensus(7, 2);
        assert_eq!(result.raft_min_nodes, 5, "Raft needs 2f+1 = 5 for f=2");
        assert_eq!(result.pbft_min_nodes, 7, "PBFT needs 3f+1 = 7 for f=2");

        let result = benchmark_consensus(10, 1);
        assert_eq!(result.raft_min_nodes, 3, "Raft needs 2f+1 = 3 for f=1");
        assert_eq!(result.pbft_min_nodes, 4, "PBFT needs 3f+1 = 4 for f=1");
    }

    #[test]
    fn per_node_counts_consistent() {
        let (raft_leader, raft_follower) = raft_per_node_messages(4);
        let raft_total = raft_leader + 3 * raft_follower;
        assert_eq!(
            raft_total,
            raft_round_messages(4),
            "per-node counts should sum to total"
        );

        let (pbft_leader, pbft_replica) = pbft_per_node_messages(4);
        let pbft_total = pbft_leader + 3 * pbft_replica;
        assert_eq!(
            pbft_total,
            pbft_round_messages(4),
            "per-node counts should sum to total"
        );
    }
}
