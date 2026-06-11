# Solution 01: Centralized Logging Architecture and Design Decisions

## Part A: Why Centralized Logging

### 1. Debugging without centralized logging

Without centralized logging, you would SSH into each of the 12 nodes
individually, grep through `/var/log/containers/` for relevant log lines, and
try to correlate timestamps manually. This introduces three critical problems:
you need to know *which* node to check (the failing pod could be on any of
them), searching serially across 12 nodes takes minutes when you need answers
in seconds, and there is no way to correlate logs from different services that
participated in the same request. During an incident, this delay directly
translates to extended outage duration.

### 2. Log aggregator vs log search engine

A **log aggregator** collects logs from multiple sources and forwards them to a
destination. It does not store or index logs itself -- it is a transport layer.
Example: Fluentd, Fluent Bit, Logstash, Promtail.

A **log search engine** stores logs in an indexed format and provides a query
language for searching and aggregating them. Example: Elasticsearch, Loki.

The aggregator sits between your applications and the search engine. This
separation lets you swap either component independently -- replace Fluentd with
Promtail without touching Elasticsearch, or replace Elasticsearch with Loki
without changing your collection layer.

### 3. Structured logging as a prerequisite

Structured logging means emitting each log line as a self-describing data
record (typically JSON) with explicit named fields rather than a free-form text
string. The most common format is single-line JSON:

```json
{"timestamp": "2024-03-15T10:30:45.123Z", "level": "ERROR", "service": "order-api", "message": "payment_failed", "order_id": "12345"}
```

Structured logging is a prerequisite for centralized logging because search
engines like Elasticsearch and Loki can index, filter, and aggregate on
individual fields without custom parsing. With unstructured text, every query
requires a regex or grok pattern that breaks when the message format changes.
Structured logging turns logs from opaque strings into queryable data.

### 4. Separation of concerns

Centralized logging systems separate collection, processing, storage, and
querying into different components for three reasons:

- **Independent scaling:** You might have 100 nodes generating logs (collection
  layer) but only need 3 Elasticsearch nodes (storage layer). Separating them
  lets you scale each independently.
- **Swappable components:** You can replace Fluentd with Fluent Bit for lighter
  resource usage without migrating your Elasticsearch cluster.
- **Fault isolation:** If Logstash crashes, Promtail buffers logs locally and
  retries. The storage layer does not lose data, and collection does not stop.
  A monolithic tool that does everything becomes a single point of failure.

## Part B: Compare the Stacks

### 1. Component roles

**Architecture 1 (ELK):**
- **Filebeat:** Lightweight agent on each node that tails log files and ships
  them to Logstash.
- **Logstash:** Processing pipeline that parses, transforms, enriches, and
  routes log data before sending it to Elasticsearch.
- **Elasticsearch:** Distributed search and analytics engine that stores logs
  as indexed documents and supports full-text search.
- **Kibana:** Web UI for searching, visualizing, and dashboarding log data
  stored in Elasticsearch.

**Architecture 2 (EFK):**
- **Fluentd:** Log collector and forwarder that runs on each node, reads
  container logs, and ships them to Elasticsearch. Lighter than Logstash and
  more Kubernetes-native.
- **Elasticsearch and Kibana:** Same roles as in ELK.

**Architecture 3 (Loki + Grafana):**
- **Promtail:** Log collection agent designed specifically for Loki. Runs on
  each node, tails log files, attaches labels, and pushes to Loki.
- **Loki:** Log aggregation system that indexes only labels (metadata), not
  log content. Stores compressed log chunks in object storage.
- **Grafana:** Unified UI for metrics, traces, and logs with native LogQL
  support and built-in correlation features.

### 2. Where full-text indexing happens

In **ELK and EFK**, Elasticsearch indexes the full content of every log
document. Every field is searchable, but this requires significant CPU and
disk I/O. At scale (hundreds of GB/day), Elasticsearch clusters need many
nodes with fast SSDs, and index management (sharding, rollover, lifecycle)
becomes a full-time job. Cost grows roughly linearly with ingest volume.

In **Loki**, only labels are indexed. The log content itself is compressed and
stored in chunks (object storage or filesystem). Queries first filter by
labels (fast, uses index), then scan the matching chunks for content (slower,
sequential reads). This makes Loki 10-100x cheaper to operate at the same
ingest volume, but full-text search across all logs is slower than
Elasticsearch.

### 3. Most Kubernetes-native

**Architecture 3 (Loki + Grafana)** is the most Kubernetes-native. Promtail
uses Kubernetes service discovery to automatically find pods and attach
metadata (namespace, pod name, node name, labels) as Loki labels. Loki's
label model matches Prometheus labels, which Kubernetes operators already
understand. The Helm charts are maintained by Grafana Labs with first-class
Kubernetes support. Fluentd (EFK) is also Kubernetes-native with DaemonSet
deployment, but Elasticsearch's operational complexity does not align well
with Kubernetes' declarative model.

### 4. Best fit for Grafana-centric correlation

**Architecture 3 (Loki + Grafana)** is the best fit. Grafana natively
supports Prometheus (metrics), Tempo (traces), and Loki (logs) as datasources
with built-in correlation features:

- Click a metric exemplar to jump to the trace in Tempo
- From a trace span, click to see correlated logs in Loki
- From a log line, click the trace_id to jump to the full trace
- All three signals in one UI with shared time ranges and variables

ELK and EFK require additional tools (Jaeger or Zipkin for traces, Prometheus
for metrics) and do not have native correlation in Kibana.

## Part C: Identify the Architecture Problems

### Scenario 1: Single Elasticsearch Node

