# Exercise 05: Monitoring Strategy for a Production Cluster

**Type:** Integration
**Difficulty:** Advanced
**Time:** 120-150 minutes

---

## Objective

Design and implement a comprehensive monitoring strategy for a production-like multi-container application. This exercise combines everything from the previous exercises into a real-world scenario with multiple services, meaningful alerting, capacity planning, and operational runbooks.

---

## Scenario

You are the DevOps engineer for an e-commerce platform. The application consists of:

| Service | Image | Purpose | Resource Limits |
|---------|-------|---------|-----------------|
| `api` | Custom Node.js | REST API server | 512MB, 1.0 CPU |
| `worker` | Custom Python | Background job processor | 256MB, 0.5 CPU |
| `postgres` | postgres:16 | Primary database | 1GB, 1.0 CPU |
| `redis` | redis:7-alpine | Cache and session store | 256MB, 0.25 CPU |
| `nginx` | nginx:1.25-alpine | Reverse proxy | 128MB, 0.25 CPU |

Your monitoring stack: cAdvisor + Prometheus + Grafana + Alertmanager.

---

## Part A: Design the monitoring architecture

Before writing any code, answer these design questions on paper or in a text file called `monitoring-design.md`:

### 1. Metric collection strategy

For each service, list:
- Which metrics are most critical (pick top 3 per service)
- What constitutes "normal" vs "abnormal" behavior
- Which USE method dimensions (Utilization, Saturation, Errors) apply

### 2. Dashboard design

Design 3 dashboards:
- **Executive Overview**: 6-8 panels showing system health at a glance
- **Service Deep Dive**: Per-service panels for detailed troubleshooting
- **Infrastructure**: Host-level metrics (CPU, memory, disk, network)

For each panel, specify:
- Panel title
- Panel type (timeseries, gauge, stat, table, heatmap)
- The PromQL query
- Why this metric matters

### 3. Alerting philosophy

Define your alerting rules:
- What conditions warrant a page (critical)?
- What conditions warrant a warning (Slack/email)?
- What is your SLO for each service (availability %, latency p99)?
- How do you prevent alert fatigue?

---

## Part B: Build the application stack

Create `production-monitoring/` with the following structure:

```
production-monitoring/
  docker-compose.yml
  prometheus.yml
  alerts.yml
  alertmanager.yml
  grafana/
    provisioning/
      datasources/
        prometheus.yml
      dashboards/
        dashboard.yml
        executive-overview.json
        service-deep-dive.json
        infrastructure.json
  apps/
    api/
      Dockerfile
      server.js
      package.json
    worker/
      Dockerfile
      worker.py
      requirements.txt
  webhook-receiver/
    Dockerfile
    server.py
    requirements.txt
```

### Sample API server (`apps/api/server.js`)

Create a Node.js Express application that:
- Has a `GET /` endpoint returning a health check
- Has a `GET /products` endpoint that queries PostgreSQL (simulated with a delay)
- Has a `GET /cache` endpoint that queries Redis (simulated with a delay)
- Has a `POST /orders` endpoint that enqueues a background job (simulated)
- Exposes Prometheus metrics at `GET /metrics`
- Tracks: request count, request duration histogram, active connections gauge, error count

### Sample Worker (`apps/worker/worker.py`)

Create a Python worker that:
- Simulates processing background jobs from a queue
- Consumes variable amounts of CPU and memory per job
- Exposes Prometheus metrics at `GET /metrics` on port 8000
- Tracks: jobs processed count, job duration histogram, queue depth gauge, error count

### Sample Database and Cache

Use official images with resource limits:
- `postgres:16` with a simple init script
- `redis:7-alpine` with default configuration

---

## Part C: Configure the monitoring stack

### Prometheus configuration

Create `prometheus.yml` that scrapes:
- cAdvisor (container metrics)
- Prometheus itself (self-monitoring)
- API server (application metrics)
- Worker (application metrics)
- PostgreSQL exporter (optional, or rely on cAdvisor)
- Redis exporter (optional, or rely on cAdvisor)

Scrape intervals:
- cAdvisor: 15s (container metrics are cheap to collect)
- Application metrics: 10s (application metrics need finer granularity)
- Prometheus self: 30s (self-monitoring is lower priority)

### Alerting rules

Create `alerts.yml` with alerts organized into three groups:

**Group 1: Resource alerts** (container-level)
- Container CPU > 80% for 5 minutes
- Container memory > 85% of limit for 2 minutes
- Container restarting > 3 times in 15 minutes
- Container disk I/O saturation > 0.5s/s for 5 minutes
- Network packet drop rate > 100/s for 5 minutes

**Group 2: Application alerts** (service-level)
- API error rate > 5% for 3 minutes
- API p99 latency > 2 seconds for 5 minutes
- Worker job failure rate > 10% for 5 minutes
- Queue depth > 1000 for 10 minutes
- Zero requests received for 5 minutes (service down)

**Group 3: SLO alerts** (business-level)
- API availability < 99.9% over 30 minutes
- API p95 latency > 1 second over 15 minutes
- Order processing throughput drops > 50% from 1 hour ago

### Alertmanager configuration

Create `alertmanager.yml` with:
- Three receivers: `critical-webhook`, `warning-webhook`, `info-webhook`
- Routes based on severity label
- Inhibition rules: critical alerts suppress warning alerts for the same container
- Grouping by service name
- Appropriate wait times

---

## Part D: Create Grafana dashboards

### Dashboard 1: Executive Overview

