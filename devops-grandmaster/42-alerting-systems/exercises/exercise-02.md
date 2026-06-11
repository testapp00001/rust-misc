# Exercise 02: AlertManager with Routing and Inhibition

## Type: Guided

## Objective

Configure AlertManager with a routing tree that directs alerts to different receivers based on severity and category, implement inhibition rules that suppress lower-severity alerts when a higher-severity alert is already firing, and test the configuration with a simulated webhook receiver.

## Prerequisites

- Docker and Docker Compose installed
- Understanding of Prometheus alerting rules (Module 39)
- Completion of Exercise 01

## Part A: Project Setup

### Task 1: Create the Project Structure

Create the following directory structure:

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

### Task 2: Create Alert Rules

Write a Prometheus alert rules file (`prometheus/alert-rules.yml`) with the following rules:

**Application alerts:**

| Alert Name | Expression | For | Severity | Service |
|-----------|------------|-----|----------|---------|
| HighErrorRate | `sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m])) > 0.05` | 2m | critical | api |
| HighLatency | `histogram_quantile(0.99, sum by (le) (rate(http_request_duration_seconds_bucket[5m]))) > 1` | 5m | warning | api |
| InstanceDown | `up == 0` | 1m | critical | (dynamic) |

**Infrastructure alerts:**

| Alert Name | Expression | For | Severity | Category |
|-----------|------------|-----|----------|----------|
| HighCPU | `100 - (avg by (instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100) > 80` | 15m | warning | infrastructure |
| HighMemory | `(1 - node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes) * 100 > 90` | 5m | warning | infrastructure |
| DiskSpaceLow | `(node_filesystem_avail_bytes{fstype!="tmpfs"} / node_filesystem_size_bytes) * 100 < 10` | 5m | critical | infrastructure |
| DiskSpaceWarning | `(node_filesystem_avail_bytes{fstype!="tmpfs"} / node_filesystem_size_bytes) * 100 < 20` | 5m | warning | infrastructure |

Each alert must include annotations for `summary`, `description`, `dashboard_url`, and `runbook_url`.

### Task 3: Create AlertManager Configuration

Write an AlertManager configuration (`alertmanager/alertmanager.yml`) with the following routing tree:

```
root (receiver: webhook-default)
  |
  +-- severity=critical --> pagerduty-webhook (group_wait: 10s)
  |                       --> slack-critical (continue: true)
  |
  +-- severity=warning  --> slack-warning
  |
  +-- category=infrastructure --> infra-webhook
```

Requirements:
- Group alerts by `alertname`, `cluster`, `service`
- `group_wait`: 30s (default), 10s for critical
- `group_interval`: 5m
- `repeat_interval`: 1h

### Task 4: Create Inhibition Rules

Add the following inhibition rules to your AlertManager configuration:

1. When a **critical** alert fires for a service, suppress all **warning** alerts for the same `alertname` and `service`.
2. When **InstanceDown** fires for an instance, suppress all other alerts for the same `instance`.
3. When **DiskSpaceLow** fires for an instance, suppress **DiskSpaceWarning** for the same `instance`.

### Task 5: Create the Webhook Receiver

Write a Python Flask application (`webhook-receiver/app.py`) that:

1. Has separate endpoints for each receiver: `/alerts`, `/pagerduty`, `/slack-critical`, `/slack-warning`, `/infra`
2. Logs each received alert to stdout with the receiver name, alert name, severity, status, and summary
3. Stores all received alerts in memory
4. Has a `/log` endpoint that returns all received alerts as JSON
5. Has a `/clear` endpoint that resets the alert log

### Task 6: Create Docker Compose

Write a `docker-compose.yml` that runs:
- Prometheus (port 9090)
- AlertManager (port 9093)
- Node Exporter (port 9100)
- Webhook Receiver (port 9999)

Prometheus must be configured to evaluate the alert rules and send firing alerts to AlertManager.

## Part B: Testing

### Task 7: Start the Stack and Verify

1. Start all services: `docker-compose up -d --build`
2. Verify all services are running: `docker-compose ps`
3. Open Prometheus alerts page at `http://localhost:9090/alerts` and verify rules are loaded.
4. Open AlertManager UI at `http://localhost:9093` and verify it is receiving from Prometheus.

### Task 8: Test Routing

