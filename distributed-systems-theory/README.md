# Distributed Systems Theory — A Hands-On Learning Project in Rust

> "All distributed systems theory is about simulating a single reliable machine using multiple unreliable machines."

## Why This Project Exists

Every distributed system you've used — Kafka, etcd, Redis Cluster, PostgreSQL replication, Kubernetes itself — was designed by engineers who deeply understood theoretical foundations. The difference between an operator and an architect is understanding:

1. **WHAT** is provably impossible (so you stop trying to build it)
2. **WHAT** trade-offs are mandatory (so you choose consciously)
3. **WHAT** algorithms exist for each trade-off (so you pick the right one)
4. **WHY** specific systems made specific choices (so you evaluate them)

Without this knowledge, you will:
- Design systems that lose data under network partitions
- Choose consistency models that don't match your requirements
- Implement consensus incorrectly (it's deceptively hard)
- Fail to understand why Kafka chose what it chose
- Build CRDTs that converge incorrectly
- Implement distributed transactions that deadlock under load

## Prerequisites

- **Rust**: 2021 edition, stable toolchain
- **Async Rust**: Familiarity with `tokio`, `Future`, `async/await`
- **Concurrency**: Understanding of `Mutex`, `RwLock`, channels, actor model
- **DevOps**: Experience with Kubernetes, Docker, microservices
- **DSA**: Graph algorithms, hash functions, basic complexity analysis

## Module Overview

| # | Module | Key Concepts | Difficulty |
|---|--------|-------------|------------|
| 01 | Two Generals | Impossibility of reliable communication, idempotency | ★★☆☆☆ |
| 02 | Time & Ordering | Lamport timestamps, vector clocks, causality | ★★★☆☆ |
| 03 | CAP Theorem | Consistency models, PACELC, trade-off analysis | ★★★☆☆ |
| 04 | Consensus | Paxos, Raft, leader election, log replication | ★★★★★ |
| 05 | FLP Impossibility | Failure detectors, partial synchrony | ★★★★☆ |
| 06 | Distributed Transactions | 2PC, 3PC, Saga, transactional outbox | ★★★★☆ |
| 07 | Consistent Hashing | Hash rings, virtual nodes, sharding | ★★★☆☆ |
| 08 | CRDTs | G-Counter, PN-Counter, OR-Set, merge properties | ★★★★☆ |
| 09 | Byzantine Fault Tolerance | PBFT, fault types, blockchain basics | ★★★★★ |
| 10 | Clock Synchronization | HLC, TrueTime, NTP simulation | ★★★★☆ |
| 11 | Integration Exercises | Combining all concepts into working systems | ★★★★★ |
| 12 | Jepsen-Style Tests | Linearizability testing under chaos | ★★★★★ |
| 13 | Capstone: Distributed Database | Full distributed DB with consensus + sharding | ★★★★★ |

## Module Dependency Graph

```
01-two-generals
    ↓
02-time-and-ordering (needs 01 for communication model)
    ↓
03-cap-theorem (needs 02 for ordering concepts)
    ↓
05-flp-impossibility (needs 03 for context)
    ↓
04-consensus (needs 03 + 05 — consensus works AROUND FLP)
    ↓
06-distributed-transactions (needs 04 for 2PC/3PC)
    ↓
07-consistent-hashing (standalone, but needs 03 for sharding theory)
    ↓
08-crdts (needs 02 for vector clocks, 03 for AP systems)
    ↓
09-byzantine-fault-tolerance (needs 04 — extends Raft to Byzantine)
    ↓
10-clock-synchronization (needs 02 — extends logical clocks)
    ↓
11-integration-exercises (needs ALL above)
    ↓
12-jepsen-style-tests (needs 11 — tests the integrated system)
    ↓
13-capstone-distributed-database (needs ALL above)
```

## How to Use This Project

### Sequential Path (Recommended)

Follow the modules in order. Each module builds on the previous ones.

```bash
# Start with module 01
cd 01-two-generals
cargo test

# When all tests pass, move to the next module
cd ../02-time-and-ordering
cargo test
```

### Exercise Format

Each exercise file (`pXX_name.rs`) follows this structure:

```rust
//! # Exercise: [Title]
//!
//! ## Theory
//! [Mathematical definition or algorithm description]
//!
//! ## Proof / Intuition
//! [Why this works, or why it's impossible]
//!
//! ## Implementation Task
//! [Exactly what to build]
//!
//! ## Verification
//! [How to prove the implementation is correct]

// ===== IMPLEMENTATION =====

// ===== TESTS =====
#[cfg(test)]
mod tests {
    // Property-based tests where possible
    // Concurrent tests for distributed behavior
    // Edge case tests for impossibility demonstrations
}
```

### Running Tests

```bash
# All modules
cargo test --workspace

# Single module
cd 04-consensus
cargo test

# Single exercise
cd 04-consensus
cargo test p06_raft_leader_election
```

### Running Benchmarks

```bash
cd 08-crdts
cargo bench
```

## Reference Documents

- [theorems.md](theorems.md) — Formal statements of all theorems and impossibility results
- [consistency-models.md](consistency-models.md) — Consistency model hierarchy with definitions

## Key Papers

| Paper | Module | Priority |
|---|---|---|
| Time, Clocks, and the Ordering of Events (Lamport, 1978) | 02 | CRITICAL |
| Impossibility of Distributed Consensus (Fischer et al., 1985) | 05 | CRITICAL |
| The Part-Time Parliament (Lamport, 1998) — Paxos | 04 | HIGH |
| In Search of an Understandable Consensus Algorithm (Ongaro, 2014) — Raft | 04 | CRITICAL |
| Practical Byzantine Fault Tolerance (Castro, Liskov, 1999) | 09 | HIGH |
| Dynamo: Amazon's Highly Available Key-Value Store (DeCandia, 2007) | 07, 08 | HIGH |
| Conflict-free Replicated Data Types (Shapiro et al., 2011) | 08 | CRITICAL |
| Spanner: Google's Globally-Distributed Database (Corbett, 2012) | 10 | HIGH |

## Technical Constraints

1. Language: Rust (2021 edition, stable toolchain)
2. Async runtime: Tokio (multi-threaded)
3. Networking: `tokio::net` (raw TCP) for inter-node communication
4. Serialization: `bincode` (efficient binary serialization)
5. Testing: `proptest` for property-based tests, `tokio::test` for async tests
6. Benchmarking: `criterion`
7. No external consensus library — implementations from scratch
8. Node simulation: each "node" is a tokio task with its own state
9. Network simulation: controllable channels with configurable delay/loss
10. Clock simulation: injectable clock abstraction (don't use real time in tests)

## Success Criteria

After completing this project, you will be able to:

1. State and prove the intuition behind CAP, FLP, and Byzantine bounds
2. Implement Raft consensus that elects leaders, replicates logs, and maintains safety under failures
3. Implement vector clocks and detect causal ordering and concurrency
4. Implement all major CRDT types and verify their algebraic properties
5. Implement 2PC, 3PC, and Saga with correct failure handling
6. Implement consistent hashing with virtual nodes and analyze load balance
7. Implement Hybrid Logical Clocks and use them for causal consistency
8. Implement PBFT with a Byzantine node and verify correctness
9. Design a distributed system and consciously choose its position in the CAP/PACELC space
10. Build a capstone distributed database combining consensus, replication, sharding, and transactions
11. Write Jepsen-style tests that verify linearizability under network chaos
12. Read and understand the original papers listed above

## License

This project is for educational purposes. Build, break, learn.
