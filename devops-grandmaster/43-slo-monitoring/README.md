# Module 43: SLO Monitoring -- Error Budgets, Burn Rate, Alerting on SLOs

> **Previous Module:** [42 -- Alerting Systems](../42-alerting-systems/README.md)
> **Next Module:** [44 -- Dashboard Design](../44-dashboard-design/README.md)
> **Phase:** 5 -- Observability

---

## 1. The Problem

Your team gets paged 30 times a month. Most alerts are for brief latency spikes that resolve themselves, CPU hitting 85% for 10 minutes, or a single pod restarting. Engineers have learned to ignore alerts because most of them are noise. Then one day, the payment service has a 30-minute outage and nobody responds -- they assumed it was another false alarm.

The fundamental problem with threshold-based alerting is that it does not distinguish between "a metric crossed a line" and "your users are being harmed." A 1% error rate might be fine for a batch processing job but catastrophic for a payment API. An arbitrary threshold like "alert when error rate > 5%" has no connection to what your users expect or what your business needs.

You need a system that alerts based on **impact to your reliability promises**, not on arbitrary metric thresholds. That system is SLO-based monitoring with error budgets and burn rate alerting.

SLOs (Service Level Objectives) define what "good" looks like for your users. Error budgets define how much "bad" you can tolerate. Burn rate alerting tells you when you are consuming your error budget too fast. Together, they create alerts that fire when your reliability is genuinely at risk -- not when a metric happens to cross a line.

---

## 2. The Naive Way

### Anti-Pattern: Static Threshold Alerts

```yaml
# Alert when error rate exceeds a fixed percentage
- alert: HighErrorRate
  expr: error_rate > 0.05
  for: 5m
```

Problems:
- 5% error rate for 5 minutes might be fine if your SLO is 99% (you have 432 minutes of error budget per month)
- 0.5% error rate sustained for 2 days might exhaust your entire monthly error budget but never trigger this alert
- The threshold is arbitrary -- not tied to user experience or business impact

### Anti-Pattern: Counting Downtime Manually

```python
# Tracking "uptime" by counting minutes of downtime
# "We were down for 43 minutes last month, that's 99.9% uptime!"
# But this ignores:
# - Partial failures (50% of requests fail)
# - Degraded performance (p99 latency went from 100ms to 10s)
# - Which users were affected (100% failure for 10% of users)
```

### Anti-Pattern: SLOs Without Error Budgets

```yaml
# Defining an SLO but not tracking the budget
slo:
  availability: 99.9%
  latency_p99: 500ms

# "We'll try to stay above 99.9%"
# But without tracking the budget, you don't know:
# - How much room you have to take risks
# - Whether to prioritize reliability work or feature work
# - When to freeze deployments due to reliability concerns
```

---

## 3. The Right Way

### Review: SLIs, SLOs, SLAs

These concepts build on Module 22 (Reliability Engineering). Here is the recap as it applies to monitoring:

**SLI (Service Level Indicator)** -- a quantitative measure of a service's behavior.

```promql
# Availability SLI: proportion of successful requests
# Good: requests with non-5xx status
# Total: all requests
availability_sli =
  sum(rate(http_requests_total{status!~"5.."}[30d]))
  /
  sum(rate(http_requests_total[30d]))

# Latency SLI: proportion of requests faster than threshold
latency_sli =
  sum(rate(http_request_duration_seconds_bucket{le="0.5"}[30d]))
  /
  sum(rate(http_request_duration_seconds_count[30d]))

# Throughput SLI: proportion of time the service accepts requests
throughput_sli =
  count_over_time(up{service="api"}[30d])
  /
  count_over_time(up[30d])
```

**SLO (Service Level Objective)** -- the target value for an SLI.

```
availability_slo: 99.9%   (allows 0.1% failed requests)
latency_slo_p99: 500ms    (99% of requests must be under 500ms)
```

**SLA (Service Level Agreement)** -- the contractual obligation, which should be less strict than the SLO.

```
availability_sla: 99.5%   (contractual; SLO is internal and more strict)
```

```
  SLA (external promise)     SLO (internal target)     Actual performance
  ----------------------------------------------------------->
  99.5%                      99.9%                     99.95%

  ^ Contract violation       ^ Error budget risk        ^ Healthy
```

### Error Budgets

An error budget is the **inverse of the SLO** -- it represents how much unreliability you can tolerate.

