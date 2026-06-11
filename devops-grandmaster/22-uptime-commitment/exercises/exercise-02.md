# Exercise 02: Define SLIs and SLOs for a Web Application

**Type:** Guided
**Objective:** Select appropriate Service Level Indicators (SLIs) and set
realistic Service Level Objectives (SLOs) for a multi-tier web application.

## Background

A Service Level Indicator (SLI) is a quantitative measure of a service aspect
that matters to users. A Service Level Objective (SLO) is the target value for
an SLI. Together they form the foundation of any reliability program.

You have been given a description of a web application. Your job is to identify
the right SLIs, define how to measure them, and set SLO targets.

## The Application

**"ShopStream"** is an e-commerce platform with the following components:

- **Frontend:** A React single-page application served via CDN.
- **API Gateway:** Routes requests to backend services. Handles authentication,
  rate limiting, and request routing.
- **Product Catalog Service:** Reads product data from a PostgreSQL database.
  Read-heavy workload.
- **Order Service:** Processes purchases. Writes to PostgreSQL and publishes
  events to a message queue.
- **Search Service:** Powers product search via Elasticsearch.
- **Payment Service:** Integrates with external payment providers (Stripe,
  PayPal). Has inherent third-party latency.
- **Notification Service:** Sends order confirmations via email and SMS.
  Asynchronous, not user-facing in real time.

## Instructions

### Part A -- Identify Candidate SLIs

For **each** of the four user-facing flows listed below, list 2-3 candidate
SLIs. Be specific about what you are measuring.

1. **Browsing products** (user searches or navigates the catalog)
2. **Viewing a product page** (user loads a single product's details)
3. **Placing an order** (user completes checkout)
4. **Checking order status** (user views their order history)

For each SLI, specify:
- **What** is being measured (e.g., latency of API response, ratio of successful
  requests)
- **Where** in the system it is measured (client-side, API gateway, database)
- **How** it is collected (e.g., application metrics, synthetic probes, logs)

### Part B -- Define SLOs

For each SLI you identified, set an SLO. Use the following format:

```
SLI: [description]
Method: [how it is measured]
SLO: [target] for [percentage]% of [time window]
Rationale: [why this target makes sense]
```

Use one of these common SLO structures:
- **Latency:** "Xth percentile latency is below Y ms"
- **Availability:** "Ratio of successful requests is at least X%"
- **Throughput:** "System can handle at least X requests per second"
- **Correctness:** "X% of operations produce the correct result"

### Part C -- Classify by Criticality

Rank each SLO as **Critical**, **High**, or **Medium** priority. Justify your
ranking based on:
- Direct user impact if the SLO is missed
- Revenue impact
- Whether the component has external dependencies (e.g., payment provider)

## Success Criteria

- [ ] At least 2 SLIs defined per user-facing flow (8 total minimum).
- [ ] Each SLI includes what, where, and how.
- [ ] Each SLO has a specific numeric target and time window.
- [ ] At least one SLO uses a percentile-based latency target.
- [ ] At least one SLO uses a ratio-based availability target.
- [ ] Part C classifies all SLOs with a justified reason.

## Hints

<details>
<summary>Hint 1 -- Good SLIs are user-centric</summary>

A good SLI answers: "What does the user experience?" For example, users do not
care about CPU utilization. They care whether the page loads in under 2 seconds.
Measure from the user's perspective, not the system's.

</details>

<details>
<summary>Hint 2 -- Latency SLIs need a percentile</summary>

Do not use average latency -- it hides outliers. Use p50 (median), p95, or p99.
For user-facing requests, p99 is common: "99% of requests complete within X ms."
For background jobs, p95 may be sufficient.

</details>

<details>
<summary>Hint 3 -- Availability is a ratio</summary>

Define availability as:

```
availability = successful_requests / total_requests
```

"Total requests" should exclude client-side errors (4xx) that are the user's
fault. Only count requests that reached your infrastructure.

</details>

<details>
<summary>Hint 4 -- External dependencies matter</summary>

The Payment Service depends on Stripe/PayPal. If Stripe has an outage, your
payment SLO may be violated through no fault of your own. Consider whether to
measure availability including or excluding third-party failures, or to set a
separate, more lenient SLO for flows that depend on external services.

</details>
