# Exercise 04: Observability and Debugging

**Type:** Challenge
**Time:** 45 min
**Difficulty:** Medium-Hard

## Objective

Use service mesh observability tools (metrics, traces, logs) to diagnose performance issues and failures in a microservices architecture.

## Scenario

Users report that the checkout flow is slow. The architecture:

```
Checkout flow:
  client → frontend → api-service → payment-service
                                      → inventory-service
                                      → notification-service

Symptoms:
- Checkout takes 8 seconds (normally 2 seconds)
- Some checkouts fail with 504 Gateway Timeout
- The problem is intermittent (affects ~30% of requests)
- No recent code deployments
```

## Tasks

### Part A: Metrics Analysis

Given these metrics from the service mesh dashboard, identify the bottleneck:

```
Service Metrics (last 5 minutes):

frontend:
  Request rate: 500 req/s
  P50 latency: 45ms
  P99 latency: 120ms
  Error rate: 0.1%

api-service:
  Request rate: 500 req/s
  P50 latency: 150ms
  P99 latency: 800ms
  Error rate: 0.5%

payment-service:
  Request rate: 500 req/s
  P50 latency: 200ms
  P99 latency: 6000ms
  Error rate: 5%
  Active connections: 500 (max: 500)

inventory-service:
  Request rate: 500 req/s
  P50 latency: 50ms
  P99 latency: 100ms
  Error rate: 0.1%

notification-service:
  Request rate: 500 req/s
  P50 latency: 30ms
  P99 latency: 80ms
  Error rate: 0.0%
```

1. Which service is the bottleneck?
2. What is the root cause?
3. Why is the problem intermittent?

<details>
<summary>Hint</summary>

Look at P99 latency (99th percentile) and active connections. A service at its connection limit will queue requests, causing high P99 latency and timeouts for some requests.

</details>

### Part B: Distributed Trace Analysis

Given this trace for a slow checkout request, identify the problem:

```
Trace ID: abc-123-def-456
Total duration: 8,234ms

[frontend] ──────────────────────────────────────────── 120ms
  [api-service] ────────────────────────────────────── 8,100ms
    [payment-service] ──────────────────────────────── 7,800ms
      ├─ POST /charge ─────────────────────────────── 200ms
      │   └─ [external] stripe.com ────────────────── 180ms
      ├─ waiting for connection ───────────────────── 7,500ms
      └─ POST /receipt ────────────────────────────── 100ms
    [inventory-service] ───────────────────────────── 50ms
      └─ GET /stock ───────────────────────────────── 50ms
    [notification-service] ────────────────────────── 30ms
      └─ POST /email ──────────────────────────────── 30ms
```

1. What does the "waiting for connection" span indicate?
2. Why does the payment-service take 7,800ms when the actual charge is only 200ms?
3. How would you fix this?

<details>
<summary>Hint</summary>

"Waiting for connection" means the request is queued because all connections to payment-service are in use. The service is at its connection pool limit. Requests must wait for a connection to become available.

</details>

### Part C: Alerting Rules

Design Prometheus alerting rules for this architecture:

1. Alert when P99 latency exceeds 5 seconds for any service
2. Alert when error rate exceeds 5% for any service
3. Alert when connection pool utilization exceeds 80%
4. Alert when circuit breaker opens for any service

Write the Prometheus rules in YAML format.

<details>
<summary>Hint</summary>

Use `histogram_quantile(0.99, ...)` for P99 latency. Use `rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m])` for error rate. Service mesh metrics are typically exported with `istio_` or `envoy_` prefixes.

</details>

### Part D: Debugging Playbook

Write a step-by-step debugging playbook for "slow checkout" incidents:

```
Step 1: Check _______________
Step 2: Check _______________
Step 3: Check _______________
...
```

<details>
<summary>Hint</summary>

Start with metrics (which service is slow?), then traces (where is time being spent?), then logs (what errors are occurring?). The service mesh provides all three through its sidecar proxy metrics and integration with tracing systems.

</details>

## Success Criteria

- [ ] You can analyze service mesh metrics to identify bottlenecks
- [ ] You can read distributed traces to find the root cause of latency
- [ ] You can design alerting rules for service mesh observability
- [ ] You can create a debugging playbook for common incidents

## What You Should Understand After This Exercise

A service mesh provides three pillars of observability: metrics (request rates, latencies, error rates), distributed traces (end-to-end request flow), and logs (individual request details). Together, they enable rapid diagnosis of performance issues. The key skill is knowing which tool to use first: metrics for overview, traces for root cause, logs for details.
