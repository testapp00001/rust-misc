# Solution 03: Design Metric-Based Canary Analysis

## Part A: Metric Thresholds

| Metric | Threshold | Justification |
|--------|-----------|---------------|
| **Error rate** | Max 1% | Industry standard for API SLOs. Above 1%, users notice errors. |
| **p99 latency** | Max 2x stable | If stable p99 is 150ms, canary should not exceed 300ms. Beyond 2x, user experience degrades noticeably. |
| **p50 latency** | Max 1.5x stable | Median latency affects most users. Even a 50% increase means slower experience for the majority. |
| **Throughput** | Min 80% of stable | If canary handles less traffic than expected, it might be crashing or rejecting requests. |
| **DB connection pool** | Max 80% | At 80%, the pool is at risk of exhaustion under load spikes. Above 80%, new requests may queue or fail. |
| **Conversion rate** | Min 95% of stable | A 5% drop in conversion represents real revenue loss. Even small UX regressions can cause this. |

### Why This Matters

No single metric tells the full story. A canary with 0% error rate but
3x latency is not healthy. A canary with good latency but 10% lower
conversion rate is not healthy. Multi-dimensional analysis catches
issues that single-metric analysis misses.

## Part B: Prometheus Queries

```yaml
# Error rate (absolute)
query: |
  sum(rate(http_requests_total{
    service="checkout",
    version="canary",
    status=~"5.."
  }[5m])) /
  sum(rate(http_requests_total{
    service="checkout",
    version="canary"
  }[5m]))

# p99 latency ratio (canary / stable)
query: |
  histogram_quantile(0.99,
    sum(rate(http_request_duration_seconds_bucket{
      service="checkout",
      version="canary"
    }[5m])) by (le)
  ) /
  histogram_quantile(0.99,
    sum(rate(http_request_duration_seconds_bucket{
      service="checkout",
      version="stable"
    }[5m])) by (le)
  )

# p50 latency ratio (canary / stable)
query: |
  histogram_quantile(0.50,
    sum(rate(http_request_duration_seconds_bucket{
      service="checkout",
      version="canary"
    }[5m])) by (le)
  ) /
  histogram_quantile(0.50,
    sum(rate(http_request_duration_seconds_bucket{
      service="checkout",
      version="stable"
    }[5m])) by (le)
  )

# Throughput (canary requests/sec)
query: |
  sum(rate(http_requests_total{
    service="checkout",
    version="canary"
  }[5m]))

# DB connection pool utilization
query: |
  db_connection_pool_active{
    service="checkout",
    version="canary"
  } /
  db_connection_pool_max{
    service="checkout",
    version="canary"
  }

# Conversion rate (canary / stable)
query: |
  (
    sum(rate(checkout_completed_total{
      service="checkout",
      version="canary"
    }[5m])) /
    sum(rate(checkout_started_total{
      service="checkout",
      version="canary"
    }[5m]))
  ) /
  (
    sum(rate(checkout_completed_total{
      service="checkout",
      version="stable"
    }[5m])) /
    sum(rate(checkout_started_total{
      service="checkout",
      version="stable"
    }[5m]))
  )
```

### Why This Works

Each query isolates the canary's metrics using the `version="canary"`
label. Ratio queries (latency, conversion) compare canary to stable,
so thresholds are relative rather than absolute. This means the analysis
adapts to the current baseline -- if stable latency is already 200ms,
the canary threshold is 400ms, not a fixed number.

The 5-minute window `[5m]` balances responsiveness (shorter window
detects issues faster) with stability (longer window smooths out
transient spikes).

## Part C: Analysis Template

```yaml
apiVersion: argoproj.io/v1alpha1
kind: AnalysisTemplate
metadata:
  name: checkout-canary-analysis
spec:
  args:
    - name: service-name
      value: checkout
  metrics:
    # Critical: Error rate (fast detection, low tolerance)
    - name: error-rate
      interval: 30s
      count: 10
      failureLimit: 1
      successCondition: result[0] <= 0.01
      provider:
        prometheus:
          address: http://prometheus:9090
          query: |
            sum(rate(http_requests_total{
              service="{{args.service-name}}",
              version="canary",
              status=~"5.."
            }[5m])) /
            sum(rate(http_requests_total{
              service="{{args.service-name}}",
              version="canary"
            }[5m]))

    # High: p99 latency ratio (moderate detection, some tolerance)
    - name: latency-p99
      interval: 1m
      count: 5
      failureLimit: 2
      successCondition: result[0] <= 2.0
      provider:
        prometheus:
          address: http://prometheus:9090
          query: |
            histogram_quantile(0.99,
              sum(rate(http_request_duration_seconds_bucket{
                service="{{args.service-name}}",
                version="canary"
              }[5m])) by (le)
            ) /
            histogram_quantile(0.99,
              sum(rate(http_request_duration_seconds_bucket{
                service="{{args.service-name}}",
                version="stable"
              }[5m])) by (le)
            )

    # Medium: p50 latency ratio
    - name: latency-p50
      interval: 1m
      count: 5
      failureLimit: 2
      successCondition: result[0] <= 1.5
      provider:
        prometheus:
          address: http://prometheus:9090
          query: |
            histogram_quantile(0.50,
              sum(rate(http_request_duration_seconds_bucket{
                service="{{args.service-name}}",
                version="canary"
              }[5m])) by (le)
            ) /
            histogram_quantile(0.50,
              sum(rate(http_request_duration_seconds_bucket{
                service="{{args.service-name}}",
                version="stable"
              }[5m])) by (le)
            )

    # Medium: Throughput
    - name: throughput
      interval: 1m
      count: 5
      failureLimit: 2
      successCondition: result[0] >= 80
      provider:
        prometheus:
          address: http://prometheus:9090
          query: |
            sum(rate(http_requests_total{
              service="{{args.service-name}}",
              version="canary"
            }[5m]))

    # High: DB connection pool
    - name: db-connections
      interval: 1m
      count: 5
      failureLimit: 1
      successCondition: result[0] <= 0.80
      provider:
        prometheus:
          address: http://prometheus:9090
          query: |
            db_connection_pool_active{
              service="{{args.service-name}}",
              version="canary"
            } /
            db_connection_pool_max{
              service="{{args.service-name}}",
              version="canary"
            }

    # High: Conversion rate
    - name: conversion-rate
      interval: 2m
      count: 3
      failureLimit: 1
      successCondition: result[0] >= 0.95
      provider:
        prometheus:
          address: http://prometheus:9090
          query: |
            (
              sum(rate(checkout_completed_total{
                service="{{args.service-name}}",
                version="canary"
              }[5m])) /
              sum(rate(checkout_started_total{
                service="{{args.service-name}}",
                version="canary"
              }[5m]))
            ) /
            (
              sum(rate(checkout_completed_total{
                service="{{args.service-name}}",
                version="stable"
              }[5m])) /
              sum(rate(checkout_started_total{
                service="{{args.service-name}}",
                version="stable"
              }[5m]))
            )
```

