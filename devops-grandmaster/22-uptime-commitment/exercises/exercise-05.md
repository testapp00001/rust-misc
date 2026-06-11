# Exercise 05: Build an SLO Dashboard with Prometheus and Grafana

**Type:** Integration
**Objective:** Instrument a service with SLI metrics in Prometheus and build a
Grafana dashboard that visualizes SLO compliance and error budget consumption.

## Background

SLOs are only useful if they are visible. This exercise walks you through
building a production-grade SLO dashboard that shows:
- Current SLI values vs. SLO targets
- Error budget remaining (absolute and percentage)
- Burn rate (how fast the budget is being consumed)
- Historical SLO compliance

## Prerequisites

- A Kubernetes cluster with Prometheus and Grafana installed
- `kubectl` configured to access the cluster
- `helm` for installing chart dependencies
- A sample application (we will deploy one if needed)

## Instructions

### Part 1 -- Deploy the Sample Application

Deploy a sample HTTP service that exposes Prometheus metrics. The service
simulates a web API with configurable error rates and latency.

Create a namespace and deploy the application:

```bash
kubectl create namespace slo-lab
```

Create a file `app-deployment.yaml` with a Deployment and Service for a sample
application that:
- Exposes a `/metrics` endpoint for Prometheus scraping
- Tracks `http_requests_total` with labels for `method`, `path`, `status`
- Tracks `http_request_duration_seconds` as a histogram
- Has a configurable failure rate (via environment variable)

Deploy it:

```bash
kubectl apply -f app-deployment.yaml -n slo-lab
```

### Part 2 -- Define SLI Recording Rules

Create Prometheus recording rules that compute your SLIs from raw metrics.

Create a file `sli-rules.yaml` with a PrometheusRule resource that defines:

1. **Availability SLI** -- Ratio of successful (2xx) requests to total requests
   over a 5-minute window:

```yaml
# Calculate total requests per 5 minutes
- record: sli:http_requests:rate5m
  expr: sum(rate(http_requests_total[5m])) by (service)

# Calculate successful requests per 5 minutes
- record: sli:http_requests_success:rate5m
  expr: sum(rate(http_requests_total{status=~"2.."}[5m])) by (service)

# Calculate availability ratio
- record: sli:availability:ratio5m
  expr: sli:http_requests_success:rate5m / sli:http_requests:rate5m
```

2. **Latency SLI** -- Ratio of requests completing under 500ms:

```yaml
# Requests under 500ms
- record: sli:http_requests_fast:rate5m
  expr: sum(rate(http_request_duration_seconds_bucket{le="0.5"}[5m])) by (service)

# Total requests (from histogram _count)
- record: sli:http_requests_total_count:rate5m
  expr: sum(rate(http_request_duration_seconds_count[5m])) by (service)

# Fast request ratio
- record: sli:latency:ratio5m
  expr: sli:http_requests_fast:rate5m / sli:http_requests_total_count:rate5m
```

3. **Error budget remaining** -- For a 99.9% availability SLO measured over a
   30-day window, create a rule that computes the error budget consumed and
   remaining. You will need to use `avg_over_time` on the availability ratio
   and calculate:

```yaml
# Error budget: 0.1% = 0.001 allowed error rate
# Budget consumed = (1 - avg_availability) / 0.001
# Budget remaining = 1 - budget_consumed

- record: sli:availability:avg30d
  expr: avg_over_time(sli:availability:ratio5m[30d])

- record: sli:error_budget:consumed
  expr: (1 - sli:availability:avg30d) / 0.001

- record: sli:error_budget:remaining
  expr: 1 - sli:error_budget:consumed
```

Apply the rules:

```bash
kubectl apply -f sli-rules.yaml -n slo-lab
```

### Part 3 -- Build the Grafana Dashboard

Create a Grafana dashboard (JSON model) with the following panels:

1. **Current Availability** -- A gauge panel showing the 30-day availability
   ratio. Green above SLO, yellow within 10% of the budget, red when over
   budget.

2. **Error Budget Remaining** -- A gauge showing the percentage of error budget
   remaining. Use thresholds: green > 25%, yellow 10-25%, red < 10%.

3. **Availability Over Time** -- A time series graph showing the 5-minute
   availability ratio over the past 7 days, with a horizontal line at the SLO
   target.

4. **Request Success Rate** -- A stat panel showing the current 5-minute success
   rate.

5. **Latency SLI (p99)** -- A time series graph showing the 99th percentile
   request latency over time.

6. **Burn Rate** -- A time series graph showing the error budget burn rate. A
   burn rate of 1.0 means the budget will be exactly exhausted by the end of the
   period. A burn rate of 2.0 means it will be exhausted in half the period.

### Part 4 -- Add Alerting Rules

Create Prometheus alerting rules that fire when:

1. **Fast burn** (burn rate > 14.4x over 1 hour) -- The budget will be consumed
   in 2 days at this rate. Severity: page.
2. **Slow burn** (burn rate > 3x over 6 hours) -- The budget will be consumed in
   10 days. Severity: ticket.
3. **Budget exhaustion** (budget remaining < 10%) -- Urgent. Severity: page.

Create a file `slo-alerts.yaml` with a PrometheusRule resource.

### Part 5 -- Validation

Verify everything works:

```bash
# Check recording rules are loaded
kubectl get prometheusrules -n slo-lab

# Port-forward to Prometheus
kubectl port-forward svc/prometheus-server 9090:80 -n monitoring

# Query the SLI
curl 'http://localhost:9090/api/v1/query?query=sli:availability:ratio5m'

# Port-forward to Grafana
kubectl port-forward svc/grafana 3000:80 -n monitoring
# Import the dashboard JSON via the Grafana UI
```

## Deliverables

Submit the following files:

1. `app-deployment.yaml` -- Application deployment manifest
2. `sli-rules.yaml` -- Prometheus recording rules for SLIs
3. `slo-alerts.yaml` -- Prometheus alerting rules
4. `dashboard.json` -- Grafana dashboard JSON model
5. A screenshot of your Grafana dashboard showing the panels

## Success Criteria

- [ ] The application is deployed and exposing metrics.
- [ ] Recording rules compute availability and latency SLIs correctly.
- [ ] Error budget calculations use a 30-day window with a 99.9% SLO.
- [ ] The Grafana dashboard has at least 4 of the 6 specified panels.
- [ ] Alerting rules include at least fast burn and slow burn alerts.
- [ ] The dashboard correctly shows green/yellow/red based on SLO thresholds.

## Hints

<details>
<summary>Hint 1 -- Bootstrap with synthetic data</summary>

If you cannot wait 30 days for the `avg_over_time` to populate, use a shorter
window for testing (e.g., 1 hour) and then switch to 30 days for production.
You can also use the `--web.enable-lifecycle` flag on Prometheus and manually
set values for testing.

</details>

<details>
<summary>Hint 2 -- Grafana dashboard variables</summary>

Use Grafana template variables for the SLO target and error budget window. This
makes the dashboard reusable across services. Define:

- `$slo_target` (default 99.9)
- `$window` (default 30d)

</details>

<details>
<summary>Hint 3 -- Multi-window burn rate</summary>

For production alerting, use multiple burn rate windows. Google's SRE book
recommends:
- 14.4x burn rate over 1 hour AND 6x over 5 minutes (page)
- 6x burn rate over 6 hours AND 1x over 30 minutes (ticket)
- 3x burn rate over 3 days (ticket)

This prevents false positives from brief spikes.

</details>
