# Solution 02: AlertManager with Routing and Inhibition

## Part A: Project Setup

### Task 1: Project Structure

```
alertmanager-lab/
  prometheus/
    prometheus.yml
    alert-rules.yml
  alertmanager/
    alertmanager.yml
  webhook-receiver/
    Dockerfile
    app.py
  docker-compose.yml
```

### Task 2: Alert Rules

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

### Task 3: AlertManager Configuration

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

### Task 4: Inhibition Rules (included above)

Three inhibition rules are configured:

1. **Critical suppresses warning for same alertname/service**: When `HighErrorRate` fires as critical, any `HighLatency` warning for the same service is suppressed (if they share the same `alertname` and `service` labels).

2. **InstanceDown suppresses everything for that instance**: When an instance is down, all other alerts for that instance (HighCPU, HighMemory, etc.) are suppressed. This prevents 50 separate alerts when a node dies.

3. **DiskSpaceLow suppresses DiskSpaceWarning**: When disk is below 10% (critical), the 20% warning is suppressed. You only need the most severe alert.

### Task 5: Webhook Receiver

```python
# webhook-receiver/app.py
from flask import Flask, request, jsonify
from datetime import datetime

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


@app.route('/clear', methods=['POST'])
def clear_log():
    alert_log.clear()
    return jsonify({"status": "cleared"})


if __name__ == '__main__':
    app.run(host='0.0.0.0', port=9999)
```

```dockerfile
# webhook-receiver/Dockerfile
FROM python:3.11-slim
WORKDIR /app
RUN pip install flask
COPY app.py .
CMD ["python", "app.py"]
```

### Task 6: Docker Compose

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

## Part B: Testing

### Task 7: Verify Stack

```bash
# Start
docker-compose up -d --build

# Verify
docker-compose ps

# Check Prometheus alerts page
open http://localhost:9090/alerts

# Check AlertManager UI
open http://localhost:9093
```

### Task 8: Test Routing

```bash
# Send a critical alert
curl -X POST http://localhost:9093/api/v2/alerts \
  -H "Content-Type: application/json" \
  -d '[{
    "labels": {
      "alertname": "HighErrorRate",
      "severity": "critical",
      "service": "api"
    },
    "annotations": {
      "summary": "API error rate above 5%",
      "description": "Error rate is 8.2%",
      "dashboard_url": "http://grafana:3000/d/api-overview",
      "runbook_url": "https://runbooks.example.com/high-error-rate"
    }
  }]'

# Wait for group_wait (10s for critical)
sleep 15

# Check webhook log -- should see entries for both pagerduty-webhook and slack-critical
curl http://localhost:9999/log | python -m json.tool
```

Expected result: The critical alert appears in both `/pagerduty` and `/slack-critical` receivers because of `continue: true` on the first matching route.

```bash
# Clear log
curl -X POST http://localhost:9999/clear

# Send a warning alert
curl -X POST http://localhost:9093/api/v2/alerts \
  -H "Content-Type: application/json" \
  -d '[{
    "labels": {
      "alertname": "HighLatency",
      "severity": "warning",
      "service": "api"
    },
    "annotations": {
      "summary": "API p99 latency above 1 second"
    }
  }]'

# Wait for group_wait (30s default)
sleep 35

# Should see entry only in slack-warning
curl http://localhost:9999/log | python -m json.tool
```

### Task 9: Test Inhibition

```bash
# Clear log
curl -X POST http://localhost:9999/clear

# Send both critical and warning for the same service
curl -X POST http://localhost:9093/api/v2/alerts \
  -H "Content-Type: application/json" \
  -d '[
    {
      "labels": {
        "alertname": "HighErrorRate",
        "severity": "critical",
        "service": "api",
        "cluster": "prod"
      },
      "annotations": { "summary": "API error rate critical" }
    },
    {
      "labels": {
        "alertname": "HighLatency",
        "severity": "warning",
        "service": "api",
        "cluster": "prod"
      },
      "annotations": { "summary": "API latency high" }
    }
  ]'

# Wait and check
sleep 35
curl http://localhost:9999/log | python -m json.tool
```

