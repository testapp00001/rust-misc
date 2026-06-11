# Solution 04: Backup Verification and Integrity Testing

## Part A: Verification Pipeline Design

```
+------------------+     +-------------------+     +------------------+
|  Backup Completes|---->|  Trigger Verify   |---->|  Create Test     |
|  (pg_dump or     |     |  Pipeline         |     |  Container       |
|   pg_basebackup) |     |  (webhook/cron)   |     |  (Docker)        |
+------------------+     +-------------------+     +--------+---------+
                                                           |
                                                           v
+------------------+     +-------------------+     +------------------+
|  Report Results  |<----|  Compare with     |<----|  Restore Backup  |
|  (Slack/Email)   |     |  Source Database   |     |  in Container    |
|  + Store Metrics |     |  (row counts,     |     |                  |
+------------------+     |   checksums)      |     +------------------+
                         +-------------------+
```

### Pipeline Steps

1. **Trigger**: Webhook from backup script or cron job
2. **Container creation**: Spin up a fresh PostgreSQL instance in Docker
3. **Restore**: Load the backup into the container
4. **Run ANALYZE**: Update statistics (detects corruption)
5. **Row count comparison**: Compare critical table counts with source
6. **Index consistency check**: Run `pg_amcheck` on all indexes
7. **Report**: Send results to Slack and store metrics in Prometheus

## Part B: Verification Scripts

### verify_backup.sh

```bash
#!/bin/bash
set -euo pipefail

# =============================================================
# verify_backup.sh -- Restore and verify a PostgreSQL backup
# =============================================================

BACKUP_FILE="$1"
SOURCE_HOST="${SOURCE_HOST:-localhost}"
SOURCE_PORT="${SOURCE_PORT:-5432}"
SOURCE_DB="${SOURCE_DB:-appdb}"
SOURCE_USER="${SOURCE_USER:-readonly}"
CONTAINER_NAME="pg_verify_$(date +%s)"
CONTAINER_PORT=15432
VERIFY_DB="verify_db"
SLACK_WEBHOOK="${SLACK_WEBHOOK:-}"

log() { echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*"; }

notify() {
    local status="$1" message="$2"
    if [[ -n "$SLACK_WEBHOOK" ]]; then
        curl -s -X POST -H 'Content-type: application/json' \
            --data "{\"text\": \"Backup Verify [$status]: $message\"}" \
            "$SLACK_WEBHOOK" || true
    fi
}

cleanup() {
    log "Cleaning up test container..."
    docker rm -f "$CONTAINER_NAME" 2>/dev/null || true
}
trap cleanup EXIT

# Step 1: Verify checksum
log "Verifying backup checksum..."
if [[ -f "${BACKUP_FILE}.sha256" ]]; then
    if sha256sum -c "${BACKUP_FILE}.sha256"; then
        log "Checksum OK"
    else
        log "CHECKSUM MISMATCH"
        notify "FAIL" "Checksum mismatch for $BACKUP_FILE"
        exit 1
    fi
else
    log "WARNING: No checksum file found"
fi

# Step 2: Start test container
log "Starting test PostgreSQL container..."
docker run -d \
    --name "$CONTAINER_NAME" \
    -e POSTGRES_PASSWORD=verify_pass \
    -e POSTGRES_DB="$VERIFY_DB" \
    -p "$CONTAINER_PORT":5432 \
    postgres:15

# Wait for PostgreSQL to be ready
log "Waiting for PostgreSQL to start..."
for i in $(seq 1 30); do
    if docker exec "$CONTAINER_NAME" pg_isready -U postgres >/dev/null 2>&1; then
        break
    fi
    sleep 2
done

# Step 3: Restore backup
log "Restoring backup..."
RESTORE_START=$SECONDS

pg_restore \
    -h localhost \
    -p "$CONTAINER_PORT" \
    -U postgres \
    -d "$VERIFY_DB" \
    --no-owner \
    --no-privileges \
    --exit-on-error \
    "$BACKUP_FILE" 2>&1 | tee /tmp/restore.log

RESTORE_EXIT=${PIPESTATUS[0]}
RESTORE_DURATION=$((SECONDS - RESTORE_START))

if [[ $RESTORE_EXIT -ne 0 ]]; then
    log "RESTORE FAILED (exit code $RESTORE_EXIT)"
    notify "FAIL" "Restore failed for $BACKUP_FILE (exit $RESTORE_EXIT)"
    exit 1
fi
log "Restore completed in ${RESTORE_DURATION}s"

# Step 4: Run ANALYZE
log "Running ANALYZE on all tables..."
docker exec "$CONTAINER_NAME" psql -U postgres -d "$VERIFY_DB" -c "ANALYZE;"

# Step 5: Row count comparison
log "Comparing row counts with source..."
TABLES=$(docker exec "$CONTAINER_NAME" psql -U postgres -d "$VERIFY_DB" -t -c \
    "SELECT tablename FROM pg_tables WHERE schemaname = 'public';")

MISMATCH_COUNT=0
for table in $TABLES; do
    table=$(echo "$table" | xargs)  # trim whitespace
    [[ -z "$table" ]] && continue

    VERIFY_COUNT=$(docker exec "$CONTAINER_NAME" psql -U postgres -d "$VERIFY_DB" -t -c \
        "SELECT count(*) FROM \"$table\";" | xargs)

    SOURCE_COUNT=$(PGPASSWORD=readonly_pass psql -h "$SOURCE_HOST" -p "$SOURCE_PORT" \
        -U "$SOURCE_USER" -d "$SOURCE_DB" -t -c \
        "SELECT count(*) FROM \"$table\";" | xargs)

    if [[ "$VERIFY_COUNT" != "$SOURCE_COUNT" ]]; then
        log "MISMATCH: $table -- source=$SOURCE_COUNT, restored=$VERIFY_COUNT"
        MISMATCH_COUNT=$((MISMATCH_COUNT + 1))
    else
        log "OK: $table -- $VERIFY_COUNT rows"
    fi
done

# Step 6: Index consistency check
log "Running pg_amcheck..."
docker exec "$CONTAINER_NAME" pg_amcheck \
    -U postgres \
    -d "$VERIFY_DB" \
    --heapallindexed \
    --no-dependencies 2>&1 | tee /tmp/amcheck.log

AMCHECK_EXIT=${PIPESTATUS[0]}

# Step 7: Report results
log "========================================="
log "Verification Results"
log "========================================="
log "Backup file:    $BACKUP_FILE"
log "Restore time:   ${RESTORE_DURATION}s"
log "Row mismatches: $MISMATCH_COUNT"
log "pg_amcheck:     $(if [[ $AMCHECK_EXIT -eq 0 ]]; then echo 'PASS'; else echo 'FAIL'; fi)"
log "========================================="

if [[ $MISMATCH_COUNT -eq 0 ]] && [[ $AMCHECK_EXIT -eq 0 ]]; then
    log "VERIFICATION PASSED"
    notify "PASS" "$BACKUP_FILE -- restored in ${RESTORE_DURATION}s, all checks passed"
    exit 0
else
    log "VERIFICATION FAILED"
    notify "FAIL" "$BACKUP_FILE -- $MISMATCH_COUNT mismatches, amcheck exit $AMCHECK_EXIT"
    exit 1
fi
```

