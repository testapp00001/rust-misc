# Solution 05: Build an SLO Dashboard with Prometheus and Grafana

## Part 1 -- Deploy the Sample Application

### app-deployment.yaml

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: slo-sample-app
  labels:
    app: slo-sample-app
    service: slo-sample-app
spec:
  replicas: 2
  selector:
    matchLabels:
      app: slo-sample-app
  template:
    metadata:
      labels:
        app: slo-sample-app
        service: slo-sample-app
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8080"
        prometheus.io/path: "/metrics"
    spec:
      containers:
        - name: app
          image: nginx-prometheus-exporter:latest
          # In practice, use a real app or the official nginx image with
          # stub_status enabled and nginx-prometheus-exporter as a sidecar.
          # For this lab we use a minimal Go application image that exposes
          # http_requests_total and http_request_duration_seconds.
          ports:
            - containerPort: 8080
              name: http
          env:
            - name: ERROR_RATE
              value: "0.001"  # 0.1% error rate -- just under our 99.9% SLO
            - name: LATENCY_MS
              value: "100"
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 200m
              memory: 256Mi
          readinessProbe:
            httpGet:
              path: /healthz
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 10
          livenessProbe:
            httpGet:
              path: /healthz
              port: 8080
            initialDelaySeconds: 10
            periodSeconds: 30
---
apiVersion: v1
kind: Service
metadata:
  name: slo-sample-app
  labels:
    app: slo-sample-app
    service: slo-sample-app
spec:
  selector:
    app: slo-sample-app
  ports:
    - port: 8080
      targetPort: 8080
      name: http
  type: ClusterIP
---
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: slo-sample-app
  labels:
    release: prometheus  # Must match Prometheus operator's serviceMonitorSelector
spec:
  selector:
    matchLabels:
      app: slo-sample-app
  endpoints:
    - port: http
      path: /metrics
      interval: 15s
```

**Deploy:**

```bash
kubectl create namespace slo-lab
kubectl apply -f app-deployment.yaml -n slo-lab
```

**Why this works:** The ServiceMonitor tells the Prometheus Operator to scrape
the `/metrics` endpoint every 15 seconds. The `ERROR_RATE` environment variable
lets you simulate failures for testing. The readiness and liveness probes
ensure Kubernetes does not route traffic to unhealthy pods.

## Part 2 -- SLI Recording Rules

### sli-rules.yaml

```yaml
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: sli-rules
  labels:
    release: prometheus
spec:
  groups:
    - name: sli.rules
      interval: 30s
      rules:
        # ============================================================
        # Availability SLI
        # ============================================================

        # Total request rate (5-minute window)
        - record: sli:http_requests:rate5m
          expr: |
            sum(
              rate(http_requests_total{service="slo-sample-app"}[5m])
            ) by (service)

        # Successful request rate (2xx responses, 5-minute window)
        - record: sli:http_requests_success:rate5m
          expr: |
            sum(
              rate(http_requests_total{service="slo-sample-app", status=~"2.."}[5m])
            ) by (service)

        # Availability ratio (5-minute window)
        - record: sli:availability:ratio5m
          expr: |
            sli:http_requests_success:rate5m
            /
            sli:http_requests:rate5m

        # 30-day average availability
        - record: sli:availability:avg30d
          expr: |
            avg_over_time(sli:availability:ratio5m[30d])

        # ============================================================
        # Latency SLI
        # ============================================================

        # Requests completing under 500ms (5-minute window)
        - record: sli:http_requests_fast:rate5m
          expr: |
            sum(
              rate(http_request_duration_seconds_bucket{service="slo-sample-app", le="0.5"}[5m])
            ) by (service)

        # Total request count from histogram (5-minute window)
        - record: sli:http_requests_total_count:rate5m
          expr: |
            sum(
              rate(http_request_duration_seconds_count{service="slo-sample-app"}[5m])
            ) by (service)

        # Fast request ratio (latency SLI)
        - record: sli:latency:ratio5m
          expr: |
            sli:http_requests_fast:rate5m
            /
            sli:http_requests_total_count:rate5m

        # p99 latency (5-minute window)
        - record: sli:latency_p99:seconds5m
          expr: |
            histogram_quantile(0.99,
              sum(
                rate(http_request_duration_seconds_bucket{service="slo-sample-app"}[5m])
              ) by (le, service)
            )

        # ============================================================
        # Error Budget (99.9% SLO, 30-day window)
        # ============================================================

        # Error budget allowed: 0.1% = 0.001
        # Budget consumed = (1 - avg_availability) / allowed_error_rate
        - record: sli:error_budget:consumed
          expr: |
            (1 - sli:availability:avg30d) / 0.001

        # Budget remaining (1.0 = full budget, 0.0 = exhausted)
        - record: sli:error_budget:remaining
          expr: |
            clamp_max(
              1 - sli:error_budget:consumed,
              1
            )

        # ============================================================
        # Burn Rate
        # ============================================================

        # Burn rate = actual error rate / allowed error rate
        # 1.0 = budget exhausted exactly at period end
        # 2.0 = budget exhausted in half the period
        - record: sli:burn_rate:5m
          expr: |
            (1 - sli:availability:ratio5m) / 0.001
