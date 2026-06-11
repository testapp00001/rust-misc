# Module 44: Dashboard Design -- What to Monitor, How to Visualize It

> **Previous Module:** [43 -- SLO Monitoring](../43-slo-monitoring/README.md)
> **Next Module:** [45 -- Database Scaling](../45-database-scaling/README.md)
> **Phase:** 5 -- Observability

---

## 1. The Problem

Your team has Prometheus collecting metrics, Grafana installed, and hundreds of time series flowing in. Someone creates a dashboard with 47 panels: CPU per core for every server, memory for every container, network I/O for every interface, disk usage for every volume, request count for every endpoint, latency for every microservice, and a few gauges thrown in for good measure.

During an incident, an engineer opens this dashboard. They stare at 47 graphs. They have no idea which one is relevant. They scroll up and down, looking for something unusual among dozens of squiggly lines. Five minutes later, they still do not know what is wrong.

The problem is not a lack of data. The problem is that the data is organized poorly. A good dashboard is a diagnostic tool. It tells a story: here is the health of your system, here is what is wrong, here is where to look next. A bad dashboard is a data dump -- it has all the information but communicates nothing.

Dashboard design is a discipline. It requires understanding what questions the dashboard must answer, who will use it, and in what context. This module teaches you how to design dashboards that are actually useful during incidents, for capacity planning, and for communicating with stakeholders.

---

## 2. The Naive Way

### Anti-Pattern: The Wall of Graphs

```json
{
  "dashboard": {
    "title": "Everything Dashboard",
    "panels": [
      {"title": "CPU Server 1 Core 0", "targets": ["node_cpu{instance='s1',cpu='0'}"]},
      {"title": "CPU Server 1 Core 1", "targets": ["node_cpu{instance='s1',cpu='1'}"]},
      {"title": "CPU Server 1 Core 2", "targets": ["node_cpu{instance='s1',cpu='2'}"]},
      {"title": "CPU Server 2 Core 0", "targets": ["node_cpu{instance='s2',cpu='0'}"]},
      "... 43 more panels ..."
    ]
  }
}
```

Problems:
- Nobody can find what they need
- Visual noise makes anomalies invisible
- No hierarchy -- everything is equally prominent
- Takes too long to load (too many queries)
- Different audiences need different views

### Anti-Pattern: Copy-Paste from the Internet

Using default Grafana dashboards (like the Node Exporter Full dashboard with 200+ panels) without understanding what each panel tells you. You end up with dashboards that look impressive but are never used because nobody understands them.

### Anti-Pattern: Dashboard per Person

Each team member creates their own dashboard with their own naming conventions, time ranges, and variables. There is no shared understanding of what "system health" means. During an incident, three people look at three different dashboards and see three different pictures.

### Anti-Pattern: No Context

```json
{
  "title": "API Latency",
  "targets": ["rate(http_request_duration_seconds_sum[5m]) / rate(http_request_duration_seconds_count[5m])"]
}
```

A line graph of average latency. Is 200ms good or bad? Is the current value normal or anomalous? Without context -- SLO lines, historical baselines, or comparison periods -- a number means nothing.

---

## 3. The Right Way

### Dashboard Hierarchy: Overview, Drill-Down, Actionable

A well-designed monitoring system has three levels of dashboards:

```
Level 1: Overview ("Is everything OK?")
+-- Single-page summary of all services
+-- Green/yellow/red status for each service
+-- SLO status and error budget
+-- Key business metrics
+-- Links to Level 2 dashboards

Level 2: Service ("What is happening with this service?")
+-- RED metrics (Rate, Errors, Duration)
+-- USE metrics (Utilization, Saturation, Errors) for dependencies
+-- SLO tracking
+-- Recent deployments overlay
+-- Links to Level 3 dashboards and logs/traces

Level 3: Deep Dive ("Why is this happening?")
+-- Detailed resource metrics
+-- Per-endpoint breakdowns
+-- Database query performance
+-- Cache hit rates
+-- Specific component internals
+-- Correlated logs and traces
```

**Example navigation:**

