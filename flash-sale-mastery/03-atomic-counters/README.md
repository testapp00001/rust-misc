# Module 03: Atomic Counters

## Motivation

The atomic counter is the beating heart of a flash sale system. When 50,000
customers hit "Buy" at the same instant for 100 limited-edition items, the
counter must guarantee that exactly 100 purchases succeed -- no more, no less.
This is the distributed consensus problem in its most practical form.

Getting this wrong means **overselling** (selling items you don't have, leading
to angry customers and fulfillment chaos) or **underselling** (rejecting valid
purchases, leaving money on the table). Both are unacceptable.

This module teaches you the patterns, trade-offs, and failure modes of
distributed atomic counters, progressing from simple in-memory solutions
to production-grade distributed patterns.

## Concept Map

```
                    Atomic Counters
                         |
          +--------------+--------------+
          |              |              |
     Local (p01)   Redis-based    Database
          |              |              |
     AtomicI64    +------+------+   SQL
          |       |      |      |     |
       Single   DECR   CAS    Lock  Optimistic
       process  (p02)  (p03)  (p05)  (p04)
                       |
                  WATCH/MULTI/EXEC
```

### Compare-and-Swap vs Pessimistic vs Atomic Operations

| Approach | Mechanism | Best For | Worst Case |
|----------|-----------|----------|------------|
| **Atomic DECR** (p02) | Single Redis command | High throughput, simple logic | Brief negative state |
| **Compare-and-Swap** (p03) | WATCH/MULTI/EXEC | Conditional updates | O(n) retries under contention |
| **Optimistic Locking** (p04) | Version number check | Database-backed stock | Wasted reads on conflict |
| **Pessimistic Locking** (p05) | Distributed lock (SET NX) | High contention, must-complete | Serialized throughput |

## Theory

### Linearizability

A counter is **linearizable** if every operation appears to take effect
atomically at some point between its invocation and response. Redis DECR
is linearizable -- when you receive the response, you know the decrement
has taken effect and no other operation can see the old value.

This is the strongest consistency model and what we need for stock counters.

### Sequential Consistency

Operations appear in some total order consistent with each client's
program order. CAS with retry achieves sequential consistency -- the
retries ensure eventual progress, and the WATCH/MULTI/EXEC ensures
the update only succeeds if no concurrent modification occurred.

### Eventual Consistency

All replicas converge to the same value eventually, but may temporarily
disagree. This is what happens between Redis and the database during
async writes -- Redis is updated first, the database catches up later.
Reconciliation (p06) handles the gap.

## Trade-offs

### Latency vs Consistency

| Approach | Latency | Consistency |
|----------|---------|-------------|
| Local AtomicI64 | ~10ns | None (single process) |
| Redis DECR | ~0.5ms | Linearizable |
| Redis CAS | ~2ms (1+ round trips) | Linearizable |
| Pessimistic Lock | ~1ms + hold time | Linearizable |
| Optimistic Lock | ~1ms + retry cost | Linearizable |

### Throughput vs Safety

| Approach | Throughput | Safety Guarantee |
|----------|------------|------------------|
| Redis DECR | ~500K ops/sec | Stock >= -1 (brief) |
| Redis Lua | ~300K ops/sec | Stock >= 0 (strict) |
| CAS | ~50K ops/sec | Stock >= 0 (strict) |
| Pessimistic Lock | ~10K ops/sec | Stock >= 0 (strict) |

## Failure Modes

### Split Brain

When a network partition isolates a node, it may continue serving requests
with stale data. Two nodes may both think they hold the lock, leading to
double-spending. Solutions: Redlock (majority consensus), fencing tokens.

### Lost Updates

Two transactions read the same value, both compute the same new value,
and both write it -- one update is lost. CAS and optimistic locking
prevent this by detecting the conflict and retrying.

### Phantom Reads

A transaction reads a value, another transaction modifies it, and the
first transaction re-reads seeing a different value. In the counter
context, this manifests as the CAS retry loop seeing a different value
on each attempt.

### Clock Skew

TTL-based locks depend on clocks agreeing. If a node's clock is ahead,
it may think a lock has expired when it hasn't, or vice versa. Solutions:
use monotonic clocks, fencing tokens, or lease-based locks.

## Connection to Other Modules

### Module 01: Redis Fundamentals

This module builds directly on Redis commands (GET, SET, DECR, WATCH,
MULTI, EXEC) and connection pooling from Module 01.

### Module 02: Redis Lua Scripting

The DECR-then-check pattern (p02) has a brief oversell window. Module 02
teaches Lua scripting, which can make the check-and-decrement truly atomic
in a single round-trip.

### Module 13: Reconciliation

Exercise p06 introduces reconciliation between Redis and the database.
Module 13 dives deep into production reconciliation with event sourcing,
audit trails, and automated correction.

## Exercise List

| # | Exercise | Key Concept | Difficulty |
|---|----------|-------------|------------|
| 01 | Local Atomic Counter | `AtomicI64`, `Arc`, compare-exchange loop | Beginner |
| 02 | Redis Atomic Counter | DECR, oversell prevention | Beginner |
| 03 | Compare-and-Swap | WATCH/MULTI/EXEC, retry loop | Intermediate |
| 04 | Optimistic Locking | Version numbers, conditional update | Intermediate |
| 05 | Pessimistic Locking | Distributed lock, SET NX EX, Lua release | Intermediate |
| 06 | Reconciliation | Redis vs DB comparison, mismatch resolution | Intermediate |
| 07 | Counter Benchmark | Performance comparison, ops/sec measurement | Intermediate |
| 08 | Failure Analysis | Connection drop, network partition, clock skew | Advanced |
| 09 | Multi-Instance Test | Concurrent pods, consistency verification | Advanced |

## Running the Exercises

```bash
# Run exercise stubs (will panic with todo!())
cargo test

# Run with solutions
cargo test --features solution

# Run a specific exercise
cargo test --features solution p01

# Run benchmarks
cargo test --features solution p07 -- --nocapture
```

## References

- [Redis DECR documentation](https://redis.io/commands/decr/)
- [Redis WATCH documentation](https://redis.io/commands/watch/)
- [Distributed locks with Redis (Redlock)](https://redis.io/topics/distlock)
- [Martin Kleppmann: How to do distributed locking](https://martin.kleppmann.com/2016/02/08/how-to-do-distributed-locking.html)
- [Compare-and-Swap (Wikipedia)](https://en.wikipedia.org/wiki/Compare-and-swap)
- [Optimistic concurrency control](https://en.wikipedia.org/wiki/Optimistic_concurrency_control)
