# Solution 04: Stateful Application Data Design

## Part A -- Storage Decision Matrix

| Component | Storage Type | Mount Point | Justification |
|-----------|-------------|-------------|---------------|
| **PostgreSQL data** | Named volume | `/var/lib/postgresql/data` | Must survive container restarts and host reboots. Named volumes are Docker-managed, portable, and the recommended approach for database persistence. They decouple data from both the container and the host filesystem structure. |
| **Redis data** | tmpfs | `/data` | Redis is used as a session cache and rate limiter. Losing this data on restart is acceptable (sessions will be re-established, rate limits will reset). tmpfs provides the fastest possible I/O since data lives in RAM. No need for persistence. |
| **Application uploads** | Bind mount | `/app/uploads` | The uploads must be accessible from the host for processing by an external tool (image thumbnailing, virus scanning). A bind mount makes the files visible on the host filesystem without needing to exec into the container. |
| **Application config** | Bind mount (read-only) | `/app/config` | Developers edit the config file on the host. A bind mount reflects changes immediately. Mounting as `:ro` prevents the application from accidentally modifying the config. |
| **Temporary files** | tmpfs | `/tmp/taskboard` | Sensitive data that must never touch disk. tmpfs stores it in RAM only. A size limit prevents memory exhaustion. Data loss on restart is acceptable since these are intermediate processing files. |

## Part B -- Docker Commands

### PostgreSQL

```bash
docker volume create taskboard-pgdata

docker run -d \
  --name taskboard-db \
  -e POSTGRES_PASSWORD=secret \
  -e POSTGRES_DB=taskboard \
  -v taskboard-pgdata:/var/lib/postgresql/data \
  postgres:16-alpine
```

Why a named volume: PostgreSQL writes data files, WAL logs, and configuration
to `/var/lib/postgresql/data`. A named volume ensures these survive container
removal. Docker manages the storage location and permissions, so you do not
need to create or chmod a host directory.

### Redis

```bash
docker run -d \
  --name taskboard-redis \
  --mount type=tmpfs,dst=/data,tmpfs-size=64m \
  redis:7-alpine \
  redis-server --appendonly no --maxmemory 50mb --maxmemory-policy allkeys-lru
```

Why tmpfs: Redis stores its dataset in memory by default. A tmpfs mount at
`/data` ensures that even if Redis writes RDB snapshots or AOF files, they go
to RAM, not disk. The `tmpfs-size` flag caps memory usage. The Redis
configuration flags disable persistence (`appendonly no`) and set a memory
limit with an LRU eviction policy.

### Application (all mounts combined)

```bash
mkdir -p ./uploads ./config

# Create a sample config
cat > ./config/config.yaml <<'EOF'
database:
  host: taskboard-db
  port: 5432
  name: taskboard
redis:
  host: taskboard-redis
  port: 6379
app:
  max_upload_size: 10m
  temp_dir: /tmp/taskboard
EOF

docker run -d \
  --name taskboard-app \
  -p 8080:8080 \
  -v ./uploads:/app/uploads \
  -v ./config/config.yaml:/app/config/config.yaml:ro \
  --mount type=tmpfs,dst=/tmp/taskboard,tmpfs-size=100m \
  --link taskboard-db:db \
  --link taskboard-redis:redis \
  my-taskboard-app:latest
```

Why each mount:
- **uploads bind mount:** The host tool (thumbnailer, virus scanner) reads and
  writes files directly in `./uploads`. No need to enter the container.
- **config bind mount (read-only):** Developers edit `config.yaml` on the host
  with their preferred editor. The `:ro` flag prevents the app from writing
  back to it.
- **tmpfs for /tmp/taskboard:** PDF conversion generates intermediate files
  that may contain sensitive content (PII in documents). tmpfs ensures they
  never touch disk. The 100MB size limit prevents runaway memory usage.

## Part C -- Backup Script

```bash
#!/usr/bin/env bash
set -euo pipefail

BACKUP_DIR="./backups"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
mkdir -p "${BACKUP_DIR}"

echo "Starting TaskBoard backup at ${TIMESTAMP}..."

# --- PostgreSQL Backup ---
# Use pg_dump, NOT a raw volume tar.
# pg_dump produces a consistent SQL dump that:
#   - Can be restored to any PostgreSQL version (not tied to the version that
#     created the data directory).
#   - Handles in-flight transactions correctly (produces a consistent snapshot).
#   - Can be restored into a database with a different name or structure.
#   - Is human-readable and can be inspected/edited if needed.
#
# A raw tar of /var/lib/postgresql/data is:
#   - Tied to the exact PostgreSQL version and OS architecture.
#   - Potentially inconsistent if the database was writing during the tar.
#   - Not portable across storage drivers or filesystems.
#   - Only restorable by starting a PostgreSQL instance with that exact data
#     directory.

echo "Backing up PostgreSQL..."
docker exec taskboard-db pg_dump -U postgres taskboard \
  > "${BACKUP_DIR}/taskboard-db-${TIMESTAMP}.sql"

echo "PostgreSQL backup saved: ${BACKUP_DIR}/taskboard-db-${TIMESTAMP}.sql"

# --- Uploads Backup ---
# Use the tar-from-container pattern (Exercise 03).
# tar is appropriate here because uploads are static files, not a running
# database with transactional consistency requirements.

echo "Backing up uploads..."
docker run --rm \
  -v taskboard-uploads:/data:ro \
  -v "$(pwd)/${BACKUP_DIR}":/backup \
  alpine tar czf "/backup/uploads-${TIMESTAMP}.tar.gz" -C /data .

echo "Uploads backup saved: ${BACKUP_DIR}/uploads-${TIMESTAMP}.tar.gz"

echo ""
echo "Backup complete: ${BACKUP_DIR}"
ls -lh "${BACKUP_DIR}"/*-"${TIMESTAMP}"*
```

