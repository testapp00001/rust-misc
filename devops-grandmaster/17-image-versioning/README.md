# Module 17: Image Versioning — Tag Strategies, Rollback, Pinning

> **Previous Module (16):** CI/CD Pipelines
> **Previous Limitation:** Your pipeline builds and deploys automatically, but it uses `latest` tags, unpinned base images, and has no reliable rollback mechanism.
> **This Module Solves That:** Predictable, traceable, and rollback-safe image versioning.

---

## 1. The Problem

Your CI/CD pipeline pushes `myapp:latest` to the registry. On Monday, `latest` points to version 1.2.3. On Tuesday, a new build pushes and `latest` now points to version 1.2.4. On Wednesday, you discover a bug. You want to roll back to Monday's version.

But which image was Monday's version? `latest` now points to Tuesday's build. The Monday image is still in the registry, but you have no way to identify it. Was it build #47? Commit `a1b2c3d`? You don't know.

**The core problem:** Tags like `latest` are mutable — they point to whatever was last pushed. You need an immutable versioning strategy that lets you answer: "What exact image is running in production right now, and how do I roll back to the previous one?"

---

## 2. The Naive Way

### Using only the `latest` tag

```dockerfile
FROM node:18-alpine
```

```bash
docker build -t myapp:latest .
docker push myapp:latest
```

```yaml
# docker-compose.yml
services:
  app:
    image: myapp:latest
```

### Why it fails

1. **`latest` is not a version.** It is a mutable pointer that changes every time someone pushes. It provides zero information about what is actually deployed.

2. **No rollback path.** If `latest` is broken, what do you roll back to? The previous `latest`? You have to dig through registry history to find it.

3. **No reproducibility.** `docker pull myapp:latest` today and `docker pull myapp:latest` tomorrow may give you completely different images.

4. **Cache confusion.** Docker caches images by tag. If `latest` changes, cached versions may not match what you expect.

5. **Base image drift.** `FROM node:18-alpine` pulls whatever the current `18-alpine` is. A security patch or breaking change can silently alter your base.

### A slightly better but still wrong approach: Using only `build number`

```bash
docker build -t myapp:$BUILD_NUMBER .
docker push myapp:$BUILD_NUMBER
```

This is traceable, but `BUILD_NUMBER` has no semantic meaning. Is build #47 newer than build #46? Probably. But is it a major change, a minor fix, or a patch? You cannot tell without looking it up.

---

## 3. The Right Way

### Semantic versioning for images

Semantic versioning (SemVer) communicates the nature of changes at a glance:

```
vMAJOR.MINOR.PATCH

v1.0.0  — Initial release
v1.0.1  — Patch: bug fix, no new features
v1.1.0  — Minor: new features, backward compatible
v2.0.0  — Major: breaking changes
```

**Tagging with SemVer:**

```bash
# Build and tag with semantic version
docker build -t myapp:v1.2.3 .
docker tag myapp:v1.2.3 myapp:v1.2    # Always point to latest patch
docker tag myapp:v1.2.3 myapp:v1       # Always point to latest minor

docker push myapp:v1.2.3
docker push myapp:v1.2
docker push myapp:v1
```

**Rollback is trivial:**

```bash
# Currently on v1.2.4, rolling back to v1.2.3
docker compose down
sed -i 's/myapp:v1.2.4/myapp:v1.2.3/' docker-compose.yml
docker compose up -d
```

### Git SHA as tags

Every build is tagged with the exact git commit:

```bash
GIT_SHA=$(git rev-parse --short HEAD)
docker build -t myapp:${GIT_SHA} .
docker push myapp:${GIT_SHA}
```

**In CI/CD (GitHub Actions):**

```yaml
- name: Build and push
  uses: docker/build-push-action@v5
  with:
    tags: |
      myapp:${{ github.sha }}
      myapp:${{ github.ref_name }}
```

**Benefits:**
- Every image is traceable to its source code
- No two commits produce the same tag
- Easy to correlate deployments with code changes

### Date-based tags

Useful for nightly builds or scheduled releases:

```bash
DATE_TAG=$(date +%Y%m%d-%H%M%S)
docker build -t myapp:${DATE_TAG} .
docker push myapp:${DATE_TAG}
```

### Environment-specific tags

Tag images with the environment they are approved for:

