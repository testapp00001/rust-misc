# Solution 01: Metric Types: Counter, Gauge, Histogram, Summary

## Part A -- Definitions

### 1. Counter

A Counter is a monotonically increasing value. It only goes up, except when
the process restarts, at which point it resets to zero. Counters track
cumulative totals: total requests served, total errors encountered, total
bytes transferred. You never read a raw Counter value directly -- you always
use `rate()` or `increase()` to compute the rate of change.

### 2. Gauge

A Gauge is a value that can go up and down arbitrarily. It represents a
current measurement: memory usage, temperature, queue depth, number of
active connections. Unlike a Counter, a Gauge has no built-in notion of
direction -- it just holds whatever value was last set.

### 3. Histogram

A Histogram samples observations (typically request durations or response
sizes) and counts them in configurable buckets. Each bucket is a cumulative
counter that tracks how many observations fell at or below that value.
Histograms also track `_sum` (total of all observations) and `_count`
(total number of observations). The key advantage is that you can compute
quantiles (p50, p95, p99) across multiple instances using
`histogram_quantile()` because the raw bucket data can be aggregated.

### 4. Summary

A Summary is similar to a Histogram but computes quantiles on the client
side. The application itself calculates the p50, p99, etc. and exposes them
as gauge-like values. The problem is that you cannot aggregate quantiles
across instances -- the average of two p99 values is not the p99 of the
combined dataset. This makes Summaries nearly useless in distributed systems
where you need cluster-wide percentiles.

### 5. Why Histogram over Summary

In a distributed system, you have multiple instances of a service. To get
the cluster-wide p99, you need to combine data from all instances. With
Histograms, each instance exports raw bucket counts that can be summed
before computing the quantile. With Summaries, each instance exports
pre-computed quantiles that cannot be meaningfully combined. Histograms are
also more flexible -- you can compute any quantile after the fact, while
Summaries are limited to the quantiles configured at collection time.

## Part B -- Classification

| # | Scenario | Type | Reasoning |
|---|----------|------|-----------|
| 1 | Total HTTP requests | **Counter** | Cumulative total that only goes up. Use `rate()` to get requests/second. |
| 2 | Active DB connections | **Gauge** | Current count that goes up and down as connections open and close. |
| 3 | Request latency | **Histogram** | Distribution of values where you need percentiles (p50, p95, p99). |
| 4 | CPU temperature | **Gauge** | Current measurement that fluctuates. |
| 5 | Total bytes sent | **Counter** | Cumulative total that only increases. Use `rate()` for bytes/second. |
| 6 | Items in queue | **Gauge** | Current count that goes up (enqueue) and down (dequeue). |
| 7 | Upload file sizes | **Histogram** | Distribution where you want to understand the spread (median, p99). |
| 8 | Current memory usage | **Gauge** | Current measurement that fluctuates as processes allocate and free memory. |
| 9 | Total errors by code | **Counter** | Cumulative total, broken down by label. Use `rate()` for errors/second. |
| 10 | Disk usage percentage | **Gauge** | Current percentage that fluctuates as data is written and deleted. |

## Part C -- PromQL Operations

| Function | Counter | Gauge | Histogram | Summary |
|----------|---------|-------|-----------|---------|
| `rate()` | Valid -- computes per-second increase | Not meaningful -- value goes up and down | Valid -- on `_bucket`, `_sum`, `_count` | Not applicable |
| `increase()` | Valid -- computes total increase over window | Not meaningful | Valid -- on `_bucket`, `_sum`, `_count` | Not applicable |
| `histogram_quantile()` | Not applicable | Not applicable | Valid -- computes quantile from buckets | Not applicable (quantiles already computed) |
| `avg()` | Not meaningful on raw counter | Valid -- average current value | Valid -- on `_bucket` series | Valid -- on quantile series |
| `sum()` | Not meaningful on raw counter | Valid -- sum current values | Valid -- on `_bucket` (aggregate before quantile) | Valid but misleading |
| `delta()` | Not meaningful (counter only goes up) | Valid -- change over time window | Not typically used | Not typically used |

## Part D -- Label Cardinality

1. **method** (GET, POST, PUT, DELETE) -- **Good idea.** Low cardinality
   (4-10 values). Useful for understanding traffic patterns and identifying
   which methods cause errors.

2. **status** (200, 404, 500, etc.) -- **Good idea.** Low cardinality
   (typically 5-10 common codes). Essential for error rate calculations.
   Consider grouping to `2xx`, `3xx`, `4xx`, `5xx` if individual codes are
   not needed.

3. **user_id** -- **Bad idea.** Unbounded cardinality. If you have 1 million
   users, this multiplies every other label combination by 1 million. Store
   user-level debugging in logs, not metrics.

4. **endpoint** (/api/users, /api/orders, etc.) -- **Good idea** if the
   number of endpoints is bounded (tens to low hundreds). **Bad idea** if
   endpoints include dynamic path parameters like `/api/users/12345` --
   normalize to `/api/users/{id}`.

5. **request_id** -- **Catastrophic.** Every request gets a unique UUID.
   This creates a new time series for every single request, making the
   metric useless for aggregation and overwhelming Prometheus storage.

6. **region** (us-east-1, eu-west-1, etc.) -- **Good idea.** Low
   cardinality (typically 3-10 regions). Useful for region-specific
   alerting and debugging.

## Common Mistakes

- **Confusing Gauge and Counter.** A queue depth is a Gauge (goes up and
  down). A total number of messages processed is a Counter (only goes up).
  If you use `rate()` on a Gauge, you get meaningless results.

- **Applying `rate()` to a Gauge.** `rate()` computes the per-second
  increase of a monotonically increasing value. On a Gauge that fluctuates,
  the result is meaningless because negative changes cancel positive ones.

- **Using Summary for distributed percentiles.** A Summary computes p99 on
  each instance. The p99 of the combined traffic is not the average of the
  per-instance p99 values. Use Histograms and `histogram_quantile()` instead.

- **Adding high-cardinality labels.** Every unique label combination creates
  a separate time series. A metric with 10 label dimensions each having 10
  values produces 10^10 = 10 billion series. Always ask: "How many unique
  values will this label have?"

- **Reading raw Counter values.** The raw value of a Counter (e.g.,
  `http_requests_total = 14523`) is meaningless on its own. Always use
  `rate()` or `increase()` to compute meaningful values from Counters.
