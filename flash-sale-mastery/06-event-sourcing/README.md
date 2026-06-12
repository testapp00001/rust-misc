# Module 06: Event Sourcing

> "The event log is the single source of truth. The current state is just a
> materialized view of the event history."

## Motivation

In a flash sale, every stock change must be traceable. A customer claims they
were charged but never received their item. A support engineer needs to
reconstruct exactly what happened, when, and why. With traditional CRUD, you
only have the current state -- the history is lost. Event sourcing solves this
by recording every change as an immutable event. The event log IS the source
of truth, and the current state can always be rebuilt by replaying it.

## Concept Map

```
 Event Created
      |
      v
 +----------+     +-----------+     +-------------+
 |  Event   | --> |   Event   | --> |    Event    |
 |  Design  |     |   Store   |     |   Replay    |
 | (p01)    |     | (p02)     |     | (p03)       |
 +----------+     +-----------+     +-------------+
      |                |                   |
      v                v                   v
 +----------+     +-----------+     +-------------+
 |  Redis   | --> | Consumer  | --> | Reconcile   |
 | Streams  |     | Groups    |     | State       |
 | (p04)    |     | (p05)     |     | (p06)       |
 +----------+     +-----------+     +-------------+
                        |
                        v
              +-------------------+
              |   Audit Query     |
              |   (p07)           |
              +-------------------+
                        |
                        v
              +-------------------+
              | Event Versioning  |
              | (p08)             |
              +-------------------+
```

## Theory

### Event Sourcing Basics

Instead of storing current state in a mutable row, you store every state
change as an immutable event:

```
Traditional (CRUD):
  UPDATE inventory SET stock = 95 WHERE product_id = 1001;
  -- Previous value (100) is lost

Event Sourcing:
  APPEND event: { type: StockDecremented, product_id: 1001, remaining: 95 }
  APPEND event: { type: StockDecremented, product_id: 1001, remaining: 94 }
  -- Full history preserved
```

### Append-Only Logs

The event store is append-only. Events are never modified or deleted. This
guarantees:
- **Immutability**: Past state is always recoverable
- **Auditability**: Every change has a timestamp and actor
- **Debuggability**: You can trace exactly what happened

### Event Replay

Current state = fold(initial_state, events). You start from zero (or a
snapshot) and apply each event in order. This is the `reduce` operation
functional programmers know well.

### Redis Streams

Redis Streams provide a durable, ordered, append-only log with consumer groups.
They combine the simplicity of Redis with message-broker semantics:
- **XADD**: Append an entry to a stream
- **XREAD**: Read entries from a stream
- **XREADGROUP**: Read as part of a consumer group (load balancing)
- **XACK**: Acknowledge processing (removes from PEL)
- **XCLAIM**: Recover stuck entries from crashed consumers

## Trade-offs

| Aspect | Event Sourcing | Traditional CRUD |
|--------|---------------|-----------------|
| Audit trail | Built-in (event log) | Requires separate audit table |
| Storage cost | Higher (all history) | Lower (only current state) |
| Query complexity | Must replay for current state | Direct read |
| Debugging | Full history available | Limited to current state |
| Schema evolution | Requires migration strategy | ALTER TABLE |
| Consistency | Eventually consistent (replay) | Immediately consistent |

### When Event Sourcing Fits Flash Sales

- **Dispute resolution**: Customer says "I bought it but didn't get it"
- **Fraud detection**: Unusual patterns across multiple purchases
- **Capacity planning**: Replay to understand traffic patterns
- **Bug investigation**: Replay to reproduce the exact conditions

### When It Doesn't Fit

- Simple CRUD with no audit requirements
- Real-time queries where replay latency is unacceptable
- Storage-constrained environments

## Failure Modes

| Failure | Impact | Mitigation |
|---------|--------|------------|
| Event store corruption | State cannot be rebuilt | Checksums, replication, backups |
| Out-of-order events | Incorrect replayed state | Sort by timestamp, use sequence numbers |
| Schema mismatch | Deserialization fails | Versioned envelopes, migration functions |
| Consumer crash | Events lost or duplicated | PEL (Pending Entry List), XCLAIM |
| Large event log | Slow replay | Periodic snapshots, truncate old events |

## Connection to Other Modules

- **Module 02 (Redis Lua Scripting)**: Lua scripts produce events that flow
  into the event store. The atomicity of Lua ensures events are consistent.
- **Module 03 (Atomic Counters)**: Stock decrements are the most common event.
  The atomic counter in Redis is the "live" view; events are the source of truth.
- **Module 13 (Reconciliation)**: The reconciliation job replays events and
  compares with live state -- directly using the patterns from this module.

## Exercise List

| # | Exercise | Focus | Difficulty |
|---|----------|-------|------------|
| 01 | Event Design | Domain events, serialization | Foundation |
| 02 | Event Store | Append-only in-memory store | Foundation |
| 03 | Event Replay | State reconstruction | Core |
| 04 | Redis Streams | XADD, XREAD | Core |
| 05 | Consumer Groups | XREADGROUP, XACK, PEL | Advanced |
| 06 | Reconciliation | Detecting state drift | Advanced |
| 07 | Audit Query | Filtering events for audits | Core |
| 08 | Event Versioning | Schema evolution, migrations | Advanced |

## Quick Start

```bash
# Run exercise tests (will fail on todo!() stubs)
cargo test

# Run solution tests (should all pass)
cargo test --features solution

# Run a specific exercise
cargo test --features solution p01_event_design
```

## References

- Martin Fowler: [Event Sourcing](https://martinfowler.com/eaaDev/EventSourcing.html)
- Redis documentation: [Streams](https://redis.io/docs/data-types/streams/)
- Greg Young: [CQRS Documents](https://cqrs.files.wordpress.com/2010/11/cqrs_documents.pdf)
- Jay Kreps: [The Log](https://engineering.linkedin.com/distributed-systems/log-what-every-software-engineer-should-know-about-real-time-datas-unifying)
