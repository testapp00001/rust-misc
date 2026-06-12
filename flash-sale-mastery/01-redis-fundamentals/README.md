# Module 01: Redis Fundamentals for Flash Sales

## Motivation: Why Redis for Flash Sales?

A flash sale is a race condition by design. Thousands of users hit the same
endpoint simultaneously, competing for a tiny pool of inventory. The hot path
-- check stock, record claim, decrement counter -- must complete in
**sub-millisecond** time or the system collapses under contention.

Redis is the only mainstream data store that offers:

- **Sub-millisecond reads and writes** on a single node (no disk I/O in the
  hot path)
- **Atomic operations** like DECR, SADD, and Lua scripts that eliminate the
  need for distributed locks
- **Built-in data structures** (strings, hashes, sets, sorted sets) that map
  directly to flash sale primitives
- **Key expiration** for self-cleaning ephemeral data (session tokens, rate
  limit windows, idempotency keys)

A traditional database with row-level locks would serialize every purchase
attempt, turning a 10-second flash sale into a 10-minute queue. Redis
parallelizes the same work across its single-threaded event loop, handling
100K+ operations per second on commodity hardware.

## Concept Map

```
                         ┌─────────────────────────────────────────┐
                         │          Flash Sale System              │
                         └────────────────┬────────────────────────┘
                                          │
            ┌─────────────┬───────────────┼───────────────┬──────────────┐
            │             │               │               │              │
     ┌──────▼──────┐ ┌────▼─────┐  ┌──────▼──────┐ ┌─────▼─────┐ ┌──────▼──────┐
     │   STRING    │ │   HASH   │  │    SET      │ │  SORTED   │ │   Pipeline  │
     │  (Stock)    │ │(Product) │  │ (Claims)    │ │   SET     │ │  (Batch)    │
     │             │ │          │  │             │ │(RateLimit)│ │             │
     │  GET/SET    │ │ HSET     │  │ SADD        │ │ ZADD      │ │  MGET/MSET  │
     │  INCR/DECR  │ │ HGET     │  │ SISMEMBER   │ │ ZCOUNT    │ │  batch ops  │
     │  SETEX      │ │ HGETALL  │  │ SREM        │ │ ZRANGEBY  │ │             │
     │  SETNX      │ │ HINCRBY  │  │ SCARD       │ │ ZREMRANGE │ │             │
     └──────┬──────┘ └────┬─────┘  └──────┬──────┘ └─────┬─────┘ └──────┬──────┘
            │             │               │               │              │
            └──────┬──────┴───────┬───────┴───────┬───────┘              │
                   │              │               │                      │
            ┌──────▼──────┐ ┌────▼─────┐  ┌──────▼──────┐       ┌──────▼──────┐
            │ Transaction │ │   TTL    │  │    Error    │       │ Resilience  │
            │ (WATCH/     │ │(Expiry)  │  │  Handling   │       │  (Retry)    │
            │  MULTI/     │ │ EXPIRE   │  │  Classify   │       │  Backoff    │
            │  EXEC)      │ │ TTL      │  │  Recover    │       │  Reconnect  │
            └─────────────┘ └──────────┘  └─────────────┘       └─────────────┘
```

## Theory: Data Types in Flash Sale Context

### STRING -- The Stock Counter

The simplest and fastest Redis type. A product's remaining stock is a single
integer value. Atomic DECR ensures no two buyers can grab the last unit
simultaneously.

```
SET  product:1001:stock  100
DECR product:1001:stock    → 99
DECR product:1001:stock    → 98
```

**When to use**: Simple counters, feature flags, session tokens, cache blobs.

### HASH -- The Product Catalog

A product has multiple fields (name, price, discount, description). A Redis
HASH stores them under one key with O(1) field access. HINCRBY atomically
updates individual fields without touching others.

```
HSET  product:1001  name "Widget"  price_cents 1999  discount_percent 50
HGET  product:1001  price_cents    → "1999"
HINCRBY product:1001  vouchers_issued  1
```

**When to use**: Structured objects, partial updates, counters per field.

### SET -- The Claim Registry

Each account can claim one unit per product. A Redis SET keyed by product
stores account IDs. SADD is idempotent (adding the same member twice is a
no-op), and SISMEMBER is O(1).

```
SADD      claims:1001  "user:5001"
SISMEMBER claims:1001  "user:5001"  → 1
SISMEMBER claims:1001  "user:9999"  → 0
SCARD     claims:1001               → 1
```

**When to use**: Unique membership, deduplication, tag systems.

### SORTED SET -- The Rate Limiter

A sliding window rate limiter tracks request timestamps. ZRANGEBYSCORE
counts entries in a time window. ZREMRANGEBYSCORE cleans up old entries.
The score is the timestamp; the member is a unique request ID.

```
ZADD    ratelimit:client:42  1700000000.0  "req:abc123"
ZCOUNT  ratelimit:client:42  1699999940    1700000000  → 1
ZREMRANGEBYSCORE ratelimit:client:42 -inf 1699999900
```

**When to use**: Leaderboards, rate limiting, time-series windows, priority
queues.

### Pipeline -- The Batch Optimizer

When the flash sale page needs stock for 50 products, sending 50 individual
GETs means 50 round-trips. A pipeline bundles all commands into one
round-trip. Redis processes them in order and returns all results together.