```bash
# Build once
docker build -t myapp:v1.2.3 .

# Tag for each environment
docker tag myapp:v1.2.3 myapp:v1.2.3-staging
docker tag myapp:v1.2.3 myapp:v1.2.3-production

# Push staging tag first
docker push myapp:v1.2.3-staging

# After approval, push production tag
docker push myapp:v1.2.3-production
```

---

## 4. The Production Way

### Tag immutability

In production, tags should never be overwritten. Once `myapp:v1.2.3` is pushed, it must never change. This guarantees that rollback always works.

**Enforcing immutability in registries:**

AWS ECR:
```bash
# Set image tag immutability
aws ecr put-image-tag-mutability \
  --repository-name myapp \
  --image-tag-mutability IMMUTABLE
```

Docker Hub (via API):
```bash
# Docker Hub does not natively support immutability
# Use a naming convention that discourages reuse
# e.g., always include git SHA: myapp:a1b2c3d-v1.2.3
```

Google Artifact Registry:
```bash
# Tags are immutable by default in Artifact Registry
# A tag can only point to one digest at a time
```

### Rollback by tag

**Strategy 1: Simple tag swap**

```bash
# Current state: myapp:v1.2.4 is running
# Rollback to: myapp:v1.2.3

# Option A: Update compose and redeploy
sed -i 's/myapp:v1.2.4/myapp:v1.2.3/' docker-compose.yml
docker compose up -d

# Option B: Use a variable
export APP_VERSION=v1.2.3
docker compose up -d
```

```yaml
# docker-compose.yml with version variable
services:
  app:
    image: myapp:${APP_VERSION:?APP_VERSION is required}
```

**Strategy 2: Blue-green rollback**

```bash
# Blue is running v1.2.4 (current)
# Deploy green with v1.2.3 (rollback target)
docker compose -f docker-compose.green.yml up -d

# Verify green is healthy
curl -f http://localhost:3001/health

# Switch traffic from blue to green
# Update load balancer / nginx config
nginx -s reload

# Remove blue
docker compose -f docker-compose.blue.yml down
```

**Strategy 3: Automatic rollback in CI/CD**

```yaml
# GitHub Actions with automatic rollback
deploy:
  steps:
    - name: Deploy
      run: |
        docker pull myapp:${{ env.APP_VERSION }}
        docker compose up -d
        sleep 30

    - name: Health check
      id: health
      run: |
        for i in $(seq 1 12); do
          if curl -sf https://example.com/health; then
            echo "healthy=true" >> $GITHUB_OUTPUT
            exit 0
          fi
          sleep 5
        done
        echo "healthy=false" >> $GITHUB_OUTPUT
        exit 1

    - name: Rollback
      if: steps.health.outputs.healthy == 'false'
      env:
        PREVIOUS_VERSION: ${{ vars.LAST_KNOWN_GOOD }}
      run: |
        echo "Rolling back from $APP_VERSION to $PREVIOUS_VERSION"
        export APP_VERSION=$PREVIOUS_VERSION
        docker compose up -d
        exit 1
```

### Pinning base image versions

Never use floating tags like `node:18-alpine`. Always pin to a specific digest:

**Bad:**
```dockerfile
FROM node:18-alpine
```

**Good:**
```dockerfile
FROM node:18.19.0-alpine3.19
```

**Best (immutable):**
```dockerfile
FROM node:18.19.0-alpine3.19@sha256:abc123def456...
```

The digest (`sha256:...`) is the image's content hash. Even if someone re-tags `18.19.0-alpine3.19`, the digest will not match, and Docker will refuse to pull it.

**Finding the digest:**

```bash
# Pull the image
docker pull node:18.19.0-alpine3.19

# Get its digest
docker inspect --format='{{index .RepoDigests 0}}' node:18.19.0-alpine3.19
# Output: node@sha256:abc123def456...

# Use in Dockerfile
FROM node:18.19.0-alpine3.19@sha256:abc123def456789...
```

**Automating base image updates with Dependabot:**

**.github/dependabot.yml:**

```yaml
version: 2
updates:
  - package-ecosystem: "docker"
    directory: "/"
    schedule:
      interval: "weekly"
    reviewers:
      - "your-team"
    labels:
      - "dependencies"
      - "docker"
```

### Multi-architecture tags

For images that run on both x86 and ARM (Apple Silicon, AWS Graviton):

