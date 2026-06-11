# Module 42: Alerting Systems -- PagerDuty, AlertManager, On-Call Rotations

> **Previous Module:** [41 -- Centralized Logging](../41-centralized-logging/README.md)
> **Next Module:** [43 -- SLO Monitoring](../43-slo-monitoring/README.md)
> **Phase:** 5 -- Observability

---

## 1. The Problem

Your monitoring stack is set up. Prometheus collects metrics, Grafana shows dashboards, Loki aggregates logs. But nobody is watching the dashboards at 3 AM on a Saturday.

A disk fills up silently. The database connection pool gets exhausted. Error rate climbs from 0.1% to 15%. By the time someone notices on Monday morning, the damage is done -- customers have churned, data has been lost, and the incident has been running for 60 hours.

You need alerts. But here is the trap: if you alert on everything, you create alert fatigue. Engineers get paged 20 times a night for non-issues. They start ignoring pages. Then a real incident hits, and nobody responds.

The challenge is building an alerting system that:
- Alerts on real problems that need human attention
- Does NOT alert on transient spikes or self-healing issues
- Routes alerts to the right team at the right time
- Provides enough context to start debugging immediately
- Supports on-call rotations so no one person is always on call

---

## 2. The Naive Way

### Anti-Pattern: Alert on Everything

```yaml
# Alerting on every possible condition
groups:
  - name: everything
    rules:
      - alert: HighCPU
        expr: node_cpu_usage > 50  # 50% CPU is normal
        for: 1m
      - alert: HighMemory
        expr: node_memory_usage > 60  # 60% memory is fine
        for: 1m
      - alert: DiskIO
        expr: node_disk_io > 0  # Any disk I/O
      - alert: HighLatency
        expr: http_latency_p99 > 0.1  # 100ms is not an emergency
        for: 1m
      - alert: AnyError
        expr: http_errors_total > 0  # Any error at all
      - alert: PodRestarted
        expr: increase(kube_pod_restart_total[1h]) > 0  # Single restart is normal
```

Result: hundreds of alerts per day, most of them noise. Engineers stop responding.

### Anti-Pattern: Email-Only Alerts

```yaml
# Sending alerts to a shared email inbox
alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:903']
receivers:
  - name: email
    email_configs:
      - to: 'oncall@example.com'
        from: 'alertmanager@example.com'
        smarthost: 'smtp.example.com:587'
```

Problems:
- Nobody checks email at 3 AM
- Emails get buried in spam filters
- No escalation -- if the on-call person is unavailable, the alert is lost
- No acknowledgment mechanism
- No routing to different teams

### Anti-Pattern: No Grouping

```yaml
# Every alert fires individually
receivers:
  - name: slack
    slack_configs:
      - channel: '#alerts'
        send_resolved: false
```

When a network issue causes 50 pods to restart, you get 50 separate Slack messages. The channel is flooded, and the real message is lost in noise.

### Anti-Pattern: No Runbook Links

```yaml
- alert: HighErrorRate
  expr: error_rate > 0.05
  for: 5m
  annotations:
    summary: "Error rate is high"  # Now what?
```

The on-call engineer gets paged, opens the alert, and has no idea what to do. They spend 20 minutes figuring out where to look, what dashboards to check, and what the likely causes are.

---

## 3. The Right Way

### Alert Design Principles

**1. Every alert must be actionable.**

If the on-call engineer cannot do anything about the alert, it should not be an alert.

```yaml
# BAD: Informational, not actionable
- alert: HighTraffic
  expr: rate(http_requests_total[5m]) > 1000
  # So what? High traffic is not a problem.

# GOOD: Actionable -- something is broken
- alert: HighErrorRate
  expr: rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m]) > 0.05
  for: 5m
  annotations:
    summary: "Error rate above 5% for 5 minutes"
    runbook_url: "https://runbooks.example.com/high-error-rate"
```

**2. Every alert needs context.**

```yaml
annotations:
  summary: "{{ $labels.service }} error rate is {{ $value | humanizePercentage }}"
  description: |
    Service: {{ $labels.service }}
    Current error rate: {{ $value | humanizePercentage }}
    Threshold: 5%
    Dashboard: https://grafana.example.com/d/service-overview?var-service={{ $labels.service }}
    Runbook: https://runbooks.example.com/high-error-rate
    Logs: https://logs.example.com/explore?query={service="{{ $labels.service }}"} | json | level="ERROR"
```