```

**Apply:**

```bash
kubectl apply -f sli-rules.yaml -n slo-lab
```

**Why these rules work:**

- `rate()` over `[5m]` provides a smoothed metric that is not jittery like
  per-second instant values.
- `avg_over_time(...[30d])` computes the rolling 30-day average availability,
  which is the standard window for SLO measurement.
- The error budget formula `(1 - availability) / allowed_error_rate` converts
  the raw availability number into a 0--1 budget consumption scale. If
  availability is exactly 99.9%, consumed = 0 and remaining = 1.0.
- `clamp_max(..., 1)` prevents the remaining budget from going above 1.0
  (which would happen if availability exceeds the SLO target).

## Part 3 -- Grafana Dashboard

### dashboard.json

```json
{
  "dashboard": {
    "id": null,
    "uid": "slo-dashboard",
    "title": "SLO Dashboard -- 99.9% Availability Target",
    "tags": ["slo", "reliability"],
    "timezone": "browser",
    "refresh": "30s",
    "time": {
      "from": "now-7d",
      "to": "now"
    },
    "templating": {
      "list": [
        {
          "name": "slo_target",
          "type": "constant",
          "label": "SLO Target (%)",
          "current": { "value": "99.9" },
          "query": "99.9"
        },
        {
          "name": "window",
          "type": "constant",
          "label": "SLO Window",
          "current": { "value": "30d" },
          "query": "30d"
        }
      ]
    },
    "panels": [
      {
        "id": 1,
        "title": "Current Availability (30-day)",
        "type": "gauge",
        "gridPos": { "h": 8, "w": 8, "x": 0, "y": 0 },
        "targets": [
          {
            "expr": "sli:availability:avg30d{service=\"slo-sample-app\"}",
            "legendFormat": "Availability"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percentunit",
            "min": 0.99,
            "max": 1.0,
            "thresholds": {
              "mode": "absolute",
              "steps": [
                { "value": null, "color": "red" },
                { "value": 0.999, "color": "yellow" },
                { "value": 0.9995, "color": "green" }
              ]
            }
          }
        }
      },
      {
        "id": 2,
        "title": "Error Budget Remaining",
        "type": "gauge",
        "gridPos": { "h": 8, "w": 8, "x": 8, "y": 0 },
        "targets": [
          {
            "expr": "sli:error_budget:remaining{service=\"slo-sample-app\"}",
            "legendFormat": "Budget Remaining"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percentunit",
            "min": 0,
            "max": 1,
            "thresholds": {
              "mode": "absolute",
              "steps": [
                { "value": null, "color": "red" },
                { "value": 0.10, "color": "yellow" },
                { "value": 0.25, "color": "green" }
              ]
            }
          }
        }
      },
      {
        "id": 3,
        "title": "Request Success Rate (5-minute)",
        "type": "stat",
        "gridPos": { "h": 8, "w": 8, "x": 16, "y": 0 },
        "targets": [
          {
            "expr": "sli:availability:ratio5m{service=\"slo-sample-app\"}",
            "legendFormat": "Success Rate"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percentunit",
            "thresholds": {
              "mode": "absolute",
              "steps": [
                { "value": null, "color": "red" },
                { "value": 0.999, "color": "yellow" },
                { "value": 0.9999, "color": "green" }
              ]
            }
          }
        }
      },
      {
        "id": 4,
        "title": "Availability Over Time",
        "type": "timeseries",
        "gridPos": { "h": 8, "w": 24, "x": 0, "y": 8 },
        "targets": [
          {
            "expr": "sli:availability:ratio5m{service=\"slo-sample-app\"}",
            "legendFormat": "Availability (5m)"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percentunit",
            "min": 0.99,
            "max": 1.0,
            "custom": {
              "lineWidth": 1,
              "fillOpacity": 10
            },
            "thresholds": {
              "mode": "absolute",
              "steps": [
                { "value": null, "color": "red" },
                { "value": 0.999, "color": "yellow" },
                { "value": 0.9999, "color": "green" }
              ]
            }
          },
          "overrides": [
            {
              "matcher": { "id": "byName", "options": "SLO Target" },
              "properties": [
                { "id": "custom.lineStyle", "value": { "fill": "dash", "dash": [10, 10] } },
                { "id": "color", "value": { "mode": "fixed", "fixedColor": "red" } },
                { "id": "custom.fillOpacity", "value": 0 }
              ]
            }
          ]
        },
        "targets_extra": [
          {
            "expr": "vector(0.999)",
            "legendFormat": "SLO Target"
          }
        ]
      },
      {
        "id": 5,
        "title": "Latency SLI -- p99",
        "type": "timeseries",
        "gridPos": { "h": 8, "w": 12, "x": 0, "y": 16 },
        "targets": [
          {
            "expr": "sli:latency_p99:seconds5m{service=\"slo-sample-app\"}",
            "legendFormat": "p99 Latency"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "s",
            "custom": {
              "lineWidth": 1,
              "fillOpacity": 10
            },
            "thresholds": {
              "mode": "absolute",
              "steps": [
                { "value": null, "color": "green" },
                { "value": 0.5, "color": "yellow" },
                { "value": 1.0, "color": "red" }
              ]
            }
          }
        }
      },
      {
        "id": 6,
        "title": "Error Budget Burn Rate",
        "type": "timeseries",
        "gridPos": { "h": 8, "w": 12, "x": 12, "y": 16 },
        "targets": [
          {
            "expr": "sli:burn_rate:5m{service=\"slo-sample-app\"}",
            "legendFormat": "Burn Rate"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "short",
            "custom": {
              "lineWidth": 1,
              "fillOpacity": 10
            },
            "thresholds": {
              "mode": "absolute",
              "steps": [
                { "value": null, "color": "green" },
                { "value": 1.0, "color": "yellow" },
                { "value": 3.0, "color": "red" }
              ]
            }
          }
        }
      }
    ]
  }
}
```

**Import into Grafana:**

```bash
kubectl port-forward svc/grafana 3000:80 -n monitoring
# Open http://localhost:3000, go to Dashboards > Import, paste the JSON
```

**Why these panels work:**

- The **Current Availability gauge** gives an instant answer: "Are we meeting
  our SLO right now?" Green/yellow/red thresholds map directly to budget
  health.
- The **Error Budget Remaining gauge** translates availability into a
  consumable resource. Engineers intuitively understand "we have 60% of our
  budget left" better than "we are at 99.93% availability."
- The **Availability Over Time time series** shows trends and incidents. A dip
  below the SLO target line is immediately visible.
- The **Request Success Rate stat** provides a real-time view without the
  smoothing of the 30-day average.
- The **Latency SLI (p99)** tracks the non-availability SLO. It is on a
  separate chart because it uses a different unit (seconds vs. ratio).
- The **Burn Rate time series** answers "how fast are we consuming our budget?"
  A burn rate above 1.0 means the budget will be exhausted before the period
  ends.

## Part 4 -- Alerting Rules

### slo-alerts.yaml

```yaml
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: slo-alerts
  labels:
    release: prometheus
