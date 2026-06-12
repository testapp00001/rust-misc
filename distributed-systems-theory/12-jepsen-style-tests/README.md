# Module 12: Jepsen-Style Tests for Distributed Systems

## Overview

Jepsen testing is a methodology for evaluating the correctness of distributed
systems under real-world failure conditions. Named after the open-source Jepsen
framework created by Kyle Kingsbury (Aphyr), it provides rigorous tools to verify
that distributed databases and coordination services uphold their consistency
claims even when the network drops messages, delays delivery, or partitions the
cluster.

This module walks you through building your own Jepsen-style verification tools
in Rust. You will implement consistency checkers, fault injectors, and
convergence validators that mirror the techniques used in production-grade
correctness testing.

---

## What Is Jepsen Testing?

A Jepsen test has four phases:

1. **Generate** -- produce a workload of operations (reads and writes) against
   the system under test.
2. **Inject faults** -- disrupt the network (partitions, latency, packet loss),
   kill processes, or introduce clock skew.
3. **Observe** -- record every operation with its start time, end time, and
   result.
4. **Check** -- verify the history of operations is consistent with the
   claimed consistency model (linearizability, sequential consistency, etc.).

The power of Jepsen lies in phase 3 and 4: by carefully logging every client
operation and checking the resulting history against a formal model, you can
discover subtle bugs that unit tests miss.

---

## Linearizability Checking

Linearizability is the strongest single-object consistency model. A history is
linearizable if every operation appears to take effect atomically at some point
between its invocation and completion, and the resulting total order is
consistent with real-time ordering.

Checking linearizability is NP-complete in the general case, but for small
histories we can use brute-force: enumerate all valid orderings of concurrent
operations and check each one.

**Key properties to verify:**

- Every read returns the value of the most recent write in the linearization
  order.
- If operation A completes before operation B begins, A must appear before B in
  the linearization.

---

## Sequential Consistency Checking

Sequential consistency requires that the result of any execution is the same as
if all operations were executed in some sequential order, and the operations of
each individual process appear in this sequence in program order.

Unlike linearizability, sequential consistency does not require real-time
ordering -- it only respects each process's program order.

**Checking strategy:** Build a total order of all operations such that for each
process, its operations appear in their original sequence number order.

---

## Eventual Convergence

Eventual convergence (eventual consistency) guarantees that if no new updates
are made, all replicas will eventually converge to the same state. Testing
convergence involves:

1. Writing values to different replicas.
2. Allowing messages to be delivered (with possible delays).
3. Verifying that all replicas eventually reach the same state.

The key insight is that convergence time depends on the network delay model and
the anti-entropy protocol used.

---

## Causal Consistency

Causal consistency preserves the causal ordering of operations. If operation A
causally precedes operation B (e.g., A wrote a value that B read), then every
process must see A before B.

Causal ordering is tracked using **vector clocks** -- a vector of counters, one
per process, that records the causal history of each operation.

**Happens-before relation:** Operation A happens-before operation B if:
- A and B are on the same process and A precedes B in program order, or
- A is a write and B is a read of the value A wrote, or
- Transitively, there exists C such that A happens-before C and C happens-before
  B.

---

## Partition Tolerance

Network partitions split the system into isolated groups. The CAP theorem tells
us we must choose between consistency (CP) and availability (AP) during a
partition.

**CP systems** reject writes when a quorum cannot be reached.
**AP systems** accept writes locally and reconcile later.

This module lets you simulate partitions and verify that the system behaves
according to its chosen strategy.

---

## Network Chaos Injection

Real networks exhibit:

- **Message loss** -- packets are dropped silently.
- **Message duplication** -- packets arrive more than once.
- **Message reordering** -- packets arrive out of order.
- **Message delay** -- packets are delayed by unpredictable amounts.

A chaos monkey applies these perturbations to a stream of messages, allowing you
to test that your system handles each case correctly.

---

## How to Test Distributed Systems Properly

1. **Define your consistency model** -- know exactly what guarantees the system
   claims.
2. **Build a history recorder** -- log every operation with timestamps and
   results.
3. **Implement a checker** -- verify the history against the formal model.
4. **Inject realistic faults** -- partitions, crashes, message loss, delays.
5. **Run at scale** -- small histories can be checked exhaustively; larger ones
   require sampling or model checking.
6. **Automate** -- integrate into CI so every commit is tested.

---

## Exercise List

| # | Exercise                         | Difficulty | File                      |
|---|----------------------------------|------------|---------------------------|
| 1 | Linearizability Checker          | Hard       | `linearizability.rs`      |
| 2 | Sequential Consistency Checker   | Medium     | `sequential_consistency.rs`|
| 3 | Eventual Convergence Simulator   | Medium     | `eventual_convergence.rs`  |
| 4 | Causal Consistency Checker       | Hard       | `causal_consistency.rs`    |
| 5 | Partition Tolerance Tests        | Medium     | `partition_tolerance.rs`   |
| 6 | Network Chaos Monkey             | Medium     | `network_chaos.rs`        |

### Difficulty Guide

- **Easy** -- Straightforward implementation, single concept.
- **Medium** -- Requires understanding of distributed systems concepts and
  careful implementation.
- **Hard** -- Complex algorithms, subtle correctness properties, or combinatorial
  search.

---

## Building and Running

```bash
# Run all tests
cargo test --manifest-path 12-jepsen-style-tests/Cargo.toml

# Run a specific module's tests
cargo test --manifest-path 12-jepsen-style-tests/Cargo.toml -- linearizability

# Run with output
cargo test --manifest-path 12-jepsen-style-tests/Cargo.toml -- --nocapture
```

---

## References

- Kyle Kingsbury, *Call Me Maybe: Jepsen Series* -- https://aphyr.com/tags/Jepsen
- The Jepsen Testing Framework -- https://github.com/jepsen-io/jepsen
- Herlihy & Wing, *Linearizability: A Correctness Condition for Concurrent
  Objects* (1990)
- Lamport, *Time, Clocks, and the Ordering of Events in a Distributed System*
  (1978)
- Schneider, *Implementing Fault-Tolerant Services Using the State Machine
  Approach* (1990)
