# Solution 05: Production-Ready Dockerfile

## Complete Dockerfile

```dockerfile
# === Build arguments ===
ARG PYTHON_VERSION=3.11

# === Base image ===
FROM python:${PYTHON_VERSION}-slim-bookworm

# === Environment variables ===
ENV PYTHONUNBUFFERED=1
ENV PYTHONDONTWRITEBYTECODE=1
ARG APP_VERSION=dev
ENV APP_VERSION=${APP_VERSION}

# === Working directory ===
WORKDIR /app

# === System dependencies (none needed for this app) ===
# If you needed system packages:
# RUN apt-get update && \
#     apt-get install -y --no-install-recommends curl && \
#     rm -rf /var/lib/apt/lists/*

# === Python dependencies ===
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# === Application code ===
COPY . .

# === Security: non-root user ===
RUN adduser --disabled-password --gecos '' appuser
USER appuser

# === Port documentation ===
EXPOSE 5000

# === Health check ===
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD python -c "import urllib.request; urllib.request.urlopen('http://localhost:5000/health')" || exit 1

# === Production command ===
CMD ["gunicorn", "--bind", "0.0.0.0:5000", "--workers", "2", "--access-logfile", "-", "app:app"]
```

## Complete .dockerignore

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

## Complete docker-compose.yml

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

---

## Detailed Explanation

### Build Arguments (ARG)

```dockerfile
ARG PYTHON_VERSION=3.11
FROM python:${PYTHON_VERSION}-slim-bookworm
```

**Why:** `ARG` before `FROM` allows you to parameterize the base image version. This lets you build with different Python versions without editing the Dockerfile:

```bash
docker build --build-arg PYTHON_VERSION=3.12 -t myapp:1.0 .
```

The default value (`3.11`) is used if no build arg is provided.

### Environment Variables (ENV)

```dockerfile
ENV PYTHONUNBUFFERED=1
ENV PYTHONDONTWRITEBYTECODE=1
```

**PYTHONUNBUFFERED=1:** Python buffers stdout/stderr by default. In a container, this means log messages sit in a buffer and may not appear until the buffer fills or the process exits. Setting this to `1` forces immediate flushing, which is critical for container logging. Without this, `docker logs` may show nothing until the container stops.

**PYTHONDONTWRITEBYTECODE=1:** Prevents Python from writing `.pyc` files to disk. In a container, these files are useless (the container filesystem is ephemeral) and add unnecessary writes.

### Version Tagging via Build Args

```dockerfile
ARG APP_VERSION=dev
ENV APP_VERSION=${APP_VERSION}
```

**Why:** The `ARG` is available at build time. By setting it as an `ENV`, the application can read it at runtime via `os.environ.get('APP_VERSION')`. This lets the app report its own version, which is useful for debugging and observability.

### Health Check Without curl

```dockerfile
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD python -c "import urllib.request; urllib.request.urlopen('http://localhost:5000/health')" || exit 1
```

**Why not curl:** Installing `curl` adds a package to the image. Since Python is already available, using `urllib.request` achieves the same result without additional dependencies.

**`--start-period=5s`:** Gives the application 5 seconds to start before health checks begin. Without this, the container might be marked unhealthy during startup, causing unnecessary restarts.

**`--retries=3`:** The container is marked unhealthy only after 3 consecutive failures. This prevents transient network issues from triggering false alarms.

### Gunicorn as Production Server

```dockerfile
CMD ["gunicorn", "--bind", "0.0.0.0:5000", "--workers", "2", "--access-logfile", "-", "app:app"]
```

**`--bind 0.0.0.0:5000`:** Binds to all network interfaces. Using `127.0.0.1` would only allow connections from inside the container.

**`--workers 2`:** Runs 2 worker processes. A common formula is `2 * CPU cores + 1`, but in containers, 2-4 workers is typical. Too many workers waste memory; too few leave CPU idle.

