# Solution 05: Design a Logging Strategy Correlating Logs, Metrics, and Traces

## Part A: Propagate Context Across Signals

### 1. Metric-to-trace connection: Exemplars

The mechanism that connects a Prometheus metric data point to a specific trace
is called an **exemplar**. An exemplar is a data point on a metric time series
that is annotated with a `trace_id` (and optionally a `span_id`). When you
graph a metric in Grafana, exemplars appear as clickable dots on the graph.
Clicking one opens the corresponding trace in Tempo.

Exemplars are attached at the application level. When your application records
a metric (e.g., a histogram observation for request duration), it also includes
the current trace ID:

```python
# Python example with OpenTelemetry
from opentelemetry import trace

span = trace.get_current_span()
trace_id = span.get_span_context().trace_id

histogram_labels = {"method": "POST", "path": "/api/orders"}
# Record metric with exemplar
request_duration.observe(duration, exemplar={"trace_id": format(trace_id, '032x')})
```

### 2. Trace-to-log connection: trace_id

The identifier that connects a trace span to a specific log line is the
**trace_id**. It is stored in two places:

- **In the trace:** Every span in a trace has a `trace_id` field (a 128-bit
  value, typically represented as a 32-character hex string).
- **In the log line:** The application must include the `trace_id` as a field
  in the structured log entry. This requires propagating the trace context
  into the logging framework.

The connection works because both the trace and the log line contain the same
`trace_id` value. Grafana uses this to link between the two signals.

### 3. Structured log format with trace context

```json
{
  "timestamp": "2024-03-15T10:30:45.123Z",
  "level": "ERROR",
  "service": "order-service",
  "message": "payment_failed",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "span_id": "b7ad6b7169203331",
  "order_id": "12345",
  "user_id": "user-42",
  "error": "stripe_timeout",
  "duration_ms": 30000
}
```

The `trace_id` and `span_id` values are injected into every log entry by the
logging framework, using the OpenTelemetry context propagation:

```python
# Python: auto-inject trace context into log entries
import logging
from opentelemetry import trace

class TracingFormatter(logging.Formatter):
    def format(self, record):
        span = trace.get_current_span()
        ctx = span.get_span_context()
        record.trace_id = format(ctx.trace_id, '032x')
        record.span_id = format(ctx.span_id, '016x')
        return super().format(record)
```

### 4. Querying a multi-service trace in Loki

To see the complete journey of a request across gateway, order-service, and
payment-service:

```logql
{namespace="production"} | json | trace_id="0af7651916cd43dd8448eb211c80319c"
```

This query:
1. Selects all log streams in the `production` namespace (covers all three
   services).
2. Parses JSON to extract fields.
3. Filters for lines where `trace_id` matches the target.

The result shows log lines from all three services in timestamp order,
giving you the complete request flow: gateway received the request,
order-service validated it, payment-service attempted the charge, and so on.
Each log line includes the service name, so you can see which service
generated each event.

## Part B: Configure Grafana Datasource Correlation

Create `datasources.yaml` for Grafana provisioning:

```yaml
# grafana/provisioning/datasources/datasources.yaml
apiVersion: 1

datasources:
  # Prometheus -- metrics with exemplars linking to Tempo
  - name: Prometheus
    type: prometheus
    uid: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    jsonData:
      exemplarTraceIdDestinations:
        - name: trace_id
          datasourceUid: tempo
          urlDisplayLabel: "View Trace"

  # Tempo -- traces with links to Loki logs
  - name: Tempo
    type: tempo
    uid: tempo
    access: proxy
    url: http://tempo:3200
    jsonData:
      tracesToLogs:
        datasourceUid: loki
        filterByTraceID: true
        tags:
          - service
          - namespace
        mappedTags:
          - key: service.name
            value: service
        spanStartTimeShift: "-1m"
        spanEndTimeShift: "1m"
      tracesToMetrics:
        datasourceUid: prometheus
        tags:
          - key: service.name
            value: service
        queries:
          - name: "Request rate"
            query: "sum(rate(traces_spanmetrics_calls_total{$$__tags}[5m]))"
          - name: "Error rate"
            query: "sum(rate(traces_spanmetrics_calls_total{$$__tags, status_code=\"STATUS_CODE_ERROR\"}[5m]))"
      serviceMap:
        datasourceUid: prometheus
      nodeGraph:
        enabled: true

  # Loki -- logs with derived fields linking to Tempo traces
  - name: Loki
    type: loki
    uid: loki
    access: proxy
    url: http://loki:3100
    jsonData:
      derivedFields:
        - datasourceUid: tempo
          matcherRegex: "trace_id\":\"(\\w+)\""
          name: TraceID
          url: "$${__value.raw}"
          urlDisplayLabel: "View Trace"
```

