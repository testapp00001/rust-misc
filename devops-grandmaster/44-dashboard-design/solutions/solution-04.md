# Solution 04: Incident Response Dashboard

## Part A -- Incident dashboard layout

### Row Design

| Row | Question | Panels |
|-----|----------|--------|
| 1 | What is the incident status? | Incident Status (stat), Duration (stat), Affected Users (stat) |
| 2 | What is the blast radius? | Affected Requests (stat), Error Rate by Service (bar chart), Error Rate Over Time (time series) |
| 3 | What changed recently? | Deployment Annotations (annotation overlay), Config Change Events (table), Traffic Pattern Shift (time series) |
| 4 | Which component is failing? | Dependency Health (state timeline), Error Breakdown by Type (time series), Slowest Endpoints (table) |
| 5 | What is the SLO impact? | Error Budget Consumed Today (gauge), Budget Consumed vs 30d (gauge), Time to Budget Exhaustion (stat) |

### Row 1: Incident Status

| Panel | Type | Title | Purpose |
|-------|------|-------|---------|
| 1a | stat | Incident Status | Shows current phase (Investigating / Identified / Monitoring / Resolved) |
| 1b | stat | Duration | Time since incident started |
| 1c | stat | Affected Users | Estimated impacted user count |

Row time range: current (auto-refresh 10s)

### Row 2: Blast Radius

| Panel | Type | Title | Purpose |
|-------|------|-------|---------|
| 2a | stat | Failed Requests (15m) | Total 5xx count with comparison to yesterday |
| 2b | barchart | Error Rate by Service | Top 5 services by error rate |
| 2c | timeseries | Error Rate Trend | Error rate over the incident window |

Row time range: 1h (to see the incident progression)

### Row 3: What Changed

| Panel | Type | Title | Purpose |
|-------|------|-------|---------|
| 3a | timeseries | Request Rate with Annotations | Traffic pattern with deployment overlays |
| 3b | table | Recent Events | Deployment, config change, scaling events |
| 3c | timeseries | Latency Shift | Latency before/after the change point |

Row time range: 6h (to see pre-incident baseline)

### Row 4: Component Failure

| Panel | Type | Title | Purpose |
|-------|------|-------|---------|
| 4a | statetimeline | Dependency Health | Up/down status of critical dependencies |
| 4b | timeseries | Error Breakdown by Status | 500, 502, 503, 504 separated |
| 4c | table | Slowest Endpoints | Endpoints with highest latency or error rate |

Row time range: 1h

### Row 5: SLO Impact

| Panel | Type | Title | Purpose |
|-------|------|-------|---------|
| 5a | gauge | Budget Consumed Today | Error budget spent in the current day |
| 5b | gauge | Budget Remaining (30d) | Rolling 30-day budget for comparison |
| 5c | stat | Time to Exhaustion | At current burn rate, when will budget be zero? |

Row time range: 30d (for SLO calculations)

**Why this structure works**:
- Row 1 gives immediate context to anyone joining the incident
- Row 2 quantifies impact (the "how bad is it?" question)
- Row 3 helps identify the trigger (the "what changed?" question)
- Row 4 narrows down the root cause (the "where is it broken?" question)
- Row 5 quantifies business impact (the "how much damage?" question)

**Common mistakes**:
- Making the dashboard too general-purpose (incidents need focus)
- Not including comparison queries (hard to know what "normal" looks like)
- Mixing investigation panels with status panels (keep them in separate rows)

---

## Part B -- PromQL queries

### Blast Radius Panel: Affected Requests (stat)

**Current period (last 15 minutes):**
```promql
sum(increase(http_requests_total{status=~"5.."}[15m]))
```

**Yesterday same period (for comparison):**
```promql
sum(increase(http_requests_total{status=~"5.."}[15m] offset 1d))
```

**Percentage change:**
```promql
(
  sum(increase(http_requests_total{status=~"5.."}[15m]))
  - sum(increase(http_requests_total{status=~"5.."}[15m] offset 1d))
)
/ sum(increase(http_requests_total{status=~"5.."}[15m] offset 1d))
* 100
```