### Why This Works

Each metric has different sensitivity settings:

| Metric | Interval | Count | FailureLimit | Rationale |
|--------|----------|-------|--------------|-----------|
| Error rate | 30s | 10 | 1 | Critical: detect fast, fail fast |
| Latency p99 | 1m | 5 | 2 | Important: allow transient spikes |
| Latency p50 | 1m | 5 | 2 | Important: allow transient spikes |
| Throughput | 1m | 5 | 2 | Important: allow transient spikes |
| DB connections | 1m | 5 | 1 | Critical: pool exhaustion cascades |
| Conversion | 2m | 3 | 1 | Critical: direct revenue impact |

Error rate and DB connections have `failureLimit: 1` because these are
hard failures -- errors and pool exhaustion should not be tolerated.
Latency and throughput have `failureLimit: 2` because they can spike
transiently due to GC pauses or network blips.

Conversion rate has a longer `interval` (2m) because conversion events
are less frequent than HTTP requests. You need a longer window to get
statistical significance.

## Part D: Weight Progression

```yaml
steps:
  # Step 1: Initial canary -- catch obvious issues
  - setWeight: 5
  - pause: {duration: 2m}
  - analysis:
      templates: [{templateName: checkout-canary-analysis}]

  # Step 2: Small increase -- validate under light load
  - setWeight: 15
  - pause: {duration: 3m}
  - analysis:
      templates: [{templateName: checkout-canary-analysis}]

  # Step 3: Moderate increase -- validate under medium load
  - setWeight: 30
  - pause: {duration: 5m}
  - analysis:
      templates: [{templateName: checkout-canary-analysis}]

  # Step 4: Majority traffic -- validate under production-like load
  - setWeight: 60
  - pause: {duration: 5m}
  - analysis:
      templates: [{templateName: checkout-canary-analysis}]

  # Step 5: Full traffic
  - setWeight: 100
```

### Why This Progression

| Weight | Pause | Rationale |
|--------|-------|-----------|
| 5% | 2m | Quick check for obvious errors. Low risk, fast feedback. |
| 15% | 3m | Enough traffic for latency percentiles to stabilize. |
| 30% | 5m | Enough load to trigger connection pool and throughput issues. |
| 60% | 5m | Production-like load. Conversion rate becomes statistically reliable. |
| 100% | - | Full promotion after all gates pass. |

The pause durations increase at higher weights because the consequences
of a bug increase with traffic percentage. At 5%, a bug affects 500
req/s. At 60%, it affects 6,000 req/s. The longer pause at 60% gives
more time to observe metrics before committing to 100%.

## Common Mistakes to Avoid

- **Only checking error rate.** A canary with 0% errors but 5x latency
  is not healthy. Users experience slowness, not errors. Always include
  latency metrics.
- **Using absolute thresholds instead of relative.** A p99 threshold of
  300ms is meaningless if stable is already at 280ms. Use ratios
  (canary/stable) so thresholds adapt to the current baseline.
- **Too-short analysis windows.** Running analysis for 30 seconds at 5%
  traffic gives you ~250 requests. That is not enough to detect a 1%
  error rate with confidence. Use longer windows at lower traffic weights.
- **Not accounting for metric cardinality.** If your metrics have high
  cardinality labels (e.g., user_id), Prometheus queries can be slow.
  Keep analysis queries focused on aggregate metrics.

## Key Takeaway

Effective canary analysis requires multiple metrics evaluated at different
sensitivities. Error rate is the most critical, but latency, throughput,
and business metrics (like conversion rate) catch issues that error rate
misses. Each metric needs its own interval, count, and failure limit
based on its importance and signal characteristics.
