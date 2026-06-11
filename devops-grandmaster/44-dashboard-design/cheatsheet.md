# Cheatsheet: Dashboard Design

## Dashboard Hierarchy

```
L1: Overview     → Is the system healthy? (1-5 panels)
L2: Service      → Is this service healthy? (5-15 panels)
L3: Deep Dive    → What's wrong? (15-30 panels)
L4: Debug        → Root cause analysis (variable)
```

## Golden Signals (Google SRE)

| Signal | What It Measures | PromQL Example |
|--------|-----------------|----------------|
| Latency | Request duration | `histogram_quantile(0.99, rate(http_duration_bucket[5m]))` |
| Traffic | Request rate | `sum(rate(http_requests_total[5m]))` |
| Errors | Failure rate | `sum(rate(http_requests_total{code=~"5.."}[5m]))` |
| Saturation | Resource usage | `process_resident_memory_bytes / node_memory_MemTotal_bytes` |

## RED Method (Services)

```
Rate    → requests per second
Errors  → error rate
Duration → latency (p50, p95, p99)
```

## USE Method (Resources)

```
Utilization → % of resource in use
Saturation  → queue depth / waiting
Errors      → error count
```

## Panel Types & When to Use

| Type | Use For |
|------|---------|
| Time series | Trends, patterns, anomalies |
| Stat | Single current value |
| Gauge | Value within a range |
| Bar chart | Comparing discrete items |
| Table | Detailed metrics, top-N |
| Heatmap | Distribution over time |

## Grafana Dashboard Structure

```json
{
  "dashboard": {
    "title": "Service Name - Overview",
    "tags": ["service-name", "slo"],
    "templating": {
      "list": [
        {"name": "namespace", "query": "label_values(..., namespace)"},
        {"name": "service", "query": "label_values(..., service)"}
      ]
    },
    "panels": [
      {"title": "Request Rate", "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0}},
      {"title": "Error Rate", "gridPos": {"h": 8, "w": 12, "x": 12, "y": 0}},
      {"title": "Latency p99", "gridPos": {"h": 8, "w": 12, "x": 0, "y": 8}},
      {"title": "Saturation", "gridPos": {"h": 8, "w": 12, "x": 12, "y": 8}}
    ]
  }
}
```

## Design Rules

1. **One question per panel** — "Is latency normal?" not "All metrics"
2. **Most important top-left** — Eye scan follows Z-pattern
3. **Use variables** — `$namespace`, `$service` for filtering
4. **Consistent time ranges** — All panels same time window
5. **Color with meaning** — Red = bad, green = good, yellow = warning
6. **Link dashboards** — Overview → Service → Deep Dive
7. **Annotate deployments** — Show when changes happened

## Incident Dashboard Checklist

- [ ] Request rate (is traffic normal?)
- [ ] Error rate (are errors elevated?)
- [ ] Latency percentiles (p50, p95, p99)
- [ ] Saturation (CPU, memory, connections)
- [ ] Recent deployments (annotations)
- [ ] Dependency health (upstream/downstream)
