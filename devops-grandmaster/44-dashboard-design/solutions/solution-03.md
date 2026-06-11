# Solution 03: L2 Service Dashboard with Variables

## Part A -- Variable definitions

### Variable: `environment`

```json
{
  "name": "environment",
  "label": "Environment",
  "type": "custom",
  "query": "production,staging",
  "current": {"text": "production", "value": "production"},
  "options": [
    {"text": "production", "value": "production", "selected": true},
    {"text": "staging", "value": "staging", "selected": false}
  ],
  "multi": false,
  "includeAll": false
}
```

### Variable: `service`

```json
{
  "name": "service",
  "label": "Service",
  "type": "query",
  "datasource": {"type": "prometheus", "uid": "prometheus"},
  "query": "label_values(http_requests_total{environment=\"$environment\"}, service)",
  "current": {"text": "All", "value": "$__all"},
  "options": [],
  "multi": false,
  "includeAll": true,
  "allValue": ".*",
  "refresh": 2,
  "sort": 1
}
```

### Variable: `instance`

```json
{
  "name": "instance",
  "label": "Instance",
  "type": "query",
  "datasource": {"type": "prometheus", "uid": "prometheus"},
  "query": "label_values(up{service=\"$service\", environment=\"$environment\"}, instance)",
  "current": {"text": "All", "value": "$__all"},
  "options": [],
  "multi": false,
  "includeAll": true,
  "allValue": ".*",
  "refresh": 2,
  "sort": 1
}
```

**Key design decisions**:
- `environment` is first because `service` depends on it
- `service` depends on `environment` via the `$environment` reference in its query
- `instance` depends on both `service` and `environment`
- `refresh: 2` means "re-query when any upstream variable changes"
- `includeAll: true` with `allValue: ".*"` allows showing aggregated data across all services/instances

**Variable dependency chain**:
```
environment (custom, no dependencies)
  --> service (query, depends on environment)
    --> instance (query, depends on service and environment)
```

When the user changes `environment`, `service` re-queries. When `service` changes, `instance` re-queries. This ensures variables are always consistent.

**Common mistakes**:
- Not setting `allValue: ".*"` -- without this, "All" passes the literal string "$__all" as a label value
- Not using `refresh: 2` -- variables become stale when dependencies change
- Circular variable dependencies (e.g., service depending on instance which depends on service)

---

## Part B -- RED method panels

### Panel 1: Request Rate (time series)

```promql
sum by (method) (
  rate(http_requests_total{service="$service", environment="$environment"}[5m])
)
```

Legend format: `{{method}}`

This shows requests per second broken down by HTTP method (GET, POST, PUT, DELETE). When `$service` is "All" (set to `.*`), the regex matches all services and the query aggregates across them.

**Why `by (method)`**: Grouping by method reveals traffic patterns -- a spike in POST requests may indicate a different problem than a spike in GETs.

### Panel 2: Error Rate (gauge + time series)

**Gauge -- current error rate:**
```promql
sum(rate(http_requests_total{service="$service", environment="$environment", status=~"5.."}[5m]))
/ sum(rate(http_requests_total{service="$service", environment="$environment"}[5m]))
* 100
```

Gauge range: 0 to 100 (displayed as percentage). Thresholds:
- Green: 0 - 0.05%
- Yellow: 0.05% - 0.1%
- Red: > 0.1%

**Time series -- error rate over time:**
```promql
sum(rate(http_requests_total{service="$service", environment="$environment", status=~"5.."}[5m]))
/ sum(rate(http_requests_total{service="$service", environment="$environment"}[5m]))
* 100
```

Add a threshold line at 0.1% (the SLO target for 99.9% availability). Legend: `Error Rate`

**Common mistakes**:
- Forgetting `* 100` for percentage display
- Using `=~"5.."` without escaping dots in some contexts (though in Prometheus regex, `.` matches any character, which is what we want here for 5xx codes)
- Not handling the divide-by-zero case when there is no traffic (use `> 0` or handle gracefully)

