# Solution 04: Replication Lag Analysis and Mitigation

---

## Part A: Root Cause Analysis

### Cause 1: High WAL Generation Rate During Bulk Imports

**Why it causes lag:** Bulk imports generate large volumes of WAL in a short period. The primary writes WAL to disk at its I/O speed, but replicas must receive, write, and replay each WAL record. If WAL generation exceeds the replica's replay capacity, lag accumulates.

**How to confirm:** Check WAL generation rate on the primary using `pg_stat_wal` or by measuring LSN changes over time.

**Metric:** `pg_stat_wal.stats_reset` and `pg_stat_wal.wal_bytes` -- calculate bytes/second.

### Cause 2: Replica I/O Bottleneck

**Why it causes lag:** The replica must write incoming WAL to disk and then replay it (read data pages, apply changes, write back). If the disk I/O is saturated, replay falls behind.

**How to confirm:** Check `iostat` on the replica for high `%util` or `await` on the WAL and data disks.

**Metric:** `iostat -x 1` -- look for `%util > 90%` on the data disk.

### Cause 3: Long-Running Queries on Replicas Blocking Replay

**Why it causes lag:** In hot standby mode, the WAL replay process must wait for queries that are reading data pages that are about to be modified. If a query holds a snapshot for a long time, replay is blocked.

**How to confirm:** Check `pg_stat_activity` on the replica for long-running queries.

**Metric:** `pg_stat_activity.state = 'active'` with `query_start` older than 10 seconds.

### Cause 4: Network Bandwidth Between Regions

**Why it causes lag:** Replica 3 is in us-west-2 while the primary is in us-east-1. Cross-region network bandwidth is limited and shared with other traffic.

**How to confirm:** Measure network throughput between primary and replica using `iperf3` or check AWS CloudWatch for `NetworkOut` on the primary.

**Metric:** `NetworkOut` on primary vs. `NetworkIn` on replica -- if primary sends more than replica receives, there is packet loss or throttling.

### Cause 5: Checkpoint Pressure on Replicas

**Why it causes lag:** Checkpoints flush dirty pages to disk. During a checkpoint, the replica's I/O is saturated, slowing down WAL replay. Frequent checkpoints (from `checkpoint_timeout` or `max_wal_size`) amplify this.

**How to confirm:** Check `pg_stat_bgwriter` on the replica for `checkpoints_timed` vs. `checkpoints_req`. High `checkpoints_req` means checkpoints are triggered by WAL volume, not time.

**Metric:** `pg_stat_bgwriter.checkpoints_req` -- if this is high relative to `checkpoints_timed`, checkpoint pressure is a factor.

## Part B: Diagnostic Queries

```sql
-- Query 1: WAL generation rate on primary
-- Run twice, 10 seconds apart, and calculate the difference
SELECT
    pg_current_wal_lsn() AS current_lsn,
    now() AS sample_time;

-- After 10 seconds:
SELECT
    pg_current_wal_lsn() AS current_lsn,
    now() AS sample_time,
    pg_wal_lsn_diff(
        pg_current_wal_lsn(),
        '0/12345678'::pg_lsn  -- replace with LSN from first query
    ) / 10 AS wal_bytes_per_second;
```

```sql
-- Query 2: Replay state on replica
SELECT
    pg_last_wal_receive_lsn() AS received_lsn,
    pg_last_wal_replay_lsn() AS replayed_lsn,
    pg_wal_lsn_diff(
        pg_last_wal_receive_lsn(),
        pg_last_wal_replay_lsn()
    ) AS replay_lag_bytes,
    CASE WHEN pg_last_wal_receive_lsn() = pg_last_wal_replay_lsn()
         THEN 0
         ELSE EXTRACT(EPOCH FROM now() - pg_last_xact_replay_timestamp())
    END AS replay_lag_seconds;
```

```sql
-- Query 3: Blocking queries on replica
SELECT
    pid,
    now() - query_start AS query_duration,
    state,
    wait_event_type,
    wait_event,
    left(query, 100) AS query_preview
FROM pg_stat_activity
WHERE state = 'active'
  AND now() - query_start > INTERVAL '10 seconds'
ORDER BY query_duration DESC;
```

```sql
-- Query 4: Replication slot lag
SELECT
    slot_name,
    active,
    pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn) AS slot_lag_bytes
FROM pg_replication_slots;
```

