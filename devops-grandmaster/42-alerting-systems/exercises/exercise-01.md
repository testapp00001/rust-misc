# Exercise 01: Alert Fatigue and Meaningful Alerts

## Type: Conceptual

## Objective

Understand the root causes of alert fatigue, learn to distinguish actionable alerts from noise, and apply design principles that produce alerts engineers actually respond to.

## Background

Your company runs 200 microservices. The monitoring team configured 1,500 alert rules. On a typical night, the on-call engineer receives 40-60 pages. Most are transient spikes, self-healing issues, or informational notifications that do not require human action. After six months of this, the team has a 12-minute average response time — but only because half the pages are never acknowledged at all.

The VP of Engineering mandates a goal: reduce pages by 80% without missing any real incidents. Your job is to figure out which alerts matter and why.

## Tasks

### Task 1: Classify the Alerts

You are given the following 15 alert rules currently running in production. Classify each as **Actionable**, **Informational**, or **Noise**. For each, explain your reasoning in one sentence.

| # | Alert Name | Expression | For | Severity |
|---|-----------|------------|-----|----------|
| 1 | HighCPU | `node_cpu_usage > 50` | 1m | critical |
| 2 | HighErrorRate | `rate(http_errors[5m]) / rate(http_requests[5m]) > 0.05` | 5m | critical |
| 3 | PodRestarted | `increase(kube_pod_restart_total[1h]) > 0` | 0s | warning |
| 4 | DiskSpaceLow | `node_filesystem_avail / node_filesystem_size < 0.10` | 5m | critical |
| 5 | HighLatency | `http_request_duration_seconds{quantile="0.99"} > 0.1` | 1m | warning |
| 6 | AnyError | `http_errors_total > 0` | 0s | critical |
| 7 | InstanceDown | `up == 0` | 1m | critical |
| 8 | HighMemory | `node_memory_usage > 60` | 1m | warning |
| 9 | CertificateExpiring | `probe_ssl_earliest_cert_expiry - time() < 604800` | 0s | warning |
| 10 | HighRequestRate | `rate(http_requests[5m]) > 1000` | 1m | warning |
| 11 | DeploymentFailed | `kube_deployment_status_replicas_available < kube_deployment_spec_replicas` | 5m | critical |
| 12 | DiskIOHigh | `node_disk_io_time > 0` | 0s | warning |
| 13 | ContainerOOMKilled | `kube_pod_container_status_last_terminated_reason{reason="OOMKilled"} == 1` | 0s | critical |
| 14 | LowTraffic | `rate(http_requests[5m]) < 10` | 15m | warning |
| 15 | SLOBudgetBurn | `slo_error_budget_remaining / slo_error_budget_total < 0.1` | 5m | critical |

### Task 2: Design Principles Analysis

For each of the following scenarios, identify which alert design principle is being violated and rewrite the alert to fix it.

**Scenario A:**
```yaml
- alert: HighCPU
  expr: node_cpu_usage > 50
  for: 0s
  annotations:
    summary: "CPU is high"
```

**Scenario B:**
```yaml
- alert: ServiceDown
  expr: up == 0
  for: 1m
  annotations:
    summary: "Service is down"
```

**Scenario C:**
```yaml
- alert: HighErrorRate
  expr: rate(http_errors[5m]) > 100
  for: 5m
  labels:
    severity: critical
```

**Scenario D:**
```yaml
- alert: DiskSpaceLow
  expr: node_filesystem_avail_bytes < 10000000000
  for: 5m
  annotations:
    summary: "Low disk space"
```

### Task 3: Severity Assignment

You are managing alerts for an e-commerce platform. Assign the correct severity (critical, warning, or info) to each scenario and justify your choice.

1. The payment service returns 500 errors for 5% of requests.
2. A non-critical logging service pod is CrashLooping.
3. CPU on a batch processing node is at 85% for 20 minutes.
4. The SSL certificate for the main website expires in 3 days.
5. A canary deployment shows 2% higher error rate than baseline.
6. The primary database is unreachable.
7. Memory usage on a node is at 70%.
8. The Redis cache hit rate dropped from 95% to 80%.
9. A Kubernetes node is marked NotReady.
10. Response time P99 increased from 200ms to 350ms but is still below the 500ms SLO.

### Task 4: Alert Quality Metrics

An alerting system needs its own metrics. Design a set of PromQL queries to measure the health of your alerting system itself.

Write PromQL queries for:

