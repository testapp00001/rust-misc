# Module 09: Volumes & Data — Persisting Data Beyond Container Lifecycle

> **Previous Module:** [08 - Container Networking](../08-container-networking/README.md)
> **Next Module:** [10 - Container Security](../10-container-security/README.md)

---

## 1. The Problem — Containers Are Ephemeral

When you run a container, Docker gives it its own filesystem. This filesystem is built from the image layers and a thin writable layer on top. Everything the container writes goes to that writable layer.

**The catch:** when the container stops, restarts, or is removed, that writable layer is destroyed. Every file, every database record, every log entry — gone.

```bash
# Start a container, write some data
docker run --name my-app -d nginx
docker exec my-app sh -c 'echo "Important data" > /data/config.txt'
docker exec my-app cat /data/config.txt
# Output: Important data

# Remove the container
docker rm -f my-app

# Start a new container from the same image
docker run --name my-app -d nginx
docker exec my-app cat /data/config.txt
# Output: cat: /data/config.txt: No such file or directory
# The data is gone.
```

This is by design. Immutable, disposable containers are a feature — they make deployments predictable and rollbacks easy. But applications need to persist data: databases, uploaded files, configuration, logs. We need a way to store data that outlives the container.

---

## 2. The Naive Way — Writing Data Inside the Container

The simplest approach is to just write data inside the container's filesystem and hope for the best.

```bash
# Naive: data lives inside the container
docker run --name postgres-naive -d \
  -e POSTGRES_PASSWORD=secret \
  postgres:16

# Insert data
docker exec postgres-naive psql -U postgres -c \
  "CREATE TABLE users (id serial PRIMARY KEY, name text);"
docker exec postgres-naive psql -U postgres -c \
  "INSERT INTO users (name) VALUES ('Alice');"

# Stop and remove the container
docker rm -f postgres-naive

# Start fresh — data is gone
docker run --name postgres-naive -d \
  -e POSTGRES_PASSWORD=secret \
  postgres:16
docker exec postgres-naive psql -U postgres -c "SELECT * FROM users;"
# ERROR: relation "users" does not exist
```

**Why this fails:**
- Container removal deletes all data permanently
- Container crashes can corrupt the writable layer
- Cannot share data between containers
- Cannot back up data independently of the container
- Scaling means duplicating data or losing it

---

## 3. The Right Way — Docker Volumes and Bind Mounts

Docker provides three mechanisms to persist data outside the container's writable layer.

### 3.1 Volumes

Volumes are Docker-managed storage. They live on the host filesystem (typically under `/var/lib/docker/volumes/`) but are managed entirely by Docker. They are the preferred mechanism for persistent data.

```bash
# Create a named volume
docker volume create postgres-data

# Run postgres with the volume mounted
docker run --name postgres-vol -d \
  -e POSTGRES_PASSWORD=secret \
  -v postgres-data:/var/lib/postgresql/data \
  postgres:16

# Insert data
docker exec postgres-vol psql -U postgres -c \
  "CREATE TABLE users (id serial PRIMARY KEY, name text);"
docker exec postgres-vol psql -U postgres -c \
  "INSERT INTO users (name) VALUES ('Alice');"

# Remove the container
docker rm -f postgres-vol

# Start a new container with the same volume
docker run --name postgres-vol -d \
  -e POSTGRES_PASSWORD=secret \
  -v postgres-data:/var/lib/postgresql/data \
  postgres:16

# Data survives
docker exec postgres-vol psql -U postgres -c "SELECT * FROM users;"
#  id | name
# ----+-------
#   1 | Alice
```

**Why volumes work:**
- Data persists independently of container lifecycle
- Docker manages the storage location
- Volumes can be listed, inspected, and backed up
- Performance is better than bind mounts on macOS and Windows
- Volume drivers enable remote/external storage

### 3.2 Bind Mounts

Bind mounts map a specific host path into the container. You control the exact location on the host.

```bash
# Bind mount a host directory
mkdir -p /tmp/nginx-html

echo "<h1>Hello from bind mount</h1>" > /tmp/nginx-html/index.html

docker run --name nginx-bind -d \
  -p 8080:80 \
  -v /tmp/nginx-html:/usr/share/nginx/html:ro \
  nginx

# Edit the file on the host
echo "<h1>Updated from host</h1>" > /tmp/nginx-html/index.html

# The container sees the change immediately
curl http://localhost:8080
# Output: <h1>Updated from host</h1>
```

