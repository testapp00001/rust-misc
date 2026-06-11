# Module 68: Traffic Surge Handling — Pre-Warming, Queue-Based Architecture

> **Previous Module:** [67 - Auto-Scaling](../67-auto-scaling/README.md)
> **Next Module:** [69 - Capacity Planning](../69-capacity-planning/README.md)

## The Problem

It is Black Friday. Marketing sends the email blast at 9:00 AM. At 9:00:01, your traffic jumps from 1,000 requests/second to 50,000 requests/second. HPA needs 2 minutes to react. Cluster Autoscaler needs 5 minutes to provision nodes. Your database connection pool is exhausted in 10 seconds. Users see 503 errors. The sale is ruined.

The fundamental issue: **auto-scaling is reactive, but surges are instantaneous**. The gap between "surge arrives" and "infrastructure ready" is where outages live.

## The Naive Way

```bash
# "Just over-provision everything, all the time"
kubectl scale deployment web --replicas=200

# Or: "Turn off autoscaling and set high limits"
# This means paying for 200 replicas 24/7 when you need them for 4 hours
```

**Why this fails:**
- Enormous cost (200 replicas running 24/7 for a 4-hour event)
- Database and downstream services still cannot handle the load
- Over-provisioning at the app tier does not fix bottlenecks elsewhere
- Still no protection against DDoS or unexpected spikes

## The Right Way

### Pre-Warming Strategy

Pre-warming means scaling up infrastructure **before** the expected surge.

```bash
#!/bin/bash
# prewarm.sh — Pre-warming script for known traffic events
# Run via CI/CD pipeline or cron before the event

set -euo pipefail

NAMESPACE="production"
EXPECTED_RPS=50000
CURRENT_RPS=$(kubectl get --raw "/apis/custom.metrics.k8s.io/v1beta1/namespaces/$NAMESPACE/pods/*/http_requests_per_second" | jq '.items | length')

echo "=== Pre-Warming for Expected Traffic: $EXPECTED_RPS RPS ==="

# Step 1: Scale up application deployments
echo "[1/5] Scaling application tiers..."
kubectl scale deployment web -n $NAMESPACE --replicas=50
kubectl scale deployment api -n $NAMESPACE --replicas=30
kubectl scale deployment worker -n $NAMESPACE --replicas=20

# Step 2: Scale up Cluster Autoscaler node groups
echo "[2/5] Pre-provisioning nodes..."
# On AWS: increase ASG min size
aws autoscaling update-auto-scaling-group \
  --auto-scaling-group-name eks-production-nodes \
  --min-size 20 \
  --max-size 100

# Step 3: Warm up database connection pools
echo "[3/5] Warming database connections..."
for pod in $(kubectl get pods -n $NAMESPACE -l app=web -o name); do
  kubectl exec -n $NAMESPACE $pod -- curl -s http://localhost:8080/health/warmup
done

# Step 4: Warm up CDN cache
echo "[4/5] Warming CDN cache..."
# Hit popular endpoints through CDN to populate edge caches
for url in $(cat popular-urls.txt); do
  curl -s -o /dev/null "https://cdn.example.com$url" &
done
wait

# Step 5: Verify readiness
echo "[5/5] Verifying readiness..."
kubectl wait --for=condition=available deployment/web -n $NAMESPACE --timeout=300s
kubectl wait --for=condition=available deployment/api -n $NAMESPACE --timeout=300s

echo "=== Pre-warming complete. System ready for traffic surge. ==="
```

### Queue-Based Architecture

Instead of handling requests synchronously (user -> web -> API -> DB -> response), queue-based architecture decouples ingestion from processing.

```yaml
# queue-architecture.yaml — Order processing with queue buffer
# Ingestion tier (handles traffic surge, writes to queue)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: order-ingestion
  namespace: production
spec:
  replicas: 10
  selector:
    matchLabels:
      app: order-ingestion
  template:
    metadata:
      labels:
        app: order-ingestion
    spec:
      containers:
        - name: ingestion
          image: order-ingestion:latest
          resources:
            requests:
              cpu: 250m
              memory: 256Mi
          env:
            - name: RABBITMQ_HOST
              value: "rabbitmq.production.svc"
            - name: MAX_QUEUE_DEPTH
              value: "100000"
          # Lightweight: just validates and queues
          # No database writes, no heavy processing
---
# Processing tier (drains queue at sustainable rate)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: order-processor
  namespace: production
spec:
  replicas: 5
  selector:
    matchLabels:
      app: order-processor
  template:
    metadata:
      labels:
        app: order-processor
    spec:
      containers:
        - name: processor
          image: order-processor:latest
          resources:
            requests:
              cpu: 500m
              memory: 512Mi
          env:
            - name: RABBITMQ_HOST
              value: "rabbitmq.production.svc"
            - name: BATCH_SIZE
              value: "50"
            - name: PROCESSING_RATE
              value: "1000"    # Max orders per second to process
```

