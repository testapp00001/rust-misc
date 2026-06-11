# Solution 03: Implementing Backpressure and Rate Limiting

## Part A: Rate Limiting Strategy

### Tiered Rate Limiting Configuration

```yaml
# Envoy/Istio rate limiting configuration
# Layer 1: Edge rate limiting (global)
apiVersion: networking.istio.io/v1beta1
kind: EnvoyFilter
metadata:
  name: global-rate-limit
  namespace: istio-system
spec:
  workloadSelector:
    labels:
      istio: ingressgateway
  configPatches:
    - applyTo: HTTP_FILTER
      match:
        context: GATEWAY
      patch:
        operation: INSERT_BEFORE
        value:
          name: envoy.filters.http.local_ratelimit
          typed_config:
            "@type": type.googleapis.com/udpa.type.v1.TypedStruct
            type_url: type.googleapis.com/envoy.extensions.filters.http.local_ratelimit.v3.LocalRateLimit
            value:
              stat_prefix: global_rate_limiter
              token_bucket:
                max_tokens: 15000        # Backend capacity
                tokens_per_fill: 15000
                fill_interval: 1s
```

```nginx
# Nginx per-client rate limiting
http {
    # Identify client type by API key or header
    map $http_x_api_key $client_type {
        default          "scraper";
        "mobile_*"       "mobile";
        "partner_*"      "partner";
    }

    # Per-client rate limit zones
    limit_req_zone $client_type zone=mobile:10m rate=20000r/s;
    limit_req_zone $client_type zone=partner:10m rate=5000r/s;
    limit_req_zone $client_type zone=scraper:10m rate=1000r/s;

    server {
        location /api/ {
            # Apply rate limits based on client type
            if ($client_type = "mobile") {
                set $limit_zone "mobile";
                set $limit_burst 5000;
            }
            if ($client_type = "partner") {
                set $limit_zone "partner";
                set $limit_burst 1000;
            }
            if ($client_type = "scraper") {
                set $limit_zone "scraper";
                set $limit_burst 200;
            }

            limit_req zone=$client_type burst=$limit_burst nodelay;
            limit_req_status 429;

            # Custom error response
            error_page 429 = @rate_limited;
        }

        location @rate_limited {
            default_type application/json;
            return 429 '{"error": "rate_limit_exceeded", "retry_after": 1}';
        }
    }
}
```

### Rate Limit Summary

| Client | Rate Limit | Burst | Exceeded Response | Enforcement Point |
|--------|-----------|-------|-------------------|-------------------|
| Mobile app | 20,000 RPS | 5,000 | 429 + Retry-After | Gateway |
| Partner API | 5,000 RPS | 1,000 | 429 + Retry-After | Gateway |
| Web scraper | 1,000 RPS | 200 | 429 + Retry-After | Edge (WAF) |
| Global | 15,000 RPS | 3,000 | 503 + degraded page | Edge |

### Why This Works

- **Global limit (15,000 RPS)** protects the backend regardless of client mix
- **Per-client limits** enforce fairness and SLAs
- **Burst allowances** handle short spikes without dropping requests
- **Gateway enforcement** means rate-limited requests never reach the backend
- **429 status** tells well-behaved clients to retry after a delay

---

## Part B: Application-Level Backpressure

