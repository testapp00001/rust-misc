# Solution 03: Observability Stack

## Part A: Prometheus Metrics Collection

```yaml
# service-monitor.yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: payment-api
  namespace: payment-system
  labels:
    release: prometheus
spec:
  selector:
    matchLabels:
      app: payment-api
  endpoints:
    - port: http
      path: /metrics
      interval: 15s
      scrapeTimeout: 10s
  namespaceSelector:
    matchNames:
      - payment-system
```

```yaml
# prometheus-rules.yaml
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: payment-api
  namespace: payment-system
  labels:
    release: prometheus
spec:
  groups:
    - name: payment-api-recording
      interval: 30s
      rules:
        - record: payment_api:request_rate:5m
          expr: |
            sum(rate(http_requests_total{
              namespace="payment-system",
              app="payment-api"
            }[5m])) by (endpoint, method)
        - record: payment_api:error_rate:5m
          expr: |
            sum(rate(http_requests_total{
              namespace="payment-system",
              app="payment-api",
              status=~"5.."
            }[5m])) by (endpoint)
            /
            sum(rate(http_requests_total{
              namespace="payment-system",
              app="payment-api"
            }[5m])) by (endpoint)
        - record: payment_api:latency_p99:5m
          expr: |
            histogram_quantile(0.99,
              sum(rate(http_request_duration_seconds_bucket{
                namespace="payment-system",
                app="payment-api"
              }[5m])) by (le, endpoint)
            )

    - name: payment-api-alerts
      rules:
        - alert: HighErrorRate
          expr: payment_api:error_rate:5m > 0.01
          for: 5m
          labels:
            severity: critical
            team: payments
          annotations:
            summary: "Error rate above 1%"
            description: "Payment API error rate is {{ $value | humanizePercentage }} for endpoint {{ $labels.endpoint }}"

        - alert: HighLatency
          expr: payment_api:latency_p99:5m > 0.2
          for: 5m
          labels:
            severity: warning
            team: payments
          annotations:
            summary: "p99 latency above 200ms"
            description: "Payment API p99 latency is {{ $value }}s for endpoint {{ $labels.endpoint }}"

        - alert: PodRestartLoop
          expr: |
            increase(kube_pod_container_status_restarts_total{
              namespace="payment-system",
              container="payment-api"
            }[10m]) > 3
          for: 0m
          labels:
            severity: critical
            team: payments
          annotations:
            summary: "Pod restarting frequently"
            description: "Pod {{ $labels.pod }} has restarted {{ $value }} times in 10 minutes"

        - alert: HPAAtMaxCapacity
          expr: |
            kube_horizontalpodautoscaler_status_current_replicas{
              namespace="payment-system",
              horizontalpodautoscaler="payment-api"
            }
            ==
            kube_horizontalpodautoscaler_spec_max_replicas{
              namespace="payment-system",
              horizontalpodautoscaler="payment-api"
            }
          for: 10m
          labels:
            severity: warning
            team: payments
          annotations:
            summary: "HPA at maximum replicas"
            description: "Payment API has been at {{ $value }} replicas (maximum) for 10 minutes"
```

**Why this works:**

- Recording rules pre-compute expensive queries. Without them, the
  Grafana dashboard would execute `histogram_quantile` over `rate` on
  every page load, which is CPU-intensive. The recording rules compute
  these once every 30 seconds and store the result as a new time series.
- The `for` duration on alerts prevents transient spikes from triggering
  pages. A 5-minute `for` means the condition must be true continuously
  for 5 minutes before the alert fires.
- The `increase(...[10m]) > 3` for pod restarts uses `for: 0m` because
  restart loops are urgent -- you want immediate notification.

## Part B: Structured Logging

**Expected JSON log format:**

```json
{
  "timestamp": "2024-01-15T10:30:45.123Z",
  "level": "info",
  "message": "payment processed",
  "request_id": "req-abc123",
  "user_id": "usr-456",
  "endpoint": "/api/v1/payments",
  "method": "POST",
  "status_code": 200,
  "duration_ms": 42,
  "error": null,
  "trace_id": "abc123def456",
  "span_id": "789ghi012"
}
```

For a failed payment:

```json
{
  "timestamp": "2024-01-15T10:30:45.123Z",
  "level": "error",
  "message": "payment failed: insufficient funds",
  "request_id": "req-xyz789",
  "user_id": "usr-456",
  "endpoint": "/api/v1/payments",
  "method": "POST",
  "status_code": 402,
  "duration_ms": 18,
  "error": "INSUFFICIENT_FUNDS",
  "trace_id": "abc123def456",
  "span_id": "789ghi012"
}
```