**Why queues work for surges:**
1. **Buffering**: Queue absorbs burst; processor drains at sustainable rate
2. **Decoupling**: Ingestion does not depend on database availability
3. **Backpressure**: When queue is full, ingestion rejects gracefully
4. **Retry**: Failed processing is retried automatically

### Backpressure Patterns

```python
# backpressure.py — Application-level backpressure
import asyncio
from collections import deque
import time

class BackpressureQueue:
    """Queue with backpressure — rejects when full."""

    def __init__(self, max_size: int = 10000):
        self.queue = asyncio.Queue(maxsize=max_size)
        self.rejected = 0
        self.processed = 0

    async def enqueue(self, item) -> bool:
        """Try to enqueue. Returns False if queue is full (backpressure)."""
        try:
            self.queue.put_nowait(item)
            return True
        except asyncio.QueueFull:
            self.rejected += 1
            return False

    async def dequeue(self):
        """Dequeue with timeout."""
        try:
            return await asyncio.queue.get(timeout=1.0)
        except asyncio.TimeoutError:
            return None

    @property
    def utilization(self) -> float:
        return self.queue.qsize() / self.queue.maxsize

# Circuit breaker for downstream services
class CircuitBreaker:
    def __init__(self, failure_threshold=5, reset_timeout=30):
        self.failure_count = 0
        self.failure_threshold = failure_threshold
        self.reset_timeout = reset_timeout
        self.state = "CLOSED"    # CLOSED, OPEN, HALF_OPEN
        self.last_failure_time = 0

    async def call(self, func, *args, **kwargs):
        if self.state == "OPEN":
            if time.time() - self.last_failure_time > self.reset_timeout:
                self.state = "HALF_OPEN"
            else:
                raise Exception("Circuit breaker is OPEN")

        try:
            result = await func(*args, **kwargs)
            if self.state == "HALF_OPEN":
                self.state = "CLOSED"
                self.failure_count = 0
            return result
        except Exception as e:
            self.failure_count += 1
            self.last_failure_time = time.time()
            if self.failure_count >= self.failure_threshold:
                self.state = "OPEN"
            raise
```

### Rate Limiting at the Edge

```nginx
# nginx-rate-limit.conf — Edge rate limiting
http {
    # Define rate limit zones
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
        keepalive 256;
    }

    server {
        listen 80;

        # Global rate limit
        limit_req zone=global burst=5000 nodelay;

        # API rate limit per client
        location /api/ {
            limit_req zone=api burst=200 nodelay;
            limit_conn addr 50;

            # Custom error for rate-limited requests
            error_page 429 = @rate_limited;

            proxy_pass http://web_backend;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        }

        # Login endpoint (strict rate limit)
        location /api/auth/login {
            limit_req zone=login burst=10 nodelay;
            proxy_pass http://web_backend;
        }

        # Static assets (no rate limit)
        location /static/ {
            proxy_pass http://web_backend;
        }

        location @rate_limited {
            default_type application/json;
            return 429 '{"error": "rate_limit_exceeded", "retry_after": 1}';
        }
    }
}
```

### Graceful Degradation