```sql
-- Query 5: Checkpoint frequency and duration on replica
SELECT
    checkpoints_timed,
    checkpoints_req,
    checkpoint_write_time / 1000 AS checkpoint_write_seconds,
    checkpoint_sync_time / 1000 AS checkpoint_sync_seconds,
    buffers_checkpoint,
    buffers_backend
FROM pg_stat_bgwriter;
```

## Part C: Solutions for Each Cause

### Solution 1: Reduce WAL Volume During Bulk Imports

```sql
-- Option A: Use UNLOGGED tables for staging data
-- (data is not WAL-logged, but lost on crash)
CREATE UNLOGGED TABLE import_staging (LIKE production_table);

-- Import into staging
COPY import_staging FROM '/data/import.csv';

-- Move to logged table in one transaction
INSERT INTO production_table SELECT * FROM import_staging;
DROP TABLE import_staging;

-- Option B: Use CREATE TABLE AS SELECT (minimal WAL)
-- Requires wal_level = minimal and single-transaction
BEGIN;
CREATE TABLE new_table AS SELECT * FROM import_data;
-- This generates minimal WAL because the table is created and
-- populated in the same transaction
COMMIT;

-- Option C: Batch the import to limit WAL rate
-- Instead of one large COPY, split into batches of 10,000 rows
-- with a small sleep between batches to let replicas catch up
```

```bash
#!/bin/bash
# Batch import script
BATCH_SIZE=10000
SLEEP_BETWEEN=2
TOTAL_ROWS=$(wc -l < /data/import.csv)

for ((offset=0; offset<TOTAL_ROWS; offset+=BATCH_SIZE)); do
    sed -n "$((offset+1)),$((offset+BATCH_SIZE))p" /data/import.csv > /tmp/batch.csv
    psql -c "\COPY staging_table FROM /tmp/batch.csv WITH CSV"
    echo "Imported rows $((offset+1)) to $((offset+BATCH_SIZE))"

    # Check replication lag before next batch
    LAG=$(psql -t -c "SELECT replay_lag FROM pg_stat_replication LIMIT 1;")
    if [[ "$LAG" > "00:00:05" ]]; then
        echo "Replication lag $LAG -- pausing..."
        sleep 10
    else
        sleep $SLEEP_BETWEEN
    fi
done
```

### Solution 2: Improve Replica I/O Performance

```ini
# postgresql.conf on replica

# Enable WAL compression to reduce I/O
wal_compression = on

# Increase shared buffers to reduce disk reads
shared_buffers = '4GB'

# Increase effective_io_concurrency for parallel I/O
effective_io_concurrency = 200

# Use a faster checkpoint completion
checkpoint_completion_target = 0.9

# Reduce checkpoint frequency
max_wal_size = '4GB'
checkpoint_timeout = '15min'
```

```bash
# Optimize I/O scheduler for SSDs
echo noop > /sys/block/nvme0n1/queue/scheduler

# Increase read-ahead for sequential WAL replay
blockdev --setra 4096 /dev/nvme0n1
```

### Solution 3: Handle Blocking Queries on Replicas

```ini
# postgresql.conf on replica

# Allow replay to cancel long-running queries that block it
max_standby_streaming_delay = '30s'

# Or, set to -1 to never cancel queries (replay waits instead)
# max_standby_streaming_delay = -1

# Enable hot standby feedback to tell the primary about long queries
hot_standby_feedback = on
```

```sql
-- Application-level: Set statement_timeout for replica queries
SET statement_timeout = '20s';

-- For analytics queries, use a read-only connection with timeout
-- and retry logic
```

### Solution 4: Optimize Cross-Region Network

```bash
# Enable TCP tuning for cross-region replication
sysctl -w net.core.rmem_max=16777216
sysctl -w net.core.wmem_max=16777216
sysctl -w net.ipv4.tcp_rmem="4096 87380 16777216"
sysctl -w net.ipv4.tcp_wmem="4096 65536 16777216"
```

```ini
# postgresql.conf on primary
# Reduce WAL sender overhead
wal_sender_timeout = '60s'
```

```yaml
# AWS-specific: Ensure dedicated bandwidth between regions
# Use VPC peering or Transit Gateway with sufficient bandwidth
# Consider AWS Global Accelerator for consistent cross-region latency
```

### Solution 5: Tune Checkpoint Settings

```ini
# postgresql.conf on replica

# Spread checkpoint I/O over a longer period
checkpoint_completion_target = 0.9

# Increase max_wal_size to reduce checkpoint frequency
max_wal_size = '4GB'

# Increase checkpoint_timeout
checkpoint_timeout = '15min'
```

