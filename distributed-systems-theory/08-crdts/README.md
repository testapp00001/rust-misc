# Module 08: CRDTs (Conflict-free Replicated Data Types)

## Overview

CRDTs are data structures that can be replicated across multiple nodes and updated independently
and concurrently, with a mathematical guarantee that all replicas will eventually converge to
the same state. They achieve Strong Eventual Consistency (SEC) without requiring consensus
protocols.

## Background

### Strong Eventual Consistency (SEC)

SEC guarantees that any two replicas that have received the same set of updates (in any order)
are in the same state. This is stronger than eventual consistency (which only guarantees
convergence eventually) because SEC replicas converge as soon as they receive the same updates,
regardless of delivery order.

### State-based CRDTs (CvRDTs)

Convergent Replicated Data Types: each replica maintains full state and periodically ships
its entire state to other replicas. Merging is done via a join function that must satisfy:

1. **Associativity**: join(join(a, b), c) = join(a, join(b, c))
2. **Commutativity**: join(a, b) = join(b, a)
3. **Idempotency**: join(a, a) = a

These three properties form a join-semilattice.

### Operation-based CRDTs (CmRDTs)

Commutative Replicated Data Types: each replica broadcasts operations (not state). The
underlying broadcast layer must guarantee: (1) exactly-once delivery, (2) causal ordering
of concurrent operations. Operations must commute.

## CRDT Types

### Counters

- **G-Counter** (Grow-only): Each replica owns a slot. Incrementing increments only that slot.
  Value = sum of all slots. Merge = element-wise max. Cannot decrement.
- **PN-Counter** (Positive-Negative): Two G-Counters -- one for increments, one for
  decrements. Value = P - N.

### Sets

- **G-Set** (Grow-only): Set union for merge. Cannot remove elements.
- **OR-Set** (Observed-Remove): Each add generates a unique tag. Remove removes all observed
  tags. Merge unions tags and removes tags that have been removed. Provides add-wins semantics
  for concurrent add/remove.

### Registers

- **LWW-Register** (Last-Writer-Wins): Each write has a timestamp. Higher timestamp wins.
  Simple but loses concurrent writes.
- **MV-Register** (Multi-Value): Tracks concurrent writes via vector clocks. Returns multiple
  values on conflict, requiring application-level resolution (similar to Riak siblings).

### Maps

- **OR-Map** (Observed-Remove Map): Keys form an OR-Set. Values can be any CRDT.
  Supports nested CRDTs with automatic merge propagation.

## Real-World Usage

| System | CRDT Usage |
|---|---|
| Riak | OR-Set, LWW-Register, PN-Counter |
| Redis CRDB | G-Counter, PN-Counter, G-Set, OR-Set, LWW-Register |
| Figma | Custom CRDTs for collaborative editing |
| Apple iCloud | CRDTs for collaborative documents |
| Automerge | Library of CRDTs for local-first software |
| Yjs | CRDT framework for collaborative editing |

## Exercise List

| # | Exercise | Difficulty | Key Concepts |
|---|---|---|---|
| p01 | G-Counter | Beginner | Grow-only counter, merge |
| p02 | PN-Counter | Beginner | Positive-negative, value = P - N |
| p03 | G-Set | Beginner | Grow-only set, union merge |
| p04 | OR-Set | Intermediate | Tags, observed-remove, add-wins |
| p05 | LWW-Register | Intermediate | Timestamp-based resolution |
| p06 | MV-Register | Intermediate | Vector clocks, multi-value |
| p07 | OR-Map | Intermediate | Nested CRDTs, key management |
| p08 | Merge Properties | Advanced | Proptest, algebraic verification |
| p09 | Replica Simulation | Advanced | Network delays, convergence |
| p10 | Shopping Cart | Advanced | Classic Riak example |
| p11 | Flash Sale Counter | Advanced | Practical CRDT application |
| p12 | CRDT Benchmarks | Advanced | Performance measurement |

## References

- Shapiro, M. et al. (2011). "A Comprehensive Study of CRDTs"
- Shapiro, M. et al. (2011). "Conflict-free Replicated Data Types"
- Baquero, C. & Preguica, N. (2016). "Why Logical Clocks are Easy"
- Brewka, G. et al. (2011). "A Review of CRDTs and Related Algorithms"
- DeCandia, G. et al. (2007). "Dynamo: Amazon's Highly Available Key-value Store"
