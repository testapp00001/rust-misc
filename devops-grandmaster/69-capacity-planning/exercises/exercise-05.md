# Exercise 05: Capacity Plan for Product Launch

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective
Create a complete capacity plan for a major product launch, combining baseline analysis, load testing, bottleneck identification, and growth forecasting into a single actionable document.

## Scenario
You are the lead SRE for a SaaS platform. The company is launching a major feature in 3 weeks. Marketing expects a 10x traffic spike on launch day (from 3,000 RPS to 30,000 RPS), settling to 5x sustained traffic (15,000 RPS) within a week.

Current infrastructure:
- 5 web pods (each: 2 vCPU, 4GB RAM)
- 1 PostgreSQL primary (8 vCPU, 32GB RAM)
- 1 Redis instance (4 vCPU, 16GB RAM)
- Load balancer (AWS ALB)
- No CDN, no read replicas, no connection pooler

Current capacity: 6,000 RPS (sustainable, determined by load testing)
Breaking point: 8,000 RPS (error rate exceeds 5%)

## Tasks

### Part A: Analyze Current State
Using the information above, determine:
1. Current headroom (percentage of capacity used at normal traffic)
2. Whether the system can handle the 10x launch spike
3. Whether the system can handle the 5x sustained traffic
4. What the gap is between current capacity and required capacity

<details>
<summary>Hint</summary>
Headroom = (1 - current_traffic / sustainable_capacity) * 100. If current traffic is 3,000 RPS and sustainable capacity is 6,000 RPS, headroom is 50%. The system cannot handle 30,000 RPS (needs 5x current capacity) or 15,000 RPS (needs 2.5x).
</details>

### Part B: Identify Required Changes
List all infrastructure changes needed to handle 30,000 RPS. For each change:
1. What to do
2. Expected impact on capacity
3. Cost estimate
4. Lead time (how long to implement)

<details>
<summary>Hint</summary>
Consider: (1) Scale web pods from 5 to 25 (5x), (2) Add PgBouncer for connection pooling, (3) Add 2 read replicas for PostgreSQL, (4) Add Redis cluster for caching, (5) Add CDN for static assets, (6) Add rate limiting at the edge.
</details>

### Part C: Create a Pre-Launch Timeline
Create a week-by-week timeline for the 3 weeks before launch:

**Week 1 (T-21 days)**: What must be done this week?
**Week 2 (T-14 days)**: What must be done this week?
**Week 3 (T-7 days)**: What must be done this week?
**Launch Day (T-0)**: What happens on launch day?

<details>
<summary>Hint</summary>
Week 1: Infrastructure changes (add read replicas, PgBouncer, CDN). Week 2: Load test at 5x and 10x, identify bottlenecks, fix issues. Week 3: Final load test, pre-warming script, runbook, rollback plan. Launch day: Pre-warming at T-60min, monitor dashboards, incident response team on standby.
</details>

### Part D: Write the Capacity Plan Document
Create a structured capacity plan document with the following sections:
1. Executive Summary (one paragraph: what, why, when, cost)
2. Current State (baseline metrics, current capacity)
3. Requirements (launch traffic, sustained traffic)
4. Gap Analysis (what is missing)
5. Recommendations (infrastructure changes with cost)
6. Timeline (week-by-week plan)
7. Risk Assessment (what could go wrong)
8. Rollback Plan (how to revert if launch traffic is lower than expected)

<details>
<summary>Hint</summary>
The executive summary should answer: "Can we handle the launch?" with a yes/no and the cost. The risk assessment should cover: traffic higher than expected, database failure, CDN failure, and deployment issues. The rollback plan should include scaling back down after the launch window.
</details>

### Part E: Define Monitoring and Alerting
Write alerting rules for launch day that trigger when:
1. Error rate exceeds 1%
2. P95 latency exceeds 1 second
3. CPU utilization exceeds 80%
4. Database connections exceed 70%
5. Queue depth exceeds 10,000

For each alert, define the severity and the response action.

<details>
<summary>Hint</summary>
Use Prometheus alerting rules syntax. Critical alerts (error rate, database connections) should page the on-call engineer immediately. Warning alerts (CPU, queue depth) should notify the team in Slack. Each alert should have a runbook link.
</details>

## Success Criteria
- [ ] Gap analysis correctly identifies that current capacity is insufficient for launch traffic.
- [ ] Infrastructure recommendations address all bottlenecks with cost estimates.
- [ ] The timeline is realistic with specific deliverables for each week.
- [ ] The capacity plan document covers all eight required sections.
- [ ] Monitoring and alerting rules cover all critical failure modes.

## What You Should Understand After This Exercise
A capacity plan is not just "add more servers." It is a systematic analysis of current capacity, required capacity, the gap between them, and a prioritized plan to close the gap. The plan must include cost estimates, timelines, risk assessments, and rollback plans. The goal is to make data-driven decisions, not guesses.
