# Exercise 03: Cache Stampede Prevention

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective
Implement and compare two cache stampede prevention strategies: lock-based (mutex) and probabilistic early expiration (XFetch). Measure the performance difference under concurrent load.

## Scenario
A popular product page has a cache entry that expires every 60 seconds. When the entry expires, 500 concurrent requests all miss the cache simultaneously and all hit the database. This causes a 10x spike in database CPU and increases latency from 50ms to 2,000ms.

## Tasks

### Part A: Implement Lock-Based Stampede Prevention
Write a Python function `get_product_with_lock(product_id)` that:
1. Checks the cache
2. On miss, attempts to acquire a distributed lock (using Redis `SET NX EX`)
3. If the lock is acquired: queries the database, populates the cache, releases the lock
4. If the lock is not acquired: waits 100ms and retries (up to 5 times)

<details>
<summary>Hint</summary>
Use `redis.set(lock_key, "1", nx=True, ex=10)` to acquire the lock atomically. `nx=True` means "set only if not exists." `ex=10` sets a 10-second expiry to prevent deadlocks. If the lock is acquired, query the database, populate the cache, then delete the lock key.
</details>

### Part B: Implement Probabilistic Early Expiration (XFetch)
Write a Python function `get_product_early_refresh(product_id)` that:
1. Checks the cache
2. If the entry exists and TTL is above a threshold, return it
3. If the entry exists but TTL is low, probabilistically refresh it before expiration
4. If the entry does not exist, query the database and populate the cache

<details>
<summary>Hint</summary>
Use `redis.ttl(key)` to get the remaining TTL. If TTL is below a threshold (e.g., 10 seconds), use a random probability to decide whether to refresh. The probability increases as TTL approaches 0. This spreads the refresh load over time instead of all requests hitting the database at once.
</details>

### Part C: Benchmark Both Strategies
Write a benchmark that:
1. Creates a cache entry with a 5-second TTL
2. Spawns 100 concurrent threads that all try to read the entry
3. Waits for the entry to expire
4. Measures total time and number of database queries for each strategy

<details>
<summary>Hint</summary>
Use `threading.Thread` to spawn concurrent readers. Track the number of database calls (each call increments a counter). Compare: without protection (100 DB calls), with lock (1 DB call), with XFetch (2-3 DB calls).
</details>

### Part D: Analyze the Results
Compare the three approaches:
1. Without protection: How many database queries? What is the total time?
2. With lock: How many database queries? What is the total time?
3. With XFetch: How many database queries? What is the total time?

Explain why each approach has different characteristics.

<details>
<summary>Hint</summary>
Without protection: 100 DB queries, fast total time but database overloaded. With lock: 1 DB query, slower total time (others wait for lock) but database protected. With XFetch: 2-3 DB queries, fast total time, database slightly loaded.
</details>

## Success Criteria
- [ ] The lock-based approach uses Redis SET NX EX for atomic lock acquisition.
- [ ] The XFetch approach uses TTL-based probabilistic refresh before expiration.
- [ ] The benchmark correctly measures database query count and total time.
- [ ] You can explain the trade-off between lock (fewer DB calls, higher latency) and XFetch (slightly more DB calls, lower latency).
- [ ] You can describe when to use each approach (lock for expensive queries, XFetch for moderate queries).

## What You Should Understand After This Exercise
Cache stampedes are a real problem when popular entries expire. Lock-based prevention is simple and effective but adds latency (other requests wait for the lock). XFetch spreads the refresh load over time but allows a small number of extra database queries. The choice depends on how expensive the database query is and how latency-sensitive the application is.
