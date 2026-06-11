# Solution 05: Dashboard Hierarchy for Microservices

## Part A -- Architecture overview

### Dashboard Hierarchy Design

```
L1: Platform Overview (1 dashboard)
    |
    +---> L2: Service Dashboard (1 parameterized dashboard, reused for all services)
              |
              +---> L3: Deep-Dive Dashboards (per-service specialized dashboards)
                        |
                        +-- payment-service-detail.json (Stripe, transactions, DB)
                        +-- product-catalog-detail.json (search, cache, S3)
                        +-- order-service-detail.json (order flow, inventory sync)
```

### Answers

**1. How many L1 dashboards?**
One. The L1 "Platform Overview" dashboard shows all 6 services on a single screen. There is no reason to split L1 -- its purpose is "is everything OK?" at a glance.

**2. How many L2 dashboards?**
One parameterized dashboard (`l2-service-template.json`). The `$service` variable makes it work for any service. This is critical for maintenance: one dashboard to update instead of six.

**3. What goes on L3 dashboards?**
L3 dashboards are service-specific deep dives. They contain:
- Panels unique to the service's dependencies (e.g., Stripe API for payment-service)
- Detailed query performance (for database-backed services)
- Business-specific metrics (e.g., transaction throughput for payment-service)
- Specialized saturation indicators (e.g., cache hit rate for product-catalog)

**4. How does an operator navigate?**
```
L1 table --> click service name --> L2 with ?var-service=<name>
L2 dependency panel --> click --> L3 deep-dive for that dependency
L2/L3 --> breadcrumb link --> back to L1
L2/L3 --> dashboard links --> logs, traces, runbooks
```

### Why this structure works

- Single L1 eliminates ambiguity about where to start
- Parameterized L2 eliminates maintenance burden (change once, applies everywhere)
- Per-service L3 allows deep specialization without cluttering L2
- Clear navigation links mean operators never get lost

**Common mistakes**:
- Creating separate L2 dashboards per service (maintenance nightmare)
- Making L1 too detailed (it should answer "is everything OK?" in 10 seconds)
- No navigation links between levels (operators have to remember URLs)

---

## Part B -- L1: Platform overview dashboard

### Panel 1: Overall Platform Health (stat)

```promql
min(
  (
    1 -
    (
      sum by (service) (rate(http_requests_total{environment="$environment", status=~"5.."}[30d]))
      / sum by (service) (rate(http_requests_total{environment="$environment"}[30d]))
    )
    / 0.001
  )
) * 100
```

Returns the minimum error budget remaining across all services. Thresholds:
- Green: > 25%
- Yellow: 10-25%
- Red: < 10%

### Panel 2: Per-Service Health Table

**Query A -- Request Rate:**
```promql
sum by (service) (rate(http_requests_total{environment="$environment"}[5m]))
```

**Query B -- Error Rate:**
```promql
sum by (service) (rate(http_requests_total{environment="$environment", status=~"5.."}[5m]))
/ sum by (service) (rate(http_requests_total{environment="$environment"}[5m]))
* 100
```

**Query C -- p99 Latency:**
```promql
histogram_quantile(0.99,
  sum by (service, le) (rate(http_request_duration_seconds_bucket{environment="$environment"}[5m]))
) * 1000
```

**Query D -- SLO Status:**
```promql
(
  1 -
  (
    sum by (service) (rate(http_requests_total{environment="$environment", status=~"5.."}[30d]))
    / sum by (service) (rate(http_requests_total{environment="$environment"}[30d]))
  )
  / (1 - (99.9 / 100))
) * 100
```

Note: The SLO target is derived per-service from the table in the exercise. For a generalized dashboard, you could use a recording rule per service that encodes the SLO target.

Transformations:
- `seriesToColumns` on `service` field
- `organize` to rename columns and hide Time fields

Data links on the Service column:
```json
{
  "title": "View ${__value.text} details",
  "url": "/d/l2-service-template?var-environment=${environment}&var-service=${__value.text}"
}
```

### Panel 3: Global Error Rate Trend (time series)

```promql
sum by (service) (
  rate(http_requests_total{environment="$environment", status=~"5.."}[5m])
  / rate(http_requests_total{environment="$environment"}[5m])
) * 100
```

Legend: `{{service}}`

Add threshold lines for each service's SLO target (or a single line at the most common target of 0.1%).

### Panel 4: SLO Budget Remaining (bar chart)

```promql
(
  1 -
  (
    sum by (service) (rate(http_requests_total{environment="$environment", status=~"5.."}[30d]))
    / sum by (service) (rate(http_requests_total{environment="$environment"}[30d]))
  )
  / 0.001
) * 100
```

