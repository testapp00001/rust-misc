# Exercise 05: Design a Logging Strategy Correlating Logs, Metrics, and Traces

**Type:** Integration
**Time:** 60 minutes
**Difficulty:** Hard

## Objective

Design and implement a strategy that connects all three pillars of
observability -- metrics, logs, and traces -- so that you can navigate from an
alert to a metric graph to a trace to the specific log line that explains the
failure, all within Grafana.

## Background

An alert fires: "Error rate on order-service is 5%." You open Grafana and see
the spike on a graph. But which requests failed? You click an exemplar on the
graph and jump to the trace in Tempo. The trace shows that the payment step
took 30 seconds and failed. But why? You click "Logs for this trace" and see
the log line: `{"level":"ERROR","message":"payment_failed","error":"stripe_timeout","trace_id":"abc123"}`.
Now you know the root cause. This is the power of correlated observability.

This exercise ties together everything from the observability modules: metrics
from Prometheus, traces from Tempo, and logs from Loki, all connected through
Grafana.

---

## Tasks

### Part A: Propagate Context Across Signals

Design how a single request's context flows from metrics to traces to logs.
Answer the following questions:

1. What identifiers connect a metric data point to a specific trace? What is
   the name of this mechanism in Prometheus?
2. What identifier connects a trace span to a specific log line? Where is this
   identifier stored in each signal?
3. Write the structured log format for an application that includes both
   `trace_id` and `span_id` fields. Show an example log line.
4. A request passes through three services: gateway, order-service, and
   payment-service. Each service logs with `trace_id`. Explain how you would
   query Loki to see the complete journey of a single request across all three
   services.

<details>
<summary>Hint</summary>

- **Exemplars** are the mechanism that connects Prometheus metrics to traces.
  An exemplar is a metric data point annotated with a `trace_id`.
- The `trace_id` is the common identifier between traces and logs. In traces,
  it is the trace ID. In logs, it is a field you must propagate from the
  tracing context.
- OpenTelemetry's W3C Trace Context header (`traceparent`) propagates the
  trace ID between services.
- To see all logs for a trace: `{namespace="production"} | json |
  trace_id="abc123def456"` -- this returns logs from every service that
  recorded that trace ID.

</details>

### Part B: Configure Grafana Datasource Correlation

Write the Grafana datasource configuration (YAML) that enables the following
navigation paths:

1. **Metrics to Traces:** Click an exemplar on a Prometheus graph and jump to
   the corresponding trace in Tempo.
2. **Traces to Logs:** From a trace span in Tempo, click to see the correlated
   log lines in Loki, filtered by `trace_id` and `service`.
3. **Logs to Traces:** From a log line in Loki, click the `trace_id` field
   to jump to the full trace in Tempo.

Write the `datasources.yaml` provisioning file with all three datasources and
their correlation settings.

<details>
<summary>Hint</summary>

- In the Prometheus datasource, set `jsonData.exemplarTraceIdDestinations` to
  map the `trace_id` label to the Tempo datasource.
- In the Tempo datasource, set `jsonData.tracesToLogs` with `datasourceUid`,
  `filterByTraceID: true`, and `tags` for label mapping.
- In the Loki datasource, set `jsonData.derivedFields` with a regex matcher
  for `trace_id` and a link to the Tempo datasource.
- All three datasources must reference each other by `uid`.

</details>

### Part C: Build a Correlated Dashboard

Design a Grafana dashboard that demonstrates the full correlation workflow.
Describe each panel and how a user would navigate from one to the next.

The dashboard should include:

1. **Service Overview (Time Series):** Shows request rate and error rate from
   Prometheus, with exemplars enabled so data points link to traces.
2. **Trace Duration (Time Series):** Shows P50, P95, P99 trace duration from
   Tempo metrics summary.
3. **Log Volume by Level (Time Series):** Shows log line rates from Loki,
   grouped by level.
4. **Error Logs (Log Panel):** Shows recent ERROR log lines with parsed fields.
   Each log line should have a clickable `trace_id` link.
5. **Request Flow (Trace Panel):** When a trace is selected, shows the full
   span waterfall with links to correlated logs.

For each panel, document the query and the navigation flow (what the user
clicks to get from this panel to the next signal).

<details>
<summary>Hint</summary>

- Panel 1 uses a Prometheus query with the Grafana "Exemplars" option enabled.
- Panel 2 uses Tempo's metrics summary or a Prometheus query against
  `traces_spanmetrics_*` metrics.
- Panel 3 uses a Loki metric query: `sum by (level) (count_over_time(
  {service=~"$service"} | json [1m]))`.
- Panel 4 uses a Loki log query with a derived field link on `trace_id`.
- Panel 5 uses the Tempo "Trace" panel type with a variable populated from
  exemplar selection or a log line's `trace_id`.

</details>

### Part D: Design a Production Logging Strategy

Your company is migrating to Kubernetes and adopting the three pillars of
observability. Write a comprehensive logging strategy document that covers:

1. **Log Collection Architecture:** How logs flow from application to queryable
   storage. Include the components, their roles, and how they are deployed.
2. **Structured Logging Standards:** What every log line must contain, the
   format, and how context (trace_id, span_id, request_id) is propagated.
3. **Label Strategy:** What labels to use in Loki, cardinality limits, and
   how labels map to Kubernetes metadata.
4. **Retention Policy:** How long to keep logs at each tier, and how to
   enforce retention per namespace or team.
5. **Cost Controls:** How to reduce log volume (filtering, sampling, level
   management) without losing critical information.
6. **Correlation with Metrics and Traces:** How the three signals connect,
   what identifiers are shared, and how Grafana is configured for navigation.
7. **Alerting Strategy:** What log-based alerts to set up, at what thresholds,
   and how they route to different teams.

Present this as a structured document with clear sections and concrete
configuration examples where appropriate.

<details>
<summary>Hint</summary>

- Reference specific tools: Promtail for collection, Loki for storage,
  Grafana for querying and alerting.
- For structured logging, specify JSON with required fields:
  timestamp, level, service, message, trace_id, span_id.
- For labels, use: namespace, service, pod, level. Avoid high-cardinality
  labels like user_id or request_id (those go in log content, not labels).
- For retention, use Loki's per-tenant overrides to set different retention
  per team.
- For cost controls, mention: drop health check logs in Promtail, use INFO
  level in production, sample DEBUG logs at 10%.
- For correlation, reference the Grafana datasource YAML from Part B.
- For alerting, reference the rules from Exercise 04.

</details>

---

## Success Criteria

- [ ] You can explain how `trace_id` and exemplars connect metrics, traces,
      and logs into a single debugging workflow
- [ ] You can write a Grafana datasource configuration that enables navigation
      from metrics to traces to logs and back
- [ ] You can design a dashboard that presents all three signals with
      interactive links between them
- [ ] You can write a production logging strategy that covers collection,
      structure, labels, retention, cost, correlation, and alerting
- [ ] You understand the end-to-end incident debugging workflow: alert to
      metric graph to trace to log line to root cause

## What You Should Understand After This Exercise

The real value of centralized logging is not the logs themselves -- it is the
connections. A log line is most useful when you can reach it from a metric
spike or a slow trace. The `trace_id` is the thread that ties all three
signals together. When you configure Grafana's datasource correlations
correctly, you create a debugging workflow where every click narrows the
scope: from "something is wrong" (alert) to "this service is failing"
(metric) to "this specific request" (trace) to "this is why" (log). This
is what transforms logging from a cost center into a force multiplier for
incident response.