### Why This Script Works

- **Container isolation**: Each verification runs in a fresh Docker container -- no state leakage between runs
- **Checksum verification**: Catches file-level corruption before even attempting restore
- **Row count comparison**: Detects data loss during backup or restore
- **pg_amcheck**: Detects index corruption that row counts cannot catch
- **Trap cleanup**: Container is always removed, even if the script fails

## Part C: Checksum Verification

### Generate Checksums After Backup

```bash
#!/bin/bash
# generate_checksums.sh -- Generate SHA-256 checksums for backup files

BACKUP_DIR="/var/lib/postgresql/backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

for file in "$BACKUP_DIR"/*.dump "$BACKUP_DIR"/*.tar.gz; do
    [[ -f "$file" ]] || continue

    sha256sum "$file" > "${file}.sha256"
    echo "Generated checksum: ${file}.sha256"
done

# Store checksums separately from backups
cp "$BACKUP_DIR"/*.sha256 /var/lib/postgresql/checksums/
```

### Verify Checksums Before Restore

```bash
#!/bin/bash
# verify_checksum.sh -- Verify backup checksum before restore

BACKUP_FILE="$1"
CHECKSUM_FILE="${BACKUP_FILE}.sha256"

if [[ ! -f "$CHECKSUM_FILE" ]]; then
    echo "WARNING: No checksum file found for $BACKUP_FILE"
    echo "Proceeding without checksum verification"
    exit 0
fi

echo "Verifying checksum for $BACKUP_FILE..."
if sha256sum -c "$CHECKSUM_FILE"; then
    echo "Checksum verified OK"
    exit 0
else
    echo "CHECKSUM VERIFICATION FAILED"
    echo "The backup file may be corrupted or tampered with."
    echo "Do NOT restore this backup."
    exit 1
fi
```