Display as a horizontal bar chart, one bar per service, sorted by budget remaining (lowest first). Thresholds:
- Green: > 25%
- Yellow: 10-25%
- Red: < 10%

**Common mistakes**:
- Using the same SLO target for all services (they have different SLOs)
- Not showing the SLO status column in the table (just showing error rate is not enough)
- Using a 30d window that is not aligned with the SLO period

---

## Part C -- L2: Service dashboard (parameterized)

### Row 1: Key Metrics (4 stat panels)

**Request Rate:**
```promql
sum(rate(http_requests_total{service="$service", environment="$environment"}[5m]))
```
Unit: reqps

**Error Rate:**
```promql
sum(rate(http_requests_total{service="$service", environment="$environment", status=~"5.."}[5m]))
/ sum(rate(http_requests_total{service="$service", environment="$environment"}[5m]))
* 100
```
Unit: percent. Thresholds: green < 0.05, yellow < 0.1, red >= 0.1

**p99 Latency:**
```promql
histogram_quantile(0.99,
  sum by (le) (rate(http_request_duration_seconds_bucket{service="$service", environment="$environment"}[5m]))
)
```
Unit: seconds. Thresholds: green < 0.2, yellow < 0.5, red >= 0.5 (adjust per service)

**SLO Budget Remaining:**
```promql
(
  1 -
  (
    sum(rate(http_requests_total{service="$service", environment="$environment", status=~"5.."}[30d]))
    / sum(rate(http_requests_total{service="$service", environment="$environment"}[30d]))
  )
  / 0.001
) * 100
```
Unit: percent. Thresholds: green > 25, yellow > 10, red <= 10

### Row 2: RED Method (3 time series)

**Rate over time:**
```promql
sum by (method) (
  rate(http_requests_total{service="$service", environment="$environment"}[5m])
)
```
Legend: `{{method}}`

**Errors over time (with SLO threshold):**
```promql
sum(rate(http_requests_total{service="$service", environment="$environment", status=~"5.."}[5m]))
/ sum(rate(http_requests_total{service="$service", environment="$environment"}[5m]))
* 100
```
Legend: `Error Rate`
Add threshold line at 0.1%

**Latency percentiles (p50, p95, p99):**
```promql
# Three targets
histogram_quantile(0.50, sum by (le) (rate(http_request_duration_seconds_bucket{service="$service", environment="$environment"}[5m])))
histogram_quantile(0.95, sum by (le) (rate(http_request_duration_seconds_bucket{service="$service", environment="$environment"}[5m])))
histogram_quantile(0.99, sum by (le) (rate(http_request_duration_seconds_bucket{service="$service", environment="$environment"}[5m])))
```
Legends: p50, p95, p99

### Row 3: Dependencies (variable panels)

**Database connection pool:**
```promql
db_connection_pool_active{service="$service", environment="$environment"}
```

**Database query latency:**
```promql
sum by (query) (
  rate(db_query_duration_seconds_sum{service="$service", environment="$environment"}[5m])
)
/ sum by (query) (
  rate(db_query_duration_seconds_count{service="$service", environment="$environment"}[5m])
)
```

**Redis hit rate:**
```promql
rate(redis_keyspace_hits_total{service="$service", environment="$environment"}[5m])
/ (
  rate(redis_keyspace_hits_total{service="$service", environment="$environment"}[5m])
  + rate(redis_keyspace_misses_total{service="$service", environment="$environment"}[5m])
) * 100
```

**External API latency (e.g., Stripe):**
```promql
histogram_quantile(0.99,
  sum by (le) (rate(stripe_api_request_duration_seconds_bucket{service="$service", environment="$environment"}[5m]))
)
```

For services without a particular dependency, the query returns no data and Grafana shows "No data" -- which is correct.

### Row 4: Infrastructure (4 gauge panels)

**CPU:**
```promql
max(
  rate(container_cpu_usage_seconds_total{pod=~"$service-.*", namespace="$environment"}[5m])
) * 100
```

**Memory:**
```promql
max(
  container_memory_working_set_bytes{pod=~"$service-.*", namespace="$environment"}
  / container_spec_memory_limit_bytes{pod=~"$service-.*", namespace="$environment"}
) * 100
```

**Disk:**
```promql
max(
  container_fs_usage_bytes{pod=~"$service-.*", namespace="$environment"}
  / container_fs_limit_bytes{pod=~"$service-.*", namespace="$environment"}
) * 100
```

**Network:**
```promql
max(
  rate(container_network_receive_bytes_total{pod=~"$service-.*", namespace="$environment"}[5m])
  + rate(container_network_transmit_bytes_total{pod=~"$service-.*", namespace="$environment"}[5m])
)
```
Unit: Bps

---

