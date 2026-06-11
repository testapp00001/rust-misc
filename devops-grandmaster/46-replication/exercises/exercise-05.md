# Exercise 05: Cross-Region Replication Architecture

**Type:** Integration | **Time:** 45 min | **Difficulty:** Hard

## Objective
Design a cross-region PostgreSQL replication architecture for a global SaaS application that balances latency, consistency, and regulatory compliance.

## Scenario
You are designing the database architecture for a global SaaS platform with the following requirements:

| Requirement | Detail |
|-------------|--------|
| Users | 5 million across US (60%), EU (25%), APAC (15%) |
| Read latency | < 50ms for 95th percentile |
| Write latency | < 200ms for 95th percentile |
| RPO | < 1 minute |
| Data residency | EU user data must stay in EU (GDPR) |
| Availability | 99.99% (52 minutes downtime/year) |
| Write volume | 10,000 writes/second globally |
| Read volume | 200,000 reads/second globally |

Current state: Single PostgreSQL primary in US-East, no replicas, all users connect to US-East.

## Tasks

### Part A: Design the Replication Topology
Design the cross-region topology. Include:
1. Number and location of database instances
2. Replication links (sync vs async) between instances
3. Client routing strategy
4. An ASCII diagram of the complete architecture

<details>
<summary>Hint</summary>
GDPR requires EU data to stay in EU, so you need a separate primary or partitioned instance in EU. US and APAC can share a primary with replicas. Consider: one primary per region vs. one global primary with regional read replicas.
</details>

### Part B: Configure Data Residency Rules
Design the data routing rules that enforce GDPR compliance:
1. Which tables are region-specific vs. global
2. How to route writes to the correct regional primary
3. How to handle cross-region queries (e.g., a US admin viewing EU customer data)
4. How to handle data that moves between regions (e.g., EU user traveling to US)

<details>
<summary>Hint</summary>
Partition user data by region (e.g., `user_region` column). Route writes based on the user's region. For cross-region access, use a federated query or read-only view with appropriate access controls. Data residency means data at rest, not data in transit -- the user's data stays in their home region.
</details>

### Part C: Handle Failover Scenarios
For each scenario, describe the failover procedure:

1. US-East primary fails completely
2. Network partition between US and EU regions
3. EU primary fails (cannot be recovered for 4 hours)
4. APAC replica falls 5 minutes behind during peak traffic

<details>
<summary>Hint</summary>
For scenario 1, promote a US replica or fail over to another region. For scenario 2, each region continues independently -- conflict resolution is needed when the partition heals. For scenario 3, activate a DR replica in another EU zone. For scenario 4, consider whether to serve stale reads or redirect to another region.
</details>

### Part D: Design the Conflict Resolution Strategy
If using multi-master across regions:
1. What conflict types can occur
2. How to detect conflicts (timestamps, version vectors)
3. How to resolve each conflict type
4. How to handle the edge case of split-brain during network partition

<details>
<summary>Hint</summary>
If data is partitioned by region (Part B), most conflicts are avoided. Cross-region conflicts only occur for global data (e.g., product catalog, system config). Use version vectors for detection and last-writer-wins for resolution. For split-brain, designate one region as the "source of truth" for global data.
</details>

## Success Criteria

- [ ] Architecture diagram shows all regions, instances, and replication links
- [ ] Data residency rules enforce GDPR compliance
- [ ] Each failover scenario has a specific, tested procedure
- [ ] Conflict resolution handles cross-region writes correctly
- [ ] Read and write latency requirements are met for all regions

## What You Should Understand After This Exercise

Cross-region replication is not just a technical problem -- it involves regulatory compliance, latency budgets, and conflict resolution trade-offs. The architecture must be designed around data residency requirements first, then optimized for latency and availability. Partitioning data by region simplifies many problems but introduces complexity for cross-region queries.
