# Exercise 03: Observability Stack

**Type:** Independent
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Implement a complete observability stack -- metrics, logs, and traces --
for the payment-api system. You will configure Prometheus for metrics
collection, Loki for log aggregation, and Jaeger for distributed
tracing. Then you will create Grafana dashboards and alerting rules
that detect problems before users notice.

## Scenario

The payment-api from Exercise 02 is running in production. The team
has no visibility into system behavior. When incidents occur, engineers
are flying blind -- they cannot answer basic questions like "which
endpoint is slow?" or "what error rate is the database returning?"

You must instrument the system so that any engineer can answer these
questions within 60 seconds of an alert firing:

1. What is the current error rate?
2. Which endpoint has the highest latency?
3. What is the database connection pool utilization?
4. Which region is receiving the most traffic?
5. What was happening in the system 5 minutes before the incident?

## Tasks

### Part A: Prometheus Metrics Collection

Write the Kubernetes manifests to deploy Prometheus with:

1. A `ServiceMonitor` that scrapes the payment-api's `/metrics` endpoint
   every 15 seconds
2. Recording rules that pre-compute the 5-minute error rate and the
   99th percentile latency
3. Alert rules that fire when:
   - Error rate exceeds 1% for 5 minutes
   - p99 latency exceeds 200ms for 5 minutes
   - Pod restart count exceeds 3 in 10 minutes
   - HPA is at maximum replicas for 10 minutes

Write the PrometheusRule manifest for both recording rules and alerts.

<details>
<summary>Hint</summary>

The ServiceMonitor is a CRD from the Prometheus Operator. It selects
services by label and tells Prometheus how to scrape them. Recording
rules use `expr` with PromQL to pre-aggregate expensive queries. Alert
rules use `expr` with thresholds and `for` durations to prevent flapping.

</details>

### Part B: Structured Logging

Design a structured logging strategy for the payment-api. Write:

1. A Fluentd/Loki DaemonSet configuration that collects logs from all
   pods in the `payment-system` namespace
2. The expected JSON log format for the payment-api (what fields, what
   structure)
3. A LogQL query that finds all failed payment attempts in the last hour
   with their error codes

The log format must include: timestamp, level, message, request_id,
user_id, endpoint, status_code, duration_ms, and error (if any).

<details>
<summary>Hint</summary>

Structured logging means every log line is a JSON object, not a freeform
string. This makes logs queryable. The DaemonSet runs a log collector
on every node. It tails container log files and ships them to Loki.
LogQL is Loki's query language -- it is similar to PromQL but for logs.

</details>

### Part C: Distributed Tracing

The payment-api calls three internal services during a payment:

1. `fraud-check` -- checks if the payment is fraudulent (p99: 50ms)
2. `ledger` -- records the transaction (p99: 30ms)
3. `notification` -- sends confirmation to the user (p99: 100ms)

Write the OpenTelemetry Collector configuration that:

1. Receives traces from all four services via OTLP (gRPC and HTTP)
2. Samples 100% of error traces and 10% of successful traces
3. Exports traces to Jaeger
4. Exports metrics derived from traces to Prometheus (request rate,
   error rate, latency histograms)

<details>
<summary>Hint</summary>

The OpenTelemetry Collector has three pipelines: receivers, processors,
and exporters. Use the `probabilistic_sampler` or `tail_sampling`
processor for sampling decisions. The `spanmetrics` connector generates
Prometheus metrics from trace spans automatically.

</details>

### Part D: Grafana Dashboard

Design a Grafana dashboard with four panels:

1. **Request Rate and Errors** -- time series showing RPS and error rate
   on the same chart (dual Y-axis)
2. **Latency Distribution** -- heatmap showing p50, p95, p99 latency
   over time
3. **Database Health** -- gauge showing connection pool utilization and
   query latency
4. **Pod Status** -- table showing each pod's CPU, memory, restart
   count, and status

Write the Grafana dashboard JSON (or describe the PromQL queries for
each panel).

<details>
<summary>Hint</summary>

Panel 1 uses two queries: `rate(http_requests_total[5m])` for RPS and
`rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m])`
for error rate. Panel 2 uses a histogram_quantile query. Panel 3 uses
the `pg_stat_activity` metrics from a PostgreSQL exporter. Panel 4 uses
`kube_pod_info` and `container_cpu_usage_seconds_total`.

</details>

### Part E: Alert Routing

Write an Alertmanager configuration that routes alerts to three
destinations:

1. **Critical** (error rate > 5%, database down) -- page the on-call
   engineer via PagerDuty
2. **Warning** (error rate > 1%, high latency) -- send to Slack
   `#payments-alerts` channel
3. **Info** (HPA at max, certificate expiring in 7 days) -- send to
   Slack `#payments-info` channel

Include inhibition rules so that a critical alert suppresses related
warning and info alerts.

<details>
<summary>Hint</summary>

Alertmanager uses `route` to match alert labels and route them to
`receivers`. Inhibition rules use `source_match` (the alert that
suppresses) and `target_match` (the alert that gets suppressed). Use
`severity` labels on your alerts to drive routing.

</details>

## Success Criteria

- [ ] Prometheus scrapes the payment-api and stores both raw metrics and pre-computed recording rules
- [ ] All four alert rules fire correctly when their conditions are met (test with manual metric injection)
- [ ] Logs are collected from all pods and queryable in Loki with structured fields
- [ ] Traces show the full request path through all four services with correct parent-child relationships
- [ ] The Grafana dashboard displays all four panels with live data
- [ ] Alert routing sends critical alerts to PagerDuty and warnings to Slack
- [ ] Inhibition rules suppress lower-severity alerts when a critical alert is active

## What You Should Understand After This Exercise

Observability is not just "add Prometheus and Grafana." Metrics tell
you *what* is happening (error rate, latency). Logs tell you *why* it
is happening (the specific error message, the request context). Traces
tell you *where* in the system it is happening (which service in the
chain is slow). All three are necessary -- metrics without logs are
alarming but unactionable, logs without traces are noisy but
unfocused, and traces without metrics lack historical context.