```
SLO: 99.9% availability
Error Budget: 0.1% = 0.001

Over a 30-day month:
  Total minutes: 30 * 24 * 60 = 43,200
  Error budget: 43,200 * 0.001 = 43.2 minutes of downtime

Or in terms of requests:
  Total requests: 10,000,000
  Error budget: 10,000,000 * 0.001 = 10,000 failed requests allowed
```

**Error budget is a shared resource** between the development team and the SRE/reliability team:

```
Error Budget Status    Action
-----------------------------------------
100% remaining         Aggressive feature releases OK
75% remaining          Normal pace, monitor closely
50% remaining          Slow down, review recent changes
25% remaining          Reliability work prioritized
10% remaining          Feature freeze, reliability focus
0% remaining           Incident mode, no deploys until recovered
```

### Calculating Error Budget in Prometheus

```promql
# Error budget for a 30-day window
# SLO: 99.9% availability

# Step 1: SLI -- actual success rate over 30 days
# Using recording rules for performance

# Recording rules (add to prometheus):
# - record: sli:http_requests:ratio_rate30d
#   expr: |
#     sum(rate(http_requests_total{status!~"5.."}[30d]))
#     /
#     sum(rate(http_requests_total[30d]))

# Step 2: Error rate
# - record: sli:http_errors:ratio_rate30d
#   expr: |
#     1 - sli:http_requests:ratio_rate30d

# Step 3: Error budget remaining (as percentage)
# (SLO_error_budget - consumed_error_budget) / SLO_error_budget
# SLO = 99.9%, so allowed error = 0.1% = 0.001
(0.001 - (1 - sli:http_requests:ratio_rate30d)) / 0.001 * 100

# If this is positive, you have budget remaining
# If this is zero or negative, you have exhausted your budget
```

**Practical implementation with Prometheus recording rules:**

```yaml
groups:
  - name: sli-recording-rules
    interval: 1m
    rules:
      # Request success rate (5-minute window for fast queries)
      - record: sli:http_requests:ratio_rate5m
        expr: |
          sum(rate(http_requests_total{status!~"5.."}[5m]))
          /
          sum(rate(http_requests_total[5m]))

      # Request success rate (30-day window for SLO calculation)
      - record: sli:http_requests:ratio_rate30d
        expr: |
          sum(rate(http_requests_total{status!~"5.."}[30d]))
          /
          sum(rate(http_requests_total[30d]))

      # Error budget consumed (as ratio of total budget)
      # SLO = 99.9%, allowed error = 0.001
      - record: sli:error_budget:consumed
        expr: |
          (1 - sli:http_requests:ratio_rate30d) / 0.001

      # Error budget remaining (percentage)
      - record: sli:error_budget:remaining_percent
        expr: |
          clamp_max(
            (0.001 - (1 - sli:http_requests:ratio_rate30d)) / 0.001 * 100,
            100
          )

      # Error budget consumed (absolute count of bad events)
      - record: sli:error_budget:bad_events
        expr: |
          sum(increase(http_requests_total{status=~"5.."}[30d]))

      # Error budget total (allowed bad requests)
      - record: sli:error_budget:total_events
        expr: |
          sum(increase(http_requests_total[30d])) * 0.001
```

### Burn Rate

Burn rate measures **how fast you are consuming your error budget** relative to the sustainable rate.

```
Burn Rate = Current Error Rate / Allowed Error Rate

If SLO = 99.9% (allowed error = 0.1%):
  - Burn rate of 1.0 = consuming budget at exactly the sustainable rate
    (will exhaust budget exactly at the end of the 30-day window)
  - Burn rate of 2.0 = consuming budget 2x too fast
    (will exhaust budget in 15 days)
  - Burn rate of 10.0 = consuming budget 10x too fast
    (will exhaust budget in 3 days)
  - Burn rate of 60.0 = consuming budget 60x too fast
    (will exhaust budget in 12 hours)
```

```promql
# Burn rate calculation
# Current error rate over the last 1 hour
current_error_rate = 1 - (
  sum(rate(http_requests_total{status!~"5.."}[1h]))
  /
  sum(rate(http_requests_total[1h]))
)

# Allowed error rate (from SLO)
allowed_error_rate = 0.001  # 99.9% SLO -> 0.1% allowed error

# Burn rate
burn_rate = current_error_rate / allowed_error_rate

# Simplified PromQL
(1 - sum(rate(http_requests_total{status!~"5.."}[1h])) / sum(rate(http_requests_total[1h]))) / 0.001
```

### Burn Rate Alerting (Single Window)

