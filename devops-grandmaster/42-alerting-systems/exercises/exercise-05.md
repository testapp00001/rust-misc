# Exercise 05: Alerting Strategy Reducing Noise by 80%

## Type: Integration

## Objective

Audit an existing alerting system that produces excessive noise, redesign the alert rules, routing, grouping, and escalation configuration to reduce notifications by 80% while maintaining full incident coverage, and validate the improvement with a simulation.

## Prerequisites

- Completion of Exercises 01-04
- Docker and Docker Compose
- Python 3.9+
- Understanding of AlertManager, routing, inhibition, grouping, and escalation

## Part A: The Problem

### Task 1: Analyze the Current State

You inherit an alerting system with the following metrics from the last 30 days:

```
Total alerts fired:           4,800
Total notifications sent:     3,200
Pages to on-call:             1,600 (50/day average)
Alerts acknowledged:          1,200 (75%)
Alerts auto-resolved < 5min:  2,100 (66%)
Alerts escalated past L1:     80 (5%)
Actual incidents requiring action: 40
False positive rate:          97.5%
On-call satisfaction score:   2.1 / 10
```

The current configuration:

```yaml
# Current alert rules (30 rules, showing problematic ones)
groups:
  - name: current-alerts
    rules:
      - alert: HighCPU
        expr: 100 - (avg by (instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100) > 50
        for: 1m
        labels:
          severity: critical

      - alert: HighMemory
        expr: (1 - node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes) * 100 > 60
        for: 1m
        labels:
          severity: critical

      - alert: HighLatency
        expr: histogram_quantile(0.99, sum by (le, service) (rate(http_request_duration_seconds_bucket[5m]))) > 0.5
        for: 1m
        labels:
          severity: warning

      - alert: ErrorRate
        expr: sum by (service, instance) (rate(http_requests_total{status=~"5.."}[1m])) > 0
        for: 0s
        labels:
          severity: critical

      - alert: PodRestart
        expr: increase(kube_pod_restart_total[10m]) > 0
        for: 0s
        labels:
          severity: warning

      - alert: DiskUsage
        expr: (node_filesystem_used_bytes / node_filesystem_size_bytes) * 100 > 50
        for: 0s
        labels:
          severity: critical

      - alert: NetworkErrors
        expr: rate(node_network_transmit_errs_total[1m]) > 0
        for: 0s
        labels:
          severity: warning

      - alert: ContainerCPU
        expr: sum by (pod, container) (rate(container_cpu_usage_seconds_total[1m])) > 0.1
        for: 0s
        labels:
          severity: warning

      - alert: LogErrors
        expr: rate(log_messages_total{level="error"}[1m]) > 0
        for: 0s
        labels:
          severity: critical

      - alert: SlowQuery
        expr: histogram_quantile(0.95, sum by (le, query) (rate(db_query_duration_seconds_bucket[5m]))) > 0.1
        for: 1m
        labels:
          severity: warning
```

```yaml
# Current AlertManager configuration
route:
  receiver: 'default-email'
  group_by: ['alertname']
  group_wait: 10s
  group_interval: 1m
  repeat_interval: 30m

receivers:
  - name: 'default-email'
    email_configs:
      - to: 'oncall@example.com'
```

Identify every problem with this configuration. For each problem, explain:
- What is wrong
- How it contributes to noise
- What the fix is

### Task 2: Categorize the Problems

Group the problems you identified into these categories:

1. **Threshold too sensitive**: Alerts that fire on normal operating conditions
2. **Missing `for` duration**: Alerts that fire on transient spikes
3. **Wrong severity**: Alerts marked critical that are not critical
4. **Missing context**: Alerts without runbooks, dashboards, or descriptions
5. **No routing**: All alerts going to one receiver
6. **No inhibition**: Lower-severity alerts firing when higher-severity is active
7. **Poor grouping**: Too many or too few notifications per group
8. **Missing deduplication logic**: Duplicate or near-duplicate alerts

## Part B: The Solution

### Task 3: Redesign Alert Rules

Rewrite the alert rules to fix all identified problems. For each alert:

1. Adjust the threshold to only fire on genuinely abnormal conditions
2. Add appropriate `for` durations
3. Set the correct severity based on user impact
4. Add annotations with `summary`, `description`, `dashboard_url`, `runbook_url`
5. Add appropriate labels for routing (`service`, `category`, `team`)
6. Remove any alerts that should not exist at all
7. Consolidate duplicate or near-duplicate alerts