```
Level 1: "API Gateway Overview"
  -> Click on "Order Service" showing yellow status
  -> Level 2: "Order Service Dashboard"
    -> See that p99 latency is elevated
    -> Click on latency panel
    -> Level 3: "Order Service Deep Dive"
      -> See that database queries are slow
      -> Click through to logs: "show me slow DB queries"
      -> Click through to traces: "show me a slow request trace"
```

### The Four Golden Signals (Google SRE)

For any distributed system, monitor these four signals:

| Signal | What It Measures | How to Measure |
|--------|-----------------|----------------|
| **Latency** | Time to serve a request | Histogram of request duration |
| **Traffic** | Demand on the system | Requests per second |
| **Errors** | Rate of failed requests | Count of 5xx responses / total |
| **Saturation** | How "full" the system is | CPU, memory, queue depth, connection pools |

**Latency:**

```promql
# p50, p95, p99 latency
histogram_quantile(0.50, sum by (le) (rate(http_request_duration_seconds_bucket[5m])))
histogram_quantile(0.95, sum by (le) (rate(http_request_duration_seconds_bucket[5m])))
histogram_quantile(0.99, sum by (le) (rate(http_request_duration_seconds_bucket[5m])))

# Separate successful vs failed request latency
histogram_quantile(0.99, sum by (le) (rate(http_request_duration_seconds_bucket{status!~"5.."}[5m])))
histogram_quantile(0.99, sum by (le) (rate(http_request_duration_seconds_bucket{status=~"5.."}[5m])))
```

**Traffic:**

```promql
# Requests per second
sum(rate(http_requests_total[5m]))

# Requests per second by endpoint
sum by (endpoint) (rate(http_requests_total[5m]))

# Requests per second by status code family
sum by (status_family) (label_replace(
  rate(http_requests_total[5m]),
  "status_family",
  "$1xx",
  "status",
  "(.).*"
))
```

**Errors:**

```promql
# Error rate (percentage)
sum(rate(http_requests_total{status=~"5.."}[5m]))
/ sum(rate(http_requests_total[5m]))
* 100

# Error rate by endpoint
sum by (endpoint) (rate(http_requests_total{status=~"5.."}[5m]))
/ sum by (endpoint) (rate(http_requests_total[5m]))
* 100

# Error count (for SLO calculations)
sum(increase(http_requests_total{status=~"5.."}[30d]))
```

**Saturation:**

```promql
# CPU saturation (load average / CPU count)
node_load1 / count by (instance) (node_cpu_seconds_total{mode="idle"})

# Memory saturation (used / total)
1 - node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes

# Disk I/O saturation
rate(node_disk_io_time_weighted_seconds_total[5m])

# Connection pool saturation
db_connections_active / db_connections_max

# Queue saturation
queue_depth / queue_capacity
```

### USE Method Dashboards (Infrastructure)

For every hardware resource, monitor Utilization, Saturation, and Errors:

**CPU USE Dashboard:**

```promql
# Utilization: percentage of time CPU was busy
100 - avg by (instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100

# Saturation: run queue length (load average per CPU)
node_load1 / count by (instance) (node_cpu_seconds_total{mode="idle"})
node_load5 / count by (instance) (node_cpu_seconds_total{mode="idle"})
node_load15 / count by (instance) (node_cpu_seconds_total{mode="idle"})

# Errors: context switch rate (high rate can indicate issues)
rate(node_context_switches_total[5m])
```

**Memory USE Dashboard:**

```promql
# Utilization: used memory
(1 - node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes) * 100

# Saturation: swap usage
node_memory_SwapTotal_bytes - node_memory_SwapFree_bytes
# Or: pages swapped per second
rate(node_vmstat_pswpin[5m]) + rate(node_vmstat_pswpout[5m])

# Errors: OOM killer events
increase(node_vmstat_oom_kill[5m])
```

**Disk USE Dashboard:**

```promql
# Utilization: disk space used
(1 - node_filesystem_avail_bytes{fstype!="tmpfs"} / node_filesystem_size_bytes) * 100

# Utilization: disk I/O time
rate(node_disk_io_time_seconds_total[5m]) * 100

# Saturation: queue length
rate(node_disk_io_time_weighted_seconds_total[5m])

# Errors: disk I/O errors
rate(node_disk_io_errors_total[5m])
```

**Network USE Dashboard:**

