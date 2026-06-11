# Solution 04: Design Health Checks for Partial Failures

## Complete Solution

### app.py

```python
from flask import Flask, jsonify, request
import time
import os
import random
import threading

app = Flask(__name__)

start_time = time.time()

# Simulated dependency states
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
        state = dependencies.get(self.name, 'down')

        # Simulate variable latency
        if state == 'healthy':
            latency = random.uniform(1, 50)
        elif state == 'degraded':
            latency = random.uniform(100, 500)
        else:
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


@app.route('/healthz')
def healthz():
    """Liveness probe.

    Always returns 200 if the process is alive. Does not check
    any dependencies. A dependency outage should not restart
    this container.
    """
    return jsonify({
        'status': 'alive',
        'uptime_seconds': round(time.time() - start_time, 1),
    })


@app.route('/readyz')
def readyz():
    """Readiness probe.

    Returns 200 if ALL critical dependencies are healthy or degraded.
    Returns 503 if ANY critical dependency is down.

    Supports ?strict=true query parameter:
    - In strict mode, returns 200 only if ALL dependencies (critical
      and non-critical) are healthy.
    - Use strict mode for load balancers that want to route only to
      fully healthy instances.

    Response includes individual dependency statuses regardless of
    the overall status code.
    """
    strict = request.args.get('strict', 'false').lower() == 'true'

    checks = {}
    for check in CHECKS:
        checks[check.name] = check.check()

    if strict:
        # Strict mode: all dependencies must be healthy
        all_ok = all(c['status'] == 'healthy' for c in checks.values())
    else:
        # Normal mode: all critical dependencies must not be down
        all_ok = all(
            c['status'] != 'down'
            for c in checks.values()
            if c['critical']
        )

    status_code = 200 if all_ok else 503

    return jsonify({
        'status': 'ready' if all_ok else 'not_ready',
        'mode': 'strict' if strict else 'normal',
        'checks': checks,
    }), status_code


@app.route('/status')
def status():
    """Detailed status page for humans and monitoring.

    Always returns 200. This is informational, not a probe.
    Includes a summary of dependency states.
    """
    checks = {}
    for check in CHECKS:
        checks[check.name] = check.check()

    healthy_count = sum(1 for c in checks.values() if c['status'] == 'healthy')
    degraded_count = sum(1 for c in checks.values() if c['status'] == 'degraded')
    down_count = sum(1 for c in checks.values() if c['status'] == 'down')

    if down_count > 0:
        overall = 'degraded' if any(
            c['status'] != 'down' for name, c in checks.items()
            if any(ch.name == name and ch.critical for ch in CHECKS)
        ) else 'critical'
    elif degraded_count > 0:
        overall = 'degraded'
    else:
        overall = 'healthy'

    return jsonify({
        'overall_status': overall,
        'summary': {
            'healthy': healthy_count,
            'degraded': degraded_count,
            'down': down_count,
            'total': len(checks),
        },
        'checks': checks,
    })


# Toggle endpoints for testing
@app.route('/set/<dependency>/<state>')
def set_dependency(dependency, state):
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

### requirements.txt

```
flask==3.0.0
```

### Dockerfile

```dockerfile
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

### docker-compose.yml

```yaml
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

### Test Results

```bash
docker compose up -d --build
sleep 5

# Scenario 1: Everything healthy
curl -s http://localhost:5000/readyz | python3 -m json.tool
# {
#   "status": "ready",
#   "mode": "normal",
#   "checks": {
#     "database": {"status": "healthy", "latency_ms": 12.3, "critical": true, ...},
#     "cache": {"status": "healthy", "latency_ms": 5.1, "critical": false, ...},
#     "message_queue": {"status": "healthy", "latency_ms": 8.7, "critical": false, ...},
#     "external_api": {"status": "healthy", "latency_ms": 23.4, "critical": false, ...}
#   }
# }
# Status: 200

# Scenario 2: Cache is down (non-critical)
curl -s http://localhost:5000/set/cache/down
curl -s http://localhost:5000/readyz | python3 -m json.tool
# Status: 200 -- cache is non-critical, service is still ready
# cache shows "status": "down" in the checks

curl -s http://localhost:5000/ | python3 -m json.tool
# Status: 200 -- main endpoint still works

# Scenario 3: Database is degraded
curl -s http://localhost:5000/set/database/degraded
curl -s http://localhost:5000/readyz | python3 -m json.tool
# Status: 200 -- degraded is not "down", service is still ready
# database shows "status": "degraded" with higher latency