Expected result: Only the critical `HighErrorRate` alert appears. The warning `HighLatency` is suppressed by the inhibition rule (critical suppresses warning for the same `service`).

Note: Inhibition requires matching on `equal` labels. In this case, the inhibition rule matches on `alertname` and `service`. Since the two alerts have different `alertname` values, the inhibition rule with `equal: ['alertname', 'service']` will NOT suppress HighLatency. The inhibition only works when the source and target share the same values for the `equal` labels.

To test the InstanceDown inhibition:
```bash
curl -X POST http://localhost:9093/api/v2/alerts \
  -H "Content-Type: application/json" \
  -d '[
    {
      "labels": {
        "alertname": "InstanceDown",
        "severity": "critical",
        "instance": "node-1:9100"
      },
      "annotations": { "summary": "Instance node-1 is down" }
    },
    {
      "labels": {
        "alertname": "HighCPU",
        "severity": "warning",
        "instance": "node-1:9100",
        "category": "infrastructure"
      },
      "annotations": { "summary": "CPU high on node-1" }
    }
  ]'
```

Expected result: Only `InstanceDown` is delivered. `HighCPU` is suppressed because InstanceDown inhibits all alerts with the same `instance` label.

## Part C: Analysis

**1. group_wait vs group_interval vs repeat_interval:**

- **group_wait**: Time to wait before sending the first notification for a new group. Batches initial alerts that fire at the same time. Increase to batch more alerts; decrease for faster notification of the first alert.
- **group_interval**: Minimum time between notifications for the same group. After sending a notification, wait this long before sending another. Increase to reduce notification frequency; decrease to get faster updates when new alerts join.
- **repeat_interval**: Time before re-sending a notification for a still-firing group. Increase to reduce noise from long-running alerts; decrease if you want regular reminders.

**2. Why `continue: true`:**

Without `continue: true`, the first matching route "consumes" the alert and it is not evaluated against subsequent routes. With `continue: true`, the alert is also sent to the next matching route. This is how you send the same critical alert to both PagerDuty AND Slack.

If you removed `continue: true`, critical alerts would only go to the PagerDuty webhook and would NOT appear in the Slack critical channel.

**3. Group determination:**

Alerts belong to the same group if they have the same values for all labels listed in `group_by`. For example, with `group_by: ['alertname', 'cluster', 'service']`, two alerts with `{alertname: "HighErrorRate", cluster: "prod", service: "api", instance: "pod-1"}` and `{alertname: "HighErrorRate", cluster: "prod", service: "api", instance: "pod-2"}` are in the same group because they share the same `alertname`, `cluster`, and `service` values.

**4. 10 alerts with different instances, grouped by alertname and service:**

All 10 alerts are in the same group (same `alertname` and `service`, `instance` is not in `group_by`). AlertManager sends **1 notification** containing all 10 alerts. This is the power of grouping -- instead of 10 separate messages, the on-call gets one consolidated message.

## Common Mistakes

1. **Forgetting `continue: true`**: If you want an alert to go to multiple receivers, you must set `continue: true` on the first matching route. Otherwise the alert is consumed by the first match.

2. **Inhibition equal labels too strict**: If you set `equal: ['alertname', 'service']` on an inhibition rule, it only suppresses alerts with the SAME alertname AND service as the source. To suppress all alerts for an instance, use `equal: ['instance']`.

3. **Wrong group_wait for critical alerts**: Using the default 30s `group_wait` for critical alerts means the on-call waits 30 seconds before being notified. Override with `group_wait: 10s` on the critical route.

4. **Webhook receiver not handling all endpoints**: Each receiver points to a different URL. The webhook receiver must have a Flask route for each URL path.

5. **Not testing with the AlertManager API**: You can test routing without waiting for Prometheus to fire rules by POSTing directly to `/api/v2/alerts`.
