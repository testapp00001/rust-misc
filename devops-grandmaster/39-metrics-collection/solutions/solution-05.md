# Solution 05: Design a Metrics Strategy for Microservices

## Part A -- Metric Naming and Types

### api-gateway

| Metric Name | Type | Labels | Unit |
|-------------|------|--------|------|
| `http_requests_total` | Counter | method, endpoint, status | requests |
| `http_request_duration_seconds` | Histogram | method, endpoint | seconds |
| `http_in_flight_requests` | Gauge | (none) | requests |
| `http_response_size_bytes` | Histogram | method, endpoint | bytes |

### order-service

| Metric Name | Type | Labels | Unit |
|-------------|------|--------|------|
| `http_requests_total` | Counter | method, endpoint, status | requests |
| `http_request_duration_seconds` | Histogram | method, endpoint | seconds |
| `orders_created_total` | Counter | (none) | orders |
| `orders_completed_total` | Counter | (none) | orders |
| `orders_in_progress` | Gauge | (none) | orders |
| `db_query_duration_seconds` | Histogram | operation | seconds |
| `db_connection_pool_active` | Gauge | (none) | connections |
| `db_connection_pool_idle` | Gauge | (none) | connections |

### payment-service

| Metric Name | Type | Labels | Unit |
|-------------|------|--------|------|
| `http_requests_total` | Counter | method, endpoint, status | requests |
| `http_request_duration_seconds` | Histogram | method, endpoint | seconds |
| `payment_attempts_total` | Counter | provider, status | payments |
| `payment_duration_seconds` | Histogram | provider | seconds |
| `payment_queue_depth` | Gauge | (none) | payments |

### postgres (exporter metrics)

| Metric Name | Type | Labels | Unit |
|-------------|------|--------|------|
| `pg_stat_activity_count` | Gauge | datname, state | connections |
| `pg_stat_database_tup_fetched` | Counter | datname | rows |
| `pg_stat_database_tup_inserted` | Counter | datname | rows |
| `pg_replication_lag_seconds` | Gauge | (none) | seconds |
| `pg_database_size_bytes` | Gauge | datname | bytes |

### Naming convention notes

- Counters end with `_total`.
- Units are in base SI: seconds (not milliseconds), bytes (not megabytes).
- Labels use snake_case. No user IDs or request IDs.
- All services share the same `http_requests_total` and
  `http_request_duration_seconds` metrics, differentiated by a `service`
  label added via relabeling or service-specific job names.

## Part B -- Exporter Selection

### PostgreSQL

**Exporter:** `prometheus-community/postgres_exporter`

Key metrics:
- `pg_stat_activity_count` -- active connections by state (active, idle,
  idle in transaction).
- `pg_replication_lag_seconds` -- how far behind replicas are from the
  primary.
- `pg_database_size_bytes` -- database size on disk.
- `pg_stat_database_tup_fetched`, `pg_stat_database_tup_inserted`,
  `pg_stat_database_tup_updated`, `pg_stat_database_tup_deleted` -- row
  operation counts.
- `pg_locks_count` -- locks by mode, useful for detecting deadlocks.
- `pg_stat_bgwriter_buffers_checkpoint_total` -- checkpoint activity.

### Redis

**Exporter:** `oliver006/redis_exporter`

Key metrics:
- `redis_connected_clients` -- number of connected clients.
- `redis_used_memory_bytes` -- memory used by Redis.
- `redis_keyspace_hits_total` / `redis_keyspace_misses_total` -- cache hit
  ratio.
- `redis_commands_processed_total` -- commands per second.
- `redis_connected_slaves` -- number of replicas.
- `redis_uptime_in_seconds` -- how long the instance has been running.

### Node/host

**Exporter:** `prom/node_exporter`

Key metrics:
- `node_cpu_seconds_total` -- CPU time per mode.
- `node_memory_MemAvailable_bytes` -- available memory.
- `node_filesystem_avail_bytes` -- available disk space.
- `node_network_receive_bytes_total` -- network bytes received.
- `node_disk_io_time_seconds_total` -- disk I/O utilization.

### Kubernetes

**Exporter:** `k8s-prometheus-stack/kube-state-metrics`

