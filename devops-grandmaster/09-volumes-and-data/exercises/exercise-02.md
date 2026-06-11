# Exercise 02: Persisting Database Data

**Type:** Guided
**Estimated Time:** 30 minutes

## Objective

Use a **named volume** to persist PostgreSQL data so that the database survives
container removal and can be reattached to a new container.

## Instructions

### Step 1 -- Create a Named Volume

```bash
docker volume create pgdata
```

Verify the volume exists:

```bash
docker volume ls
docker volume inspect pgdata
```

### Step 2 -- Run PostgreSQL with the Volume

Start a PostgreSQL container that stores its data in the `pgdata` volume:

```bash
docker run -d \
  --name pg-1 \
  -e POSTGRES_PASSWORD=exercise02 \
  -e POSTGRES_DB=appdb \
  -v pgdata:/var/lib/postgresql/data \
  postgres:16-alpine
```

Wait a few seconds for the database to initialize, then confirm it is ready:

```bash
docker logs pg-1 2>&1 | tail -5
```

You should see a line similar to:
`database system is ready to accept connections`

### Step 3 -- Write Data into the Database

Exec into the container and create a table with sample data:

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

Confirm the data is there:

```bash
docker exec -it pg-1 psql -U postgres -d appdb -c "SELECT * FROM users;"
```

### Step 4 -- Destroy the Container

```bash
docker stop pg-1
docker rm pg-1
```

Confirm the container is gone but the volume remains:

```bash
docker ps -a --filter name=pg-1
docker volume ls
```

### Step 5 -- Recreate the Container with the Same Volume

```bash
docker run -d \
  --name pg-2 \
  -e POSTGRES_PASSWORD=exercise02 \
  -e POSTGRES_DB=appdb \
  -v pgdata:/var/lib/postgresql/data \
  postgres:16-alpine
```

### Step 6 -- Verify Data Persistence

```bash
docker exec -it pg-2 psql -U postgres -d appdb -c "SELECT * FROM users;"
```

The three rows (Alice, Bob, Carol) must still be present.

### Step 7 -- Clean Up

```bash
docker stop pg-2
docker rm pg-2
docker volume rm pgdata
```

## Success Criteria

- [ ] After removing `pg-1` and starting `pg-2` with the same volume, the
      `users` table and all three rows are intact.
- [ ] You can explain why the data survived (it lived in the volume, not the
      container's writable layer).
- [ ] All resources are cleaned up at the end.

## Hints

<details>
<summary>Hint 1 -- Why not a bind mount here?</summary>

A bind mount would also work, but named volumes are the recommended approach for
database data because:

- Docker manages the storage location, so you do not need to worry about
  directory permissions on the host.
- Named volumes can be listed, inspected, and removed with Docker CLI commands.
- They are portable across different host directory structures.

</details>

<details>
<summary>Hint 2 -- What happens if you forget -v?</summary>

If you start a PostgreSQL container without a volume or bind mount, Docker
creates an anonymous volume. That volume is harder to identify by name and may
be garbage-collected when the container is removed with `docker rm` (unless you
pass `--volumes` to `docker rm` to force it).

</details>

<details>
<summary>Hint 3 -- POSTGRES_DB matters</summary>

The `POSTGRES_DB` environment variable tells PostgreSQL to create a database
with that name on first initialization. If you omit it, the default database
is `postgres`, and your `psql` commands need to reference that name instead.

</details>