```
Pipeline:
  GET product:1001:stock
  GET product:1002:stock
  GET product:1003:stock
  → [100, 50, 0]
```

**When to use**: Bulk reads, batch initialization, dashboard queries.

### Transaction (WATCH/MULTI/EXEC) -- The Optimistic Lock

When you need to read a value, make a decision, and write it back without
another client sneaking in between, WATCH provides optimistic locking. If
the watched key changes before EXEC, the transaction aborts.

```
WATCH product:1001:stock
GET   product:1001:stock → 1
MULTI
SET   product:1001:stock 0
EXEC  → nil (another client changed it, abort!)
```

**When to use**: Conditional updates, multi-key consistency without Lua.

## Trade-offs: Redis vs Database for Hot Path

| Dimension            | Redis (Hot Path)           | Database (Cold Path)          |
|----------------------|----------------------------|-------------------------------|
| Latency              | < 1ms (in-memory)          | 5-50ms (disk + network)       |
| Throughput           | 100K+ ops/sec single node  | 1K-10K ops/sec per connection |
| Consistency          | Eventual (async replication)| ACID (with trade-offs)       |
| Durability           | Optional (AOF/RDB)         | Always (WAL)                  |
| Atomicity            | Per-command or Lua script  | Transaction with isolation    |
| Data model           | Key-value, structures      | Relational, joins             |
| Failure blast radius | Cache miss → DB fallback   | Full outage                   |

**The pattern**: Redis handles the hot path (stock checks, claim recording,
rate limiting). The database handles the cold path (order history, billing,
analytics). Writes flow from Redis to the database asynchronously via a
message queue or change stream.

## Failure Modes: What Happens When Redis Fails

1. **Connection timeout**: Client retries with exponential backoff. If all
   retries fail, return 503 Service Unavailable. Stock checks fail open
   (deny purchases) or fail closed (allow with post-verification) depending
   on business policy.

2. **Redis node crash**: If using replication, Sentinel or Cluster promotes
   a replica. There is a brief window (seconds) where writes fail. Idempotency
   keys in the database prevent duplicate orders during this window.

3. **Network partition**: Client cannot reach Redis. Same as connection
   timeout. The system must decide: block all purchases (safe but losing
   revenue) or allow with database-backed verification (risk of overselling).

4. **Memory exhaustion**: Redis starts evicting keys. If stock counters are
   evicted, the system may oversell. Set `maxmemory-policy noeviction` for
   flash sale data and monitor memory usage.

5. **Replication lag**: A read from a replica may return stale stock. Always
   read stock from the primary during active sales.

## Connection to Other Modules

```
01-redis-fundamentals
        │
        ├──→ 02-redis-lua-scripting
        │    Lua scripts replace WATCH/MULTI for complex atomic operations
        │    (check stock + check claim + decrement in one script)
        │
        ├──→ 03-atomic-counters
        │    Builds on STRING ops to implement distributed counters
        │    for real-time stock tracking across multiple services
        │
        ├──→ 04-traffic-shaping
        │    Uses SORTED SET rate limiting as the foundation for
        │    token bucket and leaky bucket algorithms
        │
        └──→ 07-resilience
             Uses error handling and retry patterns as building blocks
             for circuit breakers and bulkheads
```

## Exercise List

| #  | Exercise                   | Difficulty | Key Concepts                        |
|----|----------------------------|------------|-------------------------------------|
| 01 | Connection Pool            | Easy       | deadpool-redis, pool lifecycle      |
| 02 | String Operations          | Easy       | GET, SET, DECR, SETNX               |
| 03 | Hash Operations            | Easy       | HSET, HGET, HGETALL, HINCRBY       |
| 04 | Set Operations             | Medium     | SADD, SISMEMBER, SCARD, SREM       |
| 05 | Sorted Set Operations      | Medium     | ZADD, ZCOUNT, ZREMRANGEBYSCORE     |
| 06 | Pipeline Operations        | Medium     | Pipeline, batch get/set             |
| 07 | Transaction Basic          | Hard       | WATCH, MULTI, EXEC, optimistic lock |
| 08 | Key Expiration             | Easy       | EXPIRE, TTL, SET with EX/PX        |
| 09 | Error Handling             | Medium     | Error classification, recovery      |
| 10 | Connection Resilience      | Hard       | Retry, exponential backoff          |

## Running the Exercises

```bash
# Run all exercises (stubs with todo!() will panic)
cargo test -p redis-fundamentals

# Run with solutions enabled
cargo test -p redis-fundamentals --features solution

# Run a specific exercise
cargo test -p redis-fundamentals --features solution p02_string

# Run without Redis (tests that need Redis will be skipped)
cargo test -p redis-fundamentals --features solution
```

## References

- [Redis Data Types](https://redis.io/docs/latest/develop/data-types/) -- Official documentation
- [Redis Transactions](https://redis.io/docs/latest/develop/interact/transactions/) -- WATCH/MULTI/EXEC
- [Redis Pipelining](https://redis.io/docs/latest/develop/use/pipelining/) -- Batch operations
- [deadpool-redis](https://docs.rs/deadpool-redis/) -- Connection pool for Rust
- [redis-rs](https://docs.rs/redis/) -- Redis client for Rust
- [Redis Lua Scripting](https://redis.io/docs/latest/develop/interact/programmability/eval-intro/) -- For module 02
