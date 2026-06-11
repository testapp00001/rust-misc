# Solution 01: Liveness vs Readiness vs Startup Probes

## Part A: Define Each Probe Type

### 1. Liveness probe

A **liveness probe** asks: "Is this process stuck?" It checks whether the
application is still running and responsive -- not whether it can serve
traffic, just whether it is alive. When a liveness probe fails (after
`failureThreshold` consecutive failures), the orchestrator **restarts the
container**. This is the correct action for deadlocks, infinite loops, and
unrecoverable states where the process is running but cannot do useful work.

### 2. Readiness probe

A **readiness probe** asks: "Can this instance handle requests right now?"
It checks whether the application is ready to receive traffic from the load
balancer or service mesh. When a readiness probe fails, the orchestrator
**removes the pod from the Service endpoints** -- traffic stops flowing to
it -- but the container is **not restarted**. This is the correct action
for temporary conditions: a database failover, a warming cache, or a
connection pool that is exhausted.

### 3. Startup probe

A **startup probe** asks: "Has this application finished starting?" It runs
before the liveness and readiness probes are active. While the startup probe
is running, liveness and readiness probes are disabled. When the startup
probe succeeds, the other probes take over. When it fails (exceeds
`failureThreshold * periodSeconds`), the container is restarted. This
solves the problem of slow-starting applications (Java with Spring, ML model
loading, large database WAL replay) that would be killed by a liveness
probe before they finish initializing.

### 4. Can a container be alive but not ready?

Yes. A concrete example: a web server that depends on a database. The web
server process is running (alive), but the database is temporarily
unreachable due to a network partition. The liveness probe passes (the
process is responsive), but the readiness probe fails (the application
cannot serve requests that require database access). The orchestrator
should keep the container running (no restart) but stop sending it traffic
until the database recovers.

Another example: a container that just started and is warming its cache.
The process is alive and responsive, but it cannot serve requests at full
capacity yet.

## Part B: Match the Scenario to the Probe

### 1. Java application with 45-second startup

**Startup probe.** The application takes 45 seconds to initialize. A
liveness probe with a 10-second `initialDelaySeconds` and 3 retries at
10-second intervals would kill the container at 40 seconds -- before it
finishes starting. A startup probe with `failureThreshold: 30` and
`periodSeconds: 2` gives the application 60 seconds to start, after which
the liveness probe takes over.

### 2. Web server with infinite loop bug

**Liveness probe.** The application is stuck -- it will never recover
without a restart. A liveness probe on a simple HTTP endpoint will fail
when the server enters the infinite loop (it stops responding to requests).
After `failureThreshold` failures, the orchestrator restarts the container,
which clears the stuck state.

### 3. Database failover for 30 seconds

**Readiness probe.** The application itself is healthy -- it is the
database that is temporarily unavailable. The readiness probe checks the
database connection and returns 503 during the failover. Traffic is
redirected to other instances. When the database recovers, the readiness
probe passes again and the instance re-enters the load balancer. No restart
is needed.

### 4. ML model loading at startup

**Startup probe.** The model server cannot respond to HTTP requests while
loading a 2 GB model into memory. This is a startup concern, not a
liveness concern. The startup probe gives the server enough time to load
the model (e.g., `failureThreshold: 60, periodSeconds: 5` = 5 minutes).
Once the startup probe passes, liveness and readiness probes take over.

### 5. Message queue consumer stopped processing

**Liveness probe (custom).** This is tricky because the HTTP health check
still returns 200 -- the process is running and the HTTP server is
responsive. The liveness probe needs to check whether the consumer is
actually processing messages. Options:

- A "heartbeat" that updates a timestamp every time a message is processed.
  The liveness endpoint checks if the heartbeat is recent.
- A dedicated `/healthz` endpoint that checks the consumer's internal state
  (e.g., "last message processed within 30 seconds").
- If the consumer shares a process with the HTTP server, the liveness
  endpoint can check a shared flag that the consumer sets.

This is a case where a naive HTTP 200 liveness probe is insufficient.

## Part C: Identify the Misconfiguration

### Configuration 1: No startup probe, short liveness timing

```yaml
livenessProbe:
  httpGet:
    path: /healthz
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 10
  failureThreshold: 3
```

**What goes wrong:** The application takes 60 seconds to start. The liveness
probe begins checking at 10 seconds (`initialDelaySeconds`). It checks at
10s, 20s, 30s -- all fail because the app is still starting. At 30 seconds
(3 failures), the container is restarted. The restart takes another 60
seconds to start, and the cycle repeats. The container enters a
**restart loop** and never becomes ready.

