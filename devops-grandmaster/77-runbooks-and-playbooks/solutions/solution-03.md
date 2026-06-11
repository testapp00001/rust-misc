# Solution 03: Database Failover Runbook

## Problem Statement

Write a complete runbook for PostgreSQL database failover using Patroni for
automatic failover, including manual failover procedures, verification queries,
and rollback steps.

## Complete Runbook

```markdown
# RUNBOOK: PostgreSQL Database Failover (Patroni)

| Field            | Value                                      |
|------------------|--------------------------------------------|
| **Runbook ID**   | RB-DB-001                                  |
| **Severity**     | P1 - Critical                              |
| **Owner**        | Database Reliability Engineering (DBRE)    |
| **Last Updated** | 2026-06-11                                 |
| **Review Cycle** | Monthly                                    |
| **Tags**         | postgresql, patroni, failover, ha, etcd    |

---

## 1. Overview

This runbook covers PostgreSQL high-availability failover scenarios using
Patroni as the cluster manager. Patroni uses a distributed consensus store
(etcd, Consul, or ZooKeeper) to manage leader election and automatic failover.

**Architecture:**

```
                    ┌─────────────┐
                    │   etcd      │
                    │  Cluster    │
                    │ (3 nodes)   │
                    └──────┬──────┘
                           │ leader key / consensus
              ┌────────────┼────────────┐
              │            │            │
        ┌─────┴─────┐ ┌───┴─────┐ ┌───┴─────┐
        │  Patroni   │ │ Patroni │ │ Patroni │
        │  (Leader)  │ │(Replica)│ │(Replica)│
        │            │ │         │ │         │
        │ PostgreSQL │ │Postgres │ │Postgres │
        │  Primary   │ │ Replica │ │ Replica │
        └─────┬──────┘ └───┬─────┘ └───┬─────┘
              │            │            │
              └────────────┼────────────┘
                           │
                    ┌──────┴──────┐
                    │ HAProxy /   │
                    │ PgBouncer   │
                    │ (read/write │
                    │  routing)   │
                    └─────────────┘
```

**Failover types covered:**

| Type | Trigger | Downtime | Data Loss Risk |
|------|---------|----------|----------------|
| Automatic | Leader node failure detected by Patroni | 10-30 seconds | Minimal (synchronous replication) |
| Planned (switchover) | Operator-initiated for maintenance | 5-15 seconds | None |
| Emergency manual | Patroni or etcd cluster is broken | 1-5 minutes | Possible |

---

## 2. Symptoms

Automatic failover may be triggered by:

- Primary node becomes unreachable (network partition, hardware failure)
- PostgreSQL process crashes on the primary
- Primary node runs out of disk space
- Patroni process dies on the primary (PostgreSQL is demoted)

**Alerts that indicate failover activity:**

```
# Patroni failover alert
ALERT: PatroniFailoverTriggered
  cluster: production-db
  leader: db-node-01
  reason: leader unreachable for 30s

# PostgreSQL replication lag alert
ALERT: PostgreSQLReplicationLagHigh
  cluster: production-db
  lag_bytes: 104857600
  lag_seconds: 45

# etcd leader election
ALERT: EtcdLeaderChanges
  cluster: etcd-production
  changes: 3 in 5m
```

---

## 3. Impact

- **Write unavailability**: During failover, no writes can occur (typically
  10-30 seconds for automatic, 1-5 minutes for manual).
- **Read disruption**: Replica connections may briefly fail during leader
  change detection.
- **Connection pool exhaustion**: Application connection pools may hold
  stale connections to the old primary.
- **Potential data loss**: If asynchronous replication is used, transactions
  committed on the old primary but not yet replicated may be lost.

---

## 4. Prerequisites

- [ ] SSH access to all database nodes (db-node-01, db-node-02, db-node-03)
- [ ] `patronictl` installed and configured on all nodes
- [ ] etcd cluster access (3-node etcd cluster)
- [ ] Application connection pooler (PgBouncer/HAProxy) access
- [ ] Monitoring dashboard access (Grafana + PostgreSQL exporter)
- [ ] Understanding of current replication mode (sync/async)

---

## 5. Diagnosis -- Is Automatic Failover Working?

### Step 1: Check Patroni Cluster Status

```bash
# On any database node or where patronictl is configured:
patronictl -c /etc/patroni/config.yml list

# Expected output for healthy cluster:
# + Cluster: production-db (7123456789012345678) ---+----+-----------+
# | Member       | Host        | Role    | State   | TL | Lag in MB |
# +--------------+-------------+---------+---------+----+-----------+
# | db-node-01   | 10.0.1.10   | Leader  | running | 12 |           |
# | db-node-02   | 10.0.1.11   | Replica | running | 12 | 0         |
# | db-node-03   | 10.0.1.12   | Replica | running | 12 | 0         |
# +--------------+-------------+---------+---------+----+-----------+

