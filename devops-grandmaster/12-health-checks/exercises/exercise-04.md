# Exercise 04: Design Health Checks for Partial Failures

**Type:** Challenge
**Time:** 60 minutes
**Difficulty:** Medium-Hard

## Objective

Design and implement health check endpoints for a microservice that can
experience partial failures. The service has multiple dependencies
(database, cache, message queue, external API), and each can fail
independently. Your health checks must distinguish between "completely
broken" and "degraded but functional" states, and report granular status
for each dependency.

## Background

Real applications rarely fail completely. More often, one dependency is
slow or unreachable while others work fine. A payment service might lose
its cache (degraded performance but still functional) while the database
is healthy. A naive health check that returns 503 when any dependency is
down will take the entire service out of the load balancer, even though
it can still serve most requests. You need health checks that reflect the
actual capability of the service.

---

## Instructions

### Step 1: Create the Application

```
partial-failure/
  app.py
  requirements.txt
  Dockerfile
  docker-compose.yml
```

```python
# app.py
from flask import Flask, jsonify, request
import time
import os
import threading
import random

app = Flask(__name__)

# Simulated dependency states
# Each can be: "healthy", "degraded", or "down"
dependencies = {
    'database': 'healthy',
    'cache': 'healthy',
    'message_queue': 'healthy',
    'external_api': 'healthy',
}


class DependencyCheck:
    """Represents a single dependency health check."""

    def __init__(self, name, critical=True, timeout=3):
        self.name = name
        self.critical = critical
        self.timeout = timeout

    def check(self):
        """
        Check the health of this dependency.

        Returns a dict with:
        - status: "healthy", "degraded", or "down"
        - latency_ms: response time in milliseconds
        - message: human-readable description
        - critical: whether this dependency is critical for serving traffic
        """
        state = dependencies.get(self.name, 'down')

        # Simulate variable latency
        latency = random.uniform(1, 50)
        if state == 'degraded':
            latency = random.uniform(100, 500)
        elif state == 'down':
            latency = self.timeout * 1000

        return {
            'status': state,
            'latency_ms': round(latency, 1),
            'critical': self.critical,
            'message': f'{self.name} is {state}',
        }


# Define which dependencies are critical vs non-critical
CHECKS = [
    DependencyCheck('database', critical=True),
    DependencyCheck('cache', critical=False),
    DependencyCheck('message_queue', critical=False),
    DependencyCheck('external_api', critical=False),
]


# TODO: Implement the following endpoints:

# GET /healthz -- Liveness probe (always 200 if process is alive)

# GET /readyz -- Readiness probe
# Should return 200 if ALL critical dependencies are healthy or degraded
# Should return 503 if ANY critical dependency is down
# Response should include individual dependency statuses

# GET /readyz?strict=true -- Strict readiness
# Should return 200 only if ALL dependencies (critical and non-critical)
# are healthy
# Should return 503 if ANY dependency is degraded or down

# GET /status -- Detailed status page (not used by probes, for humans)
# Should return full details of all dependencies


# Toggle endpoints for testing
@app.route('/set/<dependency>/<state>')
def set_dependency(dependency, state):
    """Set a dependency state for testing."""
    if dependency not in dependencies:
        return jsonify({'error': f'Unknown dependency: {dependency}'}), 404
    if state not in ('healthy', 'degraded', 'down'):
        return jsonify({'error': f'Invalid state: {state}'}), 400
    dependencies[dependency] = state
    return jsonify({dependency: state})


@app.route('/')
def index():
    """Main endpoint -- works if critical deps are up."""
    critical_ok = all(
        dependencies.get(c.name) != 'down'
        for c in CHECKS if c.critical
    )
    if not critical_ok:
        return jsonify({'error': 'Service unavailable'}), 503
    return jsonify({'message': 'Request processed successfully'})


if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

```text
# requirements.txt
flask==3.0.0
```

```dockerfile
# Dockerfile
FROM python:3.12-slim
RUN apt-get update && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY . .
EXPOSE 5000
CMD ["python", "app.py"]
```

### Step 2: Implement the Health Check Endpoints

Implement the four endpoints described in the code. Pay attention to these
design decisions:

**`/healthz` (Liveness):**
- Always returns 200.
- Reports uptime and process status.
- Does not call any dependency checks.

**`/readyz` (Readiness):**
- Runs all dependency checks.
- Returns 200 if all critical dependencies are healthy or degraded.
- Returns 503 if any critical dependency is down.
- Includes individual dependency statuses in the response.
- Supports `?strict=true` query parameter for strict mode.

**`/status` (Detailed Status):**
- Runs all dependency checks.
- Always returns 200 (this is informational, not a probe).
- Includes latency measurements, overall status, and a summary.

Response format for `/readyz`:
```json
{
  "status": "ready",
  "checks": {
    "database": {"status": "healthy", "latency_ms": 12.3, "critical": true},
    "cache": {"status": "degraded", "latency_ms": 250.1, "critical": false},
    "message_queue": {"status": "healthy", "latency_ms": 5.2, "critical": false},
    "external_api": {"status": "healthy", "latency_ms": 45.8, "critical": false}
  }
}
```

<details>
<summary>Hint</summary>

- Run all checks (even non-critical ones) in the readiness response. The
  orchestrator decides what to do based on the status code, but the
  response body gives humans and monitoring systems the full picture.
- For the `?strict=true` mode, change the failure condition to "any
  dependency not healthy" instead of "any critical dependency down."
- Consider running checks in parallel (using threads) if they are
  independent and each has a timeout. For this exercise, sequential is fine.

</details>

### Step 3: Create Docker Compose

```yaml
# docker-compose.yml
version: "3.8"

