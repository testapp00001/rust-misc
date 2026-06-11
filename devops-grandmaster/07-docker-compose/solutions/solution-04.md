# Solution 04: Developer Paradise -- Hot-Reload and Test Databases

## Part A: Base Compose File

**docker-compose.yml:**

```yaml
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: ${POSTGRES_DB:-taskdb}
      POSTGRES_USER: taskuser
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - pgdata:/var/lib/postgresql/data
      - ./db/init.sql:/docker-entrypoint-initdb.d/init.sql:ro
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U taskuser -d ${POSTGRES_DB:-taskdb}"]
      interval: 5s
      timeout: 3s
      retries: 5
      start_period: 10s

  redis:
    image: redis:7-alpine
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 5

  api:
    build:
      context: ./api
      dockerfile: Dockerfile
    environment:
      DATABASE_URL: postgresql://taskuser:${POSTGRES_PASSWORD}@postgres:5432/${POSTGRES_DB:-taskdb}
      REDIS_URL: redis://redis:6379
      NODE_ENV: production
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    restart: unless-stopped

volumes:
  pgdata:
```

### Why This Works

**No ports exposed**: The base file does not expose any ports to the host. This is
the production-safe default. Services communicate over the internal Compose network
only.

**Variable interpolation with defaults**: `${POSTGRES_DB:-taskdb}` uses the value
of `POSTGRES_DB` from the `.env` file, or falls back to `taskdb` if the variable
is not set. This allows per-developer customization without breaking the default.

**Init script mounted read-only**: The `:ro` flag on the init.sql mount prevents
the container from modifying the file. PostgreSQL reads files in
`/docker-entrypoint-initdb.d/` on first start only.

**No `ports:` directive**: This is intentional. In production, a reverse proxy
handles external traffic. In development, the override file adds port mappings.

---

## Part B: Development Override File

**docker-compose.override.yml:**

```yaml
services:
  postgres:
    ports:
      - "5432:5432"

  redis:
    ports:
      - "6379:6379"

  api:
    volumes:
      - ./api/src:/app/src
    command: npx nodemon src/index.js
    environment:
      DATABASE_URL: postgresql://taskuser:${POSTGRES_PASSWORD}@postgres:5432/${POSTGRES_DB:-taskdb}
      REDIS_URL: redis://redis:6379
      NODE_ENV: development
      DEBUG: "*"
    ports:
      - "3000:3000"

  adminer:
    image: adminer
    ports:
      - "8081:8080"
    depends_on:
      postgres:
        condition: service_healthy

  redis-commander:
    image: rediscommander/redis-commander
    ports:
      - "8082:8081"
    environment:
      REDIS_HOSTS: local:redis:6379
    depends_on:
      - redis
```

### Why This Works

**Hot reload via bind mount**: `./api/src:/app/src` mounts the developer's local
source directory into the container. When the developer edits a file on their host,
the change is immediately visible inside the container. `nodemon` watches for file
changes and restarts the Node.js process automatically. No rebuild required.

**Port exposure for debugging**: Developers connect pgAdmin, DBeaver, or `psql`
directly to `localhost:5432`. Redis desktop managers connect to `localhost:6379`.
This is only for development -- the base file keeps these closed.

**Debug environment variables**: `NODE_ENV=development` enables verbose error
messages and stack traces. `DEBUG=*` enables debug logging in many Node.js
libraries. These would leak information in production but are invaluable during
development.

**Adminer and Redis Commander**: These are lightweight web UIs for inspecting
the database and cache. They are defined in the override file so they only run
in development. Developers access them at `http://localhost:8081` (Adminer) and
`http://localhost:8082` (Redis Commander).

**Override merging**: Compose merges the override file with the base file. Service
definitions are merged by key. For example, the `api` service's `environment` in
the override replaces the `environment` in the base. The `volumes` in the override
are appended to the base volumes.

---

## Part C: Test Database Isolation

### How .env Interacts with Compose

The `.env` file in the same directory as `docker-compose.yml` is automatically read
by Compose. Variables are interpolated in the YAML file using `${VARIABLE_NAME}`
syntax. This happens *before* the YAML is parsed -- the variables are text
substitution, not runtime environment variables.

### Preventing Developer Collisions

When two developers use the same database name and connect to the same PostgreSQL
instance, their tests collide. The solution is per-developer database names:

**Alice's .env:**
```
POSTGRES_PASSWORD=alice_secret_2024
POSTGRES_DB=taskdb_alice
```

**Bob's .env:**
```
POSTGRES_PASSWORD=bob_secret_2024
POSTGRES_DB=taskdb_bob
```

Each developer gets their own isolated database. The `init.sql` script runs for
each database on first start.

### Test Database Cleanup

For test isolation, add a separate Compose file for tests:

**docker-compose.test.yml:**
```yaml
services:
  postgres:
    environment:
      POSTGRES_DB: testdb

  api:
    environment:
      DATABASE_URL: postgresql://taskuser:taskpass@postgres:5432/testdb
    command: npm test
```

Run tests with:

