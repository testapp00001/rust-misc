# Module 10: Caching Strategy

> A cache miss during a flash sale is not a minor slowdown -- it is a thundering
> herd that can take down your database.

## Motivation

Flash sales create extreme read pressure: tens of thousands of users hit the
same product page within seconds. Without a caching layer, every request
bounces to the database, which cannot sustain that throughput. But caching
introduces its own problems: stale data, cache stampedes, and the question of
what happens when the cache itself fails. This module teaches you to design
caching strategies that keep latency low while maintaining correctness under
flash sale conditions.

## Concept Map

```
User Request
      |
      v
+-----------------+      cache hit
| Check Cache     | ----------------> Return cached value
+-----------------+
      | cache miss
      v
+-----------------+      singleflight / mutex
| Load from       | <--- Only ONE request loads,
| Source (DB)     |      others wait
+-----------------+
      |
      v
+-----------------+
| Store in Cache  |      TTL / explicit invalidation
| (DashMap/Redis) |
+-----------------+
```

### Cache Patterns Compared

```
Cache-Aside (Lazy Loading)          Write-Through
  App reads cache                      App writes to cache
  Miss? -> read DB -> populate         Cache writes to DB synchronously
  Pros: Simple, only caches hot data   Pros: Cache always fresh
  Cons: Cache miss penalty             Cons: Write latency, cold data cached

Write-Behind (Write-Back)            Read-Through
  App writes to cache                  Cache loads from DB on miss
  Cache async writes to DB             App only talks to cache
  Pros: Low write latency              Pros: Simplified app code
  Cons: Data loss risk                 Cons: Cache is SPOF

Primary-Store Switch (p05)
  During sale: Redis is primary (fast, volatile)
  After sale:  DB is primary (durable, slower)
  Pros: Optimal for burst traffic
  Cons: Complex transition logic
```

## Theory

### Cache Warming

Before a flash sale starts, the cache should be pre-populated with product
data, stock counters, and sale configuration. This eliminates cold-start cache
misses that would otherwise cause a stampede at sale launch time.

**Key insight**: Warming is a batch operation. You know which products will be
on sale, so load them all at once rather than waiting for the first request.

### Thundering Herd (Cache Stampede)

When a popular cache entry expires, thousands of concurrent requests all miss
the cache simultaneously and all hit the database. This is the "thundering
herd" problem. Solutions include:

1. **Mutex-based stampede protection**: Only one request loads from DB; others
   wait for the result (Exercise p02)
2. **Singleflight**: Deduplicate in-flight requests so only one execution
   happens per key (Exercise p03)
3. **Probabilistic early expiration**: Refresh the cache entry *before* it
   expires, based on a probability function

### Singleflight Pattern

Singleflight ensures that for a given key, at most one request is in-flight
at any time. Concurrent callers for the same key receive the same result
without triggering redundant database queries. This is the Go
`golang.org/x/sync/singleflight` pattern adapted for async Rust.

### Cache Invalidation

> "There are only two hard things in Computer Science: cache invalidation and
> naming things." -- Phil Karlton

Strategies:
- **TTL-based**: Entries expire after a fixed duration (simple, eventual consistency)
- **Event-driven**: Invalidate when the source data changes (complex, stronger consistency)
- **Pattern-based**: Invalidate groups of related entries (e.g., all stock for a product)

### Post-Sale Transition

After a flash sale ends, the system must transition from "Redis as primary
store" back to "database as primary store." This involves:

1. **Drain**: Read all remaining data from Redis
2. **Sync**: Write drained data to the database, resolving conflicts
3. **Switch**: Change the read path to use the database
4. **Cleanup**: Remove Redis keys that are no longer needed

## Trade-offs

| Decision | Option A | Option B | Flash Sale Guidance |
|----------|----------|----------|---------------------|
| Cache location | In-memory (DashMap) | Redis | In-memory for single-node; Redis for distributed |
| Invalidation | TTL-based | Event-driven | TTL for simplicity; event-driven for correctness |
| Cache miss | Fail fast | Block and wait | Block with singleflight to protect DB |
| Consistency | Strong | Eventual | Eventual is acceptable for product info; strong for stock |
| Primary store | DB always primary | Redis primary during sale | Redis-primary for extreme throughput |

## Failure Modes

1. **Cache Stampede**: Thousands of concurrent cache misses overwhelm the DB
   - Mitigation: Singleflight, mutex-based stampede protection

2. **Stale Data**: Cached value differs from source of truth
   - Mitigation: Short TTL for stock, event-driven invalidation

3. **Cache as SPOF**: If the cache layer fails, all requests hit DB
   - Mitigation: Circuit breaker, graceful degradation, DB read replicas

4. **Inconsistent Transition**: During post-sale drain, some reads hit Redis
   and others hit DB, returning different stock values
   - Mitigation: Fence the transition with a mode flag, drain before switching

5. **Memory Pressure**: Caching too much data causes OOM
   - Mitigation: LRU eviction, size-bounded caches, cache only hot data

## Connection to Other Modules

- **Module 01 (Redis Fundamentals)**: Redis data types underpin the cache
  layer. Understanding STRING, HASH, and key expiration is prerequisite.
- **Module 03 (Atomic Counters)**: Stock counters in the cache must be updated
  atomically to prevent overselling.
- **Module 11 (Flash Sale API)**: The API layer is where caching decisions are
  made -- cache headers, cache-aside middleware, and stock reads all live here.

## Exercises

| # | Exercise | Focus |
|---|----------|-------|
| 01 | Cache Warming | Pre-populating cache before sale launch |
| 02 | Thundering Herd | Mutex-based stampede protection |
| 03 | Singleflight | Deduplicating concurrent requests for same key |
| 04 | Cache-Aside | Lazy loading with TTL and invalidation |
| 05 | Primary vs Cache | Redis-as-primary during sale, DB after |
| 06 | Invalidation | Key, pattern, and event-driven invalidation |
| 07 | Transition Strategy | Post-sale drain, sync, and mode switch |
| 08 | Cache Benchmark | Measuring hit vs miss latency and throughput |

## References

- [Facebook: Scaling Memcache at Facebook](https://www.usenix.org/system/files/conference/nsdi13/nsdi13-final170_update.pdf)
- [Google: Singleflight pattern (Go)](https://pkg.go.dev/golang.org/x/sync/singleflight)
- [Martin Kleppmann: Designing Data-Intensive Applications, Ch. 5](https://dataintensive.net/)
- [Redis: Cache-Aside Pattern](https://redis.io/glossary/cache-aside-pattern/)
- [AWS: Caching Best Practices](https://aws.amazon.com/caching/best-practices/)