1. Manually trigger an alert by sending a POST request to AlertManager's API:
   ```bash
   curl -X POST http://localhost:9093/api/v2/alerts -H "Content-Type: application/json" -d '[
     {
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
     }
   ]'
   ```

2. Check the webhook receiver log to verify the alert was routed to both `pagerduty-webhook` and `slack-critical`:
   ```bash
   curl http://localhost:9999/log | python -m json.tool
   ```

3. Send a warning-level alert and verify it goes to `slack-warning` only.

4. Send an infrastructure alert and verify it goes to `infra-webhook`.

### Task 9: Test Inhibition

1. Clear the alert log: `curl -X POST http://localhost:9999/clear`

2. Send both a critical and a warning alert for the same service:
   ```bash
   curl -X POST http://localhost:9093/api/v2/alerts -H "Content-Type: application/json" -d '[
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
   ```

3. Verify that the warning alert is suppressed (does not appear in the webhook log) while the critical alert is delivered.

4. Test InstanceDown inhibition by sending InstanceDown + HighCPU for the same instance, and verifying HighCPU is suppressed.

## Part C: Analysis

Answer the following questions:

1. What is the difference between `group_wait`, `group_interval`, and `repeat_interval`? When would you increase or decrease each?
2. Why does the routing tree use `continue: true` for the critical-to-Slack route? What would happen without it?
3. How does AlertManager determine which alerts are in the same "group"? What labels matter?
4. If you send 10 alerts with the same `alertname` and `service` but different `instance` labels, how many webhook calls will AlertManager make (assuming `group_by: ['alertname', 'service']`)?

## Success Criteria

- [ ] Prometheus is evaluating all 7 alert rules
- [ ] AlertManager routing sends critical alerts to both PagerDuty and Slack webhooks
- [ ] AlertManager routing sends warning alerts to the Slack warning webhook
- [ ] AlertManager routing sends infrastructure alerts to the infra webhook
- [ ] Inhibition suppresses DiskSpaceWarning when DiskSpaceLow is firing for the same instance
- [ ] Inhibition suppresses all other alerts when InstanceDown is firing for the same instance
- [ ] The webhook receiver logs show correct routing and suppression
- [ ] You can explain the routing tree evaluation order and `continue` behavior

## Hints

<details>
<summary>Hint 1: AlertManager routing tree evaluation</summary>

AlertManager evaluates routes top to bottom within each level. A route is matched if ALL match/match_re conditions are satisfied. By default, the first matching route "consumes" the alert. Use `continue: true` to allow the alert to also match subsequent routes. This is how you send the same alert to multiple receivers (e.g., PagerDuty AND Slack for critical alerts).

</details>

<details>
<summary>Hint 2: Inhibition rule structure</summary>

An inhibition rule has three parts:
- `source_match` (or `source_match_re`): The alert that causes the inhibition.
- `target_match` (or `target_match_re`): The alert(s) to suppress.
- `equal`: Labels that must match between source and target for the inhibition to apply.

If `equal: ['instance']`, the target alert is only suppressed if it has the same `instance` label value as the source alert.

</details>

<details>
<summary>Hint 3: Sending test alerts to AlertManager</summary>

You can send alerts directly to AlertManager's API v2 endpoint without waiting for Prometheus to fire them. This is useful for testing routing and inhibition:

```bash
curl -X POST http://localhost:9093/api/v2/alerts \
  -H "Content-Type: application/json" \
  -d '[{"labels":{"alertname":"Test","severity":"critical"},"annotations":{"summary":"test alert"}}]'
```

The `startsAt` field defaults to now if omitted. Alerts without an `EndsAt` time stay active until manually resolved.

</details>

<details>
<summary>Hint 4: Webhook receiver endpoint routing</summary>

Each receiver in AlertManager can point to a different URL. Your webhook receiver app needs a separate Flask route for each URL path. Use a shared data structure (like a Python list) to store all received alerts regardless of endpoint, then filter by receiver name when displaying.

```python
@app.route('/pagerduty', methods=['POST'])
def handle_pagerduty():
    return process_alert(request.json, 'pagerduty')
```

</details>

<details>
<summary>Hint 5: Debugging routing decisions</summary>

AlertManager logs routing decisions. Check the logs with:
```bash
docker-compose logs alertmanager | grep -i "routing\|dispatch\|inhibit"
```

You can also use `amtool` inside the container to check which routes match:
```bash
docker exec alertmanager amtool config routes test \
  alertname=HighErrorRate severity=critical service=api
```

</details>