```yaml
groups:
  - name: slo-alerts
    rules:
      # Alert if burning error budget 14.4x faster than sustainable
      # At 14.4x burn rate, budget exhausts in ~2 days (for 30-day window)
      # With 1h evaluation window, this catches ~5% of monthly budget consumed in 1h
      - alert: SLOBurnRateHigh
        expr: |
          (1 - sli:http_requests:ratio_rate5m) / 0.001 > 14.4
        for: 1h
        labels:
          severity: critical
        annotations:
          summary: "Error budget burn rate is {{ $value }}x (threshold: 14.4x)"
          description: "At this rate, monthly error budget will be exhausted in ~2 days"
```

The problem with single-window burn rate alerting:

- **Short window (5m):** Catches fast incidents but has high false positive rate from brief spikes
- **Long window (1h):** Lower false positives but slow to detect issues
- **You need both:** A fast-burning incident AND a sustained issue

### Multi-Window Multi-Burn-Rate Alerts

The Google SRE approach uses multiple windows and burn rate thresholds to catch both fast and slow burns with low false positive rates.

```
Burn Rate    Short Window    Long Window    Budget Consumed    Alert Delay
---------------------------------------------------------------------------
14.4x        5 minutes       1 hour         5%                 ~5 min
 6.0x        30 minutes      6 hours        5%                 ~30 min
 3.0x        2 hours         1 day          10%                ~2 hours
 1.0x        6 hours         3 days         10%                ~6 hours
```

**How it works:**

The alert fires when BOTH conditions are true:
1. The short window burn rate exceeds the threshold (fast detection)
2. The long window burn rate also exceeds the threshold (sustained, not just a spike)

This dramatically reduces false positives while maintaining fast detection.

```yaml
groups:
  - name: slo-multi-window-alerts
    rules:
      # =====================================================
      # CRITICAL: 14.4x burn rate -> budget exhausted in ~2 days
      # Short window: 5 minutes (catches fast incidents)
      # Long window: 1 hour (confirms it's sustained)
      # Budget consumed: 2% in 5 minutes
      # =====================================================
      - alert: SLOBurnRateCritical
        expr: |
          (
            # Short window: 14.4x burn rate over 5 minutes
            (1 - sum(rate(http_requests_total{status!~"5.."}[5m])) / sum(rate(http_requests_total[5m]))) / 0.001 > 14.4
          and
            # Long window: 14.4x burn rate over 1 hour
            (1 - sum(rate(http_requests_total{status!~"5.."}[1h])) / sum(rate(http_requests_total[1h]))) / 0.001 > 14.4
          )
        for: 2m
        labels:
          severity: critical
          slo: availability
        annotations:
          summary: "Error budget burning at {{ $value }}x rate"
          description: "5% of monthly error budget consumed in 5 minutes. At this rate, budget exhausts in ~2 days."
          runbook_url: "https://runbooks.example.com/slo/burn-rate-critical"

      # =====================================================
      # CRITICAL: 6x burn rate -> budget exhausted in ~5 days
      # Short window: 30 minutes
      # Long window: 6 hours
      # =====================================================
      - alert: SLOBurnRateHigh
        expr: |
          (
            (1 - sum(rate(http_requests_total{status!~"5.."}[30m])) / sum(rate(http_requests_total[30m]))) / 0.001 > 6
          and
            (1 - sum(rate(http_requests_total{status!~"5.."}[6h])) / sum(rate(http_requests_total[6h]))) / 0.001 > 6
          )
        for: 5m
        labels:
          severity: critical
          slo: availability
        annotations:
          summary: "Error budget burning at {{ $value }}x rate (sustained)"
          description: "5% of monthly error budget consumed in 30 minutes."

      # =====================================================
      # WARNING: 3x burn rate -> budget exhausted in ~10 days
      # Short window: 2 hours
      # Long window: 1 day
      # =====================================================
      - alert: SLOBurnRateElevated
        expr: |
          (
            (1 - sum(rate(http_requests_total{status!~"5.."}[2h])) / sum(rate(http_requests_total[2h]))) / 0.001 > 3
          and
            (1 - sum(rate(http_requests_total{status!~"5.."}[1d])) / sum(rate(http_requests_total[1d]))) / 0.001 > 3
          )
        for: 15m
        labels:
          severity: warning
          slo: availability
        annotations:
          summary: "Error budget burning at {{ $value }}x rate (1-day sustained)"
          description: "10% of monthly error budget consumed in 2 hours."

      # =====================================================
      # WARNING: 1x burn rate -> budget will exhaust on schedule
      # Short window: 6 hours
      # Long window: 3 days
      # =====================================================
      - alert: SLOBurnRateWatch
        expr: |
          (
            (1 - sum(rate(http_requests_total{status!~"5.."}[6h])) / sum(rate(http_requests_total[6h]))) / 0.001 > 1
          and
            (1 - sum(rate(http_requests_total{status!~"5.."}[3d])) / sum(rate(http_requests_total[3d]))) / 0.001 > 1
          )
        for: 30m
        labels:
          severity: warning
          slo: availability
        annotations:
          summary: "Error budget depleting at sustainable burn rate"
          description: "At current pace, error budget will be fully consumed by end of window."
```

