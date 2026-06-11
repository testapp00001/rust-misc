# Solution 03: SLO Dashboard with Error Budget Visualization

## Part A -- Dashboard layout

### Panel 1: SLO Status (Single Stat)

- **Type**: `stat`
- **Title**: "SLO Status"
- **Purpose**: At-a-glance SLO health with color coding

**PromQL**:

```promql
clamp_min(
  (1 - (avg_over_time(job:http_errors:ratio5m[30d]) / (1 - 0.999))) * 100,
  0
)
```

**Thresholds**:
- Green: > 25 (more than 25% budget remaining)
- Yellow: 10 - 25
- Red: < 10

**Value mappings**:
- Map the numeric value to text: "Healthy" / "At Risk" / "Violated"
- Display unit: `percent` (0-100)

---

### Panel 2: Error Budget Remaining (Gauge)

- **Type**: `gauge`
- **Title**: "Error Budget Remaining"
- **Purpose**: Visual representation of budget consumption

**PromQL**:

```promql
clamp_min(
  (1 - (avg_over_time(job:http_errors:ratio5m[30d]) / (1 - 0.999))) * 100,
  0
)
```

**Display settings**:
- Min: 0, Max: 100
- Unit: percent
- Thresholds:
  - Green: 25-100
  - Yellow: 10-25
  - Red: 0-10

---

### Panel 3: Error Rate Over Time (Time Series)

- **Type**: `timeseries`
- **Title**: "Error Rate vs. SLO Threshold"
- **Purpose**: Show current error rate relative to the allowed rate

**Queries**:

```promql
# Query A: 5-minute error ratio (actual)
job:http_errors:ratio5m{job="my-service"}

# Query B: SLO threshold line (allowed error rate)
(1 - 0.999)  # Returns 0.001 as a constant

# Query C: 1-hour moving average
avg_over_time(job:http_errors:ratio5m{job="my-service"}[1h])
```

**Display settings**:
- Query A: solid line, color blue
- Query B: dashed line, color red, labeled "SLO Threshold"
- Query C: solid line, color orange, opacity 50%
- Y-axis unit: `percentunit` (0.001 displays as 0.1%)
- Y-axis min: 0

---

### Panel 4: Error Budget Burndown (Time Series)

- **Type**: `timeseries`
- **Title**: "Error Budget Burndown"
- **Purpose**: Compare actual budget consumption against ideal trajectory

**Queries**:

```promql
# Query A: Actual budget consumed (minutes)
# This is a cumulative counter of error minutes over the window
(
  1 - (
    avg_over_time(job:http_errors:ratio5m[30d]) / (1 - 0.999)
  )
) * 43.2

# Query B: Ideal burndown (linear)
# At day N of 30, you should have consumed (N/30) * 43.2 minutes
# Using Grafana math expression or a recording rule:
43.2 * ((time() - (time() - (30 * 24 * 3600))) / (30 * 24 * 3600))
```

**A simpler approach for the ideal line** (using Grafana's "Reduce" or a
constant):

```promql
# Create a synthetic linear series using days elapsed
# This requires a custom recording rule:
- record: job:slo:ideal_burndown
  expr: |
    (days_in_month() - day_of_month()) / days_in_month() * 43.2
```

**Display settings**:
- Actual: solid line, color varies by value (green/yellow/red using gradient)
- Ideal: dashed line, color gray
- Y-axis: minutes remaining (countdown from 43.2 to 0)
- Area fill below the actual line, colored by threshold zones

**Threshold zones** (background colors):
- 100%-25% remaining: green background
- 25%-10% remaining: yellow background
- 10%-0% remaining: red background

---

### Panel 5: Burn Rate (Stat)

- **Type**: `stat`
- **Title**: "Current Burn Rate"
- **Purpose**: Show how fast the budget is being consumed

**PromQL**:

```promql
avg_over_time(job:http_errors:ratio5m[1h]) / (1 - 0.999)
```

**Display settings**:
- Unit: `none` (plain number)
- Thresholds:
  - Green: < 1.0 (consuming budget slower than sustainable)
  - Yellow: 1.0 - 3.0 (consuming 1x-3x the sustainable rate)
  - Red: > 3.0 (consuming more than 3x the sustainable rate)
- Sparkline: enabled (shows trend over time)

---

## Part B -- Complete PromQL query reference

| Panel | PromQL |
|-------|--------|
| SLO Status | `clamp_min((1 - (avg_over_time(job:http_errors:ratio5m[30d]) / 0.001)) * 100, 0)` |
| Budget Gauge | Same as SLO Status |
| Error Rate | `job:http_errors:ratio5m` |
| SLO Threshold | `0.001` (constant) |
| 1h Average | `avg_over_time(job:http_errors:ratio5m[1h])` |
| Burn Rate | `avg_over_time(job:http_errors:ratio5m[1h]) / 0.001` |
| Budget Burndown | `clamp_min((1 - (avg_over_time(job:http_errors:ratio5m[30d]) / 0.001)) * 43.2, 0)` |

