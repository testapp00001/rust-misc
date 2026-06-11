# Solution 05: Capacity Plan for Product Launch

## Part A: Current State Analysis

```
Current traffic:         3,000 RPS
Sustainable capacity:    6,000 RPS
Breaking point:          8,000 RPS

Headroom: (1 - 3000/6000) * 100 = 50%
Can handle 10x launch (30,000 RPS)?  NO — needs 5x current capacity
Can handle 5x sustained (15,000 RPS)? NO — needs 2.5x current capacity

Gap for launch:   30,000 - 6,000 = 24,000 RPS (5x current capacity)
Gap for sustained: 15,000 - 6,000 = 9,000 RPS (2.5x current capacity)
```

---

## Part B: Required Infrastructure Changes

### 1. Scale Web Pods (5 -> 25)
- **Impact**: 5x application capacity (from 6,000 to ~30,000 RPS)
- **Cost**: 20 additional pods * 2 vCPU * $0.10/hr * 730 hrs = $2,920/month
- **Lead time**: Immediate (kubectl scale)
- **Prerequisite**: Sufficient node capacity

### 2. Add Node Capacity
- **Impact**: Support 25 web pods + PgBouncer + workers
- **Cost**: 10 additional nodes * 4 vCPU * $0.10/hr * 730 hrs = $2,920/month
- **Lead time**: 1-2 hours (ASG scale-up)
- **Prerequisite**: ASG max size increased

### 3. Add PgBouncer (Connection Pooling)
- **Impact**: Multiplex 10,000+ app connections into 200 DB connections
- **Cost**: 3 pods * 0.5 vCPU * $0.10/hr * 730 hrs = $109.50/month
- **Lead time**: 1-2 days
- **Prerequisite**: None

### 4. Add 2 Read Replicas
- **Impact**: Offload 70% of reads from primary, reduce DB CPU from 71% to ~30%
- **Cost**: 2 replicas * 8 vCPU * $0.10/hr * 730 hrs = $1,168/month
- **Lead time**: 2-3 days
- **Prerequisite**: Application changes to route reads

### 5. Add Redis Cluster
- **Impact**: Cache 80% of database queries, reduce DB load by 5x
- **Cost**: 3 nodes * 4 vCPU * $0.10/hr * 730 hrs = $876/month
- **Lead time**: 2-3 days
- **Prerequisite**: Application changes for cache-aside pattern

### 6. Add CDN
- **Impact**: Serve 90% of static assets from edge, reduce origin load
- **Cost**: ~$200/month (CloudFront)
- **Lead time**: 1-2 days
- **Prerequisite**: DNS changes, cache header configuration

### 7. Add Rate Limiting
- **Impact**: Protect against abuse, limit per-client traffic
- **Cost**: Included in API Gateway / Nginx
- **Lead time**: 1 day
- **Prerequisite**: Nginx configuration

### Total Cost Estimate

| Item | Monthly Cost |
|------|-------------|
| Additional web pods | $2,920 |
| Additional nodes | $2,920 |
| PgBouncer | $110 |
| Read replicas | $1,168 |
| Redis cluster | $876 |
| CDN | $200 |
| Rate limiting | $0 |
| **Total** | **$8,194/month** |

---

## Part C: Pre-Launch Timeline

### Week 1 (T-21 days): Infrastructure Foundation
- [ ] Add 2 PostgreSQL read replicas
- [ ] Deploy PgBouncer in staging
- [ ] Deploy Redis cluster in staging
- [ ] Configure CDN for static assets
- [ ] Increase ASG max size to 50 nodes
- [ ] Update application to route reads to replicas
- [ ] Update application to use Redis cache-aside

### Week 2 (T-14 days): Load Testing and Validation
- [ ] Run 5x load test in staging (15,000 RPS)
- [ ] Run 10x load test in staging (30,000 RPS)
- [ ] Identify and fix bottlenecks found in testing
- [ ] Validate connection pooling under load
- [ ] Validate caching hit rates (>80%)
- [ ] Run chaos tests (kill a pod, kill a node)

### Week 3 (T-7 days): Final Preparation
- [ ] Final 10x load test with all optimizations
- [ ] Write pre-warming script
- [ ] Write runbook for launch day
- [ ] Write rollback plan
- [ ] Set up monitoring dashboards
- [ ] Configure alerting rules
- [ ] Brief the incident response team
- [ ] Deploy all changes to production

### Launch Day (T-0)
- [ ] T-60 min: Run pre-warming script
- [ ] T-30 min: Verify all pods are ready
- [ ] T-15 min: Verify CDN cache is warm
- [ ] T-5 min: Incident response team on standby
- [ ] T-0: Marketing blast goes out
- [ ] T+5 min: Check error rate, latency, queue depth
- [ ] T+30 min: Assess if scaling adjustments needed
- [ ] T+2 hrs: Begin scaling down if traffic stabilizes

