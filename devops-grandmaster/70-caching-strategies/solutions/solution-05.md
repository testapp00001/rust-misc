# Solution 05: Production Caching System

## Part A: Complete Architecture Diagram

```
                    +------------------+
                    |     Client       |
                    +--------+---------+
                             |
                    +--------v---------+
                    |   CDN (L3)       |  CloudFront / Cloudflare
                    |  Static assets   |  TTL: 1 hour
                    |  API responses   |  TTL: 5 minutes
                    +--------+---------+
                             |
                    +--------v---------+
                    |  Load Balancer   |
                    +--------+---------+
                             |
              +--------------+--------------+
              |                             |
     +--------v---------+        +---------v--------+
     |  App Instance 1  |        |  App Instance 2  |
     |  L1 Cache (dict) |        |  L1 Cache (dict) |
     |  TTL: 30s        |        |  TTL: 30s        |
     +--------+---------+        +---------+--------+
              |                             |
              +--------------+--------------+
                             |
                    +--------v---------+
                    |  Redis Cluster   |  L2 Cache
                    |  3 shards        |  TTL: 5 minutes
                    |  3 replicas      |  Shared across instances
                    +--------+---------+
                             |
                    +--------v---------+
                    |   PostgreSQL     |  Origin
                    |   Primary        |
                    |   + 2 Replicas   |
                    +------------------+

Invalidation Flow:
  DB change -> App invalidates L1 -> App invalidates L2 -> CDN purge
  Redis pub/sub notifies all instances to invalidate L1

Monitoring:
  Each layer reports hit/miss/latency to Prometheus
  Grafana dashboard shows all layers
```

---

## Part B: Cache Manager

```python
import time
import json
import redis
import hashlib
from enum import Enum
from typing import Callable, Optional

class CacheStrategy(Enum):
    CACHE_ASIDE = "cache_aside"
    WRITE_THROUGH = "write_through"

class ProductionCacheManager:
    """Production-grade multi-strategy cache manager."""

    def __init__(self, redis_cluster_hosts):
        self.redis = redis.RedisCluster(
            startup_nodes=redis_cluster_hosts,
            decode_responses=True
        )
        self.l1_cache = {}  # In-process cache
        self.strategies = {}  # namespace -> strategy
        self.l1_ttl = 30
        self.l2_ttl = 300

        # Metrics counters
        self.metrics_prefix = "cache:metrics"

    def register_namespace(self, namespace: str, strategy: CacheStrategy,
                           l1_ttl: int = 30, l2_ttl: int = 300):
        """Register a cache namespace with its strategy."""
        self.strategies[namespace] = {
            'strategy': strategy,
            'l1_ttl': l1_ttl,
            'l2_ttl': l2_ttl,
        }

    def get(self, namespace: str, key: str, loader: Callable = None):
        """Get from cache with namespace-aware strategy."""
        full_key = f"{namespace}:{key}"
        config = self.strategies.get(namespace, {
            'l1_ttl': self.l1_ttl, 'l2_ttl': self.l2_ttl
        })

        start = time.time()

        # L1 check
        if full_key in self.l1_cache:
            entry = self.l1_cache[full_key]
            if time.time() < entry['expires']:
                self._record_metric(namespace, 'hit', 'l1', time.time() - start)
                return entry['data']

        # L2 check
        l2_data = self.redis.get(full_key)
        if l2_data:
            data = json.loads(l2_data)
            self.l1_cache[full_key] = {
                'data': data,
                'expires': time.time() + config['l1_ttl']
            }
            self._record_metric(namespace, 'hit', 'l2', time.time() - start)
            return data

        # Origin
        if loader is None:
            self._record_metric(namespace, 'miss', 'origin', time.time() - start)
            return None

        data = loader()
        elapsed = time.time() - start

        # Backfill L2 and L1
        self.redis.setex(full_key, config['l2_ttl'], json.dumps(data))
        self.l1_cache[full_key] = {
            'data': data,
            'expires': time.time() + config['l1_ttl']
        }
        self._record_metric(namespace, 'miss', 'origin', elapsed)
        return data

    def set(self, namespace: str, key: str, data):
        """Set in cache using the namespace strategy."""
        full_key = f"{namespace}:{key}"
        config = self.strategies.get(namespace, {
            'l1_ttl': self.l1_ttl, 'l2_ttl': self.l2_ttl
        })
        strategy = config.get('strategy', CacheStrategy.CACHE_ASIDE)

        if strategy == CacheStrategy.WRITE_THROUGH:
            # Write to both L1 and L2 immediately
            self.l1_cache[full_key] = {
                'data': data,
                'expires': time.time() + config['l1_ttl']
            }
            self.redis.setex(full_key, config['l2_ttl'], json.dumps(data))
        # For cache-aside, set is handled by get() on miss

    def invalidate(self, namespace: str, key: str):
        """Invalidate a specific key from all layers."""
        full_key = f"{namespace}:{key}"
        self.l1_cache.pop(full_key, None)
        self.redis.delete(full_key)
        # Publish invalidation event for other instances
        self.redis.publish('cache:invalidate', json.dumps({
            'key': full_key, 'action': 'delete'
        }))

    def invalidate_pattern(self, namespace: str, pattern: str):
        """Invalidate all keys matching a pattern."""
        full_pattern = f"{namespace}:{pattern}"
        # L1
        keys_to_remove = [
            k for k in self.l1_cache if k.startswith(full_pattern.replace('*', ''))
        ]
        for key in keys_to_remove:
            del self.l1_cache[key]
        # L2
        cursor = 0
        while True:
            cursor, keys = self.redis.scan(cursor, match=full_pattern, count=100)
            if keys:
                self.redis.delete(*keys)
            if cursor == 0:
                break

    def warm(self, namespace: str, entries: list, key_func, loader_func):
        """Warm cache with pre-loaded entries."""
        config = self.strategies.get(namespace, {
            'l1_ttl': self.l1_ttl, 'l2_ttl': self.l2_ttl
        })
        pipe = self.redis.pipeline()
        warmed = 0
        for entry in entries:
            try:
                key = key_func(entry)
                full_key = f"{namespace}:{key}"
                data = loader_func(entry) if loader_func else entry
                pipe.setex(full_key, config['l2_ttl'], json.dumps(data))
                warmed += 1
            except Exception as e:
                print(f"Warm failed for {key}: {e}")
        pipe.execute()
        print(f"Warmed {warmed} entries for namespace '{namespace}'")

    def _record_metric(self, namespace: str, result: str, layer: str, latency: float):
        """Record cache metrics."""
        self.redis.incr(f"{self.metrics_prefix}:{namespace}:{result}:{layer}")
        self.redis.lpush(
            f"{self.metrics_prefix}:{namespace}:latency",
            f"{layer}:{latency * 1000:.2f}"
        )
        self.redis.ltrim(
            f"{self.metrics_prefix}:{namespace}:latency", 0, 999
        )

    def get_stats(self, namespace: str) -> dict:
        """Get cache statistics for a namespace."""
        hits_l1 = int(self.redis.get(
            f"{self.metrics_prefix}:{namespace}:hit:l1") or 0)
        hits_l2 = int(self.redis.get(
            f"{self.metrics_prefix}:{namespace}:hit:l2") or 0)
        misses = int(self.redis.get(
            f"{self.metrics_prefix}:{namespace}:miss:origin") or 0)
        total = hits_l1 + hits_l2 + misses
        return {
            'l1_hits': hits_l1,
            'l2_hits': hits_l2,
            'origin_misses': misses,
            'hit_rate': round((hits_l1 + hits_l2) / total * 100, 2) if total else 0,
            'l1_hit_rate': round(hits_l1 / total * 100, 2) if total else 0,
            'l2_hit_rate': round(hits_l2 / total * 100, 2) if total else 0,
        }
```

