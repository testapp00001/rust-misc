# Exercise 02: L1 Overview Dashboard in Grafana

## Objective

Build a Level 1 (Overview) dashboard in Grafana that shows the health of all
services at a glance. An operator should be able to open this dashboard and
answer "Is everything OK?" in under 10 seconds.

## Background

A Level 1 dashboard is the entry point for all monitoring. It answers:

1. **Is the system healthy?** (top-level status)
2. **Which services are unhealthy?** (per-service status)
3. **What is the SLO situation?** (error budget consumption)
4. **Where do I look next?** (links to L2 dashboards)

It should have no more than 6-8 panels. Every panel must earn its place.

## Instructions

### Part A -- Design the dashboard layout

Design the panel layout for an L1 overview dashboard covering 5 microservices:

- `api-gateway`
- `order-service`
- `payment-service`
- `user-service`
- `notification-service`

Each service has a 99.9% SLO over 30 days.

Sketch the dashboard as a grid. For each panel, specify:
- **Panel type** (stat, gauge, table, time series, etc.)
- **Title**
- **What question it answers**
- **Grid position** (row, column, width)

Target layout: no more than 3 rows, no more than 8 panels total.

### Part B -- Write the PromQL queries

Write the PromQL for each panel. Assume these base metrics exist for all
services:

```
http_requests_total{service, status}              -- counter
http_request_duration_seconds_bucket{service, le} -- histogram
```

#### Panel 1: Overall System Health (stat)

A single panel that shows green/yellow/red for the entire system.

Write a PromQL query that returns:
- Green if all services have error budget > 25%
- Yellow if any service has error budget 10-25%
- Red if any service has error budget < 10%

#### Panel 2: Service Health Table (table)

A table with one row per service and columns for:
- Service name
- Request rate (req/s)
- Error rate (%)
- p99 latency (ms)
- SLO status (healthy / at risk / violated)

Write one PromQL query per column.

#### Panel 3: Error Rate Overview (time series)

A time-series panel showing the error rate for all 5 services on a single
graph, with a horizontal SLO threshold line.

#### Panel 4: Error Budget Gauge (gauge)

A gauge showing the minimum error budget remaining across all services
(the worst-case service).

### Part C -- Grafana JSON

Write the Grafana dashboard JSON for the **Service Health Table** panel (Panel 2).
Include:

- The datasource configuration
- All five column queries as targets
- Column value formatting (percentages, milliseconds, req/s)
- Threshold color mappings for the error rate and SLO status columns
- Data links that navigate to L2 service dashboards

Use this skeleton:

```json
{
  "id": 2,
  "type": "table",
  "title": "Service Health",
  "gridPos": {"h": 8, "w": 24, "x": 0, "y": 4},
  "datasource": {"type": "prometheus", "uid": "prometheus"},
  "targets": [],
  "fieldConfig": {
    "defaults": {},
    "overrides": []
  },
  "transformations": [],
  "options": {}
}
```

### Part D -- Dashboard-level settings

Configure the dashboard-level settings:

1. **Time range**: What default time range should an L1 dashboard use? Why?
2. **Refresh interval**: How often should the dashboard auto-refresh?
3. **Variables**: Should an L1 dashboard use variables? If so, which ones?
4. **Annotations**: What events should be overlaid on the L1 dashboard?

Write the Grafana JSON for the dashboard-level `time`, `refresh`, and
`templating` configuration.

## Success Criteria

- [ ] Dashboard layout has no more than 8 panels across 3 rows
- [ ] Every panel answers a specific operational question
- [ ] PromQL queries are syntactically correct and return meaningful data
- [ ] Table panel JSON is valid Grafana format with proper column configuration
- [ ] Dashboard-level settings are appropriate for an overview dashboard
- [ ] Links to L2 dashboards are included in the table panel

## Hints

<details>
<summary>Hint 1: System health as a single metric</summary>

Combine per-service error budget status into a single value using `min()`:

```promql
min(
  (1 - (avg by (service) (rate(http_requests_total{status=~"5.."}[30d]))
    / avg by (service) (rate(http_requests_total[30d])))
    / 0.001) * 100
)
```

If the minimum is below 10, at least one service is in the red.

</details>

<details>
<summary>Hint 2: Table with multiple queries</summary>

In Grafana, use multiple targets (A, B, C, D, E) for each column, then use
"Outer join" transformation on the `service` label to merge them into a single
table. Use "Organize fields" transformation to rename columns and hide
internal labels.

</details>

<details>
<summary>Hint 3: L1 time range</summary>

An L1 dashboard should default to a short time range (1h or 3h) with a fast
refresh (30s). Operators open it to check current health, not to analyze
historical trends. Historical analysis belongs on L2/L3 dashboards.

</details>

<details>
<summary>Hint 4: Data links to L2 dashboards</summary>

In the table panel's field overrides, add data links:

```json
{
  "matcher": {"id": "byName", "options": "Service"},
  "properties": [
    {
      "id": "links",
      "value": [
        {
          "title": "View ${__value.text} dashboard",
          "url": "/d/l2-service?var-service=${__value.text}",
          "targetBlank": false
        }
      ]
    }
  ]
}
```

</details>
