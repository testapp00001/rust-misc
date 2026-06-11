# Module 41: Centralized Logging -- ELK Stack, Loki, Log Aggregation

> **Previous Module:** [40 -- Distributed Tracing](../40-distributed-tracing/README.md)
> **Next Module:** [42 -- Alerting Systems](../42-alerting-systems/README.md)
> **Phase:** 5 -- Observability

---

## 1. The Problem

Your application logs to stdout. In Kubernetes, the container runtime captures stdout and writes it to files on the node. When something goes wrong, you SSH into the server and `tail -f` the log file. This works for one server running one application.

Now multiply by 50 microservices across 20 nodes, each producing hundreds of log lines per second. You need to:

- Find all log lines related to a specific request ID across all services
- Search for error patterns across the entire fleet
- Correlate logs with the trace you found in Jaeger
- Set up alerts when specific error patterns appear
- Retain logs for 30 days for compliance but not pay a fortune

Without centralized logging, you are SSH-ing into servers at 3 AM, grepping through files, and hoping you pick the right server. This does not scale, and it does not work during incidents when you need answers fast.

---

## 2. The Naive Way

### Anti-Pattern: SSH and Grep

```bash
# "Centralized logging" via SSH
for host in web-{01..20}.prod.example.com; do
  echo "=== $host ==="
  ssh "$host" "grep 'ERROR' /var/log/app/app.log | tail -10"
done
```

Problems:
- You need to know which servers to check
- Searching 20 servers serially takes minutes
- No correlation between logs from different services
- No full-text search or structured querying
- Log files rotate, so you lose history
- SSH access to production is a security risk

### Anti-Pattern: Logging to a Shared Filesystem

```yaml
# Mounting a shared NFS volume for logs
volumeMounts:
  - name: shared-logs
    mountPath: /var/log/app
volumes:
  - name: shared-logs
    nfs:
      server: nfs-server
      path: /exports/logs
```

Problems:
- NFS is not designed for high-throughput writes from many sources
- File locking becomes a bottleneck
- No indexing or search -- you still grep
- Single point of failure
- Corrupts under high concurrency

### Anti-Pattern: Logging Everything at DEBUG Level

```python
# Logging every detail in production
logger.debug(f"Processing request {request_id}")
logger.debug(f"Database query: {query}")
logger.debug(f"Query result: {result}")  # Could be megabytes of data
logger.debug(f"Cache lookup: {cache_key}")
logger.debug(f"Cache hit: {cache_hit}")
logger.debug(f"Response body: {response_body}")
```

This generates enormous volumes of log data, costs a fortune to store, and buries the important messages in noise.

---

## 3. The Right Way

### Structured Logging

The foundation of effective centralized logging is structured logging -- emitting logs as structured data (JSON) rather than free-form text.

**Unstructured (bad):**
```
2024-03-15 10:23:45 ERROR [order-service] Failed to process order 12345: database connection timeout after 30s
```

**Structured (good):**
```json
{
  "timestamp": "2024-03-15T10:23:45.123Z",
  "level": "ERROR",
  "service": "order-service",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "span_id": "b7ad6b7169203331",
  "message": "Failed to process order",
  "order_id": "12345",
  "error": "database connection timeout",
  "duration_ms": 30000,
  "user_id": "user-456",
  "retry_count": 3
}
```

Structured logs are searchable by any field. You can query: "show me all ERROR logs from order-service where order_id is 12345" -- impossible with unstructured logs.

**Python (structlog):**

```python
import structlog
import logging

structlog.configure(
    processors=[
        structlog.contextvars.merge_contextvars,
        structlog.processors.add_log_level,
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.processors.StackInfoRenderer(),
        structlog.processors.format_exc_info,
        structlog.processors.JSONRenderer(),
    ],
    wrapper_class=structlog.make_filtering_bound_logger(logging.INFO),
    context_class=dict,
    logger_factory=structlog.PrintLoggerFactory(),
    cache_logger_on_first_use=True,
)

logger = structlog.get_logger()

# Usage
logger.info("order_created",
    order_id="12345",
    user_id="user-456",
    total=99.99,
    item_count=3,
)

logger.error("payment_failed",
    order_id="12345",
    error="stripe_declined",
    payment_provider="stripe",
    amount=99.99,
    retry_count=3,
)
```