**3. Use appropriate severity levels.**

| Severity | Meaning | Response Time | Notification |
|----------|---------|---------------|--------------|
| **critical** | Service is down or data loss is occurring | Immediate (page) | Phone call + SMS + Slack |
| **warning** | Service is degraded but functional | Within 1 hour | Slack + email |
| **info** | Something to be aware of | Next business day | Dashboard only |

**4. Use `for` clauses to avoid flapping.**

```yaml
# BAD: Fires on a momentary spike
- alert: HighCPU
  expr: node_cpu_usage > 90
  for: 0s  # Instant fire

# GOOD: Only fires if sustained for 15 minutes
- alert: HighCPU
  expr: node_cpu_usage > 90
  for: 15m  # Must be high for 15 minutes
```

### AlertManager Architecture

```
Prometheus --> AlertManager --> Routing Tree --> Receivers
                                  |               +-- Slack
                                  |               +-- PagerDuty
                                  +-- Inhibition  +-- Email
                                  +-- Silencing   +-- Opsgenie
                                  +-- Grouping    +-- Webhook
```

### AlertManager Configuration

```yaml
# alertmanager.yml
global:
  resolve_timeout: 5m
  slack_api_url: 'https://hooks.slack.com/services/T00/B00/xxx'
  pagerduty_url: 'https://events.pagerduty.com/v2/enqueue'

# Templates for notifications
templates:
  - '/etc/alertmanager/templates/*.tmpl'

# Route tree -- how alerts are routed
route:
  # Default receiver
  receiver: 'slack-default'

  # Group alerts by these labels
  group_by: ['alertname', 'cluster', 'service']

  # Wait before sending a group notification
  group_wait: 30s

  # Wait before sending updates about the same group
  group_interval: 5m

  # Wait before re-sending a resolved notification
  repeat_interval: 4h

  # Child routes -- matched top to bottom
  routes:
    # Critical alerts go to PagerDuty
    - match:
        severity: critical
      receiver: 'pagerduty-critical'
      group_wait: 10s
      continue: true

    # Also send critical to Slack
    - match:
        severity: critical
      receiver: 'slack-critical'

    # Warnings go to Slack
    - match:
        severity: warning
      receiver: 'slack-warning'

    # Database alerts go to DBA team
    - match_re:
        alertname: '.*Database.*|.*Postgres.*|.*Redis.*'
      receiver: 'dba-team'

    # Network alerts go to infrastructure team
    - match:
        category: network
      receiver: 'infra-team'

    # Info-level alerts only go to dashboards
    - match:
        severity: info
      receiver: 'slack-info'

# Inhibition rules -- suppress lower severity when higher fires
inhibit_rules:
  # If a critical alert is firing, suppress warning for the same service
  - source_match:
      severity: 'critical'
    target_match:
      severity: 'warning'
    equal: ['alertname', 'cluster', 'service']

  # If instance is down, suppress all other alerts for that instance
  - source_match:
      alertname: 'InstanceDown'
    target_match_re:
      alertname: '.*'
    equal: ['instance']

# Receivers
receivers:
  - name: 'slack-default'
    slack_configs:
      - channel: '#alerts'
        send_resolved: true
        title: '{{ .GroupLabels.alertname }}'
        text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'

  - name: 'slack-critical'
    slack_configs:
      - channel: '#alerts-critical'
        send_resolved: true
        color: '{{ if eq .Status "firing" }}danger{{ else }}good{{ end }}'
        title: '[CRITICAL] {{ .GroupLabels.alertname }}'
        text: |
          {{ range .Alerts }}
          *Summary:* {{ .Annotations.summary }}
          *Description:* {{ .Annotations.description }}
          *Dashboard:* {{ .Annotations.dashboard_url }}
          *Runbook:* {{ .Annotations.runbook_url }}
          {{ end }}

  - name: 'slack-warning'
    slack_configs:
      - channel: '#alerts-warning'
        send_resolved: true

  - name: 'pagerduty-critical'
    pagerduty_configs:
      - service_key: '<pagerduty-integration-key>'
        severity: '{{ .GroupLabels.severity }}'
        description: '{{ .GroupLabels.alertname }}: {{ .CommonAnnotations.summary }}'
        details:
          firing: '{{ .Alerts.Firing | len }}'
          resolved: '{{ .Alerts.Resolved | len }}'
          dashboard_url: '{{ .CommonAnnotations.dashboard_url }}'
          runbook_url: '{{ .CommonAnnotations.runbook_url }}'

  - name: 'dba-team'
    slack_configs:
      - channel: '#dba-alerts'
    pagerduty_configs:
      - service_key: '<dba-pagerduty-key>'

  - name: 'infra-team'
    slack_configs:
      - channel: '#infra-alerts'

  - name: 'slack-info'
    slack_configs:
      - channel: '#alerts-info'
        send_resolved: false
```