```python
import time
from enum import Enum
from functools import wraps
from fastapi import FastAPI, Request, HTTPException
from fastapi.responses import JSONResponse

class LoadLevel(Enum):
    NORMAL = 0
    ELEVATED = 1
    HIGH = 2
    CRITICAL = 3

class ClientPriority(Enum):
    HIGH = 0      # Mobile app
    MEDIUM = 1    # Partner API
    LOW = 2       # Scrapers

class BackpressureManager:
    def __init__(self):
        self.current_level = LoadLevel.NORMAL
        self.last_level_change = time.time()

    def get_load_level(self, queue_depth: int, error_rate: float,
                       cpu_util: float, p99_latency_ms: float) -> LoadLevel:
        """Determine current load level with hysteresis."""
        # Entering thresholds (more sensitive)
        enter_thresholds = {
            LoadLevel.ELEVATED: {
                "queue_depth": 5000, "error_rate": 0.02,
                "cpu_util": 0.70, "p99_latency_ms": 500
            },
            LoadLevel.HIGH: {
                "queue_depth": 20000, "error_rate": 0.05,
                "cpu_util": 0.85, "p99_latency_ms": 1000
            },
            LoadLevel.CRITICAL: {
                "queue_depth": 50000, "error_rate": 0.15,
                "cpu_util": 0.95, "p99_latency_ms": 3000
            },
        }
        # Exiting thresholds (less sensitive -- hysteresis)
        exit_thresholds = {
            LoadLevel.ELEVATED: {
                "queue_depth": 3000, "error_rate": 0.01,
                "cpu_util": 0.60, "p99_latency_ms": 300
            },
            LoadLevel.HIGH: {
                "queue_depth": 10000, "error_rate": 0.03,
                "cpu_util": 0.75, "p99_latency_ms": 700
            },
            LoadLevel.CRITICAL: {
                "queue_depth": 30000, "error_rate": 0.10,
                "cpu_util": 0.90, "p99_latency_ms": 2000
            },
        }

        # Determine target level based on entering thresholds
        target = LoadLevel.NORMAL
        for level in [LoadLevel.CRITICAL, LoadLevel.HIGH, LoadLevel.ELEVATED]:
            t = enter_thresholds[level]
            if (queue_depth >= t["queue_depth"] or
                error_rate >= t["error_rate"] or
                cpu_util >= t["cpu_util"] or
                p99_latency_ms >= t["p99_latency_ms"]):
                target = level
                break

        # Apply hysteresis: only change if we've been at current level for 30s
        if target != self.current_level:
            if time.time() - self.last_level_change > 30:
                self.current_level = target
                self.last_level_change = time.time()

        return self.current_level

    def should_accept(self, priority: ClientPriority) -> bool:
        """Decide whether to accept a request based on client priority and load."""
        # Priority HIGH is accepted at all levels except CRITICAL
        # Priority MEDIUM is accepted at NORMAL and ELEVATED
        # Priority LOW is accepted only at NORMAL
        acceptance_map = {
            LoadLevel.NORMAL:  {ClientPriority.HIGH, ClientPriority.MEDIUM, ClientPriority.LOW},
            LoadLevel.ELEVATED: {ClientPriority.HIGH, ClientPriority.MEDIUM},
            LoadLevel.HIGH:    {ClientPriority.HIGH},
            LoadLevel.CRITICAL: set(),  # Reject all
        }
        return priority in acceptance_map[self.current_level]

    def get_response(self, priority: ClientPriority) -> dict:
        """Get the response for a rejected request."""
        if self.current_level == LoadLevel.CRITICAL:
            return {"status": 503, "body": {
                "error": "service_overloaded",
                "message": "System is experiencing extreme load. Please try again later.",
                "retry_after": 30
            }}
        else:
            return {"status": 429, "body": {
                "error": "rate_limit_exceeded",
                "message": f"Request deprioritized due to system load (level: {self.current_level.name}).",
                "retry_after": 5
            }}
```

### Why This Works

- **Hysteresis** (30-second delay between level changes) prevents oscillation when metrics hover near thresholds
- **Priority mapping** ensures high-value traffic (mobile) is served longer than low-value traffic (scrapers)
- **Separate enter/exit thresholds** mean the system must recover significantly before degrading its load level
- **503 for CRITICAL** tells all clients the system is down; **429 for ELEVATED/HIGH** tells deprioritized clients to retry

---

## Part C: Circuit Breaker Integration

