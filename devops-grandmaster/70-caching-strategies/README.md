# 70 - Caching Strategies

> **Previous:** [69 - Capacity Planning](../69-capacity-planning/README.md)
> **Next:** [71 - Performance Tuning](../71-performance-tuning/README.md)

## The Problem

Your application repeatedly fetches the same data — database queries, API responses, computed results. Every request hits the origin server, burning CPU, I/O, and network bandwidth. Latency climbs under load. The database becomes a bottleneck. You need a way to store frequently accessed data closer to the consumer so that repeated reads are fast and cheap.

Caching is the single most impactful performance optimization you can make. A well-designed cache layer can reduce latency by 10-100x and offload 80-95% of read traffic from your database. But caching done wrong introduces stale data, thundering herds, and distributed inconsistencies that are harder to debug than the original slowness.

---

## The Naive Way

Cache everything at the application level with no strategy.

```python
# A global dictionary acting as a cache — no eviction, no TTL, no distributed awareness
cache = {}

def get_user(user_id):
    if user_id in cache:
        return cache[user_id]

    user = db.query("SELECT * FROM users WHERE id = ?", user_id)
    cache[user_id] = user  # grows forever, never expires
    return user
```

**Why this fails:**
- Memory grows without bound until the process crashes (OOM).
- No expiration — stale data persists indefinitely.
- Not distributed — each application instance has its own cache, so instances serve different data.
- No serialization — works only within a single process.
- No cache invalidation — the only way to clear it is to restart the application.
- Thread-safety issues in concurrent environments.

---

## The Right Way

Use a dedicated caching layer with explicit patterns and invalidation.

### Caching Layers

Modern applications have multiple caching layers, each serving a different purpose:

```
Client Cache (Browser/CDN)
        |
  CDN Edge Cache (CloudFront, Cloudflare)
        |
  Reverse Proxy Cache (Varnish, Nginx)
        |
  Application Cache (Redis, Memcached)
        |
  Database Query Cache (MySQL query cache, PostgreSQL shared buffers)
        |
  OS Page Cache (filesystem buffer cache)
```

Each layer reduces load on the layer below it. The key is deciding what to cache at each level.

### Cache Patterns

**Cache-Aside (Lazy Loading)**

The application manages the cache explicitly. On read, check the cache first; on miss, fetch from the database and populate the cache.

```python
import redis
import json

r = redis.Redis(host='cache.internal', port=6379, db=0)

def get_user(user_id):
    cache_key = f"user:{user_id}"

    # 1. Check cache
    cached = r.get(cache_key)
    if cached:
        return json.loads(cached)

    # 2. Cache miss — query database
    user = db.query("SELECT * FROM users WHERE id = ?", user_id)

    # 3. Populate cache with TTL
    r.setex(cache_key, 3600, json.dumps(user))  # expire in 1 hour

    return user

def update_user(user_id, data):
    # 1. Update database
    db.execute("UPDATE users SET ... WHERE id = ?", user_id, data)

    # 2. Invalidate cache
    r.delete(f"user:{user_id}")
```

**Pros:** Only caches data that is actually requested. Cache failures don't break the application.
**Cons:** Cache miss requires two round trips (cache lookup + database query). Stale data possible between write and cache invalidation.

**Write-Through**

Data is written to both the cache and the database simultaneously on every write.

```python
def update_user(user_id, data):
    # Write to database
    db.execute("UPDATE users SET ... WHERE id = ?", user_id, data)

    # Write to cache at the same time
    cache_key = f"user:{user_id}"
    r.setex(cache_key, 3600, json.dumps(data))
```

**Pros:** Cache is always fresh. Reads are always a cache hit after the first write.
**Cons:** Write latency increases (two writes per operation). Cache may store data that is never read.

**Write-Behind (Write-Back)**

Data is written to the cache immediately, and the cache asynchronously flushes to the database.

```python
# Simplified write-behind with a background worker
from collections import deque
import threading

write_queue = deque()

def update_user(user_id, data):
    # Write to cache immediately
    cache_key = f"user:{user_id}"
    r.setex(cache_key, 3600, json.dumps(data))

    # Queue database write for async processing
    write_queue.append(("UPDATE users SET ... WHERE id = ?", user_id, data))

def flush_worker():
    """Background thread that drains the write queue to the database."""
    while True:
        if write_queue:
            query, user_id, data = write_queue.popleft()
            db.execute(query, user_id, data)
        else:
            time.sleep(0.1)

threading.Thread(target=flush_worker, daemon=True).start()
```

