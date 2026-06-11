# Cheatsheet: Uptime Commitment

## SLA/SLO/SLI Definitions

| Term | What | Example |
|------|------|---------|
| SLI | What you measure | Request latency, error rate |
| SLO | What you promise yourself | 99.9% availability |
| SLA | What you promise customers | 99.5% availability (with penalties) |

## Uptime Equivalents

| Uptime | Downtime/Year | Downtime/Month |
|--------|---------------|----------------|
| 99% | 3.65 days | 7.31 hours |
| 99.5% | 1.83 days | 3.65 hours |
| 99.9% | 8.76 hours | 43.83 minutes |
| 99.95% | 4.38 hours | 21.92 minutes |
| 99.99% | 52.60 minutes | 4.38 minutes |
| 99.999% | 5.26 minutes | 26.30 seconds |

## Error Budget

```
Error Budget = 1 - SLO
SLO 99.9% → Error Budget = 0.1% = 43.83 min/month

If you burn through your error budget:
- Stop feature deployments
- Focus on reliability
- Post-mortem for every incident
```

## Common SLIs

| Category | SLI | Measurement |
|----------|-----|-------------|
| Availability | Successful requests | % of 2xx responses |
| Latency | Request duration | p50, p95, p99 |
| Throughput | Requests per second | RPS |
| Error Rate | Failed requests | % of 5xx responses |
| Saturation | Resource usage | CPU, memory, disk % |
