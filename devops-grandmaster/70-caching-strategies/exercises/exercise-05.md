# Exercise 05: Production Caching System

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective
Design and implement a complete production caching system that combines cache-aside, write-through, CDN, Redis Cluster, cache warming, stampede prevention, and monitoring into a cohesive architecture.

## Scenario
You are building the caching infrastructure for an e-commerce platform that serves 50,000 RPS. The system must:
- Cache product catalog (read-heavy, rarely updated)
- Cache user sessions (write-heavy, per-user)
- Cache API responses (mixed read/write)
- Serve static assets from CDN
- Handle cache stampedes on popular products
- Monitor cache effectiveness across all layers

## Tasks

### Part A: Design the Complete Architecture
Create an ASCII architecture diagram showing:
1. CDN layer (CloudFront/Cloudflare)
2. Application-level cache (in-process LRU)
3. Distributed cache (Redis Cluster)
4. Database (PostgreSQL)
5. Cache invalidation flow
6. Monitoring integration

<details>
<summary>Hint</summary>
The request flow is: CDN -> Application -> Redis -> Database. Cache invalidation flows: Database change -> Application invalidates Redis -> CDN purge. Monitoring collects hit rates from each layer.
</details>

### Part B: Implement the Cache Manager
Write a `ProductionCacheManager` class that:
1. Supports multiple cache strategies (cache-aside, write-through) per data type
2. Uses consistent hashing for Redis Cluster key distribution
3. Implements cache warming on startup
4. Implements stampede prevention (lock-based)
5. Tracks hit/miss/latency metrics per cache namespace
6. Supports manual cache invalidation by key or pattern

<details>
<summary>Hint</summary>
Use a dictionary to map cache namespaces to strategies. Use Redis Cluster client for distributed caching. Use `redis.set(key, "1", nx=True, ex=10)` for distributed locks. Track metrics using Redis counters with namespace prefixes.
</details>

### Part C: Implement Cache Invalidation Pipeline
Write an invalidation pipeline that:
1. Receives invalidation events (via Redis pub/sub or message queue)
2. Invalidates the local in-process cache
3. Invalidates the Redis cache
4. Sends CDN purge request (if applicable)
5. Logs the invalidation for audit

<details>
<summary>Hint</summary>
Use Redis pub/sub: publish invalidation events to a `cache:invalidate` channel. Each application instance subscribes and invalidates its local cache. For CDN purge, use the CloudFront or Cloudflare API to invalidate specific paths.
</details>

### Part D: Implement Cache Warming
Write a cache warming system that:
1. Pre-populates the cache with the top 1,000 products on startup
2. Pre-populates active user sessions (users active in the last 30 minutes)
3. Reports warming progress and completion time
4. Handles warming failures gracefully (log and continue)

<details>
<summary>Hint</summary>
Query the database for top products (by view count) and active users (by last access time). Use Redis pipeline for batch writes. Use a progress counter to track warming status. Catch and log exceptions per entry to prevent one failure from stopping the entire warmup.
</details>

### Part E: Create Monitoring Dashboard Specification
Write a specification for a Grafana dashboard that shows:
1. Cache hit rates by layer (L1, L2, CDN)
2. Cache latency by layer
3. Cache memory usage (Redis)
4. Invalidation events per minute
5. Stampede prevention events (lock acquisitions)
6. Top 10 most-missed cache keys

<details>
<summary>Hint</summary>
Use Prometheus queries for each panel. Hit rate: `sum(rate(cache_hits[5m])) / sum(rate(cache_requests[5m])) * 100`. Latency: `histogram_quantile(0.95, sum(rate(cache_latency_bucket[5m])) by (le, layer))`. Memory: `redis_memory_used_bytes`.
</details>

## Success Criteria
- [ ] The architecture diagram shows all cache layers with data flow.
- [ ] The cache manager supports multiple strategies per data type.
- [ ] Cache warming pre-populates products and sessions on startup.
- [ ] The invalidation pipeline handles local, Redis, and CDN invalidation.
- [ ] The monitoring specification covers hit rates, latency, and invalidation events.

## What You Should Understand After This Exercise
A production caching system is not just "add Redis." It requires careful design: what to cache at each layer, how to invalidate, how to prevent stampedes, how to warm the cache, and how to monitor effectiveness. The goal is to maximize cache hit rate while maintaining data consistency. Every percentage point of hit rate improvement reduces database load and improves latency.
