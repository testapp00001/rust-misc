# Module 13: Reconciliation

Reconcile Redis (fast path) with PostgreSQL (durable storage) after a flash sale. Detect discrepancies, replay events to verify state, correct mismatches, and generate structured reports.

---

## Motivation

During a flash sale, Redis is the **authoritative** inventory source. Every stock decrement, reservation, and confirmation flows through Redis for sub-millisecond latency. PostgreSQL receives asynchronous writes but may lag behind.

After the sale ends, the two systems must **agree**. If Redis says 45 units remain but PostgreSQL says 42, which is correct? If an order appears in PostgreSQL but not in Redis's fast-path log, was it lost or just delayed?

Reconciliation is the process of systematically comparing these two sources of truth, detecting every discrepancy, and applying safe corrections. It is the final safety net that catches whatever idempotency checks, event sourcing, and resilience patterns missed.

---

## Concept Map

```
                    SALE ENDS
                        |
                        v
            +-----------+-----------+
            |                       |
     Redis State              DB State
   (fast, volatile)      (durable, slower)
            |                       |
            +-----------+-----------+
                        |
                        v
               Compare Inventories
                        |
              +---------+---------+
              |                   |
         Match?              Mismatch?
              |                   |
         No Action         +------+------+
                           |             |
                     Redis > DB     Redis < DB
                           |             |
                     Update DB     Raise Alert
                           |       (possible loss)
                           v
                    Event Replay
                    (verify state)
                           |
                           v
                 Discrepancy Scan
              (orders, claims, vouchers)
                           |
                           v
                  Generate Report
                (JSON + human-readable)
```

---

## Theory

### Eventual Consistency

In a distributed system with separate Redis and PostgreSQL stores, **eventual consistency** means that if no new updates are made, all replicas will eventually converge to the same state. Reconciliation is the mechanism that enforces this convergence.

During a flash sale we accept **strong consistency in Redis** (atomic DECR, Lua scripts) and **eventual consistency in PostgreSQL** (async writes, possible failures). After the sale, reconciliation closes the gap.

### Reconciliation Patterns

| Pattern | When to Use | Trade-off |
|---------|-------------|-----------|
| **Last-write-wins** | Simple systems, low conflict rate | Can lose writes |
| **Source-of-truth hierarchy** | Redis is authoritative during sale | Simple but rigid |
| **Event-sourced rebuild** | Full audit trail available | Expensive but exact |
| **CRDT merge** | Both sides can accept writes | Complex but conflict-free |

This module uses **source-of-truth hierarchy**: Redis wins during the sale, PostgreSQL wins after. The reconciler checks which system has more recent data and applies corrections accordingly.

### Safe Reconciliation Rules

1. **During a sale**: Report only. Never modify Redis or DB while live traffic is flowing.
2. **After a sale**: Redis stock > DB stock means update DB. Redis stock < DB stock means alert (possible data loss in Redis).
3. **Always log**: Every reconciliation action must be recorded for audit.

---

## Trade-offs

| Approach | Pros | Cons |
|----------|------|------|
| **Real-time sync** (write-through) | No reconciliation needed, always consistent | Higher latency, couples Redis and DB availability |
| **Batch reconciliation** (post-sale) | Decoupled systems, sale speed unaffected | Window of inconsistency, requires reconciliation logic |
| **Hybrid** (async + reconciliation) | Best of both: fast writes, durable backup | More infrastructure, eventual consistency window |

This module implements **batch reconciliation** with the option to run lightweight checks during the sale (report-only mode).

---

## Failure Modes

1. **Reconciliation during active sale**: If the reconciler modifies DB while the sale is still writing to Redis, the DB will be overwritten again. **Mitigation**: report-only mode during sale, corrections only after sale ends.

2. **Partial sync**: The reconciler crashes mid-way through correcting products. Some are corrected, others are not. **Mitigation**: idempotent corrections, checkpoint progress, resume from last checkpoint.

3. **Event store gaps**: Some events were lost (Redis AOF failure, Kafka partition gap). Replaying events produces a state that does not match reality. **Mitigation**: compare rebuilt state with both Redis and DB; if all three disagree, flag for manual review.

4. **Clock skew**: Events are ordered by timestamp, but server clocks disagree. Events may replay in the wrong order. **Mitigation**: use sequence numbers or logical clocks instead of wall-clock time.

5. **Duplicate vouchers across time**: A voucher is used, the order fails, and the user retries with the same voucher. The reconciler may flag this as duplicate usage when it is actually a legitimate retry. **Mitigation**: correlate voucher usage with order status (cancelled orders should release voucher claims).

---

## Connection to Other Modules

- **Module 03 (Atomic Counters)**: The inventory counters in Redis that reconciliation reads and compares. If the atomic decrement logic was correct, reconciliation should find zero mismatches.

- **Module 06 (Event Sourcing)**: The event store that `EventReplay` reads from. Reconciliation is the primary consumer of the event log -- it uses events to verify that the current state is consistent with the history.

- **Module 12 (Order Worker)**: The worker that creates orders and reservations. Discrepancy detection catches cases where the worker wrote to one system but not the other.

---

## Exercises

| # | Exercise | Focus Area |
|---|----------|------------|
| 01 | `p01_inventory_sync` | Compare Redis and DB stock, determine correction action |
| 02 | `p02_event_replay` | Rebuild inventory state from event history, detect drift |
| 03 | `p03_discrepancy_detector` | Scan orders, claims, and vouchers for inconsistencies |
| 04 | `p04_report_generator` | Build structured reports with JSON and text output |

Run exercise tests:
```bash
cargo test -p reconciliation
```

Run solution tests:
```bash
cargo test -p reconciliation --features solution
```

Run the reconciliation job:
```bash
cargo run -p reconciliation --features solution
```

---

## References

- [Martin Kleppmann - Designing Data-Intensive Applications, Ch. 5: Replication](https://dataintensive.net/)
- [Jepsen: Consistency Models](https://jepsen.io/consistency)
- [Stripe - Reconciliation at Scale](https://stripe.com/blog/idempotency)
- [AWS Well-Architected - Reliability Pillar: Change Management](https://docs.aws.amazon.com/wellarchitected/latest/reliability-pillar/)
