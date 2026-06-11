# Module 05: Image Caching

## The Problem: Why Do Rebuilds Take Forever?

You have a working multi-stage build. Your image is small. But every time you change a single line of code, Docker rebuilds **everything**. The entire dependencies installation. The entire compilation. Everything.

```
$ docker build -t myapp .
[+] Building 247.3s (12/12) FINISHED
                         ^^^^^^^^
                         4 minutes for a one-line change!
```

This is unacceptable. In development, you need fast feedback loops. In CI/CD, you need fast pipelines. Waiting 4 minutes every time you fix a typo is a waste of everyone's time.

**The root cause:** You don't understand how Docker layer caching works.

## How Docker Layer Caching Works

Every instruction in your Dockerfile creates a **layer**. Docker caches each layer. When you rebuild, Docker checks: "Has anything changed since last time I ran this instruction?"

```
┌─────────────────────────────────────────┐
│ Layer 1: FROM python:3.11-slim          │ ← Always cached (rarely changes)
├─────────────────────────────────────────┤
│ Layer 2: WORKDIR /app                   │ ← Cached if unchanged
├─────────────────────────────────────────┤
│ Layer 3: COPY requirements.txt .        │ ← Cached if requirements.txt unchanged
├─────────────────────────────────────────┤
│ Layer 4: RUN pip install -r req...      │ ← Cached if Layer 3 is cached
├─────────────────────────────────────────┤
│ Layer 5: COPY . .                       │ ← Cached if ALL source files unchanged
├─────────────────────────────────────────┤
│ Layer 6: RUN python setup.py build      │ ← Cached if Layer 5 is cached
└─────────────────────────────────────────┘
```

**The critical rule:** Once a layer changes, **every layer after it is rebuilt from scratch**. This is called **cache invalidation cascading**.

```
Layer 1: FROM python:3.11-slim          → CACHED
Layer 2: WORKDIR /app                   → CACHED
Layer 3: COPY requirements.txt .        → CACHED (requirements.txt didn't change)
Layer 4: RUN pip install -r req...      → CACHED (Layer 3 is cached)
Layer 5: COPY . .                       → INVALIDATED (you changed app.py!)
Layer 6: RUN python setup.py build      → REBUILT (because Layer 5 changed)
```

In this case, only layers 5 and 6 rebuild. That's fast.

But what if you order your Dockerfile poorly?

## The Naive Way: Copy Everything First

Here's how most people write their first Dockerfile:

```dockerfile
# Dockerfile.naive
FROM python:3.11-slim

WORKDIR /app

# Copy EVERYTHING first
COPY . .

# Then install dependencies
RUN pip install -r requirements.txt

# Build
RUN python setup.py build

CMD ["python", "app.py"]
```

**What happens when you change one line in `app.py`:**

```
Layer 1: FROM python:3.11-slim          → CACHED
Layer 2: WORKDIR /app                   → CACHED
Layer 3: COPY . .                       → INVALIDATED (app.py changed!)
Layer 4: RUN pip install -r req...      → REBUILT (Layer 3 changed)
Layer 5: RUN python setup.py build      → REBUILT (Layer 4 changed)
```

**Result:** Every single code change reinstalls all dependencies. If `pip install` takes 2 minutes, you wait 2 minutes every time you touch any file. Even if `requirements.txt` hasn't changed.

**Why this is terrible:**

```bash
# Day 1: Initial build
$ docker build -t myapp .    # 247 seconds

# Day 1: Fix a typo in app.py
$ docker build -t myapp .    # 243 seconds (still slow!)

# Day 1: Fix another typo
$ docker build -t myapp .    # 241 seconds (WHY?!!)
```

Your dependencies haven't changed. Your code is 10 lines. But Docker reinstalls numpy, pandas, tensorflow, and 200 other packages every single time because you copied your code before installing dependencies.

## The Right Way: Order Instructions by Change Frequency

The secret to fast builds: **put things that change rarely at the top, things that change often at the bottom**.

```dockerfile
# Dockerfile.optimized
FROM python:3.11-slim

WORKDIR /app

# 1. System dependencies (change: almost never)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    gcc \
    libffi-dev \
    && rm -rf /var/lib/apt/lists/*

# 2. Python dependencies (change: rarely - only when requirements.txt changes)
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# 3. Application code (change: constantly - every commit)
COPY . .

# 4. Build step (change: when code changes)
RUN python setup.py build

CMD ["python", "app.py"]
```

**What happens when you change `app.py`:**

