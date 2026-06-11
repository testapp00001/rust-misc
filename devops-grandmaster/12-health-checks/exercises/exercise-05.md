# Exercise 05: Health Checks Integrated with Monitoring

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Integrate application health checks with a monitoring stack (Prometheus +
Grafana). Expose health check metrics in Prometheus format, build a Grafana
dashboard that visualizes health status and dependency latency, and
configure alerts that fire when health degrades.

## Background

Health check endpoints give the orchestrator a binary signal: healthy or
unhealthy. Monitoring systems need richer data -- historical trends, latency
percentiles, dependency-level breakdowns. By exposing health check results
as Prometheus metrics, you bridge the gap between "is the container alive?"
(orchestrator concern) and "how is the service performing over time?"
(operational concern).

---

## Instructions

### Step 1: Create the Project Structure

```
health-monitoring/
  docker-compose.yml
  prometheus/
    prometheus.yml
    alert-rules.yml
  grafana/
    provisioning/
      datasources/
        datasource.yml
      dashboards/
        dashboard.yml
        health-dashboard.json
  app/
    Dockerfile
    app.py
    requirements.txt
```

### Step 2: Create the Application with Prometheus Metrics

```python
# app/app.py
from flask import Flask, jsonify, Response
import time
import os
import random
import threading

app = Flask(__name__)

start_time = time.time()

# Simulated dependency states
dependencies = {
    'database': 'healthy',
    'cache': 'healthy',
    'message_queue': 'healthy',
}

# Metrics storage (simple in-memory counters for this exercise)
metrics = {
    'health_checks_total': 0,
    'health_checks_failed': 0,
    'readyz_checks_total': 0,
    'readyz_checks_failed': 0,
    'dependency_latency': {},  # {dependency: last_latency_ms}
    'dependency_status': {},   # {dependency: "healthy"|"degraded"|"down"}
}

metrics_lock = threading.Lock()


def check_dependency(name, timeout=3):
    """Check a dependency and record metrics."""
    state = dependencies.get(name, 'down')

    # Simulate latency
    if state == 'healthy':
        latency = random.uniform(1, 20)
    elif state == 'degraded':
        latency = random.uniform(100, 500)
    else:
        latency = timeout * 1000

    with metrics_lock:
        metrics['dependency_latency'][name] = round(latency, 1)
        metrics['dependency_status'][name] = state

    return {
        'status': state,
        'latency_ms': round(latency, 1),
    }


@app.route('/healthz')
def healthz():
    """Liveness probe."""
    with metrics_lock:
        metrics['health_checks_total'] += 1

    return jsonify({
        'status': 'alive',
        'uptime_seconds': round(time.time() - start_time, 1),
    })


@app.route('/readyz')
def readyz():
    """Readiness probe with dependency checks."""
    with metrics_lock:
        metrics['readyz_checks_total'] += 1

    checks = {}
    for dep in dependencies:
        checks[dep] = check_dependency(dep)

    all_healthy = all(c['status'] != 'down' for c in checks.values())
    status_code = 200 if all_healthy else 503

    if not all_healthy:
        with metrics_lock:
            metrics['readyz_checks_failed'] += 1

    return jsonify({
        'status': 'ready' if all_healthy else 'not_ready',
        'checks': checks,
    }), status_code


@app.route('/metrics')
def prometheus_metrics():
    """Expose metrics in Prometheus text format."""
    lines = []

    with metrics_lock:
        # Health check counters
        lines.append('# HELP app_health_checks_total Total liveness checks')
        lines.append('# TYPE app_health_checks_total counter')
        lines.append(f'app_health_checks_total {metrics["health_checks_total"]}')

        lines.append('# HELP app_health_checks_failed_total Failed liveness checks')
        lines.append('# TYPE app_health_checks_failed_total counter')
        lines.append(f'app_health_checks_failed_total {metrics["health_checks_failed"]}')

        lines.append('# HELP app_readyz_checks_total Total readiness checks')
        lines.append('# TYPE app_readyz_checks_total counter')
        lines.append(f'app_readyz_checks_total {metrics["readyz_checks_total"]}')

        lines.append('# HELP app_readyz_checks_failed_total Failed readiness checks')
        lines.append('# TYPE app_readyz_checks_failed_total counter')
        lines.append(f'app_readyz_checks_failed_total {metrics["readyz_checks_failed"]}')

        # Dependency latency gauge
        lines.append('# HELP app_dependency_latency_ms Dependency check latency')
        lines.append('# TYPE app_dependency_latency_ms gauge')
        for dep, latency in metrics['dependency_latency'].items():
            lines.append(f'app_dependency_latency_ms{{dependency="{dep}"}} {latency}')

        # Dependency status gauge (1=healthy, 0.5=degraded, 0=down)
        lines.append('# HELP app_dependency_status Dependency status (1=healthy, 0.5=degraded, 0=down)')
        lines.append('# TYPE app_dependency_status gauge')
        for dep, status in metrics['dependency_status'].items():
            value = {'healthy': 1, 'degraded': 0.5, 'down': 0}.get(status, 0)
            lines.append(f'app_dependency_status{{dependency="{dep}"}} {value}')

        # Uptime
        lines.append('# HELP app_uptime_seconds Application uptime')
        lines.append('# TYPE app_uptime_seconds gauge')
        lines.append(f'app_uptime_seconds {round(time.time() - start_time, 1)}')

    return Response('\n'.join(lines) + '\n', mimetype='text/plain')


# Toggle endpoints for testing
@app.route('/set/<dependency>/<state>')
def set_dependency(dependency, state):
    if dependency not in dependencies:
        return jsonify({'error': f'Unknown dependency: {dependency}'}), 404
    if state not in ('healthy', 'degraded', 'down'):
        return jsonify({'error': f'Invalid state: {state}'}), 400
    dependencies[dependency] = state
    return jsonify({dependency: state})


@app.route('/')
def index():
    return jsonify({'message': 'Hello!'})


if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

```text
# app/requirements.txt
flask==3.0.0
```

```dockerfile
# app/Dockerfile
FROM python:3.12-slim
RUN apt-get update && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY . .
EXPOSE 5000
CMD ["python", "app.py"]
```

### Step 3: Configure Prometheus

```yaml
# prometheus/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "alert-rules.yml"

