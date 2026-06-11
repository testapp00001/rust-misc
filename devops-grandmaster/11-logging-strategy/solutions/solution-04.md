# Solution 04: Design a Logging Strategy with Rotation, Retention, and Cost Control

## Part A: Storage Calculations

### 1. Raw storage (90 days at 10 GB/day)

```
10 GB/day x 90 days = 900 GB raw storage
```

### 2. Compressed storage (10:1 ratio)

```
900 GB / 10 = 90 GB compressed storage
```

### 3. Black Friday surge

Peak volume: 30 GB/day for 3 days. Normal volume for those 3 days would be
30 GB. Extra volume:

```
(30 - 10) GB/day x 3 days = 60 GB extra raw
60 GB / 10 (compressed) = 6 GB extra compressed
```

### 4. Total peak storage and budget check

```
Normal compressed:  90 GB
Surge compressed:    6 GB
Total:              96 GB
```

At $50/TB/month: 96 GB is roughly 10% of a TB, so about **$5/month**.
Well within the $200/month budget.

Even at 1 TB of provisioned SSD (to allow for future growth): **$50/month**.
Still within budget.

---

## Part B: Docker Log Rotation Calculations

### Peak volume per container per hour

```
Normal: 500 MB/day per container
Peak:   500 MB x 3 = 1500 MB/day = 62.5 MB/hour
```

### How fast does a 10 MB file fill?

```
62.5 MB/hour / 10 MB per file = 6.25 files per hour
```

A 10 MB log file fills in approximately **9.6 minutes** at peak.

### How many files for 2 hours of coverage?

```
6.25 files/hour x 2 hours = 12.5 files
Round up to 15 files for safety.
```

### Storage per container with 50 MB constraint

If the constraint is 50 MB per container:
- 50 MB / 15 files = 3.33 MB per file (too small, will rotate very fast)
- Alternative: 50 MB / 5 files = 10 MB per file (covers ~48 minutes at peak)

**Recommendation:** Accept that 50 MB per container is tight for peak
traffic. Use `max-size: 10m`, `max-file: 5` (50 MB total) for most services.
For critical services where you need more history, allocate a larger
`max-size` and accept the storage cost.

---

## Part C: Daemon Configuration

### /etc/docker/daemon.json

```json
{
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "10m",
    "max-file": "5",
    "compress": "true",
    "tag": "{{.Name}}"
  }
}
```

**Why each option:**

- `max-size: 10m` -- each log file is capped at 10 MB. This is a good
  balance between file count and rotation frequency.
- `max-file: 5` -- keep 5 rotated files. Total per container: 50 MB.
  Covers approximately 48 minutes of peak traffic.
- `compress: true` -- rotated files are gzip-compressed. Text logs
  compress at roughly 10:1, so 50 MB of log files occupy ~5 MB on disk.
  This is the single most effective cost reduction measure.
- `tag: {{.Name}}` -- includes the container name in each log entry's tag.
  Essential for identifying which container produced a log line when
  forwarding to a central system.

**After changing daemon.json:**

```bash
sudo systemctl restart docker
```

This only affects new containers. Existing containers keep their current
logging configuration until they are recreated.

---

## Part D: docker-compose.yml with Per-Service Overrides

```yaml
version: "3.8"

services:
  web-frontend:
    image: nginx:alpine
    ports:
      - "80:80"
    environment:
      - LOG_LEVEL=WARNING
    logging:
      driver: json-file
      options:
        max-size: "5m"
        max-file: "3"
    # Total log storage: 15 MB per container
    # High volume, low importance -- minimal retention

  api-backend:
    build: ./api-backend
    ports:
      - "8000:8000"
    environment:
      - LOG_LEVEL=INFO
    logging:
      driver: json-file
      options:
        max-size: "20m"
        max-file: "5"
        compress: "true"
    # Total log storage: 100 MB per container
    # Medium volume, high importance -- more retention

  payment-service:
    build: ./payment-service
    ports:
      - "8001:8001"
    environment:
      - LOG_LEVEL=DEBUG
    logging:
      driver: json-file
      options:
        max-size: "50m"
        max-file: "10"
        compress: "true"
    # Total log storage: 500 MB per container
    # Low volume, critical importance -- maximum retention
    # DEBUG level because payment issues need full context
```

**Rationale for per-service differences:**

