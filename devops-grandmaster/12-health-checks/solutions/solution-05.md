# Solution 05: Health Checks Integrated with Monitoring

## Complete Solution

### Project Structure

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

### app/app.py

```python
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

# Metrics storage
metrics = {
    'health_checks_total': 0,
    'health_checks_failed': 0,
    'readyz_checks_total': 0,
    'readyz_checks_failed': 0,
    'dependency_latency': {},
    'dependency_status': {},
}

metrics_lock = threading.Lock()


def check_dependency(name, timeout=3):
    """Check a dependency and record metrics."""
    state = dependencies.get(name, 'down')

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
        lines.append('# HELP app_health_checks_total Total liveness probe checks')
        lines.append('# TYPE app_health_checks_total counter')
        lines.append(f'app_health_checks_total {metrics["health_checks_total"]}')

        lines.append('# HELP app_health_checks_failed_total Failed liveness probe checks')
        lines.append('# TYPE app_health_checks_failed_total counter')
        lines.append(f'app_health_checks_failed_total {metrics["health_checks_failed"]}')

        lines.append('# HELP app_readyz_checks_total Total readiness probe checks')
        lines.append('# TYPE app_readyz_checks_total counter')
        lines.append(f'app_readyz_checks_total {metrics["readyz_checks_total"]}')

        lines.append('# HELP app_readyz_checks_failed_total Failed readiness probe checks')
        lines.append('# TYPE app_readyz_checks_failed_total counter')
        lines.append(f'app_readyz_checks_failed_total {metrics["readyz_checks_failed"]}')

        # Dependency latency gauge
        lines.append('# HELP app_dependency_latency_ms Dependency check latency in milliseconds')
        lines.append('# TYPE app_dependency_latency_ms gauge')
        for dep, latency in metrics['dependency_latency'].items():
            lines.append(f'app_dependency_latency_ms{{dependency="{dep}"}} {latency}')

        # Dependency status gauge
        lines.append('# HELP app_dependency_status Dependency status (1=healthy, 0.5=degraded, 0=down)')
        lines.append('# TYPE app_dependency_status gauge')
        for dep, status in metrics['dependency_status'].items():
            value = {'healthy': 1, 'degraded': 0.5, 'down': 0}.get(status, 0)
            lines.append(f'app_dependency_status{{dependency="{dep}"}} {value}')

        # Uptime
        lines.append('# HELP app_uptime_seconds Application uptime in seconds')
        lines.append('# TYPE app_uptime_seconds gauge')
        lines.append(f'app_uptime_seconds {round(time.time() - start_time, 1)}')

    return Response('\n'.join(lines) + '\n', mimetype='text/plain')


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

### app/requirements.txt

```
flask==3.0.0
```

### app/Dockerfile

```dockerfile
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

### prometheus/prometheus.yml

```yaml
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

### prometheus/alert-rules.yml

```yaml
groups:
  - name: health_alerts
    rules:
      # Alert when readiness checks are consistently failing.
      # rate() over 1 minute shows if failures are happening.
      # The "for: 1m" means the condition must be true for 1 minute
      # before the alert fires, avoiding false alarms on transient issues.
      - alert: ReadinessCheckFailing
        expr: rate(app_readyz_checks_failed_total[1m]) > 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Readiness checks are failing"
          description: "The application readiness probe has been failing for more than 1 minute. Traffic may not be reaching this instance."

      # Alert when any dependency is down.
      # app_dependency_status == 0 means the dependency is "down".
      # The "for: 2m" prevents flapping on brief network issues.
      # {{ $labels.dependency }} is a template that inserts the
      # dependency name from the metric label.
      - alert: DependencyDown
        expr: app_dependency_status == 0
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "Dependency {{ $labels.dependency }} is down"
          description: "The {{ $labels.dependency }} dependency has been down for more than 2 minutes."

      # Alert when dependency latency exceeds 200ms for 5 minutes.
      # This catches gradual degradation that does not trigger the
      # "down" alert.
      - alert: DependencyHighLatency
        expr: app_dependency_latency_ms > 200
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Dependency {{ $labels.dependency }} latency is high"
          description: "The {{ $labels.dependency }} dependency latency has been above 200ms for more than 5 minutes (current: {{ $value }}ms)."
```

### grafana/provisioning/datasources/datasource.yml

```yaml
apiVersion: 1
datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
```

### grafana/provisioning/dashboards/dashboard.yml

```yaml
apiVersion: 1
providers:
  - name: default
    folder: ''
    type: file
    options:
      path: /etc/grafana/provisioning/dashboards