**Go (zerolog):**

```go
package main

import (
    "os"
    "github.com/rs/zerolog"
)

func main() {
    logger := zerolog.New(os.Stdout).With().
        Timestamp().
        Str("service", "order-service").
        Logger()

    logger.Info().
        Str("order_id", "12345").
        Str("user_id", "user-456").
        Float64("total", 99.99).
        Msg("order_created")

    logger.Error().
        Str("order_id", "12345").
        Err(err).
        Int("retry_count", 3).
        Msg("payment_failed")
}
```

**Rust (tracing + tracing-subscriber):**

```rust
use tracing::{info, error, instrument};
use tracing_subscriber::fmt;

#[instrument(skip(db), fields(order_id = %order_id))]
async fn process_order(order_id: &str, db: &Database) -> Result<(), Error> {
    info!("processing order");

    let order = db.find_order(order_id).await
        .map_err(|e| {
            error!(error = %e, "database query failed");
            e
        })?;

    info!(total = order.total, item_count = order.items.len(), "order_processed");
    Ok(())
}
```

### ELK Stack (Elasticsearch, Logstash, Kibana)

The ELK stack is the most established centralized logging platform.

```
Applications --> Logstash/Flexport --> Elasticsearch --> Kibana
                    (ingest)            (store+index)    (UI)

Alternative: Filebeat --> Logstash --> Elasticsearch --> Kibana
              (ship)      (process)     (store)         (UI)
```

**Components:**

- **Elasticsearch:** Distributed search and analytics engine. Stores logs as indexed documents. Supports full-text search, aggregations, and complex queries via its Query DSL.
- **Logstash:** Data processing pipeline. Receives logs, parses/transforms them, and sends to Elasticsearch. Supports filters for parsing, enrichment, and routing.
- **Kibana:** Web UI for searching, visualizing, and dashboarding log data.
- **Filebeat:** Lightweight log shipper that runs on each node, tails log files, and forwards to Logstash or Elasticsearch.

**Logstash pipeline example:**

```ruby
# logstash.conf
input {
  beats {
    port => 5044
  }
}

filter {
  # Parse JSON logs
  json {
    source => "message"
    target => "parsed"
  }

  # Extract fields from JSON
  mutate {
    rename => {
      "[parsed][service]" => "service"
      "[parsed][level]" => "level"
      "[parsed][trace_id]" => "trace_id"
      "[parsed][order_id]" => "order_id"
    }
  }

  # Parse timestamp
  date {
    match => ["[parsed][timestamp]", "ISO8601"]
    target => "@timestamp"
  }

  # GeoIP enrichment for IP addresses
  if [client_ip] {
    geoip {
      source => "client_ip"
    }
  }

  # Drop health check logs
  if [message] =~ /health/ {
    drop { }
  }
}

output {
  elasticsearch {
    hosts => ["elasticsearch:9200"]
    index => "logs-%{service}-%{+YYYY.MM.dd}"
  }
}
```

### EFK Stack (Elasticsearch, Fluentd, Kibana)

EFK replaces Logstash with Fluentd, which is lighter weight and more Kubernetes-native.