```bash
docker compose -f docker-compose.yml -f docker-compose.test.yml up --abort-on-container-exit
docker compose down -v  # Clean up test data
```

The `--abort-on-container-exit` flag stops all services when the test command
exits. The `down -v` removes the test database volume.

---

## Part D: Seed Data

**db/init.sql:**

```sql
-- Create tasks table if it does not exist
CREATE TABLE IF NOT EXISTS tasks (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    completed BOOLEAN DEFAULT false,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert sample data only if the table is empty
INSERT INTO tasks (title, completed)
SELECT 'Learn Docker Compose', false
WHERE NOT EXISTS (SELECT 1 FROM tasks);

INSERT INTO tasks (title, completed)
SELECT 'Set up hot reload', false
WHERE NOT EXISTS (SELECT 1 FROM tasks);

INSERT INTO tasks (title, completed)
SELECT 'Write integration tests', false
WHERE NOT EXISTS (SELECT 1 FROM tasks);
```

### Why This Works

**`IF NOT EXISTS`**: The `CREATE TABLE` statement is idempotent. If the table
already exists, the statement is a no-op. This prevents errors if the script runs
multiple times.

**`WHERE NOT EXISTS`**: The `INSERT` statements check whether data already exists
before inserting. This prevents duplicate seed data on subsequent runs.

**`/docker-entrypoint-initdb.d/`**: PostgreSQL runs all `.sql`, `.sql.gz`, and
`.sh` files in this directory on first start (when the data directory is empty).
Once the data volume has been initialized, these scripts are not re-run. To re-run
them, remove the volume with `docker compose down -v` and start again.

**Read-only mount**: The `:ro` flag in the volume mount prevents accidental
modification of the init script from inside the container.

---

## Part E: Verification

### Development Mode

```bash
# Start with override (automatic)
docker compose up -d

# Check all services including debug tools
docker compose ps
```

Expected output:

```
NAME                    IMAGE                         STATUS          PORTS
myproject-api-1         myproject-api                 Up 1 min        0.0.0.0:3000->3000/tcp
myproject-postgres-1    postgres:16-alpine            Up 2 min        0.0.0.0:5432->5432/tcp
myproject-redis-1       redis:7-alpine                Up 2 min        0.0.0.0:6379->6379/tcp
myproject-adminer-1     adminer                       Up 1 min        0.0.0.0:8081->8080/tcp
myproject-redis-cmd-1   rediscommander/redis-commander Up 1 min       0.0.0.0:8082->8081/tcp
```

### Hot Reload Test

1. Edit a file in `./api/src/`.
2. Watch `docker compose logs -f api`.
3. You should see `nodemon` detect the change and restart the process.

### Production Mode (No Override)

```bash
docker compose down
docker compose -f docker-compose.yml up -d
docker compose ps
```

Expected output -- no debug tools, no exposed ports (except the API if you
choose to expose it):

```
NAME                  IMAGE               STATUS          PORTS
myproject-api-1       myproject-api       Up 1 min
myproject-postgres-1  postgres:16-alpine  Up 2 min        5432/tcp
myproject-redis-1     redis:7-alpine      Up 2 min        6379/tcp
```

Notice no ports are mapped to the host. The services are only reachable from
within the Compose network.

---

## Common Mistakes

### 1. Override Replaces Instead of Merging

When both the base and override define the same key for the same service, the
override wins entirely for that key. For example, if the base sets `environment`
with `DATABASE_URL` and `REDIS_URL`, and the override sets `environment` with
only `NODE_ENV`, the final result has only `NODE_ENV`. You must repeat all
environment variables in the override.

To avoid this, only override the keys that change. Or use `env_file` for shared
variables.

### 2. Forgetting nodemon in the Image

The bind mount makes files available, but something must watch for changes and
restart the process. If the base Dockerfile runs `node server.js`, the files
change but the process does not restart. You need either:

- `nodemon` installed in the image and used as the command
- `node --watch` (Node.js 18+ built-in file watcher)
- A Dockerfile with nodemon: `RUN npm install -g nodemon`

### 3. Init Script Not Idempotent

```sql
-- WRONG -- fails on second run
INSERT INTO tasks (title) VALUES ('Learn Docker Compose');

-- RIGHT -- checks before inserting
INSERT INTO tasks (title)
SELECT 'Learn Docker Compose'
WHERE NOT EXISTS (SELECT 1 FROM tasks WHERE title = 'Learn Docker Compose');
```

PostgreSQL only runs init scripts on first start, but during development you
frequently `docker compose down -v` and restart. If the script is not idempotent,
it may fail in unexpected ways.

### 4. Committing .env to Git

The `.env` file contains passwords and developer-specific settings. It should
be in `.gitignore`:

```gitignore
.env
docker-compose.override.yml
```

Provide a `.env.example` with placeholder values instead.

## Relevant README Sections

- [Override Files](../README.md#override-files)
- [Environment Variables](../README.md#environment-variables)
- [Compose Profiles](../README.md#compose-profiles)
- [Compose File Best Practices](../README.md#compose-file-best-practices)