Your redesigned rules should reduce the total number of alert rules from 30 to approximately 12-15 high-quality rules.

### Task 4: Redesign AlertManager Configuration

Write a complete AlertManager configuration that:

1. Routes critical alerts to a PagerDuty webhook AND Slack critical channel
2. Routes warning alerts to Slack warning channel only
3. Routes database alerts to the DBA team
4. Routes infrastructure alerts to the platform team
5. Implements inhibition rules:
   - Critical suppresses warning for the same service
   - InstanceDown suppresses all other alerts for that instance
   - ServiceDown suppresses latency and error rate alerts for that service
6. Groups alerts by `['alertname', 'cluster', 'service']`
7. Sets `group_wait: 30s`, `group_interval: 5m`, `repeat_interval: 4h`

### Task 5: Design Escalation Policies

Create escalation policies for three categories:

1. **Application Critical** (e.g., HighErrorRate on payment service)
2. **Infrastructure Critical** (e.g., NodeDown, DiskSpaceLow)
3. **Database Critical** (e.g., ReplicationLag, ConnectionPoolExhausted)

For each, define 4 escalation levels with appropriate contacts, methods, and timeouts.

### Task 6: Create the Full Stack

Build a Docker Compose stack with:

1. **Prometheus** with the redesigned alert rules
2. **AlertManager** with the new routing and inhibition configuration
3. **Node Exporter** for infrastructure metrics
4. **Webhook Receiver** that simulates PagerDuty and Slack:
   - Separate endpoints for each receiver type
   - Logs all received alerts with timestamps and receiver info
   - Tracks alert count per receiver
   - Provides `/stats` endpoint showing notification counts by receiver and severity
5. **Alert Simulator** (Python script) that generates a realistic 24-hour alert stream:
   - 30 minutes of normal operation (no alerts)
   - A brief CPU spike that resolves in 2 minutes (should NOT page)
   - A sustained error rate increase that lasts 30 minutes (SHOULD page)
   - A node going down, triggering cascading alerts (should be grouped and inhibited)
   - A database replication lag warning (should route to DBA team)
   - Scattered low-severity alerts throughout the day

## Part C: Validation

### Task 7: Run the Simulation

1. Start the stack: `docker-compose up -d --build`
2. Run the alert simulator: `python alert_simulator.py`
3. Collect the notification counts from the webhook receiver:
   ```bash
   curl http://localhost:9999/stats | python -m json.tool
   ```

### Task 8: Measure the Improvement

Calculate and present the following metrics for both the old and new configurations:

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Total alerts fired | 4,800 | ??? | ???% |
| Total notifications sent | 3,200 | ??? | ???% |
| Pages to on-call | 1,600 | ??? | ???% |
| False positive rate | 97.5% | ??? | ???% |
| Alerts per on-call per day | 50 | ??? | ???% |
| Incidents missed | 0 | ??? | ??? |

Target: 80% reduction in notifications with zero missed incidents.

### Task 9: Write an Incident Playbook

Write a runbook for the most common alert (HighErrorRate) that includes:

1. **Symptoms**: What the on-call engineer sees
2. **Impact**: Who is affected and how badly
3. **Diagnosis steps**: Exact commands to run, dashboards to check
4. **Mitigation steps**: What to do to stop the bleeding
5. **Resolution steps**: How to fix the root cause
6. **Escalation**: When and who to escalate to

## Part D: Analysis

### Task 10: Design Principles Summary

Write a one-page "Alerting Design Principles" document for your team that covers:

1. When to alert vs when to log vs when to update a dashboard
2. How to set thresholds (relative vs absolute, static vs dynamic)
3. How to choose severity levels
4. How to test alert rules before deploying them
5. How to review and prune alert rules regularly

### Task 11: Alerting Maturity Model

Define 5 levels of alerting maturity for an organization:

| Level | Name | Characteristics |
|-------|------|----------------|
| 1 | ??? | ??? |
| 2 | ??? | ??? |
| 3 | ??? | ??? |
| 4 | ??? | ??? |
| 5 | ??? | ??? |

Place your company's alerting system before and after this exercise on the maturity model.

## Success Criteria

