# Solution 03: Hardening the Stack

## Part A: Health Checks

The complete service definitions with health checks:

```yaml
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: taskdb
      POSTGRES_USER: taskuser
      POSTGRES_PASSWORD: taskpass
    volumes:
      - pgdata:/var/lib/postgresql/data
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
    image: node:20-alpine
    command: sleep infinity
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: postgresql://taskuser:taskpass@postgres:5432/taskdb
      REDIS_URL: redis://redis:6379
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 10s
      timeout: 5s
      retries: 3
    restart: on-failure:5
    deploy:
      resources:
        limits:
          cpus: "0.50"
          memory: 256M
        reservations:
          cpus: "0.25"
          memory: 128M

volumes:
  pgdata:
```

### Why This Works

**PostgreSQL health check**: `pg_isready` is a utility that comes with PostgreSQL.
It checks whether the server is accepting connections. The `-U` and `-d` flags
specify the user and database to check against. `CMD-SHELL` is used because
`pg_isready` may need shell PATH resolution.

**Redis health check**: `redis-cli ping` sends a PING command to the Redis server.
A healthy Redis responds with PONG. `CMD` (exec form) is used instead of
`CMD-SHELL` because we are calling a binary directly with no shell features needed.

**API health check**: `curl -f` makes an HTTP request to the `/health` endpoint.
The `-f` flag causes curl to exit with a non-zero status on HTTP errors (4xx, 5xx).
This assumes the API has a `/health` endpoint that returns 200 when healthy.

**Start period**: The `start_period: 10s` on PostgreSQL gives the database 10
seconds to initialize before health check failures count toward the retry limit.
This prevents the health check from marking the service as unhealthy during normal
startup.

---

## Part B: Dependency Ordering with Conditions

```yaml
  api:
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
```

### Why This Works

The simple `depends_on` list (`- postgres`) only waits for the container to
*start* -- meaning the process has been created, not that the service is ready.
PostgreSQL starts its process immediately but may take several seconds to complete
initialization and begin accepting connections.

`condition: service_healthy` waits for the health check to pass. The API will not
start until `pg_isready` returns success and `redis-cli ping` returns PONG. This
eliminates connection-refused errors during startup.

**Important**: This only works if the dependency has a `healthcheck` defined. If
you use `condition: service_healthy` on a service without a health check, Compose
will show a validation error.

---

## Part C: Restart Policies

```yaml
  postgres:
    restart: unless-stopped

  redis:
    restart: unless-stopped

  api:
    restart: on-failure:5
```

### Why Different Policies

**`unless-stopped` for databases**: Databases are stateful services that should
almost always be running. `unless-stopped` restarts the container automatically
after crashes, host reboots, or Docker daemon restarts. The only exception is when
you explicitly run `docker compose down` or `docker stop` -- in that case, Compose
respects your intent and does not restart.

**`on-failure:5` for the API**: Application code can have bugs that cause
immediate crashes (crash loops). Without a retry limit, the API would restart
forever, filling logs with error messages and consuming resources. `on-failure:5`
gives the developer 5 chances to fix the issue before giving up. The `:5` limit
prevents infinite restart loops while still recovering from transient failures
like temporary network issues.

**Why not `always` for everything?** `always` restarts even after `docker compose
down`, which can be confusing. `unless-stopped` is almost always the better choice.

---

## Part D: Resource Limits

```yaml
  api:
    deploy:
      resources:
        limits:
          cpus: "0.50"
          memory: 256M
        reservations:
          cpus: "0.25"
          memory: 128M
```

### Questions Answered

**1. Limits vs. Reservations:**
- `limits` is the hard ceiling. If the container exceeds the memory limit, the
  kernel's OOM killer terminates it. CPU usage above the limit is throttled (not
  killed) -- the container gets fewer CPU cycles.
- `reservations` is the guaranteed minimum. Docker's scheduler ensures this amount
  is available when placing the container. In a single-host Compose setup,
  reservations are informational. In a multi-host Docker Swarm setup, they
  influence which host gets the container.

**2. Exceeding memory limit:**
The Linux kernel's OOM (Out of Memory) killer terminates the container process.
Docker reports the container as "OOMKilled" in `docker inspect`. If the restart
policy allows it, the container restarts automatically.

**3. Why limit the API but not the database in development?**
In development, the developer controls the host machine and knows what else is
running. The database is unlikely to have a memory leak. The API, however, might
have bugs that cause memory leaks during development. Limiting the API prevents
a single buggy endpoint from freezing the developer's laptop. In production, every
service needs limits because you do not control the workload.

---

## Part E: Validation

### Expected `docker compose ps` Output

```
NAME                  IMAGE               STATUS                    PORTS
myproject-api-1       node:20-alpine      Up 2 min (healthy)        0.0.0.0:3000->3000/tcp
myproject-postgres-1  postgres:16-alpine  Up 3 min (healthy)        5432/tcp
myproject-redis-1     redis:7-alpine      Up 3 min (healthy)        6379/tcp
```

The `(healthy)` indicator confirms that health checks are passing.

### Expected Startup Sequence

When running `docker compose up`, the logs should show:

1. PostgreSQL starts and begins initialization.
2. Redis starts immediately.
3. The API waits (it is blocked by `condition: service_healthy`).
4. After PostgreSQL's health check passes, the API starts.
5. After Redis's health check passes, the API continues (if it needs both).

### Crash Recovery Test

After `docker compose kill api`:

1. `docker compose ps` shows the API as "Restarting".
2. After a few seconds, the API is back to "Up (healthy)".
3. `docker compose logs api` shows the crash event and the restart.

---

## Common Mistakes

### 1. Health Check Command Not Available

```yaml
# WRONG -- curl is not installed in the default node:20-alpine image
test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
```

Fix: Install curl in the Dockerfile, or use `wget` which is available in Alpine:

```yaml
test: ["CMD", "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:3000/health"]
```

### 2. Forgetting start_period

```yaml
# WRONG -- health checks count immediately, may mark as unhealthy during startup
healthcheck:
  test: ["CMD-SHELL", "pg_isready -U postgres"]
  interval: 5s
  timeout: 3s
  retries: 5
```

Without `start_period`, the 5-retry limit starts counting from the first check.
If PostgreSQL takes 30 seconds to initialize and checks run every 5 seconds, the
service is marked unhealthy after 25 seconds (5 failures) -- before it even finishes
starting.

### 3. Using Simple depends_on with Health Checks

```yaml
# WRONG -- health check exists but is not used for ordering
depends_on:
  - postgres

# RIGHT -- use the condition form
depends_on:
  postgres:
    condition: service_healthy
```

Defining a health check does not automatically change `depends_on` behavior. You
must explicitly use the `condition` form.

### 4. Setting limits Without Understanding the Values

```yaml
# DANGEROUS -- 64M is likely too low for a Node.js API
deploy:
  resources:
    limits:
      memory: 64M
```

If the limit is too low, the container gets OOM-killed repeatedly. Start with
generous limits and tighten based on observed usage (`docker stats`).

## Relevant README Sections

- [Production Way: Health Checks and Dependency Ordering](../README.md#production-way-health-checks-and-dependency-ordering)
- [Health Check Commands by Database](../README.md#health-check-commands-by-database)
- [Compose File Best Practices](../README.md#compose-file-best-practices)
