# Solution 03: Cache Stampede Prevention

## Part A: Lock-Based Stampede Prevention

```python
import time
import redis
import json

r = redis.Redis(host='localhost', port=6379, decode_responses=True)

def slow_db_query(product_id: int) -> dict:
    """Simulates a slow database query (500ms)."""
    time.sleep(0.5)
    return {"id": product_id, "name": f"Product {product_id}", "price": 29.99}

def get_product_with_lock(product_id: int, max_retries: int = 5) -> dict:
    """Cache-aside with distributed lock to prevent stampedes."""
    cache_key = f"product:{product_id}"
    lock_key = f"lock:{cache_key}"

    # Step 1: Check cache
    cached = r.get(cache_key)
    if cached:
        return json.loads(cached)

    # Step 2: Try to acquire lock
    for attempt in range(max_retries):
        # SET NX: set only if not exists. EX 10: expire in 10 seconds.
        if r.set(lock_key, "1", nx=True, ex=10):
            try:
                # Lock acquired — query database
                product = slow_db_query(product_id)

                # Populate cache with 60-second TTL
                r.setex(cache_key, 60, json.dumps(product))

                return product
            finally:
                # Always release the lock
                r.delete(lock_key)
        else:
            # Lock not acquired — another request is rebuilding the cache
            # Wait and retry (cache should be populated soon)
            time.sleep(0.1)

            # Check cache again (another request may have populated it)
            cached = r.get(cache_key)
            if cached:
                return json.loads(cached)

    # Max retries exhausted — fall through to direct query
    return slow_db_query(product_id)
```

**Why this works**: `redis.set(key, "1", nx=True, ex=10)` atomically sets the key only if it does not exist. This ensures only one request acquires the lock. The `ex=10` prevents deadlocks if the lock holder crashes. Other requests wait and retry, checking the cache on each retry.

---

## Part B: Probabilistic Early Expiration (XFetch)

```python
import random
import time

def get_product_early_refresh(product_id: int) -> dict:
    """Cache with probabilistic early expiration to prevent stampedes."""
    cache_key = f"product:{product_id}"

    # Step 1: Check cache
    cached = r.get(cache_key)
    if cached:
        data = json.loads(cached)
        ttl = r.ttl(cache_key)

        # Step 2: Probabilistic early refresh
        # If TTL is low (< 10 seconds), there is a chance we refresh early
        if ttl is not None and ttl < 10:
            # Probability of refresh increases as TTL approaches 0
            # At TTL=10: ~0% chance. At TTL=1: ~90% chance.
            refresh_probability = (10 - ttl) / 10.0
            if random.random() < refresh_probability:
                # Early refresh — query database before cache expires
                product = slow_db_query(product_id)
                r.setex(cache_key, 60, json.dumps(product))
                return product

        return data  # Serve from cache

    # Step 3: Cache miss — query database
    product = slow_db_query(product_id)
    r.setex(cache_key, 60, json.dumps(product))
    return product
```

**Why this works**: Instead of waiting for the cache to expire and then all requests hitting the database simultaneously, XFetch allows some requests to refresh the cache before it expires. The probability increases as TTL decreases. This spreads the refresh load over time, preventing a stampede.

---

## Part C: Benchmark

```python
import threading
import time
import redis
import json

r = redis.Redis(host='localhost', port=6379, decode_responses=True)

# Track database calls
db_call_count = 0
db_call_lock = threading.Lock()

def slow_db_query(product_id: int) -> dict:
    """Simulates a slow database query."""
    global db_call_count
    with db_call_lock:
        db_call_count += 1
    time.sleep(0.1)  # Simulate 100ms query
    return {"id": product_id, "name": f"Product {product_id}"}

def benchmark(get_func, label, num_threads=100):
    """Benchmark a cache function with concurrent access."""
    global db_call_count
    db_call_count = 0

    # Clear cache and set up entry that will expire
    r.delete("product:1")
    r.setex("product:1", 5, json.dumps({"id": 1, "name": "Product 1"}))

    # Wait for cache to expire
    time.sleep(5.5)

    results = []
    start = time.time()

    def worker():
        result = get_func(1)
        results.append(result)

    # Spawn concurrent threads
    threads = [threading.Thread(target=worker) for _ in range(num_threads)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()

    elapsed = time.time() - start
    print(f"{label}:")
    print(f"  Total time: {elapsed:.2f}s")
    print(f"  DB calls: {db_call_count}")
    print(f"  Threads: {num_threads}")
    print()

# Run benchmarks
benchmark(get_without_protection, "Without protection")
benchmark(get_product_with_lock, "With lock")
benchmark(get_product_early_refresh, "With XFetch")
```

---

## Part D: Analysis

### Expected Results

| Approach | DB Calls | Total Time | Description |
|----------|----------|------------|-------------|
| Without protection | 100 | ~0.2s | All threads query DB concurrently |
| With lock | 1 | ~0.5s | Only 1 thread queries DB; others wait |
| With XFetch | 2-3 | ~0.3s | A few threads refresh early |

### Why the Differences

**Without protection**: All 100 threads miss the cache simultaneously. Each queries the database. The database receives 100 concurrent queries, taking ~100ms each (they run in parallel). Total time is ~200ms (overhead + parallel queries). This is the stampede.

**With lock**: One thread acquires the lock and queries the database (100ms). The other 99 threads wait. After the first thread populates the cache, the waiting threads find the cache populated and return immediately. Total time is ~500ms (lock wait + DB query). Database receives only 1 query.

**With XFetch**: As the cache TTL approaches 0, some threads probabilistically refresh early. Typically 2-3 threads refresh before the cache fully expires. Total time is ~300ms. Database receives 2-3 queries (spread over time).

---

## Common Mistakes
1. **Not releasing the lock on exception**: If the database query throws an exception, the lock is never released. Use `try/finally` to ensure the lock is always released.
2. **Lock expiry too short**: If the database query takes longer than the lock expiry, another request may acquire the lock and query the database simultaneously. Set lock expiry to at least 2x the expected query time.
3. **Not checking cache after acquiring lock**: Another request may have populated the cache while waiting for the lock. Check the cache again after acquiring the lock.
4. **XFetch with expensive queries**: XFetch allows a few extra database queries. If the query is very expensive (seconds), use the lock-based approach instead.

## Relevant README Sections
- [Cache Stampede (Thundering Herd)](../README.md#cache-stampede-thundering-herd)
- [Lock-Based (Mutex)](../README.md#lock-based-mutex)
- [Probabilistic Early Expiration (XFetch)](../README.md#probabilistic-early-expiration-xfetch)
