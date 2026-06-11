# Exercise 04: Multi-Layer Cache Architecture

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective
Design and implement a multi-layer caching system that combines L1 (in-process), L2 (Redis), and L3 (CDN) caches for maximum performance and minimum origin load.

## Scenario
Your API serves 100,000 requests per second. The breakdown:
- 60% are for static product data (changes rarely)
- 30% are for user-specific data (session, cart)
- 10% are for real-time data (inventory, prices)

You need a multi-layer cache that:
- Serves static data from CDN (L3) or in-process cache (L1)
- Serves user data from Redis (L2)
- Serves real-time data from the origin (no caching)

## Tasks

### Part A: Design the Cache Layers
Design a three-layer cache architecture with:
1. **L1 (In-process)**: Python dict with TTL. Fastest, but not shared across instances.
2. **L2 (Redis)**: Shared across all instances. Slower than L1, but consistent.
3. **L3 (CDN)**: Edge cache for static content. Slowest cache hit, but offloads origin completely.

For each layer, specify:
- What data is cached
- TTL
- Invalidation strategy
- Expected hit rate

<details>
<summary>Hint</summary>
L1 caches hot data with short TTL (30 seconds). L2 caches all cacheable data with medium TTL (5 minutes). L3 caches static assets with long TTL (1 hour). L1 is per-instance (fastest), L2 is shared (consistent), L3 is global (highest offload).
</details>

### Part B: Implement the Multi-Layer Cache
Write a Python `MultiLayerCache` class that:
1. Checks L1 (in-process dict) first
2. On L1 miss, checks L2 (Redis)
3. On L2 miss, loads from origin (database)
4. Backfills both L1 and L2 on origin access
5. Supports invalidation at all layers

<details>
<summary>Hint</summary>
The `get()` method checks L1, then L2, then origin. On origin access, populate both L2 (with longer TTL) and L1 (with shorter TTL). The `invalidate()` method removes from both L1 and L2.
</details>

### Part C: Add CDN Integration
Write Nginx configuration that:
1. Caches `/api/products/*` responses for 5 minutes at the CDN
2. Caches `/static/*` for 1 year
3. Never caches `/api/user/*` (user-specific)
4. Adds appropriate `Cache-Control` and `Vary` headers

<details>
<summary>Hint</summary>
Use `add_header Cache-Control "public, max-age=300"` for product API. Use `add_header Cache-Control "public, max-age=31536000, immutable"` for static assets. Use `add_header Cache-Control "private, no-store"` for user-specific endpoints. Add `Vary: Authorization` for endpoints that vary by auth.
</details>

### Part D: Measure Cache Effectiveness
Write a monitoring script that reports:
1. L1 hit rate, L2 hit rate, and CDN hit rate
2. Average latency for L1 hits, L2 hits, and origin misses
3. Overall cache efficiency (total hits / total requests)
4. Origin offload percentage (requests that did NOT hit the origin)

<details>
<summary>Hint</summary>
Track hits and misses at each layer using Redis counters. Calculate hit rate as `hits / (hits + misses) * 100`. Origin offload = `(L1_hits + L2_hits + CDN_hits) / total_requests * 100`.
</details>

## Success Criteria
- [ ] The multi-layer cache checks L1, then L2, then origin in order.
- [ ] On origin access, both L1 and L2 are populated (backfill).
- [ ] Invalidation removes entries from all layers.
- [ ] CDN configuration uses appropriate Cache-Control headers for each content type.
- [ ] Monitoring reports hit rates and latency for each layer.

## What You Should Understand After This Exercise
Multi-layer caching maximizes performance by serving data from the closest, fastest cache. L1 is fastest but smallest and not shared. L2 is shared but requires network calls. L3 (CDN) offloads the origin completely but has the highest latency on miss. The key is assigning the right data to the right layer based on access patterns and consistency requirements.