---

## Part C: Invalidation Pipeline

```python
import threading
import json

class CacheInvalidationPipeline:
    """Handle cache invalidation across all layers."""

    def __init__(self, cache_manager, cdn_client=None):
        self.cache = cache_manager
        self.cdn = cdn_client
        self.redis = cache_manager.redis
        self.audit_log = []

    def start_listener(self):
        """Start listening for invalidation events via Redis pub/sub."""
        pubsub = self.redis.pubsub()
        pubsub.subscribe('cache:invalidate')

        def listener():
            for message in pubsub.listen():
                if message['type'] == 'message':
                    event = json.loads(message['data'])
                    self._handle_event(event)

        thread = threading.Thread(target=listener, daemon=True)
        thread.start()
        print("Invalidation pipeline listener started")

    def invalidate(self, key: str, cdn_path: str = None):
        """Full invalidation: L1 + L2 + CDN."""
        # Invalidate L1 and L2
        self.cache.l1_cache.pop(key, None)
        self.cache.redis.delete(key)

        # Publish event for other instances
        self.redis.publish('cache:invalidate', json.dumps({
            'key': key, 'action': 'delete', 'cdn_path': cdn_path
        }))

        # CDN purge
        if cdn_path and self.cdn:
            self._purge_cdn(cdn_path)

        # Audit log
        self._log_invalidation(key, cdn_path)

    def _handle_event(self, event: dict):
        """Handle an invalidation event from pub/sub."""
        key = event.get('key')
        action = event.get('action')
        cdn_path = event.get('cdn_path')

        if action == 'delete':
            self.cache.l1_cache.pop(key, None)
            if cdn_path and self.cdn:
                self._purge_cdn(cdn_path)

    def _purge_cdn(self, path: str):
        """Send CDN purge request."""
        try:
            # CloudFront invalidation example
            self.cdn.create_invalidation(
                DistributionId='E1234567890',
                InvalidationBatch={
                    'Paths': {'Quantity': 1, 'Items': [path]},
                    'CallerReference': str(time.time())
                }
            )
        except Exception as e:
            print(f"CDN purge failed for {path}: {e}")

    def _log_invalidation(self, key: str, cdn_path: str = None):
        """Log invalidation for audit."""
        entry = {
            'timestamp': time.time(),
            'key': key,
            'cdn_path': cdn_path,
        }
        self.audit_log.append(entry)
        self.redis.lpush('cache:invalidation:audit', json.dumps(entry))
        self.redis.ltrim('cache:invalidation:audit', 0, 9999)
```