### How the navigation works

**Metrics to Traces:**
1. Open a Prometheus graph in Grafana (e.g., request rate).
2. Exemplars appear as colored dots on the graph.
3. Hover over an exemplar to see the trace_id.
4. Click the exemplar to jump to the trace view in Tempo.

**Traces to Logs:**
1. View a trace in Tempo (the span waterfall).
2. Click a span.
3. In the span details panel, click "Logs for this span."
4. Grafana queries Loki for `{service="order-service"} | json |
   trace_id="<trace_id>"` within the span's time range.
5. The correlated log lines appear below the trace.

**Logs to Traces:**
1. View logs in Grafana Explore with the Loki datasource.
2. Each log line that contains a `trace_id` field shows a derived field link.
3. Click the "TraceID" link next to a log line.
4. Grafana opens the trace view in Tempo, showing the full trace for that
   request.

## Part C: Build a Correlated Dashboard

### Dashboard Structure

**Variables:**
- `$service` -- dropdown populated from Loki label values:
  `label_values({namespace="production"}, service)`
- `$namespace` -- dropdown populated from Loki label values

### Panel 1: Service Overview (Time Series)

**Panel type:** Time series
**Data source:** Prometheus

**Queries:**
```
# Request rate
sum(rate(http_requests_total{service="$service"}[5m]))

# Error rate
sum(rate(http_requests_total{service="$service", status=~"5.."}[5m]))
```

**Configuration:**
- Enable "Exemplars" in the query options.
- Legend: `{{status}}`
- This panel shows the metric spike that starts the debugging workflow.
- When the user sees an error rate spike, they click an exemplar to jump to
  the specific trace in Tempo.

### Panel 2: Trace Duration (Time Series)

**Panel type:** Time series
**Data source:** Tempo (or Prometheus with span metrics)

**Query:**
```
# P50, P95, P99 latency from span metrics
histogram_quantile(0.50, sum(rate(traces_spanmetrics_duration_seconds_bucket{service="$service"}[5m])) by (le))
histogram_quantile(0.95, sum(rate(traces_spanmetrics_duration_seconds_bucket{service="$service"}[5m])) by (le))
histogram_quantile(0.99, sum(rate(traces_spanmetrics_duration_seconds_bucket{service="$service"}[5m])) by (le))
```

**Configuration:**
- Legend: `P50`, `P95`, `P99`
- Y-axis unit: seconds
- Navigation: user sees a latency spike and clicks into Tempo to find the
  slow trace.

### Panel 3: Log Volume by Level (Time Series)

**Panel type:** Time series
**Data source:** Loki

**Query:**
```logql
sum by (level) (count_over_time({service="$service", namespace="$namespace"} | json [1m]))
```

**Configuration:**
- Legend: `{{level}}`
- Stack: enabled
- Colors: blue for INFO, yellow for WARN, red for ERROR
- Navigation: user sees an ERROR spike and drills into Panel 4.

### Panel 4: Error Logs (Log Panel)

**Panel type:** Logs
**Data source:** Loki

**Query:**
```logql
{service="$service", namespace="$namespace"} | json | level="ERROR"
```

**Configuration:**
- Limit: 50 lines
- Show labels: timestamp, level, message, trace_id, order_id
- Each log line has a clickable `trace_id` derived field link.
- Navigation: user sees the error log, clicks the trace_id link to jump to
  Panel 5.

### Panel 5: Request Flow (Trace Panel)

**Panel type:** Trace View (Tempo)
**Data source:** Tempo

**Query:** Populated from a dashboard variable that is set by clicking a
trace_id link from Panel 4. Alternatively, a Tempo search query filtered
by service and time range.

**Configuration:**
- Shows the full span waterfall for the selected trace.
- Each span has a "View Logs" link that opens the correlated Loki logs for
  that span's time range.
- Navigation: user sees which span is slow (e.g., the payment span took 30s),
  clicks "View Logs" to see the detailed error log from the payment service.

### The Complete Workflow

