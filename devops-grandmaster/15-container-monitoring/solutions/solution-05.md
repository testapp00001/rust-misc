# Solution 05: Monitoring Strategy for a Production Cluster

---

## Part A: Monitoring architecture design

### 1. Metric collection strategy

**API Server (Node.js)**
| Metric | USE Dimension | Normal | Abnormal |
|--------|--------------|--------|----------|
| CPU usage < 60% | Utilization | < 60% | > 80% for 5m |
| Memory usage < 70% of limit | Utilization | < 70% | > 85% for 2m |
| Request latency p99 < 500ms | Saturation | < 500ms | > 2s for 5m |
| Error rate < 1% | Errors | < 1% | > 5% for 3m |

**Worker (Python)**
| Metric | USE Dimension | Normal | Abnormal |
|--------|--------------|--------|----------|
| CPU usage < 50% | Utilization | < 50% | > 80% for 5m |
| Memory usage < 60% of limit | Utilization | < 60% | > 85% for 2m |
| Queue depth < 100 | Saturation | < 100 | > 1000 for 10m |
| Job failure rate < 2% | Errors | < 2% | > 10% for 5m |

**PostgreSQL**
| Metric | USE Dimension | Normal | Abnormal |
|--------|--------------|--------|----------|
| CPU usage < 40% | Utilization | < 40% | > 80% for 5m |
| Memory usage < 70% of limit | Utilization | < 70% | > 85% for 2m |
| Active connections < 80% of max | Saturation | < 80% | > 95% for 2m |
| Disk I/O wait < 0.3s/s | Saturation | < 0.3 | > 0.5 for 5m |

**Redis**
| Metric | USE Dimension | Normal | Abnormal |
|--------|--------------|--------|----------|
| Memory usage < 60% of limit | Utilization | < 60% | > 80% for 2m |
| Connected clients < 100 | Saturation | < 100 | > 500 for 5m |
| Evicted keys = 0 | Errors | 0 | > 0 for 5m |

**Nginx**
| Metric | USE Dimension | Normal | Abnormal |
|--------|--------------|--------|----------|
| CPU usage < 20% | Utilization | < 20% | > 50% for 5m |
| Active connections < 500 | Saturation | < 500 | > 1000 for 5m |
| 5xx error rate < 0.1% | Errors | < 0.1% | > 1% for 2m |

### 2. Dashboard design

**Executive Overview (6-8 panels)**
1. Service Health Status -- stat -- `up{job=~".*"}` -- Shows which services are up/down at a glance
2. Total Request Rate -- timeseries -- `sum(rate(http_requests_total[5m]))` -- Overall system load
3. Error Rate -- timeseries -- `sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m])) * 100` -- System-wide error percentage
4. CPU Summary -- gauge -- `rate(container_cpu_usage_seconds_total{name=~".+"}[5m]) * 100` -- Per-container CPU at a glance
5. Memory Summary -- gauge -- `container_memory_usage_bytes / container_spec_memory_limit_bytes * 100` -- Per-container memory as % of limit
6. Active Alerts -- stat -- `ALERTS{alertstate="firing"}` -- Count of firing alerts by severity
7. Container Restarts -- stat -- `container_restart_count` -- Total restarts across services
8. SLO Compliance -- gauge -- Availability percentage over 30m window

**Service Deep Dive (per-service panels)**
- CPU usage (timeseries)
- Memory usage (timeseries)
- Request rate (timeseries)
- Error rate (timeseries)
- Latency percentiles p50/p95/p99 (timeseries)
- Custom metrics (queue depth for worker, connection count for postgres, etc.)

**Infrastructure**
- CPU per core (timeseries) -- `rate(node_cpu_seconds_total[5m])` or per-container
- Memory breakdown (stacked timeseries) -- used, cache, free
- Disk I/O per container (timeseries) -- `rate(container_fs_io_time_seconds_total[5m])`
- Network throughput per container (timeseries) -- receive + transmit rates
- Container count over time (timeseries)

