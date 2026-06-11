# Exercise 05: Dashboard Hierarchy for Microservices

## Objective

Design and implement a complete three-level dashboard hierarchy for a
microservices architecture. This exercise integrates everything from the
module: L1 overview, L2 service dashboards, L3 deep-dive dashboards, Grafana
variables, dashboard linking, and dashboard provisioning as code.

## Background

A production monitoring system needs a clear navigation path:

```
L1: "Is everything OK?"
  -> Click a service
L2: "What is happening with this service?"
  -> Click a metric or component
L3: "Why is this happening?"
```

Each level has a different audience, a different time horizon, and a different
level of detail. Building this hierarchy as code (JSON files in version control)
ensures consistency, enables review, and allows automated provisioning.

## Instructions

### Part A -- Architecture overview

You are building dashboards for an e-commerce platform with these services:

| Service | SLO | Dependencies |
|---------|-----|-------------|
| `api-gateway` | 99.95% | All downstream services |
| `product-catalog` | 99.9% | PostgreSQL, Redis, S3 |
| `order-service` | 99.9% | PostgreSQL, Redis, payment-service, inventory-service |
| `payment-service` | 99.99% | PostgreSQL, external Stripe API |
| `inventory-service` | 99.9% | PostgreSQL, Redis |
| `notification-service` | 99.5% | SMTP server, Redis |

Draw (or describe in text) the dashboard hierarchy:

1. How many L1 dashboards do you need? What do they show?
2. How many L2 dashboards? One per service, or grouped?
3. What goes on L3 dashboards? How do they differ per service type?
4. How does an operator navigate between levels?

### Part B -- L1: Platform overview dashboard

Design the L1 "Platform Overview" dashboard. It should show:

1. **Overall platform health** (single stat)
2. **Per-service health table** (with links to L2 dashboards)
3. **Global error rate trend** (time series)
4. **SLO budget remaining** (per service, as a bar chart or gauge row)

Write the PromQL for panels 1 and 2. For panel 2, write the query that
produces a table with columns: Service, Rate, Errors, p99, SLO Status.

### Part C -- L2: Service dashboard (parameterized)

Design the L2 dashboard template that works for any service via the `$service`
variable. The dashboard should have:

**Row 1: Key Metrics** (4 stat panels)
- Request rate
- Error rate
- p99 latency
- SLO budget remaining

**Row 2: RED Method** (3 time series)
- Rate over time
- Errors over time (with SLO threshold)
- Latency percentiles (p50, p95, p99)

**Row 3: Dependencies** (variable number of panels)
- For each dependency of the selected service, show:
  - Connection count / pool utilization
  - Query latency (for databases)
  - Error rate (for external APIs)

**Row 4: Infrastructure** (4 gauge panels)
- CPU, Memory, Disk, Network for the selected instance

Write the PromQL for Row 1 panels using `$service` and `$environment`
variables.

### Part D -- L3: Deep-dive dashboard design

Design an L3 dashboard for the `payment-service` specifically. This dashboard
should include panels that are unique to this service:

1. **Stripe API latency** (time series): Latency of calls to the external
   Stripe API
2. **Stripe API errors** (time series): Error rate from Stripe, broken down
   by error type
3. **Transaction throughput** (time series): Successful vs failed transactions
4. **Database query performance** (table): Top 10 slowest SQL queries
5. **Connection pool saturation** (gauge): Active vs max database connections
6. **Idempotency key cache** (stat): Redis cache hit rate for idempotency keys

For panels 1, 3, and 4, write the PromQL queries. Assume these additional
metrics:

```
stripe_api_request_duration_seconds{method, endpoint, status}  -- histogram
payment_transactions_total{status}                              -- counter
db_query_duration_seconds{query, service}                       -- histogram
db_connection_pool_active{service}                              -- gauge
db_connection_pool_max{service}                                 -- gauge
redis_cache_hits_total{service, cache}                          -- counter
redis_cache_misses_total{service, cache}                        -- counter
```

