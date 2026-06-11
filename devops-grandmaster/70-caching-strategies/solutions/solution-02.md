# Solution 02: Cache-Aside with Redis

## Part A: Cache-Aside Read

```python
import redis
import json
import time

r = redis.Redis(host='localhost', port=6379, decode_responses=True)

def get_user(user_id: int) -> dict:
    """Cache-aside pattern: check cache first, fall back to database."""
    cache_key = f"user:{user_id}"

    # Step 1: Check cache
    cached = r.get(cache_key)
    if cached:
        return json.loads(cached)  # Cache hit

    # Step 2: Cache miss — query database
    user = db.query("SELECT * FROM users WHERE id = ?", user_id)

    # Step 3: Populate cache with 1-hour TTL
    r.setex(cache_key, 3600, json.dumps(user))

    return user
```

**Why this works**: `redis.get()` returns None on cache miss. `redis.setex()` atomically sets the value and expiration time. The 3600-second (1-hour) TTL ensures stale data does not persist indefinitely.

---

## Part B: Cache Invalidation

```python
def update_user(user_id: int, data: dict) -> dict:
    """Update user and invalidate cache."""
    # Step 1: Update database FIRST
    db.execute(
        "UPDATE users SET name = %s, email = %s WHERE id = %s",
        data['name'], data['email'], user_id
    )

    # Step 2: Invalidate cache AFTER database update
    r.delete(f"user:{user_id}")

    # Step 3: Return fresh data
    return db.query("SELECT * FROM users WHERE id = ?", user_id)
```

**Why this works**: The database is updated first, then the cache is invalidated. This order ensures that if the cache invalidation fails, the stale cache entry will eventually expire via TTL. If we invalidated the cache first and the database update failed, the cache would be empty while the database has old data.

---

## Part C: Cache Monitoring

```python
import time

class MonitoredCache:
    """Redis cache wrapper with hit/miss/latency tracking."""

    def __init__(self, redis_client, prefix="app"):
        self.redis = redis_client
        self.prefix = prefix

    def get(self, key: str):
        """Get with monitoring."""
        start = time.time()
        result = self.redis.get(key)
        elapsed_ms = (time.time() - start) * 1000

        if result:
            self.redis.incr(f"{self.prefix}:cache:hits")
            self.redis.incr(f"{self.prefix}:cache:hit_latency_ms", int(elapsed_ms))
        else:
            self.redis.incr(f"{self.prefix}:cache:misses")

        return result

    def set(self, key: str, value: str, ttl: int = 3600):
        """Set with TTL."""
        self.redis.setex(key, ttl, value)

    def get_stats(self) -> dict:
        """Get cache statistics."""
        hits = int(self.redis.get(f"{self.prefix}:cache:hits") or 0)
        misses = int(self.redis.get(f"{self.prefix}:cache:misses") or 0)
        total = hits + misses
        hit_rate = (hits / total * 100) if total > 0 else 0

        hit_latency_sum = int(
            self.redis.get(f"{self.prefix}:cache:hit_latency_ms") or 0
        )
        avg_latency = (hit_latency_sum / hits) if hits > 0 else 0

        return {
            "hits": hits,
            "misses": misses,
            "total": total,
            "hit_rate_percent": round(hit_rate, 2),
            "avg_hit_latency_ms": round(avg_latency, 2),
        }
```

**Why this works**: Redis `INCR` is atomic and fast (O(1)). Tracking hits and misses as Redis counters means the monitoring data survives application restarts. The hit rate formula is `hits / (hits + misses) * 100`.

---

## Part D: Cache Warming

```python
def warm_cache():
    """Pre-populate cache with top 100 users on startup."""
    print("Warming cache with top 100 users...")

    # Query database for most frequently accessed users
    top_users = db.query(
        "SELECT * FROM users ORDER BY access_count DESC LIMIT 100"
    )

    # Use pipeline for batch writes (single round trip)
    pipe = r.pipeline()
    warmed = 0
    for user in top_users:
        try:
            cache_key = f"user:{user['id']}"
            pipe.setex(cache_key, 3600, json.dumps(user))
            warmed += 1
        except Exception as e:
            print(f"Failed to warm cache for user {user['id']}: {e}")

    pipe.execute()  # Execute all commands in one round trip
    print(f"Cache warmed with {warmed} users")
```

**Why this works**: Redis pipeline batches multiple commands into a single round trip, reducing network overhead from 100 round trips to 1. The `pipe.execute()` sends all commands at once. Exception handling ensures one failed entry does not stop the entire warmup.

---

## Common Mistakes
1. **Invalidating cache before database update**: If the database update fails, the cache is empty but the database has old data. Always update the database first.
2. **Not setting TTL**: Without TTL, cache entries persist forever. If invalidation is missed, stale data is served indefinitely.
3. **Not using pipelines for batch operations**: Without pipelines, 100 cache writes require 100 round trips. With pipelines, it is 1 round trip.
4. **Ignoring cache hit rate**: A hit rate below 80% means the cache is not effective. Check TTL, key design, and cache size.
5. **Not handling Redis failures**: If Redis is down, the application must fall back to the database. Use try/except around cache operations.

## Relevant README Sections
- [Cache-Aside (Lazy Loading)](../README.md#cache-aside-lazy-loading)
- [Monitoring Cache Effectiveness](../README.md#monitoring-cache-effectiveness)
- [Cache Warming](../README.md#cache-warming)
