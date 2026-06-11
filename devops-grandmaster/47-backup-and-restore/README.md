# Module 47: Backup & Restore

**Phase 6: Database Operations**
**Previous:** [Module 46: Replication](../46-replication/README.md)
**Next:** [Module 48: Disaster Recovery](../48-disaster-recovery/README.md)

---

## The Problem

A developer runs `DELETE FROM users;` in production. There is no WHERE clause. The command executes successfully. Replication faithfully copies the deletion to all replicas. Your monitoring alerts fire 5 minutes later. You have 5 minutes of user data gone, and no way to get it back without backups.

Or worse: ransomware encrypts your database files and all replicas. Your entire data infrastructure is locked.

Backups are your last line of defense. They are the difference between "minor incident" and "company-ending disaster."

---

## The Naive Way

**"pg_dump every night"**

```bash
# Cron job: dump database every night at 2 AM
0 2 * * * pg_dump mydb > /var/backups/mydb_$(date +\%Y\%m\%d).sql

# Store on the same server
ls /var/backups/
# mydb_20240101.sql
# mydb_20240102.sql
# mydb_20240103.sql
```

**Why this fails:**
- If the server dies, backups die with it (stored on same disk)
- Daily backups mean up to 24 hours of data loss
- No encryption (PII exposed if backup is stolen)
- No verification (you discover backups are corrupt when you need them)
- pg_dump locks tables during backup (downtime)
- No point-in-time recovery (you can only restore to 2 AM)
- Backup retention is unmanaged (disk fills up)

**"Just use filesystem snapshots"**

```bash
# Take LVM snapshot
lvcreate -L 10G -s -n mydb_snap /dev/vg0/mydb

# Copy snapshot
dd if=/dev/vg0/mydb_snap of=/backup/mydb.img

# Remove snapshot
lvremove /dev/vg0/mydb_snap
```

**Why this is incomplete:**
- Filesystem snapshots are not transactionally consistent
- Database may have dirty pages in memory not yet flushed to disk
- You need to flush and lock the database first (brief downtime)
- Restoring requires same filesystem layout

---

## The Right Way

### Backup Types

```
FULL BACKUP:
  ┌─────────────────────────────────────────┐
  │ Complete copy of entire database         │
  │ Size: Full database size                 │
  │ Restore time: Fastest                    │
  │ Backup time: Slowest                     │
  └─────────────────────────────────────────┘

INCREMENTAL BACKUP:
  ┌────────┐ ┌──────┐ ┌──────┐ ┌──────┐
  │  Full  │ │ Incr │ │ Incr │ │ Incr │
  │  Day 1 │ │ Day2 │ │ Day3 │ │ Day4 │
  └────────┘ └──────┘ └──────┘ └──────┘
  Only backs up changes since last backup
  Size: Smaller
  Restore: Need full + all incrementals

DIFFERENTIAL BACKUP:
  ┌────────┐ ┌──────┐ ┌────────┐ ┌──────────┐
  │  Full  │ │ Diff │ │  Diff  │ │   Diff   │
  │  Day 1 │ │ Day2 │ │  Day3  │ │   Day4   │
  └────────┘ └──────┘ └────────┘ └──────────┘
  Backs up changes since last FULL backup
  Size: Grows each day
  Restore: Need full + latest diff only
```

### PostgreSQL Backup Strategies

**Logical Backup (pg_dump):**

```bash
# Single database dump
pg_dump -h localhost -U postgres -d mydb \
  --format=custom \
  --compress=6 \
  --file=/backups/mydb_$(date +%Y%m%d_%H%M%S).dump

# Custom format advantages over plain SQL:
# - Compression built-in
# - Selective restore (specific tables)
# - Parallel restore capability
# - pg_restore for restoration

# Dump specific tables
pg_dump -h localhost -U postgres -d mydb \
  --table=users --table=orders \
  --format=custom \
  --file=/backups/tables_$(date +%Y%m%d).dump

# Parallel dump (all databases)
pg_dumpall -h localhost -U postgres \
  --file=/backups/all_databases_$(date +%Y%m%d).sql
```

**Physical Backup (pg_basebackup):**

```bash
# Physical backup (binary copy of data directory)
pg_basebackup \
  -h localhost \
  -D /backups/base_$(date +%Y%m%d_%H%M%S) \
  -U replicator \
  -Ft \           # tar format
  -z \            # gzip compression
  -Xs \           # stream WAL during backup
  -P              # show progress

# This creates a consistent snapshot of the entire cluster
# Includes all databases, roles, and configuration
```

### Point-in-Time Recovery (PITR)

