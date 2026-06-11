# Exercise 03: Bottleneck Analysis

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective
Analyze load test results to identify the primary bottleneck in a system, determine which component is limiting capacity, and recommend specific remediation actions.

## Scenario
You ran a 10x load test (5,000 RPS) on your production-like environment. The system handled the load, but performance degraded significantly. The following metrics were collected during the test:

```
Component               Utilization    Status
-----------------------------------------------
CPU (pods)              78%            High
Memory (pods)           62%            Moderate
Database connections    85%            Critical
Database CPU            71%            Moderate
Disk I/O                45%            Low
Network bandwidth       38%            Low
Load balancer           55%            Moderate
Redis memory            28%            Low
```

Error rate at 10x: 3.2%
P95 latency at 10x: 1,800ms (target: <2,000ms)
P99 latency at 10x: 4,500ms (target: <5,000ms)

## Tasks

### Part A: Identify the Primary Bottleneck
Analyze the metrics above and identify the primary bottleneck. Explain:
1. Which component is most likely causing the performance degradation?
2. Why is this component the bottleneck and not the others?
3. What would happen if you added more application pods without fixing this bottleneck?

<details>
<summary>Hint</summary>
Look for the component with the highest utilization relative to its capacity. Database connections at 85% is the highest. When DB connections are exhausted, new requests queue up, increasing latency. Adding more app pods would make this worse (more connection contention).
</details>

### Part B: Explain the Cascading Effects
The database connection pool is at 85%. Explain how this causes:
1. Increased P99 latency (but not P50)
2. 3.2% error rate
3. Why P95 is acceptable but P99 is not

<details>
<summary>Hint</summary>
When connection pools are nearly full, most requests get a connection quickly (P50 is fine). But as the pool fills, some requests must wait for a connection to become available. The wait time grows exponentially as utilization approaches 100%. This affects the tail (P99) much more than the median.
</details>

### Part C: Recommend Remediation Actions
Write a prioritized list of actions to eliminate the database connection bottleneck. For each action, explain:
- What to do
- Expected impact
- Implementation complexity
- Cost

<details>
<summary>Hint</summary>
Consider: (1) Connection pooling (PgBouncer) to multiplex connections, (2) Read replicas to offload read queries, (3) Query optimization to reduce connection hold time, (4) Caching to reduce database queries, (5) Connection pool size increase (short-term fix).
</details>

### Part D: Calculate Capacity Headroom
Using the metrics above, calculate:
1. How much headroom exists for each component (percentage of unused capacity)
2. Which component will become the next bottleneck after you fix the database
3. How much additional traffic the system can handle after fixing the database bottleneck

<details>
<summary>Hint</summary>
Headroom = 100% - utilization. After fixing the database bottleneck (from 85% to, say, 40%), the next highest utilization is CPU at 78%. CPU will become the next bottleneck. The CPU headroom allows approximately 1.28x current traffic (100/78 = 1.28).
</details>

## Success Criteria
- [ ] You correctly identify the database connection pool as the primary bottleneck.
- [ ] You can explain why high P99 but acceptable P95 indicates connection pool contention.
- [ ] You provide at least three remediation actions with expected impact and complexity.
- [ ] You calculate headroom for each component and identify the next bottleneck.
- [ ] You can explain why fixing the bottleneck does not guarantee 10x more capacity.

## What You Should Understand After This Exercise
The bottleneck is the component with the highest utilization relative to its capacity. Fixing a bottleneck does not give you unlimited capacity -- it shifts the bottleneck to the next most constrained component. Capacity planning is an iterative process: find the bottleneck, fix it, find the next one. Always fix the bottleneck first; adding resources to non-bottleneck components is wasted money.
