# Solution 03: Multi-Tier Scaling Strategy

## Part A: Scaling Decision Matrix

| Tier | Horizontal or Vertical? | Stateful or Stateless? | Scaling Trigger | Min Instances | Max Instances |
|------|------------------------|----------------------|-----------------|---------------|---------------|
| API Gateway | Horizontal | Stateless | CPU > 70% | 2 | 6 |
| User Service | Horizontal | Stateless | CPU > 70% | 2 | 6 |
| Feed Service | Horizontal | Stateless | CPU > 60% or queue depth > 100 | 2 | 10 |
| Media Service | Horizontal | Stateless | CPU > 60% or queue depth > 50 | 2 | 8 |
| Notification Service | Horizontal | Stateless | Queue depth > 200 | 2 | 6 |
| PostgreSQL | Vertical | Stateful | CPU > 80% or connections > 80% | 1 primary + 2 read replicas | 1 primary + 4 read replicas |
| Redis | Vertical | Stateful | Memory > 75% | 1 | 1 (cluster if needed) |
| Object Storage | Managed | Managed | N/A (auto) | N/A | N/A |

### Justification for each tier:

**API Gateway:** Stateless routing and auth logic. Each request is independent. CPU-bound (TLS termination, auth token validation). Scale horizontally to handle more concurrent connections. Two instances minimum for redundancy.

**User Service:** Stateless CRUD operations. Database-backed (no in-memory state). CPU and I/O bound. Horizontal scaling is straightforward because all state lives in PostgreSQL.

**Feed Service:** This is the most CPU-intensive tier. The ranking algorithm processes thousands of posts to generate a personalized feed. This is embarrassingly parallel -- each user's feed can be computed independently. Lower CPU threshold (60%) because feed generation is expensive and you want headroom. Queue depth is a secondary trigger because feed requests can queue up when the ranking algorithm is slow.

**Media Service:** Video transcoding is CPU and memory-intensive. Each transcoding job is independent (embarrassingly parallel). Queue depth is the primary trigger because jobs queue up waiting for available workers. Two instances minimum so one can handle uploads while the other transcodes.

**Notification Service:** I/O-bound (calling external push, email, and SMS APIs). Throughput is limited by external API rate limits, not CPU. Queue depth is the right metric -- if notifications are queuing up, you need more workers to drain the queue.

**PostgreSQL:** The database is the hardest component to scale horizontally. Writes must go to a single primary. Vertical scaling (more RAM for buffer cache, faster storage) is the first step. Read replicas handle the 80% read traffic. The primary handles the 20% write traffic. Do not shard until vertical limits are reached -- sharding is operationally expensive.

**Redis:** Single-threaded and extremely fast. A single instance can handle hundreds of thousands of ops/sec. Vertical scaling (more RAM) is sufficient. Redis Cluster exists but adds complexity that is rarely needed at this scale.

**Object Storage:** S3/GCS are managed services. You do not scale them. They scale automatically.

---

## Part B: Capacity Planning

### Baseline (normal traffic, minimum instances for redundancy)

| Tier | Instances | Instance Type | Cost/instance/month | Total/month |
|------|-----------|---------------|---------------------|-------------|
| API Gateway | 2 | t3.large | $60 | $120 |
| User Service | 2 | t3.large | $60 | $120 |
| Feed Service | 2 | t3.xlarge | $120 | $240 |
| Media Service | 2 | t3.xlarge | $120 | $240 |
| Notification Service | 2 | t3.medium | $30 | $60 |
| PostgreSQL primary | 1 | db.r6g.large | $180 | $180 |
| PostgreSQL read replicas | 2 | db.r6g.large | $180 | $360 |
| Redis | 1 | cache.r6g.large | $150 | $150 |
| Load Balancer | 1 | ALB | $20 + traffic | $25 |
| **Total baseline** | | | | **$1,495** |

### Peak (3x traffic spike)

During a 3x spike, the horizontally scaled tiers need to scale up. The database and Redis stay the same (they handle spikes through connection pooling and buffering).

| Tier | Peak Instances | Additional Cost |
|------|---------------|-----------------|
| API Gateway | 6 (from 2) | +4 x $60 = +$240 |
| User Service | 6 (from 2) | +4 x $60 = +$240 |
| Feed Service | 10 (from 2) | +8 x $120 = +$960 |
| Media Service | 8 (from 2) | +6 x $120 = +$720 |
| Notification Service | 6 (from 2) | +4 x $30 = +$120 |
| PostgreSQL | Same (vertical handles spikes) | $0 |
| Redis | Same | $0 |
| **Total peak** | | **$1,495 + $2,280 = $3,775** |

### Budget check

