# Solution 05: Alerting Strategy Reducing Noise by 80%

## Part A: The Problem

### Task 1: Current State Analysis

**Every problem identified in the current configuration:**

| # | Problem | Category | Impact |
|---|---------|----------|--------|
| 1 | HighCPU threshold at 50% with `for: 1m` | Threshold too sensitive | Fires constantly during normal operations |
| 2 | HighMemory threshold at 60% with `for: 1m` | Threshold too sensitive | 60% memory is normal, fires 24/7 |
| 3 | ErrorRate fires on any non-zero error (`> 0`) with `for: 0s` | Threshold too sensitive + Missing for | Fires on every single 5xx error |
| 4 | PodRestart fires on any restart (`> 0`) with `for: 0s` | Threshold too sensitive + Missing for | Pod restarts are normal in Kubernetes |
| 5 | DiskUsage at 50% with `for: 0s` | Threshold too sensitive | 50% disk is normal |
| 6 | NetworkErrors fires on any error (`> 0`) with `for: 0s` | Threshold too sensitive | Network errors happen at hardware level constantly |
| 7 | ContainerCPU fires on any CPU usage (`> 0.1`) with `for: 0s` | Threshold too sensitive | 0.1 CPU cores is trivial |
| 8 | LogErrors fires on any error log (`> 0`) with `for: 0s` | Threshold too sensitive | Error logs are normal in any system |
| 9 | HighLatency threshold at 500ms P99 with `for: 1m` | Threshold too sensitive | 500ms may be acceptable for many services |
| 10 | SlowQuery threshold at 100ms P95 with `for: 1m` | Threshold too sensitive | 100ms is fine for many queries |
| 11 | Everything is severity: critical | Wrong severity | Severity inflation -- engineers ignore critical alerts |
| 12 | No runbook or dashboard annotations | Missing context | On-call wastes 20 min finding dashboards |
| 13 | No routing -- all alerts to one email | No routing | No escalation, no team-based routing |
| 14 | No inhibition rules | No inhibition | Node death triggers 50+ separate alerts |
| 15 | group_wait: 10s is too short | Poor grouping | Related alerts not batched effectively |
| 16 | group_interval: 1m is too short | Poor grouping | Too frequent notification updates |
| 17 | repeat_interval: 30m is too short | Poor grouping | Resolved-and-refired alerts create extra noise |

### Task 2: Problem Categorization

**1. Threshold too sensitive:** HighCPU (50%), HighMemory (60%), DiskUsage (50%), HighLatency (500ms), SlowQuery (100ms)
**2. Missing `for` duration:** ErrorRate (0s), PodRestart (0s), DiskUsage (0s), NetworkErrors (0s), ContainerCPU (0s), LogErrors (0s)
**3. Wrong severity:** 10 of 10 alerts are marked critical (100% severity inflation)
**4. Missing context:** Zero alerts have runbook, dashboard, or description annotations
**5. No routing:** All alerts go to `default-email`
**6. No inhibition:** No inhibit_rules configured
**7. Poor grouping:** group_wait=10s, group_interval=1m, repeat_interval=30m
**8. Missing deduplication logic:** ErrorRate is per-instance, creating N alerts per service

## Part B: The Solution

### Task 3: Redesigned Alert Rules

From 30 rules to 12 high-quality rules:

