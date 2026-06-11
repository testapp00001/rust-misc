# Solution 04: Cost-Optimal Scaling Analysis

## Part A: Calculate Monthly Cost for Each Option

### Option A: Vertical (always-on)

Single 4xlarge instance running 24/7.

```
Cost/hour:  $0.92
Hours/month: 730
Monthly cost: $0.92 x 730 = $671.60
```

**Monthly cost: $671.60**

This is simple but wasteful -- you pay for 1,500 req/s capacity even during the 03:00-04:00 trough when you only need 50 req/s.

---

### Option B: Horizontal (auto-scaled)

Each xlarge instance handles 300 req/s. Scale instances based on traffic.

**Traffic tier classification:**

| Hour | Traffic (req/s) | Tier | Instances needed |
|------|-----------------|------|-----------------|
| 00 | 100 | 1 | 1 |
| 01 | 80 | 1 | 1 |
| 02 | 60 | 1 | 1 |
| 03 | 50 | 1 | 1 |
| 04 | 50 | 1 | 1 |
| 05 | 80 | 1 | 1 |
| 06 | 200 | 1 | 1 |
| 07 | 400 | 2 | 2 |
| 08 | 700 | 3 | 3 |
| 09 | 1000 | 4 | 4 |
| 10 | 1200 | 4 | 4 |
| 11 | 1200 | 4 | 4 |
| 12 | 1100 | 4 | 4 |
| 13 | 1000 | 4 | 4 |
| 14 | 1100 | 4 | 4 |
| 15 | 1200 | 4 | 4 |
| 16 | 900 | 3 | 3 |
| 17 | 600 | 2 | 2 |
| 18 | 400 | 2 | 2 |
| 19 | 300 | 1 | 1 |
| 20 | 200 | 1 | 1 |
| 21 | 150 | 1 | 1 |
| 22 | 120 | 1 | 1 |
| 23 | 100 | 1 | 1 |

**Hours per tier:**
- Tier 1 (1 instance): 12 hours (00-06, 19-23)
- Tier 2 (2 instances): 3 hours (07, 17-18)
- Tier 3 (3 instances): 2 hours (08, 16)
- Tier 4 (4 instances): 7 hours (09-15)

**Daily cost calculation:**

```
Tier 1: 12 hours x 1 instance  x $0.23 = $2.76
Tier 2:  3 hours x 2 instances x $0.23 = $1.38
Tier 3:  2 hours x 3 instances x $0.23 = $1.38
Tier 4:  7 hours x 4 instances x $0.23 = $6.44
                                          ------
Daily total:                               $11.96

Monthly cost: $11.96 x 30 = $358.80
```

**Monthly cost: $358.80**

---

### Option C: Horizontal with reserved baseline

Baseline: 2 x large instances running 24/7 (2 x 150 = 300 req/s capacity).

**Baseline cost:**
```
2 x large: 2 x $0.12 x 730 = $175.20/month
```

**Additional xlarge instances needed** (only when traffic exceeds 300 req/s baseline):

| Hour | Traffic | Baseline handles | Overflow | xlarge instances needed |
|------|---------|-----------------|----------|------------------------|
| 00-06 | 50-200 | Yes (300) | 0 | 0 |
| 07 | 400 | 300 | 100 | 1 |
| 08 | 700 | 300 | 400 | 2 |
| 09 | 1000 | 300 | 700 | 3 |
| 10-11 | 1200 | 300 | 900 | 3 |
| 12 | 1100 | 300 | 800 | 3 |
| 13 | 1000 | 300 | 700 | 3 |
| 14-15 | 1100-1200 | 300 | 800-900 | 3 |
| 16 | 900 | 300 | 600 | 2 |
| 17-18 | 400-600 | 300 | 100-300 | 1 |
| 19-23 | 100-300 | Yes (300) | 0 | 0 |

**Hours per overflow tier:**
- 0 xlarge: 13 hours (00-06, 19-23)
- 1 xlarge: 3 hours (07, 17-18)
- 2 xlarge: 2 hours (08, 16)
- 3 xlarge: 6 hours (09-15)