- Baseline: $1,495/month -- well within budget.
- Peak: $3,775/month -- within the $5,000 budget with $1,225 headroom.

**Important:** The peak cost only applies during the spike. If the spike lasts a few hours, the actual monthly cost is closer to baseline + proportional spike cost. Assuming the 3x spike lasts 2 hours per day:

Additional spike cost = $2,280 x (2/24) x 30 = $5,700 x 0.083 = $474/month

Realistic monthly total: $1,495 + $474 = **$1,969/month**. Well within budget.

---

## Part C: Define the Auto-Scaling Policy

### Feed Service Auto-Scaling Policy

```yaml
# Conceptual auto-scaling policy (adaptable to Kubernetes HPA or cloud ASG)

scaling_policy:
  service: feed-service
  min_replicas: 2
  max_replicas: 10

  scale_out:
    metrics:
      - name: cpu_utilization
        threshold: 60%
        window: 60s       # Average over 60 seconds
      - name: request_queue_depth
        threshold: 100
        window: 30s
    action: add 1 replica
    cooldown: 120s         # Wait 2 min before scaling again

  scale_in:
    metrics:
      - name: cpu_utilization
        threshold: 30%
        window: 300s       # Average over 5 minutes (be conservative)
      - name: request_queue_depth
        threshold: 20
        window: 120s
    action: remove 1 replica
    cooldown: 300s         # Wait 5 min before scaling in
```

**Why these values:**
- **Scale out at 60% CPU** (not 70%): Feed generation is CPU-intensive. A single feed request can spike CPU from 40% to 90% instantly. Scaling at 60% gives headroom before the instance is saturated.
- **Scale out at queue depth 100**: If requests are queuing, the service is already under-provisioned. Scale out immediately.
- **Scale in at 30% CPU** (not lower): Below 30% means the instance is mostly idle. Safe to remove.
- **Longer scale-in cooldown (300s)**: Removing instances too aggressively can cause oscillation. Wait longer before scaling in.
- **Shorter scale-out cooldown (120s)**: Scaling out should be responsive to handle traffic spikes quickly.

### Media Service Auto-Scaling Policy

```yaml
scaling_policy:
  service: media-service
  min_replicas: 2
  max_replicas: 8

  scale_out:
    metrics:
      - name: cpu_utilization
        threshold: 60%
        window: 60s
      - name: transcoding_queue_depth
        threshold: 50
        window: 30s
    action: add 1 replica
    cooldown: 180s         # Longer cooldown -- transcoding jobs take time

  scale_in:
    metrics:
      - name: cpu_utilization
        threshold: 25%
        window: 600s       # 10 minutes -- let current jobs finish
      - name: transcoding_queue_depth
        threshold: 5
        window: 300s
    action: remove 1 replica
    cooldown: 600s         # 10 minutes -- do not kill instances mid-transcode
```

**Why these values differ from Feed Service:**
- **Longer cooldowns**: Transcoding jobs take minutes, not seconds. Scaling in too quickly would kill instances mid-job, wasting the work already done.
- **10-minute scale-in window**: Wait until the queue is nearly empty and CPU has been low for a sustained period before removing capacity.
- **Queue depth as primary metric**: CPU can be low while a queue builds up (waiting for I/O). Queue depth is a more direct measure of whether the service is keeping up with demand.

---

## Common Mistakes

1. **Applying the same scaling policy to every tier.** Each tier has different characteristics. A CPU-intensive service (Feed) needs different thresholds than an I/O-bound service (Notification). One-size-fits-all policies lead to over-provisioning some tiers and under-provisioning others.

2. **Scaling the database horizontally as the first response to load.** The database should be vertically scaled first. Read replicas help with reads, but they add replication lag and complexity. A bigger instance with more RAM is simpler and often sufficient.

3. **Setting the scale-in threshold too aggressively.** If you scale in at 50% CPU, you risk oscillation: scale out when traffic rises, scale in when it drops slightly, scale out again. Use longer cooldown periods and lower thresholds for scale-in.

4. **Forgetting about the operational cost of running many services.** A team of 3 engineers cannot babysit 10 auto-scaled tiers. Managed services (RDS instead of self-hosted PostgreSQL, ElastiCache instead of self-hosted Redis) reduce operational burden.

5. **Not accounting for cold start time in capacity planning.** When a new instance starts, it needs time to warm up (load caches, establish connections). If your scale-out policy adds an instance but it takes 60 seconds to become ready, you need enough existing capacity to handle the spike during that 60 seconds.

---

## Relevant README Sections

- [Scaling Decision Framework](../README.md#scaling-decision-framework)
- [Cost Comparison](../README.md#cost-comparison)
- [Real-World Architecture](../README.md#real-world-architecture)