- [ ] You can audit an alerting configuration and identify noise sources
- [ ] You can redesign alert rules with appropriate thresholds, durations, severity, and context
- [ ] You can configure AlertManager routing, inhibition, and grouping to reduce noise
- [ ] The redesigned system produces at least 80% fewer notifications than the original
- [ ] Zero real incidents are missed by the redesigned alert rules
- [ ] You can write a complete incident runbook
- [ ] You can articulate alerting design principles for a team

## Hints

<details>
<summary>Hint 1: Identifying noise sources</summary>

Look for these patterns in the current configuration:
- Thresholds at 50% or 60% (normal operating range for most systems)
- `for: 0s` (fires instantly on any spike)
- All alerts marked `severity: critical` (severity inflation)
- No runbook or dashboard annotations (causes slow response)
- All alerts going to one email (no routing, no escalation)
- `group_interval: 1m` (too frequent, batches too small)
- `repeat_interval: 30m` (too frequent for resolved-and-refired alerts)
- Alerting on any error at all (`> 0`)

</details>

<details>
<summary>Hint 2: Threshold design</summary>

Good thresholds are based on:
1. **Historical data**: What is the P99 of this metric over the last 30 days? Alert above that.
2. **SLO targets**: If your SLO is 99.9% success rate, alert when error rate exceeds 0.1%.
3. **Relative thresholds**: Instead of "CPU > 80%", use "CPU > 2x the 7-day average."
4. **Multi-window burn rates**: Alert when the error budget is being consumed too fast (see Module 43).

Bad thresholds:
- Round numbers (50%, 80%, 90%) chosen without data
- Same threshold for all instances (a batch node should have different CPU thresholds than a web server)
- Absolute values without context (100ms latency is fine for a cache but terrible for a search API)

</details>

<details>
<summary>Hint 3: Building the alert simulator</summary>

The simulator should produce a stream of alerts that mimics real-world patterns:

```python
def generate_alert_stream():
    alerts = []

    # Normal operation: scattered low-severity alerts
    for hour in range(24):
        for minute in range(0, 60, 15):
            t = hour * 3600 + minute * 60
            # Occasional warning alerts (should be suppressed by routing)
            if random.random() < 0.1:
                alerts.append({
                    "labels": {"alertname": "HighMemory", "severity": "warning", "service": "api", "instance": f"pod-{random.randint(1,5)}"},
                    "time": t
                })

    # CPU spike at hour 2 (resolves in 2 minutes - should NOT page)
    for i in range(3):
        alerts.append({
            "labels": {"alertname": "HighCPU", "severity": "warning", "instance": f"node-{i}"},
            "time": 7200 + i * 10
        })
    # Resolve
    for i in range(3):
        alerts.append({
            "labels": {"alertname": "HighCPU", "severity": "warning", "instance": f"node-{i}"},
            "time": 7320 + i * 10,
            "status": "resolved"
        })

    # Sustained error rate at hour 6 (SHOULD page)
    for i in range(30):
        alerts.append({
            "labels": {"alertname": "HighErrorRate", "severity": "critical", "service": "payment"},
            "time": 21600 + i * 60
        })

    return sorted(alerts, key=lambda a: a['time'])
```

</details>

<details>
<summary>Hint 4: Measuring improvement</summary>

Run the simulation twice: once with the old configuration and once with the new. Count notifications from the webhook receiver for each run.

Old configuration metrics can be estimated from the problem statement (4,800 alerts, 3,200 notifications, 1,600 pages). New configuration metrics come from the simulation.

The 80% target means:
- 3,200 notifications should drop to approximately 640
- 1,600 pages should drop to approximately 320
- 0 incidents should be missed

If you are not hitting 80%, look for additional grouping opportunities or alerts that can be demoted from paging to Slack-only.

</details>

<details>
<summary>Hint 5: Testing alert rules before deployment</summary>

Strategies for testing alert rules:

1. **Prometheus unit tests**: Write test cases in YAML that specify input metrics and expected alert states.
2. **Recording rules**: Deploy the new rule as a recording rule first. Compare the recording rule output with the old alert to see how often it would have fired.
3. **Shadow mode**: Deploy the new alert with a `for` duration of 24 hours. It will only fire if the condition persists for a full day, which is unlikely for false positives.
4. **Canary alerts**: Deploy the new alert alongside the old one. Route the new alert to a separate channel. Compare the two for a week before switching.

</details>