---

## Part C -- Grafana dashboard JSON

```json
{
  "dashboard": {
    "title": "SLO Monitoring - my-service",
    "uid": "slo-monitoring",
    "time": {
      "from": "now-30d",
      "to": "now"
    },
    "refresh": "1m",
    "panels": [
      {
        "id": 1,
        "type": "stat",
        "title": "SLO Status",
        "gridPos": { "h": 6, "w": 8, "x": 0, "y": 0 },
        "targets": [
          {
            "expr": "clamp_min((1 - (avg_over_time(job:http_errors:ratio5m{job=\"my-service\"}[30d]) / 0.001)) * 100, 0)",
            "legendFormat": "Budget Remaining",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percent",
            "min": 0,
            "max": 100,
            "thresholds": {
              "mode": "absolute",
              "steps": [
                { "color": "red", "value": null },
                { "color": "yellow", "value": 10 },
                { "color": "green", "value": 25 }
              ]
            },
            "mappings": [
              { "type": "range", "options": { "from": 0, "to": 10, "result": { "text": "VIOLATED" } } },
              { "type": "range", "options": { "from": 10, "to": 25, "result": { "text": "AT RISK" } } },
              { "type": "range", "options": { "from": 25, "to": 100, "result": { "text": "HEALTHY" } } }
            ]
          }
        }
      },
      {
        "id": 2,
        "type": "gauge",
        "title": "Error Budget Remaining",
        "gridPos": { "h": 6, "w": 8, "x": 8, "y": 0 },
        "targets": [
          {
            "expr": "clamp_min((1 - (avg_over_time(job:http_errors:ratio5m{job=\"my-service\"}[30d]) / 0.001)) * 100, 0)",
            "legendFormat": "Budget",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percent",
            "min": 0,
            "max": 100,
            "thresholds": {
              "mode": "absolute",
              "steps": [
                { "color": "red", "value": null },
                { "color": "yellow", "value": 10 },
                { "color": "green", "value": 25 }
              ]
            }
          }
        }
      },
      {
        "id": 3,
        "type": "stat",
        "title": "Burn Rate",
        "gridPos": { "h": 6, "w": 8, "x": 16, "y": 0 },
        "targets": [
          {
            "expr": "avg_over_time(job:http_errors:ratio5m{job=\"my-service\"}[1h]) / 0.001",
            "legendFormat": "Burn Rate",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "none",
            "decimals": 2,
            "thresholds": {
              "mode": "absolute",
              "steps": [
                { "color": "green", "value": null },
                { "color": "yellow", "value": 1.0 },
                { "color": "red", "value": 3.0 }
              ]
            }
          }
        }
      }
    ]
  }
}
```

**Why this works**:

- The dashboard uses a consistent PromQL pattern across all panels.
- Thresholds are aligned: all use the same green/yellow/red boundaries.
- The `stat` panels are arranged in a row for quick scanning.
- The 30-day time range matches the SLO window.
- Value mappings translate raw numbers to human-readable status text.

**Common mistakes**:
- Using a different time range than the SLO window (e.g., 7d dashboard for
  a 30d SLO)
- Not using `clamp_min` which allows negative values when the SLO is violated
- Mixing up `percent` (0-100) and `percentunit` (0.0-1.0) display units

---

## Part D -- Alert annotations in Grafana

**Configuration**:

1. Navigate to the dashboard and click the gear icon (Dashboard Settings)
2. Go to "Annotations" in the left sidebar
3. Click "Add annotation query"
4. Configure:
   - **Name**: "SLO Alerts"
   - **Data source**: Prometheus
   - **Query**: `ALERTS{alertstate="firing", job="my-service"}`
   - **Icon color**: Red
   - **Show in**: All panels (or select specific panels)

**Advanced options**:

```yaml
# In Grafana provisioning (YAML):
annotations:
  list:
    - name: SLO Alerts
      datasource:
        type: prometheus
        uid: prometheus-uid
      enable: true
      iconColor: rgba(255, 96, 96, 1)
      query: ALERTS{alertstate="firing", job="my-service"}
      titleFormat: "{{ alertname }}"
      useValuesForTags: true
      tagKeys: "severity,alertname"
```

**Why this works**:

- The `ALERTS` metric is a built-in Prometheus metric that records alert
  states as time series.
- Filtering by `alertstate="firing"` shows only active alerts.
- The `job` label matches the SLO panels to the correct service.
- Annotations appear as vertical markers with tooltips showing the alert name
  and severity.

**Common mistakes**:
- Using `ALERTS` without filtering by `alertstate` (shows pending + firing)
- Not matching the `job` label to the service being monitored
- Forgetting to enable the annotation in the panel settings (it is enabled
  by default for new annotations, but can be toggled per panel)