```promql
# Utilization: bandwidth used (as percentage of link speed)
rate(node_network_receive_bytes_total{device="eth0"}[5m]) * 8
  / node_network_speed_bytes{device="eth0"} * 100

# Saturation: dropped packets
rate(node_network_receive_drop_total{device="eth0"}[5m])
rate(node_network_transmit_drop_total{device="eth0"}[5m])

# Errors: network errors
rate(node_network_receive_errs_total{device="eth0"}[5m])
rate(node_network_transmit_errs_total{device="eth0"}[5m])
```

### RED Method Dashboards (Services)

For every microservice, monitor Rate, Errors, and Duration:

```promql
# Rate: requests per second
sum(rate(http_requests_total{service="order-service"}[5m]))

# Errors: error rate percentage
sum(rate(http_requests_total{service="order-service", status=~"5.."}[5m]))
/ sum(rate(http_requests_total{service="order-service"}[5m]))
* 100

# Duration: latency percentiles
histogram_quantile(0.50, sum by (le) (
  rate(http_request_duration_seconds_bucket{service="order-service"}[5m])
))
histogram_quantile(0.95, sum by (le) (
  rate(http_request_duration_seconds_bucket{service="order-service"}[5m])
))
histogram_quantile(0.99, sum by (le) (
  rate(http_request_duration_seconds_bucket{service="order-service"}[5m])
))

# Duration: average
sum(rate(http_request_duration_seconds_sum{service="order-service"}[5m]))
/ sum(rate(http_request_duration_seconds_count{service="order-service"}[5m]))
```

### Grafana Dashboard Design Best Practices

**1. Panel types -- choose the right visualization:**

| Panel Type | Use For | Example |
|------------|---------|---------|
| Time series | Trends over time | Request rate, latency over time |
| Stat | Single current value | Total requests, error rate |
| Gauge | Current value in a range | CPU usage (0-100%) |
| Table | Top-N, comparisons | Top 10 endpoints by latency |
| Bar chart | Categorical comparison | Requests by status code |
| Heatmap | Distribution over time | Latency distribution |
| State timeline | Status changes | Up/down status over time |
| Alert list | Active alerts | Firing alerts |

**2. Layout -- use rows and sections:**

```
Row 1: Key Metrics (4 stat panels)
+-- Requests/sec | Error Rate | p99 Latency | SLO Status
+-- Full width, high visibility
+-- Green/yellow/red thresholds

Row 2: RED Overview (3 time series panels)
+-- Rate over time | Errors over time | Latency percentiles over time
+-- 3 columns, equal width
+-- Show last 1 hour, comparison with yesterday

Row 3: Saturation (4 gauge panels)
+-- CPU | Memory | Disk | Network
+-- 4 columns
+-- Thresholds: green < 70%, yellow 70-90%, red > 90%

Row 4: Detailed Breakdown (2 panels)
+-- Latency by endpoint (table) | Errors by endpoint (table)
+-- Sortable, filterable
+-- Links to logs/traces

Row 5: Infrastructure (time series)
+-- CPU by instance | Memory by instance
+-- Full width
+-- Show all instances, highlight anomalies
```

**3. Variables -- make dashboards reusable:**

```json
{
  "templating": {
    "list": [
      {
        "name": "environment",
        "type": "query",
        "query": "label_values(up, environment)",
        "current": {"text": "production", "value": "production"}
      },
      {
        "name": "service",
        "type": "query",
        "query": "label_values(http_requests_total{environment=\"$environment\"}, service)",
        "multi": true,
        "includeAll": true
      },
      {
        "name": "instance",
        "type": "query",
        "query": "label_values(node_cpu_seconds_total{environment=\"$environment\"}, instance)"
      }
    ]
  }
}
```

Then use variables in queries:

```promql
rate(http_requests_total{service=~"$service", environment="$environment"}[5m])
```

**4. Annotations -- overlay events on graphs:**

```json
{
  "annotations": {
    "list": [
      {
        "name": "Deployments",
        "datasource": "Prometheus",
        "query": "kube_deployment_created",
        "enable": true
      },
      {
        "name": "Alerts",
        "datasource": "AlertManager",
        "enable": true
      }
    ]
  }
}
```

**5. Thresholds and reference lines:**