### SLO Dashboards

```promql
# SLO Dashboard Panels

# 1. Current SLI (big number)
1 - sum(rate(http_requests_total{status=~"5.."}[30d])) / sum(rate(http_requests_total[30d]))

# 2. Error budget remaining (%)
clamp_max(
  (0.001 - (1 - sum(rate(http_requests_total{status!~"5.."}[30d])) / sum(rate(http_requests_total[30d])))) / 0.001 * 100,
  100
)

# 3. Error budget consumed (time series -- should stay under 100%)
(1 - sum(rate(http_requests_total{status!~"5.."}[30d])) / sum(rate(http_requests_total[30d]))) / 0.001

# 4. Current burn rate (1h, 5m)
(1 - sum(rate(http_requests_total{status!~"5.."}[1h])) / sum(rate(http_requests_total[1h]))) / 0.001

# 5. Error budget burndown chart
# Compare actual bad events vs allowed bad events
# Actual bad events in the window
sum(increase(http_requests_total{status=~"5.."}[30d]))
# Allowed bad events (budget)
sum(increase(http_requests_total[30d])) * 0.001

# 6. SLI by endpoint (table)
sort_desc(
  1 - sum by (endpoint)(rate(http_requests_total{status=~"5.."}[30d]))
  / sum by (endpoint)(rate(http_requests_total[30d]))
)

# 7. Time to budget exhaustion (estimate)
# Days remaining = error_budget_remaining / current_daily_consumption
(
  sum(increase(http_requests_total[30d])) * 0.001
  - sum(increase(http_requests_total{status=~"5.."}[30d]))
)
/ (
  sum(increase(http_requests_total{status=~"5.."}[1d]))
)
```

### SLO-Based Incident Management

```
Error Budget Status          Deployment Policy         Incident Response
------------------------------------------------------------------------
> 75% remaining              Normal deployments        Standard process
50-75% remaining             Extra review required      Standard process
25-50% remaining             Reliability review needed  Priority response
10-25% remaining             Feature freeze             Immediate response
< 10% remaining              No deployments             All hands
  0% (exhausted)             Emergency only             War room
```

### Multiple SLOs Per Service

A single service typically has multiple SLOs:

```yaml
slos:
  - name: availability
    sli: |
      sum(rate(http_requests_total{status!~"5.."}[30d]))
      / sum(rate(http_requests_total[30d]))
    target: 0.999    # 99.9%

  - name: latency
    sli: |
      sum(rate(http_request_duration_seconds_bucket{le="0.5"}[30d]))
      / sum(rate(http_request_duration_seconds_count[30d]))
    target: 0.99     # 99% of requests under 500ms

  - name: freshness
    sli: |
      count_over_time(data_freshness_seconds < 60[30d])
      / count_over_time(data_freshness_seconds[30d])
    target: 0.999    # Data is fresh (<60s old) 99.9% of the time

  - name: correctness
    sli: |
      sum(rate(search_results_correct_total[30d]))
      / sum(rate(search_results_total[30d]))
    target: 0.995    # 99.5% of search results are correct
```

---

## 4. The Production Way

### Choosing the Right SLO Window

| Window | Use Case | Pros | Cons |
|--------|----------|------|------|
| 28 days | Rolling monthly | Aligns with calendar, resets regularly | Lumpy data at boundaries |
| 30 days | Rolling | Smooth, consistent | Does not align with calendar |
| Calendar month | Reporting | Easy to reason about | Boundary effects, inconsistent lengths |
| Quarter | Long-term trends | Smooths seasonal variation | Slow to detect problems |

**Recommendation:** Use a rolling 28-day window for alerting (consistent data), and calendar month for reporting (easy to communicate).

