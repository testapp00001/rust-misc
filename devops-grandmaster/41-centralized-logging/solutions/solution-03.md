# Solution 03: Write LogQL Queries and Build Grafana Dashboards

## Part A: Log Pipeline Queries

### 1. All logs from order-api

```logql
{service="order-api", namespace="default"}
```

This uses a stream selector with two label matchers. Loki finds all log
streams matching these labels and returns all lines.

### 2. Only ERROR-level logs

```logql
{service="order-api"} | json | level="ERROR"
```

The `| json` stage parses each log line as JSON, extracting fields. The
`level="ERROR"` label filter then keeps only lines where the parsed `level`
field equals "ERROR".

### 3. Logs containing "timeout"

```logql
{service="order-api"} |= "timeout"
```

The `|=` operator is a line filter -- it keeps log lines that contain the
substring "timeout" anywhere in the raw text. This works without JSON parsing.

### 4. Logs NOT containing "health"

```logql
{service="order-api"} != "health"
```

The `!=` operator is a negative line filter. It drops any line containing
"health". This is useful for filtering out health check noise from your log
queries.

### 5. ERROR logs for user-42

```logql
{service="order-api"} | json | level="ERROR" | user_id="user-42"
```

After parsing JSON, the `user_id="user-42"` filter matches on the parsed
field value. This is more precise than `|= "user-42"` which would also match
log lines where "user-42" appears in a different context (e.g., a message
saying "handled by user-42's teammate").

### 6. Single-digit user IDs

```logql
{service="order-api"} | json | user_id=~"user-[1-9]$"
```

The `=~` operator applies a regex match. The pattern `user-[1-9]$` matches
user IDs like user-1 through user-9 (single digit, end of string). The `$`
anchor prevents matching user-10, user-100, etc.

### 7. Formatted output

```logql
{service="order-api"} | json | line_format "{{.timestamp}} [{{.level}}] {{.message}}"
```

`| line_format` replaces the original log line with a formatted string using
Go template syntax. Each `{{.field_name}}` is replaced with the parsed field
value. This is useful for creating human-readable output from verbose JSON.

## Part B: Metric Pipeline Queries

### 1. Total log lines per minute

```logql
count_over_time({service="order-api"} | json [1m])
```

This returns a time series where each point is the count of log lines in a
1-minute window. The `| json` stage is included so that Loki processes the
lines through the pipeline, but no filter is applied -- all lines are counted.

### 2. ERROR log lines per minute

```logql
count_over_time({service="order-api"} | json | level="ERROR" [1m])
```

Same as above, but with a filter for `level="ERROR"`. Each data point
represents the number of error lines in that minute.

### 3. Error rate percentage

```logql
sum(rate({service="order-api"} | json | level="ERROR" [5m]))
/
sum(rate({service="order-api"} | json [5m]))
* 100
```

This computes the error rate as: (ERROR lines per second) / (total lines per
second) * 100. The `rate()` function computes per-second rate over a 5-minute
window. The `sum()` function aggregates across all matching streams (if you
have multiple replicas, each produces its own stream).

### 4. Log lines per minute by level

```logql
sum by (level) (count_over_time({service="order-api"} | json [1m]))
```

The `sum by (level)` groups the count by the `level` label. This produces
separate time series for INFO, WARN, and ERROR, which you can visualize as
stacked areas in a graph.

### 5. Top 5 ERROR messages

```logql
topk(5,
  sum by (message) (
    count_over_time({service="order-api"} | json | level="ERROR" [1h])
  )
)
```

`topk(5, ...)` returns the 5 time series with the highest values. The inner
query counts ERROR lines grouped by `message` field over 1-hour windows.
This tells you which error messages are most frequent.

### 6. P99 latency from duration_ms

```logql
quantile_over_time(0.99, {service="order-api"} | json | unwrap duration_ms [5m])
```

`| unwrap duration_ms` tells Loki to use the `duration_ms` field as a numeric
value for aggregation. `quantile_over_time(0.99, ...)` computes the 99th
percentile of that numeric value over 5-minute windows. This requires the
field to be a number in the JSON (not a string).

