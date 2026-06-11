# Solution 01: Golden Signals, RED, and USE Methods

## Part A -- Classify each component

| Component | Methodology | Why? |
|-----------|-------------|------|
| HTTP API gateway | Golden Signals | User-facing entry point; needs latency, traffic, errors, and saturation to understand both user experience and capacity. |
| PostgreSQL database server | USE | Infrastructure resource with finite capacity (connections, I/O, memory). Utilization and saturation matter more than request rates. |
| Redis cache cluster | USE | Infrastructure resource with finite memory and connection capacity. Hit rate is a supplementary custom metric, not a standard framework signal. |
| Kubernetes worker node | USE | Pure infrastructure resource with CPU, memory, disk, and network capacity constraints. |
| Message queue (RabbitMQ) | USE | Infrastructure resource where queue depth (saturation), memory usage (utilization), and dropped messages (errors) are the critical signals. |
| User-facing web application | Golden Signals | User-facing service where latency, traffic, errors, and saturation directly reflect user experience. |
| Load balancer | Golden Signals | User-facing routing layer; latency, traffic distribution, error rates, and backend saturation are all critical. |
| Network switch | USE | Infrastructure resource with finite port bandwidth (utilization), packet drops (saturation), and CRC errors (errors). |
| Background job worker | RED | Request-driven service (processes jobs, not HTTP requests, but the pattern is the same): job rate, failure rate, processing duration. |
| Object storage (S3-compatible) | RED | Request-driven service with API calls: request rate, error rate, and latency per operation type (GET, PUT, DELETE). |

**Why this works**: The key distinction is between resources with finite capacity
(USE) and request-driven services (RED/Golden Signals). Golden Signals adds
saturation explicitly, making it ideal for user-facing entry points where both
experience and capacity matter. RED is a cleaner subset for pure request-driven
microservices. USE is the right model when the primary concern is "how full is
this thing?"

**Common mistakes**:
- Applying RED to a database (databases are resources, not request-driven services)
- Applying USE to a microservice (microservices have request semantics, not resource semantics)
- Confusing Redis as a service (it is an infrastructure resource from the monitoring perspective)

---

## Part B -- Map signals to metrics

For the HTTP API gateway, the best methodology is **Golden Signals** (or equivalently RED, since they overlap significantly here).

| Signal | PromQL Query |
|--------|-------------|
| **Latency** (p99 request duration) | `histogram_quantile(0.99, sum by (le) (rate(http_request_duration_seconds_bucket[5m])))` |
| **Traffic** (requests per second) | `sum(rate(http_requests_total[5m]))` |
| **Errors** (error rate percentage) | `sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m])) * 100` |
| **Saturation** (CPU utilization) | `100 - avg(rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100` |

**Alternative saturation queries** (depending on what is the bottleneck):

```promql
# Memory saturation
(1 - node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes) * 100

# Connection pool saturation (if the gateway has one)
gateway_connections_active / gateway_connections_max * 100

# In-flight request saturation
http_inflight_requests / http_inflight_requests_max * 100
```

**Why this works**: Each signal maps to a distinct operational question:
- Latency: "Is the user experience degraded?"
- Traffic: "How much load is the system handling?"
- Errors: "Are requests failing?"
- Saturation: "Is the system about to break?"

**Common mistakes**:
- Using average latency instead of percentiles (p99 catches outlier degradation)
- Forgetting to multiply error rate by 100 for percentage display
- Using `rate()` on a histogram without the `_bucket` suffix (need `_bucket` for `histogram_quantile`)

---

## Part C -- Identify missing signals

Available metrics for Redis:
```
redis_connected_clients
redis_used_memory_bytes
redis_maxmemory_bytes
redis_commands_processed_total
redis_keyspace_hits_total
redis_keyspace_misses_total
redis_blocked_clients
redis_connected_slaves
```

### 1. USE signals that can be fully covered:

| USE Signal | Coverage | Metric |
|-----------|----------|--------|
| **Memory Utilization** | Full | `redis_used_memory_bytes / redis_maxmemory_bytes * 100` |
| **Client Utilization** | Partial | `redis_connected_clients` (but no max_clients metric available) |
| **Errors** (blocked clients) | Partial | `redis_blocked_clients` (indicates saturation, not errors per se) |

### 2. USE signals that are missing or incomplete:

| USE Signal | Gap |
|-----------|-----|
| **CPU Utilization** | No `redis_cpu_*` metric; need `process_cpu_seconds_total` or OS-level metrics |
| **Network Utilization** | No `redis_net_*` metrics for bytes in/out |
| **Disk I/O Utilization** | No `redis_rdb_*` or `redis_aof_*` I/O metrics |
| **Network Saturation** | No dropped packet or connection rejection metrics |
| **Disk Saturation** | No I/O queue depth metrics |
| **Replication Saturation** | `redis_connected_slaves` exists but no replication lag metric |
| **Errors** | No `redis_errors_total` or rejected connection counter |

### 3. Additional metrics to instrument:

To complete the USE dashboard, export these metrics (most are available via
`redis_exporter` with appropriate flags):