**When to use bind mounts:**
- Development: mounting source code for live reloading
- Configuration files that you edit on the host
- Sharing specific files (not entire volumes) with containers

**When NOT to use bind mounts:**
- Production database data (volumes are safer and more portable)
- Anything that needs to be portable across hosts
- When you don't need the host to directly access the files

### 3.3 tmpfs Mounts

tmpfs mounts store data in the host's memory only. Data is never written to disk.

```bash
# tmpfs mount — data lives in RAM only
docker run --name app-tmpfs -d \
  --tmpfs /app/cache:rw,size=100m \
  my-app

# Or with explicit mount syntax
docker run --name app-tmpfs2 -d \
  --mount type=tmpfs,target=/app/cache,tmpfs-size=104857600 \
  my-app
```

**When to use tmpfs:**
- Sensitive data that should never touch disk (encryption keys, tokens)
- Temporary scratch space that benefits from RAM speed
- Data that should be destroyed when the container stops

### 3.4 Comparison Table

```
Feature          | Volumes          | Bind Mounts      | tmpfs
-----------------+------------------+------------------+-----------------
Location         | Docker-managed   | Host path you    | Host memory
                 |                  | specify          |
Persistence      | Until explicitly | Until host       | Container
                 | deleted          | deletes it       | lifetime only
Performance      | Native (good)    | Varies by OS     | RAM speed
Portability      | High             | Low (host paths) | N/A
Host access      | Indirect         | Direct           | None
Use case         | Production data  | Dev, config      | Secrets, cache
```

---

## 4. The Production Way — How Real Systems Handle Data

### 4.1 Named Volumes with Explicit Lifecycle

In production, every volume has a name, a purpose, and a lifecycle policy.

```yaml
# docker-compose.yml
version: "3.8"

services:
  postgres:
    image: postgres:16
    volumes:
      - postgres-data:/var/lib/postgresql/data
      - ./init-scripts:/docker-entrypoint-initdb.d:ro
    environment:
      POSTGRES_PASSWORD_FILE: /run/secrets/db_password
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    volumes:
      - redis-data:/data
    command: redis-server --appendonly yes
    restart: unless-stopped

volumes:
  postgres-data:
    driver: local
    labels:
      com.example.backup: "daily"
      com.example.retention: "30d"
  redis-data:
    driver: local
    labels:
      com.example.backup: "hourly"
      com.example.retention: "7d"
```

### 4.2 Volume Drivers for External Storage

Docker volume drivers let you store data on external systems — NFS, AWS EBS, Azure Files, Ceph, and more.

```bash
# Install the Rex-Ray driver for AWS EBS
docker plugin install rexray/ebs \
  REXRAY_PREEMPT=true \
  EBS_ACCESSKEY=AKIAIOSFODNN7EXAMPLE \
  EBS_SECRETKEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY

# Create a volume backed by EBS
docker volume create \
  --driver rexray/ebs \
  --opt size=100 \
  --opt volumetype=gp3 \
  production-db-data

# Use it like any volume
docker run -d \
  -v production-db-data:/var/lib/postgresql/data \
  postgres:16
```

### 4.3 Backup and Restore

**Backup a volume:**

```bash
# Method 1: Run a temporary container to tar the volume
docker run --rm \
  -v postgres-data:/source:ro \
  -v $(pwd):/backup \
  alpine \
  tar czf /backup/postgres-data-$(date +%Y%m%d).tar.gz -C /source .

# Method 2: Use a dedicated backup container
docker run --rm \
  --volumes-from postgres-vol \
  -v $(pwd):/backup \
  alpine \
  tar czf /backup/postgres-backup.tar.gz /var/lib/postgresql/data
```

**Restore a volume:**

```bash
# Create a fresh volume
docker volume create postgres-data-restored

# Restore from backup
docker run --rm \
  -v postgres-data-restored:/target \
  -v $(pwd):/backup:ro \
  alpine \
  sh -c 'cd /target && tar xzf /backup/postgres-data-20240101.tar.gz'
```

**Automated backup script:**

```bash
#!/bin/bash
# backup-volumes.sh — Run via cron
set -euo pipefail

BACKUP_DIR="/backups/docker-volumes"
RETENTION_DAYS=30
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

# Get all volumes with backup label
for volume in $(docker volume ls --filter label=com.example.backup -q); do
    BACKUP_FREQ=$(docker volume inspect "$volume" \
        --format '{{index .Labels "com.example.backup"}}')

    echo "Backing up volume: $volume (frequency: $BACKUP_FREQ)"

    docker run --rm \
        -v "$volume":/source:ro \
        -v "$BACKUP_DIR":/backup \
        alpine \
        tar czf "/backup/${volume}-${TIMESTAMP}.tar.gz" -C /source .

    echo "Backup complete: ${volume}-${TIMESTAMP}.tar.gz"
done

# Prune old backups
find "$BACKUP_DIR" -name "*.tar.gz" -mtime +"$RETENTION_DAYS" -delete
echo "Pruned backups older than $RETENTION_DAYS days"
```