### PagerDuty Integration

PagerDuty handles on-call scheduling, escalation, and notification delivery.

```yaml
# PagerDuty service configuration
receivers:
  - name: 'pagerduty-critical'
    pagerduty_configs:
      - service_key: '<integration-key-from-pagerduty>'
        severity: critical
        description: '{{ .CommonAnnotations.summary }}'
        details:
          alertname: '{{ .GroupLabels.alertname }}'
          service: '{{ .GroupLabels.service }}'
          cluster: '{{ .GroupLabels.cluster }}'
          dashboard: '{{ .CommonAnnotations.dashboard_url }}'
          runbook: '{{ .CommonAnnotations.runbook_url }}'
          grafana: '{{ .CommonAnnotations.grafana_url }}'
```

**PagerDuty escalation policy:**

```
Level 1: On-call engineer (push notification + phone call)
    | (no acknowledgment in 10 minutes)
Level 2: Secondary on-call (push notification + phone call)
    | (no acknowledgment in 10 minutes)
Level 3: Team lead (phone call)
    | (no acknowledgment in 10 minutes)
Level 4: Engineering manager (phone call + SMS)
```

### On-Call Rotation Design

**Good rotation patterns:**

```
Weekly rotation:
  Week 1: Alice (primary), Bob (secondary)
  Week 2: Carol (primary), Dave (secondary)
  Week 3: Eve (primary), Alice (secondary)
  ...

Follow-the-sun (for global teams):
  US hours (9am-5pm PT):   US team
  EU hours (9am-5pm CET):  EU team
  APAC hours (9am-5pm JST): APAC team
```

**On-call best practices:**

1. **Minimum 2 people on-call** -- primary and secondary
2. **1-week rotations** -- long enough to learn, short enough to not burn out
3. **Compensate on-call** -- either pay or compensatory time off
4. **Post-incident reviews** -- learn from every page
5. **Runbooks for every alert** -- the on-call should not need tribal knowledge
6. **Shadow shifts** -- new team members shadow before going on-call

### Runbook Links in Alerts

Every critical alert should link to a runbook.

```yaml
groups:
  - name: api-alerts
    rules:
      - alert: HighErrorRate
        expr: |
          sum(rate(http_requests_total{service="api", status=~"5.."}[5m]))
          / sum(rate(http_requests_total{service="api"}[5m]))
          > 0.05
        for: 5m
        labels:
          severity: critical
          service: api
          category: application
        annotations:
          summary: "API error rate above 5% for 5 minutes"
          description: |
            Error rate: {{ $value | humanizePercentage }}
            Threshold: 5%
            Duration: 5+ minutes
          dashboard_url: "https://grafana.example.com/d/api-overview?var-service=api"
          runbook_url: "https://runbooks.example.com/api/high-error-rate"
          logs_url: "https://logs.example.com/explore?query={service=\"api\"}|json|level=\"ERROR\""
          traces_url: "https://traces.example.com/search?service=api&status=error"

      - alert: InstanceDown
        expr: up{job="api-server"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "API instance {{ $labels.instance }} is down"
          runbook_url: "https://runbooks.example.com/infra/instance-down"
          description: |
            Instance {{ $labels.instance }} has been unreachable for 1 minute.
            Check:
            1. Is the pod running? kubectl get pods -l app=api -n production
            2. Are there recent events? kubectl describe pod {{ $labels.pod }} -n production
            3. Check logs: kubectl logs {{ $labels.pod }} -n production --tail=100
```

### Alert Inhibition

Inhibition prevents lower-severity alerts from firing when a higher-severity alert for the same service is already active.