```yaml
# Fluentd DaemonSet for Kubernetes
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: fluentd
  namespace: logging
spec:
  selector:
    matchLabels:
      name: fluentd
  template:
    metadata:
      labels:
        name: fluentd
    spec:
      tolerations:
        - key: node-role.kubernetes.io/master
          effect: NoSchedule
      containers:
        - name: fluentd
          image: fluent/fluentd-kubernetes-daemonset:v1.16-debian-elasticsearch8-1
          env:
            - name: FLUENT_ELASTICSEARCH_HOST
              value: "elasticsearch.logging.svc.cluster.local"
            - name: FLUENT_ELASTICSEARCH_PORT
              value: "9200"
            - name: FLUENT_ELASTICSEARCH_SCHEME
              value: "http"
          volumeMounts:
            - name: varlog
              mountPath: /var/log
            - name: containers
              mountPath: /var/lib/docker/containers
              readOnly: true
      volumes:
        - name: varlog
          hostPath:
            path: /var/log
        - name: containers
          hostPath:
            path: /var/lib/docker/containers
```

### Loki + Grafana

Loki is Grafana Labs' log aggregation system. It is designed to be cost-effective: it indexes only metadata (labels), not log content. Logs are compressed and stored in object storage (S3, GCS).

```
Applications --> Promtail/Loki --> Loki --> Grafana
                 (ship)          (store)    (UI)

Alternative:  Applications --> OTel Collector --> Loki --> Grafana
```

**Why Loki instead of ELK?**

- **Cost:** Loki indexes labels only, not full text. Storage is 10-100x cheaper than Elasticsearch
- **Simplicity:** No complex mapping schemas or index management
- **Integration:** Native Grafana integration -- metrics, traces, and logs in one UI
- **LogQL:** Similar to PromQL, familiar to Prometheus users
- **Labels:** Uses the same label model as Prometheus

**Loki's trade-off:** Full-text search is slower than Elasticsearch. Loki is optimized for label-based queries with content filtering, not arbitrary full-text search.

### LogQL (Loki's Query Language)

```logql
# All logs from order-service
{service="order-service"}

# All error logs from order-service
{service="order-service", level="ERROR"}

# Logs from multiple services
{service=~"order|payment|user"}

# Filter for specific text
{service="order-service"} |= "timeout"

# Exclude health checks
{service="order-service"} != "health"

# Regex filter
{service="order-service"} |~ "status=[45]\\d{2}"

# Parse JSON and filter
{service="order-service"} | json | order_id="12345"

# Parse logfmt
{service="order-service"} | logfmt | level="ERROR"

# Line format (extract and format fields)
{service="order-service"} | json | line_format "{{.order_id}} - {{.message}}"

# Unwrap for numeric operations (e.g., latency analysis)
{service="order-service"} | json | unwrap duration_ms

# Count errors per service
sum by (service) (count_over_time({level="ERROR"}[5m]))

# Error rate
sum(rate({service="order-service", level="ERROR"}[5m]))
/ sum(rate({service="order-service"}[5m]))

# Top 10 error messages
topk(10,
  sum by (message) (count_over_time({level="ERROR"}[5m]))
)
```

### Loki Configuration

```yaml
# loki-config.yml
auth_enabled: false

server:
  http_listen_port: 3100

common:
  path_prefix: /loki
  storage:
    filesystem:
      chunks_directory: /loki/chunks
      rules_directory: /loki/rules
  replication_factor: 1
  ring:
    kvstore:
      store: inmemory

schema_config:
  configs:
    - from: 2024-01-01
      store: tsdb
      object_store: filesystem
      schema: v13
      index:
        prefix: index_
        period: 24h

limits_config:
  retention_period: 30d            # Keep logs for 30 days
  max_query_length: 721h           # Max query range
  ingestion_rate_mb: 10            # Max ingest rate per tenant
  ingestion_burst_size_mb: 20
  per_stream_rate_limit: 5MB
  per_stream_rate_limit_burst: 15MB

compactor:
  working_directory: /loki/compactor
  compaction_interval: 10m
  retention_enabled: true          # Enable retention deletion
  retention_delete_delay: 2h
  retention_delete_worker_count: 150
```

### Promtail (Log Shipper for Loki)

