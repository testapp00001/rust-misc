# Exercise 05: Production-Ready Dockerfile

**Type:** Integration
**Time:** 60 minutes
**Difficulty:** Hard

## Objective

Build a production-ready Dockerfile that combines everything from this module
with what you learned in [Module 01: Why Containers](../01-why-containers/)
and [Module 02: Your First Container](../02-your-first-container/).
This is not a toy -- the Dockerfile you write should be suitable for
deploying to a real production environment.

## The Application

You are containerizing a Python web application that will be deployed
to a production Kubernetes cluster. The application needs to be secure,
observable, and optimized.

**app.py:**
```python
import os
import logging
from flask import Flask, jsonify

# Configure logging for production
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s %(levelname)s %(message)s'
)
logger = logging.getLogger(__name__)

app = Flask(__name__)

@app.route('/')
def hello():
    logger.info('Root endpoint hit')
    return jsonify({
        'message': 'Production API',
        'version': os.environ.get('APP_VERSION', 'unknown'),
        'environment': os.environ.get('APP_ENV', 'development')
    })

@app.route('/health')
def health():
    return jsonify({'status': 'healthy'})

@app.route('/ready')
def ready():
    # In a real app, check database connections, cache, etc.
    return jsonify({'status': 'ready'})

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

**requirements.txt:**
```
flask==3.0.0
gunicorn==21.2.0
```

## Requirements

Your Dockerfile must satisfy ALL of the following production requirements:

### Security Requirements
1. Use a specific, slim base image (no `latest`)
2. Run as a non-root user
3. Do not install unnecessary packages
4. Use `--no-cache-dir` for pip to avoid storing download cache in the image
5. Do not include secrets in the image (no `.env`, no hardcoded passwords)

### Performance Requirements
6. Optimize layer caching (dependencies before code)
7. Minimize the number of layers where practical
8. Clean up package manager caches in the same layer they are created
9. Use a `.dockerignore` to keep the build context small

### Operational Requirements
10. Set environment variables for Python (unbuffered output for logging)
11. Include a HEALTHCHECK instruction
12. Document the exposed port with EXPOSE
13. Use gunicorn (not the Flask dev server) as the production process
14. Accept build-time arguments for version tagging

### Logging Requirements
15. Ensure Python logs are flushed immediately (not buffered)
    so that container orchestrators can collect them

## Tasks

### Part A: Write the Dockerfile

Write a complete Dockerfile that meets all 15 requirements above.

<details>
<summary>Hint 1: Environment Variables</summary>

Set these environment variables early in the Dockerfile:

```dockerfile
ENV PYTHONUNBUFFERED=1
ENV PYTHONDONTWRITEBYTECODE=1
```

`PYTHONUNBUFFERED=1` forces stdout/stderr to be unbuffered, which is critical
for container logging. `PYTHONDONTWRITEBYTECODE=1` prevents Python from
writing `.pyc` files.

</details>

<details>
<summary>Hint 2: Build Arguments</summary>

Use `ARG` for values that should be set at build time:

```dockerfile
ARG APP_VERSION=dev
ENV APP_VERSION=${APP_VERSION}
```

Build with: `docker build --build-arg APP_VERSION=1.2.3 -t myapp:1.2.3 .`

</details>

<details>
<summary>Hint 3: Health Check</summary>

Your app has both `/health` and `/ready` endpoints. Use `/health` for the
Docker HEALTHCHECK since it checks if the process is alive:

```dockerfile
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD python -c "import urllib.request; urllib.request.urlopen('http://localhost:5000/health')" || exit 1
```

This uses Python's built-in `urllib` instead of requiring `curl` to be
installed, keeping the image smaller.

</details>

<details>
<summary>Hint 4: Gunicorn Command</summary>

Use gunicorn with settings appropriate for containers:

```dockerfile
CMD ["gunicorn", "--bind", "0.0.0.0:5000", "--workers", "2", "--access-logfile", "-", "app:app"]
```

`--access-logfile -` sends access logs to stdout so the container
orchestrator can collect them. `--workers 2` is a reasonable default.

</details>

### Part B: Write the .dockerignore

Write a `.dockerignore` that excludes everything not needed in the
production image.

<details>
<summary>Hint</summary>

```
.git
.gitignore
__pycache__
*.pyc
*.pyo
.env
.env.*
.DS_Store
*.md
tests/
test/
docs/
Dockerfile
docker-compose*.yml
.dockerignore
```

</details>

### Part C: Write a docker-compose.yml (Bonus)

Write a `docker-compose.yml` that builds and runs your application
with proper port mapping, health check, and environment variables.

<details>
<summary>Hint</summary>

```yaml
version: "3.8"

services:
  api:
    build:
      context: .
      args:
        APP_VERSION: "1.0.0"
    ports:
      - "5000:5000"
    environment:
      - APP_ENV=production
    healthcheck:
      test: ["CMD", "python", "-c", "import urllib.request; urllib.request.urlopen('http://localhost:5000/health')"]
      interval: 30s
      timeout: 3s
      retries: 3
      start_period: 5s
    restart: unless-stopped
```

</details>

### Part D: Test Everything

Build and test your production image:

```bash
# Build with version tag
docker build --build-arg APP_VERSION=1.0.0 -t myapp:1.0.0 .

# Run the container
docker run -d --name myapp -p 5000:5000 -e APP_ENV=production myapp:1.0.0

# Test endpoints
curl http://localhost:5000/
curl http://localhost:5000/health
curl http://localhost:5000/ready

# Check health status
docker inspect --format='{{.State.Health.Status}}' myapp

# Verify non-root user
docker exec myapp whoami

# Check image size
docker images myapp:1.0.0

# Check logs (should appear immediately due to PYTHONUNBUFFERED)
docker logs myapp
```

### Part E: Security Audit

Run the following checks on your image:

```bash
# Check for root user in the running container
docker exec myapp id

# List installed packages (should be minimal)
docker exec myapp pip list

# Check for .env files (should not exist)
docker exec myapp ls -la /app/.env 2>&1

# Inspect the image layers
docker history myapp:1.0.0
```

Document any security findings.

## Success Criteria

- [ ] All 15 requirements are met in your Dockerfile
- [ ] The image builds without errors
- [ ] The container runs and responds to all three endpoints
- [ ] The container runs as a non-root user (`whoami` returns `appuser`)
- [ ] The HEALTHCHECK reports healthy status
- [ ] Logs appear immediately (not buffered)
- [ ] The image is under 200MB
- [ ] You have a `.dockerignore` that excludes unnecessary files
- [ ] You can explain every line in your Dockerfile and why it is there

## What You Should Understand After This Exercise

A production Dockerfile is not just about making code run in a container.
It is about security (non-root, no secrets, minimal packages), observability
(unbuffered logs, health checks), performance (layer caching, small images),
and operability (build args, environment variables). Every line serves a
purpose, and removing any line would make the image less production-worthy.
