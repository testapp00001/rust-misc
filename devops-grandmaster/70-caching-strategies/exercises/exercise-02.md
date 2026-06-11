# Exercise 02: Cache-Aside with Redis

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective
Implement the cache-aside pattern using Redis, including cache reads, cache writes with TTL, cache invalidation on updates, and monitoring cache hit rates.

## Scenario
You have a user service that frequently reads user profiles from a PostgreSQL database. You need to add a Redis cache layer to reduce database load. The cache should:
- Cache user profiles for 1 hour (3600 seconds)
- Invalidate the cache when a user is updated
- Track cache hit/miss rates for monitoring

## Tasks

### Part A: Implement Cache-Aside Read
Write a Python function `get_user(user_id)` that:
1. Checks Redis for the cached user profile
2. On cache hit: returns the cached data (deserialized from JSON)
3. On cache miss: queries the database, caches the result with a 1-hour TTL, and returns the data

<details>
<summary>Hint</summary>
Use `redis.get(key)` to check the cache. If the result is not None, deserialize with `json.loads()`. If None, query the database, serialize the result with `json.dumps()`, and store with `redis.setex(key, ttl, value)`.
</details>

### Part B: Implement Cache Invalidation
Write a Python function `update_user(user_id, data)` that:
1. Updates the user in the database
2. Invalidates the cache entry for that user
3. Returns the updated user data

<details>
<summary>Hint</summary>
After updating the database, call `redis.delete(f"user:{user_id}")` to invalidate the cache. The next `get_user()` call will miss the cache and repopulate it with fresh data.
</details>

### Part C: Implement Cache Monitoring
Write a `MonitoredCache` class that wraps Redis and tracks:
1. Number of cache hits
2. Number of cache misses
3. Hit rate (hits / (hits + misses))
4. Average cache read latency

<details>
<summary>Hint</summary>
Use Redis `INCR` to track hits and misses. Store latency measurements in a Redis list or use a separate counter. The hit rate is `hits / (hits + misses) * 100`.
</details>

### Part D: Implement Cache Warming
Write a function `warm_cache()` that pre-populates the cache with the 100 most frequently accessed users on application startup.

<details>
<summary>Hint</summary>
Query the database for the top 100 users (by access count or most recent). For each user, serialize and store in Redis with `setex()`. This eliminates cold-start cache misses.
</details>

## Success Criteria
- [ ] The cache-aside read function checks Redis first and falls back to the database.
- [ ] Cache entries have a 1-hour TTL to prevent indefinite staleness.
- [ ] The update function invalidates the cache entry after updating the database.
- [ ] The monitoring class tracks hits, misses, and hit rate.
- [ ] The cache warming function pre-populates frequently accessed entries.

## What You Should Understand After This Exercise
The cache-aside pattern is simple but requires the application to manage the cache explicitly. Every read must check the cache first. Every write must invalidate the cache. Monitoring hit rates is essential -- a low hit rate means the cache is not effective (wrong TTL, wrong keys, or insufficient size).