**Pros:** Lowest write latency. Can batch database writes for efficiency.
**Cons:** Risk of data loss if the cache node crashes before flushing. Complex to implement correctly.

### Redis vs Memcached

| Feature | Redis | Memcached |
|---------|-------|-----------|
| Data structures | Strings, hashes, lists, sets, sorted sets, streams, bitmaps, HyperLogLog | Strings only |
| Persistence | RDB snapshots + AOF log | None |
| Replication | Master-replica with automatic failover (Sentinel) | None built-in |
| Clustering | Native cluster mode with hash slot sharding | Client-side sharding |
| Pub/Sub | Built-in | No |
| Lua scripting | Yes | No |
| Max value size | 512 MB | 1 MB (default) |
| Memory efficiency | Higher overhead per key | More memory-efficient for simple key-value |
| Threading | Single-threaded event loop (I/O threads in Redis 6+) | Multi-threaded |
| Use case | Complex caching, leaderboards, queues, rate limiting, sessions | Simple key-value caching, session storage |

**When to use Redis:**
- You need data structures beyond simple key-value (sorted sets for leaderboards, lists for queues).
- You need persistence or replication.
- You need pub/sub for event-driven architectures.
- You need Lua scripting for atomic operations.

**When to use Memcached:**
- You only need simple key-value caching.
- You want maximum memory efficiency for large volumes of simple cached objects.
- You need multi-threaded performance on a single node.
- Your workload is purely ephemeral (no persistence needed).

### Cache Invalidation Strategies

**TTL-Based (Time-To-Live)**

Every cache entry has an expiration time. After TTL expires, the entry is evicted.

```bash
# Redis TTL examples
SET session:abc123 '{"user_id": 42}' EX 1800    # expires in 30 minutes
SET config:feature_flags '{"dark_mode": true}' EX 300  # expires in 5 minutes

# Check remaining TTL
TTL session:abc123
```

**Pros:** Simple. Prevents indefinite staleness.
**Cons:** Data is stale until TTL expires. Choosing the right TTL is hard — too short and you lose caching benefits, too long and you serve stale data.

**Event-Based Invalidation**

When data changes, explicitly invalidate or update the cache.

```python
# On user update, publish invalidation event
def update_user(user_id, data):
    db.execute("UPDATE users SET ... WHERE id = ?", user_id, data)

    # Option A: Direct invalidation
    r.delete(f"user:{user_id}")

    # Option B: Publish invalidation event for all instances
    r.publish("cache:invalidate", json.dumps({"key": f"user:{user_id}"}))
```

**Pros:** Cache is invalidated immediately on data change.
**Cons:** Requires infrastructure for invalidation events. Race conditions possible between invalidation and concurrent reads.

**Version-Based Invalidation**

Include a version number in the cache key. When data changes, the version increments, producing a new cache key.

```python
def get_user(user_id):
    version = r.get(f"user:{user_id}:version") or 0
    cache_key = f"user:{user_id}:v{version}"

    cached = r.get(cache_key)
    if cached:
        return json.loads(cached)

    user = db.query("SELECT * FROM users WHERE id = ?", user_id)
    r.setex(cache_key, 3600, json.dumps(user))
    return user

def update_user(user_id, data):
    db.execute("UPDATE users SET ... WHERE id = ?", user_id, data)
    r.incr(f"user:{user_id}:version")  # old cache key is orphaned, will expire via TTL
```

**Pros:** No race conditions — old and new versions coexist until old TTL expires.
**Cons:** Old entries consume memory until they expire naturally.

### Cache Stampede (Thundering Herd)

When a popular cache entry expires, hundreds of concurrent requests all miss the cache simultaneously and all hit the database at once.

```
T=0:    Cache key "user:1" expires
T=0.01: 500 requests arrive, all miss cache
T=0.02: 500 database queries execute simultaneously
T=0.03: Database CPU spikes, latency increases, potential crash
```

**Solutions:**

**Lock-Based (Mutex)**

Only one request rebuilds the cache; others wait.