## Part C: Trace a Request

### 1. Trace a specific request_id

Pick any request_id from recent logs:

```logql
{app="log-generator"} | json | request_id="req-42"
```

A typical sequence of events for an order request:

1. `10:30:01 INFO request_received` -- request arrives at the API
2. `10:30:01 INFO order_validated` -- input validation passes
3. `10:30:02 INFO payment_initiated` -- payment processing starts
4. `10:30:03 INFO payment_completed` -- payment succeeds (or ERROR payment_failed)
5. `10:30:03 INFO order_created` -- order persisted to database
6. `10:30:03 INFO response_sent` -- 200 response returned to client

If the request failed, you might see:

1. `10:30:01 INFO request_received`
2. `10:30:02 WARN slow_query_detected` -- database query taking too long
3. `10:30:05 ERROR payment_failed` -- stripe timeout after 3 seconds
4. `10:30:05 INFO response_sent` -- 500 response returned

### 2. Same user's activity around the request

```logql
{app="log-generator"} | json | user_id="user-42"
```

Set the time range to 1 minute before and after the traced request. This
reveals whether the user had other requests in flight (concurrent sessions),
whether previous requests succeeded or failed (pattern of failures), and
whether other services logged messages about this user.

## Part D: Build a Grafana Dashboard

### Panel 1: Log Volume (Time Series)

**Panel type:** Time series

**Query:**
```logql
sum by (level) (count_over_time({app="log-generator"} | json [1m]))
```

**Configuration:**
- Legend: `{{level}}`
- Color scheme: blue for INFO, yellow for WARN, red for ERROR
- Stack: enabled (stacked area chart)
- Y-axis: "Log lines / min"

### Panel 2: Error Rate (Gauge)

**Panel type:** Gauge

**Query:**
```logql
sum(rate({app="log-generator"} | json | level="ERROR" [5m]))
/
sum(rate({app="log-generator"} | json [5m]))
* 100
```

**Configuration:**
- Unit: percent (0-100)
- Thresholds: green < 1, yellow < 5, red >= 5
- Min: 0, Max: 100

### Panel 3: Top Errors (Table)

**Panel type:** Table

**Query:**
```logql
topk(10,
  sum by (message) (
    count_over_time({app="log-generator"} | json | level="ERROR" [1h])
  )
)
```

**Configuration:**
- Format: Table
- Column for message field, column for count
- Sort by count descending

### Panel 4: Log Volume by Service (Bar Chart)

**Panel type:** Bar chart

**Query:**
```logql
sort_desc(
  sum by (service) (
    count_over_time({app="log-generator"} | json [1h])
  )
)
```

**Configuration:**
- Format: Table (bar chart reads from table format)
- Sort descending by value

### Panel 5: Recent Error Logs (Log Panel)

**Panel type:** Logs

**Query:**
```logql
{app="log-generator"} | json | level="ERROR"
```

**Configuration:**
- Limit: 50 lines
- Show labels: level, service, user_id, request_id
- Deduplication: none (show all lines)
- Enable "Derived fields" link on `trace_id` to jump to Tempo (if configured)

## Common Mistakes

- **Forgetting `| json` before filtering fields.** Without `| json`, the
  fields are not parsed and `level="ERROR"` tries to match against the raw
  log line, which does not work. Always parse before filtering.
- **Using `|=` for field-level filtering.** `|= "ERROR"` matches the
  substring "ERROR" anywhere in the raw line -- including in messages that
  say "no ERROR found". Use `| json | level="ERROR"` for precise field
  matching.
- **Using `rate()` without `sum()`.** If you have multiple replicas, each
  produces a separate stream. `rate()` computes per-stream rates. Without
  `sum()`, you get separate time series for each replica instead of one
  aggregate.
- **Forgetting the time window in `count_over_time`.** The `[5m]` is
  required. Without it, LogQL returns a parse error. The window size
  determines the granularity of your time series.
- **Using high-cardinality fields in `sum by`.** `sum by (request_id)`
  produces one time series per request, which is thousands of lines. Use
  `sum by (level)` or `sum by (message)` for meaningful aggregations.
