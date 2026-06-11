# Solution 02: Automated Backup Pipeline Setup

## Part A: pg_dump Backup Script

```bash
#!/bin/bash
set -euo pipefail

# =============================================================
# backup_logical.sh -- Automated PostgreSQL logical backup
# =============================================================

# Configuration
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-appdb}"
DB_USER="${DB_USER:-backup_user}"
BACKUP_DIR="${BACKUP_DIR:-/var/lib/postgresql/backups/logical}"
RETENTION_DAYS="${RETENTION_DAYS:-7}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/${DB_NAME}_${TIMESTAMP}.dump"
LOG_FILE="${BACKUP_DIR}/${DB_NAME}_${TIMESTAMP}.log"
SLACK_WEBHOOK="${SLACK_WEBHOOK:-}"

# Functions
log() {
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*" | tee -a "$LOG_FILE"
}

notify_slack() {
    local message="$1"
    if [[ -n "$SLACK_WEBHOOK" ]]; then
        curl -s -X POST -H 'Content-type: application/json' \
            --data "{\"text\": \"$message\"}" \
            "$SLACK_WEBHOOK" || true
    fi
}

cleanup_old_backups() {
    log "Cleaning up backups older than ${RETENTION_DAYS} days..."
    find "$BACKUP_DIR" -name "${DB_NAME}_*.dump" -mtime +${RETENTION_DAYS} -delete
    find "$BACKUP_DIR" -name "${DB_NAME}_*.log" -mtime +${RETENTION_DAYS} -delete
    find "$BACKUP_DIR" -name "${DB_NAME}_*.sha256" -mtime +${RETENTION_DAYS} -delete
    log "Cleanup complete."
}

# Main backup logic
main() {
    log "Starting logical backup of ${DB_NAME}..."
    mkdir -p "$BACKUP_DIR"

    local start_time=$SECONDS

    # Take the backup
    pg_dump \
        -h "$DB_HOST" \
        -p "$DB_PORT" \
        -U "$DB_USER" \
        -Fc \
        --no-owner \
        --no-privileges \
        -f "$BACKUP_FILE" \
        "$DB_NAME" 2>>"$LOG_FILE"

    local exit_code=$?
    local duration=$((SECONDS - start_time))
    local backup_size=$(du -sh "$BACKUP_FILE" | cut -f1)

    if [[ $exit_code -eq 0 ]]; then
        log "Backup completed successfully."
        log "  File: ${BACKUP_FILE}"
        log "  Size: ${backup_size}"
        log "  Duration: ${duration} seconds"

        # Generate checksum
        sha256sum "$BACKUP_FILE" > "${BACKUP_FILE}.sha256"
        log "  Checksum: $(cat "${BACKUP_FILE}.sha256")"

        notify_slack "Backup SUCCESS: ${DB_NAME} - ${backup_size} in ${duration}s"
    else
        log "Backup FAILED with exit code ${exit_code}."
        notify_slack "Backup FAILED: ${DB_NAME} - exit code ${exit_code}"
        exit 1
    fi

    # Cleanup old backups
    cleanup_old_backups

    log "Backup pipeline complete."
}

main "$@"
```

### Why This Script Works

- `set -euo pipefail`: Stops on any error, undefined variable, or pipe failure
- `pg_dump -Fc`: Custom format is compressed and supports selective restore with `pg_restore`
- `--no-owner --no-privileges`: Makes the dump portable across different environments
- SHA-256 checksum: Detects file corruption before restore
- Slack notification: Immediate visibility into backup status
- Retention cleanup: Prevents disk from filling up with old backups

## Part B: WAL Archiving Configuration

### postgresql.conf

```ini
# WAL archiving configuration
archive_mode = on              # Requires restart to change
archive_command = '/usr/local/bin/archive_wal.sh %p %f'
archive_timeout = 300          # Force archive every 5 minutes (even if no activity)
```

### archive_wal.sh

```bash
#!/bin/bash
set -euo pipefail

# =============================================================
# archive_wal.sh -- Archive WAL files to local dir and S3
# Usage: archive_command = '/usr/local/bin/archive_wal.sh %p %f'
# %p = path to WAL file, %f = WAL file name
# =============================================================

WAL_PATH="$1"
WAL_NAME="$2"
LOCAL_ARCHIVE="/var/lib/postgresql/wal_archive"
S3_BUCKET="${S3_BUCKET:-s3://my-backup-bucket/wal-archive}"
MAX_RETRIES=3

# Local archive
mkdir -p "$LOCAL_ARCHIVE"
cp "$WAL_PATH" "${LOCAL_ARCHIVE}/${WAL_NAME}"

# S3 archive with retry
for attempt in $(seq 1 $MAX_RETRIES); do
    if aws s3 cp "${LOCAL_ARCHIVE}/${WAL_NAME}" "${S3_BUCKET}/${WAL_NAME}" \
        --sse AES256 \
        --quiet; then
        exit 0
    fi
    echo "S3 upload attempt $attempt failed, retrying in $((attempt * 5))s..." >&2
    sleep $((attempt * 5))
done

echo "FATAL: Failed to archive WAL file $WAL_NAME after $MAX_RETRIES attempts" >&2
exit 1
```