```
Layer 1: FROM python:3.11-slim          → CACHED
Layer 2: WORKDIR /app                   → CACHED
Layer 3: RUN apt-get update && ...      → CACHED
Layer 4: COPY requirements.txt .        → CACHED (requirements.txt unchanged)
Layer 5: RUN pip install -r req...      → CACHED (Layer 4 is cached!)
Layer 6: COPY . .                       → INVALIDATED (app.py changed)
Layer 7: RUN python setup.py build      → REBUILT (Layer 6 changed)
```

**Result:** Dependencies are cached. Only your code rebuilds. Build time drops from 247 seconds to 12 seconds.

```bash
# First build (cold cache)
$ docker build -t myapp .    # 247 seconds

# After changing app.py (warm cache)
$ docker build -t myapp .    # 12 seconds

# After changing app.py again
$ docker build -t myapp .    # 11 seconds
```

### The Principle: Separate What Changes From What Doesn't

```
Change Frequency (top = rarest, bottom = most frequent):

  ┌─────────────────────────────────────┐
  │  Base image (FROM)                  │  ← Almost never changes
  ├─────────────────────────────────────┤
  │  System packages (apt-get)          │  ← Changes monthly
  ├─────────────────────────────────────┤
  │  Language runtime (node, python)    │  ← Changes quarterly
  ├─────────────────────────────────────┤
  │  Dependency lock files              │  ← Changes weekly
  ├─────────────────────────────────────┤
  │  Dependency installation            │  ← Changes when lock files change
  ├─────────────────────────────────────┤
  │  Application source code            │  ← Changes every commit
  ├─────────────────────────────────────┤
  │  Build artifacts                    │  ← Changes every commit
  └─────────────────────────────────────┘
```

**Order your Dockerfile to match this pyramid.**

## Cache Busting: When You Need to Force a Rebuild

Sometimes you **want** to invalidate the cache. For example, when you want to ensure you get the latest security patches.

### Method 1: The `--no-cache` Flag

```bash
# Rebuild everything from scratch
docker build --no-cache -t myapp .

# Use when: you suspect cache corruption, or after base image updates
```

### Method 2: The `--no-cache-filter` Flag (BuildKit)

```bash
# Only rebuild specific stages
docker build --no-cache-filter=deps -t myapp .

# Useful in multi-stage builds where you want to refresh dependencies
# but keep other cached layers
```

### Method 3: The `ARG` Cache Buster

```dockerfile
# Force a rebuild by changing the ARG value
ARG CACHE_BUST=1
RUN apt-get update && apt-get install -y curl
```

```bash
# Change the ARG to invalidate cache from this point forward
docker build --build-arg CACHE_BUST=2 -t myapp .
```

**Use this sparingly.** If you're cache busting every build, you've defeated the purpose of caching.

### Method 4: The `ADD` URL Trick

```dockerfile
# Docker checks if the URL content has changed
ADD https://example.com/latest-version.txt /tmp/version.txt
RUN apt-get update && apt-get install -y mypackage
```

Docker will re-download the URL and if the content changes, invalidate the cache for all subsequent layers.

**When to use each method:**

| Method | When to Use | Impact |
|--------|-------------|--------|
| `--no-cache` | Debugging, security updates | Full rebuild (slow) |
| `--no-cache-filter` | Refresh dependencies only | Partial rebuild |
| `ARG CACHE_BUST` | Force specific layer invalidation | Targeted rebuild |
| `ADD URL` | Check remote version | Automatic invalidation |

## Separating Dependency Installation From Code Copying

This is the single most impactful caching optimization. Let's see it in action across different languages.

### Python

```dockerfile
# BAD: Copies all code, then installs dependencies
FROM python:3.11-slim
WORKDIR /app
COPY . .
RUN pip install -r requirements.txt
CMD ["python", "app.py"]
```

```dockerfile
# GOOD: Installs dependencies first, then copies code
FROM python:3.11-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY . .
CMD ["python", "app.py"]
```

**Why this works:** `requirements.txt` changes rarely. Your source code changes constantly. By copying `requirements.txt` first and installing dependencies, Docker caches the `pip install` layer. It only re-runs when `requirements.txt` actually changes.

### Node.js

```dockerfile
# BAD
FROM node:18-alpine
WORKDIR /app
COPY . .
RUN npm install
CMD ["node", "server.js"]
```

