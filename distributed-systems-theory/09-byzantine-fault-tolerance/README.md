# Module 09: Byzantine Fault Tolerance

## Historical Context

The Byzantine Fault Tolerance (BFT) problem originates from a thought experiment
posed by Lamport, Shostak, and Pease in their landmark 1982 paper *"The Byzantine
Generals Problem"*. The problem asks: how can a group of generals, some of whom
may be traitors sending conflicting messages, agree on a common battle plan?

The practical motivation came from the need to build reliable systems from unreliable
components -- not just components that crash silently, but components that actively
lie. This is the strongest possible failure model in distributed systems.

Key milestones in BFT research:

- **1982**: Lamport, Shostak, and Pease prove that n >= 3f + 1 nodes are needed
  to tolerate f Byzantine faults, and present the Oral Messages (OM) algorithm for
  signed and unsigned messages.
- **1999**: Castro and Liskov introduce **Practical Byzantine Fault Tolerance (PBFT)**,
  the first protocol to make BFT practical for real systems by reducing the message
  complexity from exponential to O(n^2) per consensus round.
- **2014**: Yin et al. introduce **HotStuff**, a BFT protocol with linear message
  complexity per view change, adopted by Facebook's LibraBFT (later DiemBFT).
- **2014**: Buchman et al. develop **Tendermint**, bringing BFT to blockchain consensus
  in the Cosmos ecosystem.
- **2018+**: BFT protocols become the backbone of permissioned blockchains (Hyperledger
  Fabric, Facebook Diem) and proof-of-stake systems (Ethereum 2.0 Casper FFG).

## Crash Faults vs Byzantine Faults

Understanding the distinction between fault models is fundamental:

### Crash Faults

A **crash fault** occurs when a node simply stops functioning. The node:
- Stops sending and receiving messages
- Does not corrupt data or lie to other nodes
- May or may not have persisted its state before crashing

Protocols like **Paxos** and **Raft** are designed to tolerate crash faults. They
require only n >= 2f + 1 nodes to tolerate f crash failures. This is because a
crashed node is silent -- honest nodes can detect the absence of messages.

### Byzantine Faults

A **Byzantine fault** is the most general and dangerous failure model. A Byzantine
node can:
- Send conflicting messages to different nodes
- Send fabricated messages
- Refuse to send messages (crash)
- Delay messages selectively
- Collude with other Byzantine nodes

A Byzantine node is indistinguishable from a non-Byzantine node until its lies are
detected through protocol-level redundancy.

### The 3f + 1 Bound

The fundamental result is:

> A system with n nodes can tolerate at most f Byzantine faults if and only if
> **n >= 3f + 1**.

This is strictly stronger than the crash-fault bound of n >= 2f + 1. The extra
f + 1 nodes provide the redundancy needed to cross-check messages and detect lies.

**Why 3f + 1 and not 2f + 1?**

With 2f + 1 nodes and f Byzantine nodes, the honest nodes (f + 1 of them) are
outnumbered in the worst case. A Byzantine node can tell each honest node a different
value, and honest nodes have no way to determine which value is correct without
additional redundancy. With 3f + 1 nodes, the 2f + 1 honest nodes can form a
quorum that always overlaps in at least f + 1 honest members, enabling agreement.

### Fault Tolerance Bounds Comparison

| Fault Model | Minimum Nodes (f=1) | Minimum Nodes (f=2) | Protocol Examples |
|-------------|--------------------|--------------------|-------------------|
| Crash       | 3                  | 5                  | Paxos, Raft       |
| Byzantine   | 4                  | 7                  | PBFT, HotStuff    |

## PBFT (Practical Byzantine Fault Tolerance)

Castro and Liskov's PBFT (1999) was a breakthrough that made Byzantine fault
tolerance practical. Before PBFT, BFT algorithms had exponential message complexity,
making them unusable beyond a handful of nodes.

### Protocol Overview

PBFT operates in a **view-based** system where one node is designated as the leader
(primary) in each view. The protocol proceeds in three phases:

### Pre-prepare Phase

