# Exercise 03: L2 Service Dashboard with Variables

## Objective

Build a Level 2 (Service) dashboard in Grafana that uses template variables
to make it reusable across services, environments, and instances. The dashboard
should implement the RED method with drill-down to infrastructure (USE).

## Background

A Level 2 dashboard answers: "What is happening with this specific service?"
It should be parameterized so the same dashboard JSON works for any service,
reducing maintenance burden and ensuring consistency.

Grafana variables let you build a single dashboard that adapts to context:

- `$environment` filters by environment (production, staging)
- `$service` selects the service to inspect
- `$instance` drills down to a specific pod/instance

## Instructions

### Part A -- Define the variables

Design the Grafana template variables for the L2 service dashboard. For each
variable, specify:

1. **Name**
2. **Type** (custom, query, interval, etc.)
3. **Query** (for query-type variables, the PromQL `label_values()` expression)
4. **Multi-select** (yes/no)
5. **Include "All" option** (yes/no)
6. **Default value**

Variables to define:

| Variable | Purpose |
|----------|---------|
| `environment` | Filter by environment |
| `service` | Select the service |
| `instance` | Select a specific pod/instance |

### Part B -- RED method panels

Design four panels implementing the RED method for the selected service. For
each panel, write the PromQL query using the variables from Part A.

#### Panel 1: Request Rate (time series)

Show requests per second, broken down by HTTP method (GET, POST, PUT, DELETE).

#### Panel 2: Error Rate (gauge + time series)

Show:
- Current error rate as a gauge (0-100%)
- Error rate over time as a time series
- SLO threshold line at the allowed error rate (0.1% for 99.9% SLO)

#### Panel 3: Latency Percentiles (time series)

Show p50, p95, and p99 latency on a single graph.

#### Panel 4: Latency by Endpoint (table)

A table showing the top 10 endpoints by p99 latency, with columns for:
- Endpoint
- Request rate
- Error rate
- p50, p95, p99 latency

### Part C -- USE method panels (infrastructure)

Add infrastructure panels below the RED panels, implementing the USE method
for the selected `$instance`.

#### Panel 5: CPU Utilization (time series)

CPU usage for the selected instance.

#### Panel 6: Memory Utilization (gauge)

Memory usage for the selected instance as a percentage.

#### Panel 7: Saturation Indicators (stat panels)

Two stat panels side by side:
- Active database connections (vs max pool size)
- In-flight requests (vs configured limit)

### Part D -- Grafana JSON for the variable dropdowns

Write the Grafana JSON `templating` section that defines all three variables.
The `environment` variable should be a custom type with values
`production,staging`. The `service` and `instance` variables should be query
type, dynamically populated from Prometheus labels.

### Part E -- Panel linking and navigation

Configure the dashboard so that:

1. Clicking on an endpoint row in Panel 4 opens a new panel or dashboard
   filtered to that endpoint
2. The dashboard includes a link back to the L1 overview dashboard
3. The dashboard includes a link to the relevant logs (e.g., Loki or ELK)

Describe the configuration for each link. For the log link, use a template
that constructs the URL from the `$service` and `$environment` variables.

## Success Criteria

- [ ] Variables are correctly defined with appropriate types and queries
- [ ] All RED panels use the `$service` variable in their queries
- [ ] USE panels use the `$instance` variable for per-instance filtering
- [ ] PromQL queries are syntactically correct and return meaningful data
- [ ] Grafana JSON templating section is valid and complete
- [ ] Navigation links connect L2 back to L1 and forward to logs/traces

## Hints

<details>
<summary>Hint 1: Query-type variable syntax</summary>

```json
{
  "name": "service",
  "type": "query",
  "datasource": {"type": "prometheus", "uid": "prometheus"},
  "query": "label_values(http_requests_total{environment=\"$environment\"}, service)",
  "multi": false,
  "includeAll": true,
  "allValue": ".*",
  "current": {"text": "All", "value": "$__all"}
}
```

</details>

<details>
<summary>Hint 2: Error rate with variables</summary>

```promql
sum(rate(http_requests_total{service="$service", environment="$environment", status=~"5.."}[5m]))
/ sum(rate(http_requests_total{service="$service", environment="$environment"}[5m]))
* 100
```

When `$service` is "All" (set `allValue` to `.*`), the regex matches everything.

</details>

<details>
<summary>Hint 3: Latency percentiles on one graph</summary>

Use three targets with different `legendFormat`:

```
Target A: histogram_quantile(0.50, sum by (le) (rate(http_request_duration_seconds_bucket{service="$service"}[5m])))
  legendFormat: "p50"

Target B: histogram_quantile(0.95, sum by (le) (rate(http_request_duration_seconds_bucket{service="$service"}[5m])))
  legendFormat: "p95"

Target C: histogram_quantile(0.99, sum by (le) (rate(http_request_duration_seconds_bucket{service="$service"}[5m])))
  legendFormat: "p99"
```

</details>

<details>
<summary>Hint 4: Instance variable dependency</summary>

The `instance` variable should depend on `service` so it only shows instances
belonging to the selected service:

```json
{
  "name": "instance",
  "type": "query",
  "query": "label_values(up{service=\"$service\", environment=\"$environment\"}, instance)",
  "refresh": 2
}
```

Setting `refresh: 2` means the variable re-queries when another variable changes.

</details>