---

## Part D: Cache Warming System

```python
class CacheWarmingSystem:
    """Pre-populate cache on startup."""

    def __init__(self, cache_manager, db_connection):
        self.cache = cache_manager
        self.db = db_connection

    def warm_all(self):
        """Run all warming tasks."""
        start = time.time()
        print("=== Cache Warming Started ===")

        self.warm_products(limit=1000)
        self.warm_active_sessions(limit=5000)

        elapsed = time.time() - start
        print(f"=== Cache Warming Complete ({elapsed:.1f}s) ===")

    def warm_products(self, limit=1000):
        """Pre-populate top products by view count."""
        print(f"Warming top {limit} products...")
        products = self.db.query(
            "SELECT * FROM products ORDER BY view_count DESC LIMIT %s",
            limit
        )
        pipe = self.cache.redis.pipeline()
        warmed = 0
        for product in products:
            try:
                key = f"products:{product['id']}"
                pipe.setex(key, 3600, json.dumps(product))
                warmed += 1
            except Exception as e:
                print(f"  Failed to warm product {product['id']}: {e}")
        pipe.execute()
        print(f"  Warmed {warmed} products")

    def warm_active_sessions(self, limit=5000):
        """Pre-populate sessions active in last 30 minutes."""
        print(f"Warming active sessions (limit: {limit})...")
        sessions = self.db.query(
            "SELECT * FROM sessions WHERE last_active > NOW() - INTERVAL '30 minutes' LIMIT %s",
            limit
        )
        pipe = self.cache.redis.pipeline()
        warmed = 0
        for session in sessions:
            try:
                key = f"sessions:{session['id']}"
                pipe.setex(key, 1800, json.dumps(session))
                warmed += 1
            except Exception as e:
                print(f"  Failed to warm session {session['id']}: {e}")
        pipe.execute()
        print(f"  Warmed {warmed} sessions")
```

---

## Part E: Monitoring Dashboard Specification

```yaml
# Grafana Dashboard: Cache Effectiveness
dashboard:
  title: "Cache Effectiveness"
  panels:
    - title: "Cache Hit Rates by Layer"
      type: graph
      targets:
        - expr: 'sum(rate(cache_hits{layer="l1"}[5m])) / sum(rate(cache_requests{layer="l1"}[5m])) * 100'
          legend: "L1 Hit Rate"
        - expr: 'sum(rate(cache_hits{layer="l2"}[5m])) / sum(rate(cache_requests{layer="l2"}[5m])) * 100'
          legend: "L2 Hit Rate"
        - expr: 'sum(rate(cache_hits{layer="cdn"}[5m])) / sum(rate(cache_requests{layer="cdn"}[5m])) * 100'
          legend: "CDN Hit Rate"
      unit: percent

    - title: "Cache Latency by Layer (P95)"
      type: graph
      targets:
        - expr: 'histogram_quantile(0.95, sum(rate(cache_latency_bucket{layer="l1"}[5m])) by (le))'
          legend: "L1 P95"
        - expr: 'histogram_quantile(0.95, sum(rate(cache_latency_bucket{layer="l2"}[5m])) by (le))'
          legend: "L2 P95"
      unit: milliseconds

    - title: "Redis Memory Usage"
      type: graph
      targets:
        - expr: 'redis_memory_used_bytes'
          legend: "Used"
        - expr: 'redis_memory_max_bytes'
          legend: "Max"
      unit: bytes

    - title: "Invalidation Events per Minute"
      type: graph
      targets:
        - expr: 'sum(rate(cache_invalidation_total[5m])) * 60'
          legend: "Invalidations/min"

    - title: "Stampede Prevention (Lock Acquisitions)"
      type: graph
      targets:
        - expr: 'sum(rate(cache_lock_acquired_total[5m])) * 60'
          legend: "Locks/min"

    - title: "Top 10 Most-Missed Cache Keys"
      type: table
      targets:
        - expr: 'topk(10, sum(rate(cache_misses_total[5m])) by (key))'
```

---

## Common Mistakes
1. **Not warming the cache on startup**: Cold cache causes a stampede on the database. Always warm critical data.
2. **Not invalidating across all instances**: In a multi-instance deployment, invalidation must reach all instances. Use Redis pub/sub for distributed invalidation.
3. **CDN cache without purge capability**: If you cannot purge the CDN, you must rely on TTL for freshness. This limits how aggressively you can cache.
4. **Monitoring only hit rate**: Also monitor latency (cache adds overhead), memory usage (cache can fill up), and invalidation frequency (too many invalidations reduce effectiveness).

## Relevant README Sections
- [Distributed Caching Architecture](../README.md#distributed-caching-architecture)
- [Multi-Layer Cache Strategy](../README.md#multi-layer-cache-strategy)
- [Cache Warming](../README.md#cache-warming)
- [Monitoring Cache Effectiveness](../README.md#monitoring-cache-effectiveness)