1. The **leader** receives a client request and assigns it a sequence number.
2. The leader broadcasts a **PRE-PREPARE** message containing the view number,
   sequence number, and a digest of the request.
3. Each **replica** validates the pre-prepare:
   - Is the view number current?
   - Is the message from the expected leader?
   - Has this sequence number not been used for a different request?
   - Is the request digest valid?

### Prepare Phase

4. Upon accepting a pre-prepare, each replica broadcasts a **PREPARE** message.
5. A replica considers a request **prepared** when it has received:
   - One valid pre-prepare from the leader, AND
   - 2f + 1 matching prepare messages (including its own)
6. The 2f + 1 threshold ensures that at least f + 1 honest nodes have accepted
   the same request and sequence number.

### Commit Phase

7. Once prepared, each replica broadcasts a **COMMIT** message.
8. A replica considers a request **committed** when it has received 2f + 1 commit
   messages (including its own).
9. After committing, the replica **executes** the request and returns the result
   to the client.

### Why 3 Phases?

The pre-prepare and prepare phases together establish **agreement**: they ensure that
all honest replicas process requests in the same order. The commit phase ensures
**liveness**: it guarantees that enough replicas have agreed before any replica
executes the request.

### View Change

If the leader is suspected of being faulty (e.g., requests are not being processed),
replicas trigger a **view change**:

1. A replica broadcasts a **VIEW-CHANGE** message to all nodes, incrementing the view.
2. The new leader collects 2f + 1 view-change messages.
3. The new leader broadcasts a **NEW-VIEW** message summarizing any uncommitted
   requests from the previous view.
4. The system resumes with the new leader.

View changes ensure progress even when the leader is Byzantine, at the cost of
increased message complexity.

### Message Complexity

PBFT has O(n^2) message complexity per consensus round. For n = 4 nodes with
f = 1, each round requires at most:
- 1 pre-prepare (leader to all)
- 3 prepare messages (each replica to all)
- 3 commit messages (each replica to all)
= 7 messages total per round.

For larger n, the quadratic growth becomes a bottleneck, which motivated
HotStuff's linear-complexity design.

## Modern BFT Protocols

### HotStuff (2014)

HotStuff addresses PBFT's quadratic message complexity by using a **rotating leader**
and **pipelining**:

- **Linear view change**: Each view change requires only O(n) messages (each node
  sends to the new leader only).
- **Three-phase pipeline**: Prepare -> Pre-commit -> Commit, with each phase
  requiring a simple broadcast from the leader.
- **Leader-driven**: The leader collects votes (signatures) and aggregates them,
  reducing broadcast overhead.
- **Trade-off**: Requires a reliable broadcast channel (e.g., TLS) and a
  leader-rotation schedule.

HotStuff is used in Meta's (formerly Facebook) Diem blockchain (now defunct) and
has influenced many modern BFT protocols.

### Tendermint (2014)

Tendermint, developed by Buchman, Kwon, and Milosevic, brings BFT consensus to
blockchain:

- **Round-based**: Each height (block) may go through multiple rounds if the leader
  fails.