# Scenario 4: Database is down (critical)
curl -s http://localhost:5000/set/database/down
curl -s http://localhost:5000/readyz | python3 -m json.tool
# Status: 503 -- database is critical and down

curl -s http://localhost:5000/ | python3 -m json.tool
# Status: 503 -- main endpoint returns service unavailable

# Scenario 5: Strict mode with degraded cache
curl -s http://localhost:5000/set/database/healthy
curl -s http://localhost:5000/set/cache/degraded
curl -s http://localhost:5000/readyz?strict=true | python3 -m json.tool
# Status: 503 -- strict mode requires all deps to be healthy

# Scenario 6: Multiple failures
curl -s http://localhost:5000/set/cache/down
curl -s http://localhost:5000/set/message_queue/degraded
curl -s http://localhost:5000/set/external_api/down
curl -s http://localhost:5000/readyz | python3 -m json.tool
# Status: 200 -- no critical deps are down

curl -s http://localhost:5000/status | python3 -m json.tool
# {
#   "overall_status": "degraded",
#   "summary": {"healthy": 1, "degraded": 1, "down": 2, "total": 4},
#   "checks": { ... }
# }
```

---

## Why This Works

1. **Critical vs non-critical dependencies.** The database is critical --
   if it is down, the service cannot function at all. The cache, message
   queue, and external API are non-critical -- the service can still
   process requests, just with degraded performance or reduced
   functionality. This distinction is the key design decision.

2. **Three states instead of two.** Dependencies are "healthy," "degraded,"
   or "down" -- not just "up" or "down." A dependency that responds in 300ms
   is degraded (slow but functional) versus one that times out (down). The
   readiness check treats "degraded" as "still ready" because the service
   can still serve requests.

3. **Strict mode for load balancer tiers.** In a multi-tier architecture,
   you might want some load balancer instances to route only to fully
   healthy backends (strict mode) while others accept degraded backends
   (normal mode). The `?strict=true` parameter supports this without
   separate endpoints.

4. **The liveness probe never checks dependencies.** This is the same
   principle as Exercise 02, but it is even more important here. With four
   dependencies, the chance of at least one being temporarily unavailable
   is high. A liveness probe that checks all four would cause frequent
   unnecessary restarts.

5. **The `/status` endpoint is for humans.** It returns 200 always because
   it is not a probe -- it is a dashboard endpoint for monitoring systems
   and operators. The `overall_status` field gives a quick summary:
   "healthy" (all green), "degraded" (some yellow), or "critical" (red
   dependencies are down).

## Common Mistakes

### Mistake 1: Treating all dependencies as critical

```python
# WRONG: cache being down causes 503
all_ok = all(c['status'] != 'down' for c in checks.values())
```

A cache miss should not take down the service. The application should
fall back to the database. Mark only dependencies as critical that are
truly required for the service to function.

### Mistake 2: Only two states (healthy/unhealthy)

```python
# WRONG: no distinction between slow and broken
status = 'ok' if latency < 100 else 'unhealthy'
```

A dependency responding in 200ms is slow but functional. Marking it as
unhealthy removes the service from the load balancer, which increases
load on remaining instances and makes the problem worse.

### Mistake 3: Slow health check endpoint

```python
# WRONG: sequential checks with 3-second timeouts
@app.route('/readyz')
def readyz():
    db = check_database(timeout=3)       # 3s worst case
    cache = check_cache(timeout=3)       # 3s worst case
    mq = check_message_queue(timeout=3)  # 3s worst case
    api = check_external_api(timeout=3)  # 3s worst case
    # Total: 12 seconds worst case
```

If the health check itself takes 12 seconds, the orchestrator times out
and marks the container as unhealthy even though it is fine. Run checks
in parallel or use shorter timeouts:

```python
import concurrent.futures

@app.route('/readyz')
def readyz():
    with concurrent.futures.ThreadPoolExecutor() as executor:
        futures = {
            executor.submit(check.check): check.name
            for check in CHECKS
        }
        results = {}
        for future in concurrent.futures.as_completed(futures):
            name = futures[future]
            results[name] = future.result()
    # ...
```

### Mistake 4: Not including dependency details in the response

```python
# WRONG: binary yes/no with no context
return jsonify({'status': 'ready'}), 200
```

When the readiness probe fails, the operator needs to know *which*
dependency is down. Always include individual dependency statuses in the
response.

### Mistake 5: Using strict mode as the default

If you use `?strict=true` as the default readiness check, a single
degraded dependency removes the instance from the load balancer. With
enough dependencies, you will always have at least one degraded, and
no instances will be in the load balancer.
