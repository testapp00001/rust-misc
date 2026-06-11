# Exercise 03: Hardening the Stack

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective

Add production-grade reliability features to a Compose file: health checks, restart
policies, resource limits, and proper dependency ordering. This exercise trains you
to think about failure modes and operational resilience.

## Scenario

Your three-service stack from Exercise 02 works, but it is fragile:

- The API starts before PostgreSQL is ready and crashes on connection refused.
- If the API crashes, it stays dead until someone manually restarts it.
- A memory leak in the API could consume all host resources.
- There is no way to know if PostgreSQL is actually healthy or just running.

Your task: harden the stack so it survives real-world conditions.

## Tasks

### Part A: Add Health Checks

Add health checks to all three services:

1. **PostgreSQL**: Use `pg_isready` to verify the database accepts connections.
2. **Redis**: Use `redis-cli ping` to verify the cache responds.
3. **API**: Use `curl -f http://localhost:3000/health` to verify the API responds.

For each health check, configure:
- `interval`: how often to check
- `timeout`: how long to wait for a response
- `retries`: how many failures before marking unhealthy
- `start_period`: grace period before health checks count

<details>
<summary>Hint 1: PostgreSQL Health Check</summary>

```yaml
healthcheck:
  test: ["CMD-SHELL", "pg_isready -U taskuser -d taskdb"]
  interval: 5s
  timeout: 3s
  retries: 5
  start_period: 10s
```

The `CMD-SHELL` form runs the command through `/bin/sh -c`, which is needed for
commands that use shell features like pipes or variable expansion.

</details>

<details>
<summary>Hint 2: Health Check Command Format</summary>

Docker health checks use a specific command format:

```yaml
# Shell form (runs through /bin/sh -c)
test: ["CMD-SHELL", "command here"]

# Exec form (runs the binary directly, no shell)
test: ["CMD", "binary", "arg1", "arg2"]
```

Use `CMD-SHELL` when you need shell features. Use `CMD` for simple binary calls.

</details>

### Part B: Fix Dependency Ordering

Change `depends_on` from the simple list form to the condition form so the API
waits for healthy databases:

```yaml
# BEFORE (only waits for container to start)
depends_on:
  - postgres
  - redis

# AFTER (waits for health check to pass)
depends_on:
  postgres:
    condition: service_healthy
  redis:
    condition: service_healthy
```

<details>
<summary>Hint</summary>

The `condition: service_healthy` form requires that the dependency has a
`healthcheck` defined. If you skip the health check in Part A, this will not work.

</details>

### Part C: Add Restart Policies

Add a `restart` policy to each service:

- `postgres`: `unless-stopped` -- always restart unless explicitly stopped
- `redis`: `unless-stopped`
- `api`: `on-failure:5` -- restart up to 5 times on non-zero exit codes

Explain why you would choose different policies for different services.

<details>
<summary>Hint</summary>

Restart policy options:
- `no`: never restart (default)
- `always`: always restart, even if manually stopped
- `unless-stopped`: like `always`, but respects manual stops
- `on-failure`: only restart on non-zero exit codes
- `on-failure:N`: restart up to N times on failure

For databases, `unless-stopped` is common because you want them back after a
host reboot. For application code, `on-failure` prevents crash loops.

</details>

### Part D: Set Resource Limits

Add resource limits to the `api` service:

```yaml
deploy:
  resources:
    limits:
      cpus: "0.50"
      memory: 256M
    reservations:
      cpus: "0.25"
      memory: 128M
```

Answer these questions:
1. What is the difference between `limits` and `reservations`?
2. What happens when a container exceeds its memory limit?
3. Why would you set limits on the API but maybe not on the database in development?

<details>
<summary>Hint</summary>

- `limits`: the hard ceiling. The kernel kills the container (OOM) if it exceeds
  the memory limit. CPU usage is throttled.
- `reservations`: the guaranteed minimum. Docker scheduler ensures this amount
  is available. Only matters in multi-host setups (Docker Swarm).
- In development, database limits can be relaxed because you control the host.
  In production, every service needs limits.

</details>

### Part E: Validate the Hardened Stack

```bash
# Start everything
docker compose up -d

# Watch the startup sequence -- you should see the API wait
docker compose logs -f

# Verify health status
docker compose ps

# Simulate a crash
docker compose kill api

# Watch it restart automatically
docker compose ps
docker compose logs api
```

<details>
<summary>Hint</summary>

If the API restarts too quickly and hits the failure limit, check your
`restart` policy. `on-failure:5` means it gives up after 5 consecutive failures.
You can increase the limit or switch to `unless-stopped` for testing.

</details>

## Success Criteria

- [ ] All three services have health checks defined
- [ ] The API uses `condition: service_healthy` for its dependencies
- [ ] Each service has an appropriate restart policy
- [ ] The API has resource limits for CPU and memory
- [ ] `docker compose ps` shows health status for all services
- [ ] The API automatically restarts after being killed

## What You Should Understand After This Exercise

A working Compose file is not the same as a reliable Compose file. Health checks
prevent cascading failures from services that are "up" but not ready. Restart
policies recover from crashes without human intervention. Resource limits prevent
one service from starving the others. These features turn a development toy into
something that can survive real conditions.