### Handling SLO Violations

```
1. Error budget exhausted
   -> Freeze feature deployments
   -> Notify engineering leadership
   -> Begin reliability sprint

2. Identify root cause
   -> Which deploys/changes correlate with budget consumption?
   -> Which endpoints/features are consuming the budget?
   -> Are there infrastructure issues?

3. Reliability work
   -> Fix the highest-impact issues
   -> Add redundancy where needed
   -> Improve error handling and graceful degradation

4. Recover budget
   -> Run for N days without incidents
   -> Error budget slowly recovers over the rolling window
   -> Resume feature work when budget is back above threshold
```

### SLO Review Cadence

```
Weekly:
  - Review error budget status
  - Check burn rate trends
  - Adjust if approaching budget exhaustion

Monthly:
  - SLO retrospective: did we meet targets?
  - Review false positive/negative alerts
  - Adjust SLO targets if needed

Quarterly:
  - Are SLOs aligned with user expectations?
  - Are error budgets enabling the right tradeoffs?
  - Do we need new SLIs or SLOs?
```

### Error Budget Policy

```markdown
# Error Budget Policy

## Service: Payment API
## SLO: 99.95% availability (21.6 minutes downtime/month)

### Budget Status Actions

| Budget Remaining | Deployment Policy | Review Required |
|-----------------|-------------------|-----------------|
| > 50%           | Normal pace       | None            |
| 25-50%          | Extra SRE review  | Change ticket   |
| 10-25%          | Reliability focus | Team lead       |
| < 10%           | Emergency only    | Director        |
| 0%              | Freeze all deploys| VP Engineering  |

### Budget Recovery
- Budget resets on rolling 30-day basis
- No manual resets
- Use the recovery period to address technical debt

### Exception Process
- Critical security patches are exempt
- Business-critical features require VP approval
- All exceptions are documented in post-mortem
```

### SLO as Code (Sloth / OpenSLO)

```yaml
# sloth-slo.yaml -- using Sloth format
sloth/prometheus/v1:
  service: "api-server"
  labels:
    team: platform
  slos:
    - name: availability
      objective: 99.9
      description: "Proportion of successful requests"
      sli:
        events:
          error_query: sum(rate(http_requests_total{service="api-server", status=~"5.."}[{{.window}}]))
          total_query: sum(rate(http_requests_total{service="api-server"}[{{.window}}]))
      alerting:
        name: "HighErrorBurnRate"
        labels:
          severity: critical
        annotations:
          runbook_url: "https://wiki.company.com/runbooks/slo-burn"

    - name: latency
      objective: 99.0
      description: "Proportion of requests under 300ms"
      sli:
        events:
          error_query: sum(rate(http_request_duration_seconds_bucket{service="api-server", le="0.3"}[{{.window}}]))
          total_query: sum(rate(http_request_duration_seconds_count{service="api-server"}[{{.window}}]))
```

```bash
# Generate Prometheus rules from Sloth
sloth generate -i sloth-slo.yaml -o generated-rules.yaml
kubectl apply -f generated-rules.yaml
```

---

## 5. Hands-On Lab

### Lab: Implement SLO Monitoring with Prometheus

**Objective:** Define SLIs, create recording rules, implement multi-window burn rate alerts, and build an SLO dashboard.

**Step 1: Project structure**

```bash
mkdir slo-lab && cd slo-lab
mkdir -p prometheus/rules app grafana/provisioning/datasources grafana/provisioning/dashboards
```

**Step 2: Create an application with realistic error patterns**

```python
# app/main.py
from flask import Flask, jsonify
from prometheus_client import Counter, Histogram, generate_latest, CONTENT_TYPE_LATEST
import random
import time
import os

app = Flask(__name__)

REQUEST_COUNT = Counter(
    'http_requests_total', 'Total HTTP requests',
    ['method', 'endpoint', 'status']
)
REQUEST_LATENCY = Histogram(
    'http_request_duration_seconds', 'Request latency',
    ['method', 'endpoint'],
    buckets=[0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
)

# Simulate different error rate modes
ERROR_MODE = os.getenv("ERROR_MODE", "normal")

def get_error_rate():
    if ERROR_MODE == "normal":
        return 0.001   # 0.1% error rate (well within 99.9% SLO)
    elif ERROR_MODE == "elevated":
        return 0.02    # 2% error rate (burning budget fast)
    elif ERROR_MODE == "critical":
        return 0.15    # 15% error rate (critical)
    return 0.001

@app.before_request
def before_request():
    request._start_time = time.time()

@app.after_request
def after_request(response):
    latency = time.time() - request._start_time
    REQUEST_LATENCY.labels(request.method, request.path).observe(latency)
    REQUEST_COUNT.labels(request.method, request.path, str(response.status_code)).inc()
    return response

@app.route('/api/data')
def get_data():
    time.sleep(random.uniform(0.01, 0.1))
    if random.random() < get_error_rate():
        return jsonify({"error": "Internal Server Error"}), 500
    if random.random() < 0.05:
        time.sleep(random.uniform(0.5, 2.0))
    return jsonify({"data": "ok"})

@app.route('/health')
def health():
    return jsonify({"status": "healthy"})

@app.route('/metrics')
def metrics():
    return generate_latest(), 200, {'Content-Type': 'text/plain; version=0.0.4; charset=utf-8'}

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=8080)
```

