# Exercise 05: End-to-End Scaling Architecture

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective
Design a complete database scaling architecture that combines read replicas, connection pooling, and query optimization for a production application.

## Scenario
You are the lead database engineer for a social media platform with the following requirements:

| Metric | Current | Target |
|--------|---------|--------|
| Daily active users | 100,000 | 2,000,000 |
| Read queries/sec | 5,000 | 100,000 |
| Write queries/sec | 500 | 10,000 |
| Average read latency | 50ms | < 10ms |
| Average write latency | 30ms | < 20ms |
| Database size | 50GB | 500GB |

The current architecture is a single PostgreSQL instance with direct connections from the application.

## Tasks

### Part A: Architecture Design
Design a scaling architecture that meets the target requirements. Your design must include:
1. Number and role of database instances
2. Connection pooling layer placement and configuration
3. Query routing strategy
4. Data partitioning strategy (if needed)

Draw an ASCII diagram of your architecture.

<details>
<summary>Hint</summary>
Consider a multi-tier approach: application -> connection pooler -> proxy/router -> database instances. With 100,000 read QPS and a target of < 10ms, you will need multiple read replicas behind a load balancer. Writes at 10,000 QPS may need partitioning.
</details>

### Part B: Configuration Specifications
Provide specific configuration for each component:

1. **PgBouncer configuration** for the connection pooler
2. **ProxySQL configuration** for query routing
3. **PostgreSQL configuration** changes for the primary
4. **Replication configuration** for each replica

<details>
<summary>Hint</summary>
For 100,000 read QPS across replicas, calculate how many replicas are needed given each replica can handle roughly 20,000 QPS with proper indexing. PgBouncer in transaction mode should sit between ProxySQL and PostgreSQL.
</details>

### Part C: Monitoring and Alerting
Design a monitoring strategy for this architecture. Specify:
1. Key metrics to track for each component
2. Alert thresholds for critical conditions
3. Dashboard layout for operations team

<details>
<summary>Hint</summary>
Monitor: replication lag (per replica), connection pool utilization, query latency percentiles (p50, p95, p99), QPS per instance, and cache hit ratios. Alert on replication lag > 1s, pool utilization > 80%, and p99 latency > target.
</details>

### Part D: Failure Scenarios
For each failure scenario, describe the impact and your response:

1. One read replica goes down
2. Connection pooler becomes saturated
3. Primary database disk is 90% full
4. A new feature introduces a query that causes a full table scan

<details>
<summary>Hint</summary>
For replica failure: ProxySQL health checks should automatically remove it from rotation. For pool saturation: increase `server_idle_timeout` to reclaim connections, and consider adding a second pooler. For disk: archive old data, add storage, or partition hot tables.
</details>

## Success Criteria
- [ ] Architecture diagram shows all components and data flow
- [ ] Configuration files are complete and consistent
- [ ] Monitoring covers all critical path components
- [ ] Each failure scenario has a specific, actionable response plan
- [ ] Design scales from current to target metrics with justification

## What You Should Understand After This Exercise
Production database scaling is a multi-dimensional problem. You need connection pooling to handle connection overhead, read replicas to distribute read load, query optimization to maximize throughput per instance, and monitoring to detect problems before they become outages. The architecture must be designed holistically -- optimizing one layer while ignoring another creates bottlenecks.