Key metrics:
- `kube_pod_status_ready` -- pod readiness status.
- `kube_deployment_status_replicas_available` -- available replicas.
- `kube_node_status_condition` -- node health conditions.
- `kube_job_status_failed` -- failed batch jobs.
- `kube_persistentvolumeclaim_status_phase` -- PVC status.

## Part C -- PromQL Queries

### 1. Checkout Flow Success Rate

```promql
(
  sum(rate(http_requests_total{service=~"api-gateway|order-service|payment-service", status!~"5.."}[5m]))
  /
  sum(rate(http_requests_total{service=~"api-gateway|order-service|payment-service"}[5m]))
) * 100
```

This aggregates non-5xx responses across all services in the checkout path
and divides by total responses. The result is the success rate percentage.

### 2. Top 5 Slowest Endpoints by p99 Latency

```promql
topk(5,
  histogram_quantile(0.99,
    sum by (service, endpoint, le) (rate(http_request_duration_seconds_bucket[5m]))
  )
)
```

This computes the p99 latency per service and endpoint, then returns the
top 5 highest values.

### 3. Database Connection Pool Saturation

```promql
db_connection_pool_active
/ (db_connection_pool_active + db_connection_pool_idle)
* 100
```

This shows the percentage of the connection pool that is in use. If this
approaches 100%, the pool is saturated and new requests will wait or fail.
For postgres_exporter, use:

```promql
pg_stat_activity_count
/ on (instance) pg_settings_max_connections
* 100
```

### 4. Redis Cache Hit Ratio

```promql
rate(redis_keyspace_hits_total[5m])
/ (rate(redis_keyspace_hits_total[5m]) + rate(redis_keyspace_misses_total[5m]))
* 100
```

A healthy cache hit ratio is typically above 80-90%. Below 50% means most
requests miss the cache and hit the database.

### 5. Error Budget Remaining

```promql
# Error budget: 0.1% over 30 days = 2592 seconds of allowed errors
# Actual error seconds in the last 30 days
(
  2592
  - (
      sum(increase(http_requests_total{status=~"5.."}[30d]))
      / sum(rate(http_requests_total[30d]))
      * 30 * 24 * 3600
    )
)
/ 2592
* 100
```

A simpler approach using the `up` metric for availability:

```promql
# Availability over last 30 days
avg_over_time(up{job="api-gateway"}[30d]) * 100

# Error budget consumed (fraction)
1 - (
  (avg_over_time(up{job="api-gateway"}[30d]) - 0.999)
  / (1 - 0.999)
)
```

## Part D -- Alerting Rules

```yaml
# prometheus/rules/microservice-alerts.yml
groups:
  - name: critical-alerts
    rules:
      # Service completely down
      - alert: ServiceDown
        expr: up{job=~"api-gateway|order-service|payment-service|user-service|inventory-service|notification-service"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Service {{ $labels.job }} is down"
          description: "All instances of {{ $labels.job }} are unreachable."

      # High error rate
      - alert: HighErrorRate
        expr: |
          sum by (service) (rate(http_requests_total{status=~"5.."}[5m]))
          / sum by (service) (rate(http_requests_total[5m]))
          > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Error rate above 5% on {{ $labels.service }}"
          description: "Error rate is {{ $value | humanizePercentage }}."

      # High p99 latency
      - alert: CriticalLatency
        expr: |
          histogram_quantile(0.99,
            sum by (service, le) (rate(http_request_duration_seconds_bucket[5m]))
          ) > 2.0
        for: 10m
        labels:
          severity: critical
        annotations:
          summary: "p99 latency above 2s on {{ $labels.service }}"
          description: "Current p99 latency: {{ $value }}s."

  - name: warning-alerts
    rules:
      # Elevated error rate
      - alert: ElevatedErrorRate
        expr: |
          sum by (service) (rate(http_requests_total{status=~"5.."}[5m]))
          / sum by (service) (rate(http_requests_total[5m]))
          > 0.01
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Error rate above 1% on {{ $labels.service }}"
          description: "Error rate is {{ $value | humanizePercentage }}."

      # Elevated latency
      - alert: HighLatency
        expr: |
          histogram_quantile(0.99,
            sum by (service, le) (rate(http_request_duration_seconds_bucket[5m]))
          ) > 0.5
        for: 15m
        labels:
          severity: warning
        annotations:
          summary: "p99 latency above 500ms on {{ $labels.service }}"

      # Database replica lag
      - alert: DatabaseReplicaLag
        expr: pg_replication_lag_seconds > 30
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Database replica lag above 30 seconds"
          description: "Current lag: {{ $value }}s."

      # Redis memory usage
      - alert: RedisHighMemory
        expr: |
          redis_used_memory_bytes
          / redis_memory_max_bytes
          * 100 > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Redis memory usage above 80%"
          description: "Current usage: {{ $value | humanizePercentage }}."

  - name: info-alerts
    rules:
      # Traffic spike
      - alert: TrafficSpike
        expr: |
          sum(rate(http_requests_total{service="api-gateway"}[5m]))
          > 2 * sum(rate(http_requests_total{service="api-gateway"}[1h]))
        for: 5m
        labels:
          severity: info
        annotations:
          summary: "Traffic spike detected on api-gateway"
          description: "Current rate is {{ $value }}x the 1-hour average."
```

