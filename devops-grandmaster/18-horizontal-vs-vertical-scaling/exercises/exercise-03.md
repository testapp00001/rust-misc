# Exercise 03: Multi-Tier Scaling Strategy

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective

Design a complete scaling strategy for a multi-tier application from scratch, making deliberate decisions about which tiers scale horizontally, which scale vertically, and what the scaling triggers should be.

## Scenario

You are designing the scaling strategy for a social media platform with the following tiers:

```
Tier 1: API Gateway (auth, rate limiting, routing)
Tier 2: User Service (profile CRUD, follower graphs)
Tier 3: Feed Service (timeline generation, ranking)
Tier 4: Media Service (image/video upload, transcoding)
Tier 5: Notification Service (push, email, SMS)
Tier 6: PostgreSQL (user data, posts, relationships)
Tier 7: Redis (feed cache, session store, rate limit counters)
Tier 8: Object Storage (images, videos)
```

**Traffic profile:**
- 10,000 concurrent users at peak
- 80% read, 20% write
- Feed generation is CPU-intensive (ranking algorithm)
- Media transcoding is CPU and memory-intensive
- Notifications are I/O-bound (calling external APIs)

**Constraints:**
- Budget: $5,000/month on infrastructure
- Must handle a 3x traffic spike during viral events
- Cannot have more than 2 minutes of downtime per month
- Team of 3 engineers (limited operational capacity)

## Tasks

### Part A: Scaling Decision Matrix

Complete the following table for each tier. Be specific about *why* you chose each approach.

| Tier | Horizontal or Vertical? | Stateful or Stateless? | Scaling Trigger | Min Instances | Max Instances |
|------|------------------------|----------------------|-----------------|---------------|---------------|
| API Gateway | | | | | |
| User Service | | | | | |
| Feed Service | | | | | |
| Media Service | | | | | |
| Notification Service | | | | | |
| PostgreSQL | | | | | |
| Redis | | | | | |
| Object Storage | | | | | |

<details>
<summary>Hint 1</summary>

Think about which tiers are CPU-bound (parallelizable), I/O-bound (need connection pooling), or stateful (hard to distribute).

</details>

<details>
<summary>Hint 2</summary>

Object storage (S3) is managed -- you do not scale it yourself. PostgreSQL is best scaled vertically first, with read replicas for the read-heavy workload.

</details>

### Part B: Capacity Planning

Using these instance sizes, calculate the baseline infrastructure cost:

| Instance Type | vCPUs | RAM | Cost/month | Good for |
|---------------|-------|-----|------------|----------|
| t3.medium | 2 | 4 GB | $30 | Light services |
| t3.large | 2 | 8 GB | $60 | API servers |
| t3.xlarge | 4 | 16 GB | $120 | Compute services |
| r6g.large | 2 | 16 GB | $90 | Memory-heavy |
| db.r6g.large | 2 | 16 GB | $180 | Database |
| cache.r6g.large | 2 | 13 GB | $150 | Redis |

Calculate:
1. Baseline cost (minimum instances for each tier during normal traffic).
2. Peak cost (maximum instances needed for a 3x spike).
3. Does the peak cost fit within the $5,000 budget?

<details>
<summary>Hint</summary>

Start with the minimum viable instance count for each tier (usually 2 for redundancy), then calculate what 3x traffic requires for the horizontally-scaled tiers.

</details>

### Part C: Define the Auto-Scaling Policy

Write the auto-scaling rules for the two tiers that need it most (Feed Service and Media Service). For each, specify:

1. The metric to monitor (CPU, memory, request queue depth, etc.)
2. The scale-out threshold (when to add instances)
3. The scale-in threshold (when to remove instances)
4. The cooldown period (how long to wait between scaling events)

<details>
<summary>Hint</summary>

CPU utilization is the most common scaling metric. Scale out at 70% CPU, scale in at 30% CPU. The cooldown period should be long enough to let new instances warm up (typically 60-300 seconds).

</details>

## Success Criteria

- [ ] You made a deliberate horizontal/vertical decision for every tier with justification.
- [ ] Your capacity plan fits within the $5,000/month budget for both baseline and peak.
- [ ] You identified the two tiers that benefit most from auto-scaling and wrote specific policies.
- [ ] You considered operational complexity (team of 3) in your decisions.
- [ ] You accounted for the 3x traffic spike in your max instance counts.

## What You Should Understand After This Exercise

A scaling strategy is not one-size-fits-all. Each tier in a multi-tier application has different bottlenecks, state requirements, and scaling characteristics. Stateless, CPU-bound tiers (API servers, feed generators) scale horizontally. Stateful services (databases, caches) scale vertically first. Managed services (object storage) scale automatically. The art is balancing capacity, cost, reliability, and operational complexity -- and making these decisions per-tier, not per-system.
