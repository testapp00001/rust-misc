# Solution 04: Instrument an Application with Custom Metrics

## Application Code

```python
# app/main.py
import time
import random
from flask import Flask, request, jsonify
from prometheus_client import (
    Counter, Histogram, Gauge, generate_latest, CONTENT_TYPE_LATEST
)

app = Flask(__name__)

# -- Metric Definitions --------------------------------------------------

REQUEST_COUNT = Counter(
    'http_requests_total',
    'Total HTTP requests',
    ['method', 'endpoint', 'status']
)

REQUEST_LATENCY = Histogram(
    'http_request_duration_seconds',
    'HTTP request latency in seconds',
    ['method', 'endpoint'],
    buckets=[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
)

IN_FLIGHT = Gauge(
    'http_in_flight_requests',
    'Number of HTTP requests currently being processed'
)

# -- Instrumentation Hooks ------------------------------------------------

@app.before_request
def before_request():
    IN_FLIGHT.inc()
    request._start_time = time.time()

@app.after_request
def after_request(response):
    IN_FLIGHT.dec()
    latency = time.time() - request._start_time
    REQUEST_COUNT.labels(
        method=request.method,
        endpoint=request.path,
        status=str(response.status_code)
    ).inc()
    REQUEST_LATENCY.labels(
        method=request.method,
        endpoint=request.path
    ).observe(latency)
    return response

# -- Application Endpoints ------------------------------------------------

@app.route('/api/users')
def get_users():
    time.sleep(random.uniform(0.01, 0.1))
    if random.random() < 0.05:
        return jsonify({"error": "Internal Server Error"}), 500
    return jsonify({"users": [{"id": 1, "name": "Alice"}]})

@app.route('/api/orders')
def get_orders():
    time.sleep(random.uniform(0.02, 0.2))
    if random.random() < 0.02:
        return jsonify({"error": "Service Unavailable"}), 503
    return jsonify({"orders": [{"id": 1, "total": 99.99}]})

@app.route('/health')
def health():
    return jsonify({"status": "healthy"})

@app.route('/metrics')
def metrics():
    return generate_latest(), 200, {'Content-Type': CONTENT_TYPE_LATEST}

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=8080)
```

## Dockerfile

```dockerfile
# app/Dockerfile
FROM python:3.12-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY main.py .

EXPOSE 8080

CMD ["python", "main.py"]
```

```
# app/requirements.txt
flask==3.0.0
prometheus_client==0.19.0
```

## Docker Compose

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
      - ./prometheus/rules:/etc/prometheus/rules
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.retention.time=7d'
      - '--web.enable-lifecycle'
    restart: unless-stopped

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - ./grafana/provisioning:/etc/grafana/provisioning
    depends_on:
      - prometheus
    restart: unless-stopped

  node-exporter:
    image: prom/node-exporter:latest
    ports:
      - "9100:9100"
    volumes:
      - /proc:/host/proc:ro
      - /sys:/host/sys:ro
      - /:/rootfs:ro
    command:
      - '--path.procfs=/host/proc'
      - '--path.sysfs=/host/sys'
      - '--path.rootfs=/rootfs'
    restart: unless-stopped

  app:
    build: ./app
    ports:
      - "8080:8080"
    restart: unless-stopped
```

## Prometheus Configuration

```yaml
# prometheus/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - /etc/prometheus/rules/*.yml

scrape_configs:
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']

  - job_name: 'sample-app'
    static_configs:
      - targets: ['app:8080']
