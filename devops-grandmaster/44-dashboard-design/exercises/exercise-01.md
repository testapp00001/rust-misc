# Exercise 01: Golden Signals, RED, and USE Methods

## Objective

Understand when to apply each monitoring methodology -- Golden Signals, RED, and
USE -- and how to map real-world system components to the correct signal set.
This is the conceptual foundation for every dashboard you will build.

## Background

Three complementary monitoring frameworks cover different layers of a system:

- **Golden Signals** (Google SRE): Latency, Traffic, Errors, Saturation --
  best for user-facing services
- **RED** (Tom Wilkie): Rate, Errors, Duration -- focused on request-driven
  microservices
- **USE** (Brendan Gregg): Utilization, Saturation, Errors -- focused on
  infrastructure resources

Choosing the wrong framework for a component means your dashboard will either
miss critical signals or drown operators in irrelevant data.

## Instructions

### Part A -- Classify each component

For each system component below, choose the **most appropriate** monitoring
methodology (Golden Signals, RED, or USE). Explain your choice in one sentence.

| Component | Methodology | Why? |
|-----------|-------------|------|
| HTTP API gateway | ? | ? |
| PostgreSQL database server | ? | ? |
| Redis cache cluster | ? | ? |
| Kubernetes worker node | ? | ? |
| Message queue (RabbitMQ) | ? | ? |
| User-facing web application | ? | ? |
| Load balancer | ? | ? |
| Network switch | ? | ? |
| Background job worker | ? | ? |
| Object storage (S3-compatible) | ? | ? |

### Part B -- Map signals to metrics

For the **HTTP API gateway** from Part A, write a PromQL query for each signal
in your chosen methodology. Assume these metrics exist:

```
http_requests_total{method, endpoint, status}       -- counter
http_request_duration_seconds{method, endpoint}      -- histogram
node_cpu_seconds_total{instance, mode}               -- counter
node_memory_MemAvailable_bytes{instance}             -- gauge
node_memory_MemTotal_bytes{instance}                 -- gauge
```

| Signal | PromQL Query |
|--------|-------------|
| Signal 1: ? | ? |
| Signal 2: ? | ? |
| Signal 3: ? | ? |
| Signal 4: ? | ? |

### Part C -- Identify missing signals

You are building a dashboard for a **Redis cache cluster**. You chose USE in
Part A. The available metrics are:

```
redis_connected_clients
redis_used_memory_bytes
redis_maxmemory_bytes
redis_commands_processed_total
redis_keyspace_hits_total
redis_keyspace_misses_total
redis_blocked_clients
redis_connected_slaves
```

1. Which USE signals can you fully cover with these metrics?
2. Which USE signals are missing or incomplete?
3. What additional metrics would you need to instrument (or export) to complete
   the USE dashboard?

### Part D -- Scenario: choosing the right dashboard

A new team member asks you: "I want to monitor our payment processing service.
Should I use Golden Signals, RED, or USE?"

The payment service:
- Serves HTTP requests from the API gateway
- Runs on 3 Kubernetes pods
- Connects to PostgreSQL for transaction storage
- Connects to Redis for idempotency keys
- Calls an external payment provider (Stripe)
- Processes webhook callbacks asynchronously

Design a monitoring plan: for each sub-component, choose a methodology and list
the signals you would monitor. Present your answer as a table.

### Part E -- When frameworks overlap

The Golden Signals include "Saturation" but RED does not. USE includes
"Utilization" and "Saturation" but not "Traffic" or "Latency."

1. If you are using RED for a microservice, how do you monitor saturation?
2. If you are using USE for a database server, how do you monitor query latency?
3. Is it valid to combine signals from multiple frameworks on a single dashboard?
   When would you do this?

## Success Criteria

- [ ] Part A classifications are correct with clear justifications
- [ ] Part B PromQL queries are syntactically correct and measure the intended signal
- [ ] Part C analysis identifies specific gaps in the available metrics
- [ ] Part D plan covers all sub-components with appropriate methodology choices
- [ ] Part E demonstrates understanding of how frameworks complement each other

## Hints

<details>
<summary>Hint 1: RED vs Golden Signals</summary>

RED is a subset of Golden Signals. RED covers Rate (Traffic), Errors, and
Duration (Latency). Golden Signals adds Saturation explicitly. RED is ideal
when you care primarily about request-driven behavior and want a simpler model.

</details>

<details>
<summary>Hint 2: USE for infrastructure</summary>

USE applies to resources with finite capacity: CPU cores, memory bytes, disk
I/O bandwidth, network bandwidth, connection pools, thread pools. If something
has a "fullness" metric, USE is likely the right choice.

</details>

<details>
<summary>Hint 3: Cache hit rate as a signal</summary>

Cache hit rate (`hits / (hits + misses)`) is not a standard RED or USE signal.
It is closer to a "traffic" or "quality" signal. Think about whether it fits
better as a custom addition to a standard framework or as part of a custom
dashboard row.

</details>

<details>
<summary>Hint 4: Histogram queries</summary>

For duration/latency histograms in PromQL:

```promql
# p99 latency
histogram_quantile(0.99, sum by (le) (rate(http_request_duration_seconds_bucket[5m])))

# Average latency
sum(rate(http_request_duration_seconds_sum[5m]))
/ sum(rate(http_request_duration_seconds_count[5m]))
```

</details>