spec:
  groups:
    - name: slo.alerts
      rules:
        # ============================================================
        # Fast Burn Alert
        # ============================================================
        # Burn rate > 14.4x over 1 hour means the budget will be
        # consumed in ~2 days (30d / 14.4 = 2.08 days).
        # Uses a 5-minute short-window check to reduce false positives.
        - alert: SLOFastBurn
          expr: |
            sli:burn_rate:5m{service="slo-sample-app"} > 14.4
            and
            sli:burn_rate:1h{service="slo-sample-app"} > 14.4
          for: 2m
          labels:
            severity: page
            slo: availability
            team: platform
          annotations:
            summary: "Fast burn detected on {{ $labels.service }}"
            description: >
              Error budget burn rate is {{ $value | printf "%.1f" }}x over the
              last hour. At this rate, the 30-day error budget will be exhausted
              in approximately 2 days. Immediate investigation required.
            runbook_url: "https://wiki.internal/runbooks/slo-fast-burn"

        # ============================================================
        # Slow Burn Alert
        # ============================================================
        # Burn rate > 3x over 6 hours means the budget will be
        # consumed in ~10 days (30d / 3 = 10 days).
        - alert: SLOSlowBurn
          expr: |
            sli:burn_rate:5m{service="slo-sample-app"} > 3
            and
            sli:burn_rate:6h{service="slo-sample-app"} > 3
          for: 5m
          labels:
            severity: ticket
            slo: availability
            team: platform
          annotations:
            summary: "Slow burn detected on {{ $labels.service }}"
            description: >
              Error budget burn rate is {{ $value | printf "%.1f" }}x over the
              last 6 hours. At this rate, the 30-day error budget will be
              exhausted in approximately 10 days. Investigate within 24 hours.
            runbook_url: "https://wiki.internal/runbooks/slo-slow-burn"

        # ============================================================
        # Budget Exhaustion Alert
        # ============================================================
        # Less than 10% of error budget remaining.
        - alert: SLOBudgetExhaustion
          expr: |
            sli:error_budget:remaining{service="slo-sample-app"} < 0.10
          for: 5m
          labels:
            severity: page
            slo: availability
            team: platform
          annotations:
            summary: "Error budget nearly exhausted for {{ $labels.service }}"
            description: >
              Only {{ $value | printf "%.1f" | float64Mul 100 | printf "%.0f" }}%
              of the 30-day error budget remains. Consider halting non-critical
              deployments and prioritizing reliability work.
            runbook_url: "https://wiki.internal/runbooks/slo-budget-exhaustion"

        # ============================================================
        # Multi-window burn rate rules (for the alert above)
        # ============================================================
        # These recording rules compute burn rates over longer windows
        # to support multi-window alerting (reduces false positives).
        - record: sli:burn_rate:1h
          expr: |
            (1 - avg_over_time(sli:availability:ratio5m{service="slo-sample-app"}[1h])) / 0.001

        - record: sli:burn_rate:6h
          expr: |
            (1 - avg_over_time(sli:availability:ratio5m{service="slo-sample-app"}[6h])) / 0.001
