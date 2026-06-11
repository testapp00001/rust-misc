# Solution 02: Persisting Database Data

## Complete Walkthrough

### Step 1 -- Create the Volume

```bash
docker volume create pgdata
```

Verify:

```bash
docker volume ls
# Output should include a line like:
# local    pgdata

docker volume inspect pgdata
# Shows the mountpoint (usually /var/lib/docker/volumes/pgdata/_data)
# and driver (local).
```

### Step 2 -- Start PostgreSQL

```bash
docker run -d \
  --name pg-1 \
  -e POSTGRES_PASSWORD=exercise02 \
  -e POSTGRES_DB=appdb \
  -v pgdata:/var/lib/postgresql/data \
  postgres:16-alpine
```

Wait for readiness:

```bash
docker logs pg-1 2>&1 | tail -5
# Expect: "database system is ready to accept connections"
```

### Step 3 -- Insert Data

```bash
docker exec -it pg-1 psql -U postgres -d appdb -c "
  CREATE TABLE users (
    id    SERIAL PRIMARY KEY,
    name  TEXT NOT NULL,
    email TEXT NOT NULL
  );
  INSERT INTO users (name, email) VALUES
    ('Alice', 'alice@example.com'),
    ('Bob',   'bob@example.com'),
    ('Carol', 'carol@example.com');
"
```

Verify:

```bash
docker exec -it pg-1 psql -U postgres -d appdb -c "SELECT * FROM users;"
#  id | name  |       email
# ----+-------+-------------------
#   1 | Alice | alice@example.com
#   2 | Bob   | bob@example.com
#   3 | Carol | carol@example.com
# (3 rows)
```

### Step 4 -- Remove the Container

```bash
docker stop pg-1
docker rm pg-1
```

Confirm the container is gone but the volume remains:

```bash
docker ps -a --filter name=pg-1
# No output -- container is fully removed.

docker volume ls
# pgdata still listed.
```

### Step 5 -- Recreate with the Same Volume

```bash
docker run -d \
  --name pg-2 \
  -e POSTGRES_PASSWORD=exercise02 \
  -e POSTGRES_DB=appdb \
  -v pgdata:/var/lib/postgresql/data \
  postgres:16-alpine
```

### Step 6 -- Verify Persistence

```bash
docker exec -it pg-2 psql -U postgres -d appdb -c "SELECT * FROM users;"
# Same 3 rows as before.
```

### Step 7 -- Clean Up

```bash
docker stop pg-2
docker rm pg-2
docker volume rm pgdata
```

## Why It Works

When you mount a named volume at `/var/lib/postgresql/data`, PostgreSQL writes
all its data files (WAL logs, base backup, configuration) into that volume
instead of the container's writable layer.

When you `docker rm pg-1`, Docker removes the container's writable layer but
leaves the named volume intact. The volume is a Docker-managed directory on the
host (`/var/lib/docker/volumes/pgdata/_data`) that persists independently.

When you start `pg-2` with `-v pgdata:/var/lib/postgresql/data`, Docker
attaches the same volume to the new container. PostgreSQL opens the data
directory, finds its existing database files, and starts normally -- no
re-initialization happens because `POSTGRES_DB` and `POSTGRES_PASSWORD` are
only used during the *first* initialization (when the data directory is empty).

The key insight: **the volume is the source of truth, not the container.**
Containers are ephemeral; volumes are persistent.

## Common Mistakes

1. **Forgetting `-v` on the second container:** If you start `pg-2` without
   `-v pgdata:/var/lib/postgresql/data`, PostgreSQL creates a fresh (anonymous)
   data directory and you lose access to your data. The `pgdata` volume still
   exists but is orphaned.

2. **Mismatched `POSTGRES_DB`:** The `POSTGRES_DB` variable only matters on
   first initialization. If you start `pg-2` with `POSTGRES_DB=otherdb`, it
   has no effect because the data directory already exists. The database name
   is `appdb` because that is what was set during the first run.

3. **Confusing `docker rm` with `docker volume rm`:** `docker rm` removes the
   container. `docker volume rm` removes the volume. Running `docker rm pg-1`
   does NOT delete the `pgdata` volume. You must explicitly delete the volume
   when you are done with it.

4. **Using `docker compose down -v` accidentally:** The `-v` flag on
   `docker compose down` removes all named volumes declared in the compose
   file. If you use it when you intended to keep the data, you will lose
   everything.

5. **Not waiting for PostgreSQL to initialize:** On first run, PostgreSQL
   takes a few seconds to initialize the database. If you immediately exec
   into the container and run `psql`, you may get connection errors. Use
   `docker logs` to check for the "ready to accept connections" message, or
   use a healthcheck with `pg_isready`.