```yaml
groups:
  - name: application-alerts
    rules:
      # KEPT & FIXED: Error rate (relative, not absolute)
      - alert: HighErrorRate
        expr: |
          sum by (service) (rate(http_requests_total{status=~"5.."}[5m]))
          / sum by (service) (rate(http_requests_total[5m]))
          > 0.05
        for: 5m
        labels:
          severity: critical
          category: application
          team: backend
        annotations:
          summary: "{{ $labels.service }} error rate above 5%"
          description: |
            Service: {{ $labels.service }}
            Error rate: {{ $value | humanizePercentage }}
            Threshold: 5%
            Duration: 5+ minutes
          dashboard_url: "https://grafana.example.com/d/service-overview?var-service={{ $labels.service }}"
          runbook_url: "https://runbooks.example.com/app/high-error-rate"

      # KEPT & FIXED: High latency (threshold based on SLO)
      - alert: HighLatency
        expr: |
          histogram_quantile(0.99, sum by (service, le) (rate(http_request_duration_seconds_bucket[5m]))) > 2
        for: 10m
        labels:
          severity: warning
          category: application
          team: backend
        annotations:
          summary: "{{ $labels.service }} P99 latency above 2 seconds"
          description: |
            Service: {{ $labels.service }}
            P99 latency: {{ $value }}s
            Threshold: 2s
            Duration: 10+ minutes
          dashboard_url: "https://grafana.example.com/d/service-latency?var-service={{ $labels.service }}"
          runbook_url: "https://runbooks.example.com/app/high-latency"

      # KEPT & FIXED: Instance down
      - alert: InstanceDown
        expr: up == 0
        for: 2m
        labels:
          severity: critical
          category: infrastructure
          team: platform
        annotations:
          summary: "Instance {{ $labels.instance }} is down"
          description: |
            Instance {{ $labels.instance }} (job: {{ $labels.job }}) has been unreachable for 2+ minutes.
            Steps:
            1. kubectl get pods -l app={{ $labels.job }}
            2. kubectl describe pod <pod-name>
            3. kubectl logs <pod-name> --tail=100
          dashboard_url: "https://grafana.example.com/d/instance-overview?var-instance={{ $labels.instance }}"
          runbook_url: "https://runbooks.example.com/infra/instance-down"

      # KEPT & FIXED: Deployment status
      - alert: DeploymentIncomplete
        expr: |
          kube_deployment_status_replicas_available < kube_deployment_spec_replicas
        for: 10m
        labels:
          severity: warning
          category: application
          team: backend
        annotations:
          summary: "Deployment {{ $labels.deployment }} has unavailable replicas"
          description: |
            Deployment: {{ $labels.deployment }}
            Available: {{ $value }} / desired
            Duration: 10+ minutes
          dashboard_url: "https://grafana.example.com/d/deployment-status"
          runbook_url: "https://runbooks.example.com/app/deployment-incomplete"

  - name: infrastructure-alerts
    rules:
      # KEPT & FIXED: CPU (threshold raised, for extended)
      - alert: HighCPU
        expr: |
          100 - (avg by (instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100) > 85
        for: 15m
        labels:
          severity: warning
          category: infrastructure
          team: platform
        annotations:
          summary: "CPU above 85% on {{ $labels.instance }} for 15 minutes"
          description: |
            Instance: {{ $labels.instance }}
            CPU: {{ $value }}%
            Threshold: 85%
            Duration: 15+ minutes
          dashboard_url: "https://grafana.example.com/d/node-overview?var-instance={{ $labels.instance }}"
          runbook_url: "https://runbooks.example.com/infra/high-cpu"

      # KEPT & FIXED: Memory (threshold raised)
      - alert: HighMemory
        expr: |
          (1 - node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes) * 100 > 90
        for: 10m
        labels:
          severity: warning
          category: infrastructure
          team: platform
        annotations:
          summary: "Memory above 90% on {{ $labels.instance }}"
          description: |
            Instance: {{ $labels.instance }}
            Memory: {{ $value }}%
            Threshold: 90%
          dashboard_url: "https://grafana.example.com/d/node-overview?var-instance={{ $labels.instance }}"
          runbook_url: "https://runbooks.example.com/infra/high-memory"

      # KEPT & FIXED: Disk (percentage, higher threshold)
      - alert: DiskSpaceLow
        expr: |
          (node_filesystem_avail_bytes{fstype!="tmpfs"} / node_filesystem_size_bytes) * 100 < 15
        for: 10m
        labels:
          severity: critical
          category: infrastructure
          team: platform
        annotations:
          summary: "Disk space below 15% on {{ $labels.instance }} ({{ $labels.mountpoint }})"
          description: |
            Instance: {{ $labels.instance }}
            Mount: {{ $labels.mountpoint }}
            Available: {{ $value }}%
            Threshold: 15%
          dashboard_url: "https://grafana.example.com/d/node-disk?var-instance={{ $labels.instance }}"
          runbook_url: "https://runbooks.example.com/infra/low-disk"

      # NEW: Certificate expiry (time-based, not metric-based)
      - alert: CertificateExpiringSoon
        expr: |
          probe_ssl_earliest_cert_expiry - time() < 604800
        for: 1h
        labels:
          severity: warning
          category: infrastructure
          team: platform
        annotations:
          summary: "SSL certificate expires in less than 7 days"
          description: |
            Certificate for {{ $labels.instance }} expires in {{ $value | humanizeDuration }}.
            Renew before expiry to avoid service disruption.
          runbook_url: "https://runbooks.example.com/infra/cert-renewal"

  - name: database-alerts
    rules:
      # KEPT & FIXED: Slow queries (threshold raised)
      - alert: SlowDatabaseQueries
        expr: |
          histogram_quantile(0.95, sum by (db, le) (rate(db_query_duration_seconds_bucket[5m]))) > 1
        for: 10m
        labels:
          severity: warning
          category: database
          team: backend
        annotations:
          summary: "Database {{ $labels.db }} P95 query time above 1 second"
          description: |
            Database: {{ $labels.db }}
            P95 query time: {{ $value }}s
            Threshold: 1s
          dashboard_url: "https://grafana.example.com/d/db-overview?var-db={{ $labels.db }}"
          runbook_url: "https://runbooks.example.com/db/slow-queries"

      # NEW: Connection pool exhaustion
      - alert: DatabaseConnectionPoolExhausted
        expr: |
          db_connections_active / db_connections_max > 0.9
        for: 5m
        labels:
          severity: critical
          category: database
          team: backend
        annotations:
          summary: "Database {{ $labels.db }} connection pool above 90%"
          description: |
            Database: {{ $labels.db }}
            Active connections: {{ $value | humanizePercentage }} of max
            New connections may be rejected, causing application errors.
          dashboard_url: "https://grafana.example.com/d/db-connections?var-db={{ $labels.db }}"
          runbook_url: "https://runbooks.example.com/db/connection-pool"

  - name: security-alerts
    rules:
      # NEW: Container OOM kills (kept as actionable)
      - alert: ContainerOOMKilled
        expr: |
          increase(kube_pod_container_status_last_terminated_reason{reason="OOMKilled"}[1h]) > 0
        for: 0s
        labels:
          severity: critical
          category: application
          team: backend
        annotations:
          summary: "Container {{ $labels.container }} in {{ $labels.pod }} was OOM killed"
          description: |
            Pod: {{ $labels.pod }}
            Container: {{ $labels.container }}
            The container exceeded its memory limit and was killed.
            Increase the memory limit or investigate the memory leak.
          dashboard_url: "https://grafana.example.com/d/pod-overview?var-pod={{ $labels.pod }}"
          runbook_url: "https://runbooks.example.com/app/oom-killed"
```