**Step 3: Create SLI recording rules**

```yaml
# prometheus/rules/sli-recording.yml
groups:
  - name: sli-recording
    interval: 30s
    rules:
      # ===========================================
      # Availability SLI
      # ===========================================

      # 5-minute success ratio (for fast alerting)
      - record: sli:http_availability:ratio_rate5m
        expr: |
          sum(rate(http_requests_total{status!~"5.."}[5m]))
          /
          sum(rate(http_requests_total[5m]))

      # 30-minute success ratio
      - record: sli:http_availability:ratio_rate30m
        expr: |
          sum(rate(http_requests_total{status!~"5.."}[30m]))
          /
          sum(rate(http_requests_total[30m]))

      # 1-hour success ratio
      - record: sli:http_availability:ratio_rate1h
        expr: |
          sum(rate(http_requests_total{status!~"5.."}[1h]))
          /
          sum(rate(http_requests_total[1h]))

      # 6-hour success ratio
      - record: sli:http_availability:ratio_rate6h
        expr: |
          sum(rate(http_requests_total{status!~"5.."}[6h]))
          /
          sum(rate(http_requests_total[6h]))

      # 1-day success ratio
      - record: sli:http_availability:ratio_rate1d
        expr: |
          sum(rate(http_requests_total{status!~"5.."}[1d]))
          /
          sum(rate(http_requests_total[1d]))

      # 3-day success ratio
      - record: sli:http_availability:ratio_rate3d
        expr: |
          sum(rate(http_requests_total{status!~"5.."}[3d]))
          /
          sum(rate(http_requests_total[3d]))

      # 30-day success ratio (for SLO calculation)
      - record: sli:http_availability:ratio_rate30d
        expr: |
          sum(rate(http_requests_total{status!~"5.."}[30d]))
          /
          sum(rate(http_requests_total[30d]))

      # ===========================================
      # Latency SLI
      # ===========================================

      # Proportion of requests under 500ms (5-minute)
      - record: sli:http_latency:ratio_rate5m
        expr: |
          sum(rate(http_request_duration_seconds_bucket{le="0.5"}[5m]))
          /
          sum(rate(http_request_duration_seconds_count[5m]))

      # Proportion of requests under 500ms (30-day)
      - record: sli:http_latency:ratio_rate30d
        expr: |
          sum(rate(http_request_duration_seconds_bucket{le="0.5"}[30d]))
          /
          sum(rate(http_request_duration_seconds_count[30d]))

      # ===========================================
      # Error Budget (SLO = 99.9% availability)
      # ===========================================

      # Error budget remaining as percentage
      - record: sli:error_budget:remaining_percent
        expr: |
          clamp_min(
            clamp_max(
              (0.001 - (1 - sli:http_availability:ratio_rate30d)) / 0.001 * 100,
              100
            ),
            0
          )

      # Error budget consumed as ratio (1.0 = fully consumed)
      - record: sli:error_budget:consumed_ratio
        expr: |
          clamp_min(
            (1 - sli:http_availability:ratio_rate30d) / 0.001,
            0
          )

      # ===========================================
      # Burn Rates
      # ===========================================

      # 5-minute burn rate
      - record: sli:burn_rate:5m
        expr: |
          (1 - sli:http_availability:ratio_rate5m) / 0.001

      # 30-minute burn rate
      - record: sli:burn_rate:30m
        expr: |
          (1 - sli:http_availability:ratio_rate30m) / 0.001

      # 1-hour burn rate
      - record: sli:burn_rate:1h
        expr: |
          (1 - sli:http_availability:ratio_rate1h) / 0.001

      # 6-hour burn rate
      - record: sli:burn_rate:6h
        expr: |
          (1 - sli:http_availability:ratio_rate6h) / 0.001

      # 1-day burn rate
      - record: sli:burn_rate:1d
        expr: |
          (1 - sli:http_availability:ratio_rate1d) / 0.001
```

