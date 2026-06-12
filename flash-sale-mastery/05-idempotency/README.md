# Module 05: Idempotency and Exactly-Once Semantics

> Ensure that no matter how many times a user clicks "Buy", they never receive
> extra vouchers or double charges.

## Motivation

In a flash sale, users mash the "Buy" button, browsers auto-retry on timeout,
and network glitches cause duplicate HTTP requests. Without idempotency
protection, a single user could receive 10 vouchers for a single purchase
intent. This module teaches you to build systems where repeating an operation
has the same effect as performing it once.

## Concept Map

```
User clicks "Buy" (possibly 5 times)
        |
        v
+---------------------+
| Generate Idempotent |  <-- p01: key from (user, product, sale)
| Key                 |
+---------------------+
        |
        v
+---------------------+
| Check Idempotency   |  <-- p02/p03: Redis SET NX or DB unique constraint
| Store               |
+---------------------+
        |
   +---------+---------+
   |                   |
   v                   v
 First Time         Duplicate
   |                   |
   v                   v
 Process order     Return cached
   |               result
   v
+---------------------+
| Cache Result        |  <-- p02/p03: store result for future duplicates
+---------------------+
```

## Theory

### Idempotency

An operation is **idempotent** if applying it multiple times produces the same
result as applying it once. In HTTP, GET is naturally idempotent, but POST is
not. We make POST idempotent by attaching an **idempotency key** and tracking
which keys have already been processed.

### Exactly-Once vs Effectively-Once

True exactly-once delivery is impossible in distributed systems (see the Two
Generals Problem). What we build instead is **effectively-once** semantics:
the message may be delivered multiple times, but the *side effects* happen
exactly once. This is achieved through:

1. **Deduplication** -- detect duplicate messages/requests
2. **Idempotent processing** -- re-processing a duplicate is a no-op

### Deduplication Strategies

| Strategy | Storage | Speed | Durability | Best For |
|----------|---------|-------|------------|----------|
| Redis SET NX | In-memory | ~0.1ms | Volatile | First-line defense |
| DB Unique Constraint | Disk | ~1-5ms | Durable | Authoritative record |
| Combined (Redis + DB) | Both | ~0.1ms fast path | Durable | Production systems |

## Trade-offs

### Redis SET NX (p02)
- **Pro**: Sub-millisecond checks, handles 100K+ ops/sec
- **Con**: Data lost on Redis restart (unless AOF persistence), TTL-based expiry
- **Risk**: A key that expires too early allows duplicates

### Database Unique Constraint (p03)
- **Pro**: Survives restarts, ACID guarantees, no TTL issues
- **Con**: Higher latency (~1-5ms), database becomes bottleneck
- **Risk**: Connection pool exhaustion under high load

### Combined Approach (p04)
- **Pro**: Fast path through Redis, durable fallback to DB
- **Con**: More complex, must keep Redis and DB in sync
- **Risk**: Race between Redis check and DB insert

## Failure Modes

1. **Key Collision**: Two different operations generate the same key
   - Mitigation: Include enough context in deterministic keys (user + product + sale + timestamp)

2. **TTL Too Short**: Redis key expires before the sale ends
   - Mitigation: Set TTL longer than the maximum sale duration

3. **Race Condition**: Two requests check the store simultaneously
   - Mitigation: Atomic operations (SET NX, DB INSERT ON CONFLICT)

4. **Store Unavailable**: Redis or DB is down
   - Mitigation: Fail closed (reject requests) rather than fail open (allow duplicates)

## Connection to Other Modules

- **Module 02 (Redis Lua Scripting)**: Lua scripts can make multi-step Redis
  operations atomic, which is critical for idempotency checks
- **Module 04 (Traffic Shaping)**: Rate limiting reduces duplicate request
  pressure but cannot eliminate it entirely
- **Module 11 (Flash Sale API)**: The idempotency middleware plugs directly
  into the API layer's request pipeline

## Exercises

| # | Exercise | Focus |
|---|----------|-------|
| 01 | Idempotency Key Generation | UUID v4, deterministic hashing, validation |
| 02 | Redis Idempotency Store | SET NX, TTL, atomic check-and-set |
| 03 | DB Idempotency Store | Unique constraints, SQL upsert patterns |
| 04 | Concurrent Dedup | Handling simultaneous identical requests |
| 05 | Idempotent Handler Middleware | Axum/Tower middleware integration |
| 06 | Exactly-Once Consumer | Message deduplication + idempotent processing |
| 07 | Race Condition Tests | Barrier-synchronized timing tests |
| 08 | Idempotency Benchmark | Redis vs DB vs combined performance |

## References

- [Stripe Idempotency Keys](https://stripe.com/docs/idempotency)
- [Microsoft Cloud Design Patterns: Idempotent Filter](https://learn.microsoft.com/en-us/azure/architecture/patterns/idempotent-filter)
- [Martin Kleppmann: Designing Data-Intensive Applications, Ch. 9](https://dataintensive.net/)
- [PostgreSQL UPSERT (INSERT ON CONFLICT)](https://www.postgresql.org/docs/current/sql-insert.html)