```yaml
inhibit_rules:
  # When a service is completely down, suppress latency and error rate alerts
  - source_match:
      alertname: 'ServiceDown'
    target_match_re:
      alertname: 'HighLatency|HighErrorRate|LowThroughput'
    equal: ['service', 'cluster']

  # When a node is down, suppress all alerts for pods on that node
  - source_match:
      alertname: 'NodeDown'
    target_match:
      alertname: 'PodNotReady'
    equal: ['node']

  # When a critical alert fires, suppress warnings for the same issue
  - source_match:
      severity: 'critical'
    target_match:
      severity: 'warning'
    equal: ['alertname', 'service']
```

### Alert Silencing

Silences temporarily suppress alerts during maintenance or known issues.

```bash
# Create a silence via amtool (AlertManager CLI)
amtool silence add \
  alertname="HighLatency" \
  instance="api-server-01:8080" \
  --comment="Upgrading api-server-01 to v2.1.0" \
  --duration=2h \
  --author="alice@example.com"

# Create a silence for a specific maintenance window
amtool silence add \
  service="database" \
  --comment="Database maintenance window" \
  --start="2024-03-15T02:00:00Z" \
  --end="2024-03-15T04:00:00Z" \
  --author="dba-team"

# List active silences
amtool silence query

# Expire a silence
amtool silence expire <silence-id>
```

---

## 4. The Production Way

### Multi-Tier Alerting Strategy

```
Tier 1: Pages (wake someone up)
  - Service completely down
  - Data loss occurring
  - Security breach detected
  - SLO error budget exhausted

Tier 2: Urgent notifications (respond within 1 hour)
  - Error rate above threshold
  - Latency degradation
  - Disk running low
  - Certificate expiring in 7 days

Tier 3: Informational (next business day)
  - Deployment completed
  - Certificate expiring in 30 days
  - Capacity approaching threshold
  - Performance regression detected
```

### Alert Fatigue Prevention

```yaml
# Only page for sustained issues
- alert: HighErrorRate
  expr: error_rate > 0.05
  for: 5m  # Not 1m, not 0s

# Use multi-window burn rate for SLO-based alerts (Module 43)
# This is more nuanced than simple threshold alerts

# Review and prune alerts quarterly
# Track: alerts per week, pages per on-call shift, false positive rate
```

**Alert quality metrics:**

```promql
# Total alerts firing (should be low)
ALERTS{alertstate="firing"}

# Alert rate by severity
sum by (severity) (ALERTS{alertstate="firing"})

# Time to acknowledge (from PagerDuty)
# Track MTTR (Mean Time to Resolve)
```

### AlertManager in High Availability

```yaml
# Run 3 AlertManager instances with gossip protocol
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: alertmanager
spec:
  replicas: 3
  selector:
    matchLabels:
      app: alertmanager
  template:
    metadata:
      labels:
        app: alertmanager
    spec:
      containers:
        - name: alertmanager
          image: prom/alertmanager:latest
          args:
            - '--config.file=/etc/alertmanager/alertmanager.yml'
            - '--storage.path=/alertmanager'
            - '--cluster.peer=alertmanager-0:9094'
            - '--cluster.peer=alertmanager-1:9094'
            - '--cluster.peer=alertmanager-2:9094'
            - '--cluster.listen-address=0.0.0.0:9094'
          ports:
            - containerPort: 9093
            - containerPort: 9094
          volumeMounts:
            - name: config
              mountPath: /etc/alertmanager
            - name: storage
              mountPath: /alertmanager
      volumes:
        - name: config
          configMap:
            name: alertmanager-config
  volumeClaimTemplates:
    - metadata:
        name: storage
      spec:
        accessModes: ["ReadWriteOnce"]
        resources:
          requests:
            storage: 1Gi
```

### Opsgenie Integration

```yaml
receivers:
  - name: 'opsgenie-critical'
    opsgenie_configs:
      - api_key: '<opsgenie-api-key>'
        message: '{{ .CommonAnnotations.summary }}'
        description: '{{ .CommonAnnotations.description }}'
        priority: 'P1'
        responders:
          - type: 'team'
            name: 'platform-team'
          - type: 'schedule'
            name: 'on-call-schedule'
        details:
          dashboard: '{{ .CommonAnnotations.dashboard_url }}'
          runbook: '{{ .CommonAnnotations.runbook_url }}'
        tags:
          - 'prometheus'
          - '{{ .GroupLabels.severity }}'
```

### Webhook Integration for Custom Workflows

