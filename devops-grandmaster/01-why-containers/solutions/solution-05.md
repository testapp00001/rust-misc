# Solution 05: From Broken Deployment to Containerized Pipeline

## Part A: Root Cause Chain

### Failure Chain Analysis

```
Failure 1: Python version mismatch
├── Trigger: Developer uses Python 3.11 (macOS/Homebrew), production has 3.8 (Ubuntu 20.04 system Python)
├── Prevention that was missing: No version pinning for the Python runtime
├── Prevention that would work: Container image pins Python version in FROM directive
│
Failure 2: No automated environment validation
├── Trigger: CI server runs Python 3.10, different from both dev (3.11) and prod (3.8)
├── Prevention that was missing: CI did not test in a production-like environment
├── Prevention that would work: CI builds and tests the same container image that deploys to production
│
Failure 3: Dependency version conflicts
├── Trigger: API v2.3 requires Python 3.10+, but production has 3.8
├── Prevention that was missing: requirements.txt does not specify Python version
├── Prevention that would work: Container image includes Python + all dependencies in one artifact
│
Failure 4: Rollback fails due to dependency mismatch
├── Trigger: API v2.2 requires a different Redis version than what is installed
├── Prevention that was missing: Rollback only replaced the application code, not the runtime environment
├── Prevention that would work: Rollback deploys a previous container image with its own bundled dependencies
│
Failure 5: Manual rollback process
├── Trigger: Rollback required SSH access, manual package installation, service restart
├── Prevention that was missing: No automated rollback mechanism
├── Prevention that would work: Orchestrator automatically rolls back on health check failure
│
Failure 6: No integration testing in production-like environment
├── Trigger: Three different environments (dev, CI, prod) with three different Python versions
├── Prevention that was missing: No staging environment matching production
├── Prevention that would work: Staging runs the same container image on the same infrastructure
│
Failure 7: Slow incident response
├── Trigger: 45 minutes to identify root cause, 2.5 hours to restore service
├── Prevention that was missing: No runbook for deployment failures, no automated diagnostics
├── Prevention that would work: Health checks trigger automatic rollback within 30 seconds
```

### Why This Chain Matters

Each failure is a link in a chain. Removing any one link would have reduced
the outage duration or prevented it entirely. Containers address links 1-5
simultaneously by making the runtime environment part of the deployable artifact.

## Part B: Design the Containerized Solution

### Architecture Diagram

```
Containerized Deployment Pipeline
==================================

Developer Machine
┌─────────────────────────────────────┐
│  Docker Desktop                     │
│  ┌───────────────────────────────┐  │
│  │ Same image as production      │  │
│  │ python:3.11-slim + app + deps │  │
│  └───────────────────────────────┘  │
│  docker build && docker run         │
│  Tests pass locally? Yes → git push │
└──────────────┬──────────────────────┘
               │ git push
               ▼
CI Server (GitHub Actions / GitLab CI)
┌─────────────────────────────────────┐
│  1. Checkout code                   │
│  2. docker build -t api:$SHA        │
│  3. docker run api:$SHA pytest      │
│  4. Scan image (Trivy)              │
│  5. docker push api:$SHA            │
│                                     │
│  Image stored in registry with      │
│  immutable tag (git SHA)            │
└──────────────┬──────────────────────┘
               │ image reference
               ▼
Container Registry
┌─────────────────────────────────────┐
│  myregistry.com/api                 │
│  ├── sha-abc123 (v2.3)              │
│  ├── sha-def456 (v2.2) ← rollback  │
│  └── sha-ghi789 (v2.1)             │
└──────────────┬──────────────────────┘
               │ pull image
               ▼
Production Server
┌─────────────────────────────────────┐
│  Container Runtime (Docker)         │
│  ┌───────────────────────────────┐  │
│  │ api:sha-abc123 (current)      │  │
│  │ Health check: /health         │  │
│  │ Fails? → Auto-rollback to     │  │
│  │ sha-def456 (previous)         │  │
│  └───────────────────────────────┘  │
│                                     │
│  ┌───────────────────────────────┐  │
│  │ redis:7.0-sha-xyz (sidecar)   │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

### How This Prevents Each Failure

| Original Failure | Container Prevention |
|------------------|---------------------|
| Python version mismatch | `FROM python:3.11-slim` pins version in image |
| CI differs from production | CI builds the *same image* that deploys to prod |
| Dependency conflicts | `pip install` runs during image build, not at deploy time |
| Rollback fails | Rollback = run previous image (with its own dependencies) |
| Manual rollback | Orchestrator auto-rolls back on health check failure |
| No prod-like testing | Staging runs the exact same image |
| Slow response | Automatic rollback in 30 seconds, not 2.5 hours |

## Part C: Dockerfile Design

```dockerfile
# Pin exact Python version - not "python:3.11" but "python:3.11.7"
FROM python:3.11.7-slim-bookworm