```yaml
# promtail-config.yml
server:
  http_listen_port: 9080

positions:
  filename: /tmp/positions.yaml

clients:
  - url: http://loki:3100/loki/api/v1/push

scrape_configs:
  # Kubernetes pod logs
  - job_name: kubernetes-pods
    kubernetes_sd_configs:
      - role: pod
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_label_app]
        target_label: app
      - source_labels: [__meta_kubernetes_namespace]
        target_label: namespace
      - source_labels: [__meta_kubernetes_pod_name]
        target_label: pod
      - source_labels: [__meta_kubernetes_pod_node_name]
        target_label: node
    pipeline_stages:
      # Parse container logs
      - cri: {}
      # Parse JSON structured logs
      - json:
          expressions:
            level: level
            trace_id: trace_id
            message: message
      - labels:
          level:
          trace_id:
      # Drop health check logs
      - match:
          selector: '{app="api-server"}'
          stages:
            - regex:
                expression: '/health|/ready|/metrics'
            - metrics:
                http_health_check_total:
                  type: Counter
                  description: "Total health check requests"
                  source: ""
                  config:
                    action: inc
```

### Log Retention and Cost Optimization

```yaml
# Tiered retention in Loki
schema_config:
  configs:
    # Hot tier: recent data, fast storage
    - from: 2024-01-01
      store: tsdb
      object_store: filesystem  # or SSD-backed storage
      schema: v13
      index:
        prefix: hot_
        period: 24h

# Compactor handles retention
compactor:
  retention_enabled: true
  retention_delete_delay: 2h

# Per-tenant limits
limits_config:
  # Different retention per tenant
  per_tenant_override_config: /etc/loki/overrides.yaml
```

```yaml
# overrides.yaml -- per-tenant retention
overrides:
  "premium-tenant":
    retention_period: 90d
    ingestion_rate_mb: 50
  "free-tenant":
    retention_period: 7d
    ingestion_rate_mb: 5
```

**Cost optimization strategies:**

1. **Structured logging:** JSON logs are more compressible and queryable
2. **Drop noisy logs:** Filter health checks, readiness probes, and debug logs before shipping
3. **Label wisely:** High-cardinality labels increase index size
4. **Use object storage:** S3/GCS is 10x cheaper than block storage for log data
5. **Set retention by tier:** 7 days hot (SSD), 30 days warm (HDD), 90 days cold (S3)
6. **Compress aggressively:** Enable gzip/snappy compression in your log shipper

### Logging Stack Comparison

| Feature | ELK Stack | Loki + Grafana | ClickHouse |
|---------|-----------|----------------|------------|
| Storage model | Full-text index | Label-based + chunks | Columnar |
| Query language | KQL / Lucene | LogQL | SQL |
| Resource usage | High (JVM) | Low (Go) | Medium |
| Full-text search | Excellent | Limited | Good |
| Cost at scale | High | Low | Medium |
| Best for | Complex queries | Grafana-native stacks | Analytics |

---

## 4. The Production Way

### The Three Pillars Together

In production, you correlate across all three observability signals:

```
Metric: "Error rate spiked at 10:23"
  -> Trace: "Here is a failed trace at 10:23" (via exemplars)
    -> Log: "Here is the detailed error from that trace" (via trace_id)
```

**Correlation in Grafana:**

```yaml
# Grafana datasource configuration
datasources:
  - name: Prometheus
    type: prometheus
    url: http://prometheus:9090
    jsonData:
      exemplarTraceIdDestinations:
        - name: trace_id
          datasourceUid: tempo

  - name: Tempo
    type: tempo
    url: http://tempo:3200
    jsonData:
      tracesToLogs:
        datasourceUid: loki
        filterByTraceID: true
        tags: ['service']
        mappedTags:
          - key: service.name
            value: service

  - name: Loki
    type: loki
    url: http://loki:3100
    jsonData:
      derivedFields:
        - datasourceUid: tempo
          matcherRegex: "trace_id=(\\w+)"
          name: TraceID
          url: '$${__value.raw}'
```

This lets you:
1. See a spike on a Prometheus graph
2. Click an exemplar to jump to the specific trace in Tempo
3. From the trace, click to see the correlated logs in Loki
4. From a log line, click the trace_id link to see the full trace

