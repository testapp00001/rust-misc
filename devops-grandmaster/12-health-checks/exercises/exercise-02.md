# Exercise 02: Add Health Check Endpoints to an Application

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Add proper health check endpoints (`/healthz` for liveness, `/readyz` for
readiness) to a Python Flask application. Configure Docker HEALTHCHECK in
the Dockerfile and in Docker Compose. Verify that the container reports
healthy and that the endpoints return the correct status codes.

## Background

A Dockerfile HEALTHCHECK tells Docker how to determine if your container is
healthy. Without one, Docker only knows if the main process is running. You
need to build the endpoints into the application first, then wire Docker to
call them.

---

## Instructions

### Step 1: Create the Application

Create a project directory with the following structure:

```
health-check-app/
  app.py
  requirements.txt
  Dockerfile
  docker-compose.yml
```

Start with this application code. It simulates a slow startup and a
dependency on an external service.

```python
# app.py
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


# TODO: Add /healthz endpoint (liveness)
# TODO: Add /readyz endpoint (readiness)
# TODO: Add /startupz endpoint (startup)


if __name__ == '__main__':
    port = int(os.environ.get('PORT', 5000))
    app.run(host='0.0.0.0', port=port)
```

```text
# requirements.txt
flask==3.0.0
```

### Step 2: Implement the Health Check Endpoints

Add three endpoints to `app.py`:

**`/healthz` (Liveness):**
- Should always return 200 if the process is running and not deadlocked.
- Should NOT check external dependencies. A liveness probe that fails when
  a database is down will cause restart loops.
- Response body: `{"status": "alive", "uptime": <seconds>}`

**`/readyz` (Readiness):**
- Should return 200 only when the application can serve traffic.
- Check `is_ready` (the startup flag).
- Check `dependency_healthy` (simulates an external dependency).
- If not ready, return 503 with details about what is not ready.
- Response body (ready): `{"status": "ready", "checks": {"startup": "ok", "dependency": "ok"}}`
- Response body (not ready): `{"status": "not_ready", "checks": {"startup": "ok"|"pending", "dependency": "ok"|"unhealthy"}}`

**`/startupz` (Startup):**
- Should return 200 only after initialization is complete.
- This is used by Kubernetes startup probes to avoid liveness probe
  interference during slow startups.
- Response body: `{"status": "started"}` or `{"status": "starting"}` with 503.

Add a toggle endpoint to simulate dependency failure for testing:

```python
@app.route('/toggle-dependency')
def toggle_dependency():
    """Toggle dependency health for testing."""
    global dependency_healthy
    dependency_healthy = not dependency_healthy
    return jsonify({'dependency_healthy': dependency_healthy})
```

<details>
<summary>Hint</summary>

- The liveness endpoint is the simplest -- it just returns 200 with uptime.
- The readiness endpoint needs to aggregate multiple checks and return 503
  if any check fails.
- Use `time.time() - start_time` to calculate uptime.
- The `/toggle-dependency` endpoint lets you test readiness failure without
  actually breaking a real dependency.

</details>

### Step 3: Create the Dockerfile

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

# TODO: Add HEALTHCHECK directive
# - Check every 10 seconds
# - Timeout after 3 seconds
# - Start checking after 5 seconds (start_period)
# - Mark unhealthy after 3 consecutive failures
# - Use /healthz for the liveness check

CMD ["python", "app.py"]
```

<details>
<summary>Hint</summary>

The HEALTHCHECK directive format is:
```
HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:5000/healthz || exit 1
```

Note: The Dockerfile HEALTHCHECK is a single liveness-style check. For
separate readiness and startup probes, you need Kubernetes (Exercise 03).

</details>

### Step 4: Create Docker Compose

```yaml
# docker-compose.yml
version: "3.8"

services:
  app:
    build: .
    ports:
      - "5000:5000"
    environment:
      - PORT=5000
    # TODO: Add healthcheck configuration here as well
    # This overrides the Dockerfile HEALTHCHECK
```

<details>
<summary>Hint</summary>

In Docker Compose, the healthcheck block looks like:
```yaml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:5000/healthz"]
  interval: 10s
  timeout: 3s
  retries: 3
  start_period: 5s
```

</details>

### Step 5: Build, Run, and Verify

```bash
cd health-check-app

# Build and start
docker compose up -d --build

# Watch the container status (it will show "starting" for ~10 seconds)
watch docker ps

# Test the health endpoints
curl -s http://localhost:5000/healthz | python3 -m json.tool
curl -s http://localhost:5000/readyz | python3 -m json.tool
curl -s http://localhost:5000/startupz | python3 -m json.tool

# After 10 seconds, readiness should change to "ready"
sleep 10
curl -s http://localhost:5000/readyz | python3 -m json.tool

# Simulate a dependency failure
curl -s http://localhost:5000/toggle-dependency
curl -s http://localhost:5000/readyz | python3 -m json.tool

# Check Docker's view of health
docker inspect --format='{{json .State.Health}}' health-check-app | python3 -m json.tool

# Restore dependency
curl -s http://localhost:5000/toggle-dependency
```

---

## Success Criteria

- [ ] `/healthz` returns 200 with uptime information at all times after the
      process starts
- [ ] `/readyz` returns 503 during startup and when the dependency is
      unhealthy, and 200 when everything is ready
- [ ] `/startupz` returns 503 during the first 10 seconds, then 200
- [ ] Docker reports the container as "(healthy)" after startup completes
- [ ] `docker inspect` shows the health check log with successful checks
- [ ] Toggling the dependency causes `/readyz` to return 503 without
      affecting `/healthz`

## Common Mistakes to Avoid

- Making the liveness probe check external dependencies -- this causes
  restart loops when dependencies are temporarily down
- Using the same endpoint for liveness and readiness -- they serve different
  purposes
- Setting `start_period` too short -- the container gets killed before it
  finishes initializing
- Forgetting to install `curl` in the Dockerfile -- the HEALTHCHECK command
  will fail

## What You Should Understand After This Exercise

Health check endpoints are application-level contracts. The application
knows its own state better than Docker does. By implementing `/healthz` and
`/readyz` separately, you give the orchestrator two different levers:
restart the container (liveness) or remove it from the load balancer
(readiness). The Docker HEALTHCHECK directive is a simple liveness check.
For separate readiness and startup probes, you need Kubernetes.