### Script Explanation

**`set -euo pipefail`:** The script exits immediately if any command fails
(`-e`), if an unset variable is used (`-u`), or if any command in a pipeline
fails (`-pipefail`). This prevents partial backups from being mistaken for
complete ones.

**`pg_dump` vs tar for databases:** See the inline comments in the script.
The short version: `pg_dump` produces a logical backup (SQL statements) that
is version-independent and transaction-consistent. A tar of the data directory
is a physical backup that is version-specific and may be inconsistent.

**Why two different backup methods:** Different data types have different
consistency requirements. A relational database needs transaction-consistent
backups (`pg_dump`). Static files (uploads) just need to be copied (`tar`).

### Restore Commands

```bash
# Restore PostgreSQL
docker exec -i taskboard-db psql -U postgres taskboard < ./backups/taskboard-db-20260611-143000.sql

# Restore uploads
docker volume create taskboard-uploads-restored
docker run --rm \
  -v taskboard-uploads-restored:/data \
  -v "$(pwd)/backups":/backups:ro \
  alpine sh -c 'tar xzf /backups/uploads-20260611-143000.tar.gz -C /data'
```

## Part D -- Operations README

```
## TaskBoard Operations Guide

### Starting the Stack

1. Ensure Docker and Docker Compose are installed.
2. Create required host directories: `mkdir -p ./uploads ./config`
3. Place your `config.yaml` in `./config/`.
4. Start services: `docker compose up -d` (or use the individual `docker run`
   commands documented above).
5. Wait for the database healthcheck to pass: `docker compose ps` (status
   should show "healthy").

### Data Safety Classification

| Data | Safe to Lose? | Notes |
|------|---------------|-------|
| PostgreSQL data | NO | All user accounts, projects, and tasks. Back up nightly. |
| Uploads | NO | User-uploaded files. Back up nightly alongside the database. |
| Config | NO (but recoverable) | Lives on the host in `./config/`. Check into version control. |
| Redis cache | YES | Sessions and rate limits will be rebuilt on next request. |
| Temporary files | YES | Intermediate PDF processing data. Regenerated on each job. |

### Backing Up

Run the backup script:
```bash
./backup-taskboard.sh
```

Backups are saved to `./backups/` with timestamps. Copy them to off-site
storage (S3, NAS, etc.) as part of your nightly cron job.

### Restoring from Backup

1. Stop the application: `docker compose stop app`
2. Restore the database:
   ```bash
   docker exec -i taskboard-db psql -U postgres taskboard < ./backups/taskboard-db-YYYYMMDD-HHMMSS.sql
   ```
3. Restore uploads:
   ```bash
   docker run --rm \
     -v taskboard-uploads:/data \
     -v $(pwd)/backups:/backups:ro \
     alpine sh -c 'tar xzf /backups/uploads-YYYYMMDD-HHMMSS.tar.gz -C /data'
   ```
4. Restart the application: `docker compose start app`

### Troubleshooting

- **Database connection refused:** Check that the db container is healthy:
  `docker compose ps`. If not, check logs: `docker compose logs db`.
- **Upload files not appearing:** Verify the bind mount: `ls -la ./uploads`.
  Check container mount: `docker compose exec app ls /app/uploads`.
- **Config changes not reflected:** Restart the app service after editing
  `config.yaml`: `docker compose restart app`.
```

## Common Mistakes

1. **Using tar for database backups:** This is the most common and most
   dangerous mistake. A raw tar of the PostgreSQL data directory may contain
   in-flight WAL entries, partial page writes, or other inconsistencies that
   make the backup unusable. Always use `pg_dump` for logical backups or
   `pg_basebackup` for physical backups.

2. **Not setting tmpfs size limits:** Without a size limit, a runaway process
   can fill all host memory via the tmpfs mount, potentially crashing the
   host. Always set `tmpfs-size`.

3. **Mounting config as read-write:** If the application has a bug that
   overwrites the config file, you lose your configuration. Mounting as `:ro`
   prevents this and makes the failure mode obvious (the app gets a write
   error) rather than silent (the config is corrupted).

4. **Using a bind mount for database data:** Bind mounts work but create an
   implicit dependency on the host directory structure. If you move the project
   to a different directory or a different host, the bind mount path breaks.
   Named volumes are self-contained and portable.

5. **Not using `set -euo pipefail` in backup scripts:** Without these flags,
   a failed `pg_dump` does not stop the script. The script continues, reports
   success, and you discover the backup is empty weeks later when you need it.