### Panel 3: Latency Percentiles (time series)

Three targets on one graph:

**Target A (p50):**
```promql
histogram_quantile(0.50,
  sum by (le) (rate(http_request_duration_seconds_bucket{service="$service", environment="$environment"}[5m]))
)
```
Legend: `p50`

**Target B (p95):**
```promql
histogram_quantile(0.95,
  sum by (le) (rate(http_request_duration_seconds_bucket{service="$service", environment="$environment"}[5m]))
)
```
Legend: `p95`

**Target C (p99):**
```promql
histogram_quantile(0.99,
  sum by (le) (rate(http_request_duration_seconds_bucket{service="$service", environment="$environment"}[5m]))
)
```
Legend: `p99`

Unit: seconds (Grafana auto-formats to ms when values are small).

**Why three percentiles**: p50 shows typical latency, p95 shows the "bad tail," and p99 catches outliers that affect a meaningful number of users at scale. If p99 diverges far from p50, the latency distribution is skewed.

### Panel 4: Latency by Endpoint (table)

**Query A -- Request rate per endpoint:**
```promql
topk(10,
  sum by (endpoint) (rate(http_requests_total{service="$service", environment="$environment"}[5m]))
)
```

**Query B -- Error rate per endpoint:**
```promql
topk(10,
  sum by (endpoint) (rate(http_requests_total{service="$service", environment="$environment", status=~"5.."}[5m]))
  / sum by (endpoint) (rate(http_requests_total{service="$service", environment="$environment"}[5m]))
  * 100
)
```

**Query C -- p50 latency:**
```promql
topk(10,
  histogram_quantile(0.50,
    sum by (endpoint, le) (rate(http_request_duration_seconds_bucket{service="$service", environment="$environment"}[5m]))
  )
)
```

**Query D -- p95 latency:**
```promql
topk(10,
  histogram_quantile(0.95,
    sum by (endpoint, le) (rate(http_request_duration_seconds_bucket{service="$service", environment="$environment"}[5m]))
  )
)
```

**Query E -- p99 latency:**
```promql
topk(10,
  histogram_quantile(0.99,
    sum by (endpoint, le) (rate(http_request_duration_seconds_bucket{service="$service", environment="$environment"}[5m]))
  )
)
```

Use `seriesToColumns` transformation joined on `endpoint`, then `organize` to rename columns.

**Common mistakes**:
- Not using `topk` -- without it, the table shows every endpoint, which can be hundreds
- Applying `topk` independently per query -- this can select different endpoints per column. A better approach is to use `topk` only in Query A and rely on the join to match, or use a recording rule
- Forgetting `* 1000` for millisecond display on latency columns

---

## Part C -- USE method panels (infrastructure)

### Panel 5: CPU Utilization (time series)

```promql
rate(container_cpu_usage_seconds_total{instance="$instance", environment="$environment"}[5m]) * 100
```

Legend: `CPU %`

Thresholds:
- Green: 0-80%
- Yellow: 80-95%
- Red: > 95%

When `$instance` is "All" (`.*`), this shows all instances. Consider using `max by (instance)` to show the peak across instances.

### Panel 6: Memory Utilization (gauge)

```promql
container_memory_working_set_bytes{instance="$instance", environment="$environment"}
/ container_spec_memory_limit_bytes{instance="$instance", environment="$environment"}
* 100
```

Gauge range: 0-100%. Thresholds:
- Green: 0-70%
- Yellow: 70-85%
- Red: 85-100%

**Common mistakes**:
- Not handling the case where `container_spec_memory_limit_bytes` is 0 (no limit set) -- add `> 0` filter or use `clamp_min(..., 1)`
- Using `container_memory_usage_bytes` instead of `container_memory_working_set_bytes` (working set excludes cache and is what OOM killer uses)

### Panel 7: Saturation Indicators (stat panels)

**Database connection pool utilization:**
```promql
db_connection_pool_active{service="$service", environment="$environment"}
/ db_connection_pool_max{service="$service", environment="$environment"}
* 100
```

