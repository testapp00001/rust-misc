# Solution 05: Complete Backup and DR Plan

## Part A: RPO and RTO Requirements

| Data Classification | RPO | RTO | Cost of Not Meeting | Technology |
|--------------------|-----|-----|---------------------|------------|
| Customer PII | 0 | < 1 min | Regulatory fines, lawsuits | Synchronous replication + encryption at rest |
| Order data | 0 | < 1 min | Revenue loss ($5,700/min), customer churn | Synchronous replication + hot standby |
| Product catalog | < 1 hour | < 15 min | Stale product info, lost sales | WAL archiving + pg_basebackup |
| Session data | < 5 min | < 5 min | Users must re-login, abandoned carts | Async streaming replication |
| Analytics | < 24 hours | < 1 hour | Delayed reporting, no business impact | Daily pg_dump |

### Cost Justification

```
Annual revenue: $50 million
Revenue per minute: $50,000,000 / 525,600 = $95.12/minute

Cost of 1-hour outage: $95.12 * 60 = $5,707

DR investment justification:
- Synchronous replication (2 servers): ~$50,000/year
- If it prevents just 9 hours of downtime per year, it pays for itself
- 9 hours * $5,707/hour = $51,363 saved
```

## Part B: Backup Architecture

```
+----------------------------------------------------------+
|                    PRODUCTION PRIMARY                     |
|                    (PostgreSQL 15)                        |
|                    500GB, 24/7                             |
+----------------------------------------------------------+
        |                    |                    |
        | WAL streaming      | WAL archiving      | pg_dump
        | (continuous)       | (continuous)        | (daily)
        v                    v                    v
+---------------+    +---------------+    +---------------+
| SYNC REPLICA  |    | WAL ARCHIVE   |    | LOGICAL BACKUP|
| (hot standby) |    | (local + S3)  |    | (local + S3)  |
| Same AZ       |    | /wal_archive/ |    | /backups/     |
+---------------+    +-------+-------+    +-------+-------+
        |                    |                    |
        | Promotion          | S3 upload          | S3 upload
        | (automatic)        | (encrypted)        | (encrypted)
        v                    v                    v
+---------------+    +---------------+    +---------------+
| FAILOVER      |    | S3 WAL BUCKET |    | S3 BACKUP     |
| (Patroni)     |    | (KMS enc.)    |    | BUCKET        |
| etcd consensus|    | 90-day life   |    | (KMS enc.)    |
+---------------+    +---------------+    | 30-day life   |
                                          +---------------+
                                                  |
                                                  v
                                          +---------------+
                                          | CROSS-REGION  |
                                          | REPLICATION   |
                                          | (S3 CRR)      |
                                          +---------------+
```

### Layer 1: Physical Backup (pg_basebackup)

```bash
#!/bin/bash
# backup_physical.sh -- Daily physical backup with WAL archiving

set -euo pipefail

BACKUP_DIR="/var/lib/postgresql/backups/physical"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_PATH="${BACKUP_DIR}/base_${TIMESTAMP}"
RETENTION_DAYS=7

mkdir -p "$BACKUP_DIR"

echo "Starting physical backup at $(date -u +%Y-%m-%dT%H:%M:%SZ)..."
START=$SECONDS

# Take base backup with WAL streaming
pg_basebackup \
    -h localhost \
    -D "$BACKUP_PATH" \
    -U replicator \
    -Fp \
    -Xs \
    -P \
    --checkpoint=fast \
    --label="backup_${TIMESTAMP}"

DURATION=$((SECONDS - START))
SIZE=$(du -sh "$BACKUP_PATH" | cut -f1)

echo "Physical backup completed: ${SIZE} in ${DURATION}s"

# Compress
tar -czf "${BACKUP_PATH}.tar.gz" -C "$BACKUP_DIR" "base_${TIMESTAMP}"
rm -rf "$BACKUP_PATH"

# Checksum
sha256sum "${BACKUP_PATH}.tar.gz" > "${BACKUP_PATH}.tar.gz.sha256"

# Upload to S3
aws s3 cp "${BACKUP_PATH}.tar.gz" "s3://my-bucket/physical/" \
    --sse aws:kms --sse-kms-key-id alias/backup-key

aws s3 cp "${BACKUP_PATH}.tar.gz.sha256" "s3://my-bucket/physical/" \
    --sse aws:kms --sse-kms-key-id alias/backup-key

# Cleanup local old backups
find "$BACKUP_DIR" -name "base_*.tar.gz" -mtime +${RETENTION_DAYS} -delete
find "$BACKUP_DIR" -name "base_*.sha256" -mtime +${RETENTION_DAYS} -delete

# Write status for monitoring
cat > "${BACKUP_DIR}/last_status.json" << EOF
{
    "timestamp": $(date +%s),
    "duration": $DURATION,
    "size": $(stat -c%s "${BACKUP_PATH}.tar.gz"),
    "file": "${BACKUP_PATH}.tar.gz",
    "verify_pass": false
}
EOF

echo "Physical backup pipeline complete."
```

