# Solution 02: L1 Overview Dashboard in Grafana

## Part A -- Dashboard layout

### Panel Grid Design

```
Row 1 (y=0):  [Overall Health - stat, w=6] [Error Budget Min - gauge, w=6] [Total Request Rate - stat, w=6] [Total Error Rate - stat, w=6]
Row 2 (y=4):  [Service Health Table - table, w=24]
Row 3 (y=12): [Error Rate by Service - time series, w=12] [Latency p99 by Service - time series, w=12]
```

| Panel | Type | Title | Question Answered | Grid Position |
|-------|------|-------|-------------------|---------------|
| 1 | stat | Overall System Health | "Is everything OK?" | Row 1, x=0, w=6, h=4 |
| 2 | gauge | Worst Error Budget | "Is any SLO at risk?" | Row 1, x=6, w=6, h=4 |
| 3 | stat | Total Request Rate | "How much traffic?" | Row 1, x=12, w=6, h=4 |
| 4 | stat | Total Error Rate | "Are errors elevated?" | Row 1, x=18, w=6, h=4 |
| 5 | table | Service Health | "Which service is unhealthy?" | Row 2, x=0, w=24, h=8 |
| 6 | timeseries | Error Rate by Service | "Are errors trending up?" | Row 3, x=0, w=12, h=8 |
| 7 | timeseries | p99 Latency by Service | "Is latency degrading?" | Row 3, x=12, w=12, h=8 |

Total: 7 panels, 3 rows. Within the 8-panel limit.

**Why this layout works**:
- Row 1 provides an instant "is everything OK?" answer with four high-level indicators
- Row 2 is the primary operational tool -- the service health table where you identify which service needs attention
- Row 3 provides trend context -- are things getting better or worse?

**Common mistakes**:
- Too many panels (L1 should be scannable in under 10 seconds)
- Including detailed breakdowns that belong on L2 dashboards
- Missing the "where do I look next?" navigation links in the table

---

## Part B -- PromQL queries

### Panel 1: Overall System Health (stat)

This query returns a single value: the minimum error budget percentage across all services. Map to green/yellow/red thresholds.

```promql
min(
  (
    1 -
    (
      sum by (service) (rate(http_requests_total{status=~"5.."}[30d]))
      / sum by (service) (rate(http_requests_total[30d]))
    )
    / 0.001
  )
) * 100
```

**How it works**:
1. Calculate per-service error rate over 30 days
2. Divide by the SLO target (0.001 = 0.1% for 99.9% SLO) to get budget consumption ratio
3. Subtract from 1 to get remaining budget
4. Take the minimum (worst service)
5. Multiply by 100 for percentage display

**Thresholds** (in fieldConfig):
- Green: > 25% budget remaining
- Yellow: 10-25% budget remaining
- Red: < 10% budget remaining

### Panel 2: Service Health Table (table)

Five queries, one per column, joined by the `service` label.

**Query A -- Request Rate:**
```promql
sum by (service) (rate(http_requests_total{environment="production"}[5m]))
```

**Query B -- Error Rate (%):**
```promql
sum by (service) (rate(http_requests_total{environment="production", status=~"5.."}[5m]))
/ sum by (service) (rate(http_requests_total{environment="production"}[5m]))
* 100
```

**Query C -- p99 Latency (ms):**
```promql
histogram_quantile(0.99,
  sum by (service, le) (rate(http_request_duration_seconds_bucket{environment="production"}[5m]))
) * 1000
```

**Query D -- SLO Status (budget remaining %):**
```promql
(
  1 -
  (
    sum by (service) (rate(http_requests_total{environment="production", status=~"5.."}[30d]))
    / sum by (service) (rate(http_requests_total{environment="production"}[30d]))
  )
  / 0.001
) * 100
```

Then use Grafana value mappings to convert numeric values to labels:
- \> 25 --> "Healthy" (green)
- 10-25 --> "At Risk" (yellow)
- < 10 --> "Violated" (red)

**Common mistakes**:
- Not multiplying latency by 1000 for millisecond display
- Forgetting to multiply error rate by 100 for percentage
- Using 5m window for SLO status (should be 30d to match SLO period)

### Panel 3: Error Rate Overview (time series)

```promql
sum by (service) (rate(http_requests_total{environment="production", status=~"5.."}[5m]))
/ sum by (service) (rate(http_requests_total{environment="production"}[5m]))
* 100
```

Legend format: `{{service}}`

Add a horizontal threshold line at 0.1% (the SLO target):

```json
"thresholds": {
  "steps": [
    {"color": "green", "value": null},
    {"color": "red", "value": 0.1}
  ]
}
```

### Panel 4: Error Budget Gauge (gauge)

```promql
min(
  (
    1 -
    (
      sum by (service) (rate(http_requests_total{status=~"5.."}[30d]))
      / sum by (service) (rate(http_requests_total[30d]))
    )
    / 0.001
  )
) * 100
```

Same query as Panel 1, but displayed as a gauge with 0-100 range.

---

## Part C -- Grafana JSON for the Service Health Table