```python
import time

def get_user_with_lock(user_id):
    cache_key = f"user:{user_id}"
    lock_key = f"lock:{cache_key}"

    cached = r.get(cache_key)
    if cached:
        return json.loads(cached)

    # Try to acquire lock
    if r.set(lock_key, "1", nx=True, ex=10):  # lock expires in 10s
        try:
            user = db.query("SELECT * FROM users WHERE id = ?", user_id)
            r.setex(cache_key, 3600, json.dumps(user))
            return user
        finally:
            r.delete(lock_key)
    else:
        # Another process is rebuilding — wait and retry
        time.sleep(0.1)
        return get_user_with_lock(user_id)
```

**Probabilistic Early Expiration (XFetch)**

Each request has a probability of refreshing the cache before it actually expires.

```python
import random
import time

def get_user_early_refresh(user_id):
    cache_key = f"user:{user_id}"

    cached_data = r.get(cache_key)
    ttl = r.ttl(cache_key)

    if cached_data:
        data = json.loads(cached_data)
        # Probabilistic early refresh
        beta = 1.0
        expiry = data.get('_cache_expiry', 0)
        delta = time.time() - expiry + ttl
        if delta > 0 and random.random() < 1 - (1.0 / (delta * beta)):
            pass  # serve from cache this time
        else:
            user = db.query("SELECT * FROM users WHERE id = ?", user_id)
            user['_cache_expiry'] = time.time()
            r.setex(cache_key, 3600, json.dumps(user))
            return user
        return data

    user = db.query("SELECT * FROM users WHERE id = ?", user_id)
    user['_cache_expiry'] = time.time()
    r.setex(cache_key, 3600, json.dumps(user))
    return user
```

### TTL Strategy

Choosing the right TTL depends on data volatility and consistency requirements:

| Data Type | Volatility | Recommended TTL | Rationale |
|-----------|-----------|----------------|-----------|
| User session | Medium | 30 min - 24 hr | Balance security and UX |
| Product catalog | Low | 1 - 6 hr | Changes infrequently |
| Real-time stock prices | Very high | 5 - 30 sec | Must be nearly fresh |
| API rate limit counters | High | 1 min - 1 hr | Window-based counting |
| Feature flags | Medium | 1 - 5 min | Quick propagation needed |
| Static config | Very low | 24 hr | Almost never changes |

### Distributed Caching Architecture

In production, you don't run a single Redis instance. You run a cluster.

```
                    +-----------+
                    |  Client   |
                    +-----+-----+
                          |
              +-----------+-----------+
              |                       |
        +-----+-----+          +-----+-----+
        | App Node 1 |          | App Node 2 |
        +-----+-----+          +-----+-----+
              |                       |
              +-----------+-----------+
                          |
                  +-------+-------+
                  | Redis Cluster |
                  +-------+-------+
                  |       |       |
            +-----+  +----+--+  +-+------+
            |Shard|  |Shard  |  |Shard   |
            |  1  |  |  2    |  |  3     |
            |M + R|  |M + R  |  |M + R   |
            +-----+  +-------+  +--------+
```

**Redis Cluster:**
- Data is partitioned across shards using hash slots (16384 slots).
- Each shard has a master and one or more replicas.
- Automatic failover when a master fails.
- Client library handles routing to the correct shard.

**Consistent Hashing:**
When you need client-side sharding (or use Memcached), consistent hashing minimizes key redistribution when nodes are added or removed.

```python
import hashlib
import bisect

class ConsistentHash:
    def __init__(self, nodes, replicas=150):
        self.replicas = replicas
        self.ring = []
        self.node_map = {}

        for node in nodes:
            self.add_node(node)

    def add_node(self, node):
        for i in range(self.replicas):
            key = self._hash(f"{node}:{i}")
            self.ring.append(key)
            self.node_map[key] = node
        self.ring.sort()

    def remove_node(self, node):
        for i in range(self.replicas):
            key = self._hash(f"{node}:{i}")
            self.ring.remove(key)
            del self.node_map[key]

    def get_node(self, key):
        if not self.ring:
            return None
        h = self._hash(key)
        idx = bisect.bisect_right(self.ring, h) % len(self.ring)
        return self.node_map[self.ring[idx]]

    def _hash(self, key):
        return int(hashlib.md5(key.encode()).hexdigest(), 16)
```

