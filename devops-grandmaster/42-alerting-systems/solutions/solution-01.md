# Solution 01: Alert Fatigue and Meaningful Alerts

## Task 1: Alert Classification

| # | Alert Name | Classification | Reasoning |
|---|-----------|---------------|-----------|
| 1 | HighCPU | **Noise** | 50% CPU is normal for most workloads. No engineer should be paged for this. At best it is informational for capacity planning. |
| 2 | HighErrorRate | **Actionable** | 5% error rate sustained for 5 minutes means real users are being affected and someone needs to investigate. |
| 3 | PodRestarted | **Noise** | A single pod restart in an hour is expected behavior in Kubernetes (OOM kills, node drains, deployments). Alerting on every restart creates noise with no action to take. |
| 4 | DiskSpaceLow | **Actionable** | Disk below 10% will cause write failures. Someone needs to clean up or expand the volume before the disk fills completely. |
| 5 | HighLatency | **Noise** | 100ms P99 latency is perfectly acceptable for most services. This threshold is far too low. Only alert when latency breaches the SLO (e.g., > 1s for a web API). |
| 6 | AnyError | **Noise** | Any non-zero error count is normal in a distributed system. A single 500 error in a 5-minute window does not require human intervention. This alert will fire constantly. |
| 7 | InstanceDown | **Actionable** | An instance being unreachable for 1 minute means a pod or node has failed. Someone needs to investigate and potentially restart or reschedule. |
| 8 | HighMemory | **Noise** | 60% memory usage is normal. Most applications and operating systems use available memory for caching. This will fire constantly and the on-call can do nothing about it. |
| 9 | CertificateExpiring | **Actionable** | A certificate expiring in 7 days requires renewal before it causes an outage. This is time-sensitive and someone needs to act. |
| 10 | HighRequestRate | **Noise** | High traffic is not a problem unless it causes degradation. Alerting on high traffic alone is not actionable -- what would the on-call do, tell users to stop using the service? |
| 11 | DeploymentFailed | **Actionable** | Fewer available replicas than desired means a deployment is stuck or pods are crashing. Someone needs to investigate the deployment status. |
| 12 | DiskIOHigh | **Noise** | Any non-zero disk I/O is normal -- disks are supposed to do I/O. This alert will fire 100% of the time and provides no value. |
| 13 | ContainerOOMKilled | **Actionable** | An OOM kill means the container exceeded its memory limit and was forcibly terminated. The memory limit may need to be increased or the application has a memory leak. |
| 14 | LowTraffic | **Informational** | Low traffic might indicate a problem (DNS failure, upstream routing issue) or might be expected (off-peak hours). This is useful for dashboards but should not page anyone. |
| 15 | SLOBudgetBurn | **Actionable** | Less than 10% error budget remaining means the service is close to breaching its reliability target. Immediate action is needed to prevent an SLO violation. |

### Key Insight

Out of 15 alerts, only 6 are actionable (40%). The remaining 9 are either noise (60%) or informational. This is exactly the ratio that causes alert fatigue. If you only keep the actionable alerts, you reduce pages by 60% without losing any real incident coverage.

## Task 2: Design Principles Analysis

**Scenario A: Missing `for` duration + threshold too low + missing context**

Violated principles: **De-flapping** and **Contextual**

- `for: 0s` means the alert fires the instant CPU hits 50%, even for a brief spike. CPU regularly spikes to 50% during normal operations (garbage collection, cache warming, batch jobs).
- 50% CPU is not a problem. Most systems are designed to run at 60-70% average utilization.
- "CPU is high" provides no context: which instance? What dashboard? What runbook?

Fixed:
```yaml
- alert: HighCPU
  expr: 100 - (avg by (instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100) > 85
  for: 15m
  labels:
    severity: warning
    category: infrastructure
  annotations:
    summary: "CPU above 85% on {{ $labels.instance }} for 15 minutes"
    description: |
      Instance: {{ $labels.instance }}
      Current CPU: {{ $value }}%
      Threshold: 85%
      Duration: 15+ minutes
    dashboard_url: "https://grafana.example.com/d/node-overview?var-instance={{ $labels.instance }}"
    runbook_url: "https://runbooks.example.com/infra/high-cpu"
```

**Scenario B: Missing context (no runbook, no dashboard, no description)**

Violated principle: **Contextual**

- "Service is down" tells the on-call nothing about which service, which instance, or what to do about it.
- No dashboard link means the engineer wastes time finding the right Grafana page.
- No runbook link means the engineer relies on tribal knowledge.

Fixed:
```yaml
- alert: ServiceDown
  expr: up == 0
  for: 1m
  labels:
    severity: critical
  annotations:
    summary: "Instance {{ $labels.instance }} is down"
    description: |
      Instance {{ $labels.instance }} of job {{ $labels.job }} has been unreachable for 1 minute.
      Steps:
      1. Check if the pod is running: kubectl get pods -l app={{ $labels.job }}
      2. Check recent events: kubectl describe pod {{ $labels.pod }}
      3. Check logs: kubectl logs {{ $labels.pod }} --tail=100
    dashboard_url: "https://grafana.example.com/d/service-overview?var-service={{ $labels.job }}"
    runbook_url: "https://runbooks.example.com/infra/instance-down"
```