```sql
-- Monitor checkpoint timing
SELECT
    checkpoints_timed,
    checkpoints_req,
    checkpoint_write_time / greatest(checkpoints_timed + checkpoints_req, 1) AS avg_write_ms,
    buffers_checkpoint / greatest(checkpoints_timed + checkpoints_req, 1) AS avg_buffers_per_checkpoint
FROM pg_stat_bgwriter;
```

## Part D: Monitoring Dashboard Design

### Dashboard Layout

```
+--------------------------+--------------------------+--------------------------+
|   Replication Lag (sec)  |   Replay Rate (MB/s)     |   WAL Generation (MB/s)  |
|   [Time series graph]    |   [Time series graph]    |   [Time series graph]    |
|   Threshold: 10s (warn)  |   Per-replica lines      |   On primary             |
|   Threshold: 30s (crit)  |                          |                          |
+--------------------------+--------------------------+--------------------------+
|   Replica Query Load     |   Network Throughput      |   Checkpoint Frequency   |
|   [Time series graph]    |   [Time series graph]    |   [Bar chart]            |
|   Active queries per     |   Bytes in/out per       |   Timed vs requested     |
|   replica                |   replica                |   checkpoints            |
+--------------------------+--------------------------+--------------------------+
|   Replication State      |   Slot Lag (bytes)       |   Connection Status      |
|   [Status indicators]    |   [Gauge per replica]    |   [Table: replica, state]|
|   streaming/catchup/     |                          |   WAL sender PIDs        |
|   non-streaming          |                          |                          |
+--------------------------+--------------------------+--------------------------+
```

### Prometheus Queries

```yaml
# Grafana panel queries

# Replication lag in seconds
- expr: pg_replication_lag_seconds
  legend: "{{ instance }}"

# Replay rate (bytes replayed per second)
- expr: rate(pg_wal_replay_bytes_total[1m])
  legend: "{{ instance }} replay rate"

# WAL generation rate on primary
- expr: rate(pg_wal_bytes_total{instance="primary"}[1m])
  legend: "WAL generation"

# Network throughput
- expr: rate(node_network_receive_bytes_total{device="eth0"}[1m])
  legend: "{{ instance }} RX"

# Active queries on replica
- expr: pg_stat_activity_count{state="active", instance=~"replica.*"}
  legend: "{{ instance }} active queries"

# Checkpoint requests vs timed
- expr: rate(pg_stat_bgwriter_checkpoints_req_total[5m])
  legend: "Requested checkpoints"
- expr: rate(pg_stat_bgwriter_checkpoints_timed_total[5m])
  legend: "Timed checkpoints"
```

### Alert Rules

```yaml
groups:
  - name: replication_alerts
    rules:
      - alert: ReplicationLagWarning
        expr: pg_replication_lag_seconds > 10
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "Replication lag > 10s on {{ $labels.instance }}"

      - alert: ReplicationLagCritical
        expr: pg_replication_lag_seconds > 30
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Replication lag > 30s on {{ $labels.instance }}"

      - alert: ReplicationBroken
        expr: pg_stat_wal_receiver_status != 1
        for: 30s
        labels:
          severity: critical
        annotations:
          summary: "WAL receiver not streaming on {{ $labels.instance }}"

      - alert: ReplicaBlockingReplay
        expr: pg_stat_activity_count{state="active", instance=~"replica.*"} > 5
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Many active queries on replica {{ $labels.instance }} -- may block replay"
```

## Common Mistakes to Avoid

1. **Treating all lag sources the same** -- bulk imports need different solutions than network latency
2. **Ignoring the replica's I/O capacity** -- replicas need the same disk performance as the primary
3. **Setting `max_standby_streaming_delay = -1` permanently** -- this prevents replay from ever catching up if a query runs indefinitely
4. **Not monitoring WAL generation rate** -- a sudden spike in WAL volume is the most common cause of lag
5. **Assuming more replicas solve lag** -- replicas share the same WAL stream; more replicas mean more WAL senders on the primary

## Key Takeaway

Replication lag is a symptom, not a disease. The diagnostic approach must identify the specific bottleneck: WAL generation rate, replica I/O, blocking queries, network bandwidth, or checkpoint pressure. Each cause has a targeted solution. The monitoring dashboard must show all five dimensions simultaneously so operators can correlate lag spikes with their root cause.