## Part E -- Recording Rules

```yaml
# prometheus/rules/recording-rules.yml
groups:
  - name: recording_rules
    interval: 30s
    rules:
      # Pre-compute request rate per service (used in multiple dashboards)
      - record: service:http_requests:rate5m
        expr: sum by (service) (rate(http_requests_total[5m]))

      # Pre-compute error rate per service (used in dashboards and alerts)
      - record: service:http_errors:rate5m
        expr: |
          sum by (service) (rate(http_requests_total{status=~"5.."}[5m]))
          / sum by (service) (rate(http_requests_total[5m]))

      # Pre-compute p99 latency per service (expensive histogram_quantile)
      - record: service:http_latency:p99_5m
        expr: |
          histogram_quantile(0.99,
            sum by (service, le) (rate(http_request_duration_seconds_bucket[5m]))
          )
```

### Why these three?

1. `service:http_requests:rate5m` -- The raw `rate(http_requests_total[5m])`
   query is used in at least 5 panels across dashboards. Pre-computing it
   once saves Prometheus from repeating the same computation.

2. `service:http_errors:rate5m` -- The error rate expression involves two
   `rate()` calls, a division, and label matching. It is expensive and
   reused in dashboards and the error budget calculation.

3. `service:http_latency:p99_5m` -- `histogram_quantile()` over multiple
   services is the most expensive query. Pre-computing it reduces dashboard
   load time from seconds to milliseconds.

## Part F -- Dashboard Design

### Level 1 -- Overview Dashboard

```
+---------------------------+---------------------------+-------------------+
| Total Requests/sec (Stat) | Error Rate % (Gauge)     | Uptime % (Stat)  |
+---------------------------+---------------------------+-------------------+
| Requests/sec by Service (Time series, stacked area)                      |
+--------------------------------------------------------------------------+
| p99 Latency by Service (Time series, lines)                              |
+--------------------------------------------------------------------------+
| Error Rate by Service (Time series, lines)                               |
+--------------------------------------------------------------------------+
| Active Connections (Time series) | Queue Depths (Time series)            |
+--------------------------------------------------------------------------+
| CPU by Service (Time series)     | Memory by Service (Time series)       |
+--------------------------------------------------------------------------+
```

Queries for the overview dashboard:

- **Total Requests/sec:** `sum(service:http_requests:rate5m)`
- **Error Rate %:** `service:http_errors:rate5m * 100`
- **Uptime %:** `avg_over_time(up[24h]) * 100`
- **Requests/sec by Service:** `service:http_requests:rate5m`
- **p99 Latency by Service:** `service:http_latency:p99_5m`
- **Error Rate by Service:** `service:http_errors:rate5m * 100`
- **Active Connections:** `db_connection_pool_active` by service
- **Queue Depths:** `payment_queue_depth`, `orders_in_progress`
- **CPU by Service:** `sum by (service) (rate(container_cpu_usage_seconds_total[5m]))`
- **Memory by Service:** `sum by (service) (container_memory_usage_bytes)`

Refresh interval: 30 seconds.

### Level 2 -- order-service Dashboard

