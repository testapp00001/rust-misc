# Solution 05: Multi-Service Compose with Volumes

## Complete `docker-compose.yml`

```yaml
services:
  db:
    image: postgres:16-alpine
    environment:
      POSTGRES_PASSWORD: taskboard
      POSTGRES_DB: taskboard
    volumes:
      - pgdata:/var/lib/postgresql/data    # Named volume for database files
      - backups:/backups                   # Named volume shared with backup service
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s
      timeout: 3s
      retries: 5

  redis:
    image: redis:7-alpine
    tmpfs:
      - /data                              # Ephemeral cache data in RAM

  app:
    image: nginx:alpine
    ports:
      - "8080:80"
    volumes:
      - ./config/app.conf:/app/config/app.conf:ro   # Bind mount for config (read-only)
      - uploads:/app/uploads                         # Named volume for uploads
      - type: tmpfs                                  # tmpfs for temp processing
        target: /tmp
        tmpfs:
          size: 100000000          # ~100MB
    depends_on:
      db:
        condition: service_healthy

  backup:
    image: postgres:16-alpine
    volumes:
      - backups:/backups                   # Shared volume with db service
      - ./backups:/output                  # Bind mount to host for final output
    entrypoint: >
      sh -c '
        pg_dump -h db -U postgres taskboard > /output/taskboard-$$(date +%Y%m%d-%H%M%S).sql
        echo "Backup saved to /output"
      '
    depends_on:
      db:
        condition: service_healthy
    profiles:
      - backup

volumes:
  pgdata:
  backups:
  uploads:
```

## Step-by-Step Walkthrough

### Step 1 -- Project Structure

```bash
mkdir -p taskboard-compose/config
cd taskboard-compose
```

```bash
cat > config/app.conf <<'EOF'
APP_ENV=development
DB_HOST=db
DB_PORT=5432
REDIS_HOST=redis
UPLOAD_DIR=/app/uploads
EOF
```

### Step 2 -- Start the Stack

```bash
docker compose up -d
```

Verify:

```bash
docker compose ps
# NAME                SERVICE   STATUS
# taskboard-db-1      db        running (healthy)
# taskboard-redis-1   redis     running
# taskboard-app-1     app       running
```

The backup service does not appear because it uses `profiles: [backup]` --
it only starts when explicitly invoked.

### Step 3 -- Test Data Persistence

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

Verify data survived:

```bash
docker compose exec db psql -U postgres -d taskboard -c "SELECT * FROM tasks;"
#  id |      title      | done
# ----+-----------------+------
#   1 | Learn volumes   | f
#   2 | Learn compose   | f
# (2 rows)
```

The data survived because the `pgdata` named volume persists across
`docker compose down` (without `-v`). The `down` command removes containers and
networks but leaves volumes intact by default.

### Step 4 -- Test the Backup Service

```bash
docker compose --profile backup run --rm backup
```

Verify:

```bash
ls -lh ./backups/
cat ./backups/taskboard-*.sql | head -20
```

The `.sql` file should contain PostgreSQL dump output starting with comments
and `SET` statements.

### Step 5 -- Test Config Bind Mount

```bash
echo "NEW_SETTING=true" >> config/app.conf
docker compose restart app
docker compose exec app cat /app/config/app.conf
```

The new setting should appear in the output. Since it is a bind mount, the
container sees the host file directly. After a restart (which re-reads the
config), the change is reflected.

### Step 6 -- Clean Up

```bash
docker compose down -v
rm -rf ./backups
```

The `-v` flag removes all named volumes declared in the compose file
(`pgdata`, `backups`, `uploads`).

## Why It Works

### Volume Types in Compose

**Named volumes** (`pgdata`, `backups`, `uploads`): Declared at the top level
of the compose file and referenced in service `volumes` sections. Docker
manages their lifecycle. They persist across `docker compose down` and are
removed only by `docker compose down -v` or `docker volume rm`.

**Bind mounts** (`./config/app.conf`): Referenced with a host path in the
service `volumes` section. The host path is relative to the compose file's
directory. Changes on the host are immediately visible in the container.

**tmpfs** (redis `/data`, app `/tmp`): Declared with the `tmpfs` key or the
long-form `volumes` syntax with `type: tmpfs`. Data lives in RAM and is lost
when the container stops. Cannot be shared between services.

### Profiles

The `backup` service uses `profiles: [backup]`. This means:

- `docker compose up` does NOT start it.
- `docker compose --profile backup run --rm backup` explicitly invokes it.
- This is the correct pattern for on-demand operations (backups, migrations,
  one-off scripts) that should not run with the main stack.

### Healthchecks and `depends_on`

The `db` service has a healthcheck that runs `pg_isready` every 5 seconds.
The `app` and `backup` services use `depends_on` with
`condition: service_healthy`, which means they wait until the healthcheck
passes before starting. This prevents the common race condition where the app
tries to connect to PostgreSQL before it is ready.

### Shared Volumes

The `backups` volume is mounted in both `db` and `backup` services. This is
one way to share data between services, but in this exercise the backup
service actually connects to PostgreSQL over the network (`-h db`) rather than
reading the data directory directly. The `backups` volume is used to stage the
dump file, and the bind mount (`./backups:/output`) copies it to the host.

## Common Mistakes

1. **Forgetting to declare named volumes at the top level:** If you reference
   a named volume in a service's `volumes` section but do not declare it under
   the top-level `volumes` key, Docker Compose creates it automatically (with
   a prefixed name like `taskboard-compose_pgdata`). This works but makes the
   volume name unpredictable and harder to manage. Always declare volumes
   explicitly.

2. **Using `docker compose down -v` when you want to keep data:** The `-v`
   flag removes all named volumes. If you run it accidentally, your database
   data, backups, and uploads are gone. Use `docker compose down` (without
   `-v`) for routine restarts.

3. **Not using healthchecks for database dependencies:** Without a healthcheck
   and `condition: service_healthy`, the app may start before PostgreSQL is
   ready, causing connection errors. The default `depends_on` only waits for
   the container to start, not for the service inside to be ready.

4. **Confusing `tmpfs` key with `volumes` for tmpfs mounts:** In Compose, the
   `tmpfs` key at the service level is the simplest way to add a tmpfs mount.
   You can also use the long-form `volumes` syntax with `type: tmpfs`, but
   mixing the two styles in the same service can be confusing. Pick one
   approach and be consistent.

5. **Bind mount path resolution:** In Compose, relative paths in bind mounts
   (e.g., `./config/app.conf`) are resolved relative to the directory
   containing the `docker-compose.yml` file, not relative to the current
   working directory. If you run `docker compose -f /path/to/docker-compose.yml
   up`, the bind mount still resolves relative to `/path/to/`.

6. **Not using `--rm` for one-off services:** The backup service should run
   and exit. Without `--rm`, the stopped backup container lingers and clutters
   `docker compose ps -a`. Always use `--rm` for one-off tasks.
