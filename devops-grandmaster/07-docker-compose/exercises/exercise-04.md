# Exercise 04: Developer Paradise -- Hot-Reload and Test Databases

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Create a development-optimized Compose setup with hot-reload for code changes,
isolated test databases, debugging tools on demand, and override files that keep
development and production concerns separate.

## Scenario

Your team has these development requirements:

- **Hot reload**: When a developer edits `./api/src/`, the Node.js API should
  restart automatically without rebuilding the image.
- **Test database**: Every developer needs their own isolated test database so
  tests do not collide.
- **Debug tools**: A database GUI (Adminer) and a Redis GUI (Redis Commander)
  should be available when needed, but not running all the time.
- **Exposed ports**: Developers want direct access to PostgreSQL (5432) and
  Redis (6379) from their host machine for debugging.
- **Seed data**: The test database should be seeded with sample data on first start.

Your production setup (from Exercise 03) should NOT have any of these features.

## Tasks

### Part A: Base Compose File

Create `docker-compose.yml` with the base configuration shared by all environments.
This file should contain:
- Service definitions for `postgres`, `redis`, and `api`
- Health checks for all services
- Proper `depends_on` with conditions
- Named volumes for data persistence
- NO ports exposed to the host (production-safe default)
- NO development-specific features

<details>
<summary>Hint 1: Base File Structure</summary>

The base file should be clean and production-safe:

```yaml
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: taskdb
      POSTGRES_USER: taskuser
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - pgdata:/var/lib/postgresql/data
      - ./db/init.sql:/docker-entrypoint-initdb.d/init.sql:ro
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U taskuser -d taskdb"]
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
      DATABASE_URL: postgresql://taskuser:${POSTGRES_PASSWORD}@postgres:5432/taskdb
      REDIS_URL: redis://redis:6379
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    restart: unless-stopped

volumes:
  pgdata:
```

</details>

### Part B: Development Override File

Create `docker-compose.override.yml` with development-specific features:

1. **Hot reload** for the API: bind-mount `./api/src` into the container and use
   `nodemon` or `node --watch` as the command.
2. **Exposed ports** for PostgreSQL (5432) and Redis (6379) on the host.
3. **Adminer** service for database GUI on port 8081.
4. **Redis Commander** service for Redis GUI on port 8082.
5. **Debug environment variables** on the API (e.g., `NODE_ENV=development`,
   `DEBUG=*`).

<details>
<summary>Hint 1: Hot Reload with Bind Mount</summary>

For Node.js hot reload:

```yaml
services:
  api:
    volumes:
      - ./api/src:/app/src
    command: npx nodemon src/index.js
    environment:
      NODE_ENV: development
```

This mounts your local source code into the container. When you edit a file on
your host, `nodemon` detects the change and restarts the Node.js process.

</details>

<details>
<summary>Hint 2: Debug Tools as Optional Services</summary>

Adminer and Redis Commander should not depend on the `api` service. They connect
directly to the database and cache:

```yaml
services:
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

</details>

### Part C: Test Database Isolation

Modify the setup so each developer can have their own test database. Use a `.env`
file with developer-specific values:

**.env (each developer creates their own):**
```
POSTGRES_PASSWORD=dev_alice_secret
POSTGRES_DB=taskdb_alice
```

Answer these questions:
1. How does the `.env` file interact with the Compose file?
2. What happens if two developers use the same database name?
3. How would you ensure test databases are cleaned up between test runs?

<details>
<summary>Hint</summary>

The `.env` file in the same directory as `docker-compose.yml` is automatically
read by Compose. Variables like `${POSTGRES_PASSWORD}` are replaced before
the container starts.

For test isolation, consider adding a separate test database in `init.sql`:

```sql
CREATE DATABASE testdb;
```

Or use a separate Compose file for tests: `docker-compose.test.yml`.

</details>

### Part D: Seed Data

Create a file `db/init.sql` that runs automatically when PostgreSQL starts for the
first time. It should:

1. Create a `tasks` table with columns: `id` (serial), `title` (varchar),
   `completed` (boolean), `created_at` (timestamp).
2. Insert three sample tasks.
3. The script should be idempotent (safe to run multiple times).

Mount this file in the PostgreSQL container:

```yaml
volumes:
  - ./db/init.sql:/docker-entrypoint-initdb.d/init.sql:ro
```

<details>
<summary>Hint</summary>

PostgreSQL runs files in `/docker-entrypoint-initdb.d/` only on first start
(when the data directory is empty). Use `IF NOT EXISTS` for idempotency:

```sql
CREATE TABLE IF NOT EXISTS tasks (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    completed BOOLEAN DEFAULT false,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

</details>

### Part E: Verify the Development Workflow

Test the complete development workflow:

```bash
# Start in development mode (uses override automatically)
docker compose up -d

# Verify hot reload works
# Edit a file in ./api/src/ and watch the API restart in the logs
docker compose logs -f api

# Access Adminer at http://localhost:8081
# Access Redis Commander at http://localhost:8082

# Test production mode (ignores override file)
docker compose -f docker-compose.yml up -d
```

## Success Criteria

- [ ] `docker-compose.yml` is production-safe (no exposed ports, no dev tools)
- [ ] `docker-compose.override.yml` adds hot reload, exposed ports, and debug tools
- [ ] Editing a source file triggers automatic API restart
- [ ] Adminer is accessible at http://localhost:8081
- [ ] Redis Commander is accessible at http://localhost:8082
- [ ] Seed data appears in the database on first start
- [ ] `docker compose -f docker-compose.yml up -d` runs without development features
- [ ] Each developer can use a different database via `.env`

## What You Should Understand After This Exercise

Override files let you separate concerns: the base file defines what the application
*is*, the override file defines how developers *interact with it*. Hot reload via
bind mounts eliminates rebuild cycles. Debug tools via profiles or override files
keep the default stack lean. The `.env` file enables per-developer customization
without changing the compose file.