```json
{
  "id": 2,
  "type": "table",
  "title": "Service Health",
  "gridPos": {"h": 8, "w": 24, "x": 0, "y": 4},
  "datasource": {"type": "prometheus", "uid": "prometheus"},
  "targets": [
    {
      "refId": "A",
      "expr": "sum by (service) (rate(http_requests_total{environment=\"production\"}[5m]))",
      "legendFormat": "{{service}}",
      "format": "table",
      "instant": true
    },
    {
      "refId": "B",
      "expr": "sum by (service) (rate(http_requests_total{environment=\"production\", status=~\"5..\"}[5m])) / sum by (service) (rate(http_requests_total{environment=\"production\"}[5m])) * 100",
      "legendFormat": "{{service}}",
      "format": "table",
      "instant": true
    },
    {
      "refId": "C",
      "expr": "histogram_quantile(0.99, sum by (service, le) (rate(http_request_duration_seconds_bucket{environment=\"production\"}[5m]))) * 1000",
      "legendFormat": "{{service}}",
      "format": "table",
      "instant": true
    },
    {
      "refId": "D",
      "expr": "(1 - (sum by (service) (rate(http_requests_total{environment=\"production\", status=~\"5..\"}[30d])) / sum by (service) (rate(http_requests_total{environment=\"production\"}[30d]))) / 0.001) * 100",
      "legendFormat": "{{service}}",
      "format": "table",
      "instant": true
    }
  ],
  "transformations": [
    {
      "id": "seriesToColumns",
      "options": {"byField": "service"}
    },
    {
      "id": "organize",
      "options": {
        "excludeByName": {
          "Time": true,
          "Time 1": true,
          "Time 2": true,
          "Time 3": true,
          "Time 4": true
        },
        "renameByName": {
          "service": "Service",
          "Value #A": "Request Rate (req/s)",
          "Value #B": "Error Rate (%)",
          "Value #C": "p99 Latency (ms)",
          "Value #D": "SLO Budget (%)"
        }
      }
    }
  ],
  "fieldConfig": {
    "defaults": {},
    "overrides": [
      {
        "matcher": {"id": "byName", "options": "Error Rate (%)"},
        "properties": [
          {
            "id": "custom.cellOptions",
            "value": {"type": "color-background"}
          },
          {
            "id": "thresholds",
            "value": {
              "mode": "absolute",
              "steps": [
                {"color": "green", "value": null},
                {"color": "yellow", "value": 0.05},
                {"color": "red", "value": 0.1}
              ]
            }
          },
          {
            "id": "unit",
            "value": "percent"
          }
        ]
      },
      {
        "matcher": {"id": "byName", "options": "SLO Budget (%)"},
        "properties": [
          {
            "id": "custom.cellOptions",
            "value": {"type": "color-background"}
          },
          {
            "id": "thresholds",
            "value": {
              "mode": "absolute",
              "steps": [
                {"color": "red", "value": null},
                {"color": "yellow", "value": 10},
                {"color": "green", "value": 25}
              ]
            }
          },
          {
            "id": "unit",
            "value": "percent"
          }
        ]
      },
      {
        "matcher": {"id": "byName", "options": "p99 Latency (ms)"},
        "properties": [
          {"id": "unit", "value": "ms"}
        ]
      },
      {
        "matcher": {"id": "byName", "options": "Request Rate (req/s)"},
        "properties": [
          {"id": "unit", "value": "reqps"}
        ]
      },
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
    ]
  },
  "options": {
    "showHeader": true,
    "sortBy": [{"displayName": "SLO Budget (%)", "desc": true}]
  }
}
```

**How this works**:
- `seriesToColumns` transformation pivots the per-query results into columns keyed by the `service` field
- `organize` transformation renames columns and hides internal Time fields from each target
- Field overrides apply per-column formatting (units, thresholds, cell colors)
- Data links on the Service column enable drill-down to L2 dashboards

**Common mistakes**:
- Not using `instant: true` for table queries (causes multiple time-series rows per service instead of one row)
- Forgetting to hide the Time column from each target
- Not using `seriesToColumns` with `byField: "service"` (results in misaligned rows)

---

## Part D -- Dashboard-level settings

### 1. Time range

Default: **3 hours**. Rationale: L1 is for current health checks, not historical analysis. 3 hours is long enough to see short-term trends but short enough to show current state clearly. During incidents, operators may zoom out to 6h or 24h.

### 2. Refresh interval

Default: **30 seconds**. Rationale: Fast enough to catch issues quickly, slow enough to avoid excessive Prometheus query load. During incidents, operators can manually set 10s or 5s if needed.

### 3. Variables

An L1 dashboard should have minimal variables:
- `environment` (custom: production, staging) -- to switch between environments
- No service variable -- L1 shows all services by design

### 4. Annotations

- **Deployments**: Show when new versions were deployed (from CI/CD events or Kubernetes)
- **Alerts**: Show when alerts fired (from Alertmanager)
- **Incidents**: Show incident start/end times (from incident management system)

### Dashboard-level JSON

```json
{
  "time": {
    "from": "now-3h",
    "to": "now"
  },
  "refresh": "30s",
  "templating": {
    "list": [
      {
        "name": "environment",
        "type": "custom",
        "query": "production,staging",
        "current": {"text": "production", "value": "production"},
        "options": [
          {"text": "production", "value": "production", "selected": true},
          {"text": "staging", "value": "staging", "selected": false}
        ],
        "includeAll": false,
        "multi": false
      }
    ]
  },
  "annotations": {
    "list": [
      {
        "name": "Deployments",
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "enable": true,
        "iconColor": "blue",
        "expr": "changes(kube_deployment_status_replicas_updated{environment=\"$environment\"}[5m]) > 0",
        "titleFormat": "Deployment: {{deployment}}"
      },
      {
        "name": "Alerts",
        "datasource": {"type": "alertmanager", "uid": "alertmanager"},
        "enable": true,
        "iconColor": "red",
        "type": "dashboard"
      }
    ]
  }
}
```

**Why these settings**:
- 3h default captures recent context without overwhelming the view
- 30s refresh balances timeliness against query cost
- Only `environment` as a variable keeps L1 simple and focused
- Deployment and alert annotations help correlate metric changes with real-world events

**Common mistakes**:
- Making L1 too complex with many variables (defeats the "at a glance" purpose)
- Setting refresh too fast (< 15s) causing unnecessary load on Prometheus
- Not including deployment annotations (makes it hard to correlate issues with changes)