```json
{
  "fieldConfig": {
    "defaults": {
      "thresholds": {
        "steps": [
          {"color": "green", "value": null},
          {"color": "yellow", "value": 80},
          {"color": "red", "value": 95}
        ]
      }
    }
  }
}
```

Add SLO lines to latency graphs:

```promql
# SLO target line (constant)
0.5  # 500ms SLO target
```

### Dashboard Anti-Patterns to Avoid

| Anti-Pattern | Problem | Fix |
|-------------|---------|-----|
| Too many panels | Information overload, slow loading | Max 8-12 panels per row, use drill-down |
| No time range context | Unclear if current values are normal | Add comparison period (yesterday, last week) |
| Average only | Hides outliers and distribution | Show p50, p95, p99 for latency |
| No thresholds | Unclear what is "good" or "bad" | Add color thresholds and SLO lines |
| Pie charts | Hard to compare sizes, waste space | Use bar charts or tables |
| Too many colors | Visual noise | Use a consistent color palette |
| No variables | Dashboard only works for one environment | Use Grafana variables |
| Stale dashboards | Nobody maintains or updates them | Review dashboards quarterly, delete unused |
| No drill-down | Cannot go from overview to details | Link dashboards and use links to logs/traces |

### Alerting vs Dashboarding

Not everything needs an alert. Not everything belongs on a dashboard.

**Should it be an alert?**

```
Question: "Does this need a human to take action RIGHT NOW?"
  YES -> Alert (page the on-call)
  NO  -> Does it need action within 24 hours?
    YES -> Warning-level alert or ticket
    NO  -> Dashboard panel (monitor during business hours)
```

| Signal | Alert? | Dashboard? | Why |
|--------|--------|------------|-----|
| Error rate > 5% for 5 min | Yes | Yes | Needs immediate action |
| CPU > 90% for 15 min | Maybe | Yes | Could be normal burst |
| Disk 80% full | Warning | Yes | Predictive, not urgent |
| Request count by endpoint | No | Yes | Informational |
| Deployment completed | No (annotation) | Yes | Context, not action |
| SLO budget < 25% | Yes | Yes | Proactive reliability action |
| Memory usage trending up | No | Yes | Capacity planning |

---

## 4. The Production Way

### Dashboard as Code (Grafana JSON Models)

Store dashboards in version control, not in the Grafana UI:

```json
{
  "dashboard": {
    "id": null,
    "uid": "api-overview",
    "title": "API Gateway Overview",
    "tags": ["api", "production", "overview"],
    "timezone": "browser",
    "refresh": "30s",
    "time": {
      "from": "now-1h",
      "to": "now"
    },
    "templating": {
      "list": [
        {
          "name": "environment",
          "type": "custom",
          "query": "production,staging",
          "current": {"text": "production", "value": "production"}
        }
      ]
    },
    "panels": [
      {
        "title": "Request Rate",
        "type": "stat",
        "gridPos": {"h": 4, "w": 6, "x": 0, "y": 0},
        "targets": [
          {
            "expr": "sum(rate(http_requests_total{environment=\"$environment\"}[5m]))",
            "legendFormat": "req/s"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "reqps",
            "thresholds": {
              "steps": [{"color": "green", "value": null}]
            }
          }
        }
      },
      {
        "title": "Error Rate",
        "type": "gauge",
        "gridPos": {"h": 4, "w": 6, "x": 6, "y": 0},
        "targets": [
          {
            "expr": "sum(rate(http_requests_total{environment=\"$environment\", status=~\"5..\"}[5m])) / sum(rate(http_requests_total{environment=\"$environment\"}[5m])) * 100",
            "legendFormat": "error %"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percent",
            "max": 10,
            "thresholds": {
              "steps": [
                {"color": "green", "value": null},
                {"color": "yellow", "value": 1},
                {"color": "red", "value": 5}
              ]
            }
          }
        }
      }
    ]
  }
}
```

### Grafana Provisioning

```yaml
# grafana/provisioning/dashboards/dashboards.yml
apiVersion: 1
providers:
  - name: 'production'
    orgId: 1
    folder: 'Production'
    type: file
    disableDeletion: false
    updateIntervalSeconds: 30
    options:
      path: /var/lib/grafana/dashboards/production
      foldersFromFilesStructure: true
  - name: 'infrastructure'
    orgId: 1
    folder: 'Infrastructure'
    type: file
    disableDeletion: false
    updateIntervalSeconds: 30
    options:
      path: /var/lib/grafana/dashboards/infrastructure
```