### 3. Alerting philosophy

**Critical (page on-call):**
- Container OOM killed
- Service completely down (0 requests for 5m)
- Error rate > 10% for 3m
- SLO breach (availability < 99.9% over 30m)

**Warning (Slack/email):**
- CPU > 80% for 5m
- Memory > 85% of limit for 2m
- Latency p99 > 2s for 5m
- Queue depth > 1000 for 10m

**SLO targets:**
- API availability: 99.9% (43 minutes downtime/month)
- API latency p99: < 2 seconds
- Order processing throughput: > 100 orders/hour

**Preventing alert fatigue:**
- Only alert on user-visible impact, not on internal metrics that may be transient
- Use `for` durations to prevent flapping
- Use inhibition rules to suppress lower-severity alerts when higher-severity is firing
- Review and tune thresholds monthly based on actual data
- Remove alerts that have not fired in 90 days (they add noise)

---

## Part B: Application stack

### API Server: `apps/api/server.js`

```javascript
const express = require('express');
const client = require('prom-client');

const app = express();
const register = client.register;

client.collectDefaultMetrics({ prefix: 'api_' });

const httpRequestDuration = new client.Histogram({
  name: 'api_http_request_duration_seconds',
  help: 'Duration of HTTP requests in seconds',
  labelNames: ['method', 'route', 'status'],
  buckets: [0.01, 0.05, 0.1, 0.25, 0.5, 1, 2, 5],
});

const httpRequestsTotal = new client.Counter({
  name: 'api_http_requests_total',
  help: 'Total number of HTTP requests',
  labelNames: ['method', 'route', 'status'],
});

const activeConnections = new client.Gauge({
  name: 'api_active_connections',
  help: 'Number of active connections',
});

// Middleware
app.use((req, res, next) => {
  const start = Date.now();
  activeConnections.inc();
  res.on('finish', () => {
    const duration = (Date.now() - start) / 1000;
    httpRequestDuration.observe(
      { method: req.method, route: req.path, status: String(res.statusCode) },
      duration
    );
    httpRequestsTotal.inc(
      { method: req.method, route: req.path, status: String(res.statusCode) }
    );
    activeConnections.dec();
  });
  next();
});

app.get('/', (req, res) => {
  res.json({ status: 'ok', service: 'api' });
});

app.get('/products', (req, res) => {
  // Simulate DB query
  const delay = 10 + Math.random() * 90;
  setTimeout(() => {
    res.json({ products: [{ id: 1, name: 'Widget', price: 9.99 }] });
  }, delay);
});

app.get('/cache', (req, res) => {
  // Simulate Redis lookup
  const delay = 1 + Math.random() * 10;
  setTimeout(() => {
    res.json({ cached: true, ttl: 300 });
  }, delay);
});

app.post('/orders', (req, res) => {
  // Simulate order creation
  const delay = 20 + Math.random() * 50;
  setTimeout(() => {
    res.json({ order_id: Math.floor(Math.random() * 100000), status: 'queued' });
  }, delay);
});

app.get('/health', (req, res) => {
  res.json({ status: 'healthy', uptime: process.uptime() });
});

app.get('/metrics', async (req, res) => {
  res.set('Content-Type', register.contentType);
  res.end(await register.metrics());
});

app.listen(3000, () => console.log('API server running on port 3000'));
```

### `apps/api/package.json`

```json
{
  "name": "ecommerce-api",
  "version": "1.0.0",
  "dependencies": {
    "express": "^4.18.2",
    "prom-client": "^15.0.0"
  }
}
```

### `apps/api/Dockerfile`

```dockerfile
FROM node:20-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --production
COPY server.js .
EXPOSE 3000
CMD ["node", "server.js"]
```

### Worker: `apps/worker/worker.py`

