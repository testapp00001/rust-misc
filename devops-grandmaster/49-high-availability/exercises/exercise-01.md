# Exercise 01: HA Pattern Selection

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

Understand how availability targets translate into real downtime budgets and learn to select the appropriate high availability pattern for different system requirements.

## Scenario

You are the architect for a company that operates five distinct systems. Each system has a different availability requirement based on its business impact. Your job is to analyze each system, calculate its allowed downtime, and recommend the correct HA pattern.

The five systems are:

| System | Description | Availability Target |
|--------|-------------|---------------------|
| **A** | Internal HR portal used by employees during business hours | 99.9% |
| **B** | Customer-facing e-commerce checkout service | 99.99% |
| **C** | Global payment processing gateway | 99.999% |
| **D** | Batch analytics pipeline that runs nightly | 99.0% |
| **E** | Real-time bidding engine for ad auctions | 99.99% |

## Tasks

### Part A: Calculate Allowed Downtime

For each system, calculate the maximum allowed downtime per year, per month (30 days), and per week. Fill in the table below.

| System | Availability | Downtime/Year | Downtime/Month | Downtime/Week |
|--------|--------------|---------------|----------------|---------------|
| A | 99.9% | | | |
| B | 99.99% | | | |
| C | 99.999% | | | |
| D | 99.0% | | | |
| E | 99.99% | | | |

### Part B: Map Systems to HA Patterns

For each system, select the most appropriate HA pattern from the options below. Justify your choice in one sentence.

**Available patterns:**
- **Active-Passive (Cold Standby):** Secondary sits offline; manual or automated failover takes minutes.
- **Active-Passive (Hot Standby):** Secondary is running and receives data replication; failover takes seconds.
- **Active-Active (Single-Region):** Multiple nodes serve traffic in the same region with load balancing.
- **Active-Active (Multi-Region):** Nodes in multiple geographic regions serve traffic independently.
- **No HA (Single Instance):** A single instance with best-effort recovery from backups.

Fill in the table below.

| System | Recommended Pattern | Justification |
|--------|---------------------|---------------|
| A | | |
| B | | |
| C | | |
| D | | |
| E | | |

### Part C: Identify Single Points of Failure

For each pattern you selected in Part B, identify at least two single points of failure (SPOFs) that would need to be addressed to actually achieve the target availability. Consider the full request path: DNS, load balancer, application server, database, network, and storage.

| System | SPOF 1 | SPOF 2 | Mitigation for Each |
|--------|--------|--------|---------------------|
| A | | | |
| B | | | |
| C | | | |
| D | | | |
| E | | | |

## Success Criteria

- [ ] All downtime calculations are mathematically correct
- [ ] Each system is mapped to a pattern that meets its availability target
- [ ] Justifications reference specific latency or throughput requirements
- [ ] At least two SPOFs are identified per system with concrete mitigations

## Hints

<details>
<summary>Hint 1: Downtime formula</summary>

To calculate allowed downtime:
- 99.9% = 1 - 0.999 = 0.001 of total time
- For a year: 365.25 days x 24 hours x 60 minutes x 0.001 = 525,960 x 0.001 = ~526 minutes/year
- Repeat the same calculation for month (30 x 24 x 60) and week (7 x 24 x 60)

</details>

<details>
<summary>Hint 2: Pattern selection heuristics</summary>

A useful rule of thumb:
- 99.0%-99.9%: Single instance with good backups may suffice
- 99.9%-99.99%: Active-passive with hot standby is typical
- 99.99%-99.999%: Active-active with multi-region is usually required
- Consider whether the system handles stateful or stateless workloads -- stateful systems are harder to make active-active

</details>

<details>
<summary>Hint 3: Common SPOFs</summary>

Think beyond the obvious. Common SPOFs include:
- Single DNS provider
- Single load balancer instance
- Shared storage with one controller
- Single network path between data centers
- A single etcd/consul cluster for leader election
- Certificate expiration on a single TLS termination point

</details>

## What You Should Understand After This Exercise

Availability is not just a number -- it is a contract that dictates your entire architecture. A system targeting 99.999% availability cannot tolerate a single-component failover that takes 30 seconds, because that alone would exhaust the entire yearly downtime budget. The HA pattern you choose must be driven by the math, not by preference or habit.