### Dashboard Review Process

```
Quarterly Dashboard Review Checklist:
- Who uses this dashboard? (If nobody, delete it)
- Are the queries still valid? (Do the metrics still exist?)
- Are the thresholds still appropriate?
- Does it link to the right logs/traces/alerts?
- Is it accessible to everyone who needs it?
- Are the variables up to date?
- Is the time range appropriate?
- Do the panel titles make sense to a new team member?
```

### Multi-Service Overview Dashboard

The most valuable dashboard is the service overview -- a single page that shows the health of every service.

```promql
# Service health table -- one row per service
# Columns: Service | Request Rate | Error Rate | p99 Latency | SLO Status

# Request rate per service
sum by (service) (rate(http_requests_total[5m]))

# Error rate per service
sum by (service) (rate(http_requests_total{status=~"5.."}[5m]))
/ sum by (service) (rate(http_requests_total[5m]))
* 100

# p99 latency per service
histogram_quantile(0.99,
  sum by (service, le) (rate(http_request_duration_seconds_bucket[5m]))
)

# SLO budget remaining per service
clamp_min(
  clamp_max(
    (0.001 - (1 - sum by (service)(rate(http_requests_total{status!~"5.."}[30d]))
      / sum by (service)(rate(http_requests_total[30d])))) / 0.001 * 100,
    100
  ),
  0
)
```

### Incident Response Dashboard

A specialized dashboard for use during incidents:

```
Row 1: Incident Timeline
+-- Annotations showing: deploy times, alert fires, mitigation actions
+-- Current status indicator (investigating / identified / monitoring / resolved)

Row 2: Impact Assessment
+-- Error rate (current vs baseline)
+-- Affected users/requests (count)
+-- SLO impact (budget consumed today)

Row 3: Service Health
+-- RED metrics for affected service
+-- Upstream/downstream service health
+-- Dependency health (database, cache, external APIs)

Row 4: Infrastructure
+-- Resource utilization for affected service
+-- Recent scaling events
+-- Pod restarts, OOM kills

Row 5: Debugging
+-- Recent error logs
+-- Slow traces
+-- Recent deployments (comparison)
```

---

## 5. Hands-On Lab

### Lab: Design a Production Dashboard

**Objective:** Build a multi-level dashboard system in Grafana: an overview dashboard, a service-specific dashboard, and an infrastructure dashboard.

**Step 1: Setup**

```bash
mkdir dashboard-lab && cd dashboard-lab
mkdir -p grafana/provisioning/datasources grafana/provisioning/dashboards/json prometheus app
```

**Step 2: Create a sample application**

```python
# app/main.py
from flask import Flask, jsonify
from prometheus_client import Counter, Histogram, Gauge, generate_latest, CONTENT_TYPE_LATEST
import random
import time

app = Flask(__name__)

SERVICE_NAME = 'order-service'

REQUEST_COUNT = Counter(
    'http_requests_total', 'Total HTTP requests',
    ['service', 'method', 'endpoint', 'status']
)
REQUEST_LATENCY = Histogram(
    'http_request_duration_seconds', 'Request latency',
    ['service', 'method', 'endpoint'],
    buckets=[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
)
IN_FLIGHT = Gauge('http_in_flight_requests', 'In-flight requests', ['service'])
DB_POOL_ACTIVE = Gauge('db_connection_pool_active', 'Active DB connections', ['service'])
DB_POOL_MAX = Gauge('db_connection_pool_max', 'Max DB connections', ['service'])
CACHE_HITS = Counter('cache_hits_total', 'Cache hits', ['service'])
CACHE_MISSES = Counter('cache_misses_total', 'Cache misses', ['service'])

DB_POOL_MAX.labels(service=SERVICE_NAME).set(20)

@app.before_request
def before_request():
    IN_FLIGHT.labels(service=SERVICE_NAME).inc()

@app.after_request
def after_request(response):
    IN_FLIGHT.labels(service=SERVICE_NAME).dec()
    latency = random.uniform(0.01, 0.1)
    if random.random() < 0.02:
        latency = random.uniform(0.5, 2.0)
    REQUEST_COUNT.labels(
        SERVICE_NAME, request.method, request.path, str(response.status_code)
    ).inc()
    REQUEST_LATENCY.labels(SERVICE_NAME, request.method, request.path).observe(latency)
    return response

@app.route('/api/orders')
def get_orders():
    DB_POOL_ACTIVE.labels(service=SERVICE_NAME).set(random.randint(1, 15))
    if random.random() < 0.9:
        CACHE_HITS.labels(service=SERVICE_NAME).inc()
    else:
        CACHE_MISSES.labels(service=SERVICE_NAME).inc()
    if random.random() < 0.01:
        return jsonify({"error": "Internal Server Error"}), 500
    return jsonify({"orders": []})

@app.route('/api/users')
def get_users():
    DB_POOL_ACTIVE.labels(service=SERVICE_NAME).set(random.randint(1, 10))
    if random.random() < 0.01:
        return jsonify({"error": "Service Unavailable"}), 503
    return jsonify({"users": []})

@app.route('/health')
def health():
    return jsonify({"status": "healthy"})

@app.route('/metrics')
def metrics():
    return generate_latest(), 200, {'Content-Type': CONTENT_TYPE_LATEST}

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=8080)
```

