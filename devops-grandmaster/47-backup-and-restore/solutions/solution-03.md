# Solution 03: Point-in-Time Recovery Execution

## Part A: Recovery Target Identification

### Base Backup Selection

```
Available backups:
  - 2024-03-15 02:00:00 UTC (full physical backup, completed 02:15:00 UTC)
  - 2024-03-14 02:00:00 UTC (full physical backup, completed 02:15:00 UTC)

Selected: 2024-03-15 02:00:00 UTC
Reason: Most recent backup before the corruption event at 14:35:22 UTC.
```

### WAL Segments Needed

The recovery needs to replay WAL from 02:15:00 UTC (backup end) to 14:34:00 UTC (target time).

```sql
-- On the primary (before failure), find the WAL segment for the target time
SELECT
    pg_walfile_name(pg_current_wal_lsn()) AS current_wal_segment,
    pg_current_wal_lsn() AS current_lsn;
```

WAL segments from the archive directory:
```bash
ls -la /var/lib/postgresql/wal_archive/ | grep "2024-03-15"
# Segments from 02:15 to 14:34 are needed
```

### Recovery Target Time

```
recovery_target_time = '2024-03-15 14:34:00+00'
```

## Part B: Recovery Configuration

### Step 1: Stop the Corrupted Database

```bash
# Stop PostgreSQL immediately to prevent further writes
sudo systemctl stop postgresql

# Verify it is stopped
sudo systemctl status postgresql
# Expected: inactive (dead)
```

### Step 2: Restore the Base Backup

```bash
# Move the corrupted data directory aside (do NOT delete yet)
sudo mv /var/lib/postgresql/15/main /var/lib/postgresql/15/main.corrupted

# Restore the base backup
sudo mkdir -p /var/lib/postgresql/15/main
sudo tar -xzf /var/lib/postgresql/backups/physical/base_20240315_020000.tar.gz \
    -C /var/lib/postgresql/15/main/

# Set correct ownership
sudo chown -R postgres:postgres /var/lib/postgresql/15/main
```

### Step 3: Configure Recovery

```bash
# Add recovery settings to postgresql.auto.conf
cat >> /var/lib/postgresql/15/main/postgresql.auto.conf << 'EOF'

# PITR Recovery Configuration
restore_command = 'cp /var/lib/postgresql/wal_archive/%f %p'
recovery_target_time = '2024-03-15 14:34:00+00'
recovery_target_action = 'pause'
EOF
```

### Step 4: Create recovery.signal

```bash
# Create the recovery signal file
# PostgreSQL 12+ uses this file instead of recovery.conf
sudo -u postgres touch /var/lib/postgresql/15/main/recovery.signal
```

### Why This Configuration Works

- `restore_command`: Tells PostgreSQL where to find archived WAL segments. It copies each segment from the archive to the WAL directory. The command must return 0 on success.
- `recovery_target_time`: PostgreSQL replays WAL until it reaches a commit timestamp before 14:34:00, then stops. It does NOT stop mid-transaction -- it stops at a consistent point.
- `recovery_target_action = 'pause'`: After reaching the target, PostgreSQL pauses in recovery mode. This allows you to verify the data before promoting.
- `recovery.signal`: An empty file that tells PostgreSQL to enter recovery mode instead of starting normally.

## Part C: Execute Recovery and Verify

### Step 5: Start PostgreSQL in Recovery Mode

```bash
# Start PostgreSQL -- it will enter recovery mode automatically
sudo systemctl start postgresql

# Verify it is in recovery mode
sudo -u postgres psql -c "SELECT pg_is_in_recovery();"
# Expected: t (true = in recovery)

# Monitor recovery progress
sudo -u postgres psql -c "
    SELECT
        pg_last_wal_receive_lsn() AS received_lsn,
        pg_last_wal_replay_lsn() AS replayed_lsn,
        CASE WHEN pg_last_wal_receive_lsn() = pg_last_wal_replay_lsn()
             THEN 'recovery complete'
             ELSE 'recovery in progress'
        END AS status;
"
```

### Step 6: Verify Data Integrity

```bash
# Check the customers table is intact
sudo -u postgres psql -c "SELECT count(*) FROM customers;"
# Compare with expected count (from monitoring or application logs)

# Check the deleted rows are restored
sudo -u postgres psql -c "
    SELECT id, name, email, created_at
    FROM customers
    ORDER BY created_at DESC
    LIMIT 10;
"

# Verify the deletion event did NOT happen
sudo -u postgres psql -c "
    SELECT count(*)
    FROM pg_stat_activity
    WHERE query LIKE '%DELETE FROM customers%';
"
# Expected: 0 (the deletion was rolled back via PITR)

# Check for data consistency
sudo -u postgres psql -c "
    SELECT
        schemaname,
        relname,
        n_live_tup,
        n_dead_tup
    FROM pg_stat_user_tables
    WHERE relname = 'customers';
"
```

### Step 7: Promote to Primary (If Replacing Original)

```bash
# If this recovered instance will replace the original primary:
sudo -u postgres psql -c "SELECT pg_promote();"

# Verify promotion
sudo -u postgres psql -c "SELECT pg_is_in_recovery();"
# Expected: f (false = now a primary)

# Test write capability
sudo -u postgres psql -c "
    CREATE TABLE IF NOT EXISTS pitr_test (id serial, ts timestamp);
    INSERT INTO pitr_test (ts) VALUES (now());
    SELECT * FROM pitr_test;
    DROP TABLE pitr_test;
"
```

