# Solution 04: Multi-Layer Cache Architecture

## Part A: Cache Layer Design

```
Layer    | Data Type         | TTL      | Invalidation     | Hit Rate
---------|-------------------|----------|------------------|----------
L1       | Hot product data  | 30 sec   | TTL + explicit   | 60-70%
L2       | All cacheable data| 5 min    | Event-based      | 80-90%
L3/CDN   | Static assets     | 1 hour   | Purge on deploy  | 95%+

Request Flow:
  Client -> CDN (L3) -> Application -> L1 -> L2 -> Origin
                |            |
           Cache HIT     Cache HIT
           (<10ms)       (<1ms)
```

**Why this design**: L1 catches the most frequent requests with the lowest latency. L2 catches requests that miss L1 but are still cacheable. L3 (CDN) offloads static content entirely. Each layer reduces load on the layer below it.

---

## Part B: Multi-Layer Cache Implementation

```python
import time
import json
import redis

class MultiLayerCache:
    """Three-layer cache: L1 (in-process), L2 (Redis), L3 (CDN, handled externally)."""

    def __init__(self, redis_host='localhost', redis_port=6379):
        # L1: In-process cache (dict with TTL)
        self._l1_cache = {}
        self._l1_ttl = 30  # seconds

        # L2: Redis (shared across instances)
        self._l2 = redis.Redis(
            host=redis_host, port=redis_port, decode_responses=True
        )
        self._l2_ttl = 300  # seconds (5 minutes)

    def get(self, key: str, loader=None):
        """
        Get from cache with L1 -> L2 -> origin fallback.
        Backfills both L1 and L2 on origin access.
        """
        # L1: Check in-process cache
        if key in self._l1_cache:
            entry = self._l1_cache[key]
            if time.time() < entry['expires']:
                return entry['data']
            else:
                del self._l1_cache[key]  # Expired

        # L2: Check Redis
        l2_data = self._l2.get(key)
        if l2_data:
            data = json.loads(l2_data)
            # Backfill L1
            self._l1_cache[key] = {
                'data': data,
                'expires': time.time() + self._l1_ttl
            }
            return data

        # Origin: Load from source (database, API, etc.)
        if loader is None:
            return None

        data = loader()

        # Backfill both L1 and L2
        self._l2.setex(key, self._l2_ttl, json.dumps(data))
        self._l1_cache[key] = {
            'data': data,
            'expires': time.time() + self._l1_ttl
        }

        return data

    def set(self, key: str, data, l1_ttl=None, l2_ttl=None):
        """Set in both L1 and L2."""
        l1_ttl = l1_ttl or self._l1_ttl
        l2_ttl = l2_ttl or self._l2_ttl

        self._l1_cache[key] = {
            'data': data,
            'expires': time.time() + l1_ttl
        }
        self._l2.setex(key, l2_ttl, json.dumps(data))

    def invalidate(self, key: str):
        """Remove from both L1 and L2."""
        self._l1_cache.pop(key, None)
        self._l2.delete(key)

    def invalidate_pattern(self, pattern: str):
        """Invalidate all keys matching a pattern."""
        # L1: Scan and remove matching keys
        keys_to_remove = [
            k for k in self._l1_cache
            if k.startswith(pattern.replace('*', ''))
        ]
        for key in keys_to_remove:
            del self._l1_cache[key]

        # L2: Use SCAN to find and delete matching keys
        cursor = 0
        while True:
            cursor, keys = self._l2.scan(cursor, match=pattern, count=100)
            if keys:
                self._l2.delete(*keys)
            if cursor == 0:
                break

    def get_stats(self) -> dict:
        """Get cache statistics."""
        l1_size = len(self._l1_cache)
        l2_info = self._l2.info('memory')
        return {
            'l1_entries': l1_size,
            'l2_memory_used_mb': l2_info.get('used_memory', 0) / (1024 * 1024),
        }
```

**Why this works**: The `get()` method checks L1 first (fastest, no network), then L2 (fast, shared), then origin. On origin access, both L1 and L2 are populated (backfill). L1 has a shorter TTL (30s) than L2 (300s) because L1 is per-instance and should refresh more often to maintain consistency across instances.

