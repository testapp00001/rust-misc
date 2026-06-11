# Module 12: Health Checks

## The Problem: Is Your App Actually Working?

A container is "running" — but is it actually serving requests?

```bash
docker ps
# CONTAINER ID   IMAGE   STATUS        PORTS
# abc123         myapp   Up 5 minutes  0.0.0.0:8080->8080/tcp
```

Status says "Up" — but:
- The app might be stuck in a deadlock
- The database connection might be down
- The app might be starting up and not ready yet
- The app might be crashing internally but the process hasn't exited

**"Running" ≠ "Healthy"**

## The Naive Way: Check if Process Exists

```bash
# Docker checks if the main process is running
# That's it. That's all "Up" means.
```

**Why this fails:**
- Process running ≠ app working
- App might be unresponsive
- App might have lost database connection
- App might be in a crash loop internally

## The Right Way: HTTP Health Checks

Add a health endpoint to your application:

```python
from flask import Flask, jsonify
import redis
import psycopg2

app = Flask(__name__)

@app.route('/health')
def health():
    """Basic health check — is the app running?"""
    return jsonify({'status': 'ok'})

@app.route('/ready')
def ready():
    """Readiness check — can the app serve traffic?"""
    checks = {}
    
    # Check database
    try:
        conn = psycopg2.connect(os.environ['DATABASE_URL'])
        conn.execute('SELECT 1')
        conn.close()
        checks['database'] = 'ok'
    except Exception as e:
        checks['database'] = f'error: {str(e)}'
    
    # Check Redis
    try:
        r = redis.from_url(os.environ['REDIS_URL'])
        r.ping()
        checks['redis'] = 'ok'
    except Exception as e:
        checks['redis'] = f'error: {str(e)}'
    
    all_ok = all(v == 'ok' for v in checks.values())
    status_code = 200 if all_ok else 503
    
    return jsonify({'status': 'ok' if all_ok else 'unhealthy', 'checks': checks}), status_code
```

### Liveness vs Readiness

**Liveness** — Is the app alive?
- If liveness fails → restart the container
- Use for detecting deadlocks, infinite loops

**Readiness** — Is the app ready to serve traffic?
- If readiness fails → stop sending traffic, but don't restart
- Use for startup time, dependency checks

```python
@app.route('/healthz')    # Liveness
def liveness():
    return jsonify({'status': 'alive'})

@app.route('/readyz')     # Readiness
def readiness():
    # Check all dependencies
    if not all_dependencies_ok():
        return jsonify({'status': 'not ready'}), 503
    return jsonify({'status': 'ready'})
```

## Docker Health Checks

### In Dockerfile
```dockerfile
HEALTHCHECK --interval=30s --timeout=5s --retries=3 --start-period=10s \
  CMD curl -f http://localhost:8080/health || exit 1

# --interval=30s    Check every 30 seconds
# --timeout=5s      Fail if no response in 5 seconds
# --retries=3       Mark unhealthy after 3 failures
# --start-period=10s Give container 10s to start before checking
```

### In Docker Run
```bash
docker run \
  --health-cmd="curl -f http://localhost:8080/health || exit 1" \
  --health-interval=30s \
  --health-timeout=5s \
  --health-retries=3 \
  my-app
```

### In Docker Compose
```yaml
services:
  web:
    image: my-app
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 10s
```

## Check Health Status

```bash
# See health status
docker ps
# CONTAINER ID   IMAGE   STATUS                     PORTS
# abc123         myapp   Up 5 minutes (healthy)      0.0.0.0:8080->8080/tcp
# def456         myapp   Up 2 minutes (unhealthy)    0.0.0.0:8081->8080/tcp

# Detailed health info
docker inspect --format='{{json .State.Health}}' myapp | jq

# Health check logs
docker inspect --format='{{range .State.Health.Log}}{{.Output}}{{end}}' myapp
```

## Restart Policies

```bash
# Always restart (except when manually stopped)
docker run --restart unless-stopped my-app

# Restart on failure only
docker run --restart on-failure my-app

# Restart on failure, max 5 retries
docker run --restart on-failure:5 my-app

# Never restart (default)
docker run --restart no my-app
```

## Docker Compose with Health Checks and Depends On

```yaml
version: '3.8'

services:
  db:
    image: postgres:15
    environment:
      POSTGRES_PASSWORD: secret
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5

  web:
    image: my-app
    depends_on:
      db:
        condition: service_healthy
      redis:
        condition: service_healthy
    ports:
      - "8080:8080"
```

## Hands-On Exercise

### Exercise 1: Add Health Checks to a Flask App

Create `app.py`:
```python
from flask import Flask, jsonify
import time
import threading

app = Flask(__name__)
start_time = time.time()
is_ready = False

def startup():
    """Simulate slow startup"""
    global is_ready
    time.sleep(10)  # Simulate 10s startup
    is_ready = True

threading.Thread(target=startup, daemon=True).start()

@app.route('/health')
def health():
    """Liveness: always returns 200 if process is running"""
    return jsonify({'status': 'alive', 'uptime': time.time() - start_time})

@app.route('/ready')
def ready():
    """Readiness: returns 200 only when ready to serve"""
    if is_ready:
        return jsonify({'status': 'ready'})
    return jsonify({'status': 'starting'}), 503

@app.route('/')
def index():
    if not is_ready:
        return jsonify({'error': 'not ready'}), 503
    return jsonify({'message': 'Hello!'})
```

Dockerfile:
```dockerfile
FROM python:3.11-slim
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY requirements.txt .
RUN pip install -r requirements.txt
COPY . .
EXPOSE 5000
HEALTHCHECK --interval=5s --timeout=3s --retries=3 --start-period=5s \
  CMD curl -f http://localhost:5000/health || exit 1
CMD ["python", "app.py"]
```

```bash
docker build -t health-demo .
docker run -d --name health-demo -p 5000:5000 health-demo

# Watch health status change from "starting" to "healthy"
watch docker ps

# Check health endpoint
curl http://localhost:5000/health
curl http://localhost:5000/ready
```

### Exercise 2: Health Check with Dependency Checks

```python
import redis
import psycopg2
import os

@app.route('/ready')
def ready():
    checks = {}
    
    # Database check
    try:
        conn = psycopg2.connect(os.environ.get('DATABASE_URL', 'postgresql://localhost'))
        cur = conn.cursor()
        cur.execute('SELECT 1')
        cur.close()
        conn.close()
        checks['database'] = 'ok'
    except Exception as e:
        checks['database'] = str(e)
    
    # Redis check
    try:
        r = redis.from_url(os.environ.get('REDIS_URL', 'redis://localhost'))
        r.ping()
        checks['redis'] = 'ok'
    except Exception as e:
        checks['redis'] = str(e)
    
    all_ok = all(v == 'ok' for v in checks.values())
    return jsonify({
        'status': 'ready' if all_ok else 'not ready',
        'checks': checks
    }), 200 if all_ok else 503
```

## Limitation: Health Checks Tell You IF It's Working, But Not HOW to Configure It

Your app checks health, but the configuration (database URL, API keys) is hardcoded or in environment variables that are baked into the image.

**Next problem:** How do you configure your app for different environments without rebuilding?

→ **Next module:** [13-environment-variables](../13-environment-variables/) — Config without rebuilding images

## Checklist

- [ ] I have a /health endpoint (liveness)
- [ ] I have a /ready endpoint (readiness) that checks dependencies
- [ ] I configure HEALTHCHECK in Dockerfile or docker-compose.yml
- [ ] I use --restart policies for automatic recovery
- [ ] I understand the difference between liveness and readiness
- [ ] I use depends_on with condition: service_healthy
