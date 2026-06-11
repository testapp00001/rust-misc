# Exercise 01: Metric Types: Counter, Gauge, Histogram, Summary

**Type:** Conceptual
**Estimated time:** 20 minutes

## Objective

Understand the four Prometheus metric types -- Counter, Gauge, Histogram, and
Summary -- and explain when to use each one. This exercise has no hands-on
work; it tests your understanding of the concepts.

## Background

Every metric in Prometheus is one of four types. The type determines what
operations are valid in PromQL and how the data is stored. Choosing the wrong
type leads to incorrect queries, misleading dashboards, and broken alerts.

## Instructions

### Part A -- Definitions

Answer the following questions in your own words:

1. What is a Counter? When does it reset, and why?
2. What is a Gauge? How does it differ from a Counter?
3. What is a Histogram? What are buckets, and why do they matter?
4. What is a Summary? Why is it generally less useful than a Histogram in a
   distributed system?
5. Why does Prometheus recommend Histogram over Summary for most use cases?

### Part B -- Classification

For each scenario below, state which metric type is the most appropriate
choice. Explain your reasoning.

| # | Scenario | Your Choice |
|---|----------|-------------|
| 1 | Total number of HTTP requests received by your service | |
| 2 | Current number of active database connections | |
| 3 | Request latency in milliseconds for an API endpoint | |
| 4 | Current temperature of a server's CPU in Celsius | |
| 5 | Total bytes sent over the network since the process started | |
| 6 | Number of items currently in a message queue | |
| 7 | Size distribution of uploaded files in bytes | |
| 8 | Current memory usage of a process in bytes | |
| 9 | Total number of errors encountered, broken down by error code | |
| 10 | Percentage of disk space currently used | |

### Part C -- PromQL Operations

For each metric type, state whether the following PromQL functions are valid
and explain why:

| Function | Counter | Gauge | Histogram | Summary |
|----------|---------|-------|-----------|---------|
| `rate()` | | | | |
| `increase()` | | | | |
| `histogram_quantile()` | | | | |
| `avg()` | | | | |
| `sum()` | | | | |
| `delta()` | | | | |

### Part D -- Label Cardinality

A developer wants to add the following labels to an `http_requests_total`
Counter. For each label, state whether it is a good idea or a bad idea and
explain why.

1. `method` (GET, POST, PUT, DELETE)
2. `status` (200, 404, 500, etc.)
3. `user_id` (unique per user)
4. `endpoint` (/api/users, /api/orders, etc.)
5. `request_id` (UUID per request)
6. `region` (us-east-1, eu-west-1, etc.)

## Success Criteria

- [ ] You can describe all four metric types and their differences.
- [ ] You correctly classify all ten scenarios.
- [ ] You can explain which PromQL functions work with which metric types.
- [ ] You can identify which labels cause cardinality explosion and explain
      why.
- [ ] You understand why Histogram is preferred over Summary for distributed
      systems.

## Hints

<details>
<summary>Hint 1 -- Counter vs Gauge</summary>
A Counter only goes up (or resets to zero on process restart). A Gauge goes
up and down. If the value can decrease, it is not a Counter.
</details>

<details>
<summary>Hint 2 -- Histogram buckets</summary>
Histogram buckets are cumulative counters. The bucket `le="0.1"` counts all
observations less than or equal to 0.1. Prometheus uses these buckets to
compute approximate quantiles across aggregated instances.
</details>

<details>
<summary>Hint 3 -- Summary limitations</summary>
A Summary computes quantiles on the client side. You cannot aggregate
quantiles across instances -- the math does not work. This makes Summary
useless for computing a cluster-wide p99.
</details>

<details>
<summary>Hint 4 -- rate() requires a Counter</summary>
`rate()` and `increase()` only work on Counter metrics because they compute
the per-second increase. Applying `rate()` to a Gauge produces meaningless
results since the value can go up and down.
</details>

<details>
<summary>Hint 5 -- Cardinality explosion</summary>
The number of time series equals the product of all label value combinations.
A metric with 4 methods x 6 status codes x 1000 endpoints = 24,000 series.
Adding `user_id` with 1 million users turns that into 24 billion series.
</details>