```python
# graceful_degradation.py — Serve degraded responses under pressure
import time
from enum import Enum
from functools import wraps

class DegradationLevel(Enum):
    NORMAL = 0          # Full functionality
    ELEVATED = 1        # Disable non-critical features
    HIGH = 2            # Serve cached/static responses only
    CRITICAL = 3        # Serve maintenance page

class GracefulDegradation:
    def __init__(self):
        self.current_level = DegradationLevel.NORMAL
        self.feature_flags = {}

    def check_system_health(self):
        """Determine degradation level based on system metrics."""
        cpu = self._get_avg_cpu()
        error_rate = self._get_error_rate()
        queue_depth = self._get_queue_depth()

        if error_rate > 0.5 or cpu > 0.95:
            self.current_level = DegradationLevel.CRITICAL
        elif error_rate > 0.1 or cpu > 0.85 or queue_depth > 50000:
            self.current_level = DegradationLevel.HIGH
        elif error_rate > 0.05 or cpu > 0.75 or queue_depth > 10000:
            self.current_level = DegradationLevel.ELEVATED
        else:
            self.current_level = DegradationLevel.NORMAL

        self._update_feature_flags()

    def _update_feature_flags(self):
        """Disable features based on degradation level."""
        self.feature_flags = {
            'recommendations': self.current_level.value < 1,    # Disable at ELEVATED
            'search': self.current_level.value < 2,              # Disable at HIGH
            'cart': self.current_level.value < 3,                # Disable at CRITICAL
            'checkout': self.current_level.value < 3,            # Disable at CRITICAL
            'product_listing': self.current_level.value < 3,     # Always on unless CRITICAL
        }

    def require_feature(self, feature_name):
        """Decorator to gate features on degradation level."""
        def decorator(func):
            @wraps(func)
            async def wrapper(*args, **kwargs):
                if not self.feature_flags.get(feature_name, True):
                    return self._degraded_response(feature_name)
                return await func(*args, **kwargs)
            return wrapper
        return decorator

    def _degraded_response(self, feature_name):
        """Return a degraded response for disabled features."""
        responses = {
            'recommendations': {"items": [], "message": "Recommendations temporarily unavailable"},
            'search': {"results": [], "message": "Search temporarily unavailable"},
            'cart': {"error": "Cart temporarily unavailable, please try again"},
        }
        return responses.get(feature_name, {"error": "Service temporarily unavailable"})
```

### CDN Shielding

```
# CDN shielding strategy for traffic surges

User Request Flow:
  User -> Edge CDN (200+ locations)
       -> Cache HIT: Response from edge (< 10ms)
       -> Cache MISS: Forward to Shield

  Shield (Origin Shield — single/few PoPs close to origin)
       -> Cache HIT: Response from shield (< 50ms)
       -> Cache MISS: Forward to Origin

  Origin (Your servers)
       -> Generate response
       -> Cache in Shield and Edge

Result: Only cache misses hit your origin servers.
With 95% cache hit rate at edge, 99% at shield:
  50,000 RPS at edge -> 2,500 RPS at shield -> 250 RPS at origin
```

```yaml
# cloudfront-shield.yaml — AWS CloudFront with Origin Shield
AWSTemplateFormatVersion: '2010-09-09'
Resources:
  CDN:
    Type: AWS::CloudFront::Distribution
    Properties:
      DistributionConfig:
        Enabled: true
        Origins:
          - Id: origin
            DomainName: !GetAtt LoadBalancer.DNSName
            CustomOriginConfig:
              HTTPPort: 80
              OriginProtocolPolicy: https-only
            OriginShield:
              Enabled: true
              OriginShieldRegion: us-east-1
        DefaultCacheBehavior:
          TargetOriginId: origin
          ViewerProtocolPolicy: redirect-to-https
          CachePolicyId: !Ref CachePolicy
          OriginRequestPolicyId: !Ref OriginRequestPolicy
          Compress: true
          # Cache for 1 hour at edge, revalidate with shield
          DefaultTTL: 3600
          MaxTTL: 86400
          MinTTL: 0

  CachePolicy:
    Type: AWS::CloudFront::CachePolicy
    Properties:
      CachePolicyConfig:
        Name: surge-cache-policy
        DefaultTTL: 300
        MaxTTL: 3600
        MinTTL: 0
        ParametersInCacheKeyAndForwardedToOrigin:
          CookiesConfig:
            CookieBehavior: whitelist
            Cookies: [session_id]
          HeadersConfig:
            HeaderBehavior: whitelist
            Headers: [Authorization]
          QueryStringsConfig:
            QueryStringBehavior: all
          EnableAcceptEncodingGzip: true
          EnableAcceptEncodingBrotli: true
```

## The Production Way

### Complete Surge-Handling Architecture

