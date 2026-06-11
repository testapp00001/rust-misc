# Module 39: Metrics Collection — Prometheus, Grafana, Time-Series Data

> **Previous Module:** [38 — Kubernetes Security](../38-k8s-security/README.md)
> **Next Module:** [40 — Distributed Tracing](../40-distributed-tracing/README.md)
> **Phase:** 5 — Observability

---

## 1. The Problem

Your application is running in production. Users report it feels slow, but your CPU and memory dashboards look normal. You have no idea:

- How many requests per second your service handles
- What percentage of requests fail
- How long the 99th percentile request takes
- Whether the slowness started after a deploy, a traffic spike, or a database issue

Without metrics, you are flying blind. Logs tell you what happened to individual requests. Metrics tell you the **aggregate health** of your system at every moment. They answer: "Is it broken right now? Is it getting worse? When did it start?"

Metrics are numerical measurements collected over time. They are the foundation of observability — the first thing you check when something goes wrong, and the first thing that tells you something is going wrong before users notice.

---

## 2. The Naive Way

The most common anti-pattern is ad-hoc logging as a substitute for metrics.

### Anti-Pattern: Logging Every Request

```python
# Every request logs a line
@app.route('/api/users')
def get_users():
    start = time.time()
    result = db.query("SELECT * FROM users")
    duration = time.time() - start
    app.logger.info(f"GET /api/users took {duration:.3f}s, returned {len(result)} rows")
    return jsonify(result)
```

Problems with this approach:

- **Volume:** At 10,000 req/s, this generates 10,000 log lines per second just for one endpoint
- **No aggregation:** To find the p99 latency, you must parse millions of log lines
- **No alerting:** You cannot set a threshold alert on scattered log values
- **No visualization:** You cannot graph a log line in a dashboard
- **Cost:** Log storage is 10-100x more expensive than metric storage

### Anti-Pattern: Checking Metrics Manually

```bash
# "Monitoring" via SSH
ssh prod-server-01 "top -bn1 | head -5"
ssh prod-server-01 "ss -tlnp | grep 8080"
ssh prod-server-01 "wc -l /var/log/app.log"
```

This gives you a snapshot of one server at one moment. You have no history, no trends, no comparison, and no way to correlate across servers.

### Anti-Pattern: Cloud-Only Monitoring

Relying solely on AWS CloudWatch, GCP Cloud Monitoring, or Azure Monitor. These are useful but:

- Vendor lock-in makes migration painful
- Limited query capabilities compared to Prometheus
- High cost at scale
- No visibility into application internals

---

## 3. The Right Way

### Understanding Metric Types

Every metric in Prometheus (and most monitoring systems) is one of four types.

**Counter** — a value that only goes up (resets to zero on restart):

```
# TYPE http_requests_total counter
http_requests_total{method="GET", endpoint="/api/users", status="200"} 14523
http_requests_total{method="GET", endpoint="/api/users", status="500"} 12
```

Counters track cumulative totals: requests served, errors encountered, bytes sent, messages processed.

**Gauge** — a value that goes up and down:

```
# TYPE memory_usage_bytes gauge
memory_usage_bytes{process="api-server"} 536870912

# TYPE in_flight_requests gauge
in_flight_requests{service="api"} 47
```

Gauges track current state: memory usage, temperature, queue depth, active connections.

**Histogram** — a distribution of values, bucketed:

```
# TYPE http_request_duration_seconds histogram
http_request_duration_seconds_bucket{le="0.01"} 1200
http_request_duration_seconds_bucket{le="0.05"} 4500
http_request_duration_seconds_bucket{le="0.1"} 8900
http_request_duration_seconds_bucket{le="0.5"} 9800
http_request_duration_seconds_bucket{le="1.0"} 9950
http_request_duration_seconds_bucket{le="+Inf"} 10000
http_request_duration_seconds_sum 452.3
http_request_duration_seconds_count 10000
```

Histograms let you calculate percentiles (p50, p95, p99) across aggregated data.

**Summary** — similar to histogram but calculates quantiles on the client side:

```
# TYPE http_request_duration_seconds summary
http_request_duration_seconds{quantile="0.5"} 0.042
http_request_duration_seconds{quantile="0.9"} 0.156
http_request_duration_seconds{quantile="0.99"} 0.892
```