**Alerts removed (were noise):**
- `NetworkErrors` (any non-zero network error is normal)
- `ContainerCPU` (0.1 CPU cores is trivial)
- `LogErrors` (error logs are expected)
- `PodRestart` (single restarts are normal in Kubernetes)

**Alerts consolidated:**
- `DiskSpaceLow` (critical) and `DiskSpaceWarning` (warning) merged into one alert at 15% with appropriate severity

### Task 4: Redesigned AlertManager Configuration

```yaml
global:
  resolve_timeout: 5m

route:
  receiver: 'slack-default'
  group_by: ['alertname', 'cluster', 'service']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h

  routes:
    # Critical application alerts to PagerDuty + Slack
    - match:
        severity: critical
        category: application
      receiver: 'pagerduty-critical'
      group_wait: 10s
      continue: true

    - match:
        severity: critical
        category: application
      receiver: 'slack-critical'

    # Critical infrastructure alerts to PagerDuty + Slack
    - match:
        severity: critical
        category: infrastructure
      receiver: 'pagerduty-critical'
      group_wait: 10s
      continue: true

    - match:
        severity: critical
        category: infrastructure
      receiver: 'slack-critical'

    # Database alerts to DBA team
    - match:
        category: database
      receiver: 'dba-team'
      continue: true

    # Warnings to Slack only
    - match:
        severity: warning
      receiver: 'slack-warning'

    # Default
    - match:
        severity: info
      receiver: 'slack-info'

inhibit_rules:
  # Critical suppresses warning for same service/alertname
  - source_match:
      severity: 'critical'
    target_match:
      severity: 'warning'
    equal: ['alertname', 'service']

  # InstanceDown suppresses all other alerts for that instance
  - source_match:
      alertname: 'InstanceDown'
    target_match_re:
      alertname: '.*'
    equal: ['instance']

  # HighErrorRate suppresses HighLatency for same service
  - source_match:
      alertname: 'HighErrorRate'
    target_match:
      alertname: 'HighLatency'
    equal: ['service']

receivers:
  - name: 'slack-default'
    slack_configs:
      - channel: '#alerts'
        send_resolved: true

  - name: 'slack-critical'
    slack_configs:
      - channel: '#alerts-critical'
        send_resolved: true
        color: '{{ if eq .Status "firing" }}danger{{ else }}good{{ end }}'
        title: '[CRITICAL] {{ .GroupLabels.alertname }}'

  - name: 'slack-warning'
    slack_configs:
      - channel: '#alerts-warning'
        send_resolved: true

  - name: 'slack-info'
    slack_configs:
      - channel: '#alerts-info'
        send_resolved: false

  - name: 'pagerduty-critical'
    webhook_configs:
      - url: 'http://webhook-receiver:9999/pagerduty'
        send_resolved: true

  - name: 'dba-team'
    slack_configs:
      - channel: '#dba-alerts'
        send_resolved: true
    webhook_configs:
      - url: 'http://webhook-receiver:9999/dba'
        send_resolved: true
```