services:
  app:
    build: .
    ports:
      - "5000:5000"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:5000/healthz"]
      interval: 10s
      timeout: 3s
      retries: 3
      start_period: 5s
```

### Step 4: Test Partial Failure Scenarios

Build and run, then test each scenario:

```bash
docker compose up -d --build

# Wait for healthy
sleep 5

# Scenario 1: Everything healthy
curl -s http://localhost:5000/readyz | python3 -m json.tool
# Expected: 200, all checks healthy

# Scenario 2: Cache is down (non-critical)
curl -s http://localhost:5000/set/cache/down
curl -s http://localhost:5000/readyz | python3 -m json.tool
# Expected: 200, cache shows "down" but service is still ready
curl -s http://localhost:5000/ | python3 -m json.tool
# Expected: 200, main endpoint still works

# Scenario 3: Database is degraded
curl -s http://localhost:5000/set/database/degraded
curl -s http://localhost:5000/readyz | python3 -m json.tool
# Expected: 200, database shows "degraded" but service is still ready

# Scenario 4: Database is down (critical)
curl -s http://localhost:5000/set/database/down
curl -s http://localhost:5000/readyz | python3 -m json.tool
# Expected: 503, database shows "down"
curl -s http://localhost:5000/ | python3 -m json.tool
# Expected: 503

# Scenario 5: Strict mode with degraded cache
curl -s http://localhost:5000/set/database/healthy
curl -s http://localhost:5000/set/cache/degraded
curl -s http://localhost:5000/readyz?strict=true | python3 -m json.tool
# Expected: 503, strict mode fails on degraded cache

# Scenario 6: Multiple failures
curl -s http://localhost:5000/set/cache/down
curl -s http://localhost:5000/set/message_queue/degraded
curl -s http://localhost:5000/set/external_api/down
curl -s http://localhost:5000/readyz | python3 -m json.tool
# Expected: 200, non-critical failures don't affect readiness
curl -s http://localhost:5000/status | python3 -m json.tool
# Expected: Full breakdown of all dependency states

# Restore everything
curl -s http://localhost:5000/set/database/healthy
curl -s http://localhost:5000/set/cache/healthy
curl -s http://localhost:5000/set/message_queue/healthy
curl -s http://localhost:5000/set/external_api/healthy
```

---

## Success Criteria

- [ ] `/healthz` always returns 200 regardless of dependency state
- [ ] `/readyz` returns 200 when only non-critical dependencies are down
- [ ] `/readyz` returns 503 when any critical dependency is down
- [ ] `/readyz?strict=true` returns 503 when any dependency is degraded or
      down (regardless of criticality)
- [ ] `/status` returns full dependency details with latency measurements
- [ ] The main endpoint (`/`) returns 503 only when critical dependencies
      are down, not when non-critical ones are down
- [ ] The response body clearly distinguishes between critical and
      non-critical dependency failures
- [ ] You can explain why a service should continue serving traffic when
      its cache is down

## Common Mistakes to Avoid

- Returning 503 when any dependency is down -- this turns a cache miss into
  a full outage
- Not distinguishing critical from non-critical dependencies -- the database
  being down is not the same as the external API being slow
- Checking dependencies in the liveness probe -- a database outage should
  not restart your application
- Not including latency measurements -- a dependency that responds in 5
  seconds is not "healthy" even if it eventually responds
- Making the health check endpoint itself slow -- if you have 10 dependencies
  and each takes 3 seconds to time out, your health check takes 30 seconds

## What You Should Understand After This Exercise

Partial failure is the norm in distributed systems, not the exception.
Health checks that treat any failure as total failure create cascading
outages. The key design decisions are: (1) which dependencies are critical
for the service to function at all, (2) which dependencies cause degradation
but not unavailability, and (3) whether a strict mode is needed for
load balancers that want to route only to fully healthy instances. The
health endpoint is a contract between your application and the orchestrator
-- design it carefully.