**Loki DaemonSet configuration (Promtail):**

```yaml
# promtail-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: promtail-config
  namespace: monitoring
data:
  promtail.yaml: |
    server:
      http_listen_port: 9080
    positions:
      filename: /tmp/positions.yaml
    clients:
      - url: http://loki-gateway/loki/api/v1/push
    scrape_configs:
      - job_name: payment-system
        kubernetes_sd_configs:
          - role: pod
            namespaces:
              names:
                - payment-system
        pipeline_stages:
          - cri: {}
          - json:
              expressions:
                level: level
                request_id: request_id
                user_id: user_id
                endpoint: endpoint
                status_code: status_code
                duration_ms: duration_ms
                error: error
                trace_id: trace_id
          - labels:
              level:
              endpoint:
          - metrics:
              log_lines_total:
                type: Counter
                description: "total number of log lines"
                source: level
                config:
                  value: ""
        relabel_configs:
          - source_labels: [__meta_kubernetes_pod_label_app]
            target_label: app
          - source_labels: [__meta_kubernetes_namespace]
            target_label: namespace
```

**LogQL query for failed payment attempts in the last hour:**

```logql
{namespace="payment-system", app="payment-api"}
  | json
  | status_code >= 400
  | line_format "{{.timestamp}} {{.endpoint}} {{.status_code}} {{.error}} {{.user_id}}"
```

To group by error code:

```logql
sum by (error) (
  count_over_time(
    {namespace="payment-system", app="payment-api"}
      | json
      | status_code >= 400
    [1h]
  )
)
```

## Part C: Distributed Tracing

```yaml
# otel-collector-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: otel-collector-config
  namespace: monitoring
data:
  config.yaml: |
    receivers:
      otlp:
        protocols:
          grpc:
            endpoint: 0.0.0.0:4317
          http:
            endpoint: 0.0.0.0:4318

    processors:
      tail_sampling:
        decision_wait: 10s
        policies:
          # Sample 100% of error traces
          - name: errors
            type: status_code
            status_code:
              status_codes:
                - ERROR
          # Sample 10% of successful traces
          - name: success-sample
            type: probabilistic
            probabilistic:
              sampling_percentage: 10

      batch:
        timeout: 5s
        send_batch_size: 1024

    connectors:
      spanmetrics:
        histogram:
          explicit:
            buckets: [5ms, 10ms, 25ms, 50ms, 100ms, 250ms, 500ms, 1s, 5s]
        dimensions:
          - name: http.method
          - name: http.status_code
          - name: service.name

    exporters:
      jaeger:
        endpoint: jaeger-collector:14250
        tls:
          insecure: true

      prometheus:
        endpoint: 0.0.0.0:8889
        namespace: otel_span

    service:
      pipelines:
        traces:
          receivers: [otlp]
          processors: [tail_sampling, batch]
          exporters: [jaeger, spanmetrics]
        metrics/spanmetrics:
          receivers: [spanmetrics]
          exporters: [prometheus]
```

**Why this works:**

- `tail_sampling` waits for the entire trace to arrive before making a
  sampling decision. This is necessary because you cannot know if a
  trace contains an error until the span with the error arrives.
  `probabilistic_sampling` would miss some error traces.
- The `spanmetrics` connector automatically generates Prometheus metrics
  (request rate, error rate, latency histograms) from trace spans. This
  bridges the gap between traces and metrics without requiring the
  application to emit both.
- 100% of error traces are preserved for debugging. 10% of successful
  traces provide enough data for latency analysis without overwhelming
  storage.

## Part D: Grafana Dashboard

**Panel 1: Request Rate and Errors (Time Series, dual Y-axis)**

```promql
# Left Y-axis: Request rate
sum(rate(http_requests_total{
  namespace="payment-system",
  app="payment-api"
}[5m])) by (endpoint)

# Right Y-axis: Error rate
sum(rate(http_requests_total{
  namespace="payment-system",
  app="payment-api",
  status=~"5.."
}[5m])) by (endpoint)
/
sum(rate(http_requests_total{
  namespace="payment-system",
  app="payment-api"
}[5m])) by (endpoint)
```

**Panel 2: Latency Distribution (Heatmap)**

```promql
# p50
histogram_quantile(0.50,
  sum(rate(http_request_duration_seconds_bucket{
    namespace="payment-system",
    app="payment-api"
  }[5m])) by (le)
)

# p95
histogram_quantile(0.95,
  sum(rate(http_request_duration_seconds_bucket{
    namespace="payment-system",
    app="payment-api"
  }[5m])) by (le)
)

# p99
histogram_quantile(0.99,
  sum(rate(http_request_duration_seconds_bucket{
    namespace="payment-system",
    app="payment-api"
  }[5m])) by (le)
)
```