**When to use each:**

| Type | Use For | Example |
|------|---------|---------|
| Counter | Cumulative totals | requests, errors, bytes transferred |
| Gauge | Current values | memory, CPU, queue depth, temperature |
| Histogram | Latency/size distributions | request duration, response size |
| Summary | Latency percentiles (single instance) | request duration (client-side quantiles) |

### Prometheus Architecture

Prometheus uses a **pull model** — it scrapes metrics from targets rather than receiving pushed data.

```
                    +-----------------+
                    |   Prometheus    |
                    |     Server      |
                    |                 |
                    | - TSDB storage  |
                    | - PromQL engine |
                    | - Alert rules   |
                    +--------+--------+
                             |
            scrape (pull)    |
      +----------+-----------+-----------+----------+
      |          |           |           |          |
      v          v           v           v          v
  +------+  +------+   +--------+  +--------+  +--------+
  | node |  | node |   |  app   |  |  app   |  |  app   |
  |export|  |export|   |metric  |  |metric  |  |metric  |
  | er   |  | er   |   |endpoint|  |endpoint|  |endpoint|
  +------+  +------+   +--------+  +--------+  +--------+
  :9100     :9100       :8080/m    :8081/m    :8082/m
```

**Why pull instead of push?**

- Prometheus controls the scrape interval — no need to configure every application
- Health checking is built in — if a target stops responding, Prometheus knows immediately
- Service discovery integration — targets are discovered automatically in Kubernetes
- Simpler application code — apps just expose an HTTP endpoint

### prometheus.yml — Configuration

```yaml
global:
  scrape_interval: 15s      # How often to scrape targets
  evaluation_interval: 15s  # How often to evaluate alert rules
  scrape_timeout: 10s       # Timeout for each scrape

# Alert rules files
rule_files:
  - /etc/prometheus/rules/*.yml

# AlertManager integration
alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']

# Scrape targets
scrape_configs:
  # Prometheus self-monitoring
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  # Node Exporter for host metrics
  - job_name: 'node'
    static_configs:
      - targets:
          - 'node1:9100'
          - 'node2:9100'
          - 'node3:9100'

  # Application metrics
  - job_name: 'api-server'
    metrics_path: '/metrics'
    static_configs:
      - targets: ['api1:8080', 'api2:8080']

  # Kubernetes service discovery
  - job_name: 'kubernetes-pods'
    kubernetes_sd_configs:
      - role: pod
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
        action: keep
        regex: true
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_path]
        action: replace
        target_label: __metrics_path__
        regex: (.+)
```

### PromQL Basics

PromQL (Prometheus Query Language) is how you query metrics.

**Simple queries:**

```promql
# All time series for a metric
http_requests_total

# Filter by label
http_requests_total{method="GET", status="200"}

# Filter with regex
http_requests_total{endpoint=~"/api/.*"}

# Exclude with regex
http_requests_total{status!~"2.."}

# Count all time series
count(http_requests_total)
```

**Rate functions (counters):**

```promql
# Per-second rate of increase over the last 5 minutes
rate(http_requests_total[5m])

# Per-second rate — better for alerts (handles counter resets)
irate(http_requests_total[5m])

# Total increase over the last 1 hour
increase(http_requests_total[1h])
```

**Aggregation:**

```promql
# Total requests per second across all instances
sum(rate(http_requests_total[5m]))

# Requests per second grouped by status code
sum by (status) (rate(http_requests_total[5m]))

# Requests per second grouped by instance and method
sum by (instance, method) (rate(http_requests_total[5m]))

# Top 5 endpoints by request rate
topk(5, sum by (endpoint) (rate(http_requests_total[5m])))

# Error rate as a percentage
sum(rate(http_requests_total{status=~"5.."}[5m]))
/
sum(rate(http_requests_total[5m]))
* 100
```

**Histogram queries (latency):**

```promql
# Median (p50) request duration
histogram_quantile(0.50, rate(http_request_duration_seconds_bucket[5m]))

# 95th percentile request duration
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))

# 99th percentile, grouped by endpoint
histogram_quantile(0.99,
  sum by (endpoint, le) (rate(http_request_duration_seconds_bucket[5m]))
)

# Average request duration
rate(http_request_duration_seconds_sum[5m])
/
rate(http_request_duration_seconds_count[5m])
```