```
+---------------------------+---------------------------+-------------------+
| Requests/sec (Stat)       | Error Rate % (Gauge)     | p99 Latency (Stat)|
+---------------------------+---------------------------+-------------------+
| Request Rate by Endpoint (Time series)                                    |
+--------------------------------------------------------------------------+
| Latency by Percentile: p50, p95, p99 (Time series)                       |
+--------------------------------------------------------------------------+
| Error Rate by Status Code (Time series)                                   |
+--------------------------------------------------------------------------+
| DB Query Duration by Operation (Time series)                              |
+--------------------------------------------------------------------------+
| DB Connection Pool: Active vs Idle (Time series)                          |
+--------------------------------------------------------------------------+
| Orders Created/Completed Rate (Time series)                               |
+--------------------------------------------------------------------------+
| In-flight Requests (Time series) | Orders In Progress (Time series)       |
+--------------------------------------------------------------------------+
```

### Drill-down from Overview

The overview dashboard uses Grafana template variables:

```
Variable: $service
Query: label_values(service:http_requests:rate5m, service)
Type: dropdown
```

Each panel in the overview dashboard links to the service-specific dashboard
using Grafana data links. When you click on the `order-service` line in the
latency chart, it navigates to `/d/order-service?var-service=order-service`.

The service-specific dashboard uses the same `$service` variable, so all
queries are filtered: `rate(http_requests_total{service="$service"}[5m])`.

## Why It Works

### The RED method applied

Every request-facing service exposes Rate, Errors, and Duration. This gives
a complete picture of service health with minimal metrics. The RED method
is the service-level equivalent of the USE method (Utilization, Saturation,
Errors) which applies to infrastructure resources.

### Label discipline

All services share the same metric names (`http_requests_total`,
`http_request_duration_seconds`) but are differentiated by a `service`
label. This is achieved through Prometheus relabeling or separate scrape
jobs per service. Uniform metric names allow the same dashboard template
to work for every service.

### Alert severity model

- **Critical** = page someone. Short `for` durations (1-5 minutes). These
  indicate user-facing impact.
- **Warning** = notify the team. Longer `for` durations (5-15 minutes).
  These indicate degradation that may become critical.
- **Info** = dashboard annotation only. No notification. These are
  operational signals for awareness.

### Recording rules for expensive queries

`histogram_quantile()` over aggregated histograms is computationally
expensive because it iterates over all bucket series. When the same query
appears in multiple dashboard panels, Prometheus executes it multiple times
per refresh. Recording rules execute the query once every 30 seconds and
store the result as a new time series. Dashboard panels then read the
pre-computed result instantly.

### Error budget as a burn rate

The error budget translates an SLO (99.9%) into a concrete allowance:
2592 seconds of downtime per 30 days. By tracking how much budget has been
consumed, you can make data-driven decisions about deployments. If the
budget is nearly exhausted, freeze non-critical deploys. If the budget is
healthy, you can take more risks.

## Common Mistakes

- **Inconsistent metric names across services.** If one service uses
  `http_requests_total` and another uses `api_request_count`, you cannot
  write unified dashboards or alerts. Agree on a standard and enforce it.

- **Too many labels.** Every label combination creates a time series. A
  metric with `method`, `endpoint`, `status`, `version`, and `region` labels
  can easily create millions of series. Start with minimal labels and add
  only when a specific query requires them.

- **No recording rules for expensive queries.** If every dashboard panel
  computes `histogram_quantile()` independently, Prometheus wastes CPU and
  dashboards load slowly. Profile your queries in the Prometheus UI and
  pre-compute the expensive ones.

- **Alerting on raw values instead of rates.** Alerting on
  `http_requests_total > 1000000` is meaningless because the counter
  increases forever. Always use `rate()` or `increase()` in alerting
  expressions.

- **Ignoring cardinality from business metrics.** Metrics like
  `orders_created_total` with a `product_id` label can explode if there
  are millions of products. Use aggregated labels (product category) or
  store product-level data in logs.

- **Dashboard without drill-down.** An overview dashboard that shows
  aggregates is useless if you cannot investigate a specific service.
  Always design two levels: overview and service-specific.

- **Not aligning alerts with SLOs.** If your SLO is 99.9% availability,
  alerting when error rate exceeds 5% is too permissive -- you would burn
  through your error budget before the alert fires. Alert thresholds should
  be tighter than SLO thresholds.
