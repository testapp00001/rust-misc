# Solution 02: Master-Slave Setup

---

## Part A: Primary Configuration

### PostgreSQL Configuration Changes

```ini
# postgresql.conf on primary

# WAL configuration for replication
wal_level = replica
max_wal_senders = 5
max_replication_slots = 3
wal_keep_size = '1GB'

# Enable hot standby feedback from replicas
hot_standby_feedback = on

# Shared preload libraries for monitoring
shared_preload_libraries = 'pg_stat_statements'
```

### Create Replication User

```sql
-- Create a dedicated replication user with minimal privileges
CREATE ROLE replicator WITH REPLICATION LOGIN PASSWORD 'strong_random_password_here';

-- Verify the user exists
SELECT rolname, rolreplication FROM pg_roles WHERE rolname = 'replicator';
```

### Create Replication Slots

```sql
-- Create a physical replication slot for each replica
SELECT * FROM pg_create_physical_replication_slot('replica_1_slot');
SELECT * FROM pg_create_physical_replication_slot('replica_2_slot');

-- Verify slots
SELECT slot_name, slot_type, active FROM pg_replication_slots;
```

### pg_hba.conf Changes

```ini
# Allow replication connections from replica hosts
host    replication    replicator    10.0.0.0/8    md5
```

### Why This Configuration Works

- `wal_level = replica` enables the primary to send WAL records that replicas can replay
- `max_wal_senders = 5` limits concurrent replication connections (1 per replica + buffer)
- `max_replication_slots = 3` prevents WAL cleanup on the primary before replicas consume it
- `wal_keep_size = 1GB` provides a fallback if replication slots fail
- The replication user has only the `REPLICATION` privilege -- it cannot read or write data

## Part B: Initialize the Replica

### Step 1: Take a Base Backup

```bash
# On the replica server, run pg_basebackup
pg_basebackup \
  -h primary.example.com \
  -D /var/lib/postgresql/15/main \
  -U replicator \
  -Fp \
  -Xs \
  -P \
  -R \
  --slot=replica_1_slot

# Flag explanations:
# -h: primary host
# -D: data directory on replica
# -U: replication user
# -Fp: plain format (not tar)
# -Xs: stream WAL during backup
# -P: show progress
# -R: create standby.signal and set primary_conninfo automatically
# --slot: use replication slot to prevent WAL removal
```

### Step 2: Verify standby.signal

```bash
# The -R flag creates this file automatically
cat /var/lib/postgresql/15/main/standby.signal
# Should be empty (its existence triggers standby mode)

# Check primary_conninfo was set
grep primary_conninfo /var/lib/postgresql/15/main/postgresql.auto.conf
# Should show: primary_conninfo = 'host=primary.example.com port=5432 user=replicator ...'
```

### Step 3: Start the Replica

```bash
# Start PostgreSQL on the replica
sudo systemctl start postgresql

# Verify it is in recovery mode
sudo -u postgres psql -c "SELECT pg_is_in_recovery();"
# Expected: t (true = replica/standby)

# Verify it is receiving WAL
sudo -u postgres psql -c "SELECT status FROM pg_stat_wal_receiver;"
# Expected: streaming
```

### Why This Works

`pg_basebackup -R` is the recommended way to initialize a replica. It:
1. Copies all data files from the primary (consistent snapshot)
2. Creates `standby.signal` to tell PostgreSQL to start in recovery mode
3. Sets `primary_conninfo` in `postgresql.auto.conf` with the connection string
4. Streams WAL during the backup to ensure no gaps

The replication slot ensures the primary does not remove WAL segments before the replica consumes them, even if the replica is temporarily disconnected.

## Part C: Replication Monitoring Queries

### On the Primary: Check Replication State

```sql
-- Replication state and lag for all replicas
SELECT
    client_addr,
    state,
    sent_lsn,
    write_lsn,
    flush_lsn,
    replay_lsn,
    pg_wal_lsn_diff(sent_lsn, replay_lsn) AS replay_lag_bytes,
    replay_lag
FROM pg_stat_replication;
```

### On the Replica: Check WAL Receiver Status

```sql
-- WAL receiver status
SELECT
    status,
    receive_start_lsn,
    received_lsn,
    last_msg_send_time,
    last_msg_receipt_time,
    pg_wal_lsn_diff(pg_last_wal_receive_lsn(), pg_last_wal_replay_lsn()) AS replay_lag_bytes
FROM pg_stat_wal_receiver;
```