```
TIMELINE:
  Full      WAL Archives
  Backup    ───────────────────────────────────▶
  ┌────┐   ┌───┬───┬───┬───┬───┬───┬───┬───┬───┐
  │ 2AM│   │2am│3am│4am│5am│6am│7am│8am│9am│10a│
  └────┘   └───┴───┴───┴───┴───┴───┴───┴───┴───┘
                       ▲                    ▲
                       │                    │
                  Accidental DELETE     Restore to
                  at 5:00 AM           this point (4:59 AM)
```

**WAL Archiving Configuration:**

```sql
-- postgresql.conf
wal_level = replica
archive_mode = on
archive_command = 'cp %p /var/lib/postgresql/wal_archive/%f'
-- Or for S3: 'aws s3 cp %p s3://my-bucket/wal/%f'

-- Alternatively, use pg_receivewal for streaming WAL archival
```

```bash
# pg_receivewal: stream WAL to archive in real-time
pg_receivewal \
  -h localhost \
  -U replicator \
  -D /var/lib/postgresql/wal_archive \
  --no-loop \
  -v

# This is safer than archive_command because:
# - No shell command injection risk
# - Confirms WAL segment is fully written
# - Can compress on the fly with -Z
```

### Automated Backup Script

```bash
#!/bin/bash
# backup_postgres.sh - Automated PostgreSQL backup with PITR

set -euo pipefail

# Configuration
BACKUP_DIR="/var/backups/postgresql"
WAL_ARCHIVE="/var/lib/postgresql/wal_archive"
S3_BUCKET="s3://my-company-backups/postgresql"
RETENTION_DAYS=30
PGHOST="localhost"
PGUSER="backup_user"
PGDATABASE="myapp"
LOG_FILE="/var/log/postgresql-backup.log"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOG_FILE"
}

error() {
    log "ERROR: $*"
    exit 1
}

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Step 1: Take base backup
BACKUP_NAME="base_$(date +%Y%m%d_%H%M%S)"
BACKUP_PATH="$BACKUP_DIR/$BACKUP_NAME"

log "Starting base backup: $BACKUP_NAME"

pg_basebackup \
    -h "$PGHOST" \
    -U "$PGUSER" \
    -D "$BACKUP_PATH" \
    -Ft \
    -z \
    -Xs \
    -P \
    --checkpoint=fast \
    --label="$BACKUP_NAME" \
    || error "pg_basebackup failed"

log "Base backup complete: $BACKUP_PATH"

# Step 2: Upload to S3
log "Uploading to S3..."
aws s3 sync "$BACKUP_PATH" "$S3_BUCKET/$BACKUP_NAME/" \
    --storage-class STANDARD_IA \
    || error "S3 upload failed"

log "S3 upload complete"

# Step 3: Upload WAL archives
log "Uploading WAL archives..."
aws s3 sync "$WAL_ARCHIVE" "$S3_BUCKET/wal/" \
    --storage-class STANDARD_IA \
    || error "WAL upload failed"

# Step 4: Verify backup integrity
log "Verifying backup..."
tar -tzf "$BACKUP_PATH/base.tar.gz" > /dev/null \
    || error "Backup verification failed"

log "Backup verified successfully"

# Step 5: Clean up old backups (retention policy)
log "Cleaning up backups older than $RETENTION_DAYS days..."
find "$BACKUP_DIR" -type d -name "base_*" -mtime +$RETENTION_DAYS -exec rm -rf {} + 2>/dev/null || true

# Clean up old WAL archives
find "$WAL_ARCHIVE" -type f -mtime +$RETENTION_DAYS -delete 2>/dev/null || true

log "Cleanup complete"

# Step 6: Record backup metadata
cat > "$BACKUP_PATH/metadata.json" <<EOF
{
    "backup_name": "$BACKUP_NAME",
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "type": "base",
    "pg_version": "$(pg_config --version)",
    "size_bytes": $(du -sb "$BACKUP_PATH" | cut -f1),
    "wal_position": "$(psql -h "$PGHOST" -U "$PGUSER" -d "$PGDATABASE" -t -c "SELECT pg_current_wal_lsn()")"
}
EOF

aws s3 cp "$BACKUP_PATH/metadata.json" "$S3_BUCKET/$BACKUP_NAME/metadata.json"

log "Backup process complete: $BACKUP_NAME"
```

### Backup Verification (Test Restores!)

