# Exercise 04: Multi-Window Burn Rate Alerts

## Objective

Design a multi-window, multi-burn-rate alerting strategy that catches both
fast-burning incidents (outages) and slow-burning incidents (gradual
degradation) while minimizing false positives and false negatives.

## Background

The Google SRE book recommends a multi-window approach:

| Alert Window | Burn Rate | Error Budget Consumed | Use Case |
|-------------|-----------|----------------------|----------|
| 1 hour      | 14.4x     | 2% in 1h             | Fast burn: major outage |
| 6 hours     | 6x        | 5% in 6h             | Medium burn: partial outage |
| 3 days      | 1x        | 10% in 3d            | Slow burn: gradual degradation |
| 30 days     | 1x        | 100% in 30d          | SLO violation (already happened) |

Each window uses a **long window** (for alert stability) and a **short window**
(for alert precision).

## Instructions

### Part A -- Understand the math

For a **99.9% SLO** over 30 days (43,200 minutes total, 43.2 minutes error
budget):

1. Calculate the error budget consumed in each scenario:
   - 14.4x burn rate for 1 hour
   - 6x burn rate for 6 hours
   - 1x burn rate for 3 days

2. For the 1-hour fast-burn alert:
   - What is the allowed error ratio? (hint: burn_rate * (1 - SLO))
   - What does this mean in terms of 5xx responses per second for a
     service handling 1000 req/s?

### Part B -- Implement the multi-window alerts

Write complete Prometheus alerting rules for the following strategy:

**Fast burn (Page immediately):**
- Condition: 14.4x burn rate over 1h AND 14.4x burn rate over 5m
- Severity: critical
- The 5m short window provides precision, the 1h long window provides stability

**Medium burn (Page after sustained issue):**
- Condition: 6x burn rate over 6h AND 6x burn rate over 30m
- Severity: warning
- The 30m short window provides precision, the 6h long window provides stability

**Slow burn (Ticket, not page):**
- Condition: 1x burn rate over 3d AND 1x burn rate over 6h
- Severity: ticket
- The 6h short window provides precision, the 3d long window provides stability

**Budget exhausted (Already violated):**
- Condition: Error budget consumed > 100% over 30d
- Severity: critical
- No short window needed -- this is a factual state

Write these as a complete Prometheus rule group:

```yaml
groups:
  - name: slo_multi_window_alerts
    interval: 30s
    rules:
      # Your rules here
```

### Part C -- Short window and long window explanation

For each alert tier (fast, medium, slow), explain:

1. Why do we need BOTH a short window and a long window?
2. What happens if we only use the long window?
3. What happens if we only use the short window?

Use concrete examples of scenarios that would be missed or falsely triggered.

### Part D -- Tuning the sensitivity

Your team finds that the fast-burn alert is firing too often during brief
traffic spikes that are not actual incidents.

Propose two strategies to reduce false positives without increasing the
risk of missing real incidents. For each strategy:

1. Describe the change
2. Explain the tradeoff (what you gain vs. what you lose)
3. Show the modified PromQL (if applicable)

### Part E -- Recording rules for efficiency

The multi-window alerts above use many repeated `avg_over_time()` calls.
Write recording rules that pre-compute the error ratio at each required
window size, then rewrite the alerts to use the recorded values.

This reduces Prometheus query load and improves alert evaluation speed.

## Success Criteria

- [ ] Math in Part A is correct (budget consumed per scenario)
- [ ] Alerts use `and` to combine short and long windows
- [ ] PromQL uses correct window sizes (e.g., `[1h]`, `[5m]`)
- [ ] Recording rules follow Prometheus naming conventions
- [ ] Part C explanations demonstrate deep understanding of windowing tradeoffs
- [ ] Part D strategies are practical and well-justified

## Hints

<details>
<summary>Hint 1: Burn rate error ratio</summary>

For a burn rate `b` and SLO target `s`:

```
allowed_error_ratio = b * (1 - s)
```

For 14.4x burn rate and 99.9% SLO:
```
allowed_error_ratio = 14.4 * (1 - 0.999) = 14.4 * 0.001 = 0.0144
```

This means the error ratio must exceed 1.44% over the window to trigger.

</details>

<details>
<summary>Hint 2: Window size relationship</summary>

The short window should be approximately:

```
short_window = long_window / burn_rate
```

For 1h at 14.4x: short = 60/14.4 ~= 4-5 minutes
For 6h at 6x: short = 360/6 = 60 minutes
For 3d at 1x: short = 72/24 = 3 hours (use 6h for safety)

</details>

<details>
<summary>Hint 3: Alert structure pattern</summary>

```yaml
- alert: SLOBurnRateFast
  expr: |
    (
      job:http_errors:ratio5m > (14.4 * 0.001)
      and
      job:http_errors:ratio1h > (14.4 * 0.001)
    )
  for: 2m
  labels:
    severity: critical
  annotations:
    summary: "Fast burn detected"
```

</details>

<details>
<summary>Hint 4: Recording rule pattern</summary>

```yaml
- record: job:http_errors:ratio5m
  expr: |
    sum(rate(http_requests_total{status=~"5.."}[5m])) by (job)
    /
    sum(rate(http_requests_total[5m])) by (job)

- record: job:http_errors:ratio1h
  expr: |
    sum(rate(http_requests_total{status=~"5.."}[1h])) by (job)
    /
    sum(rate(http_requests_total[1h])) by (job)
```

</details>
