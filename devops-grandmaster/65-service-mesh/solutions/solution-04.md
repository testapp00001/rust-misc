# Solution 04: Observability and Debugging

## Part A: Metrics Analysis

### 1. Which service is the bottleneck?

**payment-service** is the bottleneck.

Evidence:
- **P99 latency: 6000ms** (6 seconds) -- 30x higher than P50 (200ms)
- **Error rate: 5%** -- 10x higher than other services
- **Active connections: 500 (max: 500)** -- at the connection limit

### 2. What is the root cause?

**Connection pool exhaustion.** The payment-service has reached its maximum
connection count (500). New requests must wait for a connection to become
available before they can be processed.

```
Timeline of a slow request:
  T=0ms:    Request arrives at payment-service sidecar
  T=0ms:    Sidecar checks: all 500 connections in use
  T=0ms:    Request enters connection queue
  T=5800ms: A connection becomes available (previous request completes)
  T=5800ms: Request is forwarded to payment-service
  T=6000ms: Request completes (200ms processing)
  T=6000ms: Response returned to client
```

The 5800ms of "waiting for connection" is the queue time. The actual
processing time is only 200ms.

### 3. Why is the problem intermittent?

The problem affects ~30% of requests because connection availability is
probabilistic:

```
At any given moment:
  - 500 connections are active
  - Each connection takes ~200ms to complete
  - New requests arrive at ~500 req/s

  When all 500 connections are in use (likely at high load):
    - New requests queue up
    - Queue time depends on when a connection frees up
    - Some requests wait 100ms, others wait 5000ms+

  When some connections are free (during low-load moments):
    - Requests are processed immediately
    - No queuing, normal latency
```

The 30% represents the percentage of requests that arrive when the
connection pool is at capacity.

### Fix

```yaml
# Increase connection pool limits
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: payment-service
spec:
  host: payment-service
  trafficPolicy:
    connectionPool:
      tcp:
        maxConnections: 1000  # Increased from 500
      http:
        http1MaxPendingRequests: 1000
        http2MaxRequests: 1000
```

Also consider:
- Horizontal scaling: add more payment-service replicas
- Optimize payment-service: reduce per-request processing time
- Connection pooling: reuse connections instead of creating new ones

## Part B: Distributed Trace Analysis

### 1. What does "waiting for connection" indicate?

The span `waiting for connection: 7500ms` indicates that the request was
**queued** in the sidecar proxy because all connections to payment-service
were in use. The sidecar could not forward the request immediately.

This is NOT application latency. It is infrastructure latency caused by
resource exhaustion.

### 2. Why does payment-service take 7,800ms?

```
Total payment-service span: 7,800ms
  ├─ waiting for connection: 7,500ms  ← Queued (infrastructure issue)
  ├─ POST /charge: 200ms              ← Actual payment processing
  │   └─ stripe.com: 180ms            ← External payment provider
  └─ POST /receipt: 100ms             ← Receipt generation
  ─────────────────────────────────
  Actual processing: 300ms
  Queue time: 7,500ms (96% of the span!)
```

The service itself is fast (300ms). The queue time is the problem.

### 3. How to fix this

**Immediate fixes:**
1. Increase connection pool limits in DestinationRule
2. Add more payment-service replicas (horizontal scaling)
3. Set a timeout on the connection wait (fail fast, do not wait 7.5s)

**Long-term fixes:**
1. Optimize payment-service to handle more concurrent connections
2. Implement connection pooling in the application
3. Add circuit breaker to fail fast when payment-service is overloaded
4. Consider async processing for non-critical operations (receipt email)

```yaml
# Fail fast: timeout after 2s instead of waiting 7.5s
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: payment-service
spec:
  hosts:
  - payment-service
  http:
  - timeout: 2s
    retries:
      attempts: 2
      perTryTimeout: 1s
      retryOn: "5xx,reset,connect-failure"
```

## Part C: Alerting Rules

