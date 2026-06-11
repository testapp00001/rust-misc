# Exercise 05: From Broken Deployment to Containerized Pipeline

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Take a broken multi-service deployment, diagnose the environment issues,
and design a complete containerization strategy that prevents the entire
class of problems. This exercise integrates everything from Module 01.

## Scenario

You join a team that just had a catastrophic production failure.
The post-mortem reads:

```
POST-MORTEM: Production Outage 2024-03-15
==========================================

Duration: 4 hours
Impact: Complete service outage, ~12,000 users affected
Root Cause: Deployment of incompatible service versions

Timeline:
  09:00 - Developer deploys API v2.3 (requires Python 3.10+)
  09:01 - API server crashes on startup
  09:02 - Load balancer removes API server from pool
  09:03 - All requests return 503
  09:15 - On-call engineer begins investigation
  09:45 - Root cause identified: Python version mismatch
  10:30 - Rollback to API v2.2 attempted
  10:45 - Rollback fails: v2.2 requires different Redis version
  11:00 - Manually installs correct Python and Redis versions
  13:00 - Service restored

Contributing Factors:
  1. Production server runs Ubuntu 20.04 with system Python 3.8
  2. Developer runs macOS with Python 3.11 via Homebrew
  3. No version pinning for system dependencies
  4. CI server runs Ubuntu 22.04 with Python 3.10
  5. Three different environments, three different Python versions
  6. No integration testing in a production-like environment
  7. Rollback process is manual and error-prone
```

## Tasks

### Part A: Root Cause Chain

Map out the complete chain of failures. For each link in the chain,
identify what *should* have prevented it.

Create a document called `root-cause-analysis.md` with this format:

```
Failure 1: Python version mismatch
  Trigger: Developer uses Python 3.11, production has 3.8
  Prevention that was missing: _______________
  Prevention that would work: _______________

Failure 2: _______________
  Trigger: _______________
  Prevention that was missing: _______________
  Prevention that would work: _______________

...
```

<details>
<summary>Hint</summary>

There are at least 5 distinct failures in this chain:
1. Version mismatch between dev and prod
2. No automated environment validation
3. CI environment differs from production
4. Rollback depends on compatible dependency versions
5. Manual rollback process is slow and error-prone

For each, think about what *system-level* solution prevents it,
not just what *human behavior* change would help.

</details>

### Part B: Design the Containerized Solution

Design a container-based deployment pipeline that prevents every failure
identified in Part A. Your design should include:

1. **Development environment** -- How do developers work locally?
2. **Build process** -- How are container images created?
3. **CI/CD pipeline** -- How are images tested and promoted?
4. **Production deployment** -- How do images reach production?
5. **Rollback strategy** -- How do you recover from a bad deployment?

Draw an ASCII architecture diagram showing the complete flow.

<details>
<summary>Hint</summary>

```
Developer Machine
  └── Same container image as production
        └── git push
              └── CI Server
                    └── Build image from Dockerfile
                          └── Run tests IN the image
                                └── Push to registry
                                      └── Deploy to production
                                            └── Rollback = run previous image
```

The key insight: the container image built in CI is the *exact same image*
that runs in production. No environment differences possible.

</details>

### Part C: Dockerfile Design

Write a Dockerfile for the API server that:

1. Pins the exact Python version
2. Installs all dependencies from `requirements.txt`
3. Copies the application code
4. Runs as a non-root user
5. Defines a health check
6. Does not include development tools or test files

```python
# For reference, the API server's requirements.txt looks like:
# fastapi==0.104.1
# uvicorn==0.24.0
# redis==5.0.1
# psycopg2-binary==2.9.9
# pydantic==2.5.2
```

<details>
<summary>Hint</summary>

```dockerfile
FROM python:3.11-slim

# Pin the exact base image version
# Use slim to minimize image size

WORKDIR /app

# Install dependencies first (for caching)
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application code
COPY src/ ./src/

# Create non-root user
RUN useradd --create-home appuser
USER appuser

# Health check
HEALTHCHECK --interval=30s --timeout=5s \
  CMD curl -f http://localhost:8000/health || exit 1

CMD ["uvicorn", "src.main:app", "--host", "0.0.0.0", "--port", "8000"]
```

</details>

### Part D: Rollback Strategy

Design a rollback strategy that:
1. Takes less than 30 seconds
2. Does not require manual intervention
3. Guarantees the rollback image has compatible dependencies
4. Can be triggered by a health check failure

Write your strategy as a step-by-step procedure with commands.

<details>
<summary>Hint</summary>

With containers, rollback is just running a different image version:

```bash
# Current deployment (broken)
docker run -d --name api myregistry/api:v2.3

# Health check fails after 3 attempts
# Automatic rollback triggered:
docker stop api
docker rm api
docker run -d --name api myregistry/api:v2.2
```

The key: v2.2's Dockerfile pins its own Redis version, so the
"v2.2 requires different Redis version" problem is solved --
each image version bundles its own dependencies.

</details>

### Part E: Write the Post-Mortem Action Items

Based on your containerized design, write a list of action items that
would prevent this outage from ever happening again. For each item,
specify:

- What to implement
- How it prevents the specific failure
- Estimated effort (hours/days)
- Priority (P0/P1/P2)

## Success Criteria

- [ ] Root cause analysis identifies at least 5 distinct failures
- [ ] Each failure has a system-level prevention mechanism
- [ ] Architecture diagram shows the complete deployment pipeline
- [ ] Dockerfile pins Python version and runs as non-root
- [ ] Rollback strategy is automated and takes under 30 seconds
- [ ] Action items are prioritized and have effort estimates
- [ ] The design prevents ALL failures from the post-mortem

## What You Should Understand After This Exercise

Containers solve the "it works on my machine" problem by making the
*image* the single source of truth for the runtime environment.
The same image that passes CI is the same image that runs in production.
Rollback becomes trivial because each image version is self-contained.
This is not just convenient -- it is the foundation of reliable deployments.
