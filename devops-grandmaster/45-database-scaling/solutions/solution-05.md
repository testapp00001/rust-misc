# Solution 05: End-to-End Scaling Architecture

## Part A: Architecture Design

```
                          +------------------+
                          |   Load Balancer  |
                          |   (HAProxy/LB)   |
                          +--------+---------+
                                   |
                    +--------------+--------------+
                    |              |              |
              +-----+----+  +-----+----+  +-----+----+
              | App Node |  | App Node |  | App Node |
              |    1     |  |    2     |  |    3     |
              +-----+----+  +-----+----+  +-----+----+
                    |              |              |
                    +--------------+--------------+
                                   |
                          +--------+---------+
                          |    PgBouncer     |
                          | (Connection Pool)|
                          |  200 max clients |
                          |  60 server conns |
                          +--------+---------+
                                   |
                          +--------+---------+
                          |     ProxySQL     |
                          | (Query Router)   |
                          +--------+---------+
                                   |
                    +--------------+--------------+
                    |                             |
              +-----+-----+               +------+-----+
              |  Primary  |  ---------->  |  Replica   |
              | PostgreSQL|   streaming   | PostgreSQL |
              | (writes)  | replication   |  (reads)   |
              +-----------+               +------+-----+
                                                  |
                                        +---------+---------+
                                        |                  |
                                  +-----+-----+     +------+-----+
                                  |  Replica  |     |  Replica  |
                                  |  2 (reads)|     |  3 (reads)|
                                  +-----------+     +-----------+
```

### Component Breakdown

| Component | Count | Role | Capacity |
|-----------|-------|------|----------|
| App Nodes | 3 | Application servers | ~33,000 read QPS each |
| PgBouncer | 1 (or 2 for HA) | Connection pooling | 200 client -> 60 server connections |
| ProxySQL | 1 (or 2 for HA) | Query routing | Routes reads to replicas, writes to primary |
| Primary | 1 | Handles all writes | ~15,000 write QPS with proper indexing |
| Replicas | 3 | Handle all reads | ~35,000 read QPS each |

Total read capacity: 3 replicas * 35,000 = 105,000 QPS (meets 100,000 target)
Total write capacity: 15,000 QPS (exceeds 10,000 target)

## Part B: Configuration Specifications

### PgBouncer Configuration

```ini
[databases]
appdb = host=proxysql port=6033 dbname=appdb

[pgbouncer]
listen_addr = 0.0.0.0
listen_port = 6432
pool_mode = transaction
default_pool_size = 20
min_pool_size = 5
reserve_pool_size = 5
max_client_conn = 200
max_db_connections = 60
query_timeout = 10
client_idle_timeout = 120
server_idle_timeout = 300
```

### ProxySQL Configuration

```ini
mysql_servers=
(
    { address="pg-primary", port=5432, hostgroup=10, max_connections=20 },
    { address="pg-replica-1", port=5432, hostgroup=20, max_connections=50 },
    { address="pg-replica-2", port=5432, hostgroup=20, max_connections=50 },
    { address="pg-replica-3", port=5432, hostgroup=20, max_connections=50 }
)

mysql_query_rules=
(
    { rule_id=100, match_digest="^SELECT .* FOR UPDATE$", destination_hostgroup=10, apply=1 },
    { rule_id=200, match_digest="^SELECT", destination_hostgroup=20, apply=1 },
    { rule_id=300, match_digest="^INSERT|^UPDATE|^DELETE", destination_hostgroup=10, apply=1 }
)
```

### PostgreSQL Primary Configuration

```ini
# postgresql.conf for primary
max_connections = 100
shared_buffers = 8GB
effective_cache_size = 24GB
work_mem = 256MB
maintenance_work_mem = 1GB
wal_level = replica
max_wal_senders = 10
max_replication_slots = 5
synchronous_commit = on
wal_buffers = 64MB
checkpoint_completion_target = 0.9
random_page_cost = 1.1
effective_io_concurrency = 200
shared_preload_libraries = 'pg_stat_statements'
```

### Replication Configuration

```sql
-- On primary: create replication user and slot
CREATE ROLE replicator WITH REPLICATION LOGIN PASSWORD 'replpass';
SELECT * FROM pg_create_physical_replication_slot('replica_1_slot');
SELECT * FROM pg_create_physical_replication_slot('replica_2_slot');
SELECT * FROM pg_create_physical_replication_slot('replica_3_slot');

-- On each replica: standby.signal
-- primary_conninfo = 'host=pg-primary port=5432 user=replicator password=replpass'
-- primary_slot_name = 'replica_N_slot'
```

## Part C: Monitoring and Alerting

### Key Metrics