## Part D -- L3: Deep-dive dashboard for payment-service

### Panel 1: Stripe API Latency (time series)

```promql
histogram_quantile(0.95,
  sum by (le) (rate(stripe_api_request_duration_seconds_bucket{service="payment-service"}[5m]))
)
```
Legend: `Stripe p95`

```promql
histogram_quantile(0.99,
  sum by (le) (rate(stripe_api_request_duration_seconds_bucket{service="payment-service"}[5m]))
)
```
Legend: `Stripe p99`

Threshold: Stripe's SLA guarantees p99 < 2s for most operations.

### Panel 2: Stripe API Errors (time series)

```promql
sum by (status) (
  rate(stripe_api_request_duration_seconds_count{service="payment-service", status=~"4..|5.."}[5m])
)
```
Legend: `{{status}}`

Broken down by HTTP status code. Common Stripe error codes:
- 402: Payment declined
- 429: Rate limited
- 500/502/503: Stripe internal errors

### Panel 3: Transaction Throughput (time series)

```promql
sum by (status) (
  rate(payment_transactions_total{service="payment-service"}[5m])
)
```
Legend: `{{status}}`

Two series: `status="success"` and `status="failed"`. The gap between them is the failure rate.

### Panel 4: Database Query Performance (table)

```promql
topk(10,
  sum by (query) (
    rate(db_query_duration_seconds_sum{service="payment-service"}[5m])
  )
  / sum by (query) (
    rate(db_query_duration_seconds_count{service="payment-service"}[5m])
  )
)
```

Columns: Query, Average Duration, p99 Duration, Calls/sec

**Query B -- p99 per query:**
```promql
topk(10,
  histogram_quantile(0.99,
    sum by (query, le) (
      rate(db_query_duration_seconds_bucket{service="payment-service"}[5m])
    )
  )
)
```

**Query C -- calls per second:**
```promql
topk(10,
  sum by (query) (
    rate(db_query_duration_seconds_count{service="payment-service"}[5m])
  )
)
```

Use `seriesToColumns` on `query` field, then `organize` to rename and format.

### Panel 5: Connection Pool Saturation (gauge)

```promql
db_connection_pool_active{service="payment-service"}
/ db_connection_pool_max{service="payment-service"}
* 100
```

Gauge range: 0-100%. Thresholds:
- Green: 0-60%
- Yellow: 60-80%
- Red: 80-100%

### Panel 6: Idempotency Key Cache (stat)

```promql
rate(redis_cache_hits_total{service="payment-service", cache="idempotency"}[5m])
/ (
  rate(redis_cache_hits_total{service="payment-service", cache="idempotency"}[5m])
  + rate(redis_cache_misses_total{service="payment-service", cache="idempotency"}[5m])
) * 100
```

Display as a stat panel showing cache hit rate percentage. Thresholds:
- Green: > 95%
- Yellow: 80-95%
- Red: < 80%

**Why these panels matter for payment-service specifically**:
- Stripe API latency directly impacts payment processing time (user-facing)
- Transaction throughput reveals the actual business impact of failures
- Slow database queries are the most common cause of payment timeouts
- Connection pool exhaustion causes cascading failures during traffic spikes
- Low idempotency cache hit rate means Redis is under-provisioned or keys are expiring too fast

---

## Part E -- Dashboard provisioning as code

### Provisioning YAML

**File: `grafana/provisioning/dashboards/dashboards.yml`**

```yaml
apiVersion: 1

providers:
  - name: 'Overview'
    orgId: 1
    folder: 'Overview'
    type: file
    disableDeletion: false
    editable: true
    updateIntervalSeconds: 30
    allowUiUpdates: true
    options:
      path: /etc/grafana/provisioning/dashboards/overview
      foldersFromFilesStructure: false

  - name: 'Services'
    orgId: 1
    folder: 'Services'
    type: file
    disableDeletion: false
    editable: true
    updateIntervalSeconds: 30
    allowUiUpdates: true
    options:
      path: /etc/grafana/provisioning/dashboards/services
      foldersFromFilesStructure: false

  - name: 'Deep-Dive'
    orgId: 1
    folder: 'Deep-Dive'
    type: file
    disableDeletion: false
    editable: true
    updateIntervalSeconds: 30
    allowUiUpdates: true
    options:
      path: /etc/grafana/provisioning/dashboards/deep-dive
      foldersFromFilesStructure: false
```

### Directory structure

```
grafana/
  provisioning/
    dashboards/
      dashboards.yml                    # Provider configuration
      overview/
        platform-overview.json          # L1 dashboard
      services/
        l2-service-template.json        # L2 parameterized dashboard
      deep-dive/
        payment-service-detail.json     # L3 payment-service
        product-catalog-detail.json     # L3 product-catalog
        order-service-detail.json       # L3 order-service
```