```

**Apply:**

```bash
kubectl apply -f slo-alerts.yaml -n slo-lab
```

**Why multi-window burn rates matter:** A single-window alert on a 5-minute
burn rate would fire on brief spikes that are not sustained. By requiring
both the short window (5m) AND the long window (1h or 6h) to exceed the
threshold, we filter out transient noise. This is Google's recommended approach
from the SRE Workbook.

## Part 5 -- Validation

```bash
# 1. Verify recording rules are loaded
kubectl get prometheusrules -n slo-lab
# Expected: slo-rules and slo-alerts listed

# 2. Port-forward to Prometheus
kubectl port-forward svc/prometheus-server 9090:80 -n monitoring &

# 3. Verify SLI recording rules are evaluating
curl -s 'http://localhost:9090/api/v1/query?query=sli:availability:ratio5m' | jq .
# Expected: JSON with a result showing a value close to 0.999

# 4. Verify error budget calculation
curl -s 'http://localhost:9090/api/v1/query?query=sli:error_budget:remaining' | jq .
# Expected: JSON with a value between 0 and 1

# 5. Verify burn rate
curl -s 'http://localhost:9090/api/v1/query?query=sli:burn_rate:5m' | jq .
# Expected: JSON with a value close to 1.0 (since ERROR_RATE=0.001)