**Step 3: Create Grafana provisioning**

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
      path: /etc/grafana/provisioning/dashboards/json
```

**Step 4: Create the Overview Dashboard**

```json
// grafana/provisioning/dashboards/json/overview.json
{
  "dashboard": {
    "uid": "overview",
    "title": "System Overview",
    "tags": ["overview", "production"],
    "timezone": "browser",
    "refresh": "30s",
    "time": {"from": "now-1h", "to": "now"},
    "templating": {
      "list": [
        {
          "name": "environment",
          "type": "custom",
          "query": "production,staging",
          "current": {"text": "production", "value": "production"}
        }
      ]
    },
    "panels": [
      {
        "title": "REQUESTS PER SECOND",
        "type": "stat",
        "gridPos": {"h": 4, "w": 6, "x": 0, "y": 0},
        "targets": [
          {
            "expr": "sum(rate(http_requests_total{service=\"order-service\"}[5m]))",
            "legendFormat": "req/s"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "reqps",
            "decimals": 1,
            "thresholds": {
              "steps": [{"color": "green", "value": null}]
            }
          }
        }
      },
      {
        "title": "ERROR RATE",
        "type": "gauge",
        "gridPos": {"h": 4, "w": 6, "x": 6, "y": 0},
        "targets": [
          {
            "expr": "sum(rate(http_requests_total{service=\"order-service\", status=~\"5..\"}[5m])) / sum(rate(http_requests_total{service=\"order-service\"}[5m])) * 100",
            "legendFormat": "error %"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percent",
            "max": 5,
            "thresholds": {
              "steps": [
                {"color": "green", "value": null},
                {"color": "yellow", "value": 1},
                {"color": "red", "value": 3}
              ]
            }
          }
        }
      },
      {
        "title": "P99 LATENCY",
        "type": "stat",
        "gridPos": {"h": 4, "w": 6, "x": 12, "y": 0},
        "targets": [
          {
            "expr": "histogram_quantile(0.99, sum by (le) (rate(http_request_duration_seconds_bucket{service=\"order-service\"}[5m])))",
            "legendFormat": "p99"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "s",
            "thresholds": {
              "steps": [
                {"color": "green", "value": null},
                {"color": "yellow", "value": 0.5},
                {"color": "red", "value": 1}
              ]
            }
          }
        }
      },
      {
        "title": "IN-FLIGHT REQUESTS",
        "type": "stat",
        "gridPos": {"h": 4, "w": 6, "x": 18, "y": 0},
        "targets": [
          {
            "expr": "http_in_flight_requests{service=\"order-service\"}",
            "legendFormat": "active"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "thresholds": {
              "steps": [
                {"color": "green", "value": null},
                {"color": "yellow", "value": 50},
                {"color": "red", "value": 100}
              ]
            }
          }
        }
      },
      {
        "title": "REQUEST RATE (5m)",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 4},
        "targets": [
          {
            "expr": "sum by (status) (rate(http_requests_total{service=\"order-service\"}[5m]))",
            "legendFormat": "{{status}}"
          }
        ],
        "fieldConfig": {"defaults": {"unit": "reqps"}}
      },
      {
        "title": "LATENCY PERCENTILES (5m)",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 4},
        "targets": [
          {
            "expr": "histogram_quantile(0.50, sum by (le) (rate(http_request_duration_seconds_bucket{service=\"order-service\"}[5m])))",
            "legendFormat": "p50"
          },
          {
            "expr": "histogram_quantile(0.95, sum by (le) (rate(http_request_duration_seconds_bucket{service=\"order-service\"}[5m])))",
            "legendFormat": "p95"
          },
          {
            "expr": "histogram_quantile(0.99, sum by (le) (rate(http_request_duration_seconds_bucket{service=\"order-service\"}[5m])))",
            "legendFormat": "p99"
          },
          {
            "expr": "0.5",
            "legendFormat": "SLO Target (500ms)"
          }
        ],
        "fieldConfig": {"defaults": {"unit": "s"}}
      },
      {
        "title": "INFRASTRUCTURE -- CPU and MEMORY",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 12},
        "targets": [
          {
            "expr": "100 - avg(rate(node_cpu_seconds_total{mode=\"idle\"}[5m])) * 100",
            "legendFormat": "CPU Usage %"
          },
          {
            "expr": "(1 - node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes) * 100",
            "legendFormat": "Memory Usage %"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percent",
            "max": 100,
            "thresholds": {
              "steps": [
                {"color": "green", "value": null},
                {"color": "yellow", "value": 70},
                {"color": "red", "value": 90}
              ]
            }
          }
        }
      },
      {
        "title": "DATABASE CONNECTIONS",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 12},
        "targets": [
          {
            "expr": "db_connection_pool_active{service=\"order-service\"}",
            "legendFormat": "Active"
          },
          {
            "expr": "db_connection_pool_max{service=\"order-service\"}",
            "legendFormat": "Max"
          }
        ]
      },
      {
        "title": "CACHE HIT RATIO",
        "type": "gauge",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 20},
        "targets": [
          {
            "expr": "rate(cache_hits_total{service=\"order-service\"}[5m]) / (rate(cache_hits_total{service=\"order-service\"}[5m]) + rate(cache_misses_total{service=\"order-service\"}[5m])) * 100",
            "legendFormat": "hit ratio"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percent",
            "max": 100,
            "thresholds": {
              "steps": [
                {"color": "red", "value": null},
                {"color": "yellow", "value": 80},
                {"color": "green", "value": 95}
              ]
            }
          }
        }
      }
    ]
  }
}
```

**Step 5: Docker Compose**

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

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_DASHBOARDS_DEFAULT_HOME_DASHBOARD_PATH=/etc/grafana/provisioning/dashboards/json/overview.json
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

```yaml
# prometheus/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']
  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']
  - job_name: 'app'
    static_configs:
      - targets: ['app:8080']