### Logging in Kubernetes

```yaml
# Application deployment with logging best practices
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-server
spec:
  template:
    spec:
      containers:
        - name: api
          image: api-server:latest
          # Log to stdout/stderr -- let the platform handle collection
          env:
            - name: LOG_LEVEL
              value: "info"
            - name: LOG_FORMAT
              value: "json"
            - name: LOG_OUTPUT
              value: "stdout"
          resources:
            requests:
              memory: "256Mi"
              cpu: "250m"
            limits:
              memory: "512Mi"
              cpu: "500m"
```

### Log Volume Management

```python
# Dynamic log level adjustment
import logging
import os

class DynamicLogLevel:
    def __init__(self):
        self.level = os.getenv("LOG_LEVEL", "INFO").upper()

    def should_debug(self, module: str) -> bool:
        # Enable debug for specific modules on demand
        debug_modules = os.getenv("LOG_DEBUG_MODULES", "").split(",")
        return module in debug_modules

logger = logging.getLogger(__name__)

# In production, only these modules log at DEBUG
# LOG_DEBUG_MODULES=payment,order-processor
```

### Error Pattern Detection

```logql
# Find new error patterns (errors you haven't seen before)
# Compare error messages in last 1h vs previous 24h

# Current hour errors
sum by (message) (
  count_over_time(
    {service="api-server", level="ERROR"} | json | message != "" [1h]
  )
)
>
# Previous 24h average per hour
sum by (message) (
  count_over_time(
    {service="api-server", level="ERROR"} | json | message != "" [24h]
  )
) / 24
* 3  # 3x the average
```

### Log Query Performance

```logql
# SLOW: Searches all logs, then filters
{namespace="production"} |= "timeout"

# FASTER: Filter by service first
{namespace="production", service="order-service"} |= "timeout"

# FASTEST: Use multiple label filters
{namespace="production", service="order-service", level="ERROR"} |= "timeout"

# Use line_filters early to reduce data scanned
{service="order-service"} |= "error" !~ "health" | json | duration_ms > 1000
```

---

## 5. Hands-On Lab

### Lab: Loki + Grafana Log Aggregation

**Objective:** Set up Loki with Promtail, ship structured logs, query with LogQL, and correlate with traces.

**Step 1: Project structure**

```bash
mkdir loki-lab && cd loki-lab
mkdir -p app promtail grafana/provisioning/datasources
```

**Step 2: Create a structured logging application**