### Why This Configuration Works

- `archive_mode = on`: Enables WAL archiving (requires PostgreSQL restart)
- `archive_command`: PostgreSQL calls this for each completed WAL segment. It must return 0 on success.
- `archive_timeout = 300`: Forces WAL archival every 5 minutes even during low activity, limiting RPO to 5 minutes
- Local + S3: Local archive provides fast restore; S3 provides offsite protection
- Retry logic: Handles transient S3 failures without losing WAL segments

## Part C: Scheduling System

### Cron Configuration

```cron
# /etc/cron.d/postgresql-backups

# Environment
SHELL=/bin/bash
PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin
POSTGRES_USER=backup_user

# Full logical backup at 2 AM UTC daily
0 2 * * * postgres /usr/local/bin/backup_logical.sh >> /var/log/postgresql/backup.log 2>&1

# Physical base backup at 3 AM UTC daily
0 3 * * * postgres /usr/local/bin/backup_physical.sh >> /var/log/postgresql/backup_physical.log 2>&1

# WAL archive cleanup at 4 AM UTC daily (keep 7 days local, 90 days S3)
0 4 * * * postgres /usr/local/bin/cleanup_wal.sh >> /var/log/postgresql/cleanup.log 2>&1

# Backup verification at 5 AM UTC daily
0 5 * * * postgres /usr/local/bin/verify_backup.sh >> /var/log/postgresql/verify.log 2>&1
```

### cleanup_wal.sh

```bash
#!/bin/bash
set -euo pipefail

LOCAL_ARCHIVE="/var/lib/postgresql/wal_archive"
LOCAL_RETENTION_DAYS=7

# Clean local WAL archives older than 7 days
find "$LOCAL_ARCHIVE" -name "0000*" -mtime +${LOCAL_RETENTION_DAYS} -delete

# S3 lifecycle policy handles S3 cleanup (configured separately)
echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) WAL cleanup complete" >> /var/log/postgresql/cleanup.log
```

### S3 Lifecycle Policy

```json
{
    "Rules": [
        {
            "ID": "wal-archive-retention",
            "Status": "Enabled",
            "Filter": {
                "Prefix": "wal-archive/"
            },
            "Expiration": {
                "Days": 90
            }
        },
        {
            "ID": "backup-retention",
            "Status": "Enabled",
            "Filter": {
                "Prefix": "logical-backups/"
            },
            "Expiration": {
                "Days": 30
            }
        }
    ]
}
```

## Part D: S3 Upload with Encryption and Retry

```bash
#!/bin/bash
set -euo pipefail

# =============================================================
# upload_to_s3.sh -- Upload backup to S3 with encryption
# =============================================================

LOCAL_FILE="$1"
S3_PATH="$2"
KMS_KEY="${KMS_KEY:-alias/backup-key}"
MAX_RETRIES=5

upload_with_retry() {
    local attempt=1
    local wait_time=10

    while [[ $attempt -le $MAX_RETRIES ]]; do
        echo "Upload attempt $attempt of $MAX_RETRIES..."

        if aws s3 cp "$LOCAL_FILE" "$S3_PATH" \
            --sse aws:kms \
            --sse-kms-key-id "$KMS_KEY" \
            --expected-size $(stat -c%s "$LOCAL_FILE") \
            --quiet; then
            echo "Upload successful."
            return 0
        fi

        echo "Upload failed. Retrying in ${wait_time}s..."
        sleep "$wait_time"
        wait_time=$((wait_time * 2))
        attempt=$((attempt + 1))
    done

    echo "FATAL: Upload failed after $MAX_RETRIES attempts."
    return 1
}

# Verify local checksum before upload
LOCAL_CHECKSUM=$(sha256sum "$LOCAL_FILE" | cut -d' ' -f1)
echo "Local SHA-256: $LOCAL_CHECKSUM"

# Upload
upload_with_retry

# Verify S3 checksum
S3_CHECKSUM=$(aws s3api head-object \
    --bucket "$(echo "$S3_PATH" | sed 's|s3://||' | cut -d'/' -f1)" \
    --key "$(echo "$S3_PATH" | sed 's|s3://[^/]*/||')" \
    --query 'Metadata.sha256' \
    --output text 2>/dev/null || echo "none")

echo "Upload complete. S3 path: $S3_PATH"
```

## Common Mistakes to Avoid

- Not setting `archive_mode = on` -- this requires a PostgreSQL restart, not just a reload
- Forgetting that `archive_command` must return 0 -- any non-zero return causes PostgreSQL to keep retrying and WAL accumulates
- Using `pg_dump` without `--no-owner` -- restoration on a different server fails if the owning user does not exist
- Not monitoring backup file sizes -- a sudden size decrease may indicate corruption or incomplete backup

## Key Takeaway

A production backup pipeline requires three layers: logical backups for selective restore, physical backups for fast full recovery, and WAL archiving for point-in-time recovery. Automation with cron ensures consistency. S3 with encryption provides offsite protection. Retention policies prevent storage from growing indefinitely. The pipeline is only as good as its verification -- always test that backups can be restored.
