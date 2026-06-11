# Solution 01: Baseline Metrics Collection

## Part A: Key Baseline Metrics

### 1. Requests Per Second (RPS)
- **What**: Total HTTP requests served per second across all pods.
- **Why**: Establishes current load and growth trend. The primary metric for capacity forecasting.
- **Good**: Below 50% of known capacity. **Bad**: Above 80% of known capacity.

### 2. P95 Request Latency
- **What**: 95th percentile of request duration (95% of requests are faster than this).
- **Why**: Indicates user experience under normal load. Latency degrades non-linearly as utilization increases.
- **Good**: Below 200ms. **Bad**: Above 1,000ms.

### 3. Error Rate
- **What**: Fraction of requests returning 5xx status codes.
- **Why**: Indicates system health. Even a small error rate under normal load means the system is fragile.
- **Good**: Below 0.1%. **Bad**: Above 1%.

### 4. CPU Utilization
- **What**: Average CPU usage as a percentage of the limit.
- **Why**: CPU is often the first resource to exhaust. High CPU under normal load means no headroom for spikes.
- **Good**: Below 60%. **Bad**: Above 80%.

### 5. Memory Utilization
- **What**: Average memory usage as a percentage of the limit.
- **Why**: Memory exhaustion causes OOM kills and pod restarts. Memory leaks show up as gradual increase.
- **Good**: Below 70%. **Bad**: Above 85%.

### 6. Database Connection Count
- **What**: Number of active database connections as a percentage of the maximum.
- **Why**: Database connections are a finite resource. Exhaustion causes cascading failures.
- **Good**: Below 60%. **Bad**: Above 80%.

### 7. Database Query Latency
- **What**: Average time for database queries to complete.
- **Why**: Slow queries hold connections longer, reducing effective connection pool capacity.
- **Good**: Below 50ms. **Bad**: Above 200ms.

### 8. Queue Depth
- **What**: Number of messages waiting in processing queues.
- **Why**: Growing queue depth indicates the processing tier cannot keep up with ingestion.
- **Good**: Below 1,000. **Bad**: Above 10,000.

---

## Part B: PromQL Queries

### 1. Total HTTP requests per second
```promql
sum(rate(http_requests_total[5m]))
```
**Why**: `rate()` calculates the per-second rate of increase over 5 minutes. `sum()` aggregates across all pods.

### 2. 95th percentile request latency
```promql
histogram_quantile(0.95, sum(rate(http_request_duration_seconds_bucket[5m])) by (le))
```
**Why**: `histogram_quantile()` computes percentiles from histogram buckets. The `by (le)` grouping is required for correct calculation.

### 3. Average CPU utilization as a percentage
```promql
avg(rate(container_cpu_usage_seconds_total{container!=""}[5m])) * 100
```
**Why**: `container_cpu_usage_seconds_total` is cumulative CPU seconds. `rate()` converts to per-second usage (fraction of one CPU). Multiply by 100 for percentage. The `container!=""` filter excludes system containers.

### 4. Average memory utilization as a percentage of limit
```promql
avg(container_memory_working_set_bytes{container!=""} / container_spec_memory_limit_bytes{container!=""}) * 100
```
**Why**: `container_memory_working_set_bytes` is the actual memory used. Dividing by the limit gives utilization as a fraction. Multiply by 100 for percentage.

### 5. Database connection count
```promql
pg_stat_activity_count
```
Or for connection utilization:
```promql
pg_stat_activity_count / pg_settings_max_connections * 100
```
**Why**: `pg_stat_activity_count` reports active connections. Dividing by `pg_settings_max_connections` gives utilization percentage.

### 6. Error rate
```promql
sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m]))
```
**Why**: The numerator counts 5xx responses. The denominator counts all responses. The division gives the error rate as a fraction (0.0 to 1.0).

---

## Part C: Collection Duration and Frequency

### Duration: At Least One Full Week
Traffic patterns vary by:
- **Time of day**: Peak hours (9 AM - 5 PM) vs off-peak (midnight - 6 AM). A single day captures this.
- **Day of week**: Weekdays vs weekends. Many B2B systems see 2-3x more traffic on weekdays. A single week captures this.
- **Monthly patterns**: Month-end processing, payroll cycles. Ideally collect for 4 weeks, but 1 week is the minimum.

### Frequency: Every 5 Minutes
- **5-minute intervals** give 288 samples per day, 2,016 per week.
- This is enough to capture trends without excessive storage.
- Prometheus default scrape interval is 15-60 seconds; 5-minute query intervals smooth out short-term noise.

### Patterns to Look For
- **Daily cycles**: Traffic peaks during business hours, drops at night.
- **Weekly cycles**: Higher on weekdays, lower on weekends.
- **Growth trend**: Gradual increase in peak traffic over weeks/months.
- **Spikes**: Sudden bursts (marketing events, deployments) that exceed normal peaks.

---

## Common Mistakes
1. **Collecting for only one day**: Daily patterns do not capture weekly variation. Monday traffic can be very different from Saturday.
2. **Using averages instead of peaks**: Average CPU of 40% hides the fact that peak CPU hits 85% every afternoon. Plan for peaks, not averages.
3. **Ignoring P99 latency**: P95 hides outliers. P99 reveals the worst 1% of requests, which often indicate connection pool or garbage collection issues.
4. **Not filtering out anomalies**: A one-time deployment spike should not be included in baseline calculations. Filter outliers before analysis.

## Relevant README Sections
- [Establish Baselines](../README.md#step-1-establish-baselines)
- [The Right Way](../README.md#the-right-way)