```yaml
receivers:
  - name: 'webhook-custom'
    webhook_configs:
      - url: 'http://alert-webhook:8080/alerts'
        send_resolved: true
        http_config:
          bearer_token: '<auth-token>'
```

```python
# Custom webhook handler
from flask import Flask, request, jsonify
import json

app = Flask(__name__)

@app.route('/alerts', methods=['POST'])
def handle_alert():
    alert = request.json

    for a in alert.get('alerts', []):
        severity = a['labels'].get('severity', 'unknown')
        status = a['status']  # firing or resolved
        summary = a['annotations'].get('summary', '')
        runbook = a['annotations'].get('runbook_url', '')

        if status == 'firing' and severity == 'critical':
            # Create a Jira ticket
            create_jira_ticket(summary, runbook, a)
            # Update status page
            update_status_page(summary, 'investigating')
        elif status == 'resolved':
            # Resolve the Jira ticket
            resolve_jira_ticket(summary)
            # Update status page
            update_status_page(summary, 'resolved')

    return jsonify({"status": "ok"})
```

### Alert Templates

```yaml
# /etc/alertmanager/templates/default.tmpl
{{ define "slack.default.title" }}
[{{ .Status | toUpper }}{{ if eq .Status "firing" }}:{{ .Alerts.Firing | len }}{{ end }}] {{ .GroupLabels.alertname }}
{{ end }}

{{ define "slack.default.text" }}
{{ range .Alerts }}
*Alert:* {{ .Labels.alertname }}
*Severity:* {{ .Labels.severity }}
*Summary:* {{ .Annotations.summary }}
*Description:* {{ .Annotations.description }}
*Started:* {{ .StartsAt.Format "2006-01-02 15:04:05 MST" }}
{{ if .EndsAt }}*Ended:* {{ .EndsAt.Format "2006-01-02 15:04:05 MST" }}{{ end }}
{{ if .Annotations.dashboard_url }}*Dashboard:* <{{ .Annotations.dashboard_url }}|View>{{ end }}
{{ if .Annotations.runbook_url }}*Runbook:* <{{ .Annotations.runbook_url }}|View>{{ end }}
---
{{ end }}
{{ end }}
```

### Runbook Template

```markdown
# Runbook: HighErrorRate

## Symptoms
- Grafana dashboard shows 5xx error rate > 5%
- PagerDuty alert "HighErrorRate" is firing
- Users reporting "Service Unavailable" errors

## Impact
- ~X% of requests are failing
- Affects: [describe user impact]

## Diagnosis
1. Check Grafana dashboard: [link]
2. Check recent deployments: `kubectl rollout history deployment/api-server`
3. Check Pod logs: `kubectl logs -l app=api-server --tail=100`
4. Check upstream dependencies: [dashboard link]

## Mitigation
1. If caused by recent deployment:
   kubectl rollout undo deployment/api-server
2. If caused by dependency:
   [dependency runbook link]
3. If caused by resource exhaustion:
   kubectl scale deployment api-server --replicas=10

## Resolution
- Root cause: [fill in after incident]
- Follow-up: [link to postmortem]
```

---

## 5. Hands-On Lab

### Lab: AlertManager with PagerDuty

**Objective:** Set up AlertManager with multiple receivers, routing, inhibition, and silencing. Integrate with a notification webhook (simulating PagerDuty).

**Step 1: Project structure**

```bash
mkdir alerting-lab && cd alerting-lab
mkdir -p prometheus alertmanager webhook-receiver
```

**Step 2: Create alerting rules**