```python
import time
import random
import threading
from flask import Flask, jsonify
from prometheus_client import (
    Counter, Histogram, Gauge, generate_latest, CONTENT_TYPE_LATEST
)
from werkzeug.middleware.dispatcher import DispatcherMiddleware
from werkzeug.wrappers import Response

app = Flask(__name__)

# Prometheus metrics
jobs_processed = Counter(
    'worker_jobs_processed_total',
    'Total jobs processed',
    ['status']
)
job_duration = Histogram(
    'worker_job_duration_seconds',
    'Job processing duration',
    buckets=[0.1, 0.5, 1, 2, 5, 10, 30]
)
queue_depth = Gauge(
    'worker_queue_depth',
    'Current queue depth'
)

def process_jobs():
    """Simulate background job processing."""
    while True:
        depth = random.randint(0, 50)
        queue_depth.set(depth)

        if depth > 0:
            with job_duration.time():
                # Simulate variable work
                work_time = random.uniform(0.05, 0.5)
                time.sleep(work_time)

                # Simulate occasional failures
                if random.random() < 0.02:
                    jobs_processed.labels(status='error').inc()
                else:
                    jobs_processed.labels(status='success').inc()
        else:
            time.sleep(1)

@app.route('/health')
def health():
    return jsonify(status='healthy', queue_depth=queue_depth._value.get())

# Prometheus metrics endpoint
app.wsgi_app = DispatcherMiddleware(app.wsgi_app, {
    '/metrics': lambda environ, start_response: Response(
        generate_latest(), content_type=CONTENT_TYPE_LATEST
    )(environ, start_response)
})

if __name__ == '__main__':
    t = threading.Thread(target=process_jobs, daemon=True)
    t.start()
    app.run(host='0.0.0.0', port=8000)
```

### `apps/worker/requirements.txt`

```
flask==3.0.0
prometheus-client==0.19.0
werkzeug==3.0.0
```

### `apps/worker/Dockerfile`

```dockerfile
FROM python:3.12-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY worker.py .
EXPOSE 8000
CMD ["python", "worker.py"]
```

---

## Part C: Monitoring stack configuration

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
    scrape_interval: 15s
    static_configs:
      - targets: ['cadvisor:8080']

  - job_name: 'prometheus'
    scrape_interval: 30s
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'api'
    scrape_interval: 10s
    static_configs:
      - targets: ['api:3000']
    metrics_path: /metrics

  - job_name: 'worker'
    scrape_interval: 10s
    static_configs:
      - targets: ['worker:8000']
    metrics_path: /metrics
