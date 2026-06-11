# Exercise 01: Replication Topology Design

**Type:** Conceptual | **Time:** 15 min | **Difficulty:** Easy

## Objective
Choose the correct replication topology for different application scenarios based on consistency, latency, and availability requirements.

## Scenario
You are the database architect for a company that runs five distinct applications. Each has different requirements for data consistency, read/write patterns, and geographic distribution.

| Application | Consistency Need | Write Regions | Read Latency Budget | Availability Target |
|-------------|-----------------|---------------|--------------------|--------------------|
| Banking ledger | Strong (ACID) | 1 (US-East) | < 20ms | 99.99% |
| Social media feed | Eventual | 3 (US, EU, APAC) | < 10ms | 99.95% |
| Inventory system | Strong | 1 (US-East) | < 50ms | 99.9% |
| Content delivery | Eventual | 3 (US, EU, APAC) | < 5ms | 99.99% |
| Analytics warehouse | Eventual | 1 (US-East) | < 500ms | 99.5% |

## Tasks

### Part A: Classify Each Application
For each application, determine:
1. Is this a master-slave, multi-master, or single-node workload?
2. How many replicas are needed?
3. Should replication be synchronous or asynchronous?

<details>
<summary>Hint</summary>
Strong consistency in a single write region points to master-slave with synchronous replication. Eventual consistency across multiple regions suggests multi-master with async replication. Low availability targets may not need replication at all.
</details>

### Part B: Draw Topology Diagrams
Create an ASCII diagram for the banking ledger and social media feed topologies. Include:
- All database nodes and their roles
- Replication links with sync/async labels
- Client connection routing

<details>
<summary>Hint</summary>
Master-slave topology: one primary with arrows pointing to replicas. Multi-master: bidirectional arrows between nodes. Use labels like "sync" or "async" on each link.
</details>

### Part C: Identify Trade-offs
For each topology choice, write one paragraph explaining:
1. What you gain with this topology
2. What you sacrifice
3. A scenario where this topology would break down

<details>
<summary>Hint</summary>
Synchronous replication guarantees zero data loss but adds latency. Multi-master allows local writes but introduces conflict resolution complexity. Single-node has no replication overhead but is a single point of failure.
</details>

## Success Criteria

- [ ] Each application is correctly classified with justification
- [ ] Replication mode (sync/async) matches consistency requirements
- [ ] Topology diagrams are accurate and include all required elements
- [ ] Trade-off analysis covers both benefits and failure scenarios

## What You Should Understand After This Exercise

There is no universally best replication topology. Master-slave excels at read scaling with strong consistency. Multi-master enables low-latency writes in multiple regions but requires conflict resolution. The right choice depends on the consistency model, geographic distribution, and availability requirements of each application.