scrape_configs:
  - job_name: 'health-app'
    static_configs:
      - targets: ['app:5000']
    metrics_path: /metrics
    scrape_interval: 10s
```

### Step 4: Configure Prometheus Alert Rules

Create alert rules that fire when:

1. The readiness check has been failing for more than 1 minute.
2. Any dependency has been "down" for more than 2 minutes.
3. Dependency latency exceeds 200ms for more than 5 minutes.

```yaml
# prometheus/alert-rules.yml
groups:
  - name: health_alerts
    rules:
      # TODO: Alert when readiness checks are failing
      # - name: ReadinessCheckFailing
      #   condition: rate of failed readiness checks > 0 for 1 minute
      #   severity: critical

      # TODO: Alert when a dependency is down
      # - name: DependencyDown
      #   condition: app_dependency_status == 0 for 2 minutes
      #   severity: warning
      #   labels: include the dependency name

      # TODO: Alert when dependency latency is high
      # - name: DependencyHighLatency
      #   condition: app_dependency_latency_ms > 200 for 5 minutes
      #   severity: warning
```

<details>
<summary>Hint</summary>

Prometheus alert rules use PromQL expressions:

```yaml
- alert: ReadinessCheckFailing
  expr: rate(app_readyz_checks_failed_total[1m]) > 0
  for: 1m
  labels:
    severity: critical
  annotations:
    summary: "Readiness checks are failing"

- alert: DependencyDown
  expr: app_dependency_status == 0
  for: 2m
  labels:
    severity: warning
  annotations:
    summary: "Dependency {{ $labels.dependency }} is down"

- alert: DependencyHighLatency
  expr: app_dependency_latency_ms > 200
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "Dependency {{ $labels.dependency }} latency is high"
```

</details>

### Step 5: Configure Grafana

```yaml
# grafana/provisioning/datasources/datasource.yml
apiVersion: 1
datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
```

```yaml
# grafana/provisioning/dashboards/dashboard.yml
apiVersion: 1
providers:
  - name: default
    folder: ''
    type: file
    options:
      path: /etc/grafana/provisioning/dashboards
