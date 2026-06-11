# Cheatsheet: Metrics Collection

## Prometheus Query Language (PromQL)

```promql
# CPU usage rate
rate(container_cpu_usage_seconds_total[5m])

# Memory usage
container_memory_usage_bytes

# HTTP request rate
rate(http_requests_total[5m])

# Error rate
rate(http_requests_total{status=~"5.."}[5m])

# 95th percentile latency
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))
```

## Key Metrics

| Metric | PromQL |
|--------|--------|
| CPU usage | `rate(container_cpu_usage_seconds_total[5m])` |
| Memory usage | `container_memory_usage_bytes` |
| Request rate | `rate(http_requests_total[5m])` |
| Error rate | `rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m])` |
| Disk usage | `node_filesystem_avail_bytes` |

## Grafana Dashboard Queries
```promql
# CPU by container
sum by (name) (rate(container_cpu_usage_seconds_total[5m]))

# Memory by container
sum by (name) (container_memory_usage_bytes)

# Request rate by status
sum by (status) (rate(http_requests_total[5m]))
```
