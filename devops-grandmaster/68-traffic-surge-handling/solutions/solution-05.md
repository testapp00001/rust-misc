# Solution 05: Full Surge Architecture

## Part A: Layer Architecture Diagram

```
Layer 1: EDGE
+------------------------------------------------------------------+
|  CDN (CloudFront)  -->  WAF  -->  Rate Limiter (Nginx/API GW)   |
|  Cache HIT: <10ms    Blocks     100 req/s per IP                 |
|  Cache MISS: forward  DDoS      429 on exceed                    |
+------------------------------------------------------------------+
        |
        v
Layer 2: INGESTION
+------------------------------------------------------------------+
|  Ingestion Service (FastAPI)                                     |
|  - Validates input                                               |
|  - Checks rate limit (Redis)                                     |
|  - Checks queue depth (backpressure)                             |
|  - Enqueues to Redis list                                        |
|  - Returns order_id + "queued"                                   |
|  Lightweight: no DB writes, <1ms per request                     |
+------------------------------------------------------------------+
        |
        v
Layer 3: QUEUE
+------------------------------------------------------------------+
|  Redis List: "order_queue"                                       |
|  - Buffers up to 100,000 orders                                  |
|  - FIFO processing                                               |
|  - Backpressure signal at 80% capacity                           |
|  - Dead letter queue for poison messages                         |
+------------------------------------------------------------------+
        |
        v
Layer 4: PROCESSING
+------------------------------------------------------------------+
|  Worker Pool (auto-scaled 5-50 replicas)                         |
|  - Consumes from queue in batches of 10                          |
|  - Semaphore: max 50 concurrent per worker                       |
|  - Circuit breaker on Payment Service                            |
|  - Metrics: processed, errors, queue depth, rate                 |
+------------------------------------------------------------------+
        |
        v
Layer 5: DATA
+------------------------------------------------------------------+
|  PgBouncer (connection pooler)  -->  PostgreSQL Primary          |
|  10,000 client conns               Write-heavy                   |
|  -> 200 DB conns                                                |
|                                     PostgreSQL Replicas (2)      |
|  Redis Cluster                      Read-heavy queries           |
|  - Order status                                                 |
|  - Rate limit counters                                          |
|  - Cache                                                        |
+------------------------------------------------------------------+

Failure Modes:
  Layer 1: CDN absorbs cacheable traffic. Rate limiter rejects excess.
  Layer 2: Ingestion rejects with 429 (rate limit) or 503 (queue full).
  Layer 3: Queue buffers burst. Workers drain at sustainable rate.
  Layer 4: Circuit breaker protects against downstream failures.
  Layer 5: Connection pooling prevents DB connection exhaustion.
```

---

## Part B: Edge Rate Limiter (Nginx)

```nginx
http {
    # Rate limit zones
    limit_req_zone $binary_remote_addr zone=api:10m rate=100r/s;
    limit_req_zone $binary_remote_addr zone=login:10m rate=5r/s;
    limit_req_zone $server_name zone=global:10m rate=50000r/s;

    # Connection limits
    limit_conn_zone $binary_remote_addr zone=addr:10m;

    upstream web_backend {
        least_conn;
        server web-1:8080;
        server web-2:8080;
        server web-3:8080;
        server web-4:8080;
        server web-5:8080;
        keepalive 256;
    }

    server {
        listen 80;

        # Global rate limit
        limit_req zone=global burst=5000 nodelay;

        # API endpoints: 100 req/s per client, burst 200
        location /api/ {
            limit_req zone=api burst=200 nodelay;
            limit_conn addr 50;

            proxy_pass http://web_backend;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header Host $host;

            # Custom 429 response
            error_page 429 = @rate_limited;
        }

        # Login endpoint: 5 req/s per client (strict)
        location /api/auth/login {
            limit_req zone=login burst=10 nodelay;
            limit_conn addr 5;

            proxy_pass http://web_backend;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;

            error_page 429 = @rate_limited;
        }

        # Static assets: no rate limit (CDN handles these)
        location /static/ {
            proxy_pass http://web_backend;
            proxy_cache_valid 200 1h;
        }

        # Health check: no rate limit
        location /health {
            proxy_pass http://web_backend;
        }

        # Custom 429 response body
        location @rate_limited {
            default_type application/json;
            return 429 '{"error": "rate_limit_exceeded", "retry_after": 1}';
        }
    }
}
```

**Why this works**: Three rate limit zones handle different traffic patterns: global (50,000 RPS total), API (100 RPS per client), and login (5 RPS per client). `burst=200 nodelay` allows short bursts above the rate limit without delaying requests. The `@rate_limited` location returns a JSON error body instead of the default Nginx HTML page.

---

## Part C: Surge Handler