# Security: Update system packages
RUN apt-get update && \
    apt-get upgrade -y && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd --gid 1000 appuser && \
    useradd --uid 1000 --gid appuser --create-home appuser

WORKDIR /app

# Install Python dependencies first (layer caching)
COPY requirements.txt .
RUN pip install --no-cache-dir --no-compile -r requirements.txt

# Copy application code
COPY src/ ./src/

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Health check - curl must be installed or use python-based check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD python3 -c "import urllib.request; urllib.request.urlopen('http://localhost:8000/health')" || exit 1

# Expose port (documentation, not enforcement)
EXPOSE 8000

# Run the application
CMD ["python3", "-m", "uvicorn", "src.main:app", "--host", "0.0.0.0", "--port", "8000"]
```

### Why This Dockerfile Works

1. **`python:3.11.7-slim-bookworm`** pins the exact Python version, OS
   variant (slim), and Debian version (bookworm). No ambiguity.

2. **`apt-get upgrade -y`** patches known vulnerabilities in the base
   image's system packages.

3. **Non-root user** prevents container breakout attacks from gaining
   root access on the host.

4. **Dependencies installed before code** leverages Docker's layer caching.
   If only the code changes, the dependency layer is reused from cache.

5. **`--no-cache-dir` and `--no-compile`** reduce image size by not
   storing pip's download cache or Python bytecode.

6. **HEALTHCHECK** gives the orchestrator a way to know if the application
   is healthy. Failed health checks trigger automatic rollback.

### requirements.txt

```
fastapi==0.104.1
uvicorn==0.24.0
redis==5.0.1
psycopg2-binary==2.9.9
pydantic==2.5.2
```

Every version is pinned. No `>=` or `~=` ranges. The container image
builds deterministically.

## Part D: Rollback Strategy

### Automated Rollback Procedure

```
Step 1: Health check monitors the current deployment
═══════════════════════════════════════════════════

  Every 30 seconds:
    curl http://localhost:8000/health

  Expected: HTTP 200
  Failure threshold: 3 consecutive failures

Step 2: Health check failure triggers rollback
═══════════════════════════════════════════════════

  After 3 failures (90 seconds):
    1. Stop current container:    docker stop api-current
    2. Remove current container:  docker rm api-current
    3. Start previous image:      docker run -d --name api-current myregistry/api:sha-def456
    4. Verify health:             curl http://localhost:8000/health

  Total rollback time: ~15-30 seconds

Step 3: Verify rollback success
═══════════════════════════════════════════════════

  Health check passes on rollback image?
    Yes → Alert team, investigate failed deployment
    No  → Escalate to on-call engineer (manual intervention)
```

### Why This Works

The key insight: **each container image is self-contained.** When you roll
back to `api:sha-def456`, you get that version's Python, dependencies,
and code -- all bundled together. The "v2.2 requires different Redis version"
problem cannot happen because each image has its own Redis client library
baked in.

### Automated Rollback Script

```bash
#!/bin/bash
# rollback.sh -- Automated container rollback

REGISTRY="myregistry.com/api"
CURRENT_TAG=$1       # e.g., sha-abc123
PREVIOUS_TAG=$2      # e.g., sha-def456
HEALTH_URL="http://localhost:8000/health"
MAX_WAIT=30

echo "Rolling back from $CURRENT_TAG to $PREVIOUS_TAG..."