### How provisioning works

**Lifecycle:**
1. On Grafana startup, the provisioning system reads `dashboards.yml`
2. Each provider scans its configured `path` for `.json` files
3. Dashboards are created or updated in the specified folder
4. `updateIntervalSeconds: 30` means Grafana re-scans every 30 seconds

**Adding a new dashboard:**
1. Create the JSON file in the appropriate folder
2. Commit to Git
3. Deploy (or wait for the 30s rescan if using a volume mount)
4. Grafana creates the dashboard automatically

**Updating an existing dashboard:**
1. Edit the JSON file in Git
2. Deploy (or wait for rescan)
3. Grafana updates the dashboard

**Deleting a dashboard:**
1. Remove the JSON file from Git
2. With `disableDeletion: false`, Grafana removes the dashboard on next rescan
3. With `disableDeletion: true`, you must manually delete from the UI

**Important**: `allowUiUpdates: true` allows changes made in the Grafana UI to persist. Set to `false` in strict environments where all changes must go through Git.

**Common mistakes**:
- Setting `disableDeletion: true` and forgetting to manually clean up removed dashboards
- Not using `updateIntervalSeconds` (defaults to 0, meaning dashboards only load on startup)
- Hardcoding folder UIDs instead of using folder names
- Not committing the JSON to Git (manual edits get overwritten on restart)

### Docker Compose example

```yaml
services:
  grafana:
    image: grafana/grafana:latest
    volumes:
      - ./grafana/provisioning:/etc/grafana/provisioning
      - grafana-data:/var/lib/grafana
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin

volumes:
  grafana-data:
```

---

## Part F -- Dashboard review checklist

| # | Check | Criteria | Pass/Fail |
|---|-------|----------|-----------|
| **Technical Correctness** | | | |
| 1 | PromQL syntax | All queries parse without errors in Prometheus | |
| 2 | Metric names | All referenced metrics exist in the target environment | |
| 3 | Label names | Labels used in queries match the actual metric labels | |
| 4 | Variable references | All `$variable` references are defined in the templating section | |
| 5 | Data source | The datasource UID/type matches the intended data source | |
| 6 | Time range | Default time range is appropriate for the dashboard level | |
| 7 | Refresh rate | Auto-refresh interval is reasonable (not too fast, not too slow) | |
| 8 | JSON validity | The JSON parses without errors | |
| **Design Quality** | | | |
| 9 | Panel count | L1 <= 8 panels, L2 <= 16 panels, L3 <= 20 panels | |
| 10 | Row organization | Panels are grouped into logical rows with descriptive titles | |
| 11 | Color scheme | Thresholds use consistent colors (green/yellow/red) across panels | |
| 12 | Unit formatting | All axes and values use appropriate units (ms, req/s, %, bytes) | |
| 13 | Legend clarity | Legend labels are human-readable (not raw label keys) | |
| 14 | Responsive layout | Dashboard works on both large monitors and laptops | |
| **Operational Value** | | | |
| 15 | Question mapping | Each panel answers a specific operational question | |
| 16 | Actionability | An operator can take action based on what the panel shows | |
| 17 | Navigation | Links connect to related dashboards (L1 -> L2 -> L3) | |
| 18 | Context | Enough historical context to distinguish trends from noise | |
| 19 | SLO alignment | Error rate panels include SLO threshold lines | |
| 20 | Blast radius | L1 and incident dashboards quantify user impact | |
| **Maintenance Burden** | | | |
| 21 | Variable reuse | Dashboard is parameterized (not hardcoded for one service) | |
| 22 | Query efficiency | No unnecessarily expensive queries (e.g., unaggregated `count()`) | |
| 23 | Documentation | Dashboard description explains its purpose and audience | |
| 24 | Provisioning | Dashboard JSON is committed to Git and provisioned as code | |
| 25 | Dependency count | Minimal external dependencies (plugins, custom data sources) | |

### Review process

1. **Automated checks** (CI pipeline):
   - JSON validity
   - PromQL syntax check (use `promtool check rules` or similar)
   - Lint for common issues (missing units, no thresholds)

2. **Peer review** (PR review):
   - Design quality (layout, readability)
   - Operational value (does it answer real questions?)
   - SLO alignment

3. **UAT** (before merge):
   - Load the dashboard in a staging Grafana instance
   - Verify all panels render with real data
   - Test variable interactions
   - Test navigation links

**Common mistakes**:
- Skipping the review process ("it works on my machine")
- Not testing with production-like data volumes (queries that work on dev may timeout on prod)
- Reviewing only the JSON and not the rendered dashboard
