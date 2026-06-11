# Exercise 05: HA Architecture for Zero-Downtime Operations

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Design a complete high availability architecture that supports zero-downtime deployments and maintenance operations, integrating active-passive replication, consensus-based leader election, rolling upgrades, blue-green schema migrations, connection draining, and comprehensive monitoring.

## Scenario

You are the lead architect for a financial services platform that processes 50,000 transactions per second. The platform must maintain 99.99% availability (52 minutes downtime per year). Your stack includes:

- **Application tier:** 6 application servers behind a load balancer
- **Database tier:** PostgreSQL cluster with Patroni + etcd (3 nodes)
- **Cache tier:** Redis Sentinel cluster (3 nodes)
- **Message queue:** RabbitMQ with quorum queues (3 nodes)
- **Load balancer:** HAProxy pair in active-passive

The operations team needs to perform:
- Application deployments (weekly)
- Database schema migrations (monthly)
- OS security patches (weekly)
- PostgreSQL minor version upgrades (quarterly)
- Hardware replacements (as needed)

All of these must be achievable without any user-visible downtime.

## Tasks

### Part A: Design the Architecture with Rolling Upgrades

Design the full HA architecture. Draw (in ASCII) the topology showing all components, their replication relationships, and the consensus layer.

```
# Draw the complete architecture diagram here
# Include:
# - All 6 application servers with their connection to HAProxy
# - HAProxy active-passive pair with VIP
# - PostgreSQL 3-node cluster with Patroni + etcd
# - Redis Sentinel cluster
# - RabbitMQ quorum queue cluster
# - Replication arrows between all stateful components
# - Health check paths
```

Write the rolling upgrade procedure for the application tier:

```bash
# Write the step-by-step procedure to roll a new application version
# across 6 servers with zero downtime
# Include: drain connections, stop old version, start new version,
# health check, move to next server
# What is the minimum number of servers that must be healthy at all times?
```

Write the rolling upgrade procedure for the PostgreSQL cluster:

```bash
# Write the step-by-step procedure to upgrade PostgreSQL minor version
# (e.g., 15.3 -> 15.4) across the 3-node Patroni cluster
# Include: which node to upgrade first, how to switchover,
# how to verify replication after each step
```

### Part B: Implement Blue-Green Deployment for Database Schema Changes

Design a blue-green approach for database schema migrations that does not require application downtime.

Write the migration strategy for adding a NOT NULL column to a large table:

```sql
-- Step 1: Add the column as nullable
-- Step 2: Backfill existing rows in batches
-- Step 3: Add the NOT NULL constraint
-- Write each step as a separate SQL statement with comments
-- Explain why batching is necessary for the backfill
```

Write the application-side code that handles both the old and new schema simultaneously:

```python
# Write a Python function that:
# 1. Checks which schema version the database is running
# 2. Uses the correct column set for queries
# 3. Handles the transition period where both old and new
#    application versions are running simultaneously
# 4. Provides a feature flag to control which schema to use
```

### Part C: Handle Connection Draining During Failover

Write the connection draining logic for HAProxy and the application layer.

HAProxy configuration for graceful connection draining:

```
# Write the HAProxy backend configuration that:
# 1. Supports health checks for all database endpoints
# 2. Routes read traffic to replicas and write traffic to primary
# 3. Supports a "drain" state that stops new connections
#    but allows existing connections to complete
# 4. Sets appropriate timeouts for connection lifetime
```

Application-level connection pool draining:

```python
# Write a Python class called ConnectionPoolManager that:
# 1. Maintains a pool of database connections
# 2. Supports a drain() method that:
#    a. Stops accepting new queries
#    b. Waits for in-flight queries to complete (with timeout)
#    c. Closes all connections gracefully
# 3. Supports a health_check() method
# 4. Handles failover by detecting primary change via Patroni REST API
# 5. Reconnects to the new primary automatically
```

### Part D: Design Monitoring and Alerting for HA Components

Design a comprehensive monitoring plan for every HA component.

Fill in the monitoring matrix:

| Component | Metric | Warning Threshold | Critical Threshold | Action |
|-----------|--------|-------------------|--------------------|--------|
| PostgreSQL Primary | Replication lag (bytes) | | | |
| PostgreSQL Primary | Connection count | | | |
| PostgreSQL Standby | Replication lag (seconds) | | | |
| etcd Cluster | Leader elections per hour | | | |
| etcd Cluster | Proposal commit latency | | | |
| Patroni | Failover count per day | | | |
| HAProxy | Backend server health | | | |
| Redis Sentinel | Sentinel reachable count | | | |
| Application | Error rate (5xx) | | | |
| Application | Request latency (p99) | | | |

Write the Prometheus alert rules for the three most critical scenarios:

```yaml
# Write Prometheus alerting rules for:
# 1. PostgreSQL replication lag exceeding threshold
# 2. etcd cluster has no leader (quorum lost)
# 3. Patroni failover detected (potential incident)
# Include: alert name, condition, for duration, labels, annotations
```

Write the runbook outline for a Patroni failover event:

```bash
# Write a runbook with these sections:
# 1. Alert description and impact
# 2. First 5 minutes: What to check immediately
# 3. Diagnosis: How to determine root cause
# 4. Resolution: Steps to fix (if manual intervention needed)
# 5. Verification: How to confirm the system is healthy
# 6. Post-incident: What to document
```

## Success Criteria

- [ ] Architecture diagram shows all components with replication and consensus relationships
- [ ] Rolling upgrade procedures maintain quorum at every step (documented minimum healthy count)
- [ ] Blue-green schema migration handles the transition period without data loss
- [ ] Connection draining stops new traffic and waits for in-flight requests with a timeout
- [ ] Monitoring matrix covers all HA components with specific, actionable thresholds
- [ ] Prometheus alert rules use correct PromQL syntax and include appropriate `for` durations
- [ ] Runbook provides specific commands, not just descriptions

## Hints

<details>
<summary>Hint 1: Minimum healthy count for rolling upgrades</summary>

For a 6-node application tier behind a load balancer, you can safely take 1 node offline at a time, keeping 5/6 (83%) capacity. For the 3-node etcd/PostgreSQL cluster, you must keep 2/3 nodes running to maintain quorum. Never restart more than one database node at a time. The rolling upgrade order for PostgreSQL should be: replicas first (one at a time), then perform a Patroni switchover to promote an already-upgraded replica, then upgrade the old primary last.

</details>

<details>
<summary>Hint 2: Safe schema migration pattern</summary>

The safest pattern for adding a NOT NULL column is:
1. Add column as nullable (instant, metadata-only operation)
2. Deploy new application code that writes to both old and new columns
3. Backfill existing rows in batches of 1000-5000 with a short sleep between batches
4. Once backfill is complete, add a CHECK constraint (NOT VALID first, then VALIDATE CONSTRAINT)
5. Add NOT NULL constraint only after the CHECK is validated
6. Deploy application code that uses only the new column
7. Drop the old column in a future migration

This approach means at no point is the application unable to read or write data.

</details>

<details>
<summary>Hint 3: Patroni REST API for health</summary>

Patroni exposes a REST API on each node that provides cluster state information. Use it for monitoring and for application-level failover detection:

```bash
# Check cluster state
curl -s http://10.0.0.10:8008/cluster | python3 -m json.tool

# Check if this node is the leader
curl -s http://10.0.0.10:8008/leader
# Returns HTTP 200 if leader, HTTP 503 if not

# Trigger a controlled switchover
curl -s -X POST http://10.0.0.10:8008/switchover \
  -d '{"leader": "node1", "candidate": "node2"}'
```

</details>

## What You Should Understand After This Exercise

Zero-downtime operations require every layer of the stack to support graceful transitions. Rolling upgrades work because you never take down enough nodes to lose quorum. Blue-green schema migrations work because the application handles both old and new schemas during the transition. Connection draining works because you stop new traffic before killing old connections. And the whole system is held together by monitoring that detects problems before users notice. The architecture is only as strong as its weakest link -- a single component that does not support graceful shutdown will break your zero-downtime promise.