| Service | max-size | max-file | Total | Rationale |
|---------|----------|----------|-------|-----------|
| web-frontend | 5m | 3 | 15 MB | Static asset requests are noisy and rarely useful for debugging. Keep minimal logs. |
| api-backend | 20m | 5 | 100 MB | API logs are essential for debugging. Keep more history. |
| payment-service | 50m | 10 | 500 MB | Payment failures are rare but critical. Need full context for investigations. |

---

## Part E: Logging Policy Document

### LOGGING_POLICY.md

```markdown
# Logging Policy

## 1. Log Levels

| Level | When to Use | Examples |
|-------|-------------|----------|
| DEBUG | Detailed diagnostic info. Disabled in production. | SQL queries, cache lookups, validation steps |
| INFO | Normal operations that are worth recording. | Request received, order created, user logged in |
| WARNING | Unexpected but recoverable situations. | Retry attempted, fallback used, deprecated API called |
| ERROR | An operation failed. Requires investigation. | Payment failed, database unreachable, external API error |
| CRITICAL | The service cannot function. Requires immediate action. | Configuration missing, fatal startup error, data corruption |

**Decision rule:** If you would not want to be paged at 3 AM for this event,
it is not CRITICAL. If you would not want to investigate it tomorrow, it is
not ERROR. If it is expected behavior, it is not WARNING.

## 2. Required Fields

Every JSON log entry MUST contain:

- `timestamp` -- ISO 8601 with UTC timezone (e.g., `2024-03-15T10:30:45.123Z`)
- `level` -- one of DEBUG, INFO, WARNING, ERROR, CRITICAL
- `message` -- human-readable description of the event
- `service` -- the service name (e.g., `order-service`)
- `request_id` -- UUID for request tracing (generated in the gateway)

Optional but recommended:

- `user_id` -- the authenticated user, if applicable
- `duration_seconds` -- for timed operations
- `exception` -- full stack trace for ERROR and CRITICAL logs

**PII restriction:** Log entries MUST NOT contain passwords, credit card
numbers, social security numbers, or full email addresses. Use hashed or
truncated identifiers instead.

## 3. Retention Policy

| Tier | Storage | Retention | Compression |
|------|---------|-----------|-------------|
| Hot (Docker host) | Local SSD | 48 hours | gzip on rotation |
| Warm (Log aggregator) | NAS/S3 | 30 days | gzip |
| Cold (Archive) | S3 Glacier | 90 days | gzip |

Logs older than 90 days are deleted. This meets our compliance requirement.

## 4. Storage Budget

| Item | Monthly Cost |
|------|-------------|
| SSD for hot logs (3 hosts x 50 GB) | $15 |
| NAS for warm logs (200 GB) | $20 |
| S3 for cold logs (500 GB) | $12 |
| Elasticsearch cluster (3 nodes) | $120 |
| **Total** | **$167** |

Budget: $200/month. Current utilization: 84%. Headroom: $33/month.

**If we exceed budget:** First, reduce DEBUG/INFO retention in hot tier.
Second, increase compression. Third, reduce log volume by raising log levels
on noisy services.

## 5. Escalation

| Level | Notification Channel | Response Time |
|-------|---------------------|---------------|
| DEBUG | No notification | N/A |
| INFO | No notification | N/A |
| WARNING | Slack #warnings channel | Next business day |
| ERROR | Slack #alerts + PagerDuty (business hours) | 4 hours |
| CRITICAL | PagerDuty (24/7) | 15 minutes |

## 6. Review Schedule

- **Weekly:** Review log volume trends and storage utilization dashboards.
- **Monthly:** Review logging costs against budget. Adjust retention or
  volume if trending over budget.
- **Quarterly:** Review log levels across all services. Identify services
  that produce excessive DEBUG/INFO volume. Review this policy document
  and update as needed.
```

---

## Part F: Log Cleanup Script

### cleanup-logs.sh