```bash
#!/bin/bash
# verify_backup.sh - Actually restore a backup to verify it works

set -euo pipefail

BACKUP_PATH="$1"
VERIFY_PORT=15432
VERIFY_DIR="/tmp/backup_verify_$$"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"
}

cleanup() {
    log "Cleaning up verification environment..."
    pg_ctl -D "$VERIFY_DIR" stop -m fast 2>/dev/null || true
    rm -rf "$VERIFY_DIR"
}

trap cleanup EXIT

# Step 1: Extract backup
log "Extracting backup..."
mkdir -p "$VERIFY_DIR"
tar -xzf "$BACKUP_PATH/base.tar.gz" -C "$VERIFY_DIR"

# Step 2: Configure for verification
log "Configuring verification instance..."
cat > "$VERIFY_DIR/postgresql.conf" <<EOF
port = $VERIFY_PORT
listen_addresses = 'localhost'
logging_collector = off
EOF

touch "$VERIFY_DIR/standby.signal"  # Remove if not standby

# Step 3: Start verification instance
log "Starting verification PostgreSQL..."
pg_ctl -D "$VERIFY_DIR" -l "$VERIFY_DIR/verify.log" start

# Wait for startup
sleep 5

# Step 4: Run verification queries
log "Running verification queries..."

# Check database exists
psql -h localhost -p $VERIFY_PORT -U postgres -c "\l" || {
    log "FAIL: Cannot list databases"
    exit 1
}

# Check tables exist
psql -h localhost -p $VERIFY_PORT -U postgres -d myapp -c "\dt" || {
    log "FAIL: Cannot list tables"
    exit 1
}

# Check row counts
psql -h localhost -p $VERIFY_PORT -U postgres -d myapp -c "
SELECT 'users' as table_name, count(*) FROM users
UNION ALL
SELECT 'orders', count(*) FROM orders;
" || {
    log "FAIL: Cannot query tables"
    exit 1
}

# Check data integrity
psql -h localhost -p $VERIFY_PORT -U postgres -d myapp -c "
SELECT
    count(*) as total_users,
    count(DISTINCT email) as unique_emails,
    min(created_at) as earliest,
    max(created_at) as latest
FROM users;
" || {
    log "FAIL: Data integrity check failed"
    exit 1
}

log "SUCCESS: Backup verification passed!"
```

### Backup Encryption

```bash
# Encrypt backup before uploading
gpg --symmetric --cipher-algo AES256 \
    --output "$BACKUP_PATH.tar.gz.gpg" \
    "$BACKUP_PATH.tar.gz"

# Or use age (modern alternative to GPG)
age -p -o "$BACKUP_PATH.tar.gz.age" "$BACKUP_PATH.tar.gz"

# AWS S3: Use server-side encryption
aws s3 cp "$BACKUP_PATH.tar.gz" "s3://my-bucket/backups/" \
    --sse aws:kms \
    --sse-kms-key-id alias/backup-key
```

---

## The Production Way

### Retention Policies

```
RETENTION STRATEGY:

  Daily backups:   Keep 7 days
  Weekly backups:  Keep 4 weeks
  Monthly backups: Keep 12 months
  Yearly backups:  Keep 7 years (compliance)

  Timeline:
  ┌───┬───┬───┬───┬───┬───┬───┐
  │Mon│Tue│Wed│Thu│Fri│Sat│Sun│  Week 1: keep all
  └───┴───┴───┴───┴───┴───┴───┘
  ┌───┬───┬───┬───┬───┬───┬───┐
  │Mon│Tue│Wed│Thu│Fri│Sat│Sun│  Week 2: keep all
  └───┴───┴───┴───┴───┴───┴───┘
  ...
  ┌───┐
  │Mon│  Week 5+: keep Monday only (weekly)
  └───┘
```

### Backup Monitoring

```python
# backup_monitor.py
import boto3
from datetime import datetime, timedelta

def check_backup_freshness(bucket, prefix, max_age_hours=26):
    """Alert if latest backup is too old."""
    s3 = boto3.client('s3')

    response = s3.list_objects_v2(
        Bucket=bucket,
        Prefix=prefix,
        MaxKeys=10
    )

    if 'Contents' not in response:
        raise Alert("No backups found!")

    latest = max(response['Contents'], key=lambda x: x['LastModified'])
    age = datetime.now(latest['LastModified'].tzinfo) - latest['LastModified']

    if age > timedelta(hours=max_age_hours):
        raise Alert(f"Backup is {age.total_seconds()/3600:.1f} hours old "
                    f"(max: {max_age_hours}h)")

    return {
        'latest_backup': latest['Key'],
        'age_hours': age.total_seconds() / 3600,
        'size_mb': latest['Size'] / 1024 / 1024
    }

def verify_restore_procedure(backup_key):
    """Periodically test restore to a temporary instance."""
    # Download backup
    # Start temporary PostgreSQL
    # Restore
    # Run verification queries
    # Report success/failure
    pass
```