```yaml
# surge-architecture.yaml — Full production surge handling
# Layer 1: Edge (CDN + WAF + Rate Limiting)
# Layer 2: Ingestion (Lightweight, queue-producing)
# Layer 3: Queue (Buffer, backpressure)
# Layer 4: Processing (Sustainable rate, auto-scaled)
# Layer 5: Data (Connection pooled, read-replicated)

# Layer 1: Edge rate limiting via API Gateway
apiVersion: networking.istio.io/v1beta1
kind: EnvoyFilter
metadata:
  name: rate-limit
  namespace: istio-system
spec:
  workloadSelector:
    labels:
      istio: ingressgateway
  configPatches:
    - applyTo: HTTP_FILTER
      match:
        context: GATEWAY
        listener:
          filterChain:
            filter:
              name: envoy.filters.network.http_connection_manager
      patch:
        operation: INSERT_BEFORE
        value:
          name: envoy.filters.http.local_ratelimit
          typed_config:
            "@type": type.googleapis.com/udpa.type.v1.TypedStruct
            type_url: type.googleapis.com/envoy.extensions.filters.http.local_ratelimit.v3.LocalRateLimit
            value:
              stat_prefix: http_local_rate_limiter
              token_bucket:
                max_tokens: 10000
                tokens_per_fill: 10000
                fill_interval: 1s
              filter_enabled:
                runtime_key: local_rate_limit_enabled
                default_value:
                  numerator: 100
                  denominator: HUNDRED
              filter_enforced:
                runtime_key: local_rate_limit_enforced
                default_value:
                  numerator: 100
                  denominator: HUNDRED
```

```python
# surge_handler.py — Application-level surge handling
import asyncio
import aioredis
import json
from datetime import datetime

class SurgeHandler:
    """Production-grade traffic surge handler."""

    def __init__(self, redis_url: str, max_queue: int = 100000):
        self.redis = aioredis.from_url(redis_url)
        self.max_queue = max_queue
        self.metrics = SurgeMetrics()

    async def handle_request(self, request):
        """Handle incoming request with surge protection."""
        # Step 1: Check global rate limit
        if not await self._check_rate_limit(request.client_ip):
            return self._rate_limit_response()

        # Step 2: Check degradation level
        degradation = await self._get_degradation_level()
        if degradation == "CRITICAL":
            return self._maintenance_response()

        # Step 3: Try to enqueue for processing
        if degradation == "HIGH":
            # Serve from cache
            cached = await self._get_cached_response(request.path)
            if cached:
                return cached
            return self._degraded_response()

        # Step 4: Normal processing with queue fallback
        try:
            # Try synchronous processing (fast path)
            result = await asyncio.wait_for(
                self._process_request(request),
                timeout=2.0
            )
            return result
        except asyncio.TimeoutError:
            # Fall back to async processing (queue)
            queued = await self._enqueue_request(request)
            if queued:
                return self._queued_response(request.id)
            return self._overloaded_response()

    async def _check_rate_limit(self, client_ip: str) -> bool:
        """Sliding window rate limiter in Redis."""
        key = f"ratelimit:{client_ip}:{datetime.now().minute}"
        count = await self.redis.incr(key)
        if count == 1:
            await self.redis.expire(key, 60)
        return count <= 100  # 100 requests per minute per IP

    async def _enqueue_request(self, request) -> bool:
        """Enqueue request for async processing."""
        queue_size = await self.redis.llen("request_queue")
        if queue_size >= self.max_queue:
            return False

        await self.redis.rpush("request_queue", json.dumps({
            "id": request.id,
            "path": request.path,
            "body": request.body,
            "timestamp": datetime.now().isoformat()
        }))
        return True

    async def _get_degradation_level(self) -> str:
        """Determine system degradation level."""
        info = await self.redis.info()
        queue_size = await self.redis.llen("request_queue")
        memory_usage = info.get("used_memory", 0) / info.get("maxmemory", 1)

        if queue_size > self.max_queue * 0.9 or memory_usage > 0.95:
            return "CRITICAL"
        elif queue_size > self.max_queue * 0.5 or memory_usage > 0.8:
            return "HIGH"
        elif queue_size > self.max_queue * 0.2:
            return "ELEVATED"
        return "NORMAL"
```

### Connection Pooling