**Gauge queries:**

```promql
# Current memory usage in MB
process_resident_memory_bytes / 1024 / 1024

# Memory usage 1 hour ago
process_resident_memory_bytes offset 1h

# CPU usage percentage
rate(process_cpu_seconds_total[5m]) * 100

# Compare current to 1 week ago
process_resident_memory_bytes - process_resident_memory_bytes offset 7d
```

### Node Exporter — Host Metrics

Node Exporter exposes hardware and OS metrics from Linux hosts.

Key metrics it provides:

```
# CPU
node_cpu_seconds_total          # CPU time per mode (user, system, idle, iowait)
node_load1, node_load5, node_load15  # System load averages

# Memory
node_memory_MemTotal_bytes      # Total physical memory
node_memory_MemAvailable_bytes  # Available memory
node_memory_Buffers_bytes       # Buffer cache
node_memory_Cached_bytes        # Page cache

# Disk
node_filesystem_size_bytes      # Total filesystem size
node_filesystem_avail_bytes     # Available filesystem space
node_disk_io_time_seconds_total # Time spent doing I/O
node_disk_read_bytes_total      # Bytes read from disk
node_disk_written_bytes_total   # Bytes written to disk

# Network
node_network_receive_bytes_total    # Bytes received
node_network_transmit_bytes_total   # Bytes transmitted
node_network_receive_errs_total     # Receive errors

# System
node_boot_time_seconds          # System boot time
node_time_seconds               # Current system time
```

**Example queries:**

```promql
# CPU utilization (excluding idle)
100 - (avg by (instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)

# Memory utilization percentage
(1 - node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes) * 100

# Disk utilization percentage
(1 - node_filesystem_avail_bytes{fstype!="tmpfs"} / node_filesystem_size_bytes) * 100

# Disk I/O utilization (percentage of time disk was busy)
rate(node_disk_io_time_seconds_total[5m]) * 100

# Network bandwidth (MB/s received)
rate(node_network_receive_bytes_total{device="eth0"}[5m]) / 1024 / 1024
```

### Application Instrumentation

Instrumenting your application means adding code to expose custom metrics.

**Python (prometheus_client):**

```python
from prometheus_client import Counter, Histogram, Gauge, generate_latest
from flask import Flask, Response
import time

app = Flask(__name__)

# Define metrics
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

DB_POOL = Gauge(
    'db_connection_pool_size',
    'Database connection pool status',
    ['state']  # active, idle, waiting
)

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
        status=response.status_code
    ).inc()
    REQUEST_LATENCY.labels(
        method=request.method,
        endpoint=request.path
    ).observe(latency)
    return response

@app.route('/metrics')
def metrics():
    return Response(
        generate_latest(),
        mimetype='text/plain; version=0.0.4; charset=utf-8'
    )
```

**Go (prometheus/client_golang):**

```go
package main

import (
    "net/http"
    "time"

    "github.com/prometheus/client_golang/prometheus"
    "github.com/prometheus/client_golang/prometheus/promauto"
    "github.com/prometheus/client_golang/prometheus/promhttp"
)

var (
    httpRequestsTotal = promauto.NewCounterVec(
        prometheus.CounterOpts{
            Name: "http_requests_total",
            Help: "Total HTTP requests",
        },
        []string{"method", "endpoint", "status"},
    )

    httpRequestDuration = promauto.NewHistogramVec(
        prometheus.HistogramOpts{
            Name:    "http_request_duration_seconds",
            Help:    "HTTP request duration in seconds",
            Buckets: prometheus.DefBuckets,
        },
        []string{"method", "endpoint"},
    )
)

func instrumentHandler(next http.HandlerFunc, endpoint string) http.HandlerFunc {
    return func(w http.ResponseWriter, r *http.Request) {
        start := time.Now()
        sw := &statusWriter{ResponseWriter: w, status: 200}
        next(sw, r)
        duration := time.Since(start).Seconds()

        httpRequestsTotal.WithLabelValues(r.Method, endpoint,
            http.StatusText(sw.status)).Inc()
        httpRequestDuration.WithLabelValues(r.Method, endpoint).
            Observe(duration)
    }
}

func main() {
    http.HandleFunc("/api/users",
        instrumentHandler(handleUsers, "/api/users"))
    http.Handle("/metrics", promhttp.Handler())
    http.ListenAndServe(":8080", nil)
}
```

