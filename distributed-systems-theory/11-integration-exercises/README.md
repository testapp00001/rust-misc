# Module 11: Integration Exercises

## How All Previous Modules Connect

This module combines the theoretical foundations from all previous modules into
practical, working distributed systems implementations. Here is how each
module contributes:

| Module | Concept | Used In |
|--------|---------|---------|
| 01 - Two Generals | Communication failures | p04 (Saga compensation), p07 (failure handling) |
| 02 - Time & Ordering | Logical clocks, causality | p02 (Vector clocks), p07 (HLC) |
| 03 - CAP Theorem | Consistency/Availability tradeoffs | p02 (Eventual consistency), p07 (Consistency modes) |
| 04 - Consensus | Raft, leader election | p01 (Raft KV store), p07 (Strong consistency) |
| 05 - FLP Impossibility | Limits of consensus | p06 (PBFT tolerates Byzantine faults) |
| 06 - Distributed Transactions | 2PC, Saga pattern | p04 (Saga orchestration) |
| 07 - Consistent Hashing | Shard routing | p05 (Sharded cache) |
| 08 - CRDTs | Conflict-free data types | p03 (OR-Set shopping cart), p07 (Eventual consistency) |
| 09 - Byzantine Fault Tolerance | PBFT protocol | p06 (PBFT KV store) |
| 10 - Clock Synchronization | Physical time coordination | p07 (HLC integration) |

## Integration Patterns

### Pattern 1: Consensus for Strong Consistency
When operations require linearizability, route them through Raft consensus
(Module 04). The leader serializes writes, replicates to a majority, and
commits before acknowledging. This is used in p01 and p07.

### Pattern 2: CRDTs for Eventual Consistency
When availability is more important than strong consistency, use CRDTs
(Module 08). Writes are local and propagate asynchronously. Merge
operations are commutative, associative, and idempotent. This is used
in p02, p03, and p07.

### Pattern 3: Saga for Distributed Transactions
When a business operation spans multiple services, use the Saga pattern
(Module 06) to ensure eventual consistency. Each step has a compensating
action for rollback. This is used in p04.

### Pattern 4: Consistent Hashing for Scalability
When distributing data across nodes, use consistent hashing (Module 07)
for minimal disruption during rebalancing. Virtual nodes improve load
distribution. This is used in p05.

### Pattern 5: Byzantine Fault Tolerance for Untrusted Networks
When nodes may be malicious, use PBFT (Module 09) to achieve consensus
with 3f+1 nodes. The three-phase protocol (pre-prepare, prepare, commit)
ensures safety even with Byzantine faults. This is used in p06.

### Pattern 6: Hybrid Logical Clocks for Event Ordering
When you need to order events across nodes without synchronized physical
clocks, use HLCs (Module 10). They combine physical time with logical
counters to maintain causality. This is used in p07.

## Real-World System Design

Using the concepts from these exercises, here is how a real distributed
system might be structured:

```
                    ┌─────────────────────────────────────┐
                    │          Client Requests             │
                    └─────────────┬───────────────────────┘
                                  │
                    ┌─────────────▼───────────────────────┐
                    │       Consistency Router             │
                    │  (p07 - Mode Selection)             │
                    └──────┬──────────────┬───────────────┘
                           │              │
              ┌────────────▼──┐    ┌──────▼────────────┐
              │  Strong Mode  │    │  Eventual Mode     │
              │  (Raft p01)   │    │  (CRDT p03)        │
              └──────┬────────┘    └──────┬─────────────┘
                     │                     │
              ┌──────▼────────┐    ┌──────▼─────────────┐
              │  Consensus    │    │  Vector Clocks      │
              │  (p04 Saga)   │    │  (p02)              │
              └──────┬────────┘    └──────┬─────────────┘
                     │                     │
              ┌──────▼─────────────────────▼─────────────┐
              │     Sharded Storage (p05)                │
              │     Consistent Hashing + Virtual Nodes   │
              └─────────────────────────────────────────┘
```

## Exercise List

| # | Exercise | Difficulty | Description |
|---|----------|-----------|-------------|
| 01 | Distributed KV with Raft | Medium | Build a distributed key-value store using Raft consensus |
| 02 | Eventually Consistent Store | Medium | Implement vector clocks and anti-entropy synchronization |
| 03 | CRDT Shopping Cart | Hard | Build a conflict-free replicated shopping cart using OR-Set |
| 04 | Saga Order System | Medium | Implement the Saga pattern for distributed transactions |
| 05 | Sharded Cache | Medium | Build a sharded cache with consistent hashing |
| 06 | PBFT KV Store | Hard | Implement Byzantine fault-tolerant consensus |
| 07 | Full Distributed System | Hard | Combine all concepts into a unified system |

## Running the Exercises

```bash
# Run all tests
cargo test -p integration-exercises

# Run a specific module's tests
cargo test -p integration-exercises -- p01
cargo test -p integration-exercises -- p02
cargo test -p integration-exercises -- p03
cargo test -p integration-exercises -- p04
cargo test -p integration-exercises -- p05
cargo test -p integration-exercises -- p06
cargo test -p integration-exercises -- p07
```

## Key Takeaways

1. **No one-size-fits-all**: Different consistency models suit different use cases
2. **Tradeoffs are inevitable**: Strong consistency costs latency; eventual consistency costs correctness
3. **Composition is powerful**: Small, well-understood building blocks combine into complex systems
4. **Testing is essential**: Distributed systems bugs are hard to reproduce; thorough testing is critical
5. **Theory guides practice**: Understanding the theoretical limits (CAP, FLP) helps make informed design decisions
