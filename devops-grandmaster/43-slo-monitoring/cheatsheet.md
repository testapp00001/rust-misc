# Cheatsheet: SLO Monitoring

## SLI / SLO / SLA Hierarchy

```
SLI (Indicator)  → What you measure: "99.5% of requests succeeded"
SLO (Objective)  → Target for SLI:   "99.9% success rate over 30 days"
SLA (Agreement)  → Contract with penalties: "99.9% or credit"
```

## Common SLIs

| Service Type | SLI | Calculation |
|-------------|-----|-------------|
| Request-driven | Availability | successful_requests / total_requests |
| Request-driven | Latency | requests < threshold / total |
| Pipeline | Throughput | items_processed / time |
| Storage | Durability | 1 - (lost_objects / total_objects) |

## Error Budget Calculation

```
SLO: 99.9% availability over 30 days
Total minutes: 43,200
Error budget: 0.1% = 43.2 minutes/month

SLO: 99.95% availability over 30 days
Error budget: 0.05% = 21.6 minutes/month
```

## Burn Rate Alerting

```yaml
# Burn rate = how fast you consume error budget
# 1.0 = budget exhausted exactly at end of period
# 2.0 = budget exhausted at halfway point
# 14.4 = budget exhausted in ~2 days (for 30-day window)

# Multi-window burn rate alerts (Google SRE book)
- alert: HighBurnRate
  expr: |
    (
      rate(http_requests_total{code=~"5.."}[1h]) > (14.4 * 0.001)
      and
      rate(http_requests_total{code=~"5.."}[5m]) > (14.4 * 0.001)
    )
  for: 2m
  labels:
    severity: critical

- alert: MediumBurnRate
  expr: |
    (
      rate(http_requests_total{code=~"5.."}[6h]) > (6 * 0.001)
      and
      rate(http_requests_total{code=~"5.."}[30m]) > (6 * 0.001)
    )
  for: 15m
  labels:
    severity: warning
```

## Burn Rate Table

| Burn Rate | Time to Exhaust Budget | Alert Window | Severity |
|-----------|----------------------|--------------|----------|
| 14.4x | ~2 hours | 5m + 1h | Critical |
| 6x | ~5 hours | 30m + 6h | Warning |
| 3x | ~10 hours | 2h + 1d | Warning |
| 1x | 30 days | N/A | Informational |

## Prometheus SLO Queries

```promql
# Error rate (5xx / total)
sum(rate(http_requests_total{code=~"5.."}[30d]))
/
sum(rate(http_requests_total[30d]))

# Error budget remaining (for 99.9% SLO)
1 - (
  sum(rate(http_requests_total{code=~"5.."}[30d]))
  / sum(rate(http_requests_total[30d]))
  / 0.001
)

# Burn rate (current consumption rate)
(
  sum(rate(http_requests_total{code=~"5.."}[1h]))
  / sum(rate(http_requests_total[1h]))
) / 0.001
```

## SLO Document Template

```yaml
service: payment-api
slo:
  - sli: request_success_rate
    target: 99.9%
    window: 30d
  - sli: latency_p99
    target: 500ms
    window: 30d
error_budget:
  policy:
    - burn_rate > 14.4x for 1h → page
    - burn_rate > 6x for 6h → ticket
    - budget < 20% → freeze deployments
```