```bash
# Build for multiple architectures
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t myapp:v1.2.3 \
  --push .
```

```yaml
# GitHub Actions for multi-arch
- name: Set up QEMU
  uses: docker/setup-qemu-action@v3

- name: Set up Docker Buildx
  uses: docker/setup-buildx-action@v3

- name: Build and push multi-arch
  uses: docker/build-push-action@v5
  with:
    platforms: linux/amd64,linux/arm64
    push: true
    tags: |
      myapp:v1.2.3
      myapp:latest
```

### Complete tagging strategy

A production tagging strategy combines multiple approaches:

```bash
# Build the image
IMAGE=myapp
VERSION=v1.2.3
GIT_SHA=$(git rev-parse --short HEAD)
BUILD_DATE=$(date +%Y%m%d)
REGISTRY=registry.example.com

docker build -t ${IMAGE} .

# Tag with all strategies
docker tag ${IMAGE} ${REGISTRY}/${IMAGE}:${VERSION}        # Semantic version
docker tag ${IMAGE} ${REGISTRY}/${IMAGE}:${GIT_SHA}        # Git SHA
docker tag ${IMAGE} ${REGISTRY}/${IMAGE}:${VERSION}-${GIT_SHA}  # Combined
docker tag ${IMAGE} ${REGISTRY}/${IMAGE}:${BUILD_DATE}     # Date

# Push all tags
docker push ${REGISTRY}/${IMAGE} --all-tags
```

**What each tag is for:**

| Tag | Purpose | Mutable? |
|-----|---------|----------|
| `v1.2.3` | Human-readable version | No |
| `a1b2c3d` | Trace to source code | No |
| `v1.2.3-a1b2c3d` | Best of both | No |
| `20240115` | Nightly/CI builds | No |
| `latest` | Convenience pointer | Yes (avoid in prod) |

---

## 5. Hands-On Lab

### Lab: Implement a complete image versioning strategy

**Objective:** Build a versioning system that supports semantic versioning, git SHA tagging, rollback, and base image pinning.

#### Step 1: Create the project

```bash
mkdir -p versioning-lab && cd versioning-lab
```

#### Step 2: Initialize git

```bash
git init
```

#### Step 3: Create the application

**server.js:**

```javascript
const express = require('express');
const app = express();

// Read version from build args (baked in at build time)
const APP_VERSION = process.env.APP_VERSION || 'unknown';
const GIT_SHA = process.env.GIT_SHA || 'unknown';
const BUILD_DATE = process.env.BUILD_DATE || 'unknown';

app.get('/', (req, res) => {
  res.json({
    name: 'Versioning Lab',
    version: APP_VERSION,
    gitSha: GIT_SHA,
    buildDate: BUILD_DATE,
    uptime: process.uptime(),
  });
});

app.get('/health', (req, res) => {
  res.json({ status: 'healthy', version: APP_VERSION });
});

const port = process.env.PORT || 3000;
app.listen(port, () => console.log(`v${APP_VERSION} (${GIT_SHA}) on port ${port}`));
```

**package.json:**

```json
{
  "name": "versioning-lab",
  "version": "1.0.0",
  "dependencies": {
    "express": "^4.18.2"
  }
}
```

#### Step 4: Create a versioned Dockerfile

**Dockerfile:**

```dockerfile
# Pin the base image to a specific version and digest
FROM node:18.19.0-alpine3.19

# Build arguments for version metadata
ARG APP_VERSION=unknown
ARG GIT_SHA=unknown
ARG BUILD_DATE=unknown

# Inject as environment variables
ENV APP_VERSION=${APP_VERSION}
ENV GIT_SHA=${GIT_SHA}
ENV BUILD_DATE=${BUILD_DATE}

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY server.js .

# Labels for image metadata
LABEL org.opencontainers.image.version="${APP_VERSION}"
LABEL org.opencontainers.image.revision="${GIT_SHA}"
LABEL org.opencontainers.image.created="${BUILD_DATE}"

USER node
EXPOSE 3000
CMD ["node", "server.js"]
```

#### Step 5: Create a build script

**build.sh:**