# If failover is in progress or has failed:
# + Cluster: production-db (7123456789012345678) ---+----+-----------+
# | Member       | Host        | Role    | State      | TL | Lag in MB |
# +--------------+-------------+---------+------------+----+-----------+
# | db-node-01   | 10.0.1.10   | Leader  | start failed| 12 |          |
# | db-node-02   | 10.0.1.11   | Replica | running     | 12 | 150      |
# | db-node-03   | 10.0.1.12   | Replica | running     | 12 | 0        |
# +--------------+-------------+---------+------------+----+-----------+
```

### Step 2: Check etcd Cluster Health

```bash
# Check etcd cluster health
etcdctl endpoint health --cluster

# Expected output:
# 10.0.1.1:2379 is healthy: successfully committed proposal: took = 1.2ms
# 10.0.1.2:2379 is healthy: successfully committed proposal: took = 1.5ms
# 10.0.1.3:2379 is healthy: successfully committed proposal: took = 1.1ms

# Check who holds the Patroni leader key
etcdctl get /service/production-db/leader --print-value-only
```

### Step 3: Check Replication Status on Current Primary

```bash
# Connect to the current primary
psql -h <primary-host> -U postgres -d postgres -c "
SELECT
    client_addr,
    state,
    sent_lsn,
    write_lsn,
    flush_lsn,
    replay_lsn,
    pg_wal_lsn_diff(sent_lsn, replay_lsn) AS replay_lag_bytes
FROM pg_stat_replication;
"

# Check if synchronous replication is configured
psql -h <primary-host> -U postgres -d postgres -c "
SHOW synchronous_standby_names;
SHOW synchronous_commit;
"
```

### Step 4: Check Application Connectivity

```bash
# Check HAProxy stats (if using HAProxy for routing)
curl -s "http://haproxy-host:8404/stats;csv" | grep "postgresql"

# Check PgBouncer pools (if using PgBouncer)
psql -h pgbouncer-host -p 6432 -U pgbouncer -c "SHOW POOLS;"

# Verify application can connect
psql -h <vip-or-service-dns> -U app_user -d app_db -c "SELECT 1;"
```

---

## 6. Resolution

### Option A: Automatic Failover (Let Patroni Handle It)

**When to use**: The primary node is down, and Patroni is detecting the failure.

```bash
# 1. Monitor the failover in real-time
watch -n 2 'patronictl -c /etc/patroni/config.yml list'

# 2. Watch Patroni logs for failover events
journalctl -u patroni -f --since "5 minutes ago"

# 3. Wait for Patroni to complete the failover (typically 10-30 seconds)
# Look for this log entry on the new leader:
# "Promoted self to leader by acquiring session lock"

# 4. Once failover is complete, verify the new cluster state
patronictl -c /etc/patroni/config.yml list
```

**Do NOT intervene** unless:
- Failover has not completed within 60 seconds
- Patroni is stuck in a loop
- etcd cluster is unhealthy

### Option B: Planned Switchover (Maintenance Window)

**When to use**: Planned maintenance on the current primary node.

```bash
# 1. Verify the target replica is healthy and caught up
patronictl -c /etc/patroni/config.yml list
# Ensure target replica shows "Lag in MB: 0"

# 2. Initiate switchover (interactive)
patronictl -c /etc/patroni/config.yml switchover

# You will be prompted:
# Master [db-node-01]:
# Candidate ['db-node-02', 'db-node-03'] []: db-node-02
# When should the switchover take place (e.g. 2026-06-11T10:30 )  [now]:
# Are you sure you want to switchover cluster production-db, demoting current master db-node-01? [y/N]: y

# 3. Monitor the switchover
patronictl -c /etc/patroni/config.yml list

# 4. Verify the new primary is accepting connections
psql -h <new-primary-host> -U postgres -c "SELECT pg_is_in_recovery();"
# Expected: f (false = primary, not in recovery)
```

### Option C: Emergency Manual Failover (Patroni/etcd Broken)

**When to use**: Patroni is not functioning, etcd is down, or automatic
failover has failed completely. **This is a last resort.**

```bash
# === STEP 1: Identify the best candidate for promotion ===
# Check replication status on each replica
for node in db-node-02 db-node-03; do
    echo "=== $node ==="
    ssh $node "psql -U postgres -c 'SELECT pg_last_wal_receive_lsn(), pg_last_wal_replay_lsn();'"