### 4.4 Database Persistence Patterns

**PostgreSQL with custom data directory:**

```yaml
services:
  postgres:
    image: postgres:16
    volumes:
      - pg-data:/var/lib/postgresql/data
      - pg-wal:/var/lib/postgresql/data/pg_wal  # Separate WAL for performance
      - ./postgresql.conf:/etc/postgresql/postgresql.conf:ro
    command: postgres -c config_file=/etc/postgresql/postgresql.conf
    environment:
      POSTGRES_PASSWORD_FILE: /run/secrets/db_password
    shm_size: '256mb'  # Important: shared memory for PostgreSQL

volumes:
  pg-data:
  pg-wal:
```

**MySQL with data and logs separated:**

```yaml
services:
  mysql:
    image: mysql:8
    volumes:
      - mysql-data:/var/lib/mysql
      - mysql-logs:/var/log/mysql
      - ./my.cnf:/etc/mysql/conf.d/custom.cnf:ro
    environment:
      MYSQL_ROOT_PASSWORD_FILE: /run/secrets/mysql_root_password
    command: >
      --innodb-buffer-pool-size=1G
      --slow-query-log=ON
      --slow-query-log-file=/var/log/mysql/slow.log

volumes:
  mysql-data:
  mysql-logs:
```

**Redis with persistence:**

```yaml
services:
  redis:
    image: redis:7-alpine
    volumes:
      - redis-data:/data
    command: >
      redis-server
      --appendonly yes
      --appendfsync everysec
      --maxmemory 256mb
      --maxmemory-policy allkeys-lru

volumes:
  redis-data:
```

---

## 5. Hands-On Lab — Volume Operations in Practice

### Lab 5.1: Volume Lifecycle

```bash
# 1. Create volumes with different drivers
docker volume create lab-named-vol
docker volume create lab-driver-vol --driver local --opt type=tmpfs --opt device=tmpfs

# 2. Inspect volumes
docker volume inspect lab-named-vol
docker volume ls

# 3. Write data from one container
docker run --rm -v lab-named-vol:/data alpine sh -c \
  'echo "Hello from container 1" > /data/greeting.txt'

# 4. Read data from another container
docker run --rm -v lab-named-vol:/data alpine cat /data/greeting.txt
# Output: Hello from container 1

# 5. Use --mount syntax (more explicit than -v)
docker run --rm \
  --mount type=volume,source=lab-named-vol,target=/data \
  alpine sh -c 'echo "Appended line" >> /data/greeting.txt'

# 6. Clean up
docker volume rm lab-named-vol lab-driver-vol
```

### Lab 5.2: Database Persistence

```bash
# 1. Create a volume for PostgreSQL
docker volume create lab-postgres-data

# 2. Start PostgreSQL
docker run --name lab-pg -d \
  -e POSTGRES_PASSWORD=labpass \
  -v lab-postgres-data:/var/lib/postgresql/data \
  -p 5432:5432 \
  postgres:16

# 3. Wait for PostgreSQL to be ready
sleep 5

# 4. Create a table and insert data
docker exec lab-pg psql -U postgres -c "
  CREATE TABLE lab_data (
    id serial PRIMARY KEY,
    message text NOT NULL,
    created_at timestamp DEFAULT now()
  );
  INSERT INTO lab_data (message) VALUES ('First entry'), ('Second entry');
"

# 5. Verify data
docker exec lab-pg psql -U postgres -c "SELECT * FROM lab_data;"

# 6. Stop and remove the container
docker rm -f lab-pg

# 7. Start a NEW container with the SAME volume
docker run --name lab-pg-new -d \
  -e POSTGRES_PASSWORD=labpass \
  -v lab-postgres-data:/var/lib/postgresql/data \
  -p 5432:5432 \
  postgres:16

sleep 5

# 8. Verify data survived
docker exec lab-pg-new psql -U postgres -c "SELECT * FROM lab_data;"
# Both rows are still there.

# 9. Clean up
docker rm -f lab-pg-new
docker volume rm lab-postgres-data
```