**Fix:** Add a startup probe:
```yaml
startupProbe:
  httpGet:
    path: /healthz
    port: 8080
  failureThreshold: 30
  periodSeconds: 2
# Liveness probe starts only after startup probe succeeds
livenessProbe:
  httpGet:
    path: /healthz
    port: 8080
  periodSeconds: 10
  failureThreshold: 3
```

### Configuration 2: Liveness probe checks database

```yaml
livenessProbe:
  httpGet:
    path: /ready
    port: 8080
  periodSeconds: 5
  failureThreshold: 3
```

**What goes wrong:** The `/ready` endpoint returns 503 when the database is
unreachable. During a 30-second database failover, the liveness probe fails
3 times (at 5, 10, 15 seconds). The container is restarted. But the
container was healthy -- it was the database that was down. Restarting the
container does not fix the database. Now you have a **restart cascade**:
every instance restarts, each one fails the liveness probe again because
the database is still down, and you have a full outage that persists until
the database recovers.

**Fix:** Use a separate liveness endpoint that does not check dependencies:
```yaml
livenessProbe:
  httpGet:
    path: /healthz  # Only checks if the process is alive
    port: 8080
  periodSeconds: 10
  failureThreshold: 3
readinessProbe:
  httpGet:
    path: /ready  # Checks database and other dependencies
    port: 8080
  periodSeconds: 5
  failureThreshold: 3
```

### Configuration 3: Readiness probe always returns 200

```yaml
readinessProbe:
  httpGet:
    path: /healthz
    port: 8080
  periodSeconds: 10
```

**What goes wrong:** The `/healthz` endpoint always returns 200, even
during startup. Traffic is sent to the container immediately, before it
has finished initializing. Users get 503 errors or connection refused for
the first several seconds. In a rolling update, this means downtime during
every deployment.

**Fix:** Use a readiness endpoint that checks actual readiness:
```yaml
readinessProbe:
  httpGet:
    path: /readyz  # Returns 503 during startup
    port: 8080
  periodSeconds: 5
  failureThreshold: 3
```

## Part D: Sample Probe Strategy

```yaml
# Python Flask app with 5-15s startup, PostgreSQL and Redis dependencies

# Startup probe: handles variable startup time
# 15s max startup, checking every 2s, 10 retries = 20s max
startupProbe:
  httpGet:
    path: /healthz
    port: 5000
  periodSeconds: 2
  failureThreshold: 10

# Liveness probe: catches deadlocks and unresponsive states
# Checks only the application itself, NOT dependencies
livenessProbe:
  httpGet:
    path: /healthz
    port: 5000
  periodSeconds: 10
  failureThreshold: 3
  timeoutSeconds: 3

# Readiness probe: checks all dependencies
# Returns 503 if PostgreSQL or Redis is unreachable
readinessProbe:
  httpGet:
    path: /readyz
    port: 5000
  periodSeconds: 5
  failureThreshold: 3
  timeoutSeconds: 5
```

**Why these choices:**

- **Startup probe at `/healthz`:** The liveness endpoint. During startup,
  the app is not responding to HTTP yet, so the probe fails. With 10
  retries at 2-second intervals, the app has 20 seconds to start (covers
  the 5-15 second range with margin).
- **Liveness probe at `/healthz`:** Lightweight, no external calls. Returns
  200 if the process is responsive. If the process enters a deadlock, the
  HTTP server stops responding and the probe fails after 30 seconds
  (3 retries * 10 seconds).
- **Readiness probe at `/readyz`:** Connects to PostgreSQL and Redis.
  Returns 503 if either is unreachable. Timeout is 5 seconds (not 3) because
  database connections can be slow during recovery. Failure threshold is 3
  to avoid flapping on transient network issues.

## Common Mistakes

- **Using the same endpoint for liveness and readiness.** They serve
  different purposes. A liveness failure causes a restart; a readiness
  failure causes traffic removal. If your readiness check includes
  dependencies, a database outage will restart all your pods.
- **Not using a startup probe.** Without one, you must set
  `initialDelaySeconds` on the liveness probe to cover the worst-case
  startup time. This delays failure detection after the app is already
  running.
- **Checking dependencies in the liveness probe.** This is the most
  common and most destructive mistake. A dependency outage becomes an
  application restart cascade.
- **Setting `failureThreshold` too low.** A threshold of 1 means a single
  timeout (network hiccup, GC pause) triggers a restart or traffic removal.
  Use 3 or higher.