### Alert Query: Lag Exceeds 10 Seconds

```sql
-- Run on primary, alert if any replica has lag > 10 seconds
SELECT
    client_addr,
    replay_lag
FROM pg_stat_replication
WHERE replay_lag > INTERVAL '10 seconds';
```

### Prometheus Alert Rule

```yaml
groups:
  - name: postgresql_replication
    rules:
      - alert: PostgreSQLReplicationLagHigh
        expr: pg_replication_lag_seconds > 10
        for: 1m
        labels:
          severity: warning
        annotations:
          summary: "PostgreSQL replication lag exceeds 10 seconds"
          description: "Replica {{ $labels.instance }} has {{ $value }}s of replication lag"
```

## Part D: Patroni Configuration

### Primary Patroni Configuration

```yaml
# /etc/patroni/patroni.yml (primary)

scope: postgres-cluster
name: pg-primary

restapi:
  listen: 0.0.0.0:8008
  connect_address: primary.example.com:8008

etcd3:
  hosts: etcd1.example.com:2379,etcd2.example.com:2379,etcd3.example.com:2379

bootstrap:
  dcs:
    ttl: 30
    loop_wait: 10
    retry_timeout: 30
    maximum_lag_on_failover: 1048576  # 1MB
    synchronous_mode: true
    postgresql:
      use_pg_rewind: true
      parameters:
        wal_level: replica
        max_wal_senders: 5
        max_replication_slots: 3
        hot_standby: "on"
        synchronous_commit: "on"
        synchronous_standby_names: "*"

postgresql:
  listen: 0.0.0.0:5432
  connect_address: primary.example.com:5432
  data_dir: /var/lib/postgresql/15/main
  authentication:
    replication:
      username: replicator
      password: strong_random_password_here
    superuser:
      username: postgres
      password: postgres_password_here

tags:
  nofailover: false
  noloadbalance: false
  clonefrom: false
```

### Replica Patroni Configuration

```yaml
# /etc/patroni/patroni.yml (replica)

scope: postgres-cluster
name: pg-replica-1

restapi:
  listen: 0.0.0.0:8008
  connect_address: replica1.example.com:8008

etcd3:
  hosts: etcd1.example.com:2379,etcd2.example.com:2379,etcd3.example.com:2379

postgresql:
  listen: 0.0.0.0:5432
  connect_address: replica1.example.com:5432
  data_dir: /var/lib/postgresql/15/main
  authentication:
    replication:
      username: replicator
      password: strong_random_password_here
    superuser:
      username: postgres
      password: postgres_password_here

tags:
  nofailover: false
  noloadbalance: false
  clonefrom: true
```

### Why Patroni Works for Automatic Failover

Patroni uses distributed consensus (etcd) to maintain cluster state:
1. The primary holds a leader lock in etcd with a TTL (30 seconds)
2. The primary updates the lock every `loop_wait` seconds (10 seconds)
3. If the primary fails to update the lock within the TTL, the lock expires
4. Replicas detect the expired lock and attempt to acquire it
5. The replica with the lowest lag acquires the lock and promotes itself
6. `synchronous_mode: true` ensures the promoted replica has zero data loss

The `maximum_lag_on_failover: 1048576` setting prevents a replica with more than 1MB of lag from being promoted, ensuring data integrity.

## Common Mistakes to Avoid

1. **Not creating replication slots** -- without them, the primary may delete WAL before the replica consumes it, breaking replication
2. **Using the `postgres` superuser for replication** -- create a dedicated user with only `REPLICATION` privilege
3. **Setting `max_wal_senders` too low** -- each replica needs one slot, and you need buffer for `pg_basebackup`
4. **Forgetting `pg_hba.conf`** -- replication connections use a separate authentication path from regular connections
5. **Not monitoring replication lag** -- silent replication failure means your "hot standby" may be hours behind

## Key Takeaway

PostgreSQL streaming replication is straightforward to set up: configure WAL on the primary, create a replication user, initialize the replica with `pg_basebackup -R`, and monitor with `pg_stat_replication`. Patroni adds automatic failover on top of this by using distributed consensus to detect primary failure and promote a replica. The key detail is ensuring the replica has zero lag before promotion, which requires synchronous replication or lag checks.
