# Exercise 05: Design a Metrics Strategy for Microservices

**Type:** Integration
**Estimated time:** 60 minutes

## Objective

Design a comprehensive metrics strategy for a microservices architecture.
This capstone exercise brings together metric types, PromQL, exporters,
alerting rules, recording rules, and dashboard design into a single coherent
plan.

## Scenario

You are the observability engineer for **Acme Corp**, a company that runs
an e-commerce platform with the following microservices:

| Service | Language | Description | Instances |
|---------|----------|-------------|-----------|
| `api-gateway` | Go | Routes requests to backend services | 3 |
| `user-service` | Python | User authentication and profiles | 2 |
| `order-service` | Go | Order processing and management | 3 |
| `inventory-service` | Rust | Stock levels and reservations | 2 |
| `payment-service` | Java | Payment processing with Stripe | 2 |
| `notification-service` | Python | Email and SMS notifications | 2 |
| `postgres` | -- | Primary database | 1 (primary + 2 replicas) |
| `redis` | -- | Caching layer | 1 (3-node cluster) |

The platform handles 5,000 requests per second at peak. The SLO is 99.9%
availability with p99 latency under 500ms for the checkout flow.

## Instructions

### Part A -- Metric Naming and Types

For each service, define the key metrics using the RED method (Rate, Errors,
Duration) plus any service-specific metrics. Provide:

1. The metric name (following Prometheus naming conventions).
2. The metric type (Counter, Gauge, Histogram).
3. The labels (keep cardinality bounded).
4. The unit.

Complete this table for at least three services:

| Service | Metric Name | Type | Labels | Unit |
|---------|-------------|------|--------|------|
| api-gateway | | | | |
| order-service | | | | |
| payment-service | | | | |
| postgres | | | | |

### Part B -- Exporter Selection

For each non-application component, identify the correct exporter and the
key metrics it provides:

1. **PostgreSQL** -- which exporter? What metrics does it provide?
2. **Redis** -- which exporter? What metrics does it provide?
3. **Node/host** -- which exporter? List 5 key metrics.
4. **Kubernetes** -- which exporter? What does kube-state-metrics provide?

### Part C -- PromQL Queries

Write PromQL queries for the following operational questions:

1. **Checkout flow success rate** -- what percentage of checkout requests
   succeed (non-5xx)? Aggregate across all services in the checkout path.

2. **Top 5 slowest endpoints** across all services by p99 latency.

3. **Database connection pool saturation** -- are we running out of
   database connections?

4. **Cache hit ratio** -- what percentage of Redis lookups are hits vs
   misses?

5. **Error budget remaining** -- given a 99.9% SLO (0.1% error budget),
   how much budget remains for the current 30-day window?

### Part D -- Alerting Rules

Design alerting rules for three severity levels:

**Critical (page the on-call engineer immediately):**
1. Service completely down (no healthy instances).
2. Error rate above 5% for 5 minutes.
3. p99 latency above 2 seconds for 10 minutes.

**Warning (notify the team in Slack):**
1. Error rate above 1% for 10 minutes.
2. p99 latency above 500ms for 15 minutes.
3. Database replica lag above 30 seconds.
4. Redis memory usage above 80%.

**Info (visible in dashboards only):**
1. Deployment detected (metric change indicating new version).
2. Traffic spike (request rate 2x the 1-hour average).

Write the complete alerting rules YAML for at least two critical and two
warning alerts.

### Part E -- Recording Rules

Identify three expensive PromQL queries from your design that should be
pre-computed using recording rules. Write the recording rules YAML.

### Part F -- Dashboard Design

Design a two-level dashboard structure:

**Level 1 -- Overview Dashboard:**
- What panels would you include?
- What queries back each panel?
- What is the refresh interval?

**Level 2 -- Service-specific Dashboard:**
- What panels would you include for the `order-service`?
- How would you make it drill-down from the overview?

Describe the dashboard layout in a text wireframe (a grid of panel names).

## Success Criteria

- [ ] Metric names follow Prometheus conventions (snake_case, base unit,
      `_total` suffix for counters).
- [ ] Labels are bounded and meaningful (no user IDs, request IDs, or
      unbounded values).
- [ ] Every non-application component has a real exporter identified.
- [ ] All PromQL queries are syntactically correct and semantically
      meaningful.
- [ ] Alerting rules have appropriate thresholds, durations, and severity
      labels.
- [ ] Recording rules pre-compute queries that are expensive or reused
      frequently.
- [ ] The dashboard design follows the RED/USE methodology.
- [ ] The error budget calculation is correct for a 99.9% SLO.
- [ ] Cardinality is managed -- you can explain the series count impact
      of your label choices.

## Hints

<details>
<summary>Hint 1 -- Prometheus naming conventions</summary>
Counters end with `_total`. Units go in the suffix: `_seconds`, `_bytes`.
Use base units (seconds, not milliseconds; bytes, not megabytes). Example:
`http_request_duration_seconds`, not `http_request_duration_ms`.
</details>

<details>
<summary>Hint 2 -- Error budget math</summary>
A 99.9% SLO allows 0.1% errors over 30 days. That is:
`30 * 24 * 60 * 60 * 0.001 = 2592 seconds` of allowed downtime or errors.
To calculate remaining budget: `1 - (actual_error_seconds / allowed_error_seconds)`.
</details>

<details>
<summary>Hint 3 -- Recording rule naming</summary>
Recording rules should be named with colons as level separators:
`level:metric:operations`. Example:
`service:http_requests:rate5m` for the 5-minute request rate per service.
</details>

<details>
<summary>Hint 4 -- Alert severity model</summary>
Critical = wake someone up (PagerDuty). Warning = notify team (Slack). Info =
dashboard annotation. The `for` duration should be longer for warnings than
criticals to avoid flapping.
</details>

<details>
<summary>Hint 5 -- Dashboard drill-down</summary>
Use Grafana template variables (e.g. `$service`) to make dashboards reusable.
The overview dashboard links to service-specific dashboards using data links.
Each service dashboard filters all queries by the selected service variable.
</details>

<details>
<summary>Hint 6 -- Database metrics</summary>
The `postgres_exporter` provides metrics like `pg_stat_activity_count`
(active connections), `pg_replication_lag` (replica delay), and
`pg_stat_database_tup_fetched` (rows fetched). For connection pool
saturation, compare active connections to the max pool size.
</details>