```

### `alerts.yml`

```yaml
groups:
  - name: resource_alerts
    rules:
      - alert: ContainerHighCPU
        expr: rate(container_cpu_usage_seconds_total{name=~".+",name!~".*cadvisor.*",name!~".*prometheus.*",name!~".*alertmanager.*",name!~".*webhook.*"}[5m]) * 100 > 80
        for: 5m
        labels:
          severity: warning
          team: platform
        annotations:
          summary: "Container {{ $labels.name }} CPU above 80%"
          description: "{{ $labels.name }} at {{ printf \"%.1f\" $value }}% CPU for 5m."

      - alert: ContainerHighMemory
        expr: >
          container_memory_usage_bytes{name=~".+",name!~".*cadvisor.*",name!~".*prometheus.*",name!~".*alertmanager.*",name!~".*webhook.*"}
          / container_spec_memory_limit_bytes{name=~".+",name!~".*cadvisor.*",name!~".*prometheus.*",name!~".*alertmanager.*",name!~".*webhook.*"}
          * 100 > 85
        for: 2m
        labels:
          severity: critical
          team: platform
        annotations:
          summary: "Container {{ $labels.name }} memory above 85% of limit"
          description: "{{ $labels.name }} at {{ printf \"%.1f\" $value }}% memory. OOM risk."

      - alert: ContainerRestartLoop
        expr: increase(container_restart_count{name=~".+",name!~".*cadvisor.*",name!~".*prometheus.*",name!~".*alertmanager.*",name!~".*webhook.*"}[15m]) > 3
        labels:
          severity: critical
          team: platform
        annotations:
          summary: "Container {{ $labels.name }} restarting frequently"
          description: "{{ $labels.name }} restarted {{ printf \"%.0f\" $value }} times in 15m."

      - alert: ContainerDiskIOSaturation
        expr: rate(container_fs_io_time_seconds_total{name=~".+",name!~".*cadvisor.*",name!~".*prometheus.*"}[5m]) > 0.5
        for: 5m
        labels:
          severity: warning
          team: platform
        annotations:
          summary: "Container {{ $labels.name }} high disk I/O wait"
          description: "{{ $labels.name }} spending {{ printf \"%.2f\" $value }}s/s on I/O."

      - alert: ContainerNetworkSaturation
        expr: rate(container_network_transmit_packets_dropped_total{name=~".+",name!~".*cadvisor.*",name!~".*prometheus.*"}[5m]) > 100
        for: 5m
        labels:
          severity: warning
          team: platform
        annotations:
          summary: "Container {{ $labels.name }} dropping packets"
          description: "{{ $labels.name }} dropping {{ printf \"%.0f\" $value }} packets/sec."

  - name: application_alerts
    rules:
      - alert: APIHighErrorRate
        expr: >
          sum(rate(api_http_requests_total{status=~"5.."}[3m]))
          / sum(rate(api_http_requests_total[3m])) * 100 > 5
        for: 3m
        labels:
          severity: critical
          team: backend
        annotations:
          summary: "API error rate above 5%"
          description: "API error rate is {{ printf \"%.1f\" $value }}% for 3 minutes."

      - alert: APIHighLatency
        expr: histogram_quantile(0.99, sum(rate(api_http_request_duration_seconds_bucket[5m])) by (le)) > 2
        for: 5m
        labels:
          severity: warning
          team: backend
        annotations:
          summary: "API p99 latency above 2 seconds"
          description: "API p99 latency is {{ printf \"%.2f\" $value }}s."

      - alert: WorkerHighFailureRate
        expr: >
          sum(rate(worker_jobs_processed_total{status="error"}[5m]))
          / sum(rate(worker_jobs_processed_total[5m])) * 100 > 10
        for: 5m
        labels:
          severity: critical
          team: backend
        annotations:
          summary: "Worker job failure rate above 10%"
          description: "Worker failure rate is {{ printf \"%.1f\" $value }}%."

      - alert: WorkerQueueBacklog
        expr: worker_queue_depth > 1000
        for: 10m
        labels:
          severity: warning
          team: backend
        annotations:
          summary: "Worker queue depth above 1000"
          description: "Queue depth is {{ printf \"%.0f\" $value }} for 10 minutes."

      - alert: ServiceDown
        expr: up{job=~"api|worker"} == 0
        for: 2m
        labels:
          severity: critical
          team: platform
        annotations:
          summary: "Service {{ $labels.job }} is down"
          description: "{{ $labels.job }} has been unreachable for 2 minutes."

      - alert: APINoTraffic
        expr: sum(rate(api_http_requests_total[5m])) == 0
        for: 5m
        labels:
          severity: critical
          team: backend
        annotations:
          summary: "API receiving zero requests"
          description: "No requests received for 5 minutes. Possible upstream failure."

  - name: slo_alerts
    rules:
      - alert: SLOAvailabilityBreach
        expr: >
          1 - (
            sum(rate(api_http_requests_total{status=~"5.."}[30m]))
            / sum(rate(api_http_requests_total[30m]))
          ) < 0.999
        for: 5m
        labels:
          severity: critical
          team: platform
          slo: availability
        annotations:
          summary: "API availability SLO breached"
          description: "API availability is {{ printf \"%.3f\" $value }} (SLO: 0.999)."

      - alert: SLOLatencyBreach
        expr: histogram_quantile(0.95, sum(rate(api_http_request_duration_seconds_bucket[15m])) by (le)) > 1
        for: 5m
        labels:
          severity: warning
          team: backend
          slo: latency
        annotations:
          summary: "API p95 latency SLO breached"
          description: "API p95 latency is {{ printf \"%.2f\" $value }}s (SLO: <1s)."

      - alert: SLOThroughputDrop
        expr: >
          sum(rate(api_http_requests_total{route="/orders"}[5m]))
          < sum(rate(api_http_requests_total{route="/orders"}[1h] offset 1h)) * 0.5
        for: 10m
        labels:
          severity: warning
          team: backend
          slo: throughput
        annotations:
          summary: "Order throughput dropped >50% from 1 hour ago"
          description: "Current order rate is less than half of what it was an hour ago."
