# Solution 02: Add Health Check Endpoints to an Application

## Complete Solution

### app.py

```python
from flask import Flask, jsonify
import time
import threading
import os

app = Flask(__name__)

# Application state
start_time = time.time()
is_ready = False
dependency_healthy = True


def initialize():
    """Simulate a slow startup (10 seconds)."""
    global is_ready
    time.sleep(10)
    is_ready = True


# Start initialization in the background
threading.Thread(target=initialize, daemon=True).start()


@app.route('/')
def index():
    """Main application endpoint."""
    if not is_ready:
        return jsonify({'error': 'Application is starting'}), 503
    return jsonify({'message': 'Hello from health-check-app!'})


@app.route('/healthz')
def healthz():
    """Liveness probe.

    Always returns 200 if the process is running. Does NOT check
    external dependencies -- a database outage should not cause
    this container to restart.
    """
    return jsonify({
        'status': 'alive',
        'uptime': round(time.time() - start_time, 1),
    })


@app.route('/readyz')
def readyz():
    """Readiness probe.

    Returns 200 only when the application can serve traffic.
    Checks both startup completion and dependency health.
    Returns 503 with details when not ready.
    """
    checks = {
        'startup': 'ok' if is_ready else 'pending',
        'dependency': 'ok' if dependency_healthy else 'unhealthy',
    }

    all_ok = all(v == 'ok' for v in checks.values())
    status_code = 200 if all_ok else 503

    return jsonify({
        'status': 'ready' if all_ok else 'not_ready',
        'checks': checks,
    }), status_code


@app.route('/startupz')
def startupz():
    """Startup probe.

    Returns 200 only after initialization is complete.
    Used by Kubernetes startup probes to protect slow-starting
    applications from premature liveness probe failures.
    """
    if is_ready:
        return jsonify({'status': 'started'})
    return jsonify({'status': 'starting'}), 503


@app.route('/toggle-dependency')
def toggle_dependency():
    """Toggle dependency health for testing."""
    global dependency_healthy
    dependency_healthy = not dependency_healthy
    return jsonify({'dependency_healthy': dependency_healthy})


if __name__ == '__main__':
    port = int(os.environ.get('PORT', 5000))
    app.run(host='0.0.0.0', port=port)
```

### Dockerfile

```dockerfile
FROM python:3.12-slim

# Install curl for health checks
RUN apt-get update && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

EXPOSE 5000

# Liveness check using /healthz
# - interval=10s: check every 10 seconds
# - timeout=3s: fail if no response in 3 seconds
# - start-period=5s: do not count failures in the first 5 seconds
# - retries=3: mark unhealthy after 3 consecutive failures
HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:5000/healthz || exit 1

CMD ["python", "app.py"]
```

### docker-compose.yml

```yaml
version: "3.8"

services:
  app:
    build: .
    ports:
      - "5000:5000"
    environment:
      - PORT=5000
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:5000/healthz"]
      interval: 10s
      timeout: 3s
      retries: 3
      start_period: 5s
```

### Verification

```bash
cd health-check-app
docker compose up -d --build

# Immediately check -- container should be "starting"
curl -s http://localhost:5000/healthz
# {"status": "alive", "uptime": 1.2}

curl -s http://localhost:5000/readyz
# {"status": "not_ready", "checks": {"startup": "pending", "dependency": "ok"}} (503)

curl -s http://localhost:5000/startupz
# {"status": "starting"} (503)

# Docker shows "health: starting" during start_period
docker ps
# CONTAINER ID   IMAGE             STATUS                 PORTS
# abc123         health-check-app  Up 3s (health: starting)  0.0.0.0:5000->5000/tcp

# Wait 10 seconds for initialization
sleep 10

curl -s http://localhost:5000/readyz
# {"status": "ready", "checks": {"startup": "ok", "dependency": "ok"}} (200)

curl -s http://localhost:5000/startupz
# {"status": "started"} (200)

# Docker now shows "healthy"
docker ps
# CONTAINER ID   IMAGE             STATUS          PORTS
# abc123         health-check-app  Up 15s (healthy)  0.0.0.0:5000->5000/tcp

# Simulate dependency failure
curl -s http://localhost:5000/toggle-dependency
# {"dependency_healthy": false}

curl -s http://localhost:5000/readyz
# {"status": "not_ready", "checks": {"startup": "ok", "dependency": "unhealthy"}} (503)

# Liveness is unaffected
curl -s http://localhost:5000/healthz
# {"status": "alive", "uptime": 25.3}

# Docker still shows "healthy" (HEALTHCHECK uses /healthz)
docker ps
# CONTAINER ID   IMAGE             STATUS          PORTS
# abc123         health-check-app  Up 25s (healthy)  0.0.0.0:5000->5000/tcp

# Check Docker health log
docker inspect --format='{{range .State.Health.Log}}{{.ExitCode}} {{.Output}}{{end}}' health-check-app
```

---

## Why This Works

1. **Three endpoints, three purposes.** `/healthz` is for the orchestrator
   to decide "should I restart this?" `/readyz` is for the load balancer
   to decide "should I send traffic here?" `/startupz` is for Kubernetes
   to decide "has this finished initializing?" Each answers a different
   question.

2. **The liveness endpoint never checks dependencies.** If the database is
   down, `/healthz` still returns 200. This prevents the restart cascade
   where a database outage causes every application container to restart,
   which makes the outage worse.

3. **The readiness endpoint checks everything needed to serve traffic.**
   Both the startup state and the dependency state are checked. If either
   is not ready, the endpoint returns 503. The load balancer (or Kubernetes
   Service) removes the instance from rotation.

4. **The Docker HEALTHCHECK uses `/healthz`.** Docker only supports a single
   health check command. It is a liveness-style check. For separate
   readiness and startup probes, you need Kubernetes (Exercise 03).

5. **The `start_period` in HEALTHCHECK** tells Docker to ignore failures
   during the first 5 seconds. This prevents Docker from marking the
   container as unhealthy before the application has had a chance to start
   listening on the port.

## Common Mistakes

### Mistake 1: Checking dependencies in the liveness endpoint

```python
@app.route('/healthz')
def healthz():
    db_ok = check_database()  # WRONG
    return jsonify({'status': 'alive' if db_ok else 'dead'})
```

When the database goes down, `/healthz` returns 503, Docker marks the
container as unhealthy, and the restart policy kicks in. Restarting the
application does not fix the database. You now have a cascading failure.

### Mistake 2: Using the same endpoint for everything

```python
@app.route('/health')
def health():
    # Checks everything -- startup, dependencies, process
    if not is_ready:
        return jsonify({'status': 'not ready'}), 503
    if not check_db():
        return jsonify({'status': 'no db'}), 503
    return jsonify({'status': 'ok'})
```

If you use this for both liveness and readiness, a database outage causes
a restart (because liveness fails). You need separate endpoints.

### Mistake 3: Not installing curl in the Dockerfile

```dockerfile
FROM python:3.12-slim
# No curl installed
HEALTHCHECK CMD curl -f http://localhost:5000/healthz || exit 1
```

The `python:3.12-slim` image does not include `curl`. The HEALTHCHECK
command fails immediately with "curl: command not found" and the container
is marked unhealthy.

### Mistake 4: Setting start_period too short

```dockerfile
HEALTHCHECK --start-period=2s ...
```

If the application takes 10 seconds to start listening on the port, the
HEALTHCHECK fails at 2 seconds, 5 seconds, and 8 seconds. With `retries=3`,
the container is marked unhealthy at 8 seconds -- before it finishes
starting. Set `start_period` to cover the expected startup time plus a
margin.