```yaml
# Circuit breaker configuration for payment processor
# Envoy/Istio DestinationRule
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: payment-processor
  namespace: production
spec:
  host: payment-processor.production.svc.cluster.local
  trafficPolicy:
    connectionPool:
      tcp:
        maxConnections: 100
      http:
        h2UpgradePolicy: DEFAULT
        http1MaxPendingRequests: 100
        http2MaxRequests: 1000
        maxRequestsPerConnection: 10
        maxRetries: 3
    outlierDetection:
      consecutive5xxErrors: 5          # Open circuit after 5 failures
      interval: 30s                     # Check every 30 seconds
      baseEjectionTime: 30s             # Circuit open for 30 seconds
      maxEjectionPercent: 50            # Eject at most 50% of hosts
      minHealthPercent: 30              # Keep at least 30% healthy
    outlierDetection:
      # Half-open: allow 3 test requests
      consecutiveGatewayErrors: 3
```

### Circuit Breaker State Transitions

```
CLOSED (normal operation)
  |
  | 5 consecutive failures in 30s
  v
OPEN (fail fast, return cached/default response)
  |
  | 30 seconds elapsed
  v
HALF_OPEN (allow 3 test requests)
  |
  |-- All 3 succeed --> CLOSED
  |-- Any fail --> OPEN (reset timer)
```

### Application-Level Circuit Breaker

```python
import time

class CircuitBreaker:
    def __init__(self, failure_threshold=5, recovery_timeout=30,
                 half_open_requests=3):
        self.failure_threshold = failure_threshold
        self.recovery_timeout = recovery_timeout
        self.half_open_requests = half_open_requests

        self.state = "CLOSED"
        self.failure_count = 0
        self.last_failure_time = 0
        self.half_open_count = 0

    async def call(self, func, *args, **kwargs):
        if self.state == "OPEN":
            if time.time() - self.last_failure_time > self.recovery_timeout:
                self.state = "HALF_OPEN"
                self.half_open_count = 0
            else:
                raise CircuitOpenError("Circuit breaker is OPEN")

        if self.state == "HALF_OPEN":
            if self.half_open_count >= self.half_open_requests:
                # All test requests succeeded
                self.state = "CLOSED"
                self.failure_count = 0
            else:
                self.half_open_count += 1

        try:
            result = await func(*args, **kwargs)
            if self.state == "HALF_OPEN":
                # Test request succeeded
                if self.half_open_count >= self.half_open_requests:
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

### Why This Works

- **Circuit opens after 5 failures** prevents sending requests to a failing service
- **30-second recovery timeout** gives the downstream service time to recover
- **Half-open with 3 test requests** verifies recovery before fully closing the circuit
- **Fast failure** (when circuit is OPEN) returns immediately instead of waiting for timeouts
- **The client gets a meaningful error** instead of a 5-second timeout

---

## Common Mistakes to Avoid

- **No hysteresis on backpressure.** Without hysteresis, the system oscillates rapidly between load levels when metrics are near thresholds, causing unpredictable behavior.

- **Rate limiting at the application instead of the edge.** Rate-limited requests still consume application resources (connections, memory). Rate limit at the edge (CDN, WAF, gateway) to reject before they reach your infrastructure.

- **Circuit breaker too sensitive.** Setting the failure threshold too low (e.g., 1 failure) causes the circuit to open on transient errors. Use 5-10 failures over a time window.

- **Not returning Retry-After headers.** Without `Retry-After`, well-behaved clients retry immediately, making the overload worse.

- **Treating all clients equally.** During a surge, you must prioritize. A paying customer's request is more valuable than a scraper's request.

## Key Takeaway

Rate limiting, backpressure, and circuit breakers are complementary protection mechanisms. Rate limiting controls who can enter, backpressure controls how much you accept, and circuit breakers protect against downstream failures. Together, they ensure your system degrades gracefully rather than crashing catastrophically.

## Relevant README Sections
- [Queue-Based Architecture](../README.md#queue-based-architecture)
- [Backpressure Patterns](../README.md#backpressure-patterns)
- [Async Processing with Workers](../README.md#async-processing-with-workers)
