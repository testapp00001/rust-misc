# Exercise 04: Replication Lag Analysis and Mitigation

**Type:** Challenge | **Time:** 45 min | **Difficulty:** Medium-Hard

## Objective
Diagnose the root cause of replication lag in a production PostgreSQL cluster and implement targeted solutions.

## Scenario
Your PostgreSQL cluster has one primary and three read replicas. During business hours (9 AM - 6 PM), replication lag on all three replicas increases from a baseline of 50ms to over 30 seconds. The lag spikes correlate with:
- Bulk data imports at 10 AM and 2 PM
- Report generation queries on the replicas at 11 AM and 3 PM
- Peak user traffic from 12 PM to 2 PM

The current monitoring shows:

```
Replica 1 (us-east-1a): lag = 28s, replay_rate = 15 MB/s
Replica 2 (us-east-1b): lag = 32s, replay_rate = 12 MB/s
Replica 3 (us-west-2a): lag = 45s, replay_rate = 8 MB/s
```

## Tasks

### Part A: Identify Root Causes
Analyze the scenario and identify 5 possible causes of the replication lag. For each cause, explain:
1. Why it causes lag
2. How to confirm it is the actual cause
3. Which monitoring metric would show it

<details>
<summary>Hint</summary>
Consider: WAL generation rate on primary, replica I/O capacity, replica CPU for replay, network bandwidth between regions, long-running queries on replicas blocking replay, and checkpoint configuration.
</details>

### Part B: Write Diagnostic Queries
Write SQL queries to diagnose each cause identified in Part A:

```sql
-- Query 1: Check WAL generation rate on primary
-- (complete this)

-- Query 2: Check replay state on replica
-- (complete this)

-- Query 3: Check for blocking queries on replica
-- (complete this)

-- Query 4: Check network throughput
-- (complete this)

-- Query 5: Check checkpoint frequency
-- (complete this)
```

<details>
<summary>Hint</summary>
On the primary, use `pg_stat_wal` or `pg_current_wal_lsn()` to measure WAL generation. On the replica, use `pg_stat_activity` to find queries blocking the WAL receiver. Check `pg_stat_bgwriter` for checkpoint frequency.
</details>

### Part C: Implement Solutions
For each root cause, implement a specific solution:

1. **WAL generation too high**: How to reduce WAL volume during bulk imports
2. **Replica I/O bottleneck**: How to improve replay performance
3. **Blocking queries on replica**: How to handle long-running replica queries
4. **Network latency**: How to optimize cross-region replication
5. **Checkpoint pressure**: How to tune checkpoint settings

<details>
<summary>Hint</summary>
For bulk imports, consider `ALTER TABLE ... SET UNLOGGED` temporarily or `wal_level = minimal` for specific operations. For replica I/O, check `wal_compression` and I/O scheduler settings. For blocking queries, use `hot_standby_feedback` and `max_standby_streaming_delay`.
</details>

### Part D: Design a Monitoring Dashboard
Design a Grafana dashboard layout for replication health. Include:
1. Which panels to display
2. What queries each panel uses
3. Alert thresholds for each metric
4. A description of what each panel tells the operator

<details>
<summary>Hint</summary>
Key panels: replication lag (seconds), replay rate (MB/s), WAL generation rate, replica query load, network throughput between primary and replica, and checkpoint frequency. Use Prometheus with `postgres_exporter` for metrics collection.
</details>

## Success Criteria

- [ ] At least 5 root causes identified with explanations
- [ ] Diagnostic queries are syntactically correct and return useful metrics
- [ ] Solutions address each specific cause (not generic advice)
- [ ] Dashboard design covers all critical replication metrics
- [ ] Alert thresholds are specific and actionable

## What You Should Understand After This Exercise

Replication lag is rarely caused by a single factor. It is usually a combination of WAL generation rate, replica replay capacity, network bandwidth, and competing workloads on replicas. The solution is not always "add more replicas" -- sometimes you need to reduce WAL volume, optimize replay, or restructure how replicas are used.