**Scenario C: Wrong metric for the alert + missing context**

Violated principles: **Actionable** and **Contextual**

- `rate(http_errors[5m]) > 100` is an absolute count, not a rate relative to total traffic. 100 errors per 5 minutes during 1,000,000 requests is a 0.01% error rate (fine). 100 errors during 200 requests is a 50% error rate (catastrophic).
- No description, no dashboard, no runbook.

Fixed:
```yaml
- alert: HighErrorRate
  expr: |
    sum(rate(http_requests_total{status=~"5.."}[5m]))
    / sum(rate(http_requests_total[5m]))
    > 0.05
  for: 5m
  labels:
    severity: critical
  annotations:
    summary: "Error rate above 5% for 5 minutes"
    description: |
      Current error rate: {{ $value | humanizePercentage }}
      Threshold: 5%
      This alert fires when 5% or more of all HTTP requests return 5xx errors.
    dashboard_url: "https://grafana.example.com/d/api-overview"
    runbook_url: "https://runbooks.example.com/api/high-error-rate"
```

**Scenario D: Absolute threshold without context + missing context**

Violated principles: **Appropriate severity** and **Contextual**

- `node_filesystem_avail_bytes < 10000000000` (10GB) is an absolute value. On a 100GB disk, 10GB remaining is 10% (worrying). On a 10TB disk, 10GB remaining is 0.1% (critical). The alert should use a percentage.
- "Low disk space" does not say which disk, which instance, or what to do.

Fixed:
```yaml
- alert: DiskSpaceLow
  expr: |
    (node_filesystem_avail_bytes{fstype!="tmpfs"} / node_filesystem_size_bytes) * 100 < 10
  for: 5m
  labels:
    severity: critical
  annotations:
    summary: "Disk space below 10% on {{ $labels.instance }} ({{ $labels.mountpoint }})"
    description: |
      Instance: {{ $labels.instance }}
      Mount: {{ $labels.mountpoint }}
      Available: {{ $value }}%
      Threshold: 10%
    dashboard_url: "https://grafana.example.com/d/node-overview?var-instance={{ $labels.instance }}"
    runbook_url: "https://runbooks.example.com/infra/low-disk-space"
```

## Task 3: Severity Assignment

| # | Scenario | Severity | Justification |
|---|---------|----------|--------------|
| 1 | Payment service 500 errors at 5% | **Critical** | Revenue-generating service, real users losing transactions. Wake someone up immediately. |
| 2 | Non-critical logging service CrashLooping | **Info** | No user impact. Logging is degraded but the system functions. Fix during business hours. |
| 3 | Batch node CPU at 85% for 20 min | **Warning** | Batch processing is slower but not failing. Investigate within 1 hour, do not page at 2 AM. |
| 4 | SSL cert expires in 3 days | **Warning** | Time-sensitive but not an emergency. 3 days is enough time to renew during business hours. If it were 3 hours, it would be critical. |
| 5 | Canary 2% higher error rate | **Warning** | Canary is by definition a small percentage of traffic. The blast radius is limited. Investigate but do not page. |
| 6 | Primary database unreachable | **Critical** | This will cascade into full outage within minutes. Wake someone up immediately. |
| 7 | Memory at 70% | **Info** | 70% memory is normal for most workloads. Monitor for trends, do not alert. |
| 8 | Redis cache hit rate dropped to 80% | **Warning** | Performance degradation (more database queries) but not an outage. Investigate within 1 hour. |
| 9 | Kubernetes node NotReady | **Critical** | Pods on this node will be evicted. If it is a significant portion of capacity, this can cause cascading failures. |
| 10 | P99 latency 350ms (SLO is 500ms) | **Info** | Still within SLO. Track the trend on a dashboard. Alert only if it approaches the SLO threshold. |

### Severity Framework

The key insight: severity is about **user impact and urgency**, not about the metric value.

- **Critical**: Users are affected NOW. Revenue or data is at risk. Response time: immediate.
- **Warning**: Users might be affected SOON. System is degraded. Response time: 1 hour.
- **Info**: No user impact. Trend awareness. Response time: next business day.

## Task 4: Alert Quality Metrics

```promql
# 1. Total number of alerts currently firing
count(ALERTS{alertstate="firing"})

# 2. Number of alerts firing by severity
sum by (severity) (ALERTS{alertstate="firing"})

# 3. Alert firing rate (alerts per hour over the last 24 hours)
sum(increase(ALERTS{alertstate="firing"}[24h])) / 24

# 4. Percentage of alerts that auto-resolve within 5 minutes (flapping alerts)
# An alert that fires and resolves within 5 minutes is flapping.
# Count alerts that have both a firing and resolved event within 5 minutes.
sum(count_over_time(ALERTS{alertstate="firing"}[5m]) > 0
    and count_over_time(ALERTS{alertstate="resolved"}[5m]) > 0)
/ sum(count_over_time(ALERTS{alertstate="firing"}[5m]) > 0)
* 100
```