### Restoring from PITR

```bash
#!/bin/bash
# restore_pitr.sh - Point-in-time recovery

RESTORE_DIR="/var/lib/postgresql/16/restore"
BASE_BACKUP="/backups/base_20240101_020000"
WAL_ARCHIVE="/backups/wal"
RECOVERY_TARGET_TIME="2024-01-15 05:00:00"

# Step 1: Stop PostgreSQL
systemctl stop postgresql

# Step 2: Clear data directory
rm -rf "$RESTORE_DIR"/*

# Step 3: Extract base backup
tar -xzf "$BASE_BACKUP/base.tar.gz" -C "$RESTORE_DIR"

# Step 4: Create recovery configuration
cat > "$RESTORE_DIR/postgresql.conf" <<EOF
# Recovery settings
restore_command = 'cp $WAL_ARCHIVE/%f %p'
recovery_target_time = '$RECOVERY_TARGET_TIME'
recovery_target_action = 'promote'
EOF

# Step 5: Create recovery signal
touch "$RESTORE_DIR/recovery.signal"

# Step 6: Start PostgreSQL (will enter recovery mode)
systemctl start postgresql

# Step 7: Monitor recovery progress
tail -f /var/log/postgresql/postgresql-16-main.log

# After recovery completes, verify data
psql -U postgres -d myapp -c "SELECT max(created_at) FROM orders;"
```

---

## Hands-On Lab

### Lab: Automated PostgreSQL Backups with PITR

**Objective:** Set up a complete backup system with WAL archiving, automated backups, and point-in-time recovery.

**Prerequisites:** Docker, docker-compose

**Step 1: Create the environment**

```yaml
# docker-compose.yml
version: '3.8'

services:
  postgres:
    image: postgres:16
    container_name: postgres-backup-lab
    environment:
      POSTGRES_DB: myapp
      POSTGRES_USER: admin
      POSTGRES_PASSWORD: adminpass
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data
      - wal_archive:/var/lib/postgresql/wal_archive
      - backups:/var/backups
      - ./init.sql:/docker-entrypoint-initdb.d/init.sql
      - ./postgresql.conf:/etc/postgresql/postgresql.conf
    command: postgres -c config_file=/etc/postgresql/postgresql.conf

  minio:
    image: minio/minio
    container_name: minio-backup
    environment:
      MINIO_ROOT_USER: minioadmin
      MINIO_ROOT_PASSWORD: minioadmin
    ports:
      - "9000:9000"
      - "9001:9001"
    volumes:
      - minio_data:/data
    command: server /data --console-address ":9001"

volumes:
  pgdata:
  wal_archive:
  backups:
  minio_data:
```

**Step 2: Configure PostgreSQL for WAL archiving**

```ini
# postgresql.conf
listen_addresses = '*'
max_connections = 100

# WAL settings
wal_level = replica
archive_mode = on
archive_command = 'cp %p /var/lib/postgresql/wal_archive/%f'
max_wal_senders = 3
max_replication_slots = 3

# Checkpoint settings (for backup performance)
checkpoint_timeout = 30min
checkpoint_completion_target = 0.9

# Logging
log_destination = 'stderr'
logging_collector = on
log_directory = 'log'
log_filename = 'postgresql-%Y-%m-%d.log'
log_statement = 'ddl'
log_min_duration_statement = 1000
```

**Step 3: Initialize with sample data**

```sql
-- init.sql
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

CREATE TABLE customers (
    id SERIAL PRIMARY KEY,
    name VARCHAR(200),
    email VARCHAR(255) UNIQUE,
    plan VARCHAR(50) DEFAULT 'free',
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    customer_id INT REFERENCES customers(id),
    amount DECIMAL(12,2),
    currency VARCHAR(3) DEFAULT 'USD',
    status VARCHAR(50),
    created_at TIMESTAMP DEFAULT NOW()
);

-- Generate sample data
INSERT INTO customers (name, email, plan)
SELECT
    'Customer ' || i,
    'customer' || i || '@example.com',
    CASE (i % 3)
        WHEN 0 THEN 'free'
        WHEN 1 THEN 'pro'
        WHEN 2 THEN 'enterprise'
    END
FROM generate_series(1, 1000) AS i;

INSERT INTO transactions (customer_id, amount, status, created_at)
SELECT
    (random() * 999 + 1)::int,
    (random() * 1000 + 1)::decimal(12,2),
    CASE (random() * 3)::int
        WHEN 0 THEN 'completed'
        WHEN 1 THEN 'pending'
        WHEN 2 THEN 'failed'
    END,
    NOW() - (random() * interval '30 days')
FROM generate_series(1, 10000) AS i;

-- Create backup user
CREATE ROLE backup_user WITH LOGIN PASSWORD 'backup_pass';
GRANT CONNECT ON DATABASE myapp TO backup_user;
GRANT USAGE ON SCHEMA public TO backup_user;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO backup_user;
```