Display: the current count as the main value, the percentage change as a secondary value or sparkline.

### Blast Radius Panel: Error Rate by Service (bar chart)

```promql
topk(5,
  sum by (service) (
    rate(http_requests_total{status=~"5.."}[5m])
  )
  / sum by (service) (
    rate(http_requests_total[5m])
  )
  * 100
)
```

This shows the 5 services with the highest error rate, sorted descending.

### What Changed Panel: Deployment Timeline (annotations)

**Kubernetes deployment events:**
```promql
changes(kube_deployment_status_replicas_updated[5m]) > 0
```

Annotation configuration:
```json
{
  "name": "Deployments",
  "datasource": {"type": "prometheus", "uid": "prometheus"},
  "enable": true,
  "iconColor": "blue",
  "expr": "changes(kube_deployment_status_replicas_updated[5m]) > 0",
  "titleFormat": "Deployment: {{deployment}} in {{namespace}}"
}
```

**ConfigMap/Secret changes:**
```promql
changes(kube_configmap_metadata_resource_version[5m]) > 0
```

```json
{
  "name": "Config Changes",
  "datasource": {"type": "prometheus", "uid": "prometheus"},
  "enable": true,
  "iconColor": "orange",
  "expr": "changes(kube_configmap_metadata_resource_version[5m]) > 0",
  "titleFormat": "ConfigMap changed: {{configmap}}"
}
```

**Horizontal Pod Autoscaler events:**
```promql
changes(kube_horizontalpodautoscaler_status_current_replicas[5m]) > 0
```

```json
{
  "name": "HPA Scaling",
  "datasource": {"type": "prometheus", "uid": "prometheus"},
  "enable": true,
  "iconColor": "purple",
  "expr": "changes(kube_horizontalpodautoscaler_status_current_replicas[5m]) > 0",
  "titleFormat": "HPA scaling: {{horizontalpodautoscaler}} to {{value}} replicas"
}
```

### Component Failure Panel: Dependency Health (state timeline)

```promql
up{job=~"postgres-primary|postgres-replica|redis-primary|payment-api"}
```

This returns 1 for healthy and 0 for down. Display as a `state_timeline` panel:
- Value 1: green (UP)
- Value 0: red (DOWN)

**More detailed health check (if `up` is not sufficient):**
```promql
# PostgreSQL with actual query check
probe_success{job="postgres-health-check"}

# Redis with ping check
probe_success{job="redis-health-check"}

# External API with HTTP health endpoint
probe_success{job="payment-api-health-check"}
```

### SLO Impact Panel: Budget Consumed Today (gauge)

```promql
# Error budget consumed today (out of 100%)
(
  sum(increase(http_requests_total{status=~"5.."}[24h]))
  / sum(increase(http_requests_total[24h]))
  / 0.001  -- SLO target: 99.9% means 0.1% error budget
) * 100
```

Gauge range: 0-100%. Thresholds:
- Green: 0-50%
- Yellow: 50-80%
- Red: > 80%

If the gauge exceeds 100%, the SLO for today is already violated.

**Common mistakes**:
- Using `[24h]` when the day is not over yet (the budget calculation is incomplete)
- Not normalizing by the SLO target (0.001 for 99.9%)
- Showing 30-day budget instead of daily budget (they answer different questions)

---

## Part C -- Comparison queries

### 1. Current error rate vs. same time yesterday

```promql
# Current
sum(rate(http_requests_total{status=~"5.."}[5m]))

# Yesterday
sum(rate(http_requests_total{status=~"5.."}[5m] offset 1d))
```

**Percentage change:**
```promql
(
  sum(rate(http_requests_total{status=~"5.."}[5m]))
  - sum(rate(http_requests_total{status=~"5.."}[5m] offset 1d))
)
/ sum(rate(http_requests_total{status=~"5.."}[5m] offset 1d))
* 100
```

### 2. Current latency vs. same time last week