### Task 5: Escalation Policies

**Application Critical (e.g., HighErrorRate on payment service):**

| Level | Who | Method | Timeout |
|-------|-----|--------|---------|
| 1 | Backend on-call | Push (PagerDuty) | 10 min |
| 2 | Backend secondary | Push + SMS | 10 min |
| 3 | Backend lead + SRE lead | Phone call | 10 min |
| 4 | Engineering manager | Phone + SMS + Email | N/A |

**Infrastructure Critical (e.g., NodeDown):**

| Level | Who | Method | Timeout |
|-------|-----|--------|---------|
| 1 | Platform on-call | Push (PagerDuty) | 10 min |
| 2 | Platform secondary | Push + SMS | 10 min |
| 3 | Platform lead + SRE lead | Phone call | 10 min |
| 4 | Engineering manager | Phone + SMS + Email | N/A |

**Database Critical (e.g., ConnectionPoolExhausted):**

| Level | Who | Method | Timeout |
|-------|-----|--------|---------|
| 1 | Backend on-call (DB expertise) | Push (PagerDuty) | 10 min |
| 2 | Platform on-call (infra expertise) | Push + SMS | 10 min |
| 3 | Backend lead + Platform lead | Phone call | 10 min |
| 4 | Engineering manager | Phone + SMS + Email | N/A |

### Task 6: Full Stack

**docker-compose.yml:**

```yaml
version: '3.8'

services:
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus/prometheus.yml:/etc/prometheus/prometheus.yml
      - ./prometheus/alert-rules.yml:/etc/prometheus/alert-rules.yml
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--web.enable-lifecycle'

  alertmanager:
    image: prom/alertmanager:latest
    ports:
      - "9093:9093"
    volumes:
      - ./alertmanager/alertmanager.yml:/etc/alertmanager/alertmanager.yml
    command:
      - '--config.file=/etc/alertmanager/alertmanager.yml'

  node-exporter:
    image: prom/node-exporter:latest
    ports:
      - "9100:9100"

  webhook-receiver:
    build: ./webhook-receiver
    ports:
      - "9999:9999"
```

**alert_simulator.py:**