---

## The Production Way

### CDN Caching

For static assets and cacheable API responses, use a CDN to push content to edge locations worldwide.

```nginx
# Nginx configuration for CDN-friendly cache headers
location /static/ {
    # Cache static assets at the CDN for 1 year
    add_header Cache-Control "public, max-age=31536000, immutable";
    add_header Vary "Accept-Encoding";
    expires 1y;
}

location /api/products/ {
    # Cache API responses at the CDN for 5 minutes
    add_header Cache-Control "public, max-age=300, s-maxage=600";
    add_header Vary "Authorization, Accept-Encoding";
    expires 5m;
}

location /api/user/ {
    # Never cache user-specific responses
    add_header Cache-Control "private, no-store";
}
```

**CloudFront / Cloudflare configuration:**

```yaml
# CloudFront cache behavior (Terraform)
resource "aws_cloudfront_distribution" "api" {
  default_cache_behavior {
    target_origin_id       = "api-origin"
    viewer_protocol_policy = "redirect-to-https"
    ttl {
      default_ttl = 300
      max_ttl     = 3600
      min_ttl     = 0
    }
    forwarded_values {
      query_string = true
      headers      = ["Authorization", "Origin"]
      cookies {
        forward = "none"
      }
    }
  }

  ordered_cache_behavior {
    path_pattern           = "/static/*"
    target_origin_id       = "s3-origin"
    viewer_protocol_policy = "redirect-to-https"
    ttl {
      default_ttl = 86400
      max_ttl     = 31536000
      min_ttl     = 86400
    }
    forwarded_values {
      query_string = false
      cookies {
        forward = "none"
      }
    }
  }
}
```

### Multi-Layer Cache Strategy

Production systems combine multiple patterns for maximum efficiency:

```python
import redis
import json
from functools import lru_cache

r = redis.Redis(host='redis-cluster.internal', port=6379, decode_responses=True)

class MultiLayerCache:
    def __init__(self):
        self.local_cache = {}  # L1: in-process cache (fastest, smallest)
        self.redis = r         # L2: distributed cache (fast, shared)

    def get(self, key, loader, local_ttl=60, redis_ttl=3600):
        # L1: Check local in-process cache
        if key in self.local_cache:
            entry = self.local_cache[key]
            if time.time() < entry['expires']:
                return entry['data']

        # L2: Check Redis
        cached = self.redis.get(key)
        if cached:
            data = json.loads(cached)
            # Backfill L1
            self.local_cache[key] = {
                'data': data,
                'expires': time.time() + local_ttl
            }
            return data

        # L3: Load from origin (database, API, etc.)
        data = loader()

        # Write to both caches
        self.redis.setex(key, redis_ttl, json.dumps(data))
        self.local_cache[key] = {
            'data': data,
            'expires': time.time() + local_ttl
        }

        return data

    def invalidate(self, key):
        self.local_cache.pop(key, None)
        self.redis.delete(key)
```

### Cache Warming

Pre-populate the cache before traffic arrives:

```python
def warm_cache():
    """Run on application startup or after deployment."""
    popular_products = db.query(
        "SELECT * FROM products ORDER BY view_count DESC LIMIT 1000"
    )
    for product in popular_products:
        r.setex(
            f"product:{product['id']}",
            7200,
            json.dumps(product)
        )
    print(f"Warmed cache with {len(popular_products)} products")
```

### Monitoring Cache Effectiveness

```python
class MonitoredCache:
    def __init__(self, redis_client, prefix="app"):
        self.redis = redis_client
        self.prefix = prefix

    def get(self, key):
        start = time.time()
        result = self.redis.get(key)
        elapsed = time.time() - start

        if result:
            self.redis.incr(f"{self.prefix}:cache:hits")
            self.redis.incr(f"{self.prefix}:cache:hit_latency_ms", int(elapsed * 1000))
        else:
            self.redis.incr(f"{self.prefix}:cache:misses")

        return result

    def get_stats(self):
        hits = int(self.redis.get(f"{self.prefix}:cache:hits") or 0)
        misses = int(self.redis.get(f"{self.prefix}:cache:misses") or 0)
        total = hits + misses
        hit_rate = (hits / total * 100) if total > 0 else 0

        return {
            "hits": hits,
            "misses": misses,
            "hit_rate_percent": round(hit_rate, 2)
        }
```