1. **Alert fires:** "Error rate on order-service is 5%."
2. **Open dashboard:** The user sees the spike on Panel 1 (Prometheus metric).
3. **Click exemplar:** Opens the specific trace in Panel 5 (Tempo).
4. **Identify slow span:** The payment span shows 30s duration and error
   status.
5. **Click "View Logs":** Panel 4 shows the Loki log line:
   `{"level":"ERROR","message":"payment_failed","error":"stripe_timeout","trace_id":"abc123"}`
6. **Root cause identified:** Stripe API timeout. The log line contains the
   exact error and the span shows the timing.

## Part D: Production Logging Strategy

### 1. Log Collection Architecture

```
Application Pods ─> stdout/stderr
        |
        v
Container Runtime (containerd) ─> /var/log/containers/*.log
        |
        v
Promtail (DaemonSet, one per node)
  - Tails /var/log/containers/*.log
  - Kubernetes service discovery (pod role)
  - Pipeline: CRI parse > JSON parse > label extraction > drop health checks
        |
        v
Loki (SingleBinary or SimpleScalable mode)
  - Indexes labels only (namespace, service, level, node)
  - Stores compressed chunks in S3-compatible object storage
  - Retention enforced by compactor
        |
        v
Grafana
  - Explore for ad-hoc queries
  - Dashboards for service overview
  - Alerting for log-based alerts
  - Datasource correlation with Prometheus and Tempo
```

**Deployment:**
- Promtail: Helm chart, DaemonSet in `logging` namespace, resource limits
  128Mi memory, 100m CPU per pod.
- Loki: Helm chart, StatefulSet in `logging` namespace, 3 replicas for
  production (read/write/backend split), S3 backend for chunk storage.
- Grafana: Helm chart, Deployment in `monitoring` namespace.

### 2. Structured Logging Standards

Every log line MUST be a single line of valid JSON containing:

| Field | Required | Type | Description |
|-------|----------|------|-------------|
| `timestamp` | Yes | string (ISO 8601) | UTC timestamp with milliseconds |
| `level` | Yes | string | DEBUG, INFO, WARN, ERROR, CRITICAL |
| `service` | Yes | string | Service name (matches Kubernetes label) |
| `message` | Yes | string | Event name in snake_case ("order_created") |
| `trace_id` | Yes | string | 32-char hex from OpenTelemetry context |
| `span_id` | Yes | string | 16-char hex from OpenTelemetry context |
| `request_id` | Conditional | string | Application-level request ID |
| `exception` | Conditional | string | Stack trace (only on errors) |

**Propagation:** Use OpenTelemetry SDK auto-instrumentation to inject
`trace_id` and `span_id` into every log entry. Each language SDK provides
a logging integration (Python: `opentelemetry-instrumentation-logging`,
Go: `otelzap`, Rust: `tracing-opentelemetry`).

**Example:**
```json
{"timestamp":"2024-03-15T10:30:45.123Z","level":"ERROR","service":"order-service","message":"payment_failed","trace_id":"0af7651916cd43dd8448eb211c80319c","span_id":"b7ad6b7169203331","order_id":"12345","error":"stripe_timeout"}
```

### 3. Label Strategy

**Labels to use (low cardinality):**

| Label | Source | Cardinality | Example values |
|-------|--------|-------------|----------------|
| `namespace` | Kubernetes namespace | Low (5-20) | production, staging |
| `service` | Pod label `app.kubernetes.io/name` | Low (20-100) | order-service, payment-api |
| `level` | Parsed from JSON `level` field | Very low (5) | INFO, ERROR |
| `node` | Kubernetes node name | Low (10-50) | node-01, node-02 |

**Labels to avoid (high cardinality -- keep in log content):**

| Field | Why not a label |
|-------|-----------------|
| `user_id` | Millions of unique values |
| `request_id` | Unique per request |
| `order_id` | Unique per order |
| `ip_address` | Thousands of unique values |
| `trace_id` | Unique per trace |

**Cardinality limit:** Configure `max_label_names_per_series: 15` and
`max_label_value_length: 2048` in Loki's limits config.

### 4. Retention Policy

| Namespace | Retention | Rationale |
|-----------|-----------|-----------|
| production | 30 days | Standard compliance, debugging window |
| production (ERROR/WARN) | 90 days | Incident investigation, trend analysis |
| staging | 7 days | Short debugging window, cost savings |
| default | 3 days | Ephemeral workloads, minimal value |