**Problem:** A single Elasticsearch node is a single point of failure with no
horizontal scaling. As ingest grows, the node cannot keep up with indexing,
and queries compete with ingest for CPU and disk I/O. Disk fills up because
there is no index lifecycle management -- old indices are never deleted.

**Fix:**
- Scale to a minimum 3-node Elasticsearch cluster with dedicated master,
  data, and coordinating roles.
- Implement Index Lifecycle Management (ILM) policies: hot phase (7 days,
  SSD), warm phase (30 days, read-only), delete after 90 days.
- Add a Logstash or Beats buffer to smooth ingest spikes.
- Consider switching to Loki if full-text search is not critical -- single
  binary mode is simpler to operate.

### Scenario 2: Logging Everything

**Problem:** 500 GB/day of DEBUG logs in production is unsustainable.
Most DEBUG output is noise (loop iterations, cache lookups, variable dumps)
that adds cost without adding value. Elasticsearch storage costs grow
linearly with volume, and queries slow down as indices grow.

**Fix:**
- Set the application log level to INFO in production. DEBUG should be
  disabled by default and enabled temporarily via environment variable for
  specific debugging sessions.
- Add a pre-ingest filter in Fluentd/Logstash/Promtail to drop known noisy
  log patterns (health checks, readiness probes, metrics endpoints).
- Use sampling for high-volume INFO logs if needed (e.g., log 10% of
  successful request logs).
- Set retention policies: keep ERROR/WARN logs for 30 days, INFO logs for
  7 days.

### Scenario 3: No Labels in Loki

**Problem:** With only `{job="containers"}`, every Loki query scans all log
lines from every container. There is no way to narrow the search by service,
namespace, or pod without reading the log content. This makes queries
extremely slow and expensive because Loki must decompress and scan every chunk.

**Fix:**
Update the Promtail relabel configuration to attach meaningful labels:

```yaml
relabel_configs:
  - source_labels: [__meta_kubernetes_namespace]
    target_label: namespace
  - source_labels: [__meta_kubernetes_pod_label_app]
    target_label: app
  - source_labels: [__meta_kubernetes_pod_name]
    target_label: pod
  - source_labels: [__meta_kubernetes_pod_node_name]
    target_label: node
```

This lets you query `{namespace="production", app="order-api"}` which only
scans relevant chunks. Be careful not to add high-cardinality labels like
`pod` to every query -- use them for drill-down, not as primary selectors.

## Part D: Sample Logging Architecture

### Chosen stack: Loki + Grafana

**Reason:** Prometheus and Grafana are already in use for metrics. Loki
integrates natively, uses the same label model, and the team gets metrics,
traces, and logs in a single UI. Operational cost is lower than ELK because
Loki does not require a JVM, does not need complex index management, and can
use object storage for log data.

### Collection architecture

```
Frontend (Node.js)  ─┐
API Services (Py/Go) ─┤
PostgreSQL           ─┼─> Promtail (DaemonSet) ─> Loki ─> Grafana
Redis                ─┤
RabbitMQ             ─┤
nginx ingress        ─┘
```

- Promtail runs as a DaemonSet on every Kubernetes node.
- It tails `/var/log/containers/*.log` using Kubernetes service discovery.
- Application logs are expected to be structured JSON written to stdout.
- Database and infrastructure logs (PostgreSQL, Redis, RabbitMQ, nginx) are
  written to stdout via container configuration.

### Labels

| Label | Source | Cardinality |
|-------|--------|-------------|
| `namespace` | Kubernetes namespace | Low (5-10) |
| `app` | Pod label `app.kubernetes.io/name` | Medium (20-50) |
| `level` | Parsed from JSON `level` field | Low (5) |
| `node` | Kubernetes node name | Low (10-20) |

Labels to avoid (keep in log content, not labels): `user_id`, `request_id`,
`order_id`, `ip_address` -- these are high-cardinality and would explode the
index.

### Retention policy

| Tier | Duration | Storage | Applies to |
|------|----------|---------|------------|
| Hot | 7 days | SSD (node filesystem) | All logs |
| Warm | 30 days | S3-compatible storage | All logs |
| Cold | 90 days | S3 Glacier / archive | ERROR and WARN only |
| Delete | After 90 days | -- | All logs |

Enforced via Loki's `limits_config.retention_period` and compactor
configuration. Per-namespace overrides for compliance-sensitive namespaces.

### Volume reduction

1. **Drop at source:** Configure Promtail to drop health check, readiness
   probe, and metrics endpoint logs before shipping.
2. **Level filtering:** Applications log at INFO in production. DEBUG is
   enabled only for specific services via environment variable during
   incidents.
3. **Sampling:** High-volume successful request logs are sampled at 10%.
   Error and warning logs are never sampled.
4. **Compression:** Loki compresses chunks by default (snappy). Promtail
   sends batches to reduce HTTP overhead.

## Common Mistakes

- **Choosing ELK "because it is more powerful"** without considering
  operational cost. Elasticsearch requires significant expertise to operate at
  scale -- shard management, JVM tuning, index lifecycle. Loki trades
  full-text search speed for dramatically lower operational burden.
- **Using high-cardinality labels in Loki.** Labels like `user_id` or
  `request_id` create millions of streams and crash the index. Keep these as
  fields in log content, not as labels.
- **Not structuring logs before shipping.** Sending unstructured text to Loki
  means every query requires regex parsing, which is slow and fragile. Fix
  the application to emit JSON before building the logging infrastructure.
- **Treating all logs equally.** Not all logs need 90-day retention.
  Debug-level logs can be dropped after 7 days. Error logs with compliance
  implications need longer retention. Different tiers for different values.
- **Ignoring the collector layer.** Promtail/Fluentd runs on every node and
  consumes CPU and memory. Monitor it, set resource limits, and plan for the
  overhead in node sizing.