**Step 4: Create multi-window burn rate alert rules**

```yaml
# prometheus/rules/slo-alerts.yml
groups:
  - name: slo-alerts
    rules:
      # =================================================
      # PAGE 1: Critical -- 14.4x burn rate
      # Budget: 5% consumed in ~5 minutes
      # Short window: 5 minutes, Long window: 1 hour
      # =================================================
      - alert: SLOBurnRateCritical
        expr: |
          sli:burn_rate:5m > 14.4
          and
          sli:burn_rate:1h > 14.4
        for: 2m
        labels:
          severity: critical
          slo: availability
        annotations:
          summary: "Critical: Error budget burning at {{ $value | printf \"%.1f\" }}x rate"
          description: |
            5% of monthly error budget consumed in 5 minutes.
            At this rate, the entire budget will be exhausted in ~2 days.
            SLO: 99.9% availability
            Dashboard: http://grafana:3000/d/slo-overview
          runbook_url: "https://runbooks.example.com/slo/burn-rate-critical"

      # =================================================
      # PAGE 2: High -- 6x burn rate
      # Budget: 5% consumed in ~30 minutes
      # Short window: 30 minutes, Long window: 6 hours
      # =================================================
      - alert: SLOBurnRateHigh
        expr: |
          sli:burn_rate:30m > 6
          and
          sli:burn_rate:6h > 6
        for: 5m
        labels:
          severity: critical
          slo: availability
        annotations:
          summary: "High: Error budget burning at {{ $value | printf \"%.1f\" }}x rate (30m sustained)"
          description: |
            5% of monthly error budget consumed in 30 minutes.
            At this rate, the entire budget will be exhausted in ~5 days.
          runbook_url: "https://runbooks.example.com/slo/burn-rate-high"

      # =================================================
      # TICKET: Elevated -- 3x burn rate
      # Budget: 10% consumed in ~2 hours
      # Short window: 2 hours, Long window: 1 day
      # =================================================
      - alert: SLOBurnRateElevated
        expr: |
          sli:burn_rate:1h > 3
          and
          sli:burn_rate:1d > 3
        for: 15m
        labels:
          severity: warning
          slo: availability
        annotations:
          summary: "Elevated: Error budget burning at {{ $value | printf \"%.1f\" }}x rate (1d sustained)"
          description: |
            10% of monthly error budget consumed in 2 hours.
            At this rate, the entire budget will be exhausted in ~10 days.
          runbook_url: "https://runbooks.example.com/slo/burn-rate-elevated"

      # =================================================
      # TICKET: Slow burn -- 1x burn rate
      # Budget will be fully consumed
      # Short window: 6 hours, Long window: 3 days
      # =================================================
      - alert: SLOBurnRateWatch
        expr: |
          sli:burn_rate:6h > 1
          and
          sli:burn_rate:3d > 1
        for: 30m
        labels:
          severity: warning
          slo: availability
        annotations:
          summary: "Watch: Error budget depleting at sustainable rate"
          description: |
            Error budget is being consumed at the sustainable rate.
            At this pace, the budget will be fully exhausted by the end of the window.
            Consider investigating recent changes.

      # =================================================
      # BUDGET EXHAUSTION
      # =================================================
      - alert: SLOErrorBudgetExhausted
        expr: |
          sli:error_budget:remaining_percent <= 0
        for: 5m
        labels:
          severity: critical
          slo: availability
        annotations:
          summary: "Error budget EXHAUSTED -- SLO is being violated"
          description: |
            The monthly error budget for availability is fully consumed.
            All feature deployments should be frozen.
            Focus must shift to reliability improvements.
          runbook_url: "https://runbooks.example.com/slo/budget-exhausted"
```

**Step 5: Docker Compose**

```yaml
# docker-compose.yml
version: '3.8'
services:
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus/prometheus.yml:/etc/prometheus/prometheus.yml
      - ./prometheus/rules:/etc/prometheus/rules
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.retention.time=30d'
      - '--web.enable-lifecycle'

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - ./grafana/provisioning:/etc/grafana/provisioning

  app-normal:
    build: ./app
    ports:
      - "8080:8080"
    environment:
      - ERROR_MODE=normal

  app-elevated:
    build: ./app
    ports:
      - "8081:8080"
    environment:
      - ERROR_MODE=elevated
```