```

### grafana/provisioning/dashboards/health-dashboard.json

```json
{
  "annotations": {
    "list": []
  },
  "editable": true,
  "fiscalYearStartMonth": 0,
  "graphTooltip": 1,
  "id": null,
  "links": [],
  "panels": [
    {
      "title": "Overall Status",
      "type": "stat",
      "gridPos": {"h": 4, "w": 6, "x": 0, "y": 0},
      "targets": [
        {
          "expr": "app_dependency_status",
          "legendFormat": "{{ dependency }}"
        }
      ],
      "fieldConfig": {
        "defaults": {
          "thresholds": {
            "steps": [
              {"color": "red", "value": null},
              {"color": "yellow", "value": 0.5},
              {"color": "green", "value": 1}
            ]
          },
          "mappings": [
            {"type": "value", "options": {"0": {"text": "DOWN", "color": "red"}}},
            {"type": "value", "options": {"0.5": {"text": "DEGRADED", "color": "yellow"}}},
            {"type": "value", "options": {"1": {"text": "HEALTHY", "color": "green"}}}
          ]
        }
      }
    },
    {
      "title": "Dependency Latency Over Time",
      "type": "timeseries",
      "gridPos": {"h": 8, "w": 12, "x": 0, "y": 4},
      "targets": [
        {
          "expr": "app_dependency_latency_ms",
          "legendFormat": "{{ dependency }}"
        }
      ],
      "fieldConfig": {
        "defaults": {
          "unit": "ms",
          "thresholds": {
            "steps": [
              {"color": "green", "value": null},
              {"color": "yellow", "value": 100},
              {"color": "red", "value": 500}
            ]
          }
        }
      }
    },
    {
      "title": "Readiness Check Failure Rate",
      "type": "timeseries",
      "gridPos": {"h": 8, "w": 12, "x": 12, "y": 4},
      "targets": [
        {
          "expr": "rate(app_readyz_checks_failed_total[1m])",
          "legendFormat": "failure rate"
        }
      ],
      "fieldConfig": {
        "defaults": {
          "unit": "ops"
        }
      }
    },
    {
      "title": "Dependency Status",
      "type": "table",
      "gridPos": {"h": 6, "w": 12, "x": 0, "y": 12},
      "targets": [
        {
          "expr": "app_dependency_status",
          "legendFormat": "{{ dependency }}",
          "instant": true,
          "format": "table"
        }
      ]
    },
    {
      "title": "Uptime",
      "type": "stat",
      "gridPos": {"h": 4, "w": 6, "x": 12, "y": 0},
      "targets": [
        {
          "expr": "app_uptime_seconds",
          "legendFormat": "uptime"
        }
      ],
      "fieldConfig": {
        "defaults": {
          "unit": "s"
        }
      }
    },
    {
      "title": "Total Health Checks",
      "type": "stat",
      "gridPos": {"h": 4, "w": 6, "x": 18, "y": 0},
      "targets": [
        {
          "expr": "app_readyz_checks_total",
          "legendFormat": "total"
        }
      ]
    }
  ],
  "schemaVersion": 39,
  "tags": ["health", "monitoring"],
  "templating": {
    "list": []
  },
  "time": {
    "from": "now-30m",
    "to": "now"
  },
  "title": "Health Check Dashboard",
  "uid": "health-check-dashboard",
  "version": 1
}
```

### docker-compose.yml

```yaml
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
      - ./prometheus/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - ./prometheus/alert-rules.yml:/etc/prometheus/alert-rules.yml:ro
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.retention.time=1d'
      - '--web.enable-lifecycle'
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
      - ./grafana/provisioning:/etc/grafana/provisioning:ro
    depends_on:
      - prometheus
```

### Build and Run

```bash
cd health-monitoring

docker compose up -d --build

# Wait for all services to be healthy
docker compose ps

# Verify the metrics endpoint
curl -s http://localhost:5000/metrics
# Should show Prometheus text format with all metrics

# Verify Prometheus is scraping
curl -s 'http://localhost:9090/api/v1/targets' | python3 -m json.tool
# Should show health-app target with state "up"

# Generate traffic
for i in $(seq 1 50); do
  curl -s http://localhost:5000/healthz > /dev/null
  curl -s http://localhost:5000/readyz > /dev/null
done

# Query Prometheus
curl -s 'http://localhost:9090/api/v1/query?query=app_dependency_status' | python3 -m json.tool
curl -s 'http://localhost:9090/api/v1/query?query=app_dependency_latency_ms' | python3 -m json.tool

