# Exercise 03: Write LogQL Queries and Build Grafana Dashboards

**Type:** Independent
**Time:** 45 minutes
**Difficulty:** Medium

## Objective

Write progressively complex LogQL queries to extract insights from log data,
build a Grafana dashboard with multiple panels showing log volume, error rates,
top errors, and latency percentiles, and demonstrate the ability to trace a
specific request across multiple log lines.

## Background

Collecting logs is only useful if you can query them effectively. LogQL is
Loki's query language, designed to feel familiar to anyone who has used PromQL.
It has two parts: a *log pipeline* (select and filter log lines) and a
*metric pipeline* (aggregate log lines into time-series metrics). This exercise
assumes Loki and Promtail are already running from Exercise 02.

---

## Tasks

### Part A: Log Pipeline Queries

Write LogQL queries to answer each question. Run each query in Grafana Explore
or via the Loki HTTP API. Record the query and describe what it returns.

1. Select all logs from the `order-api` service in the `default` namespace.
2. Select only ERROR-level logs from `order-api`.
3. Select all logs from `order-api` that contain the word "timeout".
4. Select all logs from `order-api` that do NOT contain "health" (filter out
   health check noise).
5. Select all ERROR logs where the `user_id` field equals `user-42`. Parse
   the JSON to extract fields first.
6. Select all logs where `user_id` matches any value from `user-1` through
   `user-9` (single-digit user IDs).
7. Format the output to show only `timestamp`, `level`, and `message` fields
   on each line.

<details>
<summary>Hint</summary>

- Stream selectors go in curly braces: `{service="order-api", namespace="default"}`
- Use `| json` to parse JSON logs into queryable fields.
- Use `|= "timeout"` for "contains" and `!= "health"` for "does not contain".
- Use `|~ "user-[1-9]$"` for regex matching.
- Use `| line_format "{{.timestamp}} {{.level}} {{.message}}"` to reformat
  output.

</details>

### Part B: Metric Pipeline Queries

Write LogQL metric queries to produce time-series data. Record each query and
describe the output.

1. Count the total number of log lines per minute from `order-api`.
2. Count the number of ERROR log lines per minute from `order-api`.
3. Calculate the error rate as a percentage: ERROR lines divided by total
   lines, over a 5-minute window.
4. Count log lines per minute grouped by `level` (INFO, WARN, ERROR).
5. Find the top 5 most frequent ERROR messages over the last hour.
6. Calculate the 99th percentile of `duration_ms` over 5-minute windows
   (requires parsing `duration_ms` from JSON and using `quantile_over_time`).

<details>
<summary>Hint</summary>

- `count_over_time({service="order-api"}[1m])` counts log lines per minute.
- Use `sum()`, `sum by (label)()`, `rate()`, and `count_over_time()` for
  aggregation.
- Error rate: `sum(rate({service="order-api"} | json | level="ERROR" [5m]))
  / sum(rate({service="order-api"} | json [5m]))`
- `topk(5, sum by (message) (count_over_time({service="order-api"} | json |
  level="ERROR" [1h])))` gives the most common errors.
- `quantile_over_time(0.99, {service="order-api"} | json | unwrap
  duration_ms [5m])` computes P99 latency from a numeric field.

</details>

### Part C: Trace a Request

The sample application generates logs with `request_id` fields. Write queries
to trace a single request through its lifecycle.

1. Pick any `request_id` value from the logs and fetch all log lines associated
   with it. Describe the sequence of events you see.
2. Write a query that shows all log lines for the same `user_id` as the request
   you traced, within a 1-minute window around that request. What do you learn
   about the user's session?

<details>
<summary>Hint</summary>

- Use `{app="log-generator"} | json | request_id="req-1234"` to find a
  specific request.
- Combine multiple filters: `{app="log-generator"} | json | user_id="user-42"`
  with a time range around the request.

</details>

### Part D: Build a Grafana Dashboard

Create a Grafana dashboard with the following panels. Export the dashboard JSON
or document the query for each panel.

1. **Log Volume (Time Series):** Show the rate of log lines per second,
   grouped by `level`, over the last 1 hour. Use different colors for INFO,
   WARN, and ERROR.
2. **Error Rate (Gauge or Stat):** Show the current ERROR rate as a percentage
   of total log volume over the last 5 minutes. Set thresholds: green below
   1%, yellow below 5%, red above 5%.
3. **Top Errors (Table):** Show the top 10 most frequent ERROR messages in the
   last 1 hour, with a count column.
4. **Log Volume by Service (Bar Chart):** Show total log lines per service in
   the last 1 hour, sorted highest to lowest.
5. **Recent Error Logs (Log Panel):** Show the 50 most recent ERROR-level log
   lines with all fields parsed and displayed.

For each panel, document:

- The panel type you chose
- The LogQL query
- Any transformations or field mappings you configured

<details>
<summary>Hint</summary>

- Time Series panel: use `sum by (level) (count_over_time({app="log-generator"}
  | json [1m]))`
- Gauge panel: use `sum(rate({app="log-generator"} | json | level="ERROR"
  [5m])) / sum(rate({app="log-generator"} | json [5m])) * 100`
- Table panel: use `topk(10, sum by (message) (count_over_time({app="log-generator"}
  | json | level="ERROR" [1h])))`
- Log panel: use `{app="log-generator"} | json | level="ERROR"` with limit 50.
- In Grafana, set the time range to "Last 1 hour" to see enough data.

</details>

---

## Success Criteria

- [ ] You can write LogQL log pipeline queries using stream selectors, JSON
      parsing, line filters, label filters, and line formatting
- [ ] You can write LogQL metric queries using `count_over_time`, `rate`,
      `sum by`, `topk`, and `quantile_over_time`
- [ ] You can trace a specific request across multiple log lines by filtering
      on `request_id`
- [ ] You have a Grafana dashboard with at least 4 panels showing different
      views of the same log data
- [ ] You understand the difference between a log pipeline (returns log lines)
      and a metric pipeline (returns time-series data)

## What You Should Understand After This Exercise

LogQL has two modes: log queries that return log lines, and metric queries that
aggregate log lines into time-series numbers. The power of centralized logging
comes from combining both -- you use metric queries to spot anomalies (error
rate spike, latency increase) and then drill into the specific log lines using
log queries. A well-built Grafana dashboard gives you the overview at a glance
and the ability to drill down to the exact log line causing the problem.
