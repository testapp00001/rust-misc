# Exercise 01: Pre-Warming vs Reactive Scaling

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

Understand why reactive auto-scaling alone cannot handle traffic surges, and identify the scenarios where pre-warming is necessary versus where queue-based buffering is sufficient.

## Scenario

Your e-commerce platform runs a flash sale every Friday at noon. Normal traffic is 2,000 RPS. At noon on Friday, traffic jumps to 40,000 RPS within 30 seconds. Your HPA is configured with CPU-based scaling, and your Cluster Autoscaler provisions new nodes in 4 minutes.

Last Friday, this happened:

```
12:00:00 - Traffic jumps to 40,000 RPS
12:00:15 - HPA detects high CPU, begins creating pods
12:00:30 - New pods enter Pending state (no node capacity)
12:00:45 - Cluster Autoscaler requests new nodes from AWS
12:01:00 - API response time climbs to 8 seconds
12:02:00 - Database connection pool exhausted
12:03:00 - Error rate hits 40%, users see 503 errors
12:04:30 - New nodes join cluster, pods start scheduling
12:05:00 - Pods begin accepting traffic
12:06:00 - System stabilizes, error rate drops to 2%

Total outage window: ~5 minutes with 40% errors
```

## Tasks

### Part A: Analyzing the Failure Window

The 5-minute outage window exists because of a gap between when traffic arrives and when infrastructure is ready. Identify and explain each delay in the chain:

1. HPA reaction delay (why 15 seconds, not instant?)
2. Pod scheduling delay (why do pods go Pending?)
3. Node provisioning delay (why 4+ minutes?)
4. Application startup delay (what happens after pods are scheduled?)
5. Connection pool exhaustion (why does this happen before new pods are ready?)

<details>
<summary>Hint 1</summary>

HPA checks metrics every 15 seconds by default (controlled by `--horizontal-pod-autoscaler-sync-period`). Even after detecting the need to scale, it applies a stabilization window. Pods go Pending because the existing nodes do not have enough CPU/memory requests available, even if actual utilization is low.

</details>

<details>
<summary>Hint 2</summary>

On AWS EKS, node provisioning involves: API call to EC2 -> instance launch -> OS boot -> kubelet start -> node registration -> scheduler recognizes node. This chain takes 2-5 minutes depending on instance type and AMI.

</details>

### Part B: Pre-Warming vs Queue-Based Buffering

For each scenario below, choose whether **pre-warming**, **queue-based buffering**, **both**, or **neither** is the appropriate strategy. Explain your reasoning.

| Scenario | Strategy | Reasoning |
|----------|----------|-----------|
| Flash sale with known start time | ? | ? |
| Viral social media post driving unexpected traffic | ? | ? |
| Batch job processing that runs every night at 2 AM | ? | ? |
| API endpoint that receives steady 10x traffic for 4 hours | ? | ? |
| Sudden DDoS attack | ? | ? |

<details>
<summary>Hint 3</summary>

Pre-warming requires a known start time -- you scale infrastructure before the event. Queue-based buffering works for unpredictable surges because it absorbs the burst. Rate limiting protects against abuse. Some scenarios benefit from combining strategies.

</details>

### Part C: Cost Analysis

Your infrastructure costs $0.10 per pod-hour. Normal traffic needs 10 pods. Flash sale traffic needs 100 pods for 1 hour, then returns to normal.

Calculate the monthly cost difference between:
1. Running 100 pods 24/7 (over-provisioned)
2. Pre-warming to 100 pods for 1 hour per week (4 times per month)
3. Reactive scaling with HPA (100 pods for 1 hour + 5 minutes of degraded service)

<details>
<summary>Hint 4</summary>

Cost = pods * hours * rate. For option 3, include the cost of the 5-minute outage in terms of lost revenue or SLA impact, not just infrastructure cost.

</details>

## Success Criteria

- [ ] You can explain each delay in the reactive scaling chain (HPA -> Pending -> Cluster Autoscaler -> node -> pod start)
- [ ] You can correctly categorize scenarios as pre-warming, queue-based, or combined
- [ ] You understand that pre-warming solves the "known start time" problem while queues solve the "unpredictable burst" problem
- [ ] You can calculate the cost trade-off between over-provisioning and on-demand scaling
- [ ] You understand that the 5-minute gap is the fundamental problem that surge-handling strategies address

## What You Should Understand After This Exercise

Reactive auto-scaling has inherent latency -- the chain from metric detection to new pods serving traffic takes minutes, not seconds. Traffic surges are measured in seconds. Pre-warming closes the gap for known events, and queue-based architectures absorb the burst for unpredictable ones. The right strategy depends on whether you know when the surge will arrive.
