# Solution 01: Compose vs. Commands -- The Real Tradeoffs

## Part A: Failure Modes of the Script

The bash script has at least these failure modes:

### 1. No Idempotency (Running Twice Breaks)

Running the script a second time fails at `docker run --name postgres` because a
container with that name already exists. Docker enforces unique container names.

Compose handles this by managing container lifecycle: `docker compose up` recreates
containers that have changed, and leaves unchanged containers alone.

### 2. Sleep Is Not Readiness

`sleep 3` does not guarantee PostgreSQL is ready. On a slow machine, PostgreSQL
might take 10 seconds to initialize. On a fast machine, 3 seconds is wasted time.
The API will crash with "connection refused" if PostgreSQL is not ready.

Compose with health checks (`condition: service_healthy`) waits for the actual
readiness signal, not an arbitrary timer.

### 3. No Automatic Restart

If the API crashes at 3 AM, it stays dead until a human runs the script again.
The script is a one-time creation tool, not a process supervisor.

Compose with `restart: unless-stopped` or `restart: on-failure` automatically
restarts crashed containers.

### 4. No Log Aggregation

To see logs from all three services, you need three separate `docker logs` commands.
There is no way to see interleaved logs or correlate events across services.

`docker compose logs -f` shows all logs interleaved with service name prefixes.

### 5. No Clean Shutdown

Stopping everything requires running `docker stop` and `docker rm` for each
container, in the reverse order, plus `docker network rm`. Missing a step leaves
orphaned resources.

`docker compose down` stops and removes all containers, networks, and optionally
volumes in one command.

### 6. No Build Integration

The script assumes the `myapp-api:1.0` image already exists. Where does it come
from? The script does not build it. You need a separate `docker build` step.

Compose's `build:` directive builds images as part of `docker compose up`.

### 7. Platform Inconsistencies

Bash scripts behave differently on macOS (BSD tools) vs. Linux (GNU tools). They
do not work on Windows without WSL. Team members on different platforms get different
results.

Compose files are platform-independent YAML. `docker compose up` works identically
everywhere.

### 8. No Declarative State

The script says *how* to create containers but not *what* the desired state should
be. There is no way to ask "what should be running?" and compare it to "what is
actually running?"

Compose defines the desired state. `docker compose ps` shows the current state.
`docker compose up` reconciles the two.

---

## Part B: The Declarative Advantage

### 1. Adding a Fourth Service

**Script approach**: Add another `docker run` command, make sure it is in the right
order, update `--network`, and hope you do not introduce a typo in a container name.

**Compose approach**: Add a new service block. Compose handles ordering, networking,
and naming automatically.

### 2. Current State Visibility

**Script approach**: You have no single command that shows what *should* be running.
You have to read the script and mentally parse each `docker run` command.

**Compose approach**: The compose file IS the definition of what should be running.
`docker compose ps` shows what IS running. The difference is immediately visible.

### 3. Self-Documentation

**Script approach**: A new team member reads a bash script and has to mentally
simulate each command to understand the architecture.

**Compose approach**: A new team member reads the YAML and immediately sees:
services, their images, their connections, their volumes. The file IS the
documentation.

---

## Part C: What the Script Gets Right

### 1. Explicit Startup Ordering

The script makes the startup order painfully obvious: postgres first, then redis,
then api. The sleep is wrong, but the *intent* is clear. In a Compose file with
`depends_on`, the ordering is implicit -- a reader has to trace the dependency
graph mentally.

### 2. Visible Timing Assumptions

The `sleep 3` is a bad solution, but it makes the timing problem *visible*. In a
Compose file without health checks, the same problem exists but is hidden -- the
API might start before PostgreSQL is ready, and nothing in the compose file
suggests this risk. The script forces you to confront the timing question; the
Compose file lets you ignore it until production.

---

## Part D: The Missing Pieces

The basic Compose file is missing:

1. **Health checks**: `depends_on` without `condition: service_healthy` only waits
   for the container to start, not for the service to be ready. PostgreSQL might
   not be accepting connections when the API starts.

2. **Restart policies**: If the API crashes, it stays dead. The default restart
   policy is `no`.

3. **Resource limits**: Nothing prevents the API from consuming all host memory
   or CPU. A single service can starve the others.

4. **Network isolation**: All services are on the default network with no isolation.
   In production, the database should not be reachable from the frontend.

5. **Pinned versions**: `postgres:16-alpine` is good, but some teams pin to
   specific patch versions (e.g., `postgres:16.2-alpine`) for reproducibility.

---

## Common Mistakes

1. **Thinking Compose eliminates all problems**: Compose is a convenience layer,
   not a magic solution. Health checks, restart policies, and resource limits are
   still your responsibility.

2. **Confusing "runs in Compose" with "production-ready"**: A compose file that
   works on your laptop is not production-ready. You still need health checks,
   resource limits, proper secrets management, and monitoring.

3. **Overusing the script approach**: Some teams keep adding to their bash script
   instead of switching to Compose. The script gets more complex, harder to
   maintain, and eventually unmaintainable. Switch early.

## Relevant README Sections

- [The Problem: Your App Needs Friends](../README.md#the-problem-your-app-needs-friends)
- [The Naive Way: Shell Scripts](../README.md#the-naive-way-shell-scripts)
- [The Right Way: Docker Compose](../README.md#the-right-way-docker-compose)