```yaml
groups:
- name: service-mesh-alerts
  rules:
  # Alert 1: High P99 latency
  - alert: HighP99Latency
    expr: |
      histogram_quantile(0.99,
        sum(rate(istio_request_duration_milliseconds_bucket[5m]))
        by (le, destination_service_name)
      ) > 5000
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High P99 latency on {{ $labels.destination_service_name }}"
      description: "P99 latency is {{ $value }}ms (threshold: 5000ms)"

  # Alert 2: High error rate
  - alert: HighErrorRate
    expr: |
      sum(rate(istio_requests_total{response_code=~"5.."}[5m]))
        by (destination_service_name)
      /
      sum(rate(istio_requests_total[5m]))
        by (destination_service_name)
      > 0.05
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "High error rate on {{ $labels.destination_service_name }}"
      description: "Error rate is {{ $value | humanizePercentage }}"

  # Alert 3: Connection pool utilization
  - alert: ConnectionPoolExhausted
    expr: |
      sum(istio_tcp_connections_opened_total{destination_service_name=~".*"})
        by (destination_service_name)
      /
      sum(istio_tcp_connections_max{destination_service_name=~".*"})
        by (destination_service_name)
      > 0.8
    for: 2m
    labels:
      severity: warning
    annotations:
      summary: "Connection pool near exhaustion on {{ $labels.destination_service_name }}"

  # Alert 4: Circuit breaker open
  - alert: CircuitBreakerOpen
    expr: |
      sum(rate(istio_requests_total{response_flags=~"UO|503"}[5m]))
        by (destination_service_name)
      > 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "Circuit breaker open on {{ $labels.destination_service_name }}"
```

## Part D: Debugging Playbook

```
Slow Checkout Debugging Playbook
═════════════════════════════════

Step 1: Check service mesh metrics dashboard
  └─ Which service has the highest P99 latency?
  └─ Which service has the highest error rate?
  └─ Are any services at connection pool limits?

Step 2: Check distributed traces
  └─ Find a slow trace (filter by duration > 5s)
  └─ Identify which span is longest
  └─ Is the time spent in processing or waiting?

Step 3: Check service logs
  └─ Look for error messages in the slow service
  └─ Check for timeout errors
  └─ Look for connection refused errors

Step 4: Check resource utilization
  └─ CPU and memory of the slow service's pods
  └─ Are pods being OOMKilled?
  └─ Is CPU throttled?

Step 5: Check Kubernetes events
  └─ Recent pod restarts?
  └─ Node pressure or eviction?
  └─ Deployment scaling events?

Step 6: Check external dependencies
  └─ Is the external service (Stripe) responding slowly?
  └─ Are there DNS resolution delays?
  └─ Network connectivity issues?

Step 7: Apply fix
  └─ If connection pool exhausted: increase limits or scale pods
  └─ If external service slow: add timeout and circuit breaker
  └─ If resource constrained: scale horizontally
  └─ If code issue: rollback to previous version
```

### Common Mistakes to Avoid

- **Looking at logs first.** Logs tell you what happened to individual
  requests. Metrics tell you the overall health. Start with metrics to
  identify which service is problematic, then drill into traces and logs.
- **Ignoring P99 latency.** P50 (median) can look fine while P99 is
  terrible. A service with P50=50ms and P99=6000ms has a serious problem
  that P50 hides.
- **Not correlating metrics with traces.** Metrics show the symptom
  (high latency). Traces show the cause (waiting for connection). You
  need both to diagnose effectively.
- **Alerting on P50 instead of P99.** P50 misses tail latency issues.
  Always alert on P95 or P99 for latency-sensitive services.

## Key Takeaway

Service mesh observability provides three complementary views: metrics for
overall health, traces for request-level debugging, and logs for detailed
analysis. The debugging workflow is: metrics (identify which service) ->
traces (identify where in the request) -> logs (identify why). Connection
pool exhaustion is a common cause of intermittent latency that is invisible
in application logs but obvious in service mesh metrics and traces.