```

**Step 6: Launch and explore**

```bash
# Start the stack
docker-compose up -d --build

# Wait for services
sleep 15

# Generate traffic
for i in $(seq 1 3000); do
  curl -s http://localhost:8080/api/orders > /dev/null 2>&1 &
  curl -s http://localhost:8080/api/users > /dev/null 2>&1 &
  if [ $((i % 100)) -eq 0 ]; then
    wait
    sleep 0.05
  fi
done
wait

# Open Grafana
# http://localhost:3000 (admin / admin)
# The Overview dashboard should load as the home dashboard
```

**Step 7: Evaluate your dashboard**

Walk through these questions:

1. **5-second test:** Can you tell if the system is healthy in 5 seconds? (Look at the top row stat panels)
2. **Anomaly detection:** Can you spot if something is unusual? (Latency graphs with SLO lines)
3. **Drill-down:** Can you identify which endpoint is problematic? (Breakdown by endpoint)
4. **Context:** Do you know if current values are normal? (Time series with historical context)
5. **Actionability:** Do you know what to do next? (Runbook links, log/trace links)

**Step 8: Create a service-specific drill-down dashboard**

Add a second dashboard focused on a single service with more detail:

```json
// grafana/provisioning/dashboards/json/service-detail.json
{
  "dashboard": {
    "uid": "service-detail",
    "title": "Service Detail: Order Service",
    "tags": ["service", "order-service"],
    "refresh": "30s",
    "time": {"from": "now-1h", "to": "now"},
    "panels": [
      {
        "title": "Request Rate by Endpoint",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 24, "x": 0, "y": 0},
        "targets": [
          {
            "expr": "sum by (endpoint) (rate(http_requests_total{service=\"order-service\"}[5m]))",
            "legendFormat": "{{endpoint}}"
          }
        ]
      },
      {
        "title": "Error Rate by Endpoint",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 8},
        "targets": [
          {
            "expr": "sum by (endpoint) (rate(http_requests_total{service=\"order-service\", status=~\"5..\"}[5m])) / sum by (endpoint) (rate(http_requests_total{service=\"order-service\"}[5m])) * 100",
            "legendFormat": "{{endpoint}}"
          }
        ]
      },
      {
        "title": "Latency by Endpoint (p99)",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 8},
        "targets": [
          {
            "expr": "histogram_quantile(0.99, sum by (endpoint, le) (rate(http_request_duration_seconds_bucket{service=\"order-service\"}[5m])))",
            "legendFormat": "{{endpoint}}"
          },
          {
            "expr": "0.5",
            "legendFormat": "SLO Target"
          }
        ]
      },
      {
        "title": "DB Connection Pool Utilization",
        "type": "gauge",
        "gridPos": {"h": 6, "w": 8, "x": 0, "y": 16},
        "targets": [
          {
            "expr": "db_connection_pool_active{service=\"order-service\"} / db_connection_pool_max{service=\"order-service\"} * 100",
            "legendFormat": "pool usage"
          }
        ]
      },
      {
        "title": "In-Flight Requests",
        "type": "timeseries",
        "gridPos": {"h": 6, "w": 8, "x": 8, "y": 16},
        "targets": [
          {
            "expr": "http_in_flight_requests{service=\"order-service\"}",
            "legendFormat": "in-flight"
          }
        ]
      },
      {
        "title": "Cache Hit Ratio",
        "type": "gauge",
        "gridPos": {"h": 6, "w": 8, "x": 16, "y": 16},
        "targets": [
          {
            "expr": "rate(cache_hits_total{service=\"order-service\"}[5m]) / (rate(cache_hits_total{service=\"order-service\"}[5m]) + rate(cache_misses_total{service=\"order-service\"}[5m])) * 100",
            "legendFormat": "hit ratio"
          }
        ]
      }
    ]
  }
}
```

**Expected outcome:**
- Overview dashboard provides a quick health check at a glance
- Key metrics (RPS, error rate, p99, in-flight) are immediately visible in the top row
- Latency graph shows SLO target line for context
- Infrastructure metrics are available but not overwhelming
- Service drill-down dashboard allows deeper investigation
- All panels have appropriate thresholds and color coding
- Dashboard variables enable reuse across environments

---

## 6. Limitation: What About the Database?

You now have the complete observability stack:

- **Metrics** (Module 39): What is the aggregate health of the system?
- **Tracing** (Module 40): What is the journey of a single request?
- **Logging** (Module 41): What happened in detail?
- **Alerting** (Module 42): When should we be notified?
- **SLO Monitoring** (Module 43): Are we meeting our reliability targets?
- **Dashboard Design** (Module 44): How do we visualize it all?

You can observe everything. Every request, every error, every latency spike, every resource utilization metric is collected, stored, queryable, and visualized.

But there is a critical piece of infrastructure that often gets its own dedicated monitoring: the database. Databases have unique challenges -- they are stateful, they have complex internal mechanics (query planners, buffer pools, replication lag, lock contention), and they are often the bottleneck when your application scales. Standard CPU/memory dashboards do not capture the nuances of database performance.

The next phase addresses the database specifically: how to scale it, how to monitor its internals, and how to handle the operational challenges of stateful systems at scale.

**Next Module:** [45 -- Database Scaling](../45-database-scaling/README.md) -- Phase 6: Database Operations. Scale databases with replication, sharding, and connection pooling. Understand the unique operational challenges of stateful systems.
