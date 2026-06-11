# Exercise 02: Burn Rate Alerts in Prometheus

## Objective

Implement burn rate alerting in Prometheus using PromQL. Burn rate alerts fire
when your error budget is being consumed faster than the sustainable rate,
giving you time to respond before the SLO is violated.

## Background

A **burn rate** describes how fast you are consuming your error budget relative
to the sustainable rate. A burn rate of 1.0 means you are consuming the budget
at exactly the sustainable pace. A burn rate of 2.0 means you are consuming it
twice as fast -- you will exhaust the budget in half the window.

Burn rate alerting is superior to simple threshold alerting because it accounts
for the budget, not just the instantaneous error rate.

## Instructions

### Part A -- Prometheus recording rules for SLO components

You have an HTTP service with the following metric:

```
http_requests_total{method="GET", status="200|201|204|301|302|304|404"}
http_requests_total{method="GET", status="500|502|503|504"}
```

Your SLO is **99.9% availability** over a **30-day** window, measured by the
ratio of non-5xx responses to total responses.

Write Prometheus recording rules that compute:

1. **Total request rate** over a 5-minute window
2. **Error request rate** over a 5-minute window
3. **Error ratio** (errors / total) over a 5-minute window
4. **Error budget remaining** as a percentage (how much of the 0.1% budget is left)

Put your recording rules in a YAML file format:

```yaml
groups:
  - name: slo_recording_rules
    rules:
      # Your rules here
```

### Part B -- Burn rate alert

Write a Prometheus alerting rule that fires when:

- The **burn rate over 1 hour** exceeds **14.4x** the sustainable rate
- This corresponds to consuming 2% of the budget in 1 hour (enough to
  exhaust a 30-day budget in ~20 hours)
- The alert should be labeled with severity "critical"

```yaml
groups:
  - name: slo_alerts
    rules:
      # Your alert here
```

### Part C -- Multi-burn-rate basic alert

Extend your alert to also include a **6-hour window** check. The alert should
fire when BOTH conditions are true:

- The 1-hour burn rate exceeds **14.4x**
- The 6-hour burn rate exceeds **6x**

This reduces false positives by requiring sustained fast burning, not just a
short spike.

Write the complete alerting rule.

### Part D -- Error budget consumption alert

Write an alert that fires when the **total error budget consumed** over the
full 30-day window exceeds **50%**. This is a slow-burn alert that catches
gradual degradation.

The alert should:

- Use a 30-day lookback window
- Fire at the "warning" severity
- Include an annotation with the approximate remaining budget in minutes

## Success Criteria

- [ ] Recording rules use correct PromQL syntax for `rate()` and `histogram_quantile()`
- [ ] Error ratio calculation accounts for division-by-zero with `or vector(0)`
- [ ] Burn rate alerts use the correct ratio of `error_ratio / (1 - SLO_target)`
- [ ] Multi-burn-rate alert uses `and` to combine conditions
- [ ] Alert annotations human-readable (include budget info)

## Hints

<details>
<summary>Hint 1: Error ratio in PromQL</summary>

```promql
# Error ratio
sum(rate(http_requests_total{status=~"5.."}[5m]))
/
sum(rate(http_requests_total[5m]))
```

Handle the case where denominator is 0 by wrapping with `or vector(0)`.

</details>

<details>
<summary>Hint 2: Burn rate formula</summary>

Burn rate = Error ratio / (1 - SLO target)

For a 99.9% SLO:
- Allowed error rate = 0.001 (0.1%)
- Burn rate = error_ratio / 0.001

A burn rate of 14.4 means the error ratio is 0.0144 (1.44%).

</details>

<details>
<summary>Hint 3: Recording rule naming</summary>

Follow the Prometheus naming convention:
- `job:http_requests:rate5m` for request rates
- `job:http_errors:rate5m` for error rates
- `job:http_errors:ratio5m` for error ratios

The pattern is `level:metric:operations`.

</details>

<details>
<summary>Hint 4: Alert `for` duration</summary>

Use the `for` clause to require the condition to persist before firing:

- 1h burn rate alert: `for: 2m` (fast detection)
- 30d budget alert: `for: 1h` (slow, stable signal)

</details>