Target hit rates:
- Session cache: >95%
- API response cache: >80%
- Database query cache: >90%
- CDN cache: >95%

---

## Hands-On Lab

### Exercise 1: Redis Cache-Aside Pattern

```bash
# Start Redis
docker run -d --name redis-lab -p 6379:6379 redis:7-alpine

# Connect and experiment
docker exec -it redis-lab redis-cli

# Basic operations
SET user:1 '{"name":"Alice","email":"alice@example.com"}' EX 60
GET user:1
TTL user:1
GET user:999  # returns nil (cache miss)

# Hash operations for structured data
HSET user:1 name "Alice" email "alice@example.com" role "admin"
HGET user:1 name
HGETALL user:1
EXPIRE user:1 3600
```

### Exercise 2: Cache Stampede Simulation

```python
# stampede_demo.py
import redis
import time
import threading

r = redis.Redis(host='localhost', port=6379, decode_responses=True)

def slow_loader(key):
    """Simulates a slow database query."""
    time.sleep(2)
    return f"data_for_{key}"

def get_without_protection(key):
    """Vulnerable to stampede."""
    cached = r.get(key)
    if cached:
        return cached
    data = slow_loader(key)
    r.setex(key, 30, data)
    return data

def get_with_lock(key):
    """Protected by distributed lock."""
    cached = r.get(key)
    if cached:
        return cached

    lock_key = f"lock:{key}"
    if r.set(lock_key, "1", nx=True, ex=10):
        try:
            data = slow_loader(key)
            r.setex(key, 30, data)
            return data
        finally:
            r.delete(lock_key)
    else:
        time.sleep(0.1)
        return get_with_lock(key)

# Test: spawn 50 threads, measure total time
def benchmark(get_func, label):
    r.delete("test:key")
    results = []
    start = time.time()

    def worker():
        result = get_func("test:key")
        results.append(result)

    threads = [threading.Thread(target=worker) for _ in range(50)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()

    elapsed = time.time() - start
    print(f"{label}: {elapsed:.2f}s, unique queries: {1 if len(set(results)) == 1 else len(results)}")

benchmark(get_without_protection, "Without lock")
benchmark(get_with_lock, "With lock")
```

### Exercise 3: CDN Cache Headers

```bash
# Test cache headers with curl
curl -I https://httpbin.org/cache/60

# Check cache-control headers
curl -s -D - -o /dev/null https://httpbin.org/cache/60 | grep -i cache-control

# Simulate CDN behavior with Nginx
docker run -d --name nginx-cache -p 8080:80 \
  -v $(pwd)/nginx-cache.conf:/etc/nginx/conf.d/default.conf \
  nginx:alpine
```

### Exercise 4: Redis Cluster Setup

```bash
# Create a 6-node Redis cluster (3 masters + 3 replicas)
for port in 7001 7002 7003 7004 7005 7006; do
  mkdir -p /tmp/redis-cluster/$port
  cat > /tmp/redis-cluster/$port/redis.conf << EOF
port $port
cluster-enabled yes
cluster-config-file nodes-$port.conf
cluster-node-timeout 5000
appendonly yes
dir /tmp/redis-cluster/$port
EOF
  redis-server /tmp/redis-cluster/$port/redis.conf --daemonize yes
done

# Create the cluster
redis-cli --cluster create \
  127.0.0.1:7001 127.0.0.1:7002 127.0.0.1:7003 \
  127.0.0.1:7004 127.0.0.1:7005 127.0.0.1:7006 \
  --cluster-replicas 1

# Test cluster
redis-cli -c -p 7001 SET test:hello "world"
redis-cli -c -p 7002 GET test:hello  # routed automatically
```

---

## Limitation

Caching strategies assume the underlying OS and kernel are optimally configured. Cache performance depends on network stack tuning (TCP buffer sizes, connection pooling), memory management (huge pages for Redis, swappiness settings), and file descriptor limits. On a misconfigured OS, even the best caching architecture will underperform. You need OS-level tuning to unlock the full potential of your caching layer.

---

## Next Topic

[71 - Performance Tuning](../71-performance-tuning/README.md) — Tune the OS kernel, network stack, and system parameters to extract maximum performance from your infrastructure.