- **Two-phase**: Propose -> Prevote -> Precommit.
- **Deterministic finality**: Once a block is committed, it is never reverted
  (unlike Bitcoin's probabilistic finality).
- **Governance**: The validator set is managed through on-chain governance.
- **Used in**: Cosmos SDK chains, many proof-of-stake blockchains.

### PBFT vs HotStuff vs Tendermint

| Feature            | PBFT             | HotStuff           | Tendermint         |
|--------------------|------------------|--------------------|--------------------|
| Message complexity | O(n^2)           | O(n) per phase     | O(n^2)             |
| View change        | O(n^2)           | O(n)               | O(n^2)             |
| Finality           | Immediate        | Immediate          | Immediate          |
| Pipelining         | No               | Yes                | Yes                |
| Leader election    | View-based       | Rotating           | Round-robin        |
| Use case           | Permissioned     | Permissioned/PoS   | Public blockchain  |

## Blockchain as BFT

Blockchain consensus is fundamentally a BFT problem:

- **Bitcoin (Nakamoto Consensus)**: Uses proof-of-work to achieve probabilistic
  BFT. Tolerates up to 50% Byzantine power (by hash rate). Finality is probabilistic
  -- a block becomes more irreversible as more blocks are built on top of it.
- **Ethereum 2.0 (Casper FFG)**: Uses BFT-style voting with 2/3 supermajority.
  Validators vote on checkpoint blocks. Tolerates up to 1/3 Byzantine stake.
- **Hyperledger Fabric**: Uses PBFT (and later Raft for crash tolerance) for its
  ordering service.
- **Cosmos**: Uses Tendermint for BFT consensus among validators.

The connection between BFT and blockchain is direct: a blockchain is a replicated
log where all honest nodes must agree on the order of transactions. This is
exactly the Byzantine agreement problem applied to a transaction ledger.

## Exercises

| # | Exercise | Difficulty | Description |
|---|----------|------------|-------------|
| 01 | Fault Types | ★★☆☆☆ | Define and simulate crash, omission, and Byzantine faults |
| 02 | PBFT Roles | ★★☆☆☆ | Define PBFT node roles and message types |
| 03 | PBFT Pre-prepare | ★★★☆☆ | Implement the pre-prepare phase with leader validation |
| 04 | PBFT Prepare | ★★★☆☆ | Implement the prepare phase with 2f+1 threshold |
| 05 | PBFT Commit | ★★★★☆ | Implement the full 3-phase commit flow with execution |
| 06 | PBFT View Change | ★★★★☆ | Implement view change protocol for faulty leader |
| 07 | Byzantine Node Sim | ★★★★☆ | Simulate Byzantine nodes in a running PBFT system |
| 08 | Impossibility: 3 Nodes | ★★★☆☆ | Prove that 3 nodes cannot tolerate 1 Byzantine fault |
| 09 | Raft vs PBFT Benchmark | ★★★★★ | Compare crash-fault and Byzantine-fault protocols |
| 10 | Mini Blockchain | ★★★★★ | Build a simplified blockchain with BFT consensus |

## Learning Goals

1. Distinguish between crash faults and Byzantine faults, and understand why
   Byzantine tolerance requires strictly more nodes.
2. Implement the three phases of PBFT (pre-prepare, prepare, commit) and
   understand the role of each.
3. Understand the 2f + 1 quorum threshold and why it guarantees agreement.
4. Implement a view change protocol that ensures liveness when the leader fails.
5. Understand the trade-offs between PBFT, HotStuff, and Tendermint.
6. Connect Byzantine fault tolerance to real-world blockchain consensus.

## Running

```bash
cargo test
cargo test -- --nocapture  # to see println/tracing output
```

## References

- Lamport, L., Shostak, R., & Pease, M. (1982). *The Byzantine Generals Problem*.
  ACM Transactions on Programming Languages and Systems, 4(3), 382-401.
- Castro, M., & Liskov, B. (1999). *Practical Byzantine Fault Tolerance*. In
  Proceedings of the Third Symposium on Operating Systems Design and Implementation
  (OSDI '99), 173-186.
- Lamport, L. (1998). *The Part-Time Parliament*. ACM Transactions on Computer
  Systems, 16(2), 133-169.
- Yin, M., Malkhi, D., Reiter, M. K., Gueta, G. G., & Abraham, I. (2019).
  *HotStuff: BFT Consensus with Linearity and Responsiveness*. In Proceedings of
  the 2019 ACM Symposium on Principles of Distributed Computing (PODC '19), 347-356.
- Buchman, E., Kwon, J., & Milosevic, Z. (2018). *The latest gossip on BFT
  consensus*. arXiv preprint arXiv:1807.04938.
- Dwork, C., Lynch, N., & Stockmeyer, L. (1988). *Consensus in the presence of
  partial synchrony*. Journal of the ACM, 35(2), 288-323.
- Pease, M., Shostak, R., & Lamport, L. (1980). *Reaching agreement in the
  presence of faults*. Journal of the ACM, 27(2), 228-234.