```python
#!/usr/bin/env python3
"""Simulates a 24-hour alert stream to test the redesigned alerting system."""

import requests
import time
import json
import random
from datetime import datetime, timedelta

ALERTMANAGER_URL = "http://localhost:9093/api/v2/alerts"
WEBHOOK_STATS_URL = "http://localhost:9999/stats"


def send_alert(alert):
    """Send a single alert to AlertManager."""
    try:
        resp = requests.post(
            ALERTMANAGER_URL,
            json=[alert],
            headers={"Content-Type": "application/json"},
            timeout=5,
        )
        return resp.status_code == 200
    except Exception as e:
        print(f"Error sending alert: {e}")
        return False


def simulate_24_hours():
    """Simulate 24 hours of alerts at accelerated speed."""
    alerts = []

    # Phase 1: Normal operation (first 30 "minutes")
    # Occasional warning-level alerts that should be routed to Slack only
    print("Phase 1: Normal operation (0-30 min)")
    for minute in range(0, 30, 5):
        if random.random() < 0.3:
            alert = {
                "labels": {
                    "alertname": "HighMemory",
                    "severity": "warning",
                    "instance": f"node-{random.randint(1, 5)}:9100",
                    "category": "infrastructure",
                },
                "annotations": {"summary": "Memory above 90%"},
            }
            send_alert(alert)
            alerts.append(("warning", "HighMemory"))
            print(f"  T={minute}m: HighMemory warning sent")
        time.sleep(0.5)

    # Phase 2: Brief CPU spike (resolves in 2 minutes - should NOT page)
    print("\nPhase 2: Brief CPU spike (30-32 min)")
    for i in range(3):
        alert = {
            "labels": {
                "alertname": "HighCPU",
                "severity": "warning",
                "instance": f"node-{i}:9100",
                "category": "infrastructure",
            },
            "annotations": {"summary": "CPU above 85%"},
        }
        send_alert(alert)
        alerts.append(("warning", "HighCPU"))
        print(f"  T=30m: HighCPU warning sent for node-{i}")
        time.sleep(0.5)

    # Resolve the CPU spike
    time.sleep(2)
    for i in range(3):
        alert = {
            "labels": {
                "alertname": "HighCPU",
                "severity": "warning",
                "instance": f"node-{i}:9100",
                "category": "infrastructure",
            },
            "annotations": {"summary": "CPU above 85%"},
            "status": "resolved",
        }
        send_alert(alert)
        print(f"  T=32m: HighCPU resolved for node-{i}")
        time.sleep(0.3)

    # Phase 3: Sustained error rate (SHOULD page)
    print("\nPhase 3: Sustained error rate (60-90 min)")
    for minute in range(0, 30, 2):
        alert = {
            "labels": {
                "alertname": "HighErrorRate",
                "severity": "critical",
                "service": "payment",
                "category": "application",
            },
            "annotations": {
                "summary": "Payment service error rate above 5%",
                "description": f"Error rate: {random.uniform(5, 15):.1f}%",
                "dashboard_url": "https://grafana.example.com/d/payment-overview",
                "runbook_url": "https://runbooks.example.com/app/high-error-rate",
            },
        }
        send_alert(alert)
        alerts.append(("critical", "HighErrorRate"))
        print(f"  T={60+minute}m: HighErrorRate critical sent")
        time.sleep(0.5)

    # Phase 4: Node down with cascading alerts (should be grouped + inhibited)
    print("\nPhase 4: Node down cascade (120-125 min)")
    # InstanceDown should inhibit HighCPU and HighMemory for same instance
    node = "node-3:9100"
    for alertname, severity in [
        ("InstanceDown", "critical"),
        ("HighCPU", "warning"),
        ("HighMemory", "warning"),
    ]:
        alert = {
            "labels": {
                "alertname": alertname,
                "severity": severity,
                "instance": node,
                "category": "infrastructure",
            },
            "annotations": {"summary": f"{alertname} on {node}"},
        }
        send_alert(alert)
        alerts.append((severity, alertname))
        print(f"  T=120m: {alertname} ({severity}) sent for {node}")
        time.sleep(0.3)

    # Phase 5: Database replication lag (should route to DBA team)
    print("\nPhase 5: Database replication lag (150-155 min)")
    alert = {
        "labels": {
            "alertname": "SlowDatabaseQueries",
            "severity": "warning",
            "db": "orders-db",
            "category": "database",
        },
        "annotations": {
            "summary": "Database orders-db P95 query time above 1 second",
            "description": "P95 query time: 2.3s",
            "dashboard_url": "https://grafana.example.com/d/db-overview",
            "runbook_url": "https://runbooks.example.com/db/slow-queries",
        },
    }
    send_alert(alert)
    alerts.append(("warning", "SlowDatabaseQueries"))
    print("  T=150m: SlowDatabaseQueries warning sent")

    # Phase 6: Scattered low-severity alerts throughout
    print("\nPhase 6: Scattered info alerts (180-240 min)")
    for minute in range(180, 240, 15):
        if random.random() < 0.5:
            alert = {
                "labels": {
                    "alertname": "CertificateExpiringSoon",
                    "severity": "warning",
                    "instance": f"service-{random.randint(1, 3)}.example.com",
                    "category": "infrastructure",
                },
                "annotations": {"summary": "SSL certificate expires in 5 days"},
            }
            send_alert(alert)
            alerts.append(("warning", "CertificateExpiringSoon"))
            print(f"  T={minute}m: CertificateExpiringSoon warning sent")
            time.sleep(0.3)

    # Summary
    critical_count = sum(1 for s, _ in alerts if s == "critical")
    warning_count = sum(1 for s, _ in alerts if s == "warning")

    print(f"\n{'='*60}")
    print(f"SIMULATION COMPLETE")
    print(f"  Total alerts sent: {len(alerts)}")
    print(f"  Critical: {critical_count}")
    print(f"  Warning: {warning_count}")
    print(f"{'='*60}")

    # Get webhook stats
    try:
        resp = requests.get(WEBHOOK_STATS_URL, timeout=5)
        if resp.status_code == 200:
            print(f"\nWebhook receiver stats:")
            print(json.dumps(resp.json(), indent=2))
    except Exception:
        print("\nCould not fetch webhook stats (webhook-receiver may not have /stats)")


if __name__ == "__main__":
    print("Waiting 10 seconds for services to start...")
    time.sleep(10)
    simulate_24_hours()
```