### Additional Useful Metrics

```promql
# Alert noise ratio: alerts that auto-resolve vs total alerts
# A high ratio (>50%) indicates too many noisy alerts
sum(changes(ALERTS{alertstate="firing"}[1h])) / sum(count(ALERTS{alertstate="firing"}))

# Alerts per team (requires a 'team' label on alerts)
sum by (team) (ALERTS{alertstate="firing"})

# Longest-firing alert (indicates a stuck or unresolved issue)
time() - ALERTS{alertstate="firing"} == 0
```

## Task 5: Anti-Pattern Audit

The configuration contains the following anti-patterns:

### Anti-Pattern 1: Thresholds too sensitive (HighCPU at 50%)
**Impact**: Fires constantly during normal operations. CPU regularly spikes to 50% during garbage collection, deployments, and traffic bursts.
**Fix**: Raise threshold to 85% and increase `for` to 15 minutes.

### Anti-Pattern 2: Thresholds too sensitive (HighMemory at 60%)
**Impact**: 60% memory is normal. Linux uses available memory for file system cache. This alert will fire 24/7.
**Fix**: Raise threshold to 90% and increase `for` to 10 minutes.

### Anti-Pattern 3: Thresholds too sensitive (DiskUsage at 50%)
**Impact**: 50% disk usage is normal for most systems. This creates noise without any action needed.
**Fix**: Warning at 80%, critical at 90%. Use percentages, not absolute values.

### Anti-Pattern 4: Missing `for` clauses (ErrorRate, PodRestart, DiskUsage, NetworkErrors, ContainerCPU, LogErrors)
**Impact**: Alerts fire on momentary spikes that resolve on their own. A single error log line triggers a critical page.
**Fix**: Add `for` durations. Minimum 2 minutes for critical, 5 minutes for warning.

### Anti-Pattern 5: Wrong severity (everything is critical)
**Impact**: Severity inflation. When everything is critical, nothing is critical. Engineers start ignoring critical alerts because most of them are not actually critical.
**Fix**: Use severity based on user impact. HighCPU and HighMemory should be warning at most.

### Anti-Pattern 6: Alerting on non-problems (NetworkErrors > 0, ContainerCPU > 0.1, LogErrors > 0)
**Impact**: Network errors happen constantly at the hardware level. Container CPU > 0.1 cores is normal. Log errors are expected in any system. These alerts fire 100% of the time.
**Fix**: Remove these alerts or set meaningful thresholds (e.g., error rate > 1% of packets).

### Anti-Pattern 7: No routing (all alerts to one email)
**Impact**: All alerts go to one email inbox. No differentiation between critical pages and informational warnings. No escalation if the on-call is unavailable.
**Fix**: Route critical to PagerDuty, warnings to Slack, database alerts to DBA team.

### Anti-Pattern 8: No inhibition
**Impact**: When a node goes down, you get 50 separate alerts (InstanceDown, HighCPU, HighMemory, PodRestart, etc.) instead of one.
**Fix**: InstanceDown should suppress all other alerts for that instance.

### Anti-Pattern 9: No context (no runbooks, dashboards, or descriptions)
**Impact**: On-call engineer receives "ErrorRate is critical" and has no idea where to look. Spends 20 minutes finding the right dashboard.
**Fix**: Every alert needs summary, description, dashboard_url, and runbook_url annotations.

### Anti-Pattern 10: Poor grouping parameters (group_wait: 10s, group_interval: 1m, repeat_interval: 30m)
**Impact**: `group_wait: 10s` is too short to batch related alerts. `group_interval: 1m` means updates are sent too frequently. `repeat_interval: 30m` means resolved-and-refired alerts create extra noise.
**Fix**: `group_wait: 30s`, `group_interval: 5m`, `repeat_interval: 4h`.

## Common Mistakes

1. **Confusing "informational" with "actionable"**: Not every metric needs an alert. Dashboards and recording rules can track trends without paging anyone.

2. **Setting thresholds at round numbers**: 50%, 80%, 90% are arbitrary. Base thresholds on historical data and SLO targets.

3. **Using absolute values instead of percentages**: "10GB free disk" means different things on a 100GB vs 10TB disk.

4. **Not including context in annotations**: The on-call at 3 AM does not have time to hunt for dashboards and runbooks. Include links in every alert.

5. **Treating all alerts as critical**: Severity should reflect user impact, not how alarming the metric looks. A 50% CPU on a batch node is not critical.

6. **Forgetting the `for` clause**: Without it, any transient spike triggers a page. Most spikes self-heal within seconds.

7. **Alerting on symptoms, not causes**: Alert on "error rate above 5%" (symptom) rather than "CPU above 90%" (possible cause). The symptom tells you users are affected; the cause might be wrong.