done

# Choose the replica with the highest LSN (most data)

# === STEP 2: Stop Patroni on all nodes ===
# On the old primary (if accessible):
ssh db-node-01 "sudo systemctl stop patroni"

# On the replica you will promote:
ssh db-node-02 "sudo systemctl stop patroni"

# On the other replica:
ssh db-node-03 "sudo systemctl stop patroni"

# === STEP 3: Promote the chosen replica ===
ssh db-node-02 "sudo -u postgres pg_ctl promote -D /var/lib/postgresql/data"
# Or using SQL:
ssh db-node-02 "psql -U postgres -c 'SELECT pg_promote();'"

# === STEP 4: Verify promotion ===
ssh db-node-02 "psql -U postgres -c 'SELECT pg_is_in_recovery();'"
# Expected: f

# === STEP 5: Reconfigure the other replica to follow the new primary ===
ssh db-node-03 "
    sudo -u postgres psql -c \"ALTER SYSTEM SET primary_conninfo = 'host=10.0.1.11 port=5432 user=replicator';\"
    sudo -u postgres psql -c \"SELECT pg_reload_conf();\"
    # Or if using standby.signal:
    sudo -u postgres touch /var/lib/postgresql/data/standby.signal
    sudo systemctl restart postgresql
"

# === STEP 6: Restart Patroni on all nodes ===
# Start on the new primary first
ssh db-node-02 "sudo systemctl start patroni"
sleep 5

# Then on the replica
ssh db-node-03 "sudo systemctl start patroni"
sleep 5

# Then on the old primary (it will come up as a replica)
ssh db-node-01 "sudo systemctl start patroni"

# === STEP 7: Update HAProxy/PgBouncer if needed ===
# If using static config, update the primary backend:
# server db-primary 10.0.1.11:5432 check  <-- was 10.0.1.10
ssh haproxy-host "sudo systemctl reload haproxy"
```

---

## 7. Verification

### Immediate Verification (0-5 minutes)

```bash
# 1. Verify Patroni cluster is healthy
patronictl -c /etc/patroni/config.yml list
# All members should show "running", new leader in "Leader" role

# 2. Verify the new primary is accepting reads and writes
psql -h <new-primary-host> -U postgres -c "
SELECT
    pg_is_in_recovery() AS is_replica,
    pg_current_wal_lsn() AS current_lsn,
    now() AS check_time;
"
# is_replica should be: f

# 3. Verify replication is working on remaining replicas
psql -h <new-primary-host> -U postgres -c "
SELECT
    client_addr,
    state,
    sync_state,
    pg_wal_lsn_diff(pg_current_wal_lsn(), replay_lsn) AS lag_bytes
FROM pg_stat_replication;
"
# All replicas should show state=streaming, lag_bytes should be small

# 4. Verify application connectivity
psql -h <service-endpoint> -U app_user -d app_db -c "SELECT count(*) FROM orders WHERE created_at > now() - interval '5 minutes';"
# Should return a count (proving writes are working)

# 5. Verify PgBouncer/HAProxy routing
psql -h pgbouncer-host -p 6432 -U pgbouncer -c "SHOW SERVERS;"
# Active connections should be to the new primary
```

### Extended Verification (5-30 minutes)

```bash
# 6. Monitor replication lag over time
watch -n 5 'psql -h <new-primary-host> -U postgres -c "
SELECT client_addr, pg_wal_lsn_diff(pg_current_wal_lsn(), replay_lsn) AS lag_bytes
FROM pg_stat_replication;"'

# 7. Check for any data inconsistency (if sync replication was used)
# On replica:
psql -h <replica-host> -U postgres -c "
SELECT count(*) FROM pg_stat_replication;
"

# 8. Verify monitoring is reporting correctly
# Check Grafana dashboards:
#   - Replication lag graph should show normal values
#   - Connection count should be stable
#   - Transaction rate should be normal
#   - No new alerts firing
```

### Verification Checklist

- [ ] Patroni cluster shows all members healthy
- [ ] New primary is accepting writes (`pg_is_in_recovery() = false`)
- [ ] Replicas are streaming from the new primary
- [ ] Replication lag is zero or minimal
- [ ] Application is connecting and executing queries
- [ ] HAProxy/PgBouncer is routing to the correct primary
- [ ] No new alerts firing in the last 10 minutes
- [ ] Application error rate has returned to baseline

---

## 8. Rollback Procedure

If the failover caused issues and you need to revert:

### Rollback from Planned Switchover

```bash
# Simply switch back to the original primary
patronictl -c /etc/patroni/config.yml switchover
# Select the original primary as the new target