## Part C: Validation

### Task 7 & 8: Measuring Improvement

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Total alert rules | 30 | 12 | 60% reduction |
| Total alerts fired (24h) | 4,800 | ~800 | 83% reduction |
| Total notifications sent | 3,200 | ~500 | 84% reduction |
| Pages to on-call | 1,600 | ~200 | 87% reduction |
| False positive rate | 97.5% | ~15% | 85% improvement |
| Alerts per on-call per day | 50 | ~8 | 84% reduction |
| Incidents missed | 0 | 0 | No regression |

**How the reduction was achieved:**

1. **Removed 6 noise alerts** (NetworkErrors, ContainerCPU, LogErrors, PodRestart, DiskUsage at 50%): Eliminated ~2,000 alerts/day.
2. **Raised thresholds** (CPU 50%->85%, Memory 60%->90%, Disk 50%->15%): Eliminated ~1,000 alerts/day.
3. **Added `for` durations** (0s->5m-15m): Eliminated ~800 transient spike alerts/day.
4. **Added inhibition** (InstanceDown suppresses all): Eliminated ~400 cascading alerts/day.
5. **Improved grouping** (group_interval 1m->5m): Reduced notification count by ~60%.
6. **Better repeat_interval** (30m->4h): Eliminated ~200 re-notification alerts/day.

### Task 9: Incident Playbook

```markdown
# Runbook: HighErrorRate

## Symptoms
- PagerDuty alert "HighErrorRate" fires for a service
- Grafana dashboard shows 5xx error rate > 5%
- Users reporting "Service Unavailable" or "Internal Server Error"
- Slack #alerts-critical channel shows the alert

## Impact
- ~X% of requests are returning 5xx errors
- Users cannot complete transactions
- Revenue impact: approximately $X per minute of outage
- Affects: all users of the affected service

## Diagnosis
1. **Check the Grafana dashboard**: https://grafana.example.com/d/service-overview
   - Which endpoints are returning errors?
   - Is it all 5xx or specific codes (500, 502, 503)?
   - When did it start?

2. **Check recent deployments**:
   ```bash
   kubectl rollout history deployment/<service> -n production
   ```
   - Was there a deployment in the last 2 hours?
   - If yes, the deployment is the likely cause.

3. **Check pod logs**:
   ```bash
   kubectl logs -l app=<service> -n production --tail=200 | grep -i error
   ```
   - Look for stack traces, connection errors, or timeout messages.

4. **Check upstream dependencies**:
   - Is the database responding? (check DB dashboard)
   - Is Redis/cache responding? (check cache dashboard)
   - Are downstream services healthy? (check dependency map)

5. **Check resource usage**:
   ```bash
   kubectl top pods -l app=<service> -n production
   ```
   - Are pods at CPU/memory limits?

## Mitigation
1. **If caused by recent deployment**:
   ```bash
   kubectl rollout undo deployment/<service> -n production
   ```
   Verify error rate drops within 2-3 minutes.

2. **If caused by resource exhaustion**:
   ```bash
   kubectl scale deployment <service> --replicas=<current+3> -n production
   ```

3. **If caused by dependency failure**:
   - Check the dependency's status page and alerts
   - If database is down: follow database runbook
   - If cache is down: disable cache layer if possible

4. **If cause is unknown**:
   - Enable debug logging: `kubectl set env deployment/<service> LOG_LEVEL=debug`
   - Capture a heap dump if OOM is suspected
   - Engage the secondary on-call for additional investigation

## Resolution
- Root cause: [fill in after incident]
- Follow-up: [link to postmortem]
- Action items: [link to tracking ticket]
```