**Daily overflow cost:**
```
0 xlarge: 13 hours x 0 x $0.23 = $0.00
1 xlarge:  3 hours x 1 x $0.23 = $0.69
2 xlarge:  2 hours x 2 x $0.23 = $0.92
3 xlarge:  6 hours x 3 x $0.23 = $4.14
                                   -----
Daily overflow total:              $5.75

Monthly overflow: $5.75 x 30 = $172.50
```

**Monthly cost: $175.20 (baseline) + $172.50 (overflow) = $347.70**

---

### Cost Comparison Summary

| Option | Monthly Cost | Savings vs Option A |
|--------|-------------|---------------------|
| A: Vertical (always-on) | $671.60 | -- |
| B: Horizontal (auto-scaled) | $358.80 | 47% cheaper |
| C: Horizontal with baseline | $347.70 | 48% cheaper |

**Option C is the cheapest.** It combines the always-on reliability of a baseline (no cold-start lag for the first 300 req/s) with the cost efficiency of auto-scaling for peak traffic.

---

## Part B: Account for Scaling Lag

### Transition 1: 06:00 -- Traffic jumps from 80 to 200 req/s

**Current state (Option B):** 1 instance running (serving 80 req/s).

**New demand:** 200 req/s.

**Instances needed:** 1 (200 < 300, still within single instance capacity).

**Scaling lag risk:** None. The single instance can handle 200 req/s. No scaling event needed.

---

### Transition 2: 08:00 -- Traffic jumps from 400 to 700 req/s

**Current state (Option B):** 2 instances running (serving 400 req/s, each at ~67% capacity).

**New demand:** 700 req/s.

**Instances needed:** 3 (700 / 300 = 2.33, round up to 3).

**During the 3-minute scaling lag:**

```
Available capacity: 2 instances x 300 req/s = 600 req/s
Incoming demand:    700 req/s
Overload:           700 - 600 = 100 req/s excess

Overload ratio: 700 / 600 = 1.17 (17% over capacity)
Duration: 3 minutes
```

**Impact:**
- For 3 minutes, the 2 existing instances are 17% over capacity.
- Response latency increases because requests queue up.
- If each instance has a request queue of 50, the system can buffer ~100 excess requests.
- At 100 excess req/s for 180 seconds, that is 18,000 queued requests.
- Some requests will time out (likely 503 errors) if the queue fills up.

**Mitigation options:**
1. **Pre-scaling:** Scale to 3 instances at 07:57 (3 minutes before the spike).
2. **Warm pool:** Keep a third instance running but not receiving traffic (adds cost).
3. **Request queuing:** Use a message queue to buffer excess requests and process them when capacity is available.

**Option C comparison:** The baseline (300 req/s) absorbs the first 300 req/s. The 1 xlarge overflow instance (from 07:00) handles the next 300 req/s. Total capacity: 600 req/s. Still 100 req/s short, but the baseline eliminates the cold-start risk for the base capacity.

---

## Part C: Propose a Hybrid Strategy

### Hybrid Strategy: Scheduled + Reactive Scaling

```
┌─────────────────────────────────────────────────────────────────┐
│                    HYBRID SCALING STRATEGY                       │
├──────────┬──────────────┬──────────────┬────────────────────────┤
│ Time     │ Baseline     │ Scheduled    │ Reactive (auto)        │
│ Block    │ (always-on)  │ (pre-scaled) │ (on-demand)            │
├──────────┼──────────────┼──────────────┼────────────────────────┤
│ 00-05    │ 1 x large    │ --           │ --                     │
│ 06       │ 1 x large    │ +1 xlarge    │ --                     │
│ 07       │ 1 x large    │ +1 xlarge    │ --                     │
│ 08-15    │ 1 x large    │ +2 xlarge    │ +1 xlarge if CPU > 70% │
│ 16       │ 1 x large    │ +1 xlarge    │ --                     │
│ 17-18    │ 1 x large    │ +1 xlarge    │ --                     │
│ 19-23    │ 1 x large    │ --           │ --                     │
└──────────┴──────────────┴──────────────┴────────────────────────┘
```

