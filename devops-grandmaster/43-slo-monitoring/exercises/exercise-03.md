# Exercise 03: SLO Dashboard with Error Budget Visualization

## Objective

Design a Grafana dashboard that shows SLO compliance and error budget
consumption at a glance. The dashboard should make it immediately clear
whether a service is healthy, at risk, or violating its SLO.

## Background

A good SLO dashboard answers three questions in under 10 seconds:

1. **Are we meeting the SLO right now?** (current error rate)
2. **How much budget do we have left?** (consumption vs. remaining)
3. **Are we on track to meet the SLO for the full window?** (projection)

## Instructions

### Part A -- Dashboard layout design

Design a Grafana dashboard layout with the following panels. For each panel,
specify the **panel type**, **title**, **PromQL query**, and **key display
settings**.

Target SLO: 99.9% availability over 30 days.

#### Panel 1: SLO Status (Single Stat)

A single-stat panel that shows the current SLO status with color coding:
- Green: within SLO (budget > 25% remaining)
- Yellow: at risk (budget 10-25% remaining)
- Red: in violation (budget < 10% remaining or already violated)

#### Panel 2: Error Budget Remaining (Gauge)

A gauge panel showing the percentage of error budget remaining.

#### Panel 3: Error Rate Over Time (Time Series)

A time-series graph showing:
- The 5-minute error ratio
- The maximum allowed error rate (SLO threshold line)
- A 1-hour moving average

#### Panel 4: Error Budget Burndown (Time Series)

A burndown chart showing:
- Ideal burndown line (linear from 100% to 0% over 30 days)
- Actual burndown line
- Zones: green (safe), yellow (caution), red (danger)

#### Panel 5: Burn Rate (Stat)

Current burn rate with thresholds:
- Green: burn rate < 1.0
- Yellow: burn rate 1.0 - 3.0
- Red: burn rate > 3.0

### Part B -- Write the PromQL queries

For each panel above, write the exact PromQL query that would power it.

Assume these recording rules exist from Exercise 02:

```
job:http_errors:ratio5m          -- 5-minute error ratio
job:http_requests:rate5m         -- 5-minute request rate
```

### Part C -- Grafana dashboard JSON

Write the Grafana dashboard JSON for **Panel 1 (SLO Status)** and
**Panel 2 (Error Budget Gauge)**. Include:

- The datasource configuration
- The PromQL targets
- The threshold color mappings
- The value mappings

Use this skeleton:

```json
{
  "dashboard": {
    "title": "SLO Monitoring",
    "panels": [
      {
        "type": "stat",
        "title": "SLO Status",
        "targets": [
          {
            "expr": "YOUR_PROMQL_HERE"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "thresholds": {
              "mode": "percentage",
              "steps": []
            }
          }
        }
      }
    ]
  }
}
```

### Part D -- Alert annotations

Describe how you would configure Grafana to overlay Prometheus alert events
on the error rate time-series graph so that operators can see when alerts
fired alongside the metric data.

## Success Criteria

- [ ] Panel queries use correct PromQL and return meaningful values
- [ ] Color thresholds align with SLO risk levels (green/yellow/red)
- [ ] Burndown chart correctly models ideal vs. actual consumption
- [ ] Dashboard JSON is valid and includes threshold configuration
- [ ] Alert annotation approach is described with configuration details

## Hints

<details>
<summary>Hint 1: Error budget remaining formula</summary>

```promql
# Error budget consumed (percentage of total budget)
(
  (1 - (1 - avg_over_time(job:http_errors:ratio5m[30d]) / 0.001))
  * 100
)

# Simplified: budget remaining %
100 * (1 - (avg_over_time(job:http_errors:ratio5m[30d]) / 0.001))
```

The `0.001` is (1 - 0.999), the allowed error rate for a 99.9% SLO.

</details>

<details>
<summary>Hint 2: Ideal burndown line</summary>

Use a recording rule or a Grafana math expression:

```promql
# Ideal: starts at 100, decreases linearly to 0 over 30 days
100 * (1 - (time() - (time() - 30*24*3600)) / (30*24*3600))
```

Or model it as a constant target: you should consume ~3.33% per day.

</details>

<details>
<summary>Hint 3: Burn rate in PromQL</summary>

```promql
# Burn rate = current error rate / allowed error rate
avg_over_time(job:http_errors:ratio5m[1h]) / (1 - 0.999)
```

A burn rate of 1.0 means you are consuming the budget at the sustainable
pace. A burn rate of 2.0 means you will exhaust the budget in 15 days
instead of 30.

</details>

<details>
<summary>Hint 4: Grafana alert annotations</summary>

In Grafana, go to Dashboard Settings > Annotations and add a new annotation
source using the Prometheus datasource with the query:

```promql
ALERTS{alertstate="firing", job="your-service"}
```

This overlays alert events as vertical markers on your panels.

</details>