1. Total number of alerts currently firing.
2. Number of alerts firing by severity.
3. Alert firing rate (alerts per hour over the last 24 hours).
4. Percentage of alerts that auto-resolve within 5 minutes (flapping alerts).

### Task 5: The Anti-Pattern Audit

Your team has been running the following alerting configuration for 6 months. Identify every anti-pattern, explain why it is harmful, and describe how you would fix it.

```yaml
groups:
  - name: catch-all
    rules:
      - alert: HighCPU
        expr: node_cpu_usage > 50
        for: 0s
        labels:
          severity: critical
        annotations:
          summary: "CPU is high on {{ $labels.instance }}"

      - alert: HighMemory
        expr: node_memory_usage > 60
        for: 0s
        labels:
          severity: critical
        annotations:
          summary: "Memory is high on {{ $labels.instance }}"

      - alert: HighDisk
        expr: node_disk_usage > 70
        for: 0s
        labels:
          severity: critical
        annotations:
          summary: "Disk is getting full on {{ $labels.instance }}"

      - alert: PodRestart
        expr: increase(kube_pod_restart_total[5m]) > 0
        for: 0s
        labels:
          severity: warning
        annotations:
          summary: "Pod {{ $labels.pod }} restarted"

      - alert: AnyLogError
        expr: rate(log_errors_total[1m]) > 0
        for: 0s
        labels:
          severity: critical
        annotations:
          summary: "Errors found in logs"

      - alert: SlowResponse
        expr: http_request_duration_seconds{quantile="0.5"} > 0.05
        for: 0s
        labels:
          severity: warning
        annotations:
          summary: "Responses are slow"

receivers:
  - name: default
    email_configs:
      - to: 'team@example.com'

route:
  receiver: 'default'
```

## Success Criteria

- [ ] You can classify alerts as Actionable, Informational, or Noise with clear reasoning
- [ ] You can identify the four key alert design principles (actionable, contextual, appropriate severity, de-flapping)
- [ ] You can assign severity levels based on business impact, not arbitrary thresholds
- [ ] You can write PromQL to measure alert quality
- [ ] You can audit an alerting configuration and identify at least 8 anti-patterns

## Hints

<details>
<summary>Hint 1: What makes an alert actionable?</summary>

An alert is actionable if the on-call engineer can take a specific action to resolve it. Ask yourself: "If I received this page at 3 AM, could I do something about it right now?" If the answer is "wait and see" or "it will fix itself," the alert is not actionable. Examples of non-actionable alerts: high traffic (not a problem), a single pod restart (self-healing), median latency slightly elevated (within normal range).

</details>

<details>
<summary>Hint 2: The four alert design principles</summary>

1. **Actionable**: Every alert must require human intervention. If the system can self-heal, do not alert.
2. **Contextual**: Every alert must include what is wrong, where, how bad, and what to do (dashboard, runbook, logs).
3. **Appropriate severity**: Severity is based on user impact, not the metric value. A 50% CPU on a batch node is not critical.
4. **De-flapping**: Use `for` clauses to avoid alerting on transient spikes. A 1-minute spike is noise; a 15-minute sustained issue is a problem.

</details>

<details>
<summary>Hint 3: Severity assignment framework</summary>

- **Critical**: Users are affected right now. Data loss is occurring. Revenue is impacted. Response: wake someone up.
- **Warning**: Users might be affected soon. The system is degraded but functional. Response: investigate within 1 hour.
- **Info**: No user impact. Something to be aware of for capacity planning or trend analysis. Response: next business day.

When in doubt, ask: "What happens if we do not respond to this for 4 hours?" If the answer is "nothing bad," it is probably info or noise.

</details>

<details>
<summary>Hint 4: Alert quality PromQL</summary>

Prometheus exposes a built-in metric called `ALERTS` that tracks all active alerts. It has labels for `alertname`, `severity`, and `alertstate` (firing or pending). Use `count()` and `sum by ()` to aggregate. For flapping detection, compare `ALERTS{alertstate="firing"}` over short vs long time windows.

</details>

<details>
<summary>Hint 5: Anti-pattern categories</summary>

Look for these categories of anti-patterns:
- Thresholds too sensitive (50% CPU is not critical)
- Missing `for` clauses (transient spikes trigger pages)
- Everything is critical (severity inflation)
- No runbook links or context in annotations
- No routing (everything goes to one email inbox)
- Alerting on non-problems (high traffic, any error at all)
- No grouping (50 separate messages for one root cause)

</details>