```yaml
# connection-pool.yaml — Database connection pooling with PgBouncer
apiVersion: apps/v1
kind: Deployment
metadata:
  name: pgbouncer
  namespace: production
spec:
  replicas: 3
  selector:
    matchLabels:
      app: pgbouncer
  template:
    metadata:
      labels:
        app: pgbouncer
    spec:
      containers:
        - name: pgbouncer
          image: edoburu/pgbouncer:1.21.0
          ports:
            - containerPort: 5432
          env:
            - name: DATABASE_URL
              value: "postgres://user:pass@postgres-primary:5432/app"
            - name: POOL_MODE
              value: transaction    # transaction pooling for max efficiency
            - name: MAX_CLIENT_CONN
              value: "10000"        # Accept 10K client connections
            - name: DEFAULT_POOL_SIZE
              value: "100"          # But only 100 actual DB connections
            - name: MIN_POOL_SIZE
              value: "25"
            - name: RESERVE_POOL_SIZE
              value: "50"           # Extra connections for surge
            - name: RESERVE_POOL_TIMEOUT
              value: "5"
            - name: SERVER_IDLE_TIMEOUT
              value: "300"
            - name: QUERY_TIMEOUT
              value: "30"
          resources:
            requests:
              cpu: 500m
              memory: 256Mi
            limits:
              cpu: 1
              memory: 512Mi
```

### Async Processing with Workers

```yaml
# async-workers.yaml — Worker deployment for queue processing
apiVersion: apps/v1
kind: Deployment
metadata:
  name: order-worker
  namespace: production
spec:
  replicas: 5
  selector:
    matchLabels:
      app: order-worker
  template:
    metadata:
      labels:
        app: order-worker
    spec:
      containers:
        - name: worker
          image: order-worker:latest
          env:
            - name: QUEUE_URL
              value: "amqp://rabbitmq:5672"
            - name: CONCURRENCY
              value: "50"           # 50 concurrent tasks per worker
            - name: PREFETCH_COUNT
              value: "10"           # Fetch 10 messages at a time
            - name: MAX_RETRIES
              value: "3"
            - name: RETRY_DELAY
              value: "30"           # 30 seconds between retries
```

## Hands-On Lab: Design a System for 10x Traffic

### Scenario

You run an e-commerce platform. Normal traffic: 5,000 RPS. Flash sale traffic: 50,000 RPS (10x). Design and implement a system that handles the surge without dropping requests.

### Step 1: Set Up the Base Infrastructure

```bash
# Start local Kubernetes cluster
kind create cluster --name surge-lab

# Install RabbitMQ
helm repo add bitnami https://charts.bitnami.com/bitnami
helm install rabbitmq bitnami/rabbitmq \
  --set auth.username=admin \
  --set auth.password=admin123 \
  --set persistence.size=10Gi

# Install Redis
helm install redis bitnami/redis \
  --set auth.password=redis123 \
  --set master.persistence.size=5Gi
```

### Step 2: Deploy the Ingestion Layer

```python
# ingestion.py — Lightweight ingestion service
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import aioredis
import uuid
import json

app = FastAPI()
redis = aioredis.from_url("redis://redis:6379", password="redis123")

class OrderRequest(BaseModel):
    user_id: str
    items: list
    total: float

@app.post("/orders")
async def create_order(order: OrderRequest):
    """Ingest order into queue for async processing."""
    order_id = str(uuid.uuid4())

    # Check rate limit
    count = await redis.incr(f"rate:{order.user_id}")
    if count == 1:
        await redis.expire(f"rate:{order.user_id}", 60)
    if count > 10:
        raise HTTPException(429, "Rate limit exceeded")

    # Check queue depth (backpressure)
    queue_size = await redis.llen("order_queue")
    if queue_size > 50000:
        raise HTTPException(503, "System overloaded, try again later")

    # Enqueue
    await redis.rpush("order_queue", json.dumps({
        "order_id": order_id,
        "user_id": order.user_id,
        "items": order.items,
        "total": order.total
    }))

    # Check order status endpoint
    await redis.setex(f"order:{order_id}:status", 3600, "queued")

    return {"order_id": order_id, "status": "queued"}

@app.get("/orders/{order_id}/status")
async def get_order_status(order_id: str):
    status = await redis.get(f"order:{order_id}:status")
    if not status:
        raise HTTPException(404, "Order not found")
    return {"order_id": order_id, "status": status.decode()}
```

### Step 3: Deploy the Worker Layer