### Layer 2: WAL Archiving

```ini
# postgresql.conf
archive_mode = on
archive_command = '/usr/local/bin/archive_wal.sh %p %f'
archive_timeout = 60
max_wal_senders = 5
wal_level = replica
```

### Layer 3: Encryption

```bash
#!/bin/bash
# encrypt_backup.sh -- Encrypt backup with GPG before upload

BACKUP_FILE="$1"
GPG_RECIPIENT="backup@company.com"

# Encrypt
gpg --batch --yes --recipient "$GPG_RECIPIENT" \
    --output "${BACKUP_FILE}.gpg" \
    --encrypt "$BACKUP_FILE"

# Upload encrypted file
aws s3 cp "${BACKUP_FILE}.gpg" "s3://my-bucket/encrypted/" \
    --sse aws:kms

# Remove unencrypted local file
rm -f "$BACKUP_FILE"
```

## Part C: Complete Automation

### systemd Timer (replaces cron for better logging)

```ini
# /etc/systemd/system/pg-backup-physical.timer
[Unit]
Description=PostgreSQL Physical Backup Timer

[Timer]
OnCalendar=*-*-* 02:00:00
Persistent=true

[Install]
WantedBy=timers.target
```

```ini
# /etc/systemd/system/pg-backup-physical.service
[Unit]
Description=PostgreSQL Physical Backup
After=postgresql.service

[Service]
Type=oneshot
User=postgres
ExecStart=/usr/local/bin/backup_physical.sh
StandardOutput=journal
StandardError=journal
```

### S3 Lifecycle Policy

```json
{
    "Rules": [
        {
            "ID": "wal-90-day",
            "Status": "Enabled",
            "Filter": {"Prefix": "wal-archive/"},
            "Expiration": {"Days": 90}
        },
        {
            "ID": "backups-30-day",
            "Status": "Enabled",
            "Filter": {"Prefix": "physical/"},
            "Expiration": {"Days": 30}
        },
        {
            "ID": "backups-90-day-cross-region",
            "Status": "Enabled",
            "Filter": {"Prefix": "cross-region/"},
            "Expiration": {"Days": 90}
        }
    ]
}
```

### Cross-Region Replication

```bash
# Enable S3 Cross-Region Replication on the backup bucket
aws s3api put-bucket-replication \
    --bucket my-backup-bucket \
    --replication-configuration '{
        "Role": "arn:aws:iam::role/s3-replication-role",
        "Rules": [
            {
                "Status": "Enabled",
                "Destination": {
                    "Bucket": "arn:aws:s3:::my-backup-bucket-dr",
                    "StorageClass": "STANDARD_IA"
                }
            }
        ]
    }'
```

## Part D: Recovery Runbooks

### Runbook 1: Single Table Recovery

**Trigger:** Developer accidentally drops or corrupts a single table