```bash
#!/bin/bash
set -e

# Configuration
IMAGE_NAME="myapp"
REGISTRY="${REGISTRY:-localhost:5000}"

# Get version info
APP_VERSION="${1:-$(cat VERSION 2>/dev/null || echo '0.0.0')}"
GIT_SHA=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
BUILD_DATE=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# Full image name
IMAGE="${REGISTRY}/${IMAGE_NAME}"

echo "========================================="
echo "Building ${IMAGE_NAME}"
echo "  Version:   ${APP_VERSION}"
echo "  Git SHA:   ${GIT_SHA}"
echo "  Build Date: ${BUILD_DATE}"
echo "========================================="

# Build with all tags
docker build \
  --build-arg APP_VERSION="${APP_VERSION}" \
  --build-arg GIT_SHA="${GIT_SHA}" \
  --build-arg BUILD_DATE="${BUILD_DATE}" \
  -t "${IMAGE}:${APP_VERSION}" \
  -t "${IMAGE}:${GIT_SHA}" \
  -t "${IMAGE}:${APP_VERSION}-${GIT_SHA}" \
  -t "${IMAGE}:latest" \
  .

# Show all tags
echo ""
echo "Tags created:"
docker images "${IMAGE}" --format "  {{.Tag}}\t{{.ID}}\t{{.CreatedAt}}"

echo ""
echo "To push: docker push ${IMAGE} --all-tags"
```

```bash
chmod +x build.sh
```

#### Step 6: Create a version management script

**version.sh:**

```bash
#!/bin/bash
set -e

ACTION="${1}"
VERSION_FILE="VERSION"

# Read current version
if [ -f "$VERSION_FILE" ]; then
  CURRENT=$(cat "$VERSION_FILE")
else
  CURRENT="0.0.0"
fi

IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT"

case "$ACTION" in
  major)
    MAJOR=$((MAJOR + 1))
    MINOR=0
    PATCH=0
    ;;
  minor)
    MINOR=$((MINOR + 1))
    PATCH=0
    ;;
  patch)
    PATCH=$((PATCH + 1))
    ;;
  show)
    echo "Current version: v${CURRENT}"
    exit 0
    ;;
  set)
    if [ -z "$2" ]; then
      echo "Usage: $0 set <version>"
      exit 1
    fi
    echo "$2" > "$VERSION_FILE"
    echo "Version set to v$2"
    exit 0
    ;;
  *)
    echo "Usage: $0 {major|minor|patch|show|set <version>}"
    exit 1
    ;;
esac

NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}"
echo "$NEW_VERSION" > "$VERSION_FILE"
echo "Version bumped: v${CURRENT} -> v${NEW_VERSION}"
```

```bash
chmod +x version.sh
```

#### Step 7: Create docker-compose files

**docker-compose.yml (base):**

```yaml
version: '3.8'

services:
  app:
    image: ${REGISTRY:-localhost:5000}/myapp:${APP_VERSION:?APP_VERSION is required}
    ports:
      - "${APP_PORT:-3000}:3000"
    environment:
      - NODE_ENV=production
    healthcheck:
      test: ["CMD", "wget", "-qO-", "http://localhost:3000/health"]
      interval: 10s
      timeout: 5s
      retries: 3
```

**docker-compose.staging.yml:**

```yaml
version: '3.8'

services:
  app:
    image: ${REGISTRY:-localhost:5000}/myapp:${APP_VERSION:?APP_VERSION is required}
    ports:
      - "3001:3000"
    environment:
      - NODE_ENV=staging
```

**docker-compose.production.yml:**

```yaml
version: '3.8'

services:
  app:
    image: ${REGISTRY:-localhost:5000}/myapp:${APP_VERSION:?APP_VERSION is required}
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=production
    deploy:
      replicas: 2
      resources:
        limits:
          memory: 256M
          cpus: '0.5'
```

#### Step 8: Create a rollback script

**rollback.sh:**