---

## Part C: CDN Configuration (Nginx)

```nginx
server {
    listen 80;
    server_name api.example.com;

    # Product API: cache at CDN for 5 minutes
    location /api/products/ {
        add_header Cache-Control "public, max-age=300, s-maxage=600";
        add_header Vary "Authorization, Accept-Encoding";
        proxy_pass http://backend;
    }

    # Static assets: cache at CDN for 1 year (immutable)
    location /static/ {
        add_header Cache-Control "public, max-age=31536000, immutable";
        add_header Vary "Accept-Encoding";
        proxy_pass http://backend;
    }

    # User-specific API: never cache
    location /api/user/ {
        add_header Cache-Control "private, no-store";
        proxy_pass http://backend;
    }

    # Search API: cache for 1 minute
    location /api/search/ {
        add_header Cache-Control "public, max-age=60, s-maxage=120";
        add_header Vary "Authorization, Accept-Encoding, Accept-Language";
        proxy_pass http://backend;
    }
}
```

**Why these headers**:
- `public`: CDN can cache the response.
- `max-age=N`: Browser caches for N seconds.
- `s-maxage=N`: CDN caches for N seconds (overrides max-age for shared caches).
- `private, no-store`: Never cache user-specific responses.
- `immutable`: Browser will not revalidate for 1 year (use for versioned static assets).
- `Vary`: Cache different versions based on these headers.

---

## Part D: Monitoring

```python
class CacheMonitor:
    """Monitor cache effectiveness across all layers."""

    def __init__(self, redis_client):
        self.redis = redis_client

    def record_hit(self, layer: str):
        self.redis.incr(f"cache:{layer}:hits")

    def record_miss(self, layer: str):
        self.redis.incr(f"cache:{layer}:misses")

    def record_latency(self, layer: str, latency_ms: float):
        self.redis.lpush(f"cache:{layer}:latency", latency_ms)
        self.redis.ltrim(f"cache:{layer}:latency", 0, 999)  # Keep last 1000

    def get_hit_rate(self, layer: str) -> float:
        hits = int(self.redis.get(f"cache:{layer}:hits") or 0)
        misses = int(self.redis.get(f"cache:{layer}:misses") or 0)
        total = hits + misses
        return (hits / total * 100) if total > 0 else 0

    def get_avg_latency(self, layer: str) -> float:
        latencies = self.redis.lrange(f"cache:{layer}:latency", 0, -1)
        if not latencies:
            return 0
        return sum(float(l) for l in latencies) / len(latencies)

    def get_origin_offload(self) -> float:
        total = 0
        cache_hits = 0
        for layer in ['l1', 'l2', 'cdn']:
            hits = int(self.redis.get(f"cache:{layer}:hits") or 0)
            misses = int(self.redis.get(f"cache:{layer}:misses") or 0)
            cache_hits += hits
            total += hits + misses
        return (cache_hits / total * 100) if total > 0 else 0

    def get_report(self) -> dict:
        report = {}
        for layer in ['l1', 'l2', 'cdn']:
            report[layer] = {
                'hit_rate': round(self.get_hit_rate(layer), 2),
                'avg_latency_ms': round(self.get_avg_latency(layer), 2),
            }
        report['origin_offload'] = round(self.get_origin_offload(), 2)
        return report
```

---

## Common Mistakes
1. **Same TTL for L1 and L2**: L1 should have a shorter TTL than L2. If both have the same TTL, L1 does not provide additional freshness.
2. **Not invalidating all layers**: Invalidation must remove from both L1 and L2. If only L2 is invalidated, L1 serves stale data.
3. **Caching user-specific data at CDN**: User-specific responses must use `private, no-store`. If cached at CDN, users see each other's data.
4. **Not using Vary headers**: Without `Vary`, the CDN serves the same cached response to all users regardless of auth token or encoding.

## Relevant README Sections
- [Caching Layers](../README.md#caching-layers)
- [Multi-Layer Cache Strategy](../README.md#multi-layer-cache-strategy)
- [CDN Caching](../README.md#cdn-caching)
