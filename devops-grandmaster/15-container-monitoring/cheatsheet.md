# Cheatsheet: Container Monitoring

## Quick Monitoring
```bash
# Real-time stats
docker stats

# Specific container
docker stats my-app --no-stream

# Output:
# CONTAINER   CPU %   MEM USAGE / LIMIT   MEM %   NET I/O       BLOCK I/O
# my-app      0.50%   50MiB / 256MiB      19.5%   1.2kB / 0B    0B / 0B
```

## Resource Limits
```bash
# Set memory limit
docker run --memory 256m --memory-swap 512m my-app

# Set CPU limit
docker run --cpus 0.5 my-app  # Half a CPU core

# Set CPU shares (relative weight)
docker run --cpu-shares 512 my-app
```

## Prometheus + Grafana Stack
```
Container → cAdvisor → Prometheus → Grafana
              ↓
         Metrics endpoint
```

## Key Metrics to Monitor

| Metric | What | Alert When |
|--------|------|------------|
| CPU usage | Container CPU | > 80% for 5min |
| Memory usage | Container RAM | > 90% |
| Network I/O | Bytes in/out | Unusual spikes |
| Disk I/O | Read/write ops | High latency |
| Restarts | Container restarts | Any restart |
| OOM kills | Out of memory | Any OOM kill |

## USE Method
```
Utilization  → % of resource used
Saturation   → Work waiting (queue depth)
Errors       → Error count
```

## RED Method
```
Rate    → Requests per second
Errors  → Error rate
Duration → Request latency
```