```python
import asyncio
import aioredis
import json
from datetime import datetime
from enum import Enum

class DegradationLevel(Enum):
    NORMAL = "NORMAL"
    ELEVATED = "ELEVATED"
    HIGH = "HIGH"
    CRITICAL = "CRITICAL"

class SurgeHandler:
    """Production-grade traffic surge handler."""

    def __init__(self, redis_url: str, max_queue: int = 100000):
        self.redis = aioredis.from_url(redis_url, decode_responses=True)
        self.max_queue = max_queue
        self.rejected_count = 0
        self.processed_count = 0
        self.queued_count = 0

    async def handle_request(self, request: dict) -> dict:
        """Handle incoming request with full surge protection."""

        # Step 1: Rate limiting (100 req/min per IP)
        client_ip = request.get("client_ip", "unknown")
        if not await self._check_rate_limit(client_ip):
            self.rejected_count += 1
            return {
                "status": 429,
                "body": {"error": "rate_limit_exceeded", "retry_after": 60}
            }

        # Step 2: Check degradation level
        degradation = await self._get_degradation_level()

        if degradation == DegradationLevel.CRITICAL:
            self.rejected_count += 1
            return {
                "status": 503,
                "body": {"error": "system_overloaded", "message": "Please try again later"}
            }

        # Step 3: High degradation — serve from cache
        if degradation == DegradationLevel.HIGH:
            cached = await self._get_cached_response(request.get("path", "/"))
            if cached:
                return {"status": 200, "body": cached, "source": "cache"}
            return {
                "status": 503,
                "body": {"error": "degraded_mode", "message": "Serving limited functionality"}
            }

        # Step 4: Try synchronous processing (fast path)
        try:
            result = await asyncio.wait_for(
                self._process_request(request),
                timeout=2.0
            )
            self.processed_count += 1
            return {"status": 200, "body": result, "source": "sync"}
        except asyncio.TimeoutError:
            pass  # Fall through to queue

        # Step 5: Queue fallback (async processing)
        queued = await self._enqueue_request(request)
        if queued:
            self.queued_count += 1
            return {
                "status": 202,
                "body": {
                    "order_id": request.get("id"),
                    "status": "queued",
                    "message": "Order received and queued for processing"
                },
                "source": "queue"
            }

        # Step 6: Queue full — reject
        self.rejected_count += 1
        return {
            "status": 503,
            "body": {"error": "queue_full", "message": "System at capacity"}
        }

    async def _check_rate_limit(self, client_ip: str) -> bool:
        """Sliding window rate limiter using Redis."""
        minute_key = datetime.now().strftime("%Y%m%d%H%M")
        key = f"ratelimit:{client_ip}:{minute_key}"
        count = await self.redis.incr(key)
        if count == 1:
            await self.redis.expire(key, 60)
        return count <= 100

    async def _get_degradation_level(self) -> DegradationLevel:
        """Determine system degradation level from metrics."""
        queue_size = await self.redis.llen("order_queue")
        queue_ratio = queue_size / self.max_queue

        if queue_ratio > 0.9:
            return DegradationLevel.CRITICAL
        elif queue_ratio > 0.5:
            return DegradationLevel.HIGH
        elif queue_ratio > 0.2:
            return DegradationLevel.ELEVATED
        return DegradationLevel.NORMAL

    async def _get_cached_response(self, path: str):
        """Try to serve from cache."""
        cached = await self.redis.get(f"cache:{path}")
        if cached:
            return json.loads(cached)
        return None

    async def _process_request(self, request: dict) -> dict:
        """Synchronous request processing."""
        await asyncio.sleep(0.05)  # Simulate processing
        return {"order_id": request.get("id"), "status": "processed"}

    async def _enqueue_request(self, request: dict) -> bool:
        """Enqueue request for async processing."""
        queue_size = await self.redis.llen("order_queue")
        if queue_size >= self.max_queue:
            return False
        await self.redis.rpush("order_queue", json.dumps(request))
        return True

    def get_metrics(self) -> dict:
        return {
            "processed": self.processed_count,
            "queued": self.queued_count,
            "rejected": self.rejected_count,
        }
```

---

## Part D: Worker with Metrics and Dead Letter Queue