```

Create a Grafana dashboard JSON that includes:

1. A stat panel showing overall readiness status (green/red).
2. A time series graph showing dependency latency over time.
3. A table showing current dependency status and latency.
4. A counter panel showing total health checks and failure rate.

<details>
<summary>Hint</summary>

A minimal Grafana dashboard JSON structure:

```json
{
  "dashboard": {
    "title": "Health Check Dashboard",
    "panels": [
      {
        "title": "Readiness Status",
        "type": "stat",
        "targets": [
          {
            "expr": "app_readyz_checks_total - app_readyz_checks_failed_total"
          }
        ]
      },
      {
        "title": "Dependency Latency",
        "type": "timeseries",
        "targets": [
          {
            "expr": "app_dependency_latency_ms",
            "legendFormat": "{{ dependency }}"
          }
        ]
      }
    ]
  }
}
```

For a full dashboard, you can also create it manually in Grafana at
http://localhost:3000 (default credentials: admin/admin) and export the JSON.

</details>

### Step 6: Create Docker Compose

```yaml
# docker-compose.yml
version: "3.8"

services:
  app:
    build: ./app
    ports:
      - "5000:5000"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:5000/healthz"]
      interval: 10s
      timeout: 3s
      retries: 3
      start_period: 5s

  prometheus:
    image: prom/prometheus:v2.49.0
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus/prometheus.yml:/etc/prometheus/prometheus.yml
      - ./prometheus/alert-rules.yml:/etc/prometheus/alert-rules.yml
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.retention.time=1d'
    depends_on:
      app:
        condition: service_healthy

  grafana:
    image: grafana/grafana:10.3.0
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_AUTH_ANONYMOUS_ENABLED=true
      - GF_AUTH_ANONYMOUS_ORG_ROLE=Viewer
    volumes:
      - ./grafana/provisioning:/etc/grafana/provisioning
    depends_on:
      - prometheus
```

### Step 7: Run and Verify

```bash
cd health-monitoring

# Build and start
docker compose up -d --build

# Wait for everything to be healthy
docker compose ps

# Verify Prometheus can scrape the app
curl -s http://localhost:5000/metrics

# Check Prometheus targets
curl -s http://localhost:9090/api/v1/targets | python3 -m json.tool

# Generate some traffic
for i in $(seq 1 50); do
  curl -s http://localhost:5000/healthz > /dev/null
  curl -s http://localhost:5000/readyz > /dev/null
done

# Query Prometheus for health metrics
curl -s 'http://localhost:9090/api/v1/query?query=app_dependency_status' | python3 -m json.tool
curl -s 'http://localhost:9090/api/v1/query?query=app_dependency_latency_ms' | python3 -m json.tool

# Simulate a dependency failure
curl -s http://localhost:5000/set/database/down

# Wait 2 minutes for alerts to fire
# Check Prometheus alerts
curl -s http://localhost:9090/api/v1/alerts | python3 -m json.tool

# Open Grafana at http://localhost:3000
# Login: admin / admin
# Navigate to the Health Check Dashboard
# You should see the database dependency drop to 0 and latency spike

# Restore
curl -s http://localhost:5000/set/database/healthy
```

---

## Success Criteria

- [ ] The `/metrics` endpoint returns valid Prometheus text format with
      health check counters, dependency latency, and dependency status
- [ ] Prometheus successfully scrapes the `/metrics` endpoint
- [ ] The Grafana dashboard shows dependency latency over time
- [ ] Prometheus alerts fire when a dependency is down for more than 2 minutes
- [ ] You can see the health status change in Grafana in real time when
      you toggle a dependency
- [ ] The metrics distinguish between individual dependencies (each has
      its own label)
- [ ] You can explain the difference between health checks (orchestrator
      input) and monitoring metrics (operational visibility)

## Common Mistakes to Avoid

- Making the `/metrics` endpoint slow by running dependency checks on every
  scrape -- cache the results and update them on a separate schedule
- Not labeling metrics by dependency name -- you lose the ability to
  distinguish database latency from cache latency
- Using counters for latency instead of gauges or histograms -- counters
  only go up, latency fluctuates
- Setting Prometheus scrape interval too short -- scraping every 1 second
  adds load to the application
- Not configuring alert `for` duration -- instant alerts fire on transient
  spikes and create alert fatigue

## What You Should Understand After This Exercise

Health checks and monitoring serve different audiences. The orchestrator
reads `/healthz` and `/readyz` to make routing and restart decisions.
Prometheus scrapes `/metrics` to build a time-series picture of system
health. Grafana visualizes those time series for human operators. Alerts
bridge the gap by notifying on-call engineers when metrics cross thresholds.
By exposing health check data as Prometheus metrics, you get historical
visibility into dependency health that a simple "healthy/unhealthy" status
cannot provide.
