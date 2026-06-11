# Exercise 01: Scaling Strategy Identification

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective
Identify the correct scaling strategy for different database workload patterns and justify your choices.

## Scenario
You are the DBA for an e-commerce platform. The application has the following workload characteristics:

| Component | Read:Write Ratio | Peak QPS | Latency Requirement |
|-----------|------------------|----------|---------------------|
| Product catalog | 95:5 | 50,000 | < 10ms |
| Shopping cart | 60:40 | 8,000 | < 50ms |
| Order processing | 20:80 | 2,000 | < 100ms |
| Analytics dashboard | 99:1 | 500 | < 2s |
| User sessions | 70:30 | 20,000 | < 5ms |

## Tasks

### Part A: Classify Each Workload
For each component in the table above, determine:
1. Is this workload read-heavy, write-heavy, or balanced?
2. Would read replicas help this workload? Why or why not?
3. Would connection pooling provide benefit? Why?

<details>
<summary>Hint</summary>
Read-heavy workloads (high read:write ratio) benefit most from read replicas. Connection pooling helps when there are many short-lived connections or high connection churn.
</details>

### Part B: Design a Scaling Strategy
Create a scaling strategy for the entire platform. For each component, specify:
1. Number of read replicas (0, 1, 2, or more)
2. Connection pool size
3. Any special routing rules

<details>
<summary>Hint</summary>
The product catalog and analytics dashboard are prime candidates for multiple read replicas. Shopping cart and user sessions need careful consideration because of session affinity requirements.
</details>

### Part C: Identify Anti-Patterns
The following scaling approaches were proposed by a junior engineer. Identify what is wrong with each:

1. "Add 10 read replicas to handle all our traffic"
2. "Set connection pool size to 1000 for maximum throughput"
3. "Route all queries to replicas to reduce load on the primary"
4. "We do not need connection pooling because PostgreSQL handles connections fine"

<details>
<summary>Hint</summary>
Consider: replication lag, connection overhead, write query requirements, and the difference between connection handling and connection pooling.
</details>

## Success Criteria
- [ ] Each workload is correctly classified as read-heavy, write-heavy, or balanced
- [ ] Scaling strategy accounts for replication lag on write-dependent reads
- [ ] Connection pool sizes are justified with reasoning
- [ ] All four anti-patterns are correctly identified with explanations

## What You Should Understand After This Exercise
Scaling a database is not one-size-fits-all. Different workloads within the same application may need different strategies. Read replicas help read-heavy workloads but introduce replication lag. Connection pooling reduces overhead but must be sized correctly. Always match the scaling strategy to the workload characteristics.
