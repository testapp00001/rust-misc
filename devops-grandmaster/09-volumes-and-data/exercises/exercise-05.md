# Exercise 05: Multi-Service Compose with Volumes

**Type:** Integration
**Estimated Time:** 45 minutes

## Objective

Combine **named volumes**, **bind mounts**, and **tmpfs mounts** in a single
`docker-compose.yml` file to orchestrate a multi-service application with proper
data persistence.

## Scenario

You will build a compose file for a three-service stack:

1. **db** -- PostgreSQL 16 (named volume for data)
2. **redis** -- Redis 7 (tmpfs for ephemeral cache)
3. **app** -- A Python/Flask application that:
   - Reads configuration from a bind-mounted file.
   - Stores user uploads in a named volume shared with a helper container.
   - Uses `/tmp` as tmpfs for temporary file processing.

A fourth service, **backup**, runs `pg_dump` on demand to back up the database.

## Instructions

### Step 1 -- Create the Project Structure

```bash
mkdir -p taskboard-compose/config
cd taskboard-compose
```

Create a minimal config file:

```bash
cat > config/app.conf <<'EOF'
APP_ENV=development
DB_HOST=db
DB_PORT=5432
REDIS_HOST=redis
UPLOAD_DIR=/app/uploads
EOF
```

### Step 2 -- Write the Compose File

Create `docker-compose.yml` with the following requirements:

```
Services:
  db:
    image: postgres:16-alpine
    environment:
      POSTGRES_PASSWORD: taskboard
      POSTGRES_DB: taskboard
    volumes:
      # 1. Named volume for database data
      # 2. Named volume for database backups (used by backup service)
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s
      timeout: 3s
      retries: 5

  redis:
    image: redis:7-alpine
    # tmpfs mount for ephemeral data

  app:
    image: nginx:alpine  # placeholder for the real app
    ports:
      - "8080:80"
    volumes:
      # 1. Bind mount for config (read-only)
      # 2. Named volume for uploads
      # 3. tmpfs for temporary processing
    depends_on:
      db:
        condition: service_healthy

  backup:
    image: postgres:16-alpine
    volumes:
      # 1. Named volume for backups (shared with db)
      # 2. Bind mount to host for output
    entrypoint: >
      sh -c '
        pg_dump -h db -U postgres taskboard > /backups/taskboard-$$(date +%Y%m%d-%H%M%S).sql
        echo "Backup saved to /backups"
      '
    depends_on:
      db:
        condition: service_healthy
    profiles:
      - backup

Volumes:
  # Declare named volumes here

# No top-level tmpfs key -- tmpfs is per-service
```

Fill in the volume mounts and volume declarations. Use the comments as guides.

### Step 3 -- Start the Stack

```bash
docker compose up -d
```

Verify all services are running:

```bash
docker compose ps
```

### Step 4 -- Test Data Persistence

Write some data into PostgreSQL:

```bash
docker compose exec db psql -U postgres -d taskboard -c "
  CREATE TABLE tasks (id SERIAL PRIMARY KEY, title TEXT, done BOOLEAN DEFAULT false);
  INSERT INTO tasks (title) VALUES ('Learn volumes'), ('Learn compose');
"
```

Restart the stack:

```bash
docker compose down
docker compose up -d
```

Verify the data survives:

```bash
docker compose exec db psql -U postgres -d taskboard -c "SELECT * FROM tasks;"
```

### Step 5 -- Test the Backup Service

Run the backup profile:

```bash
docker compose --profile backup run --rm backup
```

Confirm the backup file exists on the host:

```bash
ls -lh ./backups/
cat ./backups/taskboard-*.sql | head -20
```

### Step 6 -- Test the Config Bind Mount

Modify the config file on the host:

```bash
echo "NEW_SETTING=true" >> config/app.conf
```

Restart the app service and verify the change is visible inside the container:

```bash
docker compose restart app
docker compose exec app cat /app/config/app.conf
```

### Step 7 -- Clean Up

```bash
docker compose down -v
rm -rf ./backups
```

## Success Criteria

- [ ] The `db` service uses a named volume that survives `docker compose down`
      (without `-v`) and `docker compose up`.
- [ ] The `redis` service uses a tmpfs mount.
- [ ] The `app` service reads config from a bind mount, uses a named volume
      for uploads, and uses tmpfs for `/tmp`.
- [ ] The `backup` service runs on demand (via `--profile backup`) and
      produces a `.sql` file on the host.
- [ ] The config file change on the host is reflected in the running container
      after a restart.
- [ ] All resources are cleaned up with `docker compose down -v`.

## Hints

<details>
<summary>Hint 1 -- Named volume declaration</summary>

At the bottom of the compose file, declare named volumes under the top-level
`volumes` key:

```yaml
volumes:
  pgdata:
  backups:
  uploads:
```

Each named volume referenced in a service's `volumes` section must be declared
here (or Docker Compose creates it automatically, but explicit declaration is
best practice).

</details>

<details>
<summary>Hint 2 -- tmpfs in Compose</summary>

Use the `tmpfs` key (not `volumes`) for tmpfs mounts in Compose:

```yaml
services:
  redis:
    image: redis:7-alpine
    tmpfs:
      - /data
```

Or use the long-form `volumes` syntax:

```yaml
volumes:
  - type: tmpfs
    target: /data
```

</details>

<details>
<summary>Hint 3 -- Read-only bind mounts</summary>

Append `:ro` to the short-form bind mount, or use `read_only: true` in
long-form:

```yaml
volumes:
  - ./config/app.conf:/app/config/app.conf:ro
```

</details>

<details>
<summary>Hint 4 -- Backup service entrypoint</summary>

The backup service shares the `backups` volume with the `db` service. But it
connects to the database over the network (`-h db`), not by reading the volume
directly. The volume is only used to place the output `.sql` file where the
`db` service can also see it if needed. The bind mount to `./backups` on the
host makes the file accessible outside Docker.

</details>