### Lab 5.3: Sharing Volumes Between Containers

```bash
# 1. Create a shared volume
docker volume create lab-shared

# 2. Start a writer container
docker run --name writer -d \
  -v lab-shared:/shared \
  alpine sh -c 'while true; do echo "$(date): heartbeat" >> /shared/log.txt; sleep 2; done'

# 3. Start a reader container (read-only access)
docker run --name reader -d \
  -v lab-shared:/shared:ro \
  alpine sh -c 'while true; do tail -1 /shared/log.txt; sleep 3; done'

# 4. Observe the reader receiving data from the writer
docker logs -f reader

# 5. Another pattern: sidecar log collector
docker run --rm -v lab-shared:/shared:ro alpine wc -l /shared/log.txt

# 6. Clean up
docker rm -f writer reader
docker volume rm lab-shared
```

### Lab 5.4: Backup and Restore

```bash
# 1. Create a volume with data
docker volume create lab-backup-source
docker run --rm -v lab-backup-source:/data alpine sh -c \
  'for i in $(seq 1 100); do echo "Record $i" >> /data/records.txt; done'

# 2. Backup the volume
mkdir -p /tmp/backups
docker run --rm \
  -v lab-backup-source:/source:ro \
  -v /tmp/backups:/backup \
  alpine \
  tar czf /backup/lab-backup.tar.gz -C /source .

# 3. Verify the backup
ls -lh /tmp/backups/lab-backup.tar.gz

# 4. Restore to a new volume
docker volume create lab-restored
docker run --rm \
  -v lab-restored:/target \
  -v /tmp/backups:/backup:ro \
  alpine \
  sh -c 'cd /target && tar xzf /backup/lab-backup.tar.gz'

# 5. Verify restoration
docker run --rm -v lab-restored:/data alpine wc -l /data/records.txt
# Output: 100

# 6. Clean up
docker volume rm lab-backup-source lab-restored
rm -rf /tmp/backups
```

### Lab 5.5: tmpfs for Sensitive Data

```bash
# 1. Run with tmpfs for temporary secrets
docker run --rm \
  --tmpfs /secrets:rw,noexec,nosuid,size=10m \
  alpine sh -c '
    echo "secret-api-key=abc123" > /secrets/config.env
    cat /secrets/config.env
    echo "Secret exists only in memory"
  '
# After the container exits, the secret is gone from memory.

# 2. Compare with a volume (data persists)
docker volume create lab-not-secret
docker run --rm -v lab-not-secret:/secrets alpine sh -c \
  echo "persisted-key=xyz" > /secrets/config.env

# The file is still on disk even after container exits
docker run --rm -v lab-not-secret:/secrets alpine cat /secrets/config.env
# Output: persisted-key=xyz

docker volume rm lab-not-secret
```

---

## 6. Limitation — What's the Next Problem?

Volumes solve data persistence, but they introduce new challenges:

**Security exposure.** Volumes are accessible by any container that mounts them. A compromised container can read or modify data in a shared volume. Bind mounts can expose sensitive host files if misconfigured.

**No access control by default.** Docker does not enforce which containers can access which volumes. There are no ACLs, no encryption at rest, and no audit logs for volume access.

**Secrets in volumes are still secrets.** Storing database passwords, API keys, or TLS certificates in a volume is better than baking them into an image, but they are still plaintext on disk and accessible to anyone who can mount the volume.

**The writable layer still exists.** Containers still have a writable layer. Applications that write to unexpected paths (not volume mount points) will lose that data. Containers that write excessively to the writable layer can fill up the host's storage.

**Network-mounted volumes add latency.** When using volume drivers backed by network storage (NFS, EBS over network), I/O latency increases. Database performance can degrade significantly if the volume driver introduces too much overhead.

The fundamental question becomes: **who is allowed to access this data, and how do we enforce that?** This leads directly to container security — controlling what a container can do, what it can access, and how to prevent malicious or accidental damage.

---

## 7. Next Topic — Container Security

With data persisting in volumes and containers communicating over networks, the next critical concern is security. Module 10 covers:

- Running containers as non-root users to limit blast radius
- Read-only filesystems to prevent runtime tampering
- Dropping Linux capabilities to reduce the attack surface
- Security profiles (AppArmor, Seccomp) for fine-grained control
- Image scanning for known vulnerabilities
- Image signing for supply chain integrity
- Secrets management without environment variables
- Container escape techniques and prevention

**Next:** [Module 10 — Container Security](../10-container-security/README.md)
