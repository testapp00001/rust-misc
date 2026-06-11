# Exercise 01: RPO and RTO Analysis

**Type:** Conceptual | **Time:** 15 min | **Difficulty:** Easy

## Objective

Analyze six critical database systems and determine appropriate Recovery Point Objective (RPO) and Recovery Time Objective (RTO) targets based on business impact.

## Scenario

Meridian Financial Services operates six critical database systems. The CTO has asked you to classify each system by its disaster recovery requirements. You must calculate downtime costs, set RPO/RTO targets, and map those targets to concrete technology choices.

| System | Description | Transactions/Hour | Revenue Dependency | Data Change Rate |
|--------|-------------|-------------------|-------------------|-----------------|
| PAYMENTS | Real-time payment processing | 50,000 | $180,000/hr direct revenue | 2,000 writes/sec |
| LEDGER | Double-entry accounting ledger | 20,000 | $0 direct, regulatory mandate | 800 writes/sec |
| CUSTOMER | Customer profiles and preferences | 5,000 | $40,000/hr indirect | 200 writes/sec |
| ANALYTICS | Business intelligence warehouse | 500 | $5,000/hr indirect | 50 writes/sec (batch) |
| SESSION | User session store (Redis-like) | 100,000 | $120,000/hr indirect | 10,000 writes/sec |
| ARCHIVE | Historical records (7-year retention) | 10 | $0/hr | 10 writes/sec (batch) |

Additional context:
- The company processes payments in a single AWS region (us-east-1).
- LEDGER data must satisfy SOX compliance (no data loss acceptable).
- SESSION data is ephemeral but drives user experience for the payment flow.
- ANALYTICS receives batch ETL loads every 4 hours.
- ARCHIVE is write-once, read-rarely, retained for regulatory purposes.

## Tasks

### Part A: Cost of Downtime Calculation

For each system, calculate the hourly cost of downtime by combining direct revenue loss, indirect revenue impact, and regulatory/compliance penalties.

1. Create a table with columns: System, Direct Loss/hr, Indirect Loss/hr, Regulatory Risk, Total Cost/hr.
2. Rank the systems from highest to lowest total cost.
3. For LEDGER, factor in a potential SOX violation fine of $500,000 if data loss exceeds zero.

<details>
<summary>Hint</summary>
Think about which costs are linear with time and which are threshold-based. A SOX violation is not proportional to downtime length -- it triggers at any data loss.
</details>

### Part B: RPO and RTO Determination

For each system, propose an RPO and RTO. Justify each value with a brief explanation.

1. Create a table with columns: System, RPO, RTO, Justification.
2. Ensure your RPO/RTO values are consistent with the cost rankings from Part A.
3. Group the systems into tiers: Tier 1 (seconds), Tier 1 (minutes), Tier 2 (minutes), Tier 3 (hours).

<details>
<summary>Hint</summary>
RPO answers: "How much data can we afford to lose?" RTO answers: "How long can we be down?" A system with $180,000/hr direct loss needs both RPO and RTO measured in seconds, not minutes.
</details>

### Part C: Technology Mapping

Map each system's RPO/RTO tier to specific technology choices.

1. For each tier, select the replication strategy: synchronous replication, asynchronous replication, periodic snapshots, or cold backups.
2. For each tier, select the failover mechanism: automatic with consensus, automatic with health checks, manual with runbook, or restore from backup.
3. Briefly explain why each technology choice fits the tier's RPO/RTO requirements.

<details>
<summary>Hint</summary>
Synchronous replication guarantees RPO=0 but adds latency. Asynchronous replication can lose seconds of data. Snapshots lose the time between snapshots. Cold backups lose everything since the last backup.
</details>

## Success Criteria

- [ ] All six systems have a calculated total cost of downtime per hour.
- [ ] Cost ranking clearly identifies which systems are most critical.
- [ ] Each system has a justified RPO and RTO value.
- [ ] Systems are grouped into 3-4 tiers with consistent RPO/RTO ranges.
- [ ] Technology choices are mapped to each tier with clear rationale.
- [ ] The SOX compliance requirement for LEDGER is explicitly addressed.

## What You Should Understand

- The difference between RPO (data loss tolerance) and RTO (downtime tolerance).
- How business impact drives technical DR requirements.
- Why a one-size-fits-all DR strategy wastes money on low-value systems and under-protects high-value ones.
- How to translate abstract business requirements into concrete replication and failover technology choices.