Stat display: percentage with thresholds at 70% (yellow) and 90% (red). Unit: percent.

**In-flight requests utilization:**
```promql
http_inflight_requests{service="$service", environment="$environment"}
/ http_inflight_requests_max{service="$service", environment="$environment"}
* 100
```

Stat display: percentage with thresholds at 70% (yellow) and 90% (red). Unit: percent.

**Why these two saturation indicators**: Connection pool exhaustion and in-flight request limits are the two most common saturation bottlenecks for microservices. When either hits 100%, requests start failing or queuing.

---

## Part D -- Grafana JSON for the variable dropdowns

```json
{
  "templating": {
    "list": [
      {
        "name": "environment",
        "label": "Environment",
        "type": "custom",
        "query": "production,staging",
        "current": {"text": "production", "value": "production"},
        "options": [
          {"text": "production", "value": "production", "selected": true},
          {"text": "staging", "value": "staging", "selected": false}
        ],
        "multi": false,
        "includeAll": false,
        "hide": 0
      },
      {
        "name": "service",
        "label": "Service",
        "type": "query",
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "query": "label_values(http_requests_total{environment=\"$environment\"}, service)",
        "current": {"text": "All", "value": "$__all"},
        "options": [],
        "multi": false,
        "includeAll": true,
        "allValue": ".*",
        "refresh": 2,
        "sort": 1,
        "hide": 0
      },
      {
        "name": "instance",
        "label": "Instance",
        "type": "query",
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "query": "label_values(up{service=\"$service\", environment=\"$environment\"}, instance)",
        "current": {"text": "All", "value": "$__all"},
        "options": [],
        "multi": false,
        "includeAll": true,
        "allValue": ".*",
        "refresh": 2,
        "sort": 1,
        "hide": 0
      }
    ]
  }
}
```

---

## Part E -- Panel linking and navigation

### 1. Endpoint drill-down from Panel 4

In the table panel's field overrides, add a data link on the `Endpoint` column:

```json
{
  "matcher": {"id": "byName", "options": "Endpoint"},
  "properties": [
    {
      "id": "links",
      "value": [
        {
          "title": "Drill into ${__value.text}",
          "url": "/d/l2-service?var-service=${service}&var-environment=${environment}&var-endpoint=${__value.text}",
          "targetBlank": false
        }
      ]
    }
  ]
}
```

This passes the selected endpoint as a URL parameter. If the L2 dashboard has an `endpoint` variable, it auto-populates. If not, the link can open a separate L3 endpoint detail dashboard.

### 2. Link back to L1 overview

Add a dashboard link at the top of the L2 dashboard:

```json
{
  "links": [
    {
      "title": "Platform Overview (L1)",
      "type": "link",
      "url": "/d/l1-overview?var-environment=${environment}",
      "icon": "external link",
      "tooltip": "Back to platform overview",
      "targetBlank": false
    }
  ]
}
```

### 3. Link to logs

Add a dashboard link to the relevant logs system:

**For Loki/Grafana:**
```json
{
  "title": "View Logs",
  "type": "link",
  "url": "/explore?left={\"datasource\":\"loki\",\"queries\":[{\"expr\":\"{service=\\\"$service\\\", environment=\\\"$environment\\\"}\"}]}",
  "icon": "bolt",
  "tooltip": "Open logs for ${service} in ${environment}",
  "targetBlank": true
}
```

**For ELK/Kibana:**
```
https://kibana.example.com/app/discover#/?_g=(refreshInterval:(pause:!t,value:0),time:(from:now-1h,to:now))&_a=(query:(language:kuery,query:'service:"${service}" AND environment:"${environment}"'))
```

**Why `targetBlank: true`**: Opening logs in a new tab keeps the dashboard visible so operators can correlate log entries with metric patterns.

**Common mistakes**:
- Hardcoding the environment in log links (should use the variable)
- Not URL-encoding special characters in the explore URL
- Opening log links in the same tab (loses dashboard context)