```

## Alerting Rules

```yaml
# prometheus/rules/app-alerts.yml
groups:
  - name: app-alerts
    rules:
      - alert: HighErrorRate
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[5m]))
          / sum(rate(http_requests_total[5m]))
          > 0.1
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Error rate above 10%"
          description: "Current error rate: {{ $value | humanizePercentage }}"

      - alert: HighLatency
        expr: |
          histogram_quantile(0.99,
            sum by (le) (rate(http_request_duration_seconds_bucket[5m]))
          ) > 1.0
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "p99 latency above 1 second"
          description: "Current p99 latency: {{ $value }}s"

      - alert: InstanceDown
        expr: up{job="sample-app"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Application instance is down"
          description: "The sample-app target is unreachable."
```

## Grafana Dashboard Queries

**Rate panel (Stat):**

```promql
sum(rate(http_requests_total[5m]))
```

**Error rate panel (Gauge):**

```promql
sum(rate(http_requests_total{status=~"5.."}[5m]))
/ sum(rate(http_requests_total[5m]))
* 100
```

**Latency panel (Time series, three queries):**

```promql
# Legend: p50
histogram_quantile(0.50, sum by (le) (rate(http_request_duration_seconds_bucket[5m])))

# Legend: p95
histogram_quantile(0.95, sum by (le) (rate(http_request_duration_seconds_bucket[5m])))

# Legend: p99
histogram_quantile(0.99, sum by (le) (rate(http_request_duration_seconds_bucket[5m])))
```

## Why It Works

### before_request / after_request hooks

Flask's `before_request` runs before every request handler. `after_request`
runs after. By storing the start time in `before_request` and computing the
duration in `after_request`, every endpoint is automatically instrumented
without modifying each route function. This pattern ensures no request is
missed.

### Gauge for in-flight requests

`IN_FLIGHT.inc()` in `before_request` and `IN_FLIGHT.dec()` in
`after_request` track the current number of concurrent requests. If the
application crashes, Prometheus reads the last value. If Prometheus scrapes
during a burst, it sees the current concurrency. This is a Gauge because
the value goes up and down.

### Histogram buckets

The bucket boundaries `[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]`
are chosen to cover typical web API latencies:
- 5ms to 10ms: fast cached responses
- 50ms to 100ms: normal database queries
- 250ms to 1s: slow queries or external API calls
- 2.5s to 10s: timeouts and degraded performance

Too few buckets lose precision. Too many buckets increase storage and
memory. These 11 buckets are a good default for HTTP APIs.

### Error rate alerting expression

```
sum(rate(http_requests_total{status=~"5.."}[5m]))
/ sum(rate(http_requests_total[5m]))
> 0.1
```

The numerator counts 5xx requests per second. The denominator counts total
requests per second. The division gives the error rate as a fraction. The
`> 0.1` threshold means the alert fires when more than 10% of requests fail.
The `for: 2m` means the condition must persist for 2 minutes before the alert
fires, avoiding false alarms from brief spikes.

### Why rate() on Counters, not sum()

`sum()` on a raw Counter adds up all the counter values across instances.
This gives a meaningless large number. `rate()` computes the per-second
increase of each counter, which is the meaningful request rate. `sum()` is
then applied to aggregate the rates across instances.

## Common Mistakes

- **Using `str(response.status_code)` inconsistently.** The status code
  must be a string label. If you use an integer in one place and a string
  in another, Prometheus treats them as different label values, doubling
  the series count.

- **Forgetting to decrement the Gauge.** If `IN_FLIGHT.dec()` is missing
  or an exception prevents `after_request` from running, the Gauge
  increases monotonically and never returns to zero. Use try/finally or
  ensure exceptions are handled.

- **Using default Histogram buckets.** The default buckets
  (`.005, .01, .025, .05, .1, .25, .5, 1, 2.5, 5, 10`) are reasonable
  but may not match your application's latency profile. Always consider
  your actual latency distribution.

- **Exposing `/metrics` without authentication.** In production, the
  metrics endpoint may leak internal application state. Consider adding
  basic auth or network-level access control.

- **Not using `sum by (le)` in histogram_quantile.** The `le` label is
  required by `histogram_quantile` to identify bucket boundaries. If you
  aggregate away the `le` label before calling `histogram_quantile`, you
  get an error.

- **Alerting without `for` duration.** Without `for`, the alert fires
  the instant the condition is true, even if it is a momentary spike.
  Always use `for` to require the condition to persist.
