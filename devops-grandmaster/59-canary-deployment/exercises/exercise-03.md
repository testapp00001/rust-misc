# Exercise 03: Design Metric-Based Canary Analysis

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective

Design a comprehensive canary analysis system that evaluates multiple
metrics to determine if a canary is healthy. This exercise trains you to
think beyond simple error rates and consider latency, saturation, and
business metrics.

## Scenario

Your `checkout-service` processes payments. A canary deployment that
passes error rate checks could still be problematic if:
- Latency increases by 3x (users abandon slow checkouts)
- Database connection pool saturates (cascading failures downstream)
- Conversion rate drops (the new checkout flow confuses users)

You need an analysis template that evaluates all of these dimensions.

## Tasks

### Part A: Define the Metric Thresholds

For each metric below, define a threshold that determines canary health:

| Metric | Description | Threshold Type |
|--------|-------------|----------------|
| Error rate | 5xx responses / total responses | Max |
| p99 latency | 99th percentile response time | Max |
| p50 latency | Median response time | Max |
| Throughput | Requests per second handled by canary | Min |
| DB connection pool | Active connections / max connections | Max |
| Conversion rate | Completed checkouts / started checkouts | Min |

<details>
<summary>Hint</summary>

Think about what percentage change is acceptable vs the stable version.
For example, p99 latency should not exceed 2x the stable version's p99.
Error rate should not exceed 1%. Conversion rate should not drop more
than 5% relative to stable.

</details>

### Part B: Write the Prometheus Queries

Write Prometheus queries for each metric that compare the canary's
performance to the stable version's performance. The queries should use
the `version` label to distinguish canary from stable pods.

<details>
<summary>Hint 1</summary>

For error rate, compare canary to stable:

```promql
# Canary error rate
sum(rate(http_requests_total{service="checkout",version="canary",status=~"5.."}[5m])) /
sum(rate(http_requests_total{service="checkout",version="canary"}[5m]))
```

</details>

<details>
<summary>Hint 2</summary>

For relative latency comparison:

```promql
# Canary p99 / Stable p99 (should be < 2.0)
histogram_quantile(0.99, sum(rate(http_request_duration_seconds_bucket{service="checkout",version="canary"}[5m])) by (le)) /
histogram_quantile(0.99, sum(rate(http_request_duration_seconds_bucket{service="checkout",version="stable"}[5m])) by (le))
```

</details>

### Part C: Write the Analysis Template

Create an Argo Rollouts `AnalysisTemplate` that evaluates all metrics.
Each metric should have its own `failureLimit` and `count` to handle
transient spikes.

<details>
<summary>Hint</summary>

Use separate metric entries with different sensitivities:

```yaml
metrics:
  - name: error-rate
    interval: 30s
    count: 5
    failureLimit: 1
    successCondition: result[0] <= 0.01
    # ... provider config
  - name: latency-ratio
    interval: 1m
    count: 3
    failureLimit: 2
    successCondition: result[0] <= 2.0
    # ... provider config
```

Error rate has a low failure limit (1) because errors are serious.
Latency has a higher failure limit (2) because latency can spike
transiently.

</details>

### Part D: Design the Weight Progression

Design a canary weight progression that balances speed and safety.
Explain why you chose each step and pause duration.

<details>
<summary>Hint</summary>

Consider:
- Start small (5-10%) to catch obvious issues quickly
- Increase gradually (20%, 40%, 60%, 80%) to catch performance issues
  that only appear under load
- Longer pauses at higher weights because more traffic = more confidence
  needed
- Analysis at every step, not just at the end

</details>

## Success Criteria

- [ ] Thresholds are defined for all 6 metrics with justification
- [ ] Prometheus queries correctly compare canary to stable performance
- [ ] Analysis template handles transient spikes with appropriate failure limits
- [ ] Weight progression balances deployment speed with safety
- [ ] You understand why error rate alone is insufficient for canary analysis

## What You Should Understand After This Exercise

Canary analysis must evaluate multiple dimensions: correctness (error rate),
performance (latency), capacity (throughput, connection pool), and business
outcomes (conversion rate). Each metric needs different thresholds,
sampling intervals, and failure tolerances. A canary that passes error
rate checks but degrades latency is not healthy.