```yaml
# prometheus/alert-rules.yml
groups:
  - name: application-alerts
    rules:
      - alert: HighErrorRate
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[5m]))
          / sum(rate(http_requests_total[5m]))
          > 0.05
        for: 2m
        labels:
          severity: critical
          service: api
        annotations:
          summary: "API error rate above 5%"
          description: "Error rate is {{ $value | humanizePercentage }} for service api"
          dashboard_url: "http://grafana:3000/d/api-overview"
          runbook_url: "https://runbooks.example.com/high-error-rate"

      - alert: HighLatency
        expr: |
          histogram_quantile(0.99, sum by (le) (rate(http_request_duration_seconds_bucket[5m]))) > 1
        for: 5m
        labels:
          severity: warning
          service: api
        annotations:
          summary: "API p99 latency above 1 second"
          description: "p99 latency is {{ $value }}s"

      - alert: InstanceDown
        expr: up == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Instance {{ $labels.instance }} is down"

  - name: infrastructure-alerts
    rules:
      - alert: HighCPU
        expr: |
          100 - (avg by (instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100) > 80
        for: 15m
        labels:
          severity: warning
          category: infrastructure
        annotations:
          summary: "CPU above 80% on {{ $labels.instance }}"

      - alert: HighMemory
        expr: |
          (1 - node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes) * 100 > 90
        for: 5m
        labels:
          severity: warning
          category: infrastructure
        annotations:
          summary: "Memory above 90% on {{ $labels.instance }}"

      - alert: DiskSpaceLow
        expr: |
          (node_filesystem_avail_bytes{fstype!="tmpfs"} / node_filesystem_size_bytes) * 100 < 10
        for: 5m
        labels:
          severity: critical
          category: infrastructure
        annotations:
          summary: "Disk space below 10% on {{ $labels.instance }}"

      - alert: DiskSpaceWarning
        expr: |
          (node_filesystem_avail_bytes{fstype!="tmpfs"} / node_filesystem_size_bytes) * 100 < 20
        for: 5m
        labels:
          severity: warning
          category: infrastructure
        annotations:
          summary: "Disk space below 20% on {{ $labels.instance }}"
```

**Step 3: Create AlertManager configuration**

```yaml
# alertmanager/alertmanager.yml
global:
  resolve_timeout: 5m

route:
  receiver: 'webhook-default'
  group_by: ['alertname', 'cluster', 'service']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 1h

  routes:
    # Critical alerts to PagerDuty (simulated webhook)
    - match:
        severity: critical
      receiver: 'pagerduty-webhook'
      group_wait: 10s
      continue: true

    # Critical also to Slack (simulated webhook)
    - match:
        severity: critical
      receiver: 'slack-critical'

    # Warnings to Slack
    - match:
        severity: warning
      receiver: 'slack-warning'

    # Infrastructure alerts to infra team
    - match:
        category: infrastructure
      receiver: 'infra-webhook'

inhibit_rules:
  # Critical inhibits warning for same alertname/service
  - source_match:
      severity: 'critical'
    target_match:
      severity: 'warning'
    equal: ['alertname', 'service']

  # InstanceDown inhibits everything for that instance
  - source_match:
      alertname: 'InstanceDown'
    target_match_re:
      alertname: '.*'
    equal: ['instance']

  # DiskSpaceLow inhibits DiskSpaceWarning for same instance
  - source_match:
      alertname: 'DiskSpaceLow'
    target_match:
      alertname: 'DiskSpaceWarning'
    equal: ['instance']

receivers:
  - name: 'webhook-default'
    webhook_configs:
      - url: 'http://webhook-receiver:9999/alerts'
        send_resolved: true

  - name: 'pagerduty-webhook'
    webhook_configs:
      - url: 'http://webhook-receiver:9999/pagerduty'
        send_resolved: true
        http_config:
          bearer_token: 'pagerduty-sim-token'

  - name: 'slack-critical'
    webhook_configs:
      - url: 'http://webhook-receiver:9999/slack-critical'
        send_resolved: true

  - name: 'slack-warning'
    webhook_configs:
      - url: 'http://webhook-receiver:9999/slack-warning'
        send_resolved: true

  - name: 'infra-webhook'
    webhook_configs:
      - url: 'http://webhook-receiver:9999/infra'
        send_resolved: true
```

**Step 4: Create webhook receiver**