# Simulate a dependency failure
curl -s http://localhost:5000/set/database/down

# Keep generating traffic to accumulate failure metrics
for i in $(seq 1 30); do
  curl -s http://localhost:5000/readyz > /dev/null
  sleep 2
done

# Check alerts (after ~2 minutes)
curl -s 'http://localhost:9090/api/v1/alerts' | python3 -m json.tool
# Should show DependencyDown alert as "firing" for database

# Open Grafana at http://localhost:3000
# Login: admin / admin (skip password change)
# Navigate to Dashboards > Health Check Dashboard
# You should see:
# - database dependency status drop to 0 (red)
# - latency spike for database
# - readiness failure rate increase

# Restore dependency
curl -s http://localhost:5000/set/database/healthy
```

---

## Why This Works

1. **The `/metrics` endpoint bridges health checks and monitoring.** The
   orchestrator reads `/healthz` and `/readyz` to make binary decisions
   (restart or remove from load balancer). Prometheus scrapes `/metrics` to
   build a time-series history. You get both real-time orchestration and
   historical analysis from the same application.

2. **Counters vs gauges.** Health check totals are counters (they only go
   up). Dependency latency and status are gauges (they fluctuate). This
   follows Prometheus conventions and enables correct rate calculations
   (`rate(app_readyz_checks_failed_total[1m])` gives the failure rate per
   second).

3. **Labels distinguish dependencies.** Each dependency gets its own time
   series via the `dependency` label. In Grafana, you can filter, group, and
   compare dependencies. In alerts, `{{ $labels.dependency }}` tells you
   which dependency triggered the alert.

4. **Alert rules use `for` duration.** The `for: 1m` (or 2m, 5m) prevents
   alerts from firing on transient spikes. A dependency that is down for 10
   seconds during a network blip does not page the on-call engineer. A
   dependency that is down for 2 minutes does.

5. **Grafana auto-provisioning.** The dashboard and datasource are
   provisioned from YAML files, so they are version-controlled and
   reproducible. No manual configuration in the Grafana UI.

6. **Docker Compose dependency ordering.** Prometheus waits for the app to
   be healthy before starting. Grafana waits for Prometheus. This prevents
   Grafana from showing "no data" on first load because Prometheus has not
   scraped yet.

## Common Mistakes

### Mistake 1: Running dependency checks on every /metrics scrape

```python
# WRONG: Prometheus scrapes every 10 seconds, each scrape runs all checks
@app.route('/metrics')
def metrics():
    for dep in dependencies:
        check_dependency(dep)  # Runs on every scrape
    # ...
```

This adds load to your dependencies. Instead, run checks on a separate
schedule (e.g., every 5 seconds in a background thread) and cache the
results. The `/metrics` endpoint reads the cached values.

### Mistake 2: Using counters for latency

```python
# WRONG: counters only go up, latency fluctuates
lines.append(f'app_dependency_latency_ms{{dependency="{dep}"}} {total_latency}')
```

Use gauges for values that go up and down (latency, status). Use counters
for values that only increase (total checks, total failures).

### Mistake 3: No labels on dependency metrics

```python
# WRONG: one metric for all dependencies, no way to distinguish
lines.append(f'app_dependency_status {average_status}')
```

Use labels: `app_dependency_status{dependency="database"} 1`. This lets
you query, filter, and alert on individual dependencies.

### Mistake 4: Alerting without `for` duration

```yaml
# WRONG: fires immediately on any failure
- alert: DependencyDown
  expr: app_dependency_status == 0
  # No "for" -- fires on a single scrape
```

A single failed check (network hiccup, GC pause) triggers the alert. Use
`for: 2m` to require sustained failure before alerting.

### Mistake 5: Scraping too frequently

```yaml
# WRONG: scraping every 1 second
scrape_configs:
  - job_name: 'health-app'
    scrape_interval: 1s
```

Scraping every second adds load to the application and generates a lot of
data. For health check metrics, 10-15 seconds is sufficient. The alert
`for` duration handles the latency between scrape and detection.

### Mistake 6: Not distinguishing health checks from monitoring

The `/healthz` and `/readyz` endpoints are for the orchestrator -- they
should be fast and binary. The `/metrics` endpoint is for Prometheus -- it
should expose rich data. Do not combine them. The orchestrator does not
need dependency latency percentiles, and Prometheus does not need to make
restart decisions.