```yaml
# CPU (from OS or cgroup, not Redis itself)
process_cpu_seconds_total{job="redis"}

# Network
redis_net_input_bytes_total       # bytes received
redis_net_output_bytes_total      # bytes sent
redis_rejected_connections_total  # connections rejected (maxclients hit)

# Disk I/O
redis_rdb_last_bgsave_status      # background save errors
redis_aof_last_write_status       # AOF write errors
redis_aof_last_cow_size_bytes     # copy-on-write memory usage during persistence

# Replication
redis_master_repl_offset          # for calculating replication lag
redis_slave_repl_offset           # slave replication offset

# Connection pool saturation
redis_maxclients                  # max client connections (config value)
```

**Why this works**: Redis exposes operational metrics through `INFO` command
fields, but not all are automatically exported as Prometheus metrics. The
`redis_exporter` can export most of these, but some require explicit
configuration flags (`--check-single-keys`, `--check-key-groups`).

**Common mistakes**:
- Assuming `redis_commands_processed_total` covers "errors" (it does not)
- Ignoring network saturation (Redis is often network-bound, not CPU-bound)
- Not monitoring replication lag (silent data staleness)

---

## Part D -- Scenario: monitoring plan for the payment service

| Sub-Component | Methodology | Signals | PromQL Key Metric |
|--------------|-------------|---------|-------------------|
| **HTTP API (inbound requests)** | RED | Rate, Errors, Duration | `rate(http_requests_total{service="payment-service"}[5m])` |
| **Kubernetes pods (3 instances)** | USE | CPU util, Memory util, Saturation | `rate(container_cpu_usage_seconds_total{pod=~"payment-.*"}[5m])` |
| **PostgreSQL connection pool** | USE | Active conns, Queue depth, Errors | `db_connection_pool_active{service="payment"} / db_connection_pool_max{service="payment"}` |
| **Redis (idempotency keys)** | USE | Memory util, Hit rate, Evictions | `redis_used_memory_bytes / redis_maxmemory_bytes` |
| **Stripe API (outbound calls)** | RED | Call rate, Error rate, Latency | `histogram_quantile(0.99, sum by (le) (rate(stripe_api_duration_seconds_bucket[5m])))` |
| **Webhook processing (async)** | RED | Job rate, Failure rate, Processing duration | `rate(webhook_processed_total{status="failed"}[5m])` |

**Why this works**: Each sub-component has different operational semantics.
Inbound HTTP and Stripe API calls are request-driven (RED). Infrastructure
resources like pods, DB pools, and Redis are capacity-driven (USE). The async
webhook processor is job-driven (RED applied to a queue consumer).

**Common mistakes**:
- Using a single methodology for the entire service
- Not monitoring outbound dependencies (Stripe) separately from inbound traffic
- Treating the database pool as RED when it is a resource (USE)

---

## Part E -- When frameworks overlap

### 1. Monitoring saturation with RED

RED does not explicitly include saturation, but you add it as a supplementary
row below the RED panels:

```promql
# CPU saturation for the service's pods
max(
  rate(container_cpu_usage_seconds_total{service="order-service"}[5m])
  / container_spec_cpu_quota{service="order-service"}
  * container_spec_cpu_period{service="order-service"}
) * 100

# Memory saturation
max(
  container_memory_working_set_bytes{service="order-service"}
  / container_spec_memory_limit_bytes{service="order-service"}
) * 100

# Connection pool saturation
db_connection_pool_active{service="order-service"}
/ db_connection_pool_max{service="order-service"}
* 100
```

This is common practice: RED for the request path, a supplemental USE section
for infrastructure underneath.

### 2. Monitoring query latency with USE

USE does not include latency, but for a database you add it as a supplementary
metric:

```promql
# Query latency (supplementary to USE)
histogram_quantile(0.99,
  sum by (le) (rate(pg_query_duration_seconds_bucket[5m]))
)

# Or for MySQL
histogram_quantile(0.99,
  sum by (le) (rate(mysql_query_duration_seconds_bucket[5m]))
)
```

Database dashboards typically show USE for the resource, plus a "Query
Performance" section that covers latency, throughput, and slow queries.

### 3. Combining frameworks on a single dashboard

**Yes, it is valid and common.** A well-designed L2 service dashboard often
combines:

- **RED** for the service's request path (top rows)
- **USE** for the infrastructure it runs on (bottom rows)
- **Custom signals** for business-specific metrics (order value, conversion rate)

The frameworks are not competing -- they are complementary lenses on the same
system. The dashboard structure would be:

```
Row 1: Key Metrics (stat panels) -- custom selection
Row 2: RED Method (rate, errors, duration) -- service behavior
Row 3: Dependencies (database, cache, external APIs) -- RED per dependency
Row 4: USE Method (CPU, memory, disk, network) -- infrastructure health
Row 5: Business Metrics (custom) -- domain-specific signals
```

**When to combine**: Always for L2 dashboards. L1 dashboards should be
simplified (typically just the key metrics from each framework). L3 dashboards
are specialized and may use only one framework deeply.

**Common mistakes**:
- Rigidly sticking to one framework and missing critical signals
- Mixing frameworks without clear row separation (visual confusion)
- Duplicating the same signal from different frameworks (e.g., error rate in both RED and Golden Signals rows)
