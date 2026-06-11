# Exercise 04: Instrument an Application with Custom Metrics

**Type:** Challenge
**Estimated time:** 45 minutes

## Objective

Build a simple web application that exposes custom Prometheus metrics.
Instrument it with counters, gauges, and histograms following the RED method
(Rate, Errors, Duration). Integrate the application with the Prometheus stack
from Exercise 02, create alerting rules, and verify everything works.

## Prerequisites

- The Prometheus + Grafana stack from Exercise 02 is running
- Python 3.9+ installed (or a language of your choice)
- Docker installed
- `curl` for generating traffic

## Instructions

### Step 1 -- Build the Instrumented Application

Create a directory `app/` and build a Flask application that exposes the
following metrics at `/metrics`:

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `http_requests_total` | Counter | method, endpoint, status | Total HTTP requests |
| `http_request_duration_seconds` | Histogram | method, endpoint | Request latency |
| `http_in_flight_requests` | Gauge | (none) | Currently processing requests |

The application should have at least three endpoints:

- `GET /api/users` -- returns a JSON list, simulated latency 10-100ms, 5%
  chance of 500 error.
- `GET /api/orders` -- returns a JSON list, simulated latency 20-200ms, 2%
  chance of 503 error.
- `GET /health` -- returns `{"status": "healthy"}`.

Use `before_request` and `after_request` hooks to instrument all endpoints
automatically.

### Step 2 -- Containerize the Application

Write a `Dockerfile` for the application. Use a slim Python base image.
The container should expose port 8080.

### Step 3 -- Add the Application to Docker Compose

Update your `docker-compose.yml` from Exercise 02 to include the `app`
service. Update the Prometheus configuration to scrape the application at
`app:8080/metrics`.

### Step 4 -- Write Alerting Rules

Create `prometheus/rules/app-alerts.yml` with the following alerts:

1. **HighErrorRate** -- fires when the error rate (5xx responses as a
   fraction of total requests) exceeds 10% for 2 minutes. Severity:
   critical.

2. **HighLatency** -- fires when the p99 request duration exceeds 1 second
   for 5 minutes. Severity: warning.

3. **InstanceDown** -- fires when the `up` metric for the `sample-app` job
   equals 0 for 1 minute. Severity: critical.

### Step 5 -- Restart the Stack and Generate Traffic

```bash
docker compose down
docker compose up -d
```

Generate traffic to produce meaningful metrics:

```bash
# Generate 1000 requests
for i in $(seq 1 1000); do
  curl -s http://localhost:8080/api/users > /dev/null &
  curl -s http://localhost:8080/api/orders > /dev/null &
done
wait
```

### Step 6 -- Verify in Prometheus

Open `http://localhost:9090` and run these queries:

1. `rate(http_requests_total[5m])` -- should show request rates.
2. `sum by (status) (rate(http_requests_total[5m]))` -- requests grouped
   by status.
3. `histogram_quantile(0.99, sum by (le) (rate(http_request_duration_seconds_bucket[5m])))` --
   p99 latency.
4. Go to **Alerts** and verify your three alerting rules appear.

### Step 7 -- Build a Grafana Dashboard

Create a Grafana dashboard with the RED method panels:

1. **Rate** -- total requests per second (Stat panel).
2. **Errors** -- error rate as a percentage (Gauge panel, thresholds: green
   0-5%, yellow 5-10%, red 10%+).
3. **Duration** -- p50, p95, p99 latency on a single time series panel with
   three queries.

## Success Criteria

- [ ] The application exposes a `/metrics` endpoint with valid Prometheus
      text format.
- [ ] `http_requests_total` increments correctly for each request.
- [ ] `http_request_duration_seconds_bucket` shows realistic latency
      distributions.
- [ ] `http_in_flight_requests` returns to 0 when no traffic is flowing.
- [ ] Prometheus scrapes the application successfully (target shows UP).
- [ ] All three alerting rules appear in the Prometheus Alerts page.
- [ ] The Grafana dashboard shows Rate, Errors, and Duration panels with
      data.
- [ ] You can explain why `rate()` is used on Counters instead of `sum()`.
- [ ] You can explain why histograms use `histogram_quantile()` and not
      raw bucket values.

## Hints

<details>
<summary>Hint 1 -- prometheus_client for Python</summary>
Install with `pip install prometheus_client flask`. Use `Counter`, `Histogram`,
and `Gauge` classes. The `/metrics` endpoint calls `generate_latest()` to
produce the text format output.
</details>

<details>
<summary>Hint 2 -- Histogram bucket configuration</summary>
Default buckets may not match your latency range. For web APIs, good
buckets are: `[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]`.
These cover sub-millisecond to multi-second latencies.
</details>

<details>
<summary>Hint 3 -- before_request timing</summary>
Store the start time on the request object in `before_request`:
`request._start_time = time.time()`. Compute the duration in
`after_request`: `time.time() - request._start_time`.
</details>

<details>
<summary>Hint 4 -- Alerting rule expressions</summary>
For the error rate alert, divide the 5xx rate by the total rate:
```
sum(rate(http_requests_total{status=~"5.."}[5m]))
/ sum(rate(http_requests_total[5m]))
> 0.1
```
</details>

<details>
<summary>Hint 5 -- Prometheus rules directory</summary>
Mount the rules directory in the Prometheus container:
```yaml
volumes:
  - ./prometheus/rules:/etc/prometheus/rules
```
Ensure the `rule_files` directive in prometheus.yml points to
`/etc/prometheus/rules/*.yml`.
</details>