### S3 Checksum Verification

```bash
#!/bin/bash
# verify_s3_checksum.sh -- Verify backup integrity after S3 download

S3_PATH="$1"
LOCAL_FILE="$2"

# Download
aws s3 cp "$S3_PATH" "$LOCAL_FILE"

# Get S3 ETag (MD5 for non-multipart uploads)
S3_ETAG=$(aws s3api head-object \
    --bucket "$(echo "$S3_PATH" | sed 's|s3://||' | cut -d'/' -f1)" \
    --key "$(echo "$S3_PATH" | sed 's|s3://[^/]*/||')" \
    --query 'ETag' \
    --output text | tr -d '"')

# Calculate local MD5
LOCAL_MD5=$(md5sum "$LOCAL_FILE" | cut -d' ' -f1)

if [[ "$S3_ETAG" == "$LOCAL_MD5" ]]; then
    echo "S3 integrity check passed"
else
    echo "WARNING: S3 ETag ($S3_ETAG) does not match local MD5 ($LOCAL_MD5)"
    echo "This may be expected for multipart uploads"
    # Fall back to SHA-256 verification
    sha256sum -c "${LOCAL_FILE}.sha256"
fi
```

## Part D: Alerting for Backup Failures

### Prometheus Metrics

```python
# backup_metrics.py -- Export backup status as Prometheus metrics
from prometheus_client import Gauge, start_http_server
import subprocess
import time

backup_last_success = Gauge(
    'pg_backup_last_success_timestamp',
    'Timestamp of last successful backup',
    ['backup_type']
)

backup_duration = Gauge(
    'pg_backup_duration_seconds',
    'Duration of last backup in seconds',
    ['backup_type']
)

backup_size = Gauge(
    'pg_backup_size_bytes',
    'Size of last backup in bytes',
    ['backup_type']
)

backup_verify_status = Gauge(
    'pg_backup_verify_status',
    'Status of last backup verification (1=pass, 0=fail)',
    ['backup_type']
)

def collect_metrics():
    # Check last backup status from a status file
    for backup_type in ['logical', 'physical']:
        status_file = f"/var/lib/postgresql/backups/{backup_type}/last_status.json"
        try:
            with open(status_file) as f:
                import json
                status = json.load(f)
                backup_last_success.labels(backup_type=backup_type).set(
                    status.get('timestamp', 0))
                backup_duration.labels(backup_type=backup_type).set(
                    status.get('duration', 0))
                backup_size.labels(backup_type=backup_type).set(
                    status.get('size', 0))
                backup_verify_status.labels(backup_type=backup_type).set(
                    1 if status.get('verify_pass', False) else 0)
        except FileNotFoundError:
            pass

if __name__ == '__main__':
    start_http_server(9188)
    while True:
        collect_metrics()
        time.sleep(60)
```

### Alert Rules

```yaml
groups:
  - name: backup_alerts
    rules:
      - alert: BackupFailed
        expr: pg_backup_verify_status == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Backup verification failed for {{ $labels.backup_type }}"
          description: "The last {{ $labels.backup_type }} backup failed verification."

      - alert: BackupStale
        expr: (time() - pg_backup_last_success_timestamp) > 90000
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "No successful backup in 25 hours"
          description: "Last {{ $labels.backup_type }} backup was {{ $value }} seconds ago."

      - alert: BackupSizeDrop
        expr: pg_backup_size_bytes < (pg_backup_size_bytes offset 1d) * 0.5
        for: 1m
        labels:
          severity: warning
        annotations:
          summary: "Backup size dropped significantly"
          description: "{{ $labels.backup_type }} backup is less than half the previous size."

      - alert: BackupDurationHigh
        expr: pg_backup_duration_seconds > 3600
        for: 1m
        labels:
          severity: warning
        annotations:
          summary: "Backup taking longer than 1 hour"
          description: "{{ $labels.backup_type }} backup took {{ $value }} seconds."
```

## Common Mistakes to Avoid

- Verifying backups on the same server as the production database -- a disk failure affects both
- Not testing the restore process end-to-end -- a backup that restores but produces corrupt data is useless
- Ignoring pg_amcheck output -- index corruption is invisible to row count checks
- Setting alert thresholds too tight -- a 1-hour backup on a large database is normal, not a failure

## Key Takeaway

Backup verification is not optional -- it is a critical part of the backup pipeline. The verification must actually restore the backup (not just check file existence), compare data with the source, and check index integrity. Checksums catch file corruption, but only a full restore test proves the backup is usable. Automate verification and alert on any failure.