# 6. Verify alerting rules are loaded
curl -s 'http://localhost:9090/api/v1/rules' | jq '.data.groups[].rules[] | .name'
# Expected: SLOFastBurn, SLOSlowBurn, SLOBudgetExhaustion listed

# 7. Verify Grafana dashboard
kubectl port-forward svc/grafana 3000:80 -n monitoring &
# Open http://localhost:3000, navigate to the imported dashboard
# All 6 panels should show data

# 8. Test alerting by increasing error rate
# Patch the deployment to set ERROR_RATE=0.05 (5% errors)
kubectl set env deployment/slo-sample-app ERROR_RATE=0.05 -n slo-lab
# Wait 2-3 minutes, then check:
curl -s 'http://localhost:9090/api/v1/alerts' | jq '.data.alerts[] | select(.labels.alertname | startswith("SLO"))'
# Expected: SLOFastBurn alert should be firing

# 9. Restore normal error rate
kubectl set env deployment/slo-sample-app ERROR_RATE=0.001 -n slo-lab
```

## Why This Solution Works

1. **Recording rules separate computation from alerting.** The SLI calculations
   (availability ratio, latency p99, error budget) are pre-computed by
   Prometheus at evaluation time. Alerts and dashboards query these pre-computed
   values, which is efficient and ensures consistency.

2. **Multi-window burn rate alerting.** The fast burn alert requires both a
   5-minute and a 1-hour window to exceed the threshold. This prevents false
   positives from brief spikes while still catching sustained degradation
   quickly. The slow burn uses 5-minute and 6-hour windows.

3. **Error budget is a first-class metric.** By computing `sli:error_budget:remaining`
   as a recording rule, every dashboard panel and alert can reference the same
   budget calculation. There is no risk of panels using different formulas.

4. **Dashboard panels answer specific questions.** Each panel has a clear
   purpose: "Are we meeting the SLO?" (gauges), "What is the trend?" (time
   series), "How fast are we burning?" (burn rate). An engineer can glance at
   the dashboard and know the system's reliability status in seconds.

5. **Testable with synthetic error rates.** The `ERROR_RATE` environment
   variable allows you to simulate failures without injecting actual faults.
   You can validate that alerts fire correctly by temporarily increasing the
   error rate.

## Common Mistakes

1. **Using instant queries instead of recording rules for SLIs.** Querying
   `rate(http_requests_total[5m])` directly in every dashboard panel and alert
   means Prometheus re-computes the same expression dozens of times. Recording
   rules compute it once; everything else references the result.

2. **No multi-window burn rate.** A single 5-minute burn rate alert fires on
   every brief spike. This causes alert fatigue, and engineers start ignoring
   SLO alerts. Always pair a short window with a longer window.

3. **Forgetting `clamp_max` on error budget remaining.** When availability
   exceeds the SLO (e.g., 99.95% against a 99.9% target), the remaining budget
   calculation yields a value greater than 1.0. Without `clamp_max`, the gauge
   shows 120% remaining, which is confusing. Clamp to 1.0.

4. **Dashboard without SLO target reference line.** An availability time series
   chart without a horizontal line at the SLO target (0.999) makes it hard to
   tell at a glance whether you are above or below the target. Always add a
   `vector(0.999)` target for the reference line.

5. **Alerting on availability directly instead of burn rate.** Alerting
   `sli:availability:ratio5m < 0.999` fires on every moment the SLO is
   breached, including during expected fluctuations. Burn rate alerts fire on
   sustained degradation, which is what actually threatens the error budget.

6. **Not labeling recording rules by service.** If you add a second service
   later, recording rules without `by (service)` aggregation will mix metrics
   from both services. Always aggregate by service label from the start.