| Component | Metric | Description |
|-----------|--------|-------------|
| Replicas | replication_lag_bytes | WAL bytes behind primary |
| Replicas | replication_lag_seconds | Time behind primary |
| PgBouncer | cl_active | Active client connections |
| PgBouncer | sv_active | Active server connections |
| PgBouncer | avg_query_time | Average query duration |
| ProxySQL | hostgroup_latency | Latency per hostgroup |
| ProxySQL | query_digest_stats | Query routing distribution |
| Primary | tps | Transactions per second |
| Primary | cache_hit_ratio | Buffer cache effectiveness |
| All | cpu_usage | CPU utilization |
| All | disk_usage | Disk space remaining |

### Alert Thresholds

```yaml
alerts:
  - name: replication_lag_critical
    condition: replication_lag_seconds > 5
    severity: critical
    action: Page on-call DBA

  - name: replication_lag_warning
    condition: replication_lag_seconds > 1
    severity: warning
    action: Alert in Slack

  - name: pool_exhaustion
    condition: sv_idle == 0 AND sv_active == max_db_connections
    severity: critical
    action: Page on-call DBA

  - name: high_p99_latency
    condition: p99_latency_ms > 100
    severity: warning
    action: Investigate slow queries

  - name: disk_space_low
    condition: disk_usage_percent > 85
    severity: warning
    action: Plan storage expansion

  - name: primary_cpu_high
    condition: cpu_usage_percent > 80 FOR 5m
    severity: warning
    action: Review query load
```

### Dashboard Layout

```
+------------------+------------------+------------------+
|   Overall QPS    |  Replication Lag |  Connection Pool |
|   [graph]        |  [graph]         |  [gauge]         |
+------------------+------------------+------------------+
|   Primary CPU    |  Replica Latency |  Error Rate      |
|   [graph]        |  [graph]         |  [counter]       |
+------------------+------------------+------------------+
|          Query Latency Percentiles (p50/p95/p99)      |
|                    [graph]                             |
+-------------------------------------------------------+
```

## Part D: Failure Scenarios

### 1. One Read Replica Goes Down

**Impact**: 33% reduction in read capacity (2 of 3 replicas remain). Read latency may increase slightly due to higher load on remaining replicas.

**Response**:
1. ProxySQL health checks detect the failure within `monitor_read_only_interval` (15 seconds)
2. ProxySQL automatically removes the failed replica from rotation
3. Alert fires -- investigate the root cause (disk failure, OOM, network partition)
4. If the replica cannot be recovered within 30 minutes, spin up a new replica from a base backup of the primary
5. Once recovered, re-add the replica to ProxySQL and verify replication lag is zero before routing reads

### 2. Connection Pooler Becomes Saturated

**Impact**: New client connections are queued. Application requests experience increased latency. If `query_wait_timeout` is reached, clients receive errors.

**Response**:
1. Immediately check for long-running queries: `SHOW CLIENTS;` in PgBouncer
2. Kill long-running queries that exceed `query_timeout`
3. Check for connection leaks -- clients holding connections without activity
4. Short-term fix: increase `max_client_conn` by 50% and add `reserve_pool_size`
5. Long-term fix: add a second PgBouncer instance behind a load balancer
6. Review application code for connection leak patterns

### 3. Primary Database Disk is 90% Full

**Impact**: If the disk fills completely, PostgreSQL will refuse writes and the entire system will halt. Replication will stop.

**Response**:
1. **Immediate**: Check for large WAL files or temporary files: `du -sh /var/lib/postgresql/data/pg_wal/`
2. **Immediate**: Run `VACUUM FULL` on bloated tables if space can be reclaimed
3. **Short-term**: Archive old data to cold storage and delete from the database
4. **Short-term**: Add storage (expand EBS volume or add a new tablespace on a separate disk)
5. **Long-term**: Implement partitioning to enable dropping old partitions instead of deleting rows
6. **Prevention**: Set up disk usage alerts at 70% and 80%

### 4. New Feature Introduces a Full Table Scan

**Impact**: A single unoptimized query causes sequential scans on large tables. CPU spikes, query latency increases across all queries, and replicas may fall behind due to resource contention.

**Response**:
1. Identify the query from `pg_stat_statements` ordered by `total_time` or `mean_time`
2. Run `EXPLAIN ANALYZE` to confirm the full table scan
3. **Immediate mitigation**: Add `statement_timeout` to kill queries longer than 5 seconds
4. **Short-term fix**: Add the missing index identified by the execution plan
5. **Long-term prevention**: Require `EXPLAIN ANALYZE` output for all new queries in code review
6. **Long-term prevention**: Set up query performance regression testing in CI/CD

## Common Mistakes to Avoid
- Designing for current load instead of target load -- build for 2x the target
- Not having automated failover -- manual failover takes too long during incidents
- Monitoring only the primary and forgetting replicas -- replicas can fail silently
- Over-provisioning replicas for reads while under-provisioning the primary for writes

## Key Takeaway
A production database scaling architecture requires multiple layers: connection pooling to handle connection overhead, query routing to distribute reads across replicas, proper PostgreSQL tuning for each role, and comprehensive monitoring to detect problems early. Every component must be designed for failure -- health checks, automatic removal from rotation, and recovery procedures should all be in place before they are needed.
