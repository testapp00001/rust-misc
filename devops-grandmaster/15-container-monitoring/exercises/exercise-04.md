# Exercise 04: Resource-Based Alerting and Auto-Scaling

**Type:** Challenge
**Difficulty:** Intermediate-Advanced
**Time:** 90-120 minutes

---

## Objective

Configure Prometheus alerting rules for container resource thresholds, set up Alertmanager to route notifications, and simulate a scenario where a container triggers alerts and gets replaced by a new instance (manual auto-scaling recovery).

---

## Prerequisites

- Completed Exercise 02 and Exercise 03
- Your `monitoring-setup/` directory with cAdvisor, Prometheus, and Grafana running

---

## Part A: Create Prometheus alerting rules

Create `alerts.yml` with the following alerting rules:

### Alert 1: High CPU Usage

- Alert name: `ContainerHighCPU`
- Expression: Container CPU usage exceeds 80% for 5 minutes
- Severity label: `warning`
- Annotations: include container name and current CPU percentage

### Alert 2: High Memory Usage

- Alert name: `ContainerHighMemory`
- Expression: Container memory usage exceeds 85% of its configured limit for 2 minutes
- Severity label: `critical`
- Annotations: include container name, current usage, and limit

### Alert 3: Container Restarting Frequently

- Alert name: `ContainerRestartLoop`
- Expression: A container has restarted more than 3 times in the last 10 minutes
- Severity label: `critical`
- Annotations: include container name and restart count

### Alert 4: Container OOM Killed

- Alert name: `ContainerOOMKilled`
- Expression: A container has been OOM killed (use the `container_oom_events_total` metric or detect via restart count spike with memory at limit)
- Severity label: `critical`
- Annotations: include container name

### Alert 5: Network Saturation

- Alert name: `ContainerNetworkSaturation`
- Expression: Packet drop rate exceeds 100 packets/second for 5 minutes
- Severity label: `warning`
- Annotations: include container name and drop rate

### Alert 6: Disk I/O Saturation

- Alert name: `ContainerDiskIOSaturation`
- Expression: Disk I/O time exceeds 0.5 seconds/second (50% of time spent waiting on I/O) for 5 minutes
- Severity label: `warning`
- Annotations: include container name

---

## Part B: Update Prometheus configuration

Update `prometheus.yml` to:

1. Reference the alerting rules file:

```yaml
rule_files:
  - /etc/prometheus/alerts.yml
```

2. Add Alertmanager configuration:

```yaml
alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']
```

3. Mount the `alerts.yml` file in the Prometheus container in `docker-compose.yml`.

---

## Part C: Add Alertmanager

1. Create `alertmanager.yml` configuration:

Write an Alertmanager configuration that:
- Has a default receiver that sends to a webhook
- Routes alerts by severity label:
  - `critical` alerts go to a receiver called `critical-webhook`
  - `warning` alerts go to a receiver called `warning-webhook`
- Groups alerts by container name
- Sets a group wait of 30 seconds
- Sets a group interval of 5 minutes
- Sets a repeat interval of 4 hours

2. Create a simple webhook receiver for testing:

Create `webhook-receiver/server.py`:

```python
from flask import Flask, request
import json
from datetime import datetime

app = Flask(__name__)

@app.route('/webhook', methods=['POST'])
def webhook():
    data = request.json
    print(f"\n{'='*60}")
    print(f"ALERT RECEIVED at {datetime.now()}")
    print(f"{'='*60}")
    for alert in data.get('alerts', []):
        print(f"  Status: {alert.get('status')}")
        print(f"  Alert: {alert.get('labels', {}).get('alertname')}")
        print(f"  Severity: {alert.get('labels', {}).get('severity')}")
        print(f"  Container: {alert.get('labels', {}).get('name')}")
        print(f"  Summary: {alert.get('annotations', {}).get('summary')}")
        print(f"  Description: {alert.get('annotations', {}).get('description')}")
        print()
    return '', 200

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5001)
```

Create `webhook-receiver/Dockerfile`:

Write a Dockerfile for the webhook receiver using `python:3.12-slim` and Flask.

3. Add to `docker-compose.yml`:

- `alertmanager` service using `prom/alertmanager:v0.26.0`
  - Mount `alertmanager.yml` to `/etc/alertmanager/alertmanager.yml`
  - Port: 9093:9093
- `webhook-receiver` service
  - Build from `./webhook-receiver`
  - Port: 5001:5001

---

## Part D: Verify alerting pipeline

1. Restart the full stack:

```bash
docker compose up -d --build
```

2. Verify Alertmanager is running:

```bash
curl -s http://localhost:9093/api/v2/status
```

3. Verify Prometheus has loaded the alert rules:

```bash
curl -s http://localhost:9090/api/v1/rules | python3 -m json.tool
```

4. Verify the webhook receiver is running:

```bash
curl -s http://localhost:5001/
```

---

## Part E: Trigger an alert

1. Create a stress test that forces the sample app container to exceed its memory limit.