---

## Part D: Capacity Plan Document

### 1. Executive Summary

The product launch on [date] is expected to increase traffic from 3,000 RPS to 30,000 RPS (10x). Current infrastructure supports 6,000 RPS. We need to scale infrastructure 5x at an estimated cost of $8,194/month. The launch can proceed safely with the recommended changes.

### 2. Current State

- 5 web pods (2 vCPU, 4GB each)
- 1 PostgreSQL primary (8 vCPU, 32GB)
- 1 Redis instance (4 vCPU, 16GB)
- Sustainable capacity: 6,000 RPS
- Breaking point: 8,000 RPS
- Headroom: 50%

### 3. Requirements

- Launch day: 30,000 RPS (10x)
- Sustained: 15,000 RPS (5x)
- Error rate: < 1%
- P95 latency: < 2 seconds

### 4. Gap Analysis

| Metric | Current | Required | Gap |
|--------|---------|----------|-----|
| Sustainable RPS | 6,000 | 30,000 | 24,000 (5x) |
| Web pods | 5 | 25 | 20 |
| DB connections | 200 (max) | 10,000+ (needs pooling) | PgBouncer needed |
| Read capacity | Primary only | Primary + 2 replicas | 2 replicas needed |
| Cache hit rate | 0% | 80%+ | Redis cluster needed |

### 5. Recommendations
See Part B above.

### 6. Timeline
See Part C above.

### 7. Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Traffic exceeds 30,000 RPS | Medium | High | ASG max set to 50 nodes; rate limiting at edge |
| Database primary failure | Low | Critical | Read replicas can serve reads; promote replica |
| CDN failure | Low | Medium | Origin can serve directly; monitor CDN health |
| Deployment breaks under load | Medium | High | Canary deployment; instant rollback capability |
| Connection pool exhaustion | Low | High | PgBouncer with reserve pool; monitoring alerts |

### 8. Rollback Plan

If launch traffic is lower than expected:
- T+2 hrs: Scale web pods from 25 to 15
- T+4 hrs: Scale web pods from 15 to 10
- T+24 hrs: Scale ASG min size back to 5
- T+1 week: Evaluate if read replicas and Redis cluster are still needed
- T+1 month: Scale down or remove unused resources

---

## Part E: Monitoring and Alerting

### Prometheus Alerting Rules

```yaml
groups:
  - name: launch-day-alerts
    rules:
      - alert: HighErrorRate
        expr: sum(rate(http_requests_total{status=~"5.."}[1m])) / sum(rate(http_requests_total[1m])) > 0.01
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Error rate exceeds 1%"
          runbook: "https://wiki.example.com/runbooks/high-error-rate"

      - alert: HighLatency
        expr: histogram_quantile(0.95, sum(rate(http_request_duration_seconds_bucket[5m])) by (le)) > 1
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "P95 latency exceeds 1 second"

      - alert: HighCPU
        expr: avg(rate(container_cpu_usage_seconds_total{container!=""}[5m])) > 0.8
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "CPU utilization exceeds 80%"

      - alert: DatabaseConnectionsHigh
        expr: pg_stat_activity_count / pg_settings_max_connections > 0.7
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Database connections exceed 70%"

      - alert: QueueDepthHigh
        expr: rabbitmq_queue_messages > 10000
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Queue depth exceeds 10,000"
```

### Alert Response Actions

| Alert | Severity | Response |
|-------|----------|----------|
| HighErrorRate | Critical | Page on-call immediately. Check DB connections, pod health, queue depth. |
| HighLatency | Warning | Check database query times. Check if caching is working. |
| HighCPU | Warning | Scale web pods if below 25. Check for runaway queries. |
| DatabaseConnectionsHigh | Critical | Check PgBouncer health. Check for connection leaks. |
| QueueDepthHigh | Warning | Scale workers. Check for processing errors. |

---

## Common Mistakes
1. **Planning for average traffic instead of peak**: Launch traffic is a spike, not an average. Plan for the peak (30,000 RPS), not the sustained (15,000 RPS).
2. **Not load testing with the new architecture**: Adding read replicas and caching changes the system behavior. Re-test after every change.
3. **Forgetting to scale down**: After the launch window, scale down to avoid paying for unused capacity.
4. **No rollback plan**: If something goes wrong, you need a plan to revert quickly. Document rollback steps before launch.
5. **Alerting on the wrong thresholds**: Launch day will have higher traffic than normal. Adjust alert thresholds to avoid alert fatigue.

## Relevant README Sections
- [Integrated Capacity Planning Pipeline](../README.md#integrated-capacity-planning-pipeline)
- [Automated Scaling Policies](../README.md#automated-scaling-policies-based-on-capacity-data)
- [Hands-On Lab](../README.md#hands-on-lab-capacity-planning-for-a-product-launch)