```python
# worker.py — Queue consumer with controlled processing rate
import asyncio
import aioredis
import json
import time

class OrderWorker:
    def __init__(self, concurrency=50):
        self.redis = aioredis.from_url("redis://redis:6379", password="redis123")
        self.concurrency = concurrency
        self.semaphore = asyncio.Semaphore(concurrency)
        self.processed = 0
        self.errors = 0

    async def process_order(self, order_data: dict):
        """Process a single order."""
        order_id = order_data["order_id"]
        async with self.semaphore:
            try:
                # Update status
                await self.redis.setex(
                    f"order:{order_id}:status", 3600, "processing"
                )

                # Simulate processing (database write, payment, etc.)
                await asyncio.sleep(0.1)

                # Mark complete
                await self.redis.setex(
                    f"order:{order_id}:status", 3600, "completed"
                )
                self.processed += 1
            except Exception as e:
                await self.redis.setex(
                    f"order:{order_id}:status", 3600, f"failed: {e}"
                )
                self.errors += 1

    async def run(self):
        """Main worker loop."""
        print(f"Worker started with concurrency={self.concurrency}")

        while True:
            # Fetch batch of messages
            pipe = self.redis.pipeline()
            for _ in range(10):
                pipe.lpop("order_queue")
            messages = await pipe.execute()

            tasks = []
            for msg in messages:
                if msg:
                    order_data = json.loads(msg)
                    tasks.append(self.process_order(order_data))

            if tasks:
                await asyncio.gather(*tasks)

            # Metrics
            queue_size = await self.redis.llen("order_queue")
            print(f"Queue: {queue_size}, Processed: {self.processed}, Errors: {self.errors}")

            await asyncio.sleep(0.1)

if __name__ == "__main__":
    worker = OrderWorker()
    asyncio.run(worker.run())
```

### Step 4: Generate 10x Load

```javascript
// k6-load-test.js
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

const errorRate = new Rate('errors');

export let options = {
  stages: [
    { duration: '1m', target: 100 },     // Ramp to 100 VUs (normal)
    { duration: '2m', target: 100 },     // Hold normal
    { duration: '30s', target: 1000 },   // SURGE: 10x traffic
    { duration: '3m', target: 1000 },    // Hold surge
    { duration: '1m', target: 100 },     // Ramp down
    { duration: '2m', target: 100 },     // Hold normal
    { duration: '30s', target: 0 },      // Shutdown
  ],
  thresholds: {
    'errors': ['rate<0.01'],             // < 1% error rate
    'http_req_duration': ['p(95)<2000'], // 95th percentile < 2s
  },
};

export default function () {
  const payload = JSON.stringify({
    user_id: `user_${__VU}`,
    items: [{ sku: 'ITEM001', qty: 1 }],
    total: 29.99,
  });

  const params = { headers: { 'Content-Type': 'application/json' } };
  const res = http.post('http://localhost:8000/orders', payload, params);

  check(res, {
    'status is 2xx': (r) => r.status >= 200 && r.status < 300,
    'status is not 503': (r) => r.status !== 503,
  });

  errorRate.add(res.status >= 500);
  sleep(0.1);
}
```

### Step 5: Observe and Validate

```bash
# Run load test
k6 run k6-load-test.js

# Monitor queue depth in real-time
watch -n 1 'redis-cli llen order_queue'

# Monitor worker processing rate
kubectl logs -f deployment/order-worker

# Check for backpressure (429/503 responses)
kubectl logs -f deployment/ingestion | grep -E "429|503"

# Monitor HPA scaling
watch -n 2 kubectl get hpa
```

### Lab Validation Checklist

- [ ] Ingestion layer handles 50K RPS without crashing
- [ ] Queue absorbs traffic burst (depth increases during surge)
- [ ] Workers process queue at sustainable rate
- [ ] No orders are lost (queue drained completely after test)
- [ ] Error rate stays below 1% during surge
- [ ] Rate limiting blocks abusive clients
- [ ] System returns to normal after surge subsides

## Limitation -> Next Topic

You can handle a 10x surge now — but what about 50x? Or 100x? How do you know your limits before the traffic arrives? How do you plan capacity for growth, not just for known events?

The answer is systematic capacity planning: load testing to find your breaking points, forecasting growth, and making data-driven infrastructure decisions.

**Next: [Module 69 — Capacity Planning](../69-capacity-planning/README.md)**