6-8 panels showing:
1. **Service Health Status** (stat panel): UP/DOWN for each service
2. **Total Request Rate** (timeseries): requests/second across all services
3. **Error Rate** (timeseries): error percentage across all services
4. **Container CPU Summary** (gauge panel): CPU usage for each container
5. **Container Memory Summary** (gauge panel): Memory usage as % of limit
6. **Active Alerts** (stat panel): Count of firing alerts by severity
7. **Container Restarts** (stat panel): Total restarts across all containers
8. **SLO Compliance** (gauge): API availability percentage

### Dashboard 2: Service Deep Dive

Panels for each service with:
- CPU usage (timeseries)
- Memory usage (timeseries)
- Request rate (timeseries)
- Error rate (timeseries)
- Latency percentiles (timeseries with p50, p95, p99)
- Custom business metrics

### Dashboard 3: Infrastructure

Host-level panels:
- CPU usage per core
- Memory breakdown (used, cached, free)
- Disk I/O per container
- Network throughput per container
- Container count over time

---

## Part E: Document operational runbooks

Create `runbooks.md` with runbooks for your top 3 alerts:

### Runbook format:

```
## Alert: <AlertName>

### Summary
What this alert means and why it matters.

### Impact
What users/systems are affected.

### Diagnosis Steps
1. Step 1: ...
2. Step 2: ...
3. Step 3: ...

### Resolution Steps
1. Step 1: ...
2. Step 2: ...

### Escalation
When and whom to escalate to.
```

Write runbooks for:
1. `ContainerHighMemory`
2. `APIHighErrorRate`
3. `ContainerRestartLoop`

---

## Part F: Test the complete stack

1. Start everything:

```bash
cd production-monitoring
docker compose up -d --build
```

2. Verify all services are running:

```bash
docker compose ps
```

3. Verify all Prometheus targets are UP:

```bash
curl -s http://localhost:9090/api/v1/targets | python3 -c "
import json, sys
data = json.load(sys.stdin)
for target in data['data']['activeTargets']:
    print(f\"{target['labels']['job']}: {target['health']}\")
"
```

4. Verify Grafana dashboards are provisioned:

```bash
curl -s -u admin:admin http://localhost:3001/api/search | python3 -m json.tool
```

5. Generate load:

```bash
# Hit the API endpoints
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

6. Verify dashboards show the load.

7. Trigger an alert and verify the full pipeline (alert -> Alertmanager -> webhook).

---

## Success Criteria

- [ ] All 5 application services are running with resource limits.
- [ ] Prometheus scrapes all targets successfully (all UP).
- [ ] 3 Grafana dashboards are provisioned with meaningful panels.
- [ ] Alerting rules cover resource, application, and SLO levels.
- [ ] Alertmanager routes alerts by severity to correct receivers.
- [ ] Webhook receiver logs show alerts with correct metadata.
- [ ] `runbooks.md` contains actionable runbooks for the top 3 alerts.
- [ ] You can explain the difference between resource alerts, application alerts, and SLO alerts.

---

## Hints

<details>
<summary>Hint 1: Application metrics naming</summary>

Follow Prometheus naming conventions:
- Counters end with `_total` (e.g., `http_requests_total`)
- Histograms use `_bucket`, `_sum`, `_count` suffixes
- Use labels for dimensions: `method`, `status`, `route`
- Prefix with your app name: `api_http_requests_total`, `worker_jobs_processed_total`

</details>

<details>
<summary>Hint 2: SLO calculations in PromQL</summary>

Availability SLO (percentage of successful requests):

```promql
# 30-minute availability
1 - (
  sum(rate(http_requests_total{status=~"5.."}[30m]))
  /
  sum(rate(http_requests_total[30m]))
)
```

Latency SLO (percentage of requests under threshold):

```promql
# Percentage of requests under 1s
sum(rate(http_request_duration_seconds_bucket{le="1.0"}[15m]))
/
sum(rate(http_request_duration_seconds_count[15m]))
```

</details>

<details>
<summary>Hint 3: Inhibition rules in Alertmanager</summary>

Inhibition prevents lower-severity alerts from firing when a higher-severity alert is already active for the same target:

```yaml
inhibit_rules:
  - source_match:
      severity: 'critical'
    target_match:
      severity: 'warning'
    equal: ['name']
```

This means: if a `critical` alert is firing for container `X`, suppress `warning` alerts for the same container `X`.

</details>

<details>
<summary>Hint 4: Dashboard template variables</summary>

Use Grafana template variables to make dashboards interactive:

```json
"templating": {
  "list": [
    {
      "name": "container",
      "type": "query",
      "query": "label_values(container_memory_usage_bytes, name)",
      "datasource": "Prometheus",
      "refresh": 2,
      "includeAll": true,
      "multi": true
    }
  ]
}
```

Then reference in queries: `container_memory_usage_bytes{name=~"$container"}`

</details>

<details>
<summary>Hint 5: Multi-stage Dockerfiles for Node.js</summary>

For the API server, use a multi-stage build for smaller images:

```dockerfile
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci --production

FROM node:20-alpine
WORKDIR /app
COPY --from=builder /app/node_modules ./node_modules
COPY . .
EXPOSE 3000
CMD ["node", "server.js"]
```

</details>

<details>
<summary>Hint 6: Health check endpoints</summary>

Add a dedicated health check endpoint to each service. Prometheus can scrape these, and Docker can use them for container health checks:

```yaml
# In docker-compose.yml
services:
  api:
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 10s
```

</details>