```yaml
# prometheus/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - /etc/prometheus/rules/*.yml

scrape_configs:
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'app-normal'
    static_configs:
      - targets: ['app-normal:8080']

  - job_name: 'app-elevated'
    static_configs:
      - targets: ['app-elevated:8080']
```

**Step 6: Generate traffic and observe SLO behavior**

```bash
# Start the stack
docker-compose up -d --build

# Wait for services
sleep 15

# Generate baseline traffic for both apps
generate_traffic() {
  local url=$1
  for i in $(seq 1 5000); do
    curl -s "$url/api/data" > /dev/null 2>&1 &
    if [ $((i % 50)) -eq 0 ]; then
      wait
      sleep 0.01
    fi
  done
  wait
}

# Generate traffic to both apps
generate_traffic http://localhost:8080 &
generate_traffic http://localhost:8081 &
wait

echo "Traffic generated. Check Prometheus and Grafana."

# Open Prometheus to check alerts
open http://localhost:9090/alerts

# Open Grafana
open http://localhost:3000
```

**Step 7: Explore in Prometheus**

Try these PromQL queries:

```promql
# Current SLI for normal app
sli:http_availability:ratio_rate5m{job="app-normal"}

# Current SLI for elevated error app
sli:http_availability:ratio_rate5m{job="app-elevated"}

# Error budget remaining
sli:error_budget:remaining_percent{job="app-normal"}
sli:error_budget:remaining_percent{job="app-elevated"}

# Current burn rate
sli:burn_rate:5m{job="app-elevated"}
sli:burn_rate:1h{job="app-elevated"}

# Compare burn rates across time windows
sli:burn_rate:5m{job="app-elevated"}
sli:burn_rate:30m{job="app-elevated"}
sli:burn_rate:1h{job="app-elevated"}
```

**Step 8: Build an SLO Dashboard**

Create a Grafana dashboard with these panels:

1. **SLI Current Value** (Stat):
   - `sli:http_availability:ratio_rate30d{job="app-elevated"}`
   - Unit: percent, threshold: green > 99.9, yellow > 99.5, red < 99.5

2. **Error Budget Remaining** (Gauge):
   - `sli:error_budget:remaining_percent{job="app-elevated"}`
   - Range: 0-100, thresholds: green > 50, yellow > 25, red < 10

3. **Burn Rate** (Time Series):
   - `sli:burn_rate:5m{job="app-elevated"}`
   - `sli:burn_rate:30m{job="app-elevated"}`
   - `sli:burn_rate:1h{job="app-elevated"}`
   - Threshold line at 1.0 (sustainable rate)

4. **Error Budget Burndown** (Time Series):
   - `sli:error_budget:consumed_ratio{job="app-elevated"}`
   - Threshold line at 1.0 (exhausted)

5. **Requests by Status** (Time Series):
   - `sum by (status) (rate(http_requests_total{job="app-elevated"}[5m]))`

6. **Latency p99** (Time Series):
   - `histogram_quantile(0.99, sum by (le) (rate(http_request_duration_seconds_bucket{job="app-elevated"}[5m])))`

**Expected outcome:**
- The "normal" app maintains a burn rate near 1.0 and error budget near 100%
- The "elevated" app shows increasing burn rate and decreasing error budget
- Multi-window alerts fire for the elevated app but not the normal app
- The SLO dashboard clearly shows the health of each service's error budget

---

## 6. Limitation

SLO monitoring with error budgets and burn rate alerting gives you precision. You know exactly how much reliability budget you have, how fast you are consuming it, and when to take action. Alerts fire based on actual impact to your reliability targets, not arbitrary thresholds.

But all of this data -- metrics, traces, logs, SLOs, error budgets -- is only useful if people can understand it at a glance. A well-designed dashboard lets an engineer assess the health of a system in 5 seconds. A poorly designed dashboard is a wall of graphs that tells you nothing.

What you monitor matters, but how you visualize it matters just as much. The next step is learning dashboard design: what to show, how to organize it, and how to make dashboards that are actually useful during incidents.

**Next Module:** [44 -- Dashboard Design](../44-dashboard-design/README.md) -- Design production dashboards that communicate system health at a glance, using the four golden signals, USE/RED patterns, and Grafana best practices.