### Part E -- Dashboard provisioning as code

Write the Grafana provisioning configuration that:

1. Organizes dashboards into folders: `Overview`, `Services`, `Deep-Dive`
2. Uses file-based provisioning (dashboards stored as JSON files)
3. Sets up automatic refresh from a Git repository or local directory

Write the provisioning YAML files:

```
grafana/provisioning/dashboards/dashboards.yml   -- provider config
grafana/provisioning/dashboards/overview/         -- L1 dashboards
grafana/provisioning/dashboards/services/         -- L2 dashboards
grafana/provisioning/dashboards/deep-dive/        -- L3 dashboards
```

Also write the directory structure and a brief explanation of how the
provisioning lifecycle works (when changes take effect, how to add a new
dashboard).

### Part F -- Dashboard review checklist

Write a dashboard review checklist that a team should use before merging a new
dashboard into the shared repository. The checklist should cover:

1. Technical correctness (queries, panels, variables)
2. Design quality (layout, readability, purpose)
3. Operational value (does it answer real questions?)
4. Maintenance burden (variables, dependencies, refresh rate)

Present the checklist as a markdown table with columns: Check, Criteria, Pass/Fail.

## Success Criteria

- [ ] Dashboard hierarchy has clear L1/L2/L3 separation with logical navigation
- [ ] L1 dashboard shows all services with drill-down links
- [ ] L2 dashboard is fully parameterized with Grafana variables
- [ ] L3 dashboard includes service-specific panels with correct queries
- [ ] Provisioning configuration is valid and organizes dashboards into folders
- [ ] Review checklist covers technical, design, and operational dimensions

## Hints

<details>
<summary>Hint 1: Dashboard linking pattern</summary>

Use Grafana data links and dashboard links to create the navigation hierarchy:

```json
// In L1 dashboard: link to L2
{
  "links": [
    {
      "title": "Service Dashboards",
      "type": "dashboards",
      "tags": ["l2", "service"],
      "icon": "external link"
    }
  ]
}

// In table panel: per-row link to L2
{
  "dataLinks": [
    {
      "title": "View ${__value.text} details",
      "url": "/d/l2-service?var-service=${__value.text}&var-environment=${environment}"
    }
  ]
}
```

</details>

<details>
<summary>Hint 2: Service health table query</summary>

```promql
# One query per column, joined by service label

# Column: Request Rate
sum by (service) (rate(http_requests_total{environment="$environment"}[5m]))

# Column: Error Rate
sum by (service) (rate(http_requests_total{environment="$environment", status=~"5.."}[5m]))
/ sum by (service) (rate(http_requests_total{environment="$environment"}[5m]))
* 100

# Column: p99 Latency
histogram_quantile(0.99,
  sum by (service, le) (rate(http_request_duration_seconds_bucket{environment="$environment"}[5m]))
)

# Column: SLO Status
# Use a mapping: >25% budget = Healthy, 10-25% = At Risk, <10% = Violated
```

Use Grafana "Outer join" transformation on the `service` field to merge
these into one table.

</details>

<details>
<summary>Hint 3: Provisioning directory structure</summary>

```
grafana/
  provisioning/
    dashboards/
      dashboards.yml
      overview/
        platform-overview.json
      services/
        l2-service-template.json
      deep-dive/
        payment-service-detail.json
        product-catalog-detail.json
```

The `dashboards.yml` can define multiple providers, one per folder.

</details>

<details>
<summary>Hint 4: Dependency panels with conditional logic</summary>

Use Grafana's `${__data.fields.service}` or query the dependency labels:

```promql
# Database latency for the selected service
sum by (query) (
  rate(db_query_duration_seconds_sum{service="$service"}[5m])
)
/ sum by (query) (
  rate(db_query_duration_seconds_count{service="$service"}[5m])
)
```

For services that do not have a particular dependency, the query returns
no data, and Grafana shows "No data" -- which is correct behavior.

</details>