Implementation via Loki's per-tenant overrides:

```yaml
# overrides.yaml
overrides:
  "production":
    retention_period: 720h  # 30 days
  "staging":
    retention_period: 168h  # 7 days
```

For ERROR/WARN retention at 90 days, use a separate Loki tenant for error
logs or archive error logs to long-term storage via a recording rule that
writes to a separate stream.

### 5. Cost Controls

| Strategy | Implementation | Volume Reduction |
|----------|----------------|------------------|
| Drop health checks | Promtail pipeline stage: `match { selector='{app=~".*"}' } drop if message =~ /health\|ready\|metrics/` | 20-40% |
| INFO level in production | Application config: `LOG_LEVEL=INFO` | 50-70% vs DEBUG |
| Sample success logs | Promtail: `sampling { rate = 0.1 }` for INFO-level request logs | 30-50% |
| Compress at shipper | Promtail: `batchwait: 1s`, `batchsize: 1048576` (1MB batches) | 10-20% bandwidth |
| Object storage | Loki chunks stored in S3/GCS instead of block storage | 10x cost reduction |
| Tiered retention | Hot (7d, SSD) / Warm (30d, S3 Standard) / Cold (90d, S3 Glacier) | 60-80% storage cost |

### 6. Correlation with Metrics and Traces

**Shared identifiers:**
- `trace_id` connects all three signals. It is propagated via W3C Trace
  Context headers (`traceparent`) between services and injected into logs
  and metric exemplars.
- `service` label is consistent across Prometheus metrics, Tempo traces, and
  Loki logs. It is the primary navigation dimension.

**Grafana configuration:** See Part B above for the full `datasources.yaml`
with exemplar linking (Prometheus -> Tempo), trace-to-log linking (Tempo ->
Loki), and derived field linking (Loki -> Tempo).

**Additional setup:**
- Enable span metrics in Tempo to generate `traces_spanmetrics_*` metrics
  in Prometheus. This creates a bridge between traces and metrics without
  application changes.
- Enable service graph in Tempo to visualize service dependencies derived
  from trace data.

### 7. Alerting Strategy

| Alert | Query | Threshold | Severity | Routing |
|-------|-------|-----------|----------|---------|
| High error rate | `log:error_rate:ratio5m` | > 1% for 2m | warning | Slack |
| Critical error rate | `log:error_rate:ratio5m` | > 10% for 1m | critical | PagerDuty |
| No logs | `absent_over_time({service="X"} [5m])` | present | critical | PagerDuty |
| New error pattern | keyword match for FATAL, OOM, panic | > 0 | critical | PagerDuty |
| Log volume spike | `log:volume:rate5m` | > 3x baseline for 5m | warning | Slack |
| Log volume drop | `log:volume:rate5m` | < 0.1x baseline for 10m | warning | Slack |

**Recording rules pre-compute:**
- `log:errors:rate5m` -- ERROR lines per second
- `log:volume:rate5m` -- total lines per second
- `log:error_rate:ratio5m` -- error rate ratio

**Notification policy:** Critical -> PagerDuty (30s group_wait, 4h repeat).
Warning -> Slack (1m group_wait, 12h repeat). Info -> Jira (5m group_wait,
24h repeat).

## Common Mistakes

- **Not propagating trace_id into logs.** Without `trace_id` in log entries,
  the connection between traces and logs is broken. You can see the trace in
  Tempo but cannot find the corresponding log line. This is the most common
  gap in observability setups.
- **Using high-cardinality labels in Loki.** Adding `user_id` or `request_id`
  as a Loki label creates millions of streams, crashes the index, and makes
  Loki unusable. Keep high-cardinality data as fields in log content.
- **Not configuring datasource correlation in Grafana.** Even with trace_id
  in logs, the navigation links between Prometheus, Tempo, and Loki do not
  appear unless the `jsonData` sections are configured in the datasource
  provisioning YAML.
- **Ignoring the cost of unstructured logging.** JSON logs are more
  compressible and queryable than plain text. Investing in structured logging
  upfront saves storage costs and debugging time downstream.
- **Setting up alerting without recording rules.** Raw LogQL queries in alert
  rules are expensive. Without recording rules, each alert evaluation scans
  log chunks, which degrades Loki performance under load.
- **Treating all environments the same.** Staging logs do not need 30-day
  retention. Development logs can be dropped entirely. Different tiers for
  different environments reduce cost without sacrificing production visibility.