```bash
# Step 1: Identify the table and the time of corruption
TABLE_NAME="customers"
CORRUPTION_TIME="2024-03-15 14:35:00"
TARGET_TIME="2024-03-15 14:34:00"

# Step 2: Start a temporary PostgreSQL instance
docker run -d --name pg_restore_temp \
    -e POSTGRES_PASSWORD=temp \
    -p 15432:5432 \
    postgres:15

# Step 3: Restore backup to temporary instance
pg_restore -h localhost -p 15432 -U postgres -d postgres \
    --data-only --table="$TABLE_NAME" \
    /var/lib/postgresql/backups/physical/base_latest.tar.gz

# Step 4: Dump the table from temporary instance
pg_dump -h localhost -p 15432 -U postgres \
    --data-only --table="$TABLE_NAME" \
    -f "/tmp/${TABLE_NAME}_restore.sql" postgres

# Step 5: Truncate and restore on production
psql -h production -U admin -d appdb -c "TRUNCATE TABLE $TABLE_NAME CASCADE;"
psql -h production -U admin -d appdb -f "/tmp/${TABLE_NAME}_restore.sql"

# Step 6: Verify
psql -h production -U admin -d appdb -c "SELECT count(*) FROM $TABLE_NAME;"

# Step 7: Cleanup
docker rm -f pg_restore_temp
rm -f "/tmp/${TABLE_NAME}_restore.sql"
```

### Runbook 2: Full Database Recovery

**Trigger:** Primary disk failure, database completely lost

```bash
# Step 1: Provision new server (or use standby)
# Step 2: Restore base backup
aws s3 cp s3://my-bucket/physical/base_latest.tar.gz /tmp/
mkdir -p /var/lib/postgresql/15/main
tar -xzf /tmp/base_latest.tar.gz -C /var/lib/postgresql/15/main/

# Step 3: Configure WAL replay
cat >> /var/lib/postgresql/15/main/postgresql.auto.conf << 'EOF'
restore_command = 'aws s3 cp s3://my-bucket/wal-archive/%f %p'
EOF

# Step 4: Start PostgreSQL (auto-replays WAL)
sudo chown -R postgres:postgres /var/lib/postgresql/15/main
sudo systemctl start postgresql

# Step 5: Verify and promote
sudo -u postgres psql -c "SELECT pg_is_in_recovery();"
sudo -u postgres psql -c "SELECT pg_promote();"
```

### Runbook 3: Point-in-Time Recovery

(See Solution 03 for the complete PITR runbook)

### Runbook 4: Cross-Region Recovery

**Trigger:** Entire AWS region is unavailable

```bash
# Step 1: Switch to DR region
aws configure set region us-west-2

# Step 2: Restore from cross-region replicated backups
aws s3 cp s3://my-backup-bucket-dr/physical/base_latest.tar.gz /tmp/

# Step 3: Restore on DR server
# (same as Runbook 2, steps 2-5)

# Step 4: Update DNS to point to DR region
aws route53 change-resource-record-sets \
    --hosted-zone-id Z1234567890 \
    --change-batch '{
        "Changes": [{
            "Action": "UPSERT",
            "ResourceRecordSet": {
                "Name": "db.example.com",
                "Type": "CNAME",
                "TTL": 60,
                "ResourceRecords": [{"Value": "db.dr.us-west-2.example.com"}]
            }
        }]
    }'

# Step 5: Update application connection strings
# (via configuration management -- Ansible, etc.)
```

## Common Mistakes to Avoid

- Not encrypting backups at rest -- a compromised S3 bucket exposes all customer data
- Storing backups in the same region as the primary -- a regional outage destroys both
- Not testing cross-region recovery -- the DR runbook may have assumptions that break in the DR region
- Forgetting that PITR requires all WAL segments -- a gap in the WAL archive means you cannot recover past the gap

## Key Takeaway

A complete backup and DR plan is a layered defense: physical backups for fast full recovery, WAL archiving for point-in-time recovery, logical backups for selective restoration, encryption for security, cross-region replication for site failures, and verification for confidence. Each layer addresses a different failure mode. The runbooks ensure the team can execute recovery under pressure. The investment in backup infrastructure is justified by the cost of downtime.