```dockerfile
# GOOD
FROM node:18-alpine
WORKDIR /app
# Copy package files first
COPY package.json package-lock.json ./
RUN npm ci --only=production
# Then copy source code
COPY . .
CMD ["node", "server.js"]
```

**Pro tip:** Use `npm ci` instead of `npm install`. `npm ci` installs from the lockfile exactly, is faster, and is designed for CI/CD environments.

### Go

```dockerfile
# BAD
FROM golang:1.21-alpine
WORKDIR /app
COPY . .
RUN go mod download
RUN go build -o server .
CMD ["./server"]
```

```dockerfile
# GOOD
FROM golang:1.21-alpine
WORKDIR /app
# Copy go.mod and go.sum first
COPY go.mod go.sum ./
RUN go mod download
# Then copy source code
COPY . .
RUN go build -o server .
CMD ["./server"]
```

**Why this matters for Go:** `go mod download` fetches all dependencies from the internet. If you copy your code first, Docker sees the code changed and re-downloads all dependencies even though `go.mod` hasn't changed.

### Rust

```dockerfile
# BAD
FROM rust:1.75-slim
WORKDIR /app
COPY . .
RUN cargo build --release
CMD ["./target/release/myapp"]
```

```dockerfile
# GOOD (with a trick for dependency caching)
FROM rust:1.75-slim
WORKDIR /app

# Create a dummy project to cache dependencies
RUN cargo init --name myapp
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release && \
    rm -rf src/

# Now copy real source code and rebuild
COPY src/ ./src/
RUN cargo build --release
CMD ["./target/release/myapp"]
```

**Why Rust needs a trick:** Cargo doesn't have a `--only-deps` flag. The workaround is to create a dummy project with the same `Cargo.toml`, build dependencies, then replace the source code. The dependency layer is cached until `Cargo.toml` or `Cargo.lock` changes.

## BuildKit Cache Mounts: The Advanced Technique