```python
# app/main.py
from flask import Flask, jsonify, request
import json
import logging
import os
import sys
import time
import random
import uuid

# Configure structured JSON logging
class JSONFormatter(logging.Formatter):
    def format(self, record):
        log_record = {
            "timestamp": self.formatTime(record, self.datefmt),
            "level": record.levelname,
            "service": os.getenv("SERVICE_NAME", "api-server"),
            "message": record.getMessage(),
            "logger": record.name,
        }
        # Add extra fields
        if hasattr(record, 'trace_id'):
            log_record["trace_id"] = record.trace_id
        if hasattr(record, 'order_id'):
            log_record["order_id"] = record.order_id
        if hasattr(record, 'user_id'):
            log_record["user_id"] = record.user_id
        if hasattr(record, 'duration_ms'):
            log_record["duration_ms"] = record.duration_ms
        if record.exc_info:
            log_record["exception"] = self.formatException(record.exc_info)
        return json.dumps(log_record)

# Set up logger
handler = logging.StreamHandler(sys.stdout)
handler.setFormatter(JSONFormatter())
logger = logging.getLogger(__name__)
logger.addHandler(handler)
logger.setLevel(getattr(logging, os.getenv("LOG_LEVEL", "INFO")))

app = Flask(__name__)

@app.route('/api/orders', methods=['POST'])
def create_order():
    start = time.time()
    order_id = f"order-{random.randint(1000, 9999)}"
    trace_id = uuid.uuid4().hex[:32]
    user_id = request.json.get("user_id", "anonymous") if request.json else "anonymous"

    extra = {"trace_id": trace_id, "order_id": order_id, "user_id": user_id}

    logger.info("order_received", extra=extra)

    # Simulate processing
    time.sleep(random.uniform(0.05, 0.2))

    # Simulate occasional errors
    if random.random() < 0.1:
        duration_ms = (time.time() - start) * 1000
        logger.error("payment_failed",
            extra={**extra, "error": "stripe_declined", "duration_ms": round(duration_ms)})
        return jsonify({"error": "Payment failed", "order_id": order_id}), 402

    # Simulate occasional slow requests
    if random.random() < 0.05:
        time.sleep(random.uniform(1, 3))
        logger.warning("slow_processing_detected",
            extra={**extra, "duration_ms": round((time.time() - start) * 1000)})

    duration_ms = (time.time() - start) * 1000
    logger.info("order_completed",
        extra={**extra, "duration_ms": round(duration_ms), "total": 99.99})

    return jsonify({"order_id": order_id, "status": "created"})

@app.route('/api/orders/<order_id>')
def get_order(order_id):
    trace_id = uuid.uuid4().hex[:32]
    logger.info("order_fetched", extra={"trace_id": trace_id, "order_id": order_id})
    return jsonify({"order_id": order_id, "status": "shipped"})

@app.route('/health')
def health():
    return jsonify({"status": "healthy"})

@app.route('/api/error')
def trigger_error():
    try:
        raise ValueError("Simulated error for testing")
    except ValueError:
        logger.error("unhandled_exception", exc_info=True, extra={
            "trace_id": uuid.uuid4().hex[:32],
            "endpoint": "/api/error"
        })
        return jsonify({"error": "Internal Server Error"}), 500

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=8080)
```

```dockerfile
# app/Dockerfile
FROM python:3.11-slim
WORKDIR /app
RUN pip install flask
COPY main.py .
CMD ["python", "main.py"]
```

**Step 3: Loki configuration**

```yaml
# loki-config.yml
auth_enabled: false

server:
  http_listen_port: 3100

common:
  path_prefix: /loki
  storage:
    filesystem:
      chunks_directory: /loki/chunks
      rules_directory: /loki/rules
  replication_factor: 1
  ring:
    kvstore:
      store: inmemory

schema_config:
  configs:
    - from: 2024-01-01
      store: tsdb
      object_store: filesystem
      schema: v13
      index:
        prefix: index_
        period: 24h

limits_config:
  retention_period: 168h  # 7 days
```

**Step 4: Promtail configuration**

```yaml
# promtail-config.yml
server:
  http_listen_port: 9080

positions:
  filename: /tmp/positions.yaml

clients:
  - url: http://loki:3100/loki/api/v1/push

scrape_configs:
  - job_name: docker
    docker_sd_configs:
      - host: unix:///var/run/docker.sock
        refresh_interval: 5s
    relabel_configs:
      - source_labels: ['__meta_docker_container_name']
        regex: '/(.*)'
        target_label: 'container'
      - source_labels: ['__meta_docker_container_log_stream']
        target_label: 'logstream'
      - source_labels: ['__meta_docker_container_label_com_docker_compose_service']
        target_label: 'service'
    pipeline_stages:
      - docker: {}
      - json:
          expressions:
            level: level
            service: service
            trace_id: trace_id
            order_id: order_id
            message: message
            duration_ms: duration_ms
      - labels:
          level:
          service:
```

**Step 5: Grafana provisioning**

```yaml
# grafana/provisioning/datasources/loki.yml
apiVersion: 1
datasources:
  - name: Loki
    type: loki
    access: proxy
    url: http://loki:3100
    isDefault: true
    jsonData:
      derivedFields:
        - datasourceUid: ""
          matcherRegex: "trace_id=(\\w+)"
          name: TraceID
          url: ''
```

**Step 6: Docker Compose**