```bash
#!/bin/bash
set -e

PREVIOUS_VERSION="${1}"
ENVIRONMENT="${2:-production}"

if [ -z "$PREVIOUS_VERSION" ]; then
  echo "Usage: $0 <version> [environment]"
  echo ""
  echo "Available versions:"
  docker images myapp --format "  {{.Tag}}" | grep -v latest | sort -V
  exit 1
fi

IMAGE="${REGISTRY:-localhost:5000}/myapp:${PREVIOUS_VERSION}"

echo "========================================="
echo "ROLLING BACK"
echo "  Environment: ${ENVIRONMENT}"
echo "  Version:     ${PREVIOUS_VERSION}"
echo "  Image:       ${IMAGE}"
echo "========================================="

# Verify image exists
if ! docker image inspect "${IMAGE}" > /dev/null 2>&1; then
  echo "ERROR: Image ${IMAGE} not found locally"
  echo "Pulling from registry..."
  docker pull "${IMAGE}"
fi

# Deploy the rollback
export APP_PORT=$([ "$ENVIRONMENT" = "staging" ] && echo "3001" || echo "3000")
export APP_VERSION="${PREVIOUS_VERSION}"

docker compose -f "docker-compose.${ENVIRONMENT}.yml" up -d

# Health check
echo ""
echo "Waiting for health check..."
for i in $(seq 1 30); do
  if curl -sf "http://localhost:${APP_PORT}/health" > /dev/null 2>&1; then
    echo "SUCCESS: Rolled back to v${PREVIOUS_VERSION}"
    curl -s "http://localhost:${APP_PORT}/" | python3 -m json.tool
    exit 0
  fi
  sleep 2
done

echo "WARNING: Health check did not pass within 60 seconds"
exit 1
```

```bash
chmod +x rollback.sh
```

#### Step 9: Create a GitHub Actions workflow with proper versioning

**.github/workflows/release.yml:**

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  release:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write

    steps:
      - uses: actions/checkout@v4

      - name: Extract version
        id: version
        run: |
          VERSION=${GITHUB_REF#refs/tags/}
          echo "version=$VERSION" >> $GITHUB_OUTPUT
          echo "sha=$(git rev-parse --short HEAD)" >> $GITHUB_OUTPUT

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Log in to Container Registry
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          platforms: linux/amd64,linux/arm64
          build-args: |
            APP_VERSION=${{ steps.version.outputs.version }}
            GIT_SHA=${{ steps.version.outputs.sha }}
            BUILD_DATE=${{ github.event.head_commit.timestamp }}
          tags: |
            ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ steps.version.outputs.version }}
            ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ steps.version.outputs.sha }}
            ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:latest
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

#### Step 10: Test the versioning system

```bash
# Initialize the version
./version.sh set 1.0.0

# Build version 1.0.0
./build.sh 1.0.0

# Bump to 1.0.1 and build
./version.sh patch
./build.sh

# Bump to 1.1.0 and build
./version.sh minor
./build.sh

# Bump to 2.0.0 and build
./version.sh major
./build.sh

# List all images
docker images myapp

# Test each version
APP_VERSION=1.0.0 docker compose up -d
curl http://localhost:3000/
docker compose down

APP_VERSION=1.1.0 docker compose up -d
curl http://localhost:3000/
docker compose down

# Simulate rollback
APP_VERSION=2.0.0 docker compose up -d
curl http://localhost:3000/  # "Oh no, v2.0.0 has a bug!"
./rollback.sh 1.1.0          # Roll back to v1.1.0
curl http://localhost:3000/  # Back to the working version
```

#### Step 11: Verify immutability

```bash
# Build v1.0.0 with a specific code
./build.sh 1.0.0
docker inspect myapp:1.0.0 | grep -i version

# Modify the code
echo "// bug fix" >> server.js

# Build v1.0.0 again (overwrites the tag!)
./build.sh 1.0.0
docker inspect myapp:1.0.0 | grep -i version

# The tags point to different images now!
# This is why production registries should enforce tag immutability
```

#### Cleanup

```bash
docker compose down -v
docker images myapp -q | xargs docker rmi -f
```

---

## 6. Limitation

You now have a solid versioning strategy. Every image is tagged with its version, git SHA, and build date. You can roll back to any previous version in seconds.

But you are running a single instance of your application. What happens when:

- Traffic spikes and one container cannot handle the load?
- You need zero-downtime deployments?
- Your single container crashes and users get errors while it restarts?
- You want to run closer to users in multiple regions?

Running a single container is fine for development, but production requires running multiple instances. Scaling from 1 to N containers introduces new challenges: load balancing, session management, data consistency, and service discovery.

---

## 7. Next Topic

**Module 18: Horizontal vs. Vertical Scaling** — We will learn when to scale up (bigger machines) versus scale out (more machines), how to run multiple container instances behind a load balancer, and the architectural patterns that make scaling possible.
