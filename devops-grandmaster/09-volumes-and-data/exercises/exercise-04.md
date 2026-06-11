# Exercise 04: Stateful Application Data Design

**Type:** Challenge
**Estimated Time:** 60 minutes

## Objective

Design a complete **data management strategy** for a stateful application with
multiple data types, each with different persistence and performance
requirements. You will decide which storage type to use, write the Docker
commands, and document your rationale.

## Scenario

You are deploying **"TaskBoard"**, a project management application with the
following components:

| Component | Data | Characteristics |
|-----------|------|-----------------|
| PostgreSQL | Structured relational data | Must survive container restarts and host reboots. Must be backed up nightly. |
| Redis | Session cache, rate-limiter state | Ephemeral -- acceptable to lose on restart, but must be fast. |
| Application uploads | User-uploaded files (images, PDFs) | Must persist. Must be accessible from the host for processing by an external tool. |
| Application config | `config.yaml` | Lives on the host. Developers edit it from the host; the container must read it. |
| Temporary processing | Intermediate files during PDF conversion | Sensitive -- must never touch disk. Lost on restart is fine. |

## Instructions

### Part A -- Storage Decision Matrix

Complete the table below. For each component, choose a storage type (named
volume, bind mount, or tmpfs) and justify your choice.

```
| Component        | Storage Type | Mount Point            | Justification |
|------------------|--------------|------------------------|---------------|
| PostgreSQL data  |              | /var/lib/postgresql/data|               |
| Redis data       |              | /data                  |               |
| Application uploads |          | /app/uploads           |               |
| Application config |            | /app/config            |               |
| Temporary files  |              | /tmp/taskboard         |               |
```

### Part B -- Implement with Docker Commands

Write the `docker run` command for each component. Use appropriate flags for
each storage type.

**PostgreSQL:**

```bash
docker volume create taskboard-pgdata

docker run -d \
  --name taskboard-db \
  -e POSTGRES_PASSWORD=secret \
  -e POSTGRES_DB=taskboard \
  # Add volume mount here
  postgres:16-alpine
```

**Redis:**

```bash
docker run -d \
  --name taskboard-redis \
  # Add the appropriate mount here
  redis:7-alpine
```

**Application uploads (bind mount from host):**

```bash
mkdir -p ./uploads

docker run -d \
  --name taskboard-app \
  # Add bind mount for uploads
  # Add bind mount for config
  # Add tmpfs mount for temp processing
  my-taskboard-app:latest
```

### Part C -- Write a Backup Script

Create a shell script `backup-taskboard.sh` that:

1. Backs up the PostgreSQL database using `pg_dump` (not volume tar -- explain
   why `pg_dump` is preferred for databases).
2. Backs up the uploads volume to a tar archive.
3. Timestamps both backup files.
4. Stores them in a `./backups` directory on the host.

Write the script below:

```bash
#!/usr/bin/env bash
set -euo pipefail

BACKUP_DIR="./backups"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

# Create backup directory
# ...

# Backup PostgreSQL
# ...

# Backup uploads
# ...

echo "Backup complete: ${BACKUP_DIR}"
```

### Part D -- Documentation

Write a short README section (5-10 bullet points) that a new team member could
follow to:

1. Start all TaskBoard services with correct volume mounts.
2. Restore from a backup.
3. Understand which data is safe to lose and which is not.

## Success Criteria

- [ ] Every component uses the correct storage type for its characteristics.
- [ ] PostgreSQL uses a named volume, not a bind mount.
- [ ] The config file uses a bind mount from the host, not a volume.
- [ ] Temporary processing files use tmpfs with a size limit.
- [ ] The backup script uses `pg_dump` for the database and `tar` for uploads.
- [ ] The backup script produces timestamped files.
- [ ] Your documentation is clear enough for someone else to follow.

## Hints

<details>
<summary>Hint 1 -- Why pg_dump over tar for databases?</summary>

`pg_dump` produces a consistent SQL dump that can be restored to any PostgreSQL
version. A tar archive of the raw data directory is version-specific, may
contain in-flight transactions, and can be corrupted if the database is running
during the backup. Always use `pg_dump` for relational databases.

</details>

<details>
<summary>Hint 2 -- tmpfs size limit</summary>

Use the `tmpfs-size` option to prevent the container from consuming all host
memory:

```
--mount type=tmpfs,dst=/tmp/taskboard,tmpfs-size=100m
```

</details>

<details>
<summary>Hint 3 -- Host directory for uploads</summary>

Using a bind mount for uploads (`-v $(pwd)/uploads:/app/uploads`) lets an
external tool on the host process uploaded files (e.g., image thumbnailing,
virus scanning) without entering the container. This is a common pattern for
file-processing pipelines.

</details>

<details>
<summary>Hint 4 -- Backup script structure</summary>

For PostgreSQL, exec into the running container:

```bash
docker exec taskboard-db pg_dump -U postgres taskboard > "${BACKUP_DIR}/pg-${TIMESTAMP}.sql"
```

For uploads, use the tar-from-a-container pattern from Exercise 03.

</details>