**`--access-logfile -`:** Sends access logs to stdout. Container orchestrators (Docker, Kubernetes) collect stdout/stderr logs. Writing to a file inside the container makes logs harder to collect and they are lost when the container stops.

**`app:app`:** Tells gunicorn to import the `app` object from the `app` module. This is the standard gunicorn WSGI entry point notation.

### Security Checklist

| Requirement | How It Is Met |
|-------------|--------------|
| Specific base image tag | `python:3.11-slim-bookworm` (not `latest`) |
| Non-root user | `adduser` + `USER appuser` |
| No unnecessary packages | No `apt-get install` (none needed) |
| No pip cache | `--no-cache-dir` flag |
| No secrets in image | `.env` excluded via `.dockerignore` |

### Observability Checklist

| Requirement | How It Is Met |
|-------------|--------------|
| Unbuffered logs | `PYTHONUNBUFFERED=1` |
| Health check | `HEALTHCHECK` with `--start-period` and `--retries` |
| Access logs to stdout | `--access-logfile -` in gunicorn |
| Version reporting | `APP_VERSION` environment variable |

### Performance Checklist

| Requirement | How It Is Met |
|-------------|--------------|
| Layer caching | `requirements.txt` copied before application code |
| Minimal layers | Combined related commands |
| Small image | Slim base image, no unnecessary packages |
| Small build context | `.dockerignore` excludes `.git`, tests, docs |

---

## Verification Commands

```bash
# Build with version
docker build --build-arg APP_VERSION=1.0.0 -t myapp:1.0.0 .

# Run
docker run -d --name myapp -p 5000:5000 -e APP_ENV=production myapp:1.0.0

# Test endpoints
curl http://localhost:5000/
curl http://localhost:5000/health
curl http://localhost:5000/ready

# Check health status (wait 30+ seconds for first check)
docker inspect --format='{{.State.Health.Status}}' myapp

# Verify non-root
docker exec myapp whoami
# Expected: appuser

# Verify version is available
docker exec myapp python -c "import os; print(os.environ.get('APP_VERSION'))"
# Expected: 1.0.0

# Check image size
docker images myapp:1.0.0
# Expected: ~160MB

# Check logs (should appear immediately)
docker logs myapp

# Check layers
docker history myapp:1.0.0

# Clean up
docker stop myapp && docker rm myapp
```

---

## Common Mistakes to Avoid

1. **Forgetting PYTHONUNBUFFERED.** Without it, logs may not appear in `docker logs` until the container stops. This makes debugging production issues nearly impossible.

2. **Installing curl for health checks.** Python's `urllib` is already available. Installing curl adds an unnecessary package.

3. **Not using --start-period.** Without it, the container is health-checked immediately on start. If the app takes a few seconds to initialize, it gets marked unhealthy and restarted repeatedly.

4. **Hardcoding the version in the Dockerfile.** Use `ARG` so the version can be set at build time. This enables CI/CD pipelines to tag images with the git commit or release version.

5. **Writing access logs to a file.** Container orchestrators collect stdout/stderr. Logs written to files inside the container are lost when the container stops and are not available in centralized logging systems.

6. **Using too many gunicorn workers.** Each worker consumes memory. In a container with a memory limit, too many workers cause OOM kills. Start with 2-4 workers and tune based on actual load.

7. **Not excluding .env in .dockerignore.** The `.env` file often contains database passwords, API keys, and other secrets. If it is in the build context, it gets copied into the image. Anyone with access to the image can extract the secrets.

---

## How This Connects to Previous Modules

- **Module 01 (Why Containers):** This Dockerfile solves the "works on my machine" problem by packaging the exact Python version, dependencies, and configuration into a reproducible image.

- **Module 02 (Your First Container):** This Dockerfile builds on the basics by adding production hardening: non-root user, health checks, proper logging, and optimized layer caching.

The progression from "run someone else's container" to "write a basic Dockerfile" to "write a production-ready Dockerfile" mirrors the DevOps maturity model: adoption, competence, and operational excellence.