### Detailed rules:

**Rule 1 -- Baseline (always-on):**
- 1 x large instance running 24/7.
- Capacity: 150 req/s.
- Cost: $0.12 x 730 = $87.60/month.

**Rule 2 -- Scheduled scaling (based on known traffic pattern):**
- 06:00: Start 1 xlarge instance (pre-warmed before business hours).
- 08:00: Start 1 additional xlarge instance (total 2 xlarge scheduled).
- 16:00: Stop 1 xlarge instance (traffic declining).
- 18:00: Stop the remaining xlarge instance (evening trough).

**Rule 3 -- Reactive auto-scaling (for unexpected spikes):**
- Metric: CPU utilization across all instances.
- Scale out: Add 1 xlarge instance if average CPU > 70% for 60 seconds.
- Scale in: Remove 1 xlarge instance if average CPU < 30% for 300 seconds.
- Max reactive instances: 2 (safety cap to prevent runaway costs).
- Cooldown: 120 seconds.

### Cost calculation:

```
Baseline (1 x large, 24/7):
  $0.12 x 730 = $87.60

Scheduled xlarge instances:
  06:00-08:00: 1 instance x 2 hours = 2 instance-hours
  08:00-16:00: 2 instances x 8 hours = 16 instance-hours
  16:00-18:00: 1 instance x 2 hours = 2 instance-hours
  Daily scheduled: 20 instance-hours
  Monthly: 20 x 30 = 600 instance-hours
  Cost: 600 x $0.23 = $138.00

Reactive (assume 1 extra instance for 1 hour/day on average):
  Monthly: 1 x 30 = 30 instance-hours
  Cost: 30 x $0.23 = $6.90

Total monthly cost: $87.60 + $138.00 + $6.90 = $232.50
```

### Comparison:

| Strategy | Monthly Cost | Cold Start Risk | Complexity |
|----------|-------------|-----------------|------------|
| Option A (vertical) | $671.60 | None | Low |
| Option B (reactive) | $358.80 | High | Medium |
| Option C (baseline + reactive) | $347.70 | Low | Medium |
| **Hybrid (scheduled + reactive)** | **$232.50** | **Very low** | **Medium-High** |

The hybrid strategy is **65% cheaper than vertical scaling** and **35% cheaper than pure reactive auto-scaling**, with virtually no cold-start risk because instances are pre-warmed before known traffic ramps.

### Why this works:

1. **Scheduled scaling eliminates cold-start lag** for predictable traffic patterns. You know when business hours start -- scale up 3 minutes before.
2. **Reactive scaling handles unexpected spikes** (viral events, bot traffic) that deviate from the normal pattern.
3. **The small baseline** ensures the system is always responsive, even during the trough. Starting from zero instances would mean cold-start lag on every request.
4. **The cost is low** because you only pay for peak capacity during business hours, not 24/7.

---

## Common Mistakes

1. **Using always-on vertical scaling for predictable traffic.** If you know the traffic pattern, scheduled horizontal scaling is dramatically cheaper. Paying for a large instance 24/7 when you only need it 10 hours a day wastes 58% of the budget.

2. **Ignoring cold-start time in reactive auto-scaling.** A 3-minute cold start during a sudden spike means 3 minutes of degraded performance. For predictable spikes, use scheduled scaling. For unpredictable spikes, keep a warm pool.

3. **Scaling to exactly the required capacity with no headroom.** If traffic is 1,200 req/s and capacity is 1,200 req/s, any small spike causes overload. Always maintain 15-20% headroom.

4. **Not accounting for instance granularity.** You cannot buy 2.33 instances. Round up. If you need 700 req/s and each instance does 300, you need 3 instances (900 req/s), not 2.33.

5. **Setting reactive scaling thresholds too low.** If you scale out at 50% CPU, you will scale during normal operation and waste money. 70% is a better threshold -- it means the instance is busy but not saturated.

---

## Relevant README Sections

- [Cost Comparison](../README.md#cost-comparison)
- [Scaling Decision Framework](../README.md#scaling-decision-framework)
- [The Problem](../README.md#1-the-problem)