# Verify
patronictl -c /etc/patroni/config.yml list
```

### Rollback from Emergency Manual Failover

```bash
# 1. Stop the promoted node
ssh db-node-02 "sudo systemctl stop patroni"
ssh db-node-02 "sudo systemctl stop postgresql"

# 2. Rebuild the old primary as a new replica
ssh db-node-01 "
    sudo systemctl stop patroni
    sudo systemctl stop postgresql
    sudo -u postgres rm -rf /var/lib/postgresql/data/*
    sudo -u postgres pg_basebackup -h 10.0.1.11 -D /var/lib/postgresql/data -Xs -P
    sudo -u postgres touch /var/lib/postgresql/data/standby.signal
    sudo systemctl start postgresql
    sudo systemctl start patroni
"

# 3. Promote the old primary back
patronictl -c /etc/patroni/config.yml switchover

# 4. Rebuild the other replica from the restored primary
# Repeat the basebackup process for db-node-03
```

**Warning**: Rollback from emergency failover may result in data loss if
the old primary had uncommitted transactions. Always verify data consistency
before rollback.

---

## 9. Post-Resolution

### Immediate Actions

1. **Document the timeline**: Record exactly when failover started, completed,
   and when service was fully restored.
2. **Verify data integrity**: Run consistency checks on critical tables.
3. **Notify stakeholders**: Confirm the database is stable.

### Root Cause Analysis

Answer these questions in the incident ticket:

1. What triggered the failover? (node crash, network partition, disk full, etc.)
2. Did automatic failover work as expected?
3. Was there any data loss?
4. How long was the service unavailable?
5. Were there any issues with application reconnection?

### Preventive Measures

| Root Cause | Preventive Action |
|-----------|-------------------|
| Node hardware failure | Ensure all nodes are in different failure domains |
| Network partition | Verify network redundancy between nodes |
| Disk full | Set up disk usage alerting at 70% and 85% |
| etcd cluster down | Monitor etcd health; run 5 etcd nodes for production |
| Replication lag | Set up lag alerting; tune `wal_keep_size` |
| Patroni misconfiguration | Version-control Patroni config; test in staging |

---

## 10. Escalation

| Time Elapsed | Action |
|-------------|--------|
| 0 min | Automatic failover begins; monitor Patroni logs |
| 2 min | If failover not complete, begin manual diagnosis |
| 5 min | Page DBRE on-call lead |
| 15 min | Declare P1 incident; engage incident commander |
| 30 min | Engage all database team members |
| 60 min | Engage engineering VP; consider external communication |
```

## Why This Works

1. **Three escalation levels**: The runbook distinguishes between automatic
   (hands-off), planned (controlled), and emergency (manual) failover. Most
   runbooks only cover one scenario, leaving engineers unprepared when the
   automation fails.

2. **Architecture diagram included**: Understanding the etcd -> Patroni ->
   PostgreSQL -> HAProxy chain is essential for diagnosis. Without it,
   engineers waste time figuring out which component is broken.

3. **Verification at every stage**: Each resolution option is followed by
   specific verification steps. This prevents the classic mistake of declaring
   victory after `patronictl list` shows a leader, without verifying that
   replication and application connectivity actually work.

4. **Rollback is explicit**: Many failover runbooks assume the failover
   succeeds. This runbook includes rollback procedures for when the failover
   itself causes problems (e.g., promoting a replica with stale data).

5. **Post-resolution addresses root cause**: The table mapping root causes
   to preventive actions turns a reactive procedure into a proactive
   improvement cycle.

## Common Mistakes

1. **Not checking replication lag before switchover**: If the target replica
   is 500MB behind the primary, switching over loses those transactions.
   Always verify `Lag in MB: 0` before initiating a planned switchover.

2. **Starting Patroni on the old primary too early**: In emergency failover,
   if you restart Patroni on the old primary before reconfiguring it as a
   replica, it may attempt to reassert itself as leader, causing a split-brain
   scenario.

3. **Forgetting to update connection routing**: After failover, HAProxy or
   PgBouncer may still be routing to the old primary. If using static config
   (not Patroni-aware routing), this must be updated manually.

4. **Not verifying application reconnection**: Connection pools (HikariCP,
   PgBouncer) may hold stale connections to the old primary. Applications
   need to detect the connection failure and reconnect. Verify this works
   before closing the incident.

5. **Ignoring etcd health**: Patroni failover depends on etcd. If etcd is
   unhealthy (split brain, quorum loss), Patroni cannot elect a new leader.
   Always check etcd health as part of diagnosis.

6. **No testing of failover in staging**: Failover procedures that have never
   been tested will fail in production. Schedule regular failover drills.