```python
import asyncio
import aioredis
import json
import time

class SurgeWorker:
    def __init__(self, concurrency=50, max_retries=3):
        self.redis = None
        self.concurrency = concurrency
        self.max_retries = max_retries
        self.semaphore = asyncio.Semaphore(concurrency)
        self.processed = 0
        self.errors = 0
        self.dead_lettered = 0
        self.start_time = time.time()

    async def connect(self, redis_url: str):
        self.redis = aioredis.from_url(redis_url, decode_responses=True)

    async def process_message(self, message: dict):
        """Process a single message with retry tracking."""
        msg_id = message.get("id", "unknown")
        retry_count = message.get("_retry_count", 0)

        async with self.semaphore:
            try:
                await asyncio.sleep(0.1)  # Simulate processing
                self.processed += 1

            except Exception as e:
                if retry_count >= self.max_retries:
                    # Move to dead letter queue
                    message["_error"] = str(e)
                    message["_dead_lettered_at"] = time.time()
                    await self.redis.rpush("dead_letter_queue", json.dumps(message))
                    self.dead_lettered += 1
                else:
                    # Re-queue with incremented retry count
                    message["_retry_count"] = retry_count + 1
                    await self.redis.rpush("order_queue", json.dumps(message))
                self.errors += 1

    async def report_metrics(self):
        """Report metrics every 10 seconds."""
        while True:
            await asyncio.sleep(10)
            queue_depth = await self.redis.llen("order_queue")
            dlq_depth = await self.redis.llen("dead_letter_queue")
            elapsed = time.time() - self.start_time
            rate = self.processed / elapsed if elapsed > 0 else 0

            print(
                f"[Worker Metrics] "
                f"Queue: {queue_depth} | "
                f"DLQ: {dlq_depth} | "
                f"Processed: {self.processed} | "
                f"Errors: {self.errors} | "
                f"Dead-lettered: {self.dead_lettered} | "
                f"Rate: {rate:.1f}/s"
            )

    async def run(self, redis_url: str):
        """Main worker loop."""
        await self.connect(redis_url)
        print(f"Worker started with concurrency={self.concurrency}")

        asyncio.create_task(self.report_metrics())

        while True:
            # Batch fetch up to 10 messages
            pipe = self.redis.pipeline()
            for _ in range(10):
                pipe.lpop("order_queue")
            messages = await pipe.execute()

            tasks = []
            for msg in messages:
                if msg:
                    try:
                        data = json.loads(msg)
                        tasks.append(self.process_message(data))
                    except json.JSONDecodeError:
                        # Malformed message — send to dead letter queue
                        await self.redis.rpush(
                            "dead_letter_queue",
                            json.dumps({"_raw": msg, "_error": "JSON decode error"})
                        )
                        self.dead_lettered += 1

            if tasks:
                await asyncio.gather(*tasks)
            else:
                await asyncio.sleep(0.1)
```

---

## Part E: k6 Load Test

```javascript
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

const errorRate = new Rate('errors');
const latencyTrend = new Trend('request_latency');

export const options = {
  stages: [
    { duration: '30s', target: 100 },    // Ramp up to normal
    { duration: '30s', target: 1000 },   // SURGE: ramp to 1000 VUs
    { duration: '3m', target: 1000 },    // Hold surge
    { duration: '30s', target: 100 },    // Ramp down
    { duration: '1m', target: 100 },     // Hold normal (recovery)
    { duration: '30s', target: 0 },      // Shutdown
  ],
  thresholds: {
    'errors': ['rate<0.01'],              // < 1% error rate
    'http_req_duration': ['p(95)<2000'],  // p95 < 2 seconds
  },
};

const BASE_URL = __ENV.TARGET_URL || 'http://localhost:8000';

export default function () {
  const userId = `user_${__VU}`;
  const payload = JSON.stringify({
    user_id: userId,
    items: [{ sku: 'FLASH_SALE_ITEM', qty: 1 }],
    total: 49.99,
  });

  const params = {
    headers: {
      'Content-Type': 'application/json',
      'X-Forwarded-For': `192.168.${__VU % 256}.${__VU % 256}`,
    },
  };

  const res = http.post(`${BASE_URL}/orders`, payload, params);

  const success = check(res, {
    'status is 2xx or 429': (r) => r.status === 200 || r.status === 202 || r.status === 429,
    'status is not 500': (r) => r.status !== 500,
    'latency < 5s': (r) => r.timings.duration < 5000,
  });

  errorRate.add(!success);
  latencyTrend.add(res.timings.duration);

  sleep(Math.random() * 0.5 + 0.1);
}
```

**Why this works**: The load test simulates a realistic surge pattern: normal traffic, sudden spike to 1000 VUs, sustained surge, then ramp down. The thresholds enforce that error rate stays below 1% and p95 latency stays below 2 seconds. The check for 429 as acceptable ensures that rate limiting is counted as expected behavior, not an error. The `X-Forwarded-For` header simulates different client IPs to exercise per-client rate limiting.

---

## Common Mistakes
1. **Not testing with realistic traffic patterns**: A constant-rate load test does not simulate a surge. Use k6 stages to ramp up, hold, and ramp down.
2. **Counting 429 as an error**: Rate limiting is working correctly when it returns 429. Your thresholds should accept 429 as a valid response.
3. **Not testing the queue drain**: After the surge subsides, the queue should drain completely. Verify that all queued orders are eventually processed.
4. **Ignoring the dead letter queue**: Messages that repeatedly fail should be moved to a DLQ for manual investigation. Do not let poison messages block the queue.
5. **Not monitoring all layers**: You need metrics from the edge (rate limit hits), ingestion (queue depth), workers (processing rate), and database (connection count). Monitor all layers simultaneously during the load test.

## Relevant README Sections
- [Complete Surge-Handling Architecture](../README.md#complete-surge-handling-architecture)
- [Surge Handler](../README.md#surge-handler)
- [Connection Pooling](../README.md#connection-pooling)
- [Hands-On Lab](../README.md#hands-on-lab-design-a-system-for-10x-traffic)