```

### `alertmanager.yml`

```yaml
global:
  resolve_timeout: 5m

route:
  receiver: 'info-webhook'
  group_by: ['name', 'job']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  routes:
    - match:
        severity: critical
      receiver: 'critical-webhook'
      group_wait: 15s
      repeat_interval: 1h
    - match:
        severity: warning
      receiver: 'warning-webhook'
      repeat_interval: 4h

inhibit_rules:
  - source_match:
      severity: 'critical'
    target_match:
      severity: 'warning'
    equal: ['name']

receivers:
  - name: 'critical-webhook'
    webhook_configs:
      - url: 'http://webhook-receiver:5001/webhook'
        send_resolved: true

  - name: 'warning-webhook'
    webhook_configs:
      - url: 'http://webhook-receiver:5001/webhook'
        send_resolved: true

  - name: 'info-webhook'
    webhook_configs:
      - url: 'http://webhook-receiver:5001/webhook'
        send_resolved: true
```

---

## Part E: Operational runbooks

### Runbook: ContainerHighMemory

**Summary:** A container is using more than 85% of its configured memory limit and is at risk of being OOM-killed by the Linux kernel.

**Impact:** If the container is OOM-killed, it restarts. Any in-flight requests are lost. Users experience errors or timeouts. If the container enters a restart loop, the service is effectively down.

**Diagnosis Steps:**
1. Check Grafana dashboard for memory usage trend -- is it growing steadily (leak) or spiking (load burst)?
2. Run `docker stats <container>` to see current memory usage.
3. Run `docker inspect <container> | grep -i oom` to check if it was OOM-killed.
4. Check application logs: `docker logs <container> --tail 100`.
5. If memory is growing steadily, check for memory leaks in the application code.
6. If memory spiked suddenly, check for unusual traffic or large batch jobs.

**Resolution Steps:**
1. If the spike is transient (e.g., a batch job), wait for it to pass. The alert will auto-resolve.
2. If memory is growing steadily, restart the container: `docker compose restart <service>`.
3. If the limit is too low, increase it in `docker-compose.yml` and redeploy.
4. If there is a memory leak, file a bug and deploy a fix.

**Escalation:** If the container OOM-kills 3+ times in 15 minutes (restart loop alert), escalate to the on-call engineer immediately.

---

### Runbook: APIHighErrorRate

**Summary:** The API is returning 5xx errors at a rate above 5% for more than 3 minutes.

**Impact:** Users are experiencing errors when making requests. Depending on the error rate, the service may be effectively unusable.

**Diagnosis Steps:**
1. Check Grafana dashboard -- which routes are returning errors?
2. Check application logs: `docker logs <api-container> --tail 200`.
3. Check if downstream services (PostgreSQL, Redis) are healthy.
4. Check container resource usage -- is the API container CPU/memory exhausted?
5. Check if there was a recent deployment that introduced a bug.

**Resolution Steps:**
1. If the error is caused by resource exhaustion, scale up the container limits or add replicas.
2. If downstream services are down, fix them first.
3. If a recent deployment introduced the bug, roll back: `docker compose up -d --build api` with the previous image.
4. If the error is a code bug, hotfix and redeploy.

**Escalation:** If error rate exceeds 20% or the service is completely down, escalate immediately.

---

### Runbook: ContainerRestartLoop

**Summary:** A container has restarted more than 3 times in 15 minutes. The container is crash-looping.

**Impact:** The service is unavailable or severely degraded. Users are experiencing errors.

**Diagnosis Steps:**
1. Check container logs for the crash reason: `docker logs <container> --tail 100`.
2. Check if the container was OOM-killed: `docker inspect <container> | grep OOMKilled`.
3. Check kernel logs: `dmesg | grep -i oom`.
4. Check if the container's entrypoint/command is failing (misconfiguration, missing files, bad environment variables).
5. Check if a dependency service (database, cache) is unreachable.

**Resolution Steps:**
1. If OOM-killed: increase memory limit in `docker-compose.yml`.
2. If misconfiguration: fix the environment variables or configuration files and redeploy.
3. If dependency is down: restart the dependency first, then restart this container.
4. If a bad deployment: roll back to the previous working image.

**Escalation:** This is already a critical alert. If you cannot resolve within 15 minutes, escalate to the senior on-call engineer.

---

## Part F: Testing

### Start the stack

```bash
cd production-monitoring
docker compose up -d --build
```

### Verify services

```bash
docker compose ps
# All 8+ services should show "Up" status
```

### Verify Prometheus targets

```bash
curl -s http://localhost:9090/api/v1/targets | python3 -c "
import json, sys
data = json.load(sys.stdin)
for target in data['data']['activeTargets']:
    print(f\"{target['labels']['job']:20s} {target['health']}\")
"
```

Expected output:

```
cadvisor               up
prometheus             up
api                    up
worker                 up
```

### Verify Grafana dashboards

```bash
curl -s -u admin:admin http://localhost:3001/api/search | python3 -c "
import json, sys
for d in json.load(sys.stdin):
    print(f\"  - {d['title']}\")
"
```

Expected: Three dashboards listed (Executive Overview, Service Deep Dive, Infrastructure).

### Generate load

```bash
for i in $(seq 1 500); do
  curl -s http://localhost:8080/ > /dev/null &
  curl -s http://localhost:8080/products > /dev/null &
  curl -s http://localhost:8080/cache > /dev/null &
  if [ $((i % 50)) -eq 0 ]; then
    echo "Sent $i batches"
    sleep 1
  fi
done
wait
```

### Trigger an alert

Stress the API to trigger `ContainerHighMemory`:

```bash
# The API uses ~50MB baseline. With the 512MB limit, we need sustained load.
# Use a tool like hey or wrk for sustained load:
# hey -z 5m -c 100 http://localhost:8080/products
```

Or add a memory stress endpoint to the API and call it directly.

---

## Key insights

### Three levels of alerting

1. **Resource alerts** (container-level): CPU, memory, disk, network. These detect infrastructure problems. They answer: "Is the container healthy?"

2. **Application alerts** (service-level): Error rates, latency, queue depth, throughput. These detect application problems. They answer: "Is the service working correctly for users?"

3. **SLO alerts** (business-level): Availability percentage, latency percentiles, throughput changes. These detect business impact. They answer: "Are we meeting our commitments to users?"

A container can have high CPU (resource alert) without impacting users. A service can have no resource alerts but high error rates (application alert). SLO alerts are the ultimate truth -- they measure what users experience.

### Inhibition rules prevent noise

When a `ContainerHighMemory` critical alert is firing for a container, the `ContainerHighCPU` warning alert for the same container is suppressed. This is because if the container is about to be OOM-killed, CPU usage is a secondary concern. The inhibition rule in Alertmanager handles this automatically.

### Dashboard hierarchy

- **Executive Overview** is for quick health checks. If everything is green, you do not need to look further.
- **Service Deep Dive** is for troubleshooting. When the overview shows a problem, drill into the specific service.
- **Infrastructure** is for capacity planning and root cause analysis. When a service has resource issues, check the infrastructure view.
