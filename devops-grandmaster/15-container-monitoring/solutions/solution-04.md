# Solution 04: Resource-Based Alerting and Auto-Scaling

---

## Part A: Alerting rules

### `alerts.yml`

```yaml
groups:
  - name: container_resource_alerts
    rules:
      - alert: ContainerHighCPU
        expr: rate(container_cpu_usage_seconds_total{name=~".+",name!~".*cadvisor.*",name!~".*prometheus.*"}[5m]) * 100 > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Container {{ $labels.name }} CPU above 80%"
          description: "Container {{ $labels.name }} has been using {{ printf \"%.1f\" $value }}% CPU for 5 minutes."

      - alert: ContainerHighMemory
        expr: >
          container_memory_usage_bytes{name=~".+",name!~".*cadvisor.*",name!~".*prometheus.*"}
          / container_spec_memory_limit_bytes{name=~".+",name!~".*cadvisor.*",name!~".*prometheus.*"}
          * 100 > 85
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Container {{ $labels.name }} memory above 85% of limit"
          description: "Container {{ $labels.name }} is using {{ printf \"%.1f\" $value }}% of its memory limit. OOM kill risk."

      - alert: ContainerRestartLoop
        expr: increase(container_restart_count{name=~".+",name!~".*cadvisor.*",name!~".*prometheus.*"}[10m]) > 3
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "Container {{ $labels.name }} restarting frequently"
          description: "Container {{ $labels.name }} has restarted {{ printf \"%.0f\" $value }} times in the last 10 minutes."

      - alert: ContainerOOMKilled
        expr: >
          (container_restart_count{name=~".+",name!~".*cadvisor.*",name!~".*prometheus.*"} > 0)
          and
          (container_memory_usage_bytes{name=~".+",name!~".*cadvisor.*",name!~".*prometheus.*"}
           / container_spec_memory_limit_bytes{name=~".+",name!~".*cadvisor.*",name!~".*prometheus.*"} > 0.95)
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Container {{ $labels.name }} likely OOM killed"
          description: "Container {{ $labels.name }} restarted while memory was above 95% of its limit."

      - alert: ContainerNetworkSaturation
        expr: rate(container_network_transmit_packets_dropped_total{name=~".+",name!~".*cadvisor.*",name!~".*prometheus.*"}[5m]) > 100
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Container {{ $labels.name }} dropping network packets"
          description: "Container {{ $labels.name }} is dropping {{ printf \"%.0f\" $value }} packets/sec."

      - alert: ContainerDiskIOSaturation
        expr: rate(container_fs_io_time_seconds_total{name=~".+",name!~".*cadvisor.*",name!~".*prometheus.*"}[5m]) > 0.5
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Container {{ $labels.name }} high disk I/O wait"
          description: "Container {{ $labels.name }} is spending {{ printf \"%.2f\" $value }} seconds/second on disk I/O."
```

---

## Part B: Updated Prometheus configuration

### `prometheus.yml`

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - /etc/prometheus/alerts.yml

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']

scrape_configs:
  - job_name: 'cadvisor'
    static_configs:
      - targets: ['cadvisor:8080']

  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'sample-app'
    static_configs:
      - targets: ['app:5000']
    metrics_path: /
```

Mount `alerts.yml` in the Prometheus container by adding to the `prometheus` service volumes:

```yaml
volumes:
  - ./prometheus.yml:/etc/prometheus/prometheus.yml
  - ./alerts.yml:/etc/prometheus/alerts.yml
  - prometheus_data:/prometheus
```

---

## Part C: Alertmanager and webhook receiver

### `alertmanager.yml`

```yaml
global:
  resolve_timeout: 5m

route:
  receiver: 'warning-webhook'
  group_by: ['name']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  routes:
    - match:
        severity: critical
      receiver: 'critical-webhook'
      group_wait: 15s
    - match:
        severity: warning
      receiver: 'warning-webhook'

receivers:
  - name: 'critical-webhook'
    webhook_configs:
      - url: 'http://webhook-receiver:5001/webhook'
        send_resolved: true

  - name: 'warning-webhook'
    webhook_configs:
      - url: 'http://webhook-receiver:5001/webhook'
        send_resolved: true
```

**Explanation:**
- `group_by: ['name']` groups alerts by container name, so multiple alerts for the same container are batched into one notification.
- `group_wait: 30s` waits 30 seconds before sending the first notification for a new group (batches multiple alerts firing at the same time).
- `group_interval: 5m` waits 5 minutes between notifications for the same group.
- `repeat_interval: 4h` re-sends unresolved alerts every 4 hours.
- `send_resolved: true` sends a "resolved" notification when an alert clears.

### `webhook-receiver/server.py`

The provided Python script from the exercise is correct. No changes needed.

### `webhook-receiver/Dockerfile`

```dockerfile
FROM python:3.12-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY server.py .

EXPOSE 5001

CMD ["python", "server.py"]
```

### `webhook-receiver/requirements.txt`

```
flask==3.0.0
```

### Updated `docker-compose.yml` additions

Add to the services:

```yaml
  alertmanager:
    image: prom/alertmanager:v0.26.0
    volumes:
      - ./alertmanager.yml:/etc/alertmanager/alertmanager.yml
    ports:
      - "9093:9093"
    command:
      - '--config.file=/etc/alertmanager/alertmanager.yml'

  webhook-receiver:
    build: ./webhook-receiver
    ports:
      - "5001:5001"