```promql
# Current p99
histogram_quantile(0.99,
  sum by (le) (rate(http_request_duration_seconds_bucket[5m]))
)

# Last week p99
histogram_quantile(0.99,
  sum by (le) (rate(http_request_duration_seconds_bucket[5m] offset 7d))
)
```

**Ratio (how many times worse):**
```promql
histogram_quantile(0.99,
  sum by (le) (rate(http_request_duration_seconds_bucket[5m]))
)
/
histogram_quantile(0.99,
  sum by (le) (rate(http_request_duration_seconds_bucket[5m] offset 7d))
)
```

### 3. Current request rate vs. average of the last 7 days

```promql
# Current request rate
sum(rate(http_requests_total[5m]))

# Average of last 7 days at the same time
avg_over_time(
  (
    sum(rate(http_requests_total[5m]))
  )[7d:5m]
)
```

Note: The `[7d:5m]` range with step `5m` creates a subquery that samples every 5 minutes over the past 7 days. `avg_over_time` then computes the average of those samples.

**Common mistakes**:
- Using `offset 1d` without considering timezone differences
- Forgetting that `offset` shifts the entire query window (the `[5m]` still looks at 5 minutes, just 1 day earlier)
- Using `avg_over_time` on a range vector without understanding subquery semantics

---

## Part D -- Dashboard JSON for the status row

```json
{
  "id": 1,
  "type": "row",
  "title": "Incident Status",
  "gridPos": {"h": 1, "w": 24, "x": 0, "y": 0},
  "panels": [
    {
      "id": 10,
      "type": "stat",
      "title": "Incident Status",
      "gridPos": {"h": 4, "w": 8, "x": 0, "y": 1},
      "datasource": {"type": "prometheus", "uid": "prometheus"},
      "targets": [
        {
          "refId": "A",
          "expr": "1",
          "instant": true
        }
      ],
      "fieldConfig": {
        "defaults": {
          "mappings": [
            {
              "type": "value",
              "options": {
                "1": {"text": "${incident_status}", "color": "orange"}
              }
            }
          ],
          "thresholds": {
            "steps": [
              {"color": "orange", "value": null}
            ]
          }
        },
        "overrides": []
      },
      "options": {
        "reduceOptions": {
          "calcs": ["lastNotNull"]
        },
        "textMode": "value",
        "colorMode": "background"
      }
    },
    {
      "id": 11,
      "type": "stat",
      "title": "Duration",
      "gridPos": {"h": 4, "w": 8, "x": 8, "y": 1},
      "datasource": {"type": "prometheus", "uid": "prometheus"},
      "targets": [
        {
          "refId": "A",
          "expr": "time() - min_over_time((timestamp(changes(http_requests_total{status=~\"5..\"}[1m]) > 0))[1h:1m])",
          "instant": true
        }
      ],
      "fieldConfig": {
        "defaults": {
          "unit": "s",
          "thresholds": {
            "steps": [
              {"color": "green", "value": null},
              {"color": "yellow", "value": 900},
              {"color": "red", "value": 3600}
            ]
          }
        },
        "overrides": []
      },
      "options": {
        "reduceOptions": {
          "calcs": ["lastNotNull"]
        },
        "textMode": "value",
        "colorMode": "value"
      }
    },
    {
      "id": 12,
      "type": "stat",
      "title": "Affected Users (est.)",
      "gridPos": {"h": 4, "w": 8, "x": 16, "y": 1},
      "datasource": {"type": "prometheus", "uid": "prometheus"},
      "targets": [
        {
          "refId": "A",
          "expr": "sum(increase(http_requests_total{status=~\"5..\"}[15m])) * 2.5",
          "instant": true
        }
      ],
      "fieldConfig": {
        "defaults": {
          "unit": "short",
          "thresholds": {
            "steps": [
              {"color": "green", "value": null},
              {"color": "yellow", "value": 1000},
              {"color": "red", "value": 10000}
            ]
          }
        },
        "overrides": []
      },
      "options": {
        "reduceOptions": {
          "calcs": ["lastNotNull"]
        },
        "textMode": "value",
        "colorMode": "value"
      }
    }
  ],
  "templating": {
    "list": [
      {
        "name": "incident_status",
        "type": "textbox",
        "label": "Incident Status",
        "current": {"text": "Investigating", "value": "Investigating"}
      }
    ]
  }
}
```

