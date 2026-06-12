# Module 04: Traffic Shaping and Rate Limiting

> Protect your flash sale system from overload with multiple layers of
> rate limiting, admission control, and graceful degradation.

## Motivation

When a flash sale opens, traffic can spike from near-zero to 500,000 requests
per second in under a minute. Without traffic shaping, this tsunami of requests
would exhaust database connections, saturate CPU, and trigger cascading failures
across every service in the stack. The result: the entire platform goes down,
not just the flash sale.

Traffic shaping is the art of saying "no" efficiently. By controlling the rate
at which requests enter the system, you ensure that the requests you *do* process
complete successfully and quickly. A well-shaped system can serve 90% of users
within SLA while gracefully handling the remaining 10%.

## Concept Map

```
Incoming Request (100K+/sec)
        |
        v
+---------------------+
| Layer 1: Per-IP     |  <-- p09: Token bucket per client IP
| Rate Limit          |      Blocks bots and single-machine floods
+---------------------+
        |
        v
+---------------------+
| Layer 2: Per-Account|  <-- p09: Token bucket per user account
| Rate Limit          |      Prevents single-user hogging
+---------------------+
        |
        v
+---------------------+
| Layer 3: Admission  |  <-- p05: Admit / Queue / Reject
| Control             |      Considers load, stock, reputation
+---------------------+
        |
   +---------+---------+
   |                   |
   v                   v
 Admitted           Waiting Room  <-- p06: HTTP 202 + position
   |                   |
   v                   v
+-------------------+  |
| Backpressure      |  |  <-- p07: Bounded channels
| Channel           |  |      Prevents queue explosion
+-------------------+  |
   |                   |
   v                   v
+-------------------+  |
| Leaky Bucket      |  |  <-- p04: Constant output rate
| (smooth output)   |  |      Protects downstream services
+-------------------+  |
   |                   |
   v                   v
+-------------------+  |
| Process Order     |  |
+-------------------+  |
                        |
  Degraded responses <--+-- p08: Serve sold-out, 503, stale data
```

## Theory

### Rate Limiting Algorithms

| Algorithm | Burst Handling | Memory | Precision | Best For |
|-----------|---------------|--------|-----------|----------|
| Token Bucket (p01) | Allows bursts up to capacity | O(1) | Exact | API gateway, per-IP limits |
| Sliding Window Log (p02) | No burst smoothing | O(n) | Exact | Low-volume, high-value ops |
| Sliding Window Counter (p03) | Slight smoothing | O(1) | Approximate | High-throughput counters |
| Leaky Bucket (p04) | Absorbs bursts, constant output | O(1) | Exact | Downstream protection |

### Queuing Theory Basics

**Little's Law**: L = lambda * W

- L = average number of requests in the system
- lambda = average arrival rate (requests/sec)
- W = average time a request spends in the system

For a flash sale with lambda = 100K req/sec and a target W of 5ms:
L = 100,000 * 0.005 = 500 concurrent requests in flight.

This means your system needs to handle at least 500 concurrent requests.
If your capacity is only 200, you need to either:
1. **Reduce lambda** -- rate limiting, admission control
2. **Reduce W** -- optimize processing time
3. **Increase capacity** -- scale horizontally

### Token Bucket vs Leaky Bucket

**Token Bucket**: Tokens accumulate when idle, allowing bursts. Good for APIs
where burst tolerance improves user experience.

**Leaky Bucket**: Drains at a constant rate regardless of input. Good for
protecting services that need predictable load (databases, payment processors).

### The Sliding Window Problem

Fixed windows have a boundary problem: a user who makes 100 requests at T=59s
and 100 more at T=61s appears to have made 200 requests in 2 seconds, but each
60-second window only shows 100. The sliding window (either log or counter)
solves this by weighting requests across window boundaries.

## Trade-offs

### Precision vs Memory vs CPU