```bash
#!/usr/bin/env bash
#
# cleanup-logs.sh -- Docker log cleanup and compression
#
# Usage:
#   ./cleanup-logs.sh              # Run cleanup
#   ./cleanup-logs.sh --dry-run    # Show what would be done without doing it
#
# Cron example (daily at 3 AM):
#   0 3 * * * /opt/scripts/cleanup-logs.sh >> /var/log/cleanup-logs.log 2>&1

set -euo pipefail

DOCKER_LOG_DIR="/var/lib/docker/containers"
COMPRESS_AGE_DAYS=1
DELETE_AGE_DAYS=90
DRY_RUN=false

if [[ "${1:-}" == "--dry-run" ]]; then
    DRY_RUN=true
    echo "[DRY RUN] No files will be modified."
fi

bytes_before=0
bytes_after=0
files_compressed=0
files_deleted=0

echo "=== Log cleanup started at $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="

# --- Step 1: Compress uncompressed rotated log files older than 1 day ---
echo ""
echo "--- Step 1: Compressing old uncompressed log files ---"

while IFS= read -r -d '' file; do
    bytes_before=$((bytes_before + $(stat -c%s "$file" 2>/dev/null || echo 0)))

    if $DRY_RUN; then
        echo "[DRY RUN] Would compress: $file"
    else
        gzip "$file"
        echo "Compressed: $file"
    fi
    files_compressed=$((files_compressed + 1))
done < <(find "$DOCKER_LOG_DIR" -name "*-json.log.*" ! -name "*.gz" -mtime +$COMPRESS_AGE_DAYS -print0 2>/dev/null)

echo "Files compressed: $files_compressed"

# --- Step 2: Delete compressed log files older than 90 days ---
echo ""
echo "--- Step 2: Deleting compressed log files older than $DELETE_AGE_DAYS days ---"

while IFS= read -r -d '' file; do
    file_size=$(stat -c%s "$file" 2>/dev/null || echo 0)
    bytes_after=$((bytes_after + file_size))

    if $DRY_RUN; then
        echo "[DRY RUN] Would delete: $file ($(numfmt --to=iec "$file_size"))"
    else
        rm -f "$file"
        echo "Deleted: $file ($(numfmt --to=iec "$file_size"))"
    fi
    files_deleted=$((files_deleted + 1))
done < <(find "$DOCKER_LOG_DIR" -name "*-json.log.*.gz" -mtime +$DELETE_AGE_DAYS -print0 2>/dev/null)

echo "Files deleted: $files_deleted"

# --- Step 3: Report ---
echo ""
echo "=== Summary ==="
echo "Files compressed: $files_compressed"
echo "Files deleted:    $files_deleted"
echo "Space reclaimed:  $(numfmt --to=iec $((bytes_before > bytes_after ? bytes_before - bytes_after : 0)))"
echo "=== Cleanup finished at $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
```

**Deploy the script:**

```bash
chmod +x cleanup-logs.sh

# Test with dry run first
./cleanup-logs.sh --dry-run

# Run for real
sudo ./cleanup-logs.sh

# Add to cron (daily at 3 AM)
echo "0 3 * * * root /opt/scripts/cleanup-logs.sh >> /var/log/cleanup-logs.log 2>&1" \
  | sudo tee /etc/cron.d/docker-log-cleanup
```

---

## Why This Works

1. **Layered defense.** Docker's `max-size`/`max-file` prevents any single
   container from filling the disk (first layer). The cleanup script
   compresses and deletes old files (second layer). The retention policy
   defines how long logs are kept (third layer).

2. **Compression is the biggest win.** Text logs compress at 10:1 or better.
   Enabling `compress: true` in daemon.json turns 500 MB of rotated logs
   into ~50 MB on disk. This alone can keep you within budget.

3. **Per-service tuning.** Not all services have the same logging value. A
   static file server generates high-volume, low-value logs. A payment
   service generates low-volume, high-value logs. Tuning rotation per
   service matches storage allocation to value.

4. **The cleanup script is a safety net.** Even with rotation configured,
   edge cases exist: a container might be created without rotation options,
   a misconfiguration might disable rotation, or a burst of traffic might
   produce more logs than expected. The script catches these cases.

## Common Mistakes

### Mistake 1: Setting max-size too small

A `max-size` of 1 MB means the log file rotates every few seconds during
normal traffic. This creates many small files and excessive I/O. Use at
least 5-10 MB.

### Mistake 2: Forgetting that daemon.json changes require a restart

```bash
# Edit daemon.json
sudo vim /etc/docker/daemon.json

# MUST restart Docker
sudo systemctl restart docker
```

New containers pick up the new defaults. Existing containers keep their old
configuration until recreated.

### Mistake 3: Not enabling compression

Without compression, 500 MB/day x 90 days = 45 GB on disk. With compression:
4.5 GB. This is a 10x difference for a single configuration flag.

### Mistake 4: Relying only on Docker's built-in rotation

Docker rotation works for the json-file driver, but if you forward logs to
a central system, the forwarding buffer can grow if the destination is
unreachable. Set `--log-opt max-buffer-size` for drivers that support it.

### Mistake 5: Not monitoring log volume

If a bug causes a service to log at 10x normal volume, your disk fills in
hours. Set up alerts on log volume growth rate, not just disk usage.