### Step 8: Reconfigure Application Connection

```bash
# Update PgBouncer to point to the recovered instance
sudo sed -i 's/host=primary-old/host=primary-recovered/' /etc/pgbouncer/pgbouncer.ini

# Reload PgBouncer
psql -h localhost -p 6432 -U admin pgbouncer -c "RELOAD;"

# Verify application connectivity
curl -s http://localhost:8080/health | jq .
```

## Part D: Recovery Runbook

### Runbook: Point-in-Time Recovery for Data Corruption

**Trigger:** Data corruption confirmed (e.g., accidental DELETE, DROP TABLE, or bad migration)

**Prerequisites:**
- Access to the PostgreSQL server (SSH)
- Access to backup storage (local and/or S3)
- Knowledge of the exact corruption time (from application logs or audit trail)

**Estimated Duration:** 15-30 minutes (depending on database size and WAL volume)

---

**Step 1: Identify the target time (2 minutes)**

```bash
# Find the exact time of the corruption
# Check PostgreSQL logs
sudo grep -i "delete\|drop\|truncate" /var/log/postgresql/postgresql-*.log | tail -5

# Or check application logs
sudo grep -i "DELETE FROM customers" /var/log/app/audit.log

# Set the target time to 1 minute before the corruption
TARGET_TIME="2024-03-15 14:34:00+00"
echo "Recovery target: $TARGET_TIME"
```

**Step 2: Stop the database (1 minute)**

```bash
sudo systemctl stop postgresql
echo "PostgreSQL stopped at $(date -u +%Y-%m-%dT%H:%M:%SZ)"
```

**Step 3: Preserve the corrupted data (1 minute)**

```bash
# Move aside (do NOT delete until recovery is verified)
sudo mv /var/lib/postgresql/15/main /var/lib/postgresql/15/main.corrupted.$(date +%Y%m%d_%H%M%S)
echo "Corrupted data preserved"
```

**Step 4: Restore base backup (5-15 minutes)**

```bash
# Find the most recent backup before the corruption
BACKUP_FILE=$(ls -t /var/lib/postgresql/backups/physical/base_*.tar.gz | head -1)
echo "Using backup: $BACKUP_FILE"

# Restore
sudo mkdir -p /var/lib/postgresql/15/main
sudo tar -xzf "$BACKUP_FILE" -C /var/lib/postgresql/15/main/
sudo chown -R postgres:postgres /var/lib/postgresql/15/main
echo "Base backup restored"
```

**Step 5: Configure recovery (1 minute)**

```bash
cat >> /var/lib/postgresql/15/main/postgresql.auto.conf << EOF
restore_command = 'cp /var/lib/postgresql/wal_archive/%f %p'
recovery_target_time = '$TARGET_TIME'
recovery_target_action = 'pause'
EOF

sudo -u postgres touch /var/lib/postgresql/15/main/recovery.signal
echo "Recovery configured for $TARGET_TIME"
```

**Step 6: Start and verify (3-5 minutes)**

```bash
sudo systemctl start postgresql
sleep 5

# Check recovery status
sudo -u postgres psql -c "SELECT pg_is_in_recovery();"
# Should return 't'

# Wait for recovery to complete
while sudo -u postgres psql -t -c "SELECT pg_is_in_recovery();" | grep -q "t"; do
    echo "Recovery in progress..."
    sleep 5
done

# Verify data
sudo -u postgres psql -c "SELECT count(*) FROM customers;"
echo "Recovery complete. Verify row count matches expected value."
```

**Step 7: Promote and reconnect (2 minutes)**

```bash
sudo -u postgres psql -c "SELECT pg_promote();"
sleep 2

# Verify writes work
sudo -u postgres psql -c "INSERT INTO pitr_test VALUES (1, now());"

# Reconnect application
psql -h localhost -p 6432 -U admin pgbouncer -c "RELOAD;"
echo "Application reconnected."
```

**Verification Checklist:**
- [ ] Customers table row count matches expected value
- [ ] Recent orders are intact
- [ ] Application health check passes
- [ ] No error logs in PostgreSQL or application

**Rollback Plan (if recovery fails):**
```bash
# Stop the failed recovery
sudo systemctl stop postgresql

# Restore the corrupted data directory
sudo mv /var/lib/postgresql/15/main.corrupted.* /var/lib/postgresql/15/main

# Start the original (corrupted) database
sudo systemctl start postgresql

# Investigate the failure and try again
```

## Common Mistakes to Avoid

- Deleting the corrupted data directory before verifying the recovery -- always preserve it until recovery is confirmed
- Forgetting `recovery.signal` -- without it, PostgreSQL starts normally and does not replay WAL
- Setting `recovery_target_action = 'promote'` without verification -- you cannot go back after promotion
- Not checking that all WAL segments are available in the archive -- missing segments cause recovery to stop early
- Using the wrong timezone in `recovery_target_time` -- PostgreSQL uses the server's timezone; ensure the target time matches

## Key Takeaway

Point-in-time recovery is a precise operation: restore the base backup, configure the recovery target, and let PostgreSQL replay WAL to the target time. The key detail is that PostgreSQL stops at a consistent point (a committed transaction), not mid-transaction. Always verify the data before promoting, and always preserve the corrupted data until recovery is confirmed successful.