| Approach | Memory | CPU per Check | Precision |
|----------|--------|---------------|-----------|
| Token Bucket | 24 bytes (4 fields) | ~50ns | Exact for burst/Rate |
| Sliding Window Log | 8 bytes * N requests | O(n) cleanup | Exact count |
| Sliding Window Counter | 24 bytes (3 fields) | ~100ns | ~95% accurate |
| Leaky Bucket | 24 bytes (4 fields) | ~50ns | Exact for drain rate |

At 100K req/sec, the log approach consumes 800KB/sec of timestamp storage
per rate limit key. The counter approach uses 24 bytes total, period.

### Centralized vs Distributed Rate Limiting

- **In-process** (this module): Fastest (~50ns), but each process has its own
  limit. 10 processes with 100 req/sec each = 1000 req/sec effective limit.
- **Redis-based**: Shared state across processes (~0.5ms per check), but adds
  network latency and Redis becomes a single point of failure.
- **Hybrid**: In-process first, periodic sync with Redis. Best of both worlds
  but more complex.

## Failure Modes

1. **Rate Limiter as Bottleneck**: The rate limiter itself becomes the
   bottleneck under extreme load.
   - Mitigation: Use lock-free data structures (DashMap), O(1) algorithms

2. **Clock Skew in Distributed Systems**: Different nodes see different times,
   causing inconsistent rate limits.
   - Mitigation: Use monotonic clocks (Instant), not wall clocks

3. **Rate Limiter Memory Leak**: Per-key buckets accumulate and never expire.
   - Mitigation: Periodic cleanup of stale entries, LRU eviction

4. **Thundering Herd on Refill**: All blocked requests succeed simultaneously
   when the bucket refills.
   - Mitigation: Jittered retry, randomized token distribution

5. **False Rejection Under Load**: Legitimate users get rate limited because
   the system can't distinguish bots from humans.
   - Mitigation: Per-account limits (not just per-IP), reputation scoring

## Connection to Other Modules

- **Module 01 (Redis Fundamentals)**: Redis-based rate limiting uses sorted
  sets and Lua scripts for distributed counters
- **Module 02 (Redis Lua Scripting)**: Atomic check-and-decrement for
  distributed rate limit operations
- **Module 11 (Flash Sale API)**: The rate limit middleware (p10) plugs
  directly into the Axum API layer as a Tower middleware

## Exercises

| # | Exercise | Focus |
|---|----------|-------|
| 01 | Token Bucket | Classic rate limiter, burst tolerance, lazy refill |
| 02 | Sliding Window Log | Exact counting, timestamp storage, window cleanup |
| 03 | Sliding Window Counter | Approximate counting, memory efficiency |
| 04 | Leaky Bucket | Constant output rate, burst absorption |
| 05 | Admission Control | Business-aware decisions: admit/queue/reject |
| 06 | Virtual Waiting Room | FIFO queue, batch release, HTTP 202 |
| 07 | Backpressure Channel | Tokio bounded channels, try_send vs timeout |
| 08 | Graceful Degradation | Fast fallback responses, feature flags |
| 09 | Multi-Layer Rate Limit | Composing limiters: IP, account, product, global |
| 10 | Rate Limit Middleware | Tower Layer/Service, Axum integration, 429 responses |

## References

- [IETF RFC 6585: Additional HTTP Status Codes (429)](https://tools.ietf.org/html/rfc6585)
- [Stripe Rate Limiting](https://stripe.com/blog/rate-limiters)
- [Cloudflare: How we built rate limiting](https://blog.cloudflare.com/counting-things-a-lot-of-different-things/)
- [Google Cloud: Rate Limiting Strategies and Techniques](https://cloud.google.com/architecture/rate-limiting-strategies-techniques)
- [AWS: Rate Limiting](https://docs.aws.amazon.com/apigateway/latest/developerguide/api-gateway-request-throttling.html)
- [Token Bucket Algorithm (Wikipedia)](https://en.wikipedia.org/wiki/Token_bucket)
- [Leaky Bucket Algorithm (Wikipedia)](https://en.wikipedia.org/wiki/Leaky_bucket)
- [Little's Law (Wikipedia)](https://en.wikipedia.org/wiki/Little%27s_law)
- [Tower Service Documentation](https://docs.rs/tower/latest/tower/trait.Service.html)