```python
# webhook-receiver/app.py
from flask import Flask, request, jsonify
from datetime import datetime
import json

app = Flask(__name__)

alert_log = []

@app.route('/alerts', methods=['POST'])
def handle_default():
    return process_alert(request.json, 'default')

@app.route('/pagerduty', methods=['POST'])
def handle_pagerduty():
    return process_alert(request.json, 'pagerduty')

@app.route('/slack-critical', methods=['POST'])
def handle_slack_critical():
    return process_alert(request.json, 'slack-critical')

@app.route('/slack-warning', methods=['POST'])
def handle_slack_warning():
    return process_alert(request.json, 'slack-warning')

@app.route('/infra', methods=['POST'])
def handle_infra():
    return process_alert(request.json, 'infra')

def process_alert(data, receiver):
    timestamp = datetime.now().isoformat()

    for alert in data.get('alerts', []):
        entry = {
            'timestamp': timestamp,
            'receiver': receiver,
            'status': alert.get('status', 'unknown'),
            'alertname': alert.get('labels', {}).get('alertname', 'unknown'),
            'severity': alert.get('labels', {}).get('severity', 'unknown'),
            'summary': alert.get('annotations', {}).get('summary', ''),
            'description': alert.get('annotations', {}).get('description', ''),
            'instance': alert.get('labels', {}).get('instance', ''),
            'runbook': alert.get('annotations', {}).get('runbook_url', ''),
        }

        alert_log.append(entry)

        # Format output based on receiver
        if receiver == 'pagerduty':
            print(f"\n{'='*60}")
            print(f"PAGERDUTY ALERT - {entry['status'].upper()}")
            print(f"  Alert: {entry['alertname']}")
            print(f"  Severity: {entry['severity']}")
            print(f"  Summary: {entry['summary']}")
            print(f"  Runbook: {entry['runbook']}")
            print(f"  Time: {entry['timestamp']}")
            print(f"{'='*60}")
        elif 'slack' in receiver:
            color = 'danger' if entry['severity'] == 'critical' else 'warning'
            print(f"\n[{receiver.upper()}] {entry['alertname']}")
            print(f"  {entry['summary']}")
            print(f"  Severity: {entry['severity']} | Status: {entry['status']}")
        else:
            print(f"\n[{receiver}] {entry['alertname']}: {entry['summary']} ({entry['status']})")

    return jsonify({"status": "received"})

@app.route('/log')
def get_log():
    return jsonify(alert_log)

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=9999)
```

**Step 5: Prometheus configuration**

```yaml
# prometheus/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - /etc/prometheus/alert-rules.yml

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']

scrape_configs:
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']

  - job_name: 'alertmanager'
    static_configs:
      - targets: ['alertmanager:9093']
```

**Step 6: Docker Compose**

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

```dockerfile
# webhook-receiver/Dockerfile
FROM python:3.11-slim
WORKDIR /app
RUN pip install flask
COPY app.py .
CMD ["python", "app.py"]
```

**Step 7: Test the alerting system**

```bash
# Start everything
docker-compose up -d --build

# Wait for services
sleep 10

# Check AlertManager UI
open http://localhost:9093

# Check Prometheus alerts page
open http://localhost:9090/alerts

# Generate load to trigger alerts
for i in $(seq 1 2000); do
  curl -s http://localhost:8080/api/users > /dev/null 2>&1 &
  if [ $((i % 10)) -eq 0 ]; then
    wait
  fi
done

# Check webhook receiver for received alerts
curl http://localhost:9999/log | python -m json.tool

# Watch the webhook receiver logs
docker-compose logs -f webhook-receiver
```

**Step 8: Test silencing**

```bash
# Install amtool
docker exec -it alertmanager amtool silence add \
  alertname="HighCPU" \
  --comment="Expected high load during batch job" \
  --duration=1h

# List silences
docker exec -it alertmanager amtool silence query

# Expire the silence
docker exec -it alertmanager amtool expire <silence-id>
```

**Expected outcome:**
- Prometheus evaluates alert rules every 15 seconds
- Firing alerts are sent to AlertManager
- AlertManager routes to appropriate receivers based on labels
- Critical alerts trigger the PagerDuty webhook
- Warning alerts trigger the Slack webhook
- Inhibition suppresses lower-severity alerts when critical fires
- Silencing prevents alerts during maintenance windows

---

## 6. Limitation

Alerting tells you when something is wrong. But "wrong" is relative -- is a 2% error rate a problem? It depends on your SLO. If your SLO is 99.9% success rate, 2% errors (98% success) is a serious breach. If your SLO is 95%, 2% errors is well within budget.

Traditional threshold-based alerts (error rate > 5%, latency > 1s) do not account for your actual reliability targets. They either fire too often (causing fatigue) or too rarely (missing real problems).

The better approach is to alert based on SLOs and error budgets -- alerting when you are burning through your error budget too fast, rather than when a metric crosses an arbitrary threshold.

**Next Module:** [43 -- SLO Monitoring](../43-slo-monitoring/README.md) -- Implement SLO-based alerting with error budgets and burn rate, so alerts fire when your reliability targets are at risk, not when arbitrary thresholds are crossed.