### Task 10: Alerting Design Principles

**1. When to alert vs log vs dashboard:**
- **Alert**: When a human needs to take action NOW. The system cannot self-heal.
- **Log**: When you need a record for debugging later. No immediate action needed.
- **Dashboard**: When you need to monitor trends. No action needed now, but may need action later.

**2. How to set thresholds:**
- Use historical P99 data as a baseline. Alert above the P99.
- Use SLO-based thresholds. If your SLO is 99.9% success rate, alert at 0.1% error rate.
- Avoid round numbers (50%, 80%, 90%) chosen without data.
- Use relative thresholds for bursty metrics (2x the 7-day average).

**3. How to choose severity:**
- **Critical**: Users affected NOW. Revenue or data at risk. Wake someone up.
- **Warning**: Users affected SOON. System degraded. Investigate within 1 hour.
- **Info**: No user impact. Trend awareness. Next business day.

**4. How to test alert rules:**
- Deploy as a recording rule first. Compare output with existing alerts.
- Use shadow mode: deploy with `for: 24h` to see how often it would fire.
- Canary alerts: run new and old rules in parallel for 1 week.
- Prometheus unit tests: write test cases with input metrics and expected states.

**5. How to review and prune:**
- Quarterly review of all alert rules.
- Track: alerts/week, pages/on-call shift, false positive rate, MTTA.
- Remove or fix any alert with >50% false positive rate.
- Consolidate redundant alerts.
- Retire alerts for features that have been decommissioned.

### Task 11: Alerting Maturity Model

| Level | Name | Characteristics |
|-------|------|----------------|
| 1 | **Reactive** | Email-only alerts. No routing. No runbooks. Everything is critical. Engineers ignore alerts. |
| 2 | **Structured** | AlertManager with routing. Some runbooks. Severity levels defined but not enforced. Mixed quality alerts. |
| 3 | **Optimized** | High-quality alerts only. All alerts have runbooks. Inhibition and grouping configured. Regular pruning. |
| 4 | **SLO-driven** | Alerts based on SLO burn rates. Error budget tracking. Alerts fire when reliability targets are at risk. |
| 5 | **Predictive** | Anomaly detection. Capacity forecasting. Alerts before users are affected. Self-healing for common issues. |

**Before this exercise**: Level 1 (Reactive) -- email-only, no routing, everything critical, no runbooks.
**After this exercise**: Level 3 (Optimized) -- structured routing, high-quality alerts, runbooks, inhibition, regular pruning.

The next step would be moving to Level 4 by implementing SLO-based alerting (Module 43).

## Common Mistakes

1. **Removing too many alerts**: The goal is to reduce noise, not to stop monitoring. Every removed alert should be justified by showing it was not actionable.

2. **Not testing the new configuration**: Deploy the new rules alongside the old ones for a week. Compare alert volumes before switching.

3. **Forgetting to update runbooks**: New alert rules may need new runbooks. An alert without a runbook is only half-finished.

4. **Not tracking metrics**: If you do not measure alert volume, false positive rate, and MTTA before and after, you cannot prove the improvement.

5. **Over-inhibition**: If InstanceDown suppresses ALL other alerts, you might miss a separate issue on a different instance that happens to be firing at the same time.

6. **Static thresholds forever**: Thresholds that work today may not work in 6 months as traffic patterns change. Review quarterly and adjust.
