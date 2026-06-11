# Solution 04: BuildKit Cache Mounts for a Polyglot Project

## Part A: BuildKit Syntax Directive

```dockerfile
# syntax=docker/dockerfile:1
```

This directive must be the very first line of the Dockerfile (before any
comments or instructions). It tells Docker to use the specified Dockerfile
frontend, which provides the `--mount=type=cache` syntax.

Without this directive, Docker falls back to the legacy frontend (or the
default BuildKit frontend pinned to your Docker version), which may not
support cache mounts. The build will fail with:

```
failed to solve: dockerfile parse error on line X: unknown instruction: RUN--MOUNT
```

## Part B: Go Cache Mounts

```dockerfile
# Gateway
FROM golang:1.21-alpine AS gateway
WORKDIR /app

# Copy dependency files first
COPY gateway/go.mod gateway/go.sum ./

# Download dependencies with module cache mount
RUN --mount=type=cache,target=/go/pkg/mod \
    go mod download

# Copy source code
COPY gateway/ .

# Build with module and build cache mounts
RUN --mount=type=cache,target=/go/pkg/mod \
    --mount=type=cache,target=/root/.cache/go-build \
    go build -o /gateway .
```

Go has two cache directories:

| Cache | Directory | Purpose |
|-------|-----------|---------|
| Module cache | `/go/pkg/mod` | Downloaded Go modules (like `node_modules`) |
| Build cache | `/root/.cache/go-build` | Compiled object files and build intermediates |

The module cache avoids re-downloading packages. The build cache avoids
re-compiling unchanged packages. Together, they can reduce build times
from 45 seconds to 5 seconds on code changes.

## Part C: pip Cache Mount

```dockerfile
# ML Service
FROM python:3.11-slim AS ml
WORKDIR /app

# Copy dependency file first
COPY ml-service/requirements.txt .

# Install with pip cache mount
RUN --mount=type=cache,target=/root/.cache/pip \
    pip install --no-cache-dir -r requirements.txt

# Copy source code
COPY ml-service/ .
```

The `--no-cache-dir` flag tells pip not to store packages inside the layer
itself (at `/root/.cache/pip` inside the layer). Instead, packages are only
stored in the cache mount. This prevents doubling storage -- without it,
packages exist both in the layer and in the cache mount.

## Part D: npm Cache Mount

```dockerfile
# Dashboard
FROM node:20-alpine AS dashboard
WORKDIR /app

# Copy package files first
COPY dashboard/package.json dashboard/package-lock.json ./

# Install with npm cache mount
RUN --mount=type=cache,target=/root/.npm \
    npm ci

# Copy source code
COPY dashboard/ .

# Build TypeScript
RUN npm run build
```

Key details:

- Use `npm ci` instead of `npm install`. `npm ci` is faster in CI, installs
  exactly from the lockfile, and deletes `node_modules` before installing.
- The npm cache directory is `/root/.npm`. This stores downloaded tarballs
  so they do not need to be re-downloaded on the next build.

## Part E: Complete Dockerfile

```dockerfile
# syntax=docker/dockerfile:1

# ============================================================
# Stage 1: Go API Gateway
# ============================================================
FROM golang:1.21-alpine AS gateway
WORKDIR /app

COPY gateway/go.mod gateway/go.sum ./
RUN --mount=type=cache,target=/go/pkg/mod \
    go mod download

COPY gateway/ .
RUN --mount=type=cache,target=/go/pkg/mod \
    --mount=type=cache,target=/root/.cache/go-build \
    go build -o /gateway .

# ============================================================
# Stage 2: Python ML Service
# ============================================================
FROM python:3.11-slim AS ml
WORKDIR /app

COPY ml-service/requirements.txt .
RUN --mount=type=cache,target=/root/.cache/pip \
    pip install --no-cache-dir -r requirements.txt

COPY ml-service/ .

# ============================================================
# Stage 3: Node.js Dashboard
# ============================================================
FROM node:20-alpine AS dashboard
WORKDIR /app

COPY dashboard/package.json dashboard/package-lock.json ./
RUN --mount=type=cache,target=/root/.npm \
    npm ci

COPY dashboard/ .
RUN npm run build

# ============================================================
# Stage 4: Final image
# ============================================================
FROM ubuntu:22.04

# Copy built artifacts from each stage
COPY --from=gateway /gateway /usr/local/bin/gateway
COPY --from=ml /app /opt/ml-service
COPY --from=dashboard /app/dist /opt/dashboard

CMD ["sh", "-c", "echo 'Run services individually'"]
```

Cache behavior by change type:

```
Change gateway/main.go:
  gateway stage: COPY gateway/ → REBUILT, go build → REBUILT
  ml stage: ALL CACHED (independent stage)
  dashboard stage: ALL CACHED (independent stage)

Change ml-service/model.py:
  gateway stage: ALL CACHED
  ml stage: COPY ml-service/ → REBUILT
  dashboard stage: ALL CACHED

Change dashboard/package-lock.json:
  gateway stage: ALL CACHED
  ml stage: ALL CACHED
  dashboard stage: COPY package-lock.json → REBUILT, npm ci → REBUILT
```

Each stage caches independently. A change in one service does not affect
the cache of other services.

## Part F: Verification

```bash
# Step 1: Cold build
time docker build -t platform .

# Step 2: Change a Go source file
echo "// comment" >> gateway/main.go

# Step 3: Warm build (should only rebuild gateway stage)
time docker build -t platform .

# Step 4: See which layers are cached
docker build --progress=plain -t platform . 2>&1 | grep -E "CACHED|DONE"
```

Expected results:

```
Cold build:     ~120 seconds (all stages build from scratch)
Warm build:     ~10 seconds (only gateway stage rebuilds)
```

The `--progress=plain` output will show `CACHED` next to layers that were
skipped, confirming that the cache mounts and layer caching are working.

## Common Mistakes

1. **Mounting the wrong directory for a package manager.** Each package
   manager has a specific cache directory. Mounting `/app` instead of
   `/root/.npm` caches the source code, not the downloaded packages.

2. **Not using `--no-cache-dir` with pip.** Without this flag, pip stores
   packages both in the layer and in the cache mount, doubling storage
   and potentially causing confusion.

3. **Using `npm install` instead of `npm ci`.** `npm install` can modify
   `package-lock.json`, making builds non-deterministic. `npm ci` installs
   exactly what the lockfile specifies and is faster.

4. **Forgetting the `# syntax=docker/dockerfile:1` directive.** Without it,
   the `--mount=type=cache` syntax is not recognized.

5. **Not separating dependency files from source code.** Even with cache
   mounts, if you `COPY . .` before `pip install`, any source code change
   invalidates the pip install layer. The cache mount speeds up the re-run,
   but the layer still re-runs unnecessarily.