**Rust (prometheus crate):**

```rust
use prometheus::{
    Encoder, IntCounterVec, HistogramVec, Gauge,
    register_int_counter_vec, register_histogram_vec, register_gauge,
    TextEncoder,
};
use actix_web::{web, App, HttpServer, HttpResponse, middleware};
use std::time::Instant;

lazy_static! {
    static ref HTTP_REQUESTS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "http_requests_total",
        "Total HTTP requests",
        &["method", "endpoint", "status"]
    ).unwrap();

    static ref HTTP_REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds",
        "HTTP request duration",
        &["method", "endpoint"],
        vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]
    ).unwrap();

    static ref IN_FLIGHT: Gauge = register_gauge!(
        "http_in_flight_requests",
        "Currently processing requests"
    ).unwrap();
}

async fn metrics() -> HttpResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(buffer)
}
```

### The RED Method

The RED method, coined by Tom Wilkie, defines the three key metrics for every microservice:

| Metric | What It Measures | Type | Example |
|--------|-----------------|------|---------|
| **R**ate | Requests per second | Counter | `rate(http_requests_total[5m])` |
| **E**rrors | Error rate (percentage) | Counter | `rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m])` |
| **D**uration | Latency distribution | Histogram | `histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m]))` |

```promql
# RED Dashboard for a service

# Rate: requests per second
sum(rate(http_requests_total{service="api"}[5m]))

# Errors: error rate percentage
sum(rate(http_requests_total{service="api", status=~"5.."}[5m]))
/ sum(rate(http_requests_total{service="api"}[5m]))
* 100

# Duration: p50, p95, p99
histogram_quantile(0.50, sum by (le) (rate(http_request_duration_seconds_bucket{service="api"}[5m])))
histogram_quantile(0.95, sum by (le) (rate(http_request_duration_seconds_bucket{service="api"}[5m])))
histogram_quantile(0.99, sum by (le) (rate(http_request_duration_seconds_bucket{service="api"}[5m])))
```

### The USE Method

Brendan Gregg's USE method applies to **infrastructure resources**:

| Metric | What It Measures | Example |
|--------|-----------------|---------|
| **U**tilization | Percentage of time the resource is busy | CPU: `rate(node_cpu_seconds_total{mode!="idle"}[5m])` |
| **S**aturation | Degree of queued work | CPU: `node_load1 / count(node_cpu_seconds_total{mode="idle"}) by (instance)` |
| **E**rrors | Error count | Disk: `node_disk_io_errors_total` |

```promql
# USE Dashboard for CPU

# Utilization: CPU busy percentage
100 - (avg by (instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)

# Saturation: load average per CPU
node_load1 / count by (instance) (node_cpu_seconds_total{mode="idle"})

# Errors: context switches (high rate can indicate issues)
rate(node_context_switches_total[5m])

# USE Dashboard for Memory

# Utilization: used memory percentage
(1 - node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes) * 100

# Saturation: pages swapped in/out per second
rate(node_vmstat_pswpin[5m]) + rate(node_vmstat_pswpout[5m])

# Errors: OOM killer invocations
increase(node_vmstat_oom_kill[5m])

# USE Dashboard for Disk

# Utilization: disk I/O time percentage
rate(node_disk_io_time_seconds_total[5m]) * 100

# Saturation: average queue length
rate(node_disk_io_time_weighted_seconds_total[5m])

# Errors: I/O errors
node_disk_io_errors_total
```

### Alerting Rules

Prometheus alerting rules are defined in YAML files and evaluated by the Prometheus server.