**Panel 3: Database Health (Gauge)**

```promql
# Connection pool utilization
pg_stat_activity_count{datname="payments"}
  /
pg_settings_max_connections{datname="payments"}

# Query latency
rate(pg_stat_activity_max_tx_duration{datname="payments"}[5m])
```

**Panel 4: Pod Status (Table)**

```promql
# CPU per pod
sum by (pod) (
  rate(container_cpu_usage_seconds_total{
    namespace="payment-system",
    container="payment-api"
  }[5m])
)

# Memory per pod
sum by (pod) (
  container_memory_working_set_bytes{
    namespace="payment-system",
    container="payment-api"
  }
)

# Restart count
kube_pod_container_status_restarts_total{
  namespace="payment-system",
  container="payment-api"
}

# Pod status
kube_pod_status_phase{
  namespace="payment-system",
  phase="Running"
}
```

## Part E: Alert Routing

```yaml
# alertmanager-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: alertmanager-config
  namespace: monitoring
data:
  alertmanager.yml: |
    global:
      resolve_timeout: 5m
      pagerduty_url: 'https://events.pagerduty.com/v2/enqueue'
      slack_api_url: 'https://hooks.slack.com/services/xxx/yyy/zzz'

    route:
      receiver: slack-info
      group_by: [alertname, namespace, endpoint]
      group_wait: 30s
      group_interval: 5m
      repeat_interval: 4h
      routes:
        - receiver: pagerduty-critical
          match:
            severity: critical
          group_wait: 10s
          repeat_interval: 15m
          continue: true
        - receiver: slack-warning
          match:
            severity: warning
          repeat_interval: 1h

    receivers:
      - name: pagerduty-critical
        pagerduty_configs:
          - service_key: '<pagerduty-service-key>'
            severity: critical
            description: '{{ .CommonAnnotations.summary }}'
            details:
              firing: '{{ template "pagerduty.default.instances" .Alerts.Firing }}'

      - name: slack-warning
        slack_configs:
          - channel: '#payments-alerts'
            title: '{{ .GroupLabels.alertname }}'
            text: '{{ .CommonAnnotations.description }}'
            color: 'warning'

      - name: slack-info
        slack_configs:
          - channel: '#payments-info'
            title: '{{ .GroupLabels.alertname }}'
            text: '{{ .CommonAnnotations.description }}'
            color: 'good'

    inhibit_rules:
      - source_match:
          severity: critical
        target_match:
          severity: warning
        equal: [alertname, namespace]

      - source_match:
          severity: critical
        target_match:
          severity: info
        equal: [namespace]

      - source_match:
          severity: warning
        target_match_re:
          severity: info
        equal: [namespace]
```

**Why this works:**

- The `route` tree matches alert labels and routes to receivers. The
  `continue: true` on the critical route means it does not stop
  processing -- a critical alert also matches the lower routes, but
  the inhibition rules suppress them.
- `group_by` groups related alerts into a single notification. If 5 pods
  fire `HighErrorRate` simultaneously, PagerDuty receives one page, not
  five.
- Inhibition rules ensure that when `HighErrorRate` (critical) fires,
  the `HighLatency` (warning) and `HPAAtMaxCapacity` (info) alerts for
  the same namespace are suppressed. This prevents alert fatigue during
  major incidents.

## Common Mistakes

1. **Not using recording rules for dashboard queries.** Without recording
   rules, every Grafana panel executes expensive `histogram_quantile`
   and `rate` queries on every page load. This puts heavy load on
   Prometheus and makes dashboards slow. Pre-compute expensive queries
   with recording rules.

2. **Sampling traces before error detection.** Using probabilistic
   sampling at the SDK level means you lose 90% of error traces. Use
   tail-based sampling in the collector so that 100% of error traces
   are preserved regardless of the sampling rate for successful traces.

3. **Logging in freeform text instead of structured JSON.** Freeform
   logs cannot be queried, aggregated, or alerted on. Every log line
   must be a JSON object with consistent field names. This is the
   difference between "search for the error" and "query for all errors
   in the last hour grouped by endpoint."

4. **Setting alert `for` durations too short.** A `for: 0m` alert fires
   on every transient spike. A `for: 5m` alert fires only if the
   condition persists. Use 5 minutes for most alerts, 10+ minutes for
   capacity alerts, and 0 minutes only for urgent conditions like pod
   restart loops.

5. **Forgetting to add trace context to logs.** Logs and traces are
   useless in isolation during an incident. The `trace_id` and
   `span_id` fields in every log line allow you to jump from a log
   entry directly to the distributed trace that produced it.
