# Exercise 01: CAP Theorem and PACELC

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

You will apply the CAP theorem and PACELC extension to classify real-world distributed systems, understand why "CA" systems do not exist in practice, and identify the trade-offs each system makes.

## Scenario

Your team is evaluating databases for a new microservices platform. The architect asks: "Which databases are CP and which are AP? And what does that even mean for our application?" You need to explain the CAP theorem, apply it to real systems, and extend the analysis with PACELC.

## Tasks

### Part A: CAP Theorem Classification

Classify each system as CP, AP, or CA. For each, explain what happens during a network partition.

1. etcd
2. Cassandra
3. PostgreSQL (single node)
4. MongoDB (with majority write concern)
5. DynamoDB
6. CockroachDB

<details>
<summary>Hint</summary>
CP systems refuse writes during partitions to maintain consistency. AP systems accept writes during partitions and resolve conflicts later. CA systems do not tolerate partitions (only possible on a single node).
</details>

### Part B: PACELC Analysis

The PACELC theorem says: if Partition, choose Availability or Consistency; Else (no partition), choose Latency or Consistency. For each system below, fill in both choices:

| System | Partition: A or C | Else: L or C |
|--------|-------------------|---------------|
| Cassandra | | |
| MongoDB | | |
| DynamoDB | | |
| PostgreSQL | | |
| CockroachDB | | |

<details>
<summary>Hint</summary>
Cassandra always favors availability and low latency. MongoDB favors consistency (majority writes, primary reads). CockroachDB favors consistency in both cases (serializable isolation).
</details>

### Part C: Real-World Impact

Your application has two requirements:

1. **User session store:** Users must be able to log in and access their sessions even during a network partition. Stale session data is acceptable for a few minutes.
2. **Financial ledger:** Every transaction must be consistent. If there is a partition, the system should refuse to process transactions rather than risk inconsistency.

For each requirement, recommend a CP or AP system and explain why the trade-off is acceptable.

<details>
<summary>Hint</summary>
Think about the cost of inconsistency in each case. A stale session is annoying. A stale financial ledger is a compliance violation.
</details>

## Success Criteria

- [ ] All 6 systems are correctly classified with explanations.
- [ ] PACELC table is complete and accurate.
- [ ] Each real-world recommendation is justified with a concrete consequence of getting it wrong.
- [ ] You can explain why "CA" is not achievable in a real distributed system.

## What You Should Understand After This Exercise

The CAP theorem is not a menu of features to pick from -- it is a constraint. Network partitions are unavoidable in distributed systems, so the real choice is between consistency (CP) and availability (AP). PACELC extends this by acknowledging that even without partitions, there is a trade-off between latency and consistency. The right choice depends on the cost of inconsistency for your specific use case.