```yaml
groups:
  - name: api-alerts
    rules:
      # High error rate
      - alert: HighErrorRate
        expr: |
          sum(rate(http_requests_total{service="api", status=~"5.."}[5m]))
          / sum(rate(http_requests_total{service="api"}[5m]))
          > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate on {{ $labels.service }}"
          description: "Error rate is {{ $value | humanizePercentage }} (threshold: 5%)"
          runbook_url: "https://runbooks.example.com/high-error-rate"

      # High latency
      - alert: HighLatency
        expr: |
          histogram_quantile(0.99,
            sum by (le) (rate(http_request_duration_seconds_bucket{service="api"}[5m]))
          ) > 1.0
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High p99 latency on {{ $labels.service }}"
          description: "p99 latency is {{ $value }}s (threshold: 1s)"

      # Instance down
      - alert: InstanceDown
        expr: up{job="api-server"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Instance {{ $labels.instance }} is down"

  - name: infrastructure-alerts
    rules:
      - alert: HighCPU
        expr: |
          100 - (avg by (instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100) > 90
        for: 15m
        labels:
          severity: warning

      - alert: HighMemory
        expr: |
          (1 - node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes) * 100 > 90
        for: 5m
        labels:
          severity: warning

      - alert: DiskSpaceLow
        expr: |
          (node_filesystem_avail_bytes{fstype!="tmpfs"} / node_filesystem_size_bytes) * 100 < 10
        for: 5m
        labels:
          severity: critical
```

---

## 4. The Production Way

### Service Discovery in Kubernetes

Do not hardcode scrape targets. Use Kubernetes service discovery.

```yaml
# prometheus.yml for Kubernetes
scrape_configs:
  # Scrape all pods with prometheus.io/scrape annotation
  - job_name: 'kubernetes-pods'
    kubernetes_sd_configs:
      - role: pod
    relabel_configs:
      # Only scrape pods with annotation prometheus.io/scrape=true
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
        action: keep
        regex: true
      # Use custom path if specified
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_path]
        action: replace
        target_label: __metrics_path__
        regex: (.+)
      # Use custom port if specified
      - source_labels: [__address__, __meta_kubernetes_pod_annotation_prometheus_io_port]
        action: replace
        regex: ([^:]+)(?::\d+)?;(\d+)
        replacement: $1:$2
        target_label: __address__
      # Add pod metadata as labels
      - source_labels: [__meta_kubernetes_namespace]
        target_label: namespace
      - source_labels: [__meta_kubernetes_pod_name]
        target_label: pod
      - source_labels: [__meta_kubernetes_pod_label_app]
        target_label: app
```

Application deployment with metrics annotations:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-server
spec:
  template:
    metadata:
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8080"
        prometheus.io/path: "/metrics"
    spec:
      containers:
        - name: api
          image: api-server:latest
          ports:
            - containerPort: 8080
          resources:
            requests:
              memory: "256Mi"
              cpu: "250m"
            limits:
              memory: "512Mi"
              cpu: "500m"
```

### Prometheus Operator / kube-prometheus-stack

For production Kubernetes, use the kube-prometheus-stack Helm chart:

```bash
# Add the Helm repo
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo update

# Install with custom values
helm install monitoring prometheus-community/kube-prometheus-stack \
  --namespace monitoring \
  --create-namespace \
  --set prometheus.prometheusSpec.retention=30d \
  --set prometheus.prometheusSpec.storageSpec.volumeClaimTemplate.spec.storageClassName=gp3 \
  --set prometheus.prometheusSpec.storageSpec.volumeClaimTemplate.spec.resources.requests.storage=100Gi \
  --set grafana.adminPassword=changeme \
  --set alertmanager.config.global.resolve_timeout=5m
