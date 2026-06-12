# Module 03: CAP Theorem

## Overview

This module explores the CAP theorem (Brewer's conjecture), one of the most
foundational results in distributed systems theory. You will implement and
test systems that make different trade-offs along the CAP spectrum, gaining
hands-on understanding of why the theorem matters and how real systems
navigate its constraints.

## Background

### Brewer's Conjecture (2000)

In his 2000 PODC keynote, Eric Brewer conjectured that a distributed data store
can provide at most two of the following three guarantees simultaneously:

- **Consistency (C):** Every read receives the most recent write or an error.
  All nodes see the same data at the same time.
- **Availability (A):** Every request receives a (non-error) response, without
  guaranteeing that it contains the most recent write.
- **Partition Tolerance (P):** The system continues to operate despite network
  partitions (messages may be dropped or delayed between nodes).

### Gilbert & Lynch Proof (2002)

Seth Gilbert and Nancy Lynch formally proved Brewer's conjecture in 2002. Their
proof shows that in the presence of a network partition, a system must choose
between consistency and availability. Since network partitions are inevitable
in real distributed systems, the practical choice is really between CP and AP.

### Why "Pick 2 of 3" Is Misleading

The common "pick any 2 of 3" framing is misleading for several reasons:

1. **Partitions are not optional.** Networks fail. You cannot choose to not have
   partitions, so P is effectively always required. The real choice is CP vs AP.
2. **Consistency and availability are not binary.** They exist on a spectrum with
   many intermediate consistency levels (strong, causal, eventual, etc.).
3. **The trade-off is per-operation, not per-system.** A system might be CP for
   some operations and AP for others.

### PACELC Theorem

Daniel Abadi extended CAP with the PACELC theorem (2010):

- If there is a **P**artition: choose between **A**vailability and
  **C**onsistency.
- **E**lse (normal operation): choose between **L**atency and **C**onsistency.

This captures the additional trade-off that even without partitions, systems
must balance consistency against response latency.

### Consistency Model Hierarchy

```
Strong (Linearizable)
  |
  +-- Sequential
  |     |
  |     +-- Causal
  |           |
  |           +-- Read-Your-Writes
  |           |     |
  |           |     +-- Monotonic Reads
  |           |           |
  |           |           +-- Eventual
  |
  +-- Read-Your-Writes (independent branch)
```

Strong consistency requires all operations to appear as if executed in some
total order that respects real-time ordering. Eventual consistency only
guarantees that, given enough time without new writes, all replicas will
converge to the same value.

### Real System Classifications

| System    | Classification | Partition Behavior | Normal Behavior |
|-----------|---------------|-------------------|-----------------|
| etcd      | CP            | Rejects writes    | Strong (linearizable) |
| ZooKeeper | CP            | Rejects writes    | Strong (linearizable) |
| Cassandra | AP            | Accepts writes    | Tunable (eventual by default) |
| DynamoDB   | AP           | Accepts writes    | Eventually consistent (default) |
| HBase     | CP            | Rejects writes    | Strong |
| CockroachDB | CP          | Rejects writes    | Serializable |

## Exercises

| #   | Exercise                    | Difficulty | Key Concept |
|-----|-----------------------------|------------|-------------|
| 01  | Replicated KV Store         | ★★☆☆☆      | Basic replication |
| 02  | CP Mode                     | ★★★☆☆      | Quorum writes |
| 03  | AP Mode                     | ★★★☆☆      | Conflict resolution |
| 04  | Partition Simulator         | ★★☆☆☆      | Network partitioning |
| 05  | Consistency Levels          | ★★★★☆      | Consistency spectrum |
| 06  | Linearizability Test        | ★★★★☆      | Formal verification |
| 07  | Eventual Convergence        | ★★★☆☆      | Convergence proof |
| 08  | PACELC Analysis             | ★★☆☆☆      | System classification |
| 09  | Read-Your-Writes            | ★★★☆☆      | Session consistency |
| 10  | Monotonic Reads             | ★★★☆☆      | Read monotonicity |

## Learning Goals

1. Understand why the CAP theorem constrains distributed system design.
2. Implement both CP and AP systems and observe their trade-offs firsthand.
3. Simulate network partitions and measure their impact on consistency and
   availability.
4. Classify real-world systems using both CAP and PACELC frameworks.
5. Implement and test various consistency levels from strong to eventual.

## Running

```bash
cargo test
cargo test -- --nocapture  # to see println/tracing output
```
