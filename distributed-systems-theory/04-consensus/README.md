# Module 04: Consensus

## Overview

This module covers the consensus problem -- one of the most fundamental
challenges in distributed systems. You will implement Paxos and Raft, the
two most important consensus algorithms, and build a linearizable key-value
store on top of Raft.

## The Consensus Problem

Consensus requires that a group of nodes agree on a single value, even in
the presence of failures. A correct consensus algorithm must satisfy four
properties:

- **Agreement:** All correct nodes decide on the same value.
- **Validity:** The decided value must have been proposed by some node.
- **Termination:** All correct nodes eventually decide (liveness).
- **Integrity:** Each node decides at most one value.

The FLP impossibility result (Fischer, Lynch, Paterson, 1985) proves that
no deterministic consensus algorithm can guarantee termination in an
asynchronous system with even one faulty process. Practical algorithms
 circumvent this by using randomization, timeouts, or partial synchrony
assumptions.

## Paxos (Lamport, 1989)

Paxos is the foundational consensus algorithm. It operates in two phases:

### Phase 1 (Prepare)
1. **Proposer** selects a unique proposal number `n` and sends `Prepare(n)`
   to a majority of acceptors.
2. **Acceptor** responds with a `Promise` containing the highest-numbered
   proposal it has accepted (if any).

### Phase 2 (Accept)
1. **Proposer** sends `Accept(n, v)` to acceptors, where `v` is either
   the value from the highest-numbered promise, or the proposer's own value.
2. **Acceptor** accepts the proposal if `n >= promised_n`, and sends
   `Accepted(n, v)`.

### Roles
- **Proposer:** Initiates proposals.
- **Acceptor:** Votes on proposals.
- **Learner:** Learns the decided value (often combined with acceptor).

### Challenges with Paxos
- **Liveness:** Multiple proposers can cause livelock (dueling proposers).
- **Complexity:** Multi-Paxos (for log replication) adds significant
  complexity with leader election and log indexing.

## Raft (Ongaro, 2014)

Raft was designed to be more understandable than Paxos while providing
equivalent safety guarantees. It decomposes consensus into three subproblems:

### Leader Election
- Nodes start as **Followers**.
- If a follower hears nothing from a leader, it becomes a **Candidate**
  and requests votes.
- A candidate that receives votes from a majority becomes the **Leader**.
- Randomized election timeouts prevent split votes.

### Log Replication
- The leader receives client commands and appends them to its log.
- The leader sends `AppendEntries` RPCs to followers.
- A log entry is **committed** when a majority of nodes have replicated it.
- The state machine applies committed entries.

### Safety
- Election restriction: a candidate must have a log at least as up-to-date
  as a majority of nodes to win an election.
- This ensures committed entries are never lost.

### Term Numbers
Each election increments the term number. Nodes defer to higher terms,
ensuring that stale leaders step down.

## State Machine Replication

Consensus algorithms enable state machine replication: all nodes apply the
same sequence of commands to the same initial state, producing the same
final state. The consensus algorithm ensures all nodes agree on the
command sequence (the log).

```
Client -> Leader -> Log[1,2,3,...] -> State Machine -> Response
                     |
                     v
              Follower Logs (replicated via AppendEntries)
```

## Exercises

| #   | Exercise                    | Difficulty | Key Concept |
|-----|-----------------------------|------------|-------------|
| 01  | Consensus Definition        | ★★☆☆☆      | Formal properties |
| 02  | Paxos Proposer              | ★★★☆☆      | Phase 1a/2a |
| 03  | Paxos Acceptor              | ★★★☆☆      | Phase 1b/2b |
| 04  | Paxos Full Round            | ★★★★☆      | Multi-node Paxos |
| 05  | Raft State Machine          | ★★☆☆☆      | State transitions |
| 06  | Raft Leader Election        | ★★★★☆      | Election protocol |
| 07  | Raft Log Replication        | ★★★★☆      | AppendEntries |
| 08  | Raft Combined               | ★★★★★      | Full Raft |
| 09  | Raft Persistence            | ★★★★☆      | Crash recovery |
| 10  | Raft Snapshot               | ★★★★☆      | Log compaction |
| 11  | Raft KV Store               | ★★★★★      | State machine |
| 12  | Linearizability Checker     | ★★★★★      | Formal verification |
| 13  | Leader Election Comparison  | ★★★☆☆      | Algorithm comparison |

**Note:** This is the hardest module in the course. Exercises 08, 11, and 12
are particularly challenging and build on earlier exercises.

## Learning Goals

1. Understand the formal properties of consensus (agreement, validity,
   termination, integrity).
2. Implement Paxos proposer and acceptor logic.
3. Implement Raft leader election and log replication.
4. Build a linearizable KV store on top of Raft.
5. Understand why consensus is hard (FLP, livelock, split votes).

## Running

```bash
cargo test
cargo test -- --nocapture  # to see tracing output
```