```

---

## Part D: Verification

### All services running

```bash
docker compose up -d --build
docker compose ps
```

Expected: 5 services running (app, cadvisor, prometheus, alertmanager, webhook-receiver).

### Alertmanager running

```bash
curl -s http://localhost:9093/api/v2/status
```

Expected: JSON with `cluster` status information.

### Prometheus loaded rules

```bash
curl -s http://localhost:9090/api/v1/rules | python3 -m json.tool
```

Expected: JSON showing the `container_resource_alerts` group with all 6 rules.

---

## Part E: Triggering the alert

### Modified app server.py additions

Add these endpoints to the Flask app:

```python
allocated_blocks = []

@app.route('/stress/memory/<int:mb>')
def stress_memory(mb):
    block = bytearray(mb * 1024 * 1024)
    allocated_blocks.append(block)
    return jsonify(allocated_mb=len(allocated_blocks) * mb)

@app.route('/stress/cpu/<int:seconds>')
def stress_cpu(seconds):
    import time
    end = time.time() + seconds
    while time.time() < end:
        _ = sum(i * i for i in range(100000))
    return jsonify(cpu_stress_seconds=seconds)
```

### Triggering the alert

```bash
# Allocate memory in 50MB chunks
for i in $(seq 1 4); do
  curl -s http://localhost:5000/stress/memory/50
  sleep 2
done
```

After allocating ~200MB (out of 256MB limit), the container is at ~78% memory. Allocate one more chunk:

```bash
curl -s http://localhost:5000/stress/memory/50
```

Now at ~250MB/256MB = ~97%, which exceeds the 85% threshold.

### Check Prometheus alerts

```bash
curl -s http://localhost:9090/api/v1/alerts | python3 -m json.tool
```

Expected: The `ContainerHighMemory` alert should be in `pending` state (waiting for the 2-minute `for` duration). After 2 minutes, it transitions to `firing`.

### Check Alertmanager

```bash
curl -s http://localhost:9093/api/v2/alerts | python3 -m json.tool
```

Expected: The alert appears with severity `critical` and the container name.

### Check webhook receiver logs

```bash
docker compose logs -f webhook-receiver
```

Expected output:

```
============================================================
ALERT RECEIVED at 2024-01-15 10:30:45.123456
============================================================
  Status: firing
  Alert: ContainerHighMemory
  Severity: critical
  Container: monitoring-setup-app-1
  Summary: Container monitoring-setup-app-1 memory above 85% of limit
  Description: Container monitoring-setup-app-1 is using 97.6% of its memory limit. OOM kill risk.
```

---

## Part F: Recovery

```bash
docker compose restart app
```

After the container restarts, its memory is cleared (allocated_blocks list is empty). The memory usage drops below 85%, and after the 2-minute `for` duration, the alert resolves.

Check webhook logs for the resolved notification:

```
  Status: resolved
  Alert: ContainerHighMemory
  ...
```

---

## Part G: Bonus -- Auto-recovery webhook

```python
from flask import Flask, request
import subprocess
import json
from datetime import datetime

app = Flask(__name__)

@app.route('/webhook', methods=['POST'])
def webhook():
    data = request.json
    for alert in data.get('alerts', []):
        if alert.get('status') == 'firing':
            alertname = alert.get('labels', {}).get('alertname', '')
            container = alert.get('labels', {}).get('name', '')

            if alertname == 'ContainerHighMemory' and container:
                print(f"[{datetime.now()}] AUTO-RECOVERY: Restarting {container}")
                try:
                    result = subprocess.run(
                        ['docker', 'restart', container],
                        capture_output=True, text=True, timeout=30
                    )
                    if result.returncode == 0:
                        print(f"[{datetime.now()}] SUCCESS: {container} restarted")
                    else:
                        print(f"[{datetime.now()}] FAILED: {result.stderr}")
                except Exception as e:
                    print(f"[{datetime.now()}] ERROR: {e}")
        else:
            print(f"[{datetime.now()}] RESOLVED: {alert.get('labels', {}).get('alertname')}")

    return '', 200

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5001)
```

**Warning:** This requires the webhook-receiver container to have access to the Docker socket:

```yaml
webhook-receiver:
  build: ./webhook-receiver
  ports:
    - "5001:5001"
  volumes:
    - /var/run/docker.sock:/var/run/docker.sock
```

---

## Key insights

### `for` duration vs `group_wait`

- **`for`** (in alert rule): How long the PromQL expression must be continuously true before the alert transitions from `pending` to `firing`. This prevents flapping (brief spikes triggering alerts).
- **`group_wait`** (in Alertmanager): How long Alertmanager waits before sending the first notification for a new group of alerts. This batches multiple alerts that fire simultaneously into one notification.
- **`group_interval`**: Minimum time between notifications for the same group.
- **`repeat_interval`**: How often to re-send notifications for unresolved alerts.

### Why `for: 0m` for ContainerRestartLoop

The restart count is already an aggregate (it uses `increase(...[10m])`). The 10-minute window in the PromQL expression already provides stability. Adding a `for` duration on top would delay detection. For cumulative metrics with built-in time windows, `for: 0m` is appropriate.

### Alert severity design

- **warning**: Something is approaching a problem. Someone should look at it soon. Used for CPU > 80%, network drops.
- **critical**: Something is broken or about to break. Someone needs to act immediately. Used for memory > 85% (OOM risk), restart loops, OOM kills.