**How the panels work**:
- **Incident Status**: Displays the value of the `$incident_status` variable. The incident commander updates this variable as the incident progresses through phases.
- **Duration**: Calculates time since the first error spike by finding the minimum timestamp of error rate changes in the last hour.
- **Affected Users**: Estimates user impact by multiplying failed requests by 2.5 (assuming average 2.5 requests per user session -- adjust based on your traffic patterns).

**Common mistakes**:
- Hardcoding incident status instead of using a variable (requires dashboard edit to update)
- Using a complex duration calculation that breaks when there are no errors (add fallback)
- Showing exact user counts when only estimates are possible (add "(est.)" to the title)

---

## Part E -- Incident workflow integration

### 1. Automatic dashboard switching on alert fire

Configure the Alertmanager notification to include a link to the incident dashboard:

```yaml
# alertmanager.yml
receivers:
  - name: incident-response
    webhook_configs:
      - url: 'http://grafana:3000/api/alerts/notify'
    pagerduty_configs:
      - service_key: '<key>'
        description: '{{ .CommonAnnotations.summary }}'
        details:
          dashboard_url: 'http://grafana:3000/d/incident-response?var-incident_status=Investigating&from=now-1h&to=now'
```

In Grafana alert rules, configure the dashboard UID and panel ID:

```yaml
# grafana alert rule
annotations:
  dashboard_uid: "incident-response"
  panel_id: "10"
```

When an alert fires, the notification includes a one-click link that opens the incident dashboard with the relevant time range pre-set.

For PagerDuty or Opsgenie integrations, embed the dashboard URL in the incident description so responders can click through directly from the alert.

### 2. Persisting incident timeline for post-mortem

**During the incident:**
- Annotations added via the Grafana UI (or API) are stored in Grafana's database
- Use the Grafana API to programmatically create annotations:

```bash
curl -X POST http://grafana:3000/api/annotations \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <api-key>" \
  -d '{
    "dashboardUID": "incident-response",
    "time": 1620000000000,
    "text": "Investigation started - elevated 5xx on payment-service",
    "tags": ["incident", "start"]
  }'
```

**After the incident:**
- Export the dashboard snapshot (Grafana > Share > Snapshot) to preserve the exact state
- Use the Grafana API to query all annotations in the incident time window:

```bash
curl "http://grafana:3000/api/annotations?dashboardUID=incident-response&from=1620000000000&to=1620036000000" \
  -H "Authorization: Bearer <api-key>"
```

- Include the exported annotations in the post-mortem document
- Store the snapshot URL in the incident ticket for future reference

### 3. Sharing with non-Grafana stakeholders

**Option 1: Grafana Snapshot**
- Create a snapshot (Grafana > Share > Snapshot)
- Snapshots are publicly accessible without authentication
- Store on `snapshots.raintank.io` or your own Grafana instance
- Share the URL in Slack/email

**Option 2: Rendered Image**
- Use the Grafana image renderer plugin to generate PNG images
- Schedule automated screenshots via the Grafana API:

```bash
curl "http://grafana:3000/render/d/incident-response?var-incident_status=Identified&from=now-2h&to=now&width=1200&height=800" \
  -o incident-dashboard.png
```

- Attach the image to incident updates

**Option 3: Public Dashboard**
- Grafana 9+ supports public dashboards (no login required)
- Enable for the incident dashboard during the incident
- Disable after resolution to maintain security

**Option 4: Automated Reports**
- Use Grafana reporting (Enterprise feature) to send PDF reports
- Schedule a report for the incident time range
- Distribute via email to stakeholders

**Best practice**: Use snapshots for real-time sharing during incidents, and rendered images or reports for post-mortem documentation.