BuildKit (Docker's modern builder) supports **cache mounts** that persist build caches across builds. This is a game-changer for package managers.

### Enabling BuildKit

```bash
# Option 1: Set environment variable
export DOCKER_BUILDKIT=1

# Option 2: In Docker daemon config (/etc/docker/daemon.json)
{
  "features": {
    "buildkit": true
  }
}

# Option 3: Docker Desktop (enabled by default)
```

### Cache Mount for pip (Python)

```dockerfile
# syntax=docker/dockerfile:1

FROM python:3.11-slim

WORKDIR /app

# Cache pip packages across builds
RUN --mount=type=cache,target=/root/.cache/pip \
    pip install --no-cache-dir -r requirements.txt

COPY . .

CMD ["python", "app.py"]
```

**How it works:**

```
Build 1 (cold cache):
  pip downloads packages → stores in /root/.cache/pip (cache mount)
  Total time: 120 seconds

Build 2 (requirements.txt unchanged):
  Layer cached normally → skips entirely
  Total time: 2 seconds

Build 3 (requirements.txt changed, added one package):
  pip checks cache mount → finds most packages already cached
  Only downloads the new package
  Total time: 15 seconds (instead of 120 seconds!)
```

### Cache Mount for npm (Node.js)

```dockerfile
# syntax=docker/dockerfile:1

FROM node:18-alpine

WORKDIR /app

COPY package.json package-lock.json ./

# Cache npm packages across builds
RUN --mount=type=cache,target=/root/.npm \
    npm ci --only=production

COPY . .

CMD ["node", "server.js"]
```

### Cache Mount for Go modules

```dockerfile
# syntax=docker/dockerfile:1

FROM golang:1.21-alpine

WORKDIR /app

COPY go.mod go.sum ./

# Cache Go module downloads
RUN --mount=type=cache,target=/go/pkg/mod \
    go mod download

COPY . .

# Cache Go build cache
RUN --mount=type=cache,target=/root/.cache/go-build \
    go build -o server .

CMD ["./server"]
```

### Cache Mount for Cargo (Rust)

```dockerfile
# syntax=docker/dockerfile:1

FROM rust:1.75-slim

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

# Cache Cargo registry and build artifacts
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo init --name myapp && \
    cargo build --release && \
    rm -rf src/

COPY src/ ./src/

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    cp target/release/myapp /usr/local/bin/myapp

CMD ["myapp"]
```

**Why cache mounts are powerful:**

```
Without cache mount:
  Build 1: pip install → downloads 200 packages (120s)
  Build 2: pip install → downloads 200 packages (120s)  ← REDOWNLOADS EVERYTHING
  Build 3: pip install → downloads 200 packages (120s)

With cache mount:
  Build 1: pip install → downloads 200 packages (120s)  ← stores in cache mount
  Build 2: pip install → finds 200 packages in cache (5s)  ← REUSES CACHE
  Build 3: pip install → finds 200 packages in cache (5s)
```

### Cache Mount vs Layer Caching

| Feature | Layer Caching | Cache Mount |
|---------|---------------|-------------|
| Scope | Per-layer | Per-instruction |
| Persistence | Docker build cache | Named volume |
| Granularity | All or nothing | Finer control |
| Use case | Simple caching | Package manager caches |
| Complexity | Low | Medium |

**Use both together.** Layer caching for instruction-level caching, cache mounts for package manager caches.

## Using --cache-from for CI/CD

In CI/CD, each build starts with a fresh environment. There's no local Docker cache. Every build starts from scratch.

### The Problem: CI/CD Builds Are Slow

```
GitHub Actions Runner:
  - Fresh VM every time
  - No Docker cache
  - Builds from scratch every time
  - 10 minute builds become 30 minute builds
```

### The Solution: Remote Cache

```bash
# Build and push cache to registry
docker build \
  --cache-from myregistry.com/myapp:cache \
  --cache-to myregistry.com/myapp:cache \
  -t myapp:latest \
  .

# In CI/CD, pull cache first
docker pull myregistry.com/myapp:cache
docker build \
  --cache-from myregistry.com/myapp:cache \
  -t myapp:latest \
  .
```

### GitHub Actions Example

```yaml
# .github/workflows/build.yml
name: Build and Push

on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Login to DockerHub
        uses: docker/login-action@v3
        with:
          username: ${{ secrets.DOCKERHUB_USERNAME }}
          password: ${{ secrets.DOCKERHUB_TOKEN }}

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: myuser/myapp:latest
          cache-from: type=registry,ref=myuser/myapp:buildcache
          cache-to: type=registry,ref=myuser/myapp:buildcache,mode=max
```

**What `mode=max` does:**

```
mode=min (default):
  - Only caches the final image layers
  - Smaller cache size
  - Less effective for multi-stage builds

mode=max:
  - Caches ALL intermediate layers
  - Larger cache size
  - Better for multi-stage builds
  - Recommended for most cases
```

### GitLab CI Example

```yaml
# .gitlab-ci.yml
build:
  image: docker:24
  services:
    - docker:24-dind
  script:
    - docker login -u $CI_REGISTRY_USER -p $CI_REGISTRY_PASSWORD $CI_REGISTRY
    - docker pull $CI_REGISTRY_IMAGE:cache || true
    - docker build
        --cache-from $CI_REGISTRY_IMAGE:cache
        --cache-to $CI_REGISTRY_IMAGE:cache
        -t $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA
        .
    - docker push $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA
    - docker push $CI_REGISTRY_IMAGE:cache
```

### Cache Invalidation Strategy for CI/CD

```bash
# Strategy 1: Branch-based cache
# Each branch gets its own cache
docker build \
  --cache-from myregistry.com/myapp:cache-${BRANCH_NAME} \
  --cache-to myregistry.com/myapp:cache-${BRANCH_NAME} \
  -t myapp .

# Strategy 2: Tag-based cache
# Use the previous successful build as cache source
docker build \
  --cache-from myregistry.com/myapp:${PREVIOUS_TAG} \
  --cache-to myregistry.com/myapp:cache \
  -t myapp .

# Strategy 3: Always pull latest cache, push new cache
docker build \
  --cache-from myregistry.com/myapp:cache \
  --cache-to myregistry.com/myapp:cache \
  -t myapp .
```

## Cache Invalidation Strategies

Cache invalidation is one of the two hard problems in computer science (along with naming things and off-by-one errors).

### Strategy 1: Dependency Lock Files

```dockerfile
# Use lock files for deterministic caching
COPY package-lock.json .     # NOT package.json
COPY Cargo.lock .            # NOT Cargo.toml
COPY go.sum .                # NOT go.mod
```

**Why:** Lock files only change when dependencies actually change. `package.json` can change for version range updates without actually changing installed packages.

### Strategy 2: Separate Concerns with Multi-Stage Builds

```dockerfile
# Stage 1: Dependencies (cached until requirements.txt changes)
FROM python:3.11-slim AS deps
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Stage 2: Application (cached until source code changes)
FROM deps AS app
WORKDIR /app
COPY . .

# Stage 3: Build (cached until source code changes)
FROM app AS build
RUN python setup.py build

# Stage 4: Production (minimal image)
FROM python:3.11-slim AS production
WORKDIR /app
COPY --from=deps /usr/local/lib/python3.11/site-packages /usr/local/lib/python3.11/site-packages
COPY --from=build /app/dist ./dist
CMD ["python", "dist/app.py"]
```

**Benefits:**

```
Requirements change → rebuilds from Stage 1
Source code change → rebuilds from Stage 2 only
Build config change → rebuilds from Stage 3 only
```

### Strategy 3: Dockerignore Everything

```bash
# .dockerignore
.git
.gitignore
node_modules
*.md
.env
.env.local
.DS_Store
__pycache__
*.pyc
.pytest_cache
.coverage
htmlcov
dist
build
*.egg-info
.cache
.mypy_cache
.ruff_cache
```

**Why:** If you don't ignore these files, Docker's build context includes them. Changing `.git` (every commit) invalidates the `COPY . .` layer even if your code hasn't changed.

### Strategy 4: Build Context Optimization

```bash
# BAD: Large build context sent to Docker daemon
docker build -t myapp .    # Sends 2GB of files

# GOOD: Specific build context
docker build -t myapp -f Dockerfile ./src    # Sends only src/ directory
```

## Practical Examples: Build Time Before/After Caching

### Example 1: Python Data Science Application

**Project structure:**

```
my-datascience-app/
├── Dockerfile
├── requirements.txt
├── setup.py
├── src/
│   ├── __init__.py
│   ├── data_loader.py
│   ├── model.py
│   └── train.py
└── tests/
    └── test_model.py
```

**Before (naive Dockerfile):**

```dockerfile
FROM python:3.11-slim
WORKDIR /app
COPY . .
RUN pip install -r requirements.txt
RUN python setup.py install
CMD ["python", "src/train.py"]
```

**requirements.txt:**

```
numpy==1.24.3
pandas==2.0.3
scikit-learn==1.3.0
tensorflow==2.13.0
matplotlib==3.7.2
seaborn==0.12.2
jupyter==1.0.0
```

**Build times:**

```bash
# First build
$ time docker build -t myapp .
real    4m32.118s

# Change one line in data_loader.py
$ time docker build -t myapp .
real    4m28.943s    # Still 4+ minutes!

# Change one line in data_loader.py again
$ time docker build -t myapp .
real    4m31.207s    # STILL 4+ minutes!
```

**After (optimized Dockerfile):**

```dockerfile
# syntax=docker/dockerfile:1

FROM python:3.11-slim

WORKDIR /app

# Install system dependencies (rarely changes)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install Python dependencies (changes rarely)
COPY requirements.txt .
RUN --mount=type=cache,target=/root/.cache/pip \
    pip install -r requirements.txt

# Install package (changes sometimes)
COPY setup.py .
RUN pip install -e .

# Copy source code (changes constantly)
COPY src/ ./src/

CMD ["python", "src/train.py"]
```

**Build times:**

```bash
# First build (cold cache)
$ time docker build -t myapp .
real    4m35.221s    # Same as before

# Change one line in data_loader.py
$ time docker build -t myapp .
real    0m12.847s    # 12 seconds!

# Change one line in data_loader.py again
$ time docker build -t myapp .
real    0m11.203s    # 11 seconds!
```

**Improvement: 4m32s → 12s (22x faster)**

### Example 2: Node.js Microservice

**Project structure:**

```
user-service/
├── Dockerfile
├── package.json
├── package-lock.json
├── tsconfig.json
├── src/
│   ├── index.ts
│   ├── routes/
│   │   └── users.ts
│   └── models/
│       └── user.ts
└── tests/
    └── users.test.ts
```

**Before (naive):**

```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY . .
RUN npm install
RUN npm run build
CMD ["node", "dist/index.js"]
```

**Build times:**

```bash
# First build
$ time docker build -t user-service .
real    2m15.320s

# Change users.ts
$ time docker build -t user-service .
real    2m12.847s    # Reinstalls all 400+ packages
```

**After (optimized):**

```dockerfile
# syntax=docker/dockerfile:1

FROM node:18-alpine

WORKDIR /app

# Install dependencies (changes rarely)
COPY package.json package-lock.json ./
RUN --mount=type=cache,target=/root/.npm \
    npm ci --only=production

# Copy TypeScript config
COPY tsconfig.json ./

# Copy source code (changes constantly)
COPY src/ ./src/

# Build TypeScript
RUN npm run build

CMD ["node", "dist/index.js"]
```

**Build times:**

```bash
# First build (cold cache)
$ time docker build -t user-service .
real    2m18.543s

# Change users.ts
$ time docker build -t user-service .
real    0m8.321s    # 8 seconds!
```

**Improvement: 2m15s → 8s (17x faster)**

### Example 3: Go API Server

**Before:**

```dockerfile
FROM golang:1.21
WORKDIR /app
COPY . .
RUN go mod download
RUN go build -o server .
CMD ["./server"]
```

**After:**

```dockerfile
# syntax=docker/dockerfile:1

FROM golang:1.21-alpine

WORKDIR /app

# Download dependencies (changes rarely)
COPY go.mod go.sum ./
RUN --mount=type=cache,target=/go/pkg/mod \
    go mod download

# Copy source code (changes constantly)
COPY . .

# Build with cached build artifacts
RUN --mount=type=cache,target=/root/.cache/go-build \
    go build -o server .

# Minimal final image
FROM alpine:3.18
RUN apk --no-cache add ca-certificates
WORKDIR /app
COPY --from=0 /app/server .
CMD ["./server"]
```

**Build times:**

```bash
# Before: Every change = 45 seconds
# After: Code change = 3 seconds
# Improvement: 15x faster
```

## Common Caching Mistakes

### Mistake 1: Copying Everything Before Installing Dependencies

```dockerfile
# BAD
COPY . .
RUN pip install -r requirements.txt

# GOOD
COPY requirements.txt .
RUN pip install -r requirements.txt
COPY . .
```

**Why it's bad:** Any file change invalidates the dependency layer.

### Mistake 2: Using `COPY . .` Too Early

```dockerfile
# BAD
COPY . .
RUN apt-get update && apt-get install -y curl
RUN npm install

# GOOD
RUN apt-get update && apt-get install -y curl
COPY package.json package-lock.json ./
RUN npm install
COPY . .
```

**Why it's bad:** `.dockerignore` might not exclude everything. A change to `.git` invalidates the layer.

### Mistake 3: Not Using `.dockerignore`

```bash
# Without .dockerignore
$ docker build -t myapp .
Sending build context to Docker daemon  2.1GB

# With .dockerignore
$ docker build -t myapp .
Sending build context to Docker daemon  12.3MB
```

**Impact:** Large build contexts slow down builds and can invalidate caches unnecessarily.

### Mistake 4: Combining Commands That Change at Different Rates

```dockerfile
# BAD: One layer for two things that change at different rates
RUN apt-get update && apt-get install -y \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/* \
    && npm install \
    && pip install -r requirements.txt

# GOOD: Separate layers for separate concerns
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

COPY requirements.txt .
RUN pip install -r requirements.txt

COPY package.json package-lock.json ./
RUN npm install
```

**Why it's bad:** If you add a new apt package, you also re-run `npm install` and `pip install`.

### Mistake 5: Using `ADD` Instead of `COPY`

```dockerfile
# BAD: ADD has implicit behaviors
ADD . .

# GOOD: COPY is explicit
COPY . .
```

**Why it's bad:** `ADD` can extract tarballs and fetch URLs, which can cause unexpected cache invalidation.

### Mistake 6: Not Pinning Base Image Versions

```dockerfile
# BAD: Cache invalidates when :latest changes
FROM python:latest

# GOOD: Cache is stable
FROM python:3.11.4-slim-bookworm
```

**Why it's bad:** `latest` changes frequently, invalidating all layers.

### Mistake 7: Running Commands That Always Produce Different Output

```dockerfile
# BAD: Date changes every second
RUN echo "Built at $(date)" > /app/build-info.txt
RUN apt-get update

# GOOD: Use build args for timestamps
ARG BUILD_DATE
RUN echo "Built at ${BUILD_DATE}" > /app/build-info.txt
```

**Why it's bad:** `apt-get update` fetches new package lists every time, even if nothing changed.

### Mistake 8: Cache Mounts Without Proper Targeting

```dockerfile
# BAD: Cache mount on wrong directory
RUN --mount=type=cache,target=/app \
    npm install

# GOOD: Cache mount on npm cache directory
RUN --mount=type=cache,target=/root/.npm \
    npm install
```

**Why it's bad:** Caching `/app` means your source code is cached, not your dependencies.

## Hands-On Lab: Build Time Optimization

Let's practice optimizing a real build. Create this project structure:

```bash
mkdir -p caching-lab/src caching-lab/tests
cd caching-lab
```

### Step 1: Create the Application

**`caching-lab/app.py`:**

```python
from flask import Flask, jsonify
import numpy as np
import pandas as pd

app = Flask(__name__)

@app.route('/')
def hello():
    return jsonify({
        "message": "Hello from cached build!",
        "numpy_version": np.__version__,
        "pandas_version": pd.__version__
    })

@app.route('/health')
def health():
    return jsonify({"status": "ok"})

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

**`caching-lab/requirements.txt`:**

```
flask==3.0.0
numpy==1.24.3
pandas==2.0.3
```

**`caching-lab/tests/test_app.py`:**

```python
from app import app

def test_hello():
    client = app.test_client()
    response = client.get('/')
    assert response.status_code == 200
    assert 'Hello' in response.get_json()['message']
```

### Step 2: Build the Naive Way

**`caching-lab/Dockerfile.naive`:**

```dockerfile
FROM python:3.11-slim

WORKDIR /app

# Copy everything first
COPY . .

# Install dependencies
RUN pip install -r requirements.txt

# Run tests
RUN python -m pytest tests/ -v

CMD ["python", "app.py"]
```

**Build and measure:**

```bash
# First build
time docker build -f Dockerfile.naive -t caching-lab:naive .

# Change app.py (add a comment)
echo "# Changed!" >> app.py

# Second build
time docker build -f Dockerfile.naive -t caching-lab:naive .

# Change app.py again
echo "# Changed again!" >> app.py

# Third build
time docker build -f Dockerfile.naive -t caching-lab:naive .
```

**Expected results:**

```
First build:  ~90 seconds
Second build: ~85 seconds (reinstalls dependencies!)
Third build:  ~85 seconds (reinstalls dependencies AGAIN!)
```

### Step 3: Build the Optimized Way

**`caching-lab/Dockerfile.optimized`:**

```dockerfile
# syntax=docker/dockerfile:1

FROM python:3.11-slim

WORKDIR /app

# Install dependencies first (rarely changes)
COPY requirements.txt .
RUN --mount=type=cache,target=/root/.cache/pip \
    pip install -r requirements.txt

# Copy source code (changes constantly)
COPY app.py .
COPY tests/ ./tests/

# Run tests
RUN python -m pytest tests/ -v

CMD ["python", "app.py"]
```

**Build and measure:**

```bash
# First build (cold cache)
time docker build -f Dockerfile.optimized -t caching-lab:optimized .

# Change app.py
echo "# Changed!" >> app.py

# Second build
time docker build -f Dockerfile.optimized -t caching-lab:optimized .

# Change app.py again
echo "# Changed again!" >> app.py

# Third build
time docker build -f Dockerfile.optimized -t caching-lab:optimized .
```

**Expected results:**

```
First build:  ~90 seconds (same as naive - cold cache)
Second build: ~8 seconds (dependencies cached!)
Third build:  ~7 seconds (dependencies cached!)
```

### Step 4: Add .dockerignore

**`caching-lab/.dockerignore`:**

```
.git
.gitignore
__pycache__
*.pyc
.pytest_cache
.env
.DS_Store
*.md
Dockerfile*
.dockerignore
```

**Build again:**

```bash
# Notice smaller build context
time docker build -f Dockerfile.optimized -t caching-lab:optimized .
```

### Step 5: Multi-Stage Build with Caching

**`caching-lab/Dockerfile.multistage`:**

```dockerfile
# syntax=docker/dockerfile:1

# Stage 1: Dependencies
FROM python:3.11-slim AS deps

WORKDIR /app

COPY requirements.txt .
RUN --mount=type=cache,target=/root/.cache/pip \
    pip install --no-cache-dir -r requirements.txt

# Stage 2: Test
FROM deps AS test

WORKDIR /app

COPY app.py .
COPY tests/ ./tests/

RUN python -m pytest tests/ -v

# Stage 3: Production
FROM python:3.11-slim AS production

WORKDIR /app

# Copy installed packages from deps stage
COPY --from=deps /usr/local/lib/python3.11/site-packages /usr/local/lib/python3.11/site-packages
COPY --from=deps /usr/local/bin /usr/local/bin

# Copy application code
COPY app.py .

# Create non-root user
RUN adduser --disabled-password --gecos '' appuser
USER appuser

EXPOSE 5000

CMD ["python", "app.py"]
```

**Build and measure:**

```bash
# Build with tests
time docker build -f Dockerfile.multistage -t caching-lab:production .

# Verify image size
docker images caching-lab:production

# Run the production image
docker run -p 5000:5000 caching-lab:production
```

### Step 6: Compare Results

```bash
# Run all builds and compare
echo "=== Naive Build ==="
time docker build -f Dockerfile.naive -t caching-lab:naive . 2>&1 | tail -1

echo "=== Optimized Build ==="
time docker build -f Dockerfile.optimized -t caching-lab:optimized . 2>&1 | tail -1

echo "=== Multi-Stage Build ==="
time docker build -f Dockerfile.multistage -t caching-lab:production . 2>&1 | tail -1

# Compare image sizes
echo "=== Image Sizes ==="
docker images | grep caching-lab
```

**Expected output:**

```
=== Naive Build ===
real    1m25.432s

=== Optimized Build ===
real    0m8.234s

=== Multi-Stage Build ===
real    0m9.876s

=== Image Sizes ===
caching-lab:production   latest   abc123   145MB
caching-lab:optimized    latest   def456   912MB
caching-lab:naive        latest   ghi789   912MB
```

### Step 7: CI/CD Simulation

**`caching-lab/Dockerfile.ci`:**

```dockerfile
# syntax=docker/dockerfile:1

FROM python:3.11-slim AS builder

WORKDIR /app

# Use cache mount for pip
RUN --mount=type=cache,target=/root/.cache/pip \
    --mount=type=bind,source=requirements.txt,target=requirements.txt \
    pip install --no-cache-dir -r requirements.txt

# Copy source
COPY . .

# Run tests
RUN python -m pytest tests/ -v -x

# Production image
FROM python:3.11-slim

WORKDIR /app

COPY --from=builder /usr/local/lib/python3.11/site-packages /usr/local/lib/python3.11/site-packages
COPY --from=builder /usr/local/bin /usr/local/bin
COPY app.py .

EXPOSE 5000

CMD ["python", "app.py"]
```

**Simulate CI/CD with cache-from:**

```bash
# First build (simulate CI)
docker build \
  --cache-from caching-lab:cache \
  --cache-to caching-lab:cache \
  -f Dockerfile.ci \
  -t caching-lab:ci .

# Second build (should be fast)
docker build \
  --cache-from caching-lab:cache \
  -f Dockerfile.ci \
  -t caching-lab:ci .
```

## Limitation: Images Are Small and Build Fast, But...

You've mastered image caching. Your builds are fast. Your images are small (thanks to multi-stage builds from Module 04).

**But there's a new problem:**

```
Developer A (laptop):     Built myapp:latest
Developer B (laptop):     Built myapp:latest (different version!)
CI Server:                Built myapp:latest (yet another version!)
Production Server:        Which myapp:latest is running?!
```

You can build images fast, but you can't **share** them efficiently. Every machine builds its own copy. There's no single source of truth.

- How do you share images across your team?
- How do you ensure production runs the same image that passed CI/CD tests?
- How do you store images securely?
- How do you manage image versions and rollbacks?

**Next problem:** You need a centralized place to store and distribute images.

→ **Next module:** [06-image-registries](../06-image-registries/) — Sharing and storing images across teams and servers

## Key Terms

| Term | Definition |
|------|-----------|
| **Layer** | Each instruction in a Dockerfile creates a layer |
| **Cache hit** | Docker reuses a cached layer instead of rebuilding |
| **Cache miss** | Docker must rebuild a layer because something changed |
| **Cache invalidation** | When a layer changes, all subsequent layers rebuild |
| **BuildKit** | Docker's modern build engine with advanced features |
| **Cache mount** | Persistent storage for build caches across builds |
| **--cache-from** | Use a remote image as cache source |
| **--cache-to** | Export build cache to a remote location |
| **Build context** | Files sent to Docker daemon during build |

## Checklist

- [ ] I understand how Docker layer caching works
- [ ] I know that cache invalidation cascades to all subsequent layers
- [ ] I order Dockerfile instructions by change frequency (rarest first)
- [ ] I separate dependency installation from code copying
- [ ] I use .dockerignore to exclude unnecessary files from build context
- [ ] I use BuildKit cache mounts for package manager caches
- [ ] I use --cache-from for CI/CD builds
- [ ] I can measure and compare build times before/after optimization
- [ ] I know common caching mistakes and how to avoid them
- [ ] I understand when to use --no-cache to force a rebuild
