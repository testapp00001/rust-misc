# Exercise 04: Cost-Optimal Scaling Analysis

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Analyze real-world traffic patterns and calculate the most cost-effective scaling strategy, comparing always-on vertical scaling against auto-scaled horizontal deployments across different time periods.

## Scenario

You run a B2B SaaS application with predictable but uneven traffic patterns. The application is a stateless API server backed by a PostgreSQL database. You must serve the following daily traffic profile:

```
Requests/second by hour (UTC):

 00:  100  ░░░
 01:   80  ░░░
 02:   60  ░░░
 03:   50  ░░░
 04:   50  ░░░
 05:   80  ░░░
 06:  200  ░░░░░
 07:  400  ░░░░░░░░░
 08:  700  ░░░░░░░░░░░░░░
 09: 1000  ░░░░░░░░░░░░░░░░░░░░
 10: 1200  ░░░░░░░░░░░░░░░░░░░░░░░░
 11: 1200  ░░░░░░░░░░░░░░░░░░░░░░░░
 12: 1100  ░░░░░░░░░░░░░░░░░░░░░░░
 13: 1000  ░░░░░░░░░░░░░░░░░░░░
 14: 1100  ░░░░░░░░░░░░░░░░░░░░░░░
 15: 1200  ░░░░░░░░░░░░░░░░░░░░░░░░
 16:  900  ░░░░░░░░░░░░░░░░░░░
 17:  600  ░░░░░░░░░░░░░░
 18:  400  ░░░░░░░░░
 19:  300  ░░░░░░
 20:  200  ░░░░░
 21:  150  ░░░
 22:  120  ░░░
 23:  100  ░░░
```

Peak: 1,200 req/s (10:00-11:00 and 15:00 UTC)
Minimum: 50 req/s (03:00-04:00 UTC)

## Instance Options

**Option A -- Vertical (always-on):**

| Instance | vCPUs | RAM | Cost/hour | Capacity (req/s) |
|----------|-------|-----|-----------|-----------------|
| 4xlarge | 16 | 64 GB | $0.92 | 1,500 |

One instance, always running. Can handle peak with headroom.

**Option B -- Horizontal (auto-scaled):**

| Instance | vCPUs | RAM | Cost/hour | Capacity (req/s) |
|----------|-------|-----|-----------|-----------------|
| xlarge | 4 | 16 GB | $0.23 | 300 |

Each instance handles 300 req/s. Scale from 1 to N instances based on load. Assume scaling takes 3 minutes (cold start).

**Option C -- Horizontal with reserved baseline:**

Run 2 small instances 24/7 (baseline), auto-scale with xlarge instances for peak.

| Instance | vCPUs | RAM | Cost/hour | Capacity (req/s) |
|----------|-------|-----|-----------|-----------------|
| large | 2 | 8 GB | $0.12 | 150 |
| xlarge | 4 | 16 GB | $0.23 | 300 |

## Tasks

### Part A: Calculate Monthly Cost for Each Option

Calculate the total monthly cost (730 hours) for each of the three options. Show your work for at least one option in detail.

For Option B, determine how many instances are needed during each traffic tier:
- Tier 1: 0-300 req/s (1 instance)
- Tier 2: 301-600 req/s (2 instances)
- Tier 3: 601-900 req/s (3 instances)
- Tier 4: 901-1200 req/s (4 instances)

Sum the hourly costs across all 24 hours, then multiply by 30 days.

<details>
<summary>Hint 1</summary>

For Option B, count how many hours fall into each traffic tier. For example, if 00:00-05:00 is Tier 1 (6 hours), that is 6 hours at 1 instance.

</details>

<details>
<summary>Hint 2</summary>

For Option C, the baseline (2 x large) runs 24/7. Calculate that cost separately, then add the auto-scaled xlarge instances only during hours where the baseline capacity is insufficient.

</details>

### Part B: Account for Scaling Lag

Option B has a 3-minute cold start. During a sudden traffic spike, new instances are not ready immediately. Analyze the 06:00-08:00 transition:

- At 06:00, traffic jumps from 80 to 200 req/s.
- At 08:00, traffic jumps from 400 to 700 req/s.

For each transition:
1. How many instances are currently running?
2. How many are needed for the new traffic level?
3. What happens during the 3-minute scaling lag?
4. What is the risk (dropped requests, degraded latency)?

<details>
<summary>Hint</summary>

During the lag, the existing instances absorb all traffic. If they are already at capacity, requests queue up or time out. Calculate the overload ratio: (incoming req/s) / (current capacity).

</details>

### Part C: Propose a Hybrid Strategy

Design a scaling strategy that is cheaper than Option A but handles spikes better than pure Option B. Consider:

1. Scheduled scaling (pre-scale before known peak hours).
2. A warm pool of instances that are started but not receiving traffic.
3. Over-provisioning during business hours, scaling down at night.

Write your strategy as a set of rules with specific instance counts per time block.

<details>
<summary>Hint</summary>

You know the traffic pattern in advance. Use scheduled scaling to add instances 10 minutes before the known traffic ramp-up. This eliminates the cold-start lag at a fraction of the cost of always-on vertical scaling.

</details>

## Success Criteria

- [ ] You calculated the monthly cost for all three options with correct math.
- [ ] You identified the scaling lag risk during traffic transitions.
- [ ] Your hybrid strategy is cheaper than Option A and more responsive than Option B.
- [ ] You showed specific instance counts per time block in your hybrid strategy.
- [ ] You can articulate the trade-off between cost and responsiveness.

## What You Should Understand After This Exercise

The cheapest scaling strategy depends on traffic patterns. For predictable traffic, scheduled scaling (pre-scaling before known peaks) offers the best cost-to-performance ratio. Always-on vertical scaling wastes money during off-peak hours. Pure reactive auto-scaling risks cold-start lag during sudden spikes. The optimal approach combines a small always-on baseline with scheduled and reactive scaling on top -- matching capacity to demand without over-provisioning.