```yaml
version: '3.8'
services:
  loki:
    image: grafana/loki:latest
    ports:
      - "3100:3100"
    volumes:
      - ./loki-config.yml:/etc/loki/config.yml
      - loki-data:/loki
    command: -config.file=/etc/loki/config.yml

  promtail:
    image: grafana/promtail:latest
    volumes:
      - ./promtail-config.yml:/etc/promtail/config.yml
      - /var/run/docker.sock:/var/run/docker.sock:ro
    command: -config.file=/etc/promtail/config.yml
    depends_on:
      - loki

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - ./grafana/provisioning:/etc/grafana/provisioning

  app:
    build: ./app
    ports:
      - "8080:8080"
    labels:
      com.docker.compose.service: api-server

volumes:
  loki-data:
```

**Step 7: Generate traffic and query logs**

```bash
# Start the stack
docker-compose up -d --build

# Wait for services
sleep 10

# Generate traffic
for i in $(seq 1 100); do
  curl -s -X POST http://localhost:8080/api/orders \
    -H "Content-Type: application/json" \
    -d '{"user_id": "user-'$((RANDOM % 100))'"}' > /dev/null &
  sleep 0.1
done
wait

# Generate some errors
for i in $(seq 1 10); do
  curl -s http://localhost:8080/api/error > /dev/null
done
```

**Step 8: Explore in Grafana**

Open http://localhost:3000 (admin/admin) and go to Explore.

Try these LogQL queries:

```logql
# All logs from our app
{service="api-server"}

# Only errors
{service="api-server"} | json | level="ERROR"

# Search for payment failures
{service="api-server"} | json | message="payment_failed"

# Find slow requests
{service="api-server"} | json | duration_ms > 500

# Count errors per minute
sum(count_over_time({service="api-server"} | json | level="ERROR" [1m]))

# Error rate over time
sum(rate({service="api-server"} | json | level="ERROR" [5m]))
/ sum(rate({service="api-server"} | json [5m]))

# Top error messages
topk(5, sum by (message) (count_over_time({service="api-server"} | json | level="ERROR" [5m])))

# Find a specific order's journey
{service="api-server"} | json | order_id="order-1234"

# Latency distribution
{service="api-server"} | json | unwrap duration_ms | quantile_over_time(0.99, {service="api-server"} | json | unwrap duration_ms [5m])
```

**Step 9: Create a Grafana Dashboard**

Create panels:
1. **Log Volume** (Time series): `sum by (level) (count_over_time({service="api-server"} | json [1m]))`
2. **Error Rate** (Gauge): `sum(rate({service="api-server"} | json | level="ERROR" [5m])) / sum(rate({service="api-server"} | json [5m])) * 100`
3. **Top Errors** (Table): `topk(10, sum by (message) (count_over_time({service="api-server"} | json | level="ERROR" [1h])))`
4. **P99 Latency** (Time series): `quantile_over_time(0.99, {service="api-server"} | json | unwrap duration_ms [5m])`

**Expected outcome:**
- Structured JSON logs flowing from the app through Promtail to Loki
- All logs queryable in Grafana Explore
- Dashboard showing log volume, error rates, and latency
- Ability to trace a specific order_id across multiple log lines

---

## 6. Limitation

Centralized logging gives you the ability to search, filter, and analyze logs from all services in one place. Combined with structured logging and correlation with traces, you can debug issues quickly: find the error, see the full request path, read the detailed context.

But finding the problem is only half the battle. You also need to be **notified** when problems occur -- before users start complaining. A human cannot watch dashboards 24/7. You need automated alerting that wakes up the right person, at the right time, with the right context.

The challenge is doing this without creating alert fatigue -- too many false alarms desensitize on-call engineers and cause them to miss real incidents.

**Next Module:** [42 -- Alerting Systems](../42-alerting-systems/README.md) -- Build intelligent alerting with AlertManager, PagerDuty, and on-call rotations that notify the right people without causing fatigue.