Modify your `app/server.py` to add an endpoint that allocates memory:

```python
import resource

allocated_blocks = []

@app.route('/stress/memory/<int:mb>')
def stress_memory(mb):
    """Allocate MB megabytes of memory."""
    block = bytearray(mb * 1024 * 1024)
    allocated_blocks.append(block)
    return jsonify(allocated_mb=len(allocated_blocks) * mb)

@app.route('/stress/cpu/<int:seconds>')
def stress_cpu(seconds):
    """Burn CPU for N seconds."""
    import time
    end = time.time() + seconds
    while time.time() < end:
        _ = sum(i * i for i in range(100000))
    return jsonify(cpu_stress_seconds=seconds)
```

2. Rebuild and restart the app:

```bash
docker compose up -d --build app
```

3. Trigger the HighMemory alert:

```bash
# Allocate memory in 50MB chunks until the container is near its 256MB limit
for i in $(seq 1 4); do
  curl -s http://localhost:5000/stress/memory/50
  sleep 2
done
```

4. Watch for the alert in Prometheus:

```bash
curl -s http://localhost:9090/api/v1/alerts | python3 -m json.tool
```

5. Check that Alertmanager received the alert:

```bash
curl -s http://localhost:9093/api/v2/alerts | python3 -m json.tool
```

6. Check the webhook receiver logs:

```bash
docker compose logs webhook-receiver
```

---

## Part F: Simulate recovery (manual auto-scaling)

1. After the alert fires, restart the container to clear its memory:

```bash
docker compose restart app
```

2. Verify the alert resolves:

```bash
# Wait 2 minutes for the alert to clear
sleep 120
curl -s http://localhost:9090/api/v1/alerts | python3 -m json.tool
```

3. Verify the webhook received a "resolved" notification:

```bash
docker compose logs webhook-receiver
```

---

## Part G: Bonus -- Custom webhook handler that auto-recovers

Write a webhook handler that, upon receiving a `ContainerHighMemory` alert, automatically restarts the offending container using the Docker API. This is a simplified version of what tools like Kubernetes HPA do.

Requirements:
- The webhook receives the alert payload
- It extracts the container name from the alert labels
- It runs `docker restart <container_name>`
- It logs the action taken

Warning: This is for learning only. Never auto-restart containers in production based on a single alert without human oversight.

---

## Success Criteria

- [ ] `alerts.yml` contains all 6 alerting rules with correct PromQL expressions.
- [ ] Prometheus loads the rules and shows them at `/api/v1/rules`.
- [ ] Alertmanager is running and configured with severity-based routing.
- [ ] Webhook receiver logs show received alerts with correct severity and container name.
- [ ] You can trigger the `ContainerHighMemory` alert by stressing the app.
- [ ] You can verify the alert resolves after the container is restarted.
- [ ] You can explain the difference between `for` duration and `group_wait` in alerting.

---

## Hints

<details>
<summary>Hint 1: Alert rule structure</summary>

Each alert rule needs:
- `alert`: The alert name (string)
- `expr`: PromQL expression that evaluates to true when the alert should fire
- `for`: How long the expression must be true before firing (prevents flapping)
- `labels`: Metadata attached to the alert (use for routing)
- `annotations`: Human-readable descriptions (use template variables like `{{ $labels.name }}` and `{{ $value }}`)

</details>

<details>
<summary>Hint 2: Alertmanager routing tree</summary>

Alertmanager routes alerts through a tree of `route` blocks. The top-level route matches all alerts. Child routes match based on label matchers:

```yaml
route:
  receiver: 'default-webhook'
  group_by: ['name']
  group_wait: 30s
  routes:
    - match:
        severity: critical
      receiver: 'critical-webhook'
    - match:
        severity: warning
      receiver: 'warning-webhook'
```

</details>

<details>
<summary>Hint 3: Detecting OOM kills in PromQL</summary>

There is no direct `container_oom_kills_total` metric in all cAdvisor versions. You can detect OOM kills indirectly:

```promql
# Container restarted and memory was at its limit before restart
(container_restart_count{name=~".+"} > 0)
  and
(container_memory_usage_bytes{name=~".+"} / container_spec_memory_limit_bytes{name=~".+"} > 0.95)
```

Or use the kernel-level metric if available:

```promql
container_oom_events_total{name=~".+"} > 0
```

</details>

<details>
<summary>Hint 4: Webhook receiver requirements.txt</summary>

```
flask==3.0.0
```

The webhook receiver Dockerfile is similar to the app Dockerfile. No special dependencies are needed.

</details>

<details>
<summary>Hint 5: Testing alerts without real load</summary>

You can use Prometheus's `/api/v1/alerts` endpoint to see pending and firing alerts. An alert in `pending` state means the expression has been true but the `for` duration has not elapsed yet. An alert in `firing` state means it has been true long enough to fire.

To speed up testing, temporarily reduce the `for` duration in your alert rules (e.g., `for: 30s` instead of `for: 5m`).

</details>