# Stop and remove current container
docker stop api-current 2>/dev/null
docker rm api-current 2>/dev/null

# Start previous version
docker run -d \
    --name api-current \
    -p 8000:8000 \
    --restart unless-stopped \
    ${REGISTRY}:${PREVIOUS_TAG}

# Wait for health check
echo "Waiting for health check..."
for i in $(seq 1 $MAX_WAIT); do
    if curl -sf $HEALTH_URL > /dev/null 2>&1; then
        echo "Rollback successful. Service healthy after ${i}s."
        exit 0
    fi
    sleep 1
done

echo "ERROR: Rollback failed. Service not healthy after ${MAX_WAIT}s."
exit 1
```

### With an Orchestrator (Docker Compose example)

```yaml
# docker-compose.yml
version: "3.8"

services:
  api:
    image: myregistry/api:sha-abc123  # Change this to roll back
    ports:
      - "8000:8000"
    healthcheck:
      test: ["CMD", "python3", "-c", "import urllib.request; urllib.request.urlopen('http://localhost:8000/health')"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 10s
    restart: unless-stopped
```

Rollback:

```bash
# Change image tag in docker-compose.yml, then:
docker-compose up -d
# Old container stopped, new (previous) container started
```

## Part E: Post-Mortem Action Items

| # | Action Item | Prevents | Effort | Priority |
|---|-------------|----------|--------|----------|
| 1 | Create Dockerfile for API server | Python version mismatch, dependency conflicts | 2 hours | P0 |
| 2 | Set up container registry | Image distribution | 1 hour | P0 |
| 3 | Add container build step to CI | CI/prod environment mismatch | 4 hours | P0 |
| 4 | Add health check endpoint to API | Enables automatic rollback detection | 1 hour | P0 |
| 5 | Implement automated rollback | Manual rollback failures, slow recovery | 4 hours | P0 |
| 6 | Pin all dependency versions | Dependency conflicts during rollback | 2 hours | P1 |
| 7 | Add image vulnerability scanning | Security risks in dependencies | 2 hours | P1 |
| 8 | Create staging environment with same images | Catching issues before production | 1 day | P1 |
| 9 | Write deployment runbook | Faster manual intervention when needed | 2 hours | P1 |
| 10 | Implement canary deployments | Catching issues before full rollout | 2 days | P2 |
| 11 | Set up monitoring and alerting | Faster detection of failures | 1 day | P2 |
| 12 | Document rollback procedure | Team knowledge sharing | 1 hour | P2 |

### Implementation Order

```
Week 1 (P0 items):
  Day 1: Create Dockerfile (#1) + health check endpoint (#4)
  Day 2: Set up registry (#2) + CI pipeline (#3)
  Day 3: Implement automated rollback (#5) + test end-to-end
  Day 4: Pin dependencies (#6) + validate rollback works
  Day 5: Run drill: simulate failure, verify automatic rollback

Week 2 (P1 items):
  Image scanning (#7) + staging environment (#8) + runbook (#9)

Week 3+ (P2 items):
  Canary deployments (#10) + monitoring (#11) + documentation (#12)
```

### Common Mistakes to Avoid

- **Not testing the rollback.** A rollback strategy that has never been
  tested is not a strategy -- it is a hope. Run rollback drills regularly.
- **Using mutable tags.** `api:latest` can point to different images at
  different times. Always use immutable tags (git SHA, build number).
- **Forgetting about database migrations.** Rolling back the application
  does not roll back the database schema. Design migrations to be
  backward-compatible (expand-and-contract pattern).
- **No health check endpoint.** Without a health check, the orchestrator
  cannot know if the application is working. Always implement `/health`.
- **Rolling back to a version with known vulnerabilities.** Keep track
  of which image versions have been security-scanned. A rollback to an
  unscanned image might reintroduce vulnerabilities.

## Key Takeaway

The container image is the single source of truth. It contains the runtime,
the dependencies, and the code. The image that passes CI is the image that
runs in production. Rollback is trivial because each image is self-contained.
This eliminates the entire class of "it works on my machine" problems and
turns a 4-hour outage into a 30-second automatic recovery.