```

This installs:
- Prometheus Operator (manages Prometheus instances)
- Prometheus (time-series database)
- Alertmanager (alert routing)
- Grafana (dashboards)
- Node Exporter (host metrics)
- kube-state-metrics (Kubernetes object metrics)

### Recording Rules

Recording rules pre-compute expensive PromQL expressions and store the results as new time series.

```yaml
groups:
  - name: recording_rules
    interval: 30s
    rules:
      # Pre-compute request rate per service
      - record: service:http_requests:rate5m
        expr: sum by (service) (rate(http_requests_total[5m]))

      # Pre-compute error rate per service
      - record: service:http_errors:rate5m
        expr: |
          sum by (service) (rate(http_requests_total{status=~"5.."}[5m]))
          / sum by (service) (rate(http_requests_total[5m]))

      # Pre-compute p99 latency per service
      - record: service:http_latency:p99_5m
        expr: |
          histogram_quantile(0.99,
            sum by (service, le) (rate(http_request_duration_seconds_bucket[5m]))
          )

      # Pre-compute node CPU usage
      - record: node:cpu_usage:percent
        expr: |
          100 - (avg by (instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)

      # Pre-compute node memory usage
      - record: node:memory_usage:percent
        expr: |
          (1 - node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes) * 100
```

Use recording rules in dashboards:

```promql
# Instead of this expensive query in Grafana:
histogram_quantile(0.99, sum by (le) (rate(http_request_duration_seconds_bucket[5m])))

# Use the pre-computed recording rule:
service:http_latency:p99_5m
```

### Prometheus Storage and Retention

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

# Command-line flags
--storage.tsdb.path=/data/prometheus
--storage.tsdb.retention.time=30d
--storage.tsdb.retention.size=50GB
--storage.tsdb.wal-compression
```

For long-term storage, use Thanos or Cortex:

```
Prometheus --> Thanos Sidecar --> Object Storage (S3)
                                    |
                               Thanos Query (global view)
                                    |
                               Thanos Store (historical data)
```

### Grafana Dashboard Setup

```yaml
# docker-compose.yml for Prometheus + Grafana
version: '3.8'
services:
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - ./rules:/etc/prometheus/rules
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.retention.time=30d'
      - '--web.enable-lifecycle'

  grafana:
    image:grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_INSTALL_PLUGINS=grafana-clock-panel,grafana-piechart-panel
    volumes:
      - grafana-data:/var/lib/grafana
      - ./grafana/provisioning:/etc/grafana/provisioning
    depends_on:
      - prometheus

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

volumes:
  prometheus-data:
  grafana-data:
```

Grafana datasource provisioning:

```yaml
# grafana/provisioning/datasources/prometheus.yml
apiVersion: 1
datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: false
```

### Metric Cardinality Management

Cardinality is the number of unique time series. High cardinality kills Prometheus.

```python
# BAD: Unbounded cardinality
http_requests_total[user_id="12345", session_id="abc-def-ghi"]  # Millions of unique users

# BAD: High cardinality from status codes
http_requests_total[status="200"]  # OK
http_requests_total[status="500"]  # OK
http_requests_total[status="404"]  # OK
http_requests_total[status_code="every_possible_http_code"]  # Still OK
http_requests_total[request_id="uuid-v4"]  # CATASTROPHIC

# GOOD: Bounded, meaningful labels
http_requests_total[method="GET", endpoint="/api/users", status="200"]
http_requests_total[method="POST", endpoint="/api/orders", status="500"]
```

Check cardinality:

```promql
# Total number of active time series
prometheus_tsdb_head_series

# Top metrics by series count
topk(20, count by (__name__)({__name__=~".+"}))

# Series count for a specific metric
count(http_requests_total)
```

---

## 5. Hands-On Lab

### Lab: Prometheus + Grafana + Node Exporter + Application Metrics

**Objective:** Set up a complete metrics stack, instrument an application, build a dashboard, and configure alerts.

**Step 1: Create the project structure**

```bash
mkdir prometheus-lab && cd prometheus-lab

# Create directories
mkdir -p prometheus/rules grafana/provisioning/datasources grafana/provisioning/dashboards app
```

**Step 2: Create a sample application with metrics**

```python
# app/main.py
from flask import Flask, request, jsonify
from prometheus_client import Counter, Histogram, Gauge, generate_latest, CONTENT_TYPE_LATEST
import random
import time

app = Flask(__name__)

REQUEST_COUNT = Counter(
    'http_requests_total', 'Total HTTP requests',
    ['method', 'endpoint', 'status']
)
REQUEST_LATENCY = Histogram(
    'http_request_duration_seconds', 'Request latency',
    ['method', 'endpoint'],
    buckets=[0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]
)
IN_FLIGHT = Gauge('http_in_flight_requests', 'In-flight requests')
DB_QUERY_DURATION = Histogram(
    'db_query_duration_seconds', 'Database query duration',
    ['operation'],
    buckets=[0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]
)

@app.before_request
def before_request():
    IN_FLIGHT.inc()
    request._start_time = time.time()

@app.after_request
def after_request(response):
    IN_FLIGHT.dec()
    latency = time.time() - request._start_time
    REQUEST_COUNT.labels(request.method, request.path, str(response.status_code)).inc()
    REQUEST_LATENCY.labels(request.method, request.path).observe(latency)
    return response

@app.route('/api/users')
def get_users():
    time.sleep(random.uniform(0.01, 0.1))  # Simulate work
    if random.random() < 0.05:  # 5% error rate
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

```dockerfile
# app/Dockerfile
FROM python:3.11-slim
WORKDIR /app
RUN pip install flask prometheus_client
COPY main.py .
CMD ["python", "main.py"]
```

**Step 3: Create Prometheus configuration**

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
          histogram_quantile(0.99, sum by (le) (rate(http_request_duration_seconds_bucket[5m]))) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "p99 latency above 1 second"
```

**Step 4: Create Grafana provisioning**

```yaml
# grafana/provisioning/datasources/prometheus.yml
apiVersion: 1
datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
```

```yaml
# grafana/provisioning/dashboards/dashboards.yml
apiVersion: 1
providers:
  - name: 'default'
    orgId: 1
    folder: ''
    type: file
    disableDeletion: false
    updateIntervalSeconds: 10
    options:
      path: /etc/grafana/provisioning/dashboards
      foldersFromFilesStructure: true
```

**Step 5: Create docker-compose.yml**

```yaml
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

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - ./grafana/provisioning:/etc/grafana/provisioning

  node-exporter:
    image: prom/node-exporter:latest
    ports:
      - "9100:9100"

  app:
    build: ./app
    ports:
      - "8080:8080"
```

**Step 6: Generate traffic and explore**

```bash
# Start the stack
docker-compose up -d

# Generate traffic
for i in $(seq 1 1000); do
  curl -s http://localhost:8080/api/users > /dev/null &
  curl -s http://localhost:8080/api/orders > /dev/null &
  sleep 0.01
done
wait

# Explore Prometheus
open http://localhost:9090

# Try these PromQL queries in the Prometheus UI:
# rate(http_requests_total[5m])
# sum by (status) (rate(http_requests_total[5m]))
# histogram_quantile(0.99, sum by (le, endpoint) (rate(http_request_duration_seconds_bucket[5m])))
# 100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)

# Explore Grafana
open http://localhost:3000
# Login: admin / admin
# Add Prometheus datasource: http://prometheus:9090
# Create a dashboard with the RED metrics
```

**Step 7: Build a Grafana Dashboard**

Create panels for the RED method:

1. **Rate Panel** (Stat panel):
   ```promql
   sum(rate(http_requests_total[5m]))
   ```

2. **Error Rate Panel** (Gauge panel):
   ```promql
   sum(rate(http_requests_total{status=~"5.."}[5m]))
   / sum(rate(http_requests_total[5m]))
   * 100
   ```

3. **Latency Panel** (Time series):
   ```promql
   histogram_quantile(0.50, sum by (le) (rate(http_request_duration_seconds_bucket[5m])))
   histogram_quantile(0.95, sum by (le) (rate(http_request_duration_seconds_bucket[5m])))
   histogram_quantile(0.99, sum by (le) (rate(http_request_duration_seconds_bucket[5m])))
   ```

4. **Request Rate by Endpoint** (Pie chart):
   ```promql
   sum by (endpoint) (rate(http_requests_total[5m]))
   ```

**Expected outcome:**

- Prometheus scraping all targets every 15s
- Node Exporter reporting host metrics
- Application exposing custom metrics at /metrics
- Grafana dashboard showing RED metrics
- Alerts visible in Prometheus UI under Alerts

---

## 6. Limitation → Next Topic

Metrics give you the aggregate view: how many requests, what error rate, what latency distribution. They answer "is the system healthy?" and "when did it start degrading?"

But metrics cannot tell you **why** a specific request was slow. When a user reports "my request took 5 seconds," you need to know:

- Which services did that request touch?
- Where did it spend most of its time?
- What was the request path through your microservices?

Metrics are aggregated — they lose individual request context. To debug slow requests, you need to **follow the request across services**. You need distributed tracing.

**Next Module:** [40 — Distributed Tracing](../40-distributed-tracing/README.md) — Trace requests across microservices with Jaeger and OpenTelemetry, understand exactly where time is spent, and correlate traces with your metrics.