**Step 4: Start and verify**

```bash
# Start environment
docker-compose up -d

# Wait for initialization
sleep 10

# Verify data
docker exec postgres-backup-lab psql -U admin -d myapp -c \
    "SELECT count(*) FROM customers;"

# Verify WAL archiving is active
docker exec postgres-backup-lab psql -U admin -d myapp -c \
    "SHOW archive_mode; SHOW wal_level;"

# Check WAL archive directory
docker exec postgres-backup-lab ls -la /var/lib/postgresql/wal_archive/
```

**Step 5: Take a backup**

```bash
# Take base backup
docker exec postgres-backup-lab bash -c "
    mkdir -p /var/backups/base
    pg_basebackup -h localhost -U backup_user -D /var/backups/base/current \
        -Ft -z -Xs -P --checkpoint=fast
"

# Record the WAL position
docker exec postgres-backup-lab psql -U admin -d myapp -c \
    "SELECT pg_current_wal_lsn();"

# Note the LSN value (e.g., 0/1A000000)
```

**Step 6: Simulate data loss and recover**

```bash
# Record current time
RECOVERY_TIME=$(date -u +"%Y-%m-%d %H:%M:%S")
echo "Time before disaster: $RECOVERY_TIME"

# Insert some "future" data
docker exec postgres-backup-lab psql -U admin -d myapp -c "
INSERT INTO customers (name, email, plan)
VALUES ('Important Customer', 'important@example.com', 'enterprise');

INSERT INTO transactions (customer_id, amount, status)
VALUES (1001, 50000.00, 'completed');
"

# Verify the data exists
docker exec postgres-backup-lab psql -U admin -d myapp -c \
    "SELECT * FROM customers WHERE email = 'important@example.com';"

# Wait a moment for WAL to be archived
sleep 5

# SIMULATE DISASTER: Drop the table
docker exec postgres-backup-lab psql -U admin -d myapp -c \
    "DROP TABLE transactions CASCADE;"

# Verify data is gone
docker exec postgres-backup-lab psql -U admin -d myapp -c \
    "\dt"
```

**Step 7: Perform PITR**

```bash
# Stop PostgreSQL
docker exec postgres-backup-lab pg_ctl -D /var/lib/postgresql/data stop -m fast

# Clear data directory
docker exec postgres-backup-lab bash -c "
    rm -rf /var/lib/postgresql/data/*
    tar -xzf /var/backups/base/current/base.tar.gz -C /var/lib/postgresql/data/
"

# Configure recovery
docker exec postgres-backup-lab bash -c "
cat > /var/lib/postgresql/data/postgresql.auto.conf <<EOF
restore_command = 'cp /var/lib/postgresql/wal_archive/%f %p'
recovery_target_time = '$RECOVERY_TIME'
recovery_target_action = 'promote'
EOF

touch /var/lib/postgresql/data/recovery.signal
"

# Restart PostgreSQL
docker exec postgres-backup-lab pg_ctl -D /var/lib/postgresql/data start

# Wait for recovery
sleep 10

# Verify data is restored
docker exec postgres-backup-lab psql -U admin -d myapp -c "\dt"
docker exec postgres-backup-lab psql -U admin -d myapp -c \
    "SELECT * FROM customers WHERE email = 'important@example.com';"
docker exec postgres-backup-lab psql -U admin -d myapp -c \
    "SELECT count(*) FROM transactions;"
```

---

## Limitation

Backups protect your data, but they do not protect your **uptime**. Restoring a large database from backup can take hours. During that time, your application is down. Users cannot access the system. Revenue is lost.

This is the difference between:
- **RPO (Recovery Point Objective):** How much data can you lose? (Backups define this)
- **RTO (Recovery Time Objective):** How long can you be down? (Restore speed defines this)

To minimize RTO, you need automatic failover, standby systems, and a tested disaster recovery plan.

**Next:** [Module 48: Disaster Recovery](../48-disaster-recovery/README.md) — We will cover RPO/RTO planning, failover procedures, and how to build systems that survive catastrophic failures.
