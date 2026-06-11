# Solution 05: Caching Strategy for a Multi-Service Application

## Part A: Corrected Dockerfiles

### api-gateway (Go)

```dockerfile
# syntax=docker/dockerfile:1

FROM golang:1.21-alpine AS builder
WORKDIR /app

COPY go.mod go.sum ./
RUN --mount=type=cache,target=/go/pkg/mod \
    go mod download

COPY . .
RUN --mount=type=cache,target=/go/pkg/mod \
    --mount=type=cache,target=/root/.cache/go-build \
    CGO_ENABLED=0 go build -o /api-gateway .

FROM alpine:3.18
RUN apk --no-cache add ca-certificates
COPY --from=builder /api-gateway /usr/local/bin/api-gateway
EXPOSE 8080
CMD ["api-gateway"]
```

Estimated image size: 1.2GB → 25MB (using alpine final stage instead of
golang base image).

### product-catalog (Python/FastAPI)

```dockerfile
# syntax=docker/dockerfile:1

FROM python:3.11-slim AS builder
WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends gcc libpq-dev && \
    rm -rf /var/lib/apt/lists/*

COPY requirements.txt .
RUN --mount=type=cache,target=/root/.cache/pip \
    pip install --no-cache-dir --prefix=/install -r requirements.txt

FROM python:3.11-slim
WORKDIR /app

COPY --from=builder /install /usr/local
COPY . .

EXPOSE 8000
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8000"]
```

Estimated image size: 2.8GB → 180MB (slim base + multi-stage, no build tools
in final image).

### order-service (Node.js)

```dockerfile
# syntax=docker/dockerfile:1

FROM node:20-alpine AS builder
WORKDIR /app

COPY package.json package-lock.json ./
RUN --mount=type=cache,target=/root/.npm \
    npm ci

COPY . .
RUN npm run build

FROM node:20-alpine
WORKDIR /app

COPY --from=builder /app/dist ./dist
COPY --from=builder /app/node_modules ./node_modules
COPY package.json ./

EXPOSE 3000
CMD ["node", "dist/index.js"]
```

Estimated image size: 1.5GB → 250MB (alpine + only production deps in
final image).

### recommendation-ml (Python/PyTorch)

```dockerfile
# syntax=docker/dockerfile:1

FROM python:3.11-slim AS deps
WORKDIR /app

# PyTorch is large -- install in a separate cached stage
COPY requirements.txt .
RUN --mount=type=cache,target=/root/.cache/pip \
    pip install --no-cache-dir --prefix=/install -r requirements.txt

FROM python:3.11-slim AS model
WORKDIR /app

COPY --from=deps /install /usr/local
COPY . .

# Export model (if needed)
RUN python export_model.py

FROM python:3.11-slim
WORKDIR /app

COPY --from=deps /install /usr/local
COPY --from=model /app/model ./model
COPY serve.py .

EXPOSE 8501
CMD ["python", "serve.py"]
```

Estimated image size: 5.2GB → 2.1GB (PyTorch is inherently large, but
build tools and source code are excluded from final image).

### notification-svc (Go)

```dockerfile
# syntax=docker/dockerfile:1

FROM golang:1.21-alpine AS builder
WORKDIR /app

COPY go.mod go.sum ./
RUN --mount=type=cache,target=/go/pkg/mod \
    go mod download

COPY . .
RUN --mount=type=cache,target=/go/pkg/mod \
    --mount=type=cache,target=/root/.cache/go-build \
    CGO_ENABLED=0 go build -o /notification-svc .

FROM scratch
COPY --from=builder /notification-svc /notification-svc
CMD ["/notification-svc"]
```

Estimated image size: 1.1GB → 12MB (using `scratch` -- no OS at all, just
the static binary).

## Part B: CI Pipeline

```yaml
# .github/workflows/build.yml
name: Build and Push Services

on:
  push:
    branches: [main]

jobs:
  detect-changes:
    runs-on: ubuntu-latest
    outputs:
      api-gateway: ${{ steps.changes.outputs.api-gateway }}
      product-catalog: ${{ steps.changes.outputs.product-catalog }}
      order-service: ${{ steps.changes.outputs.order-service }}
      recommendation-ml: ${{ steps.changes.outputs.recommendation-ml }}
      notification-svc: ${{ steps.changes.outputs.notification-svc }}
    steps:
      - uses: actions/checkout@v4
      - uses: dorny/paths-filter@v2
        id: changes
        with:
          filters: |
            api-gateway:
              - 'services/api-gateway/**'
            product-catalog:
              - 'services/product-catalog/**'
            order-service:
              - 'services/order-service/**'
            recommendation-ml:
              - 'services/recommendation-ml/**'
            notification-svc:
              - 'services/notification-svc/**'

  build-api-gateway:
    needs: detect-changes
    if: needs.detect-changes.outputs.api-gateway == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: docker/setup-buildx-action@v3
      - uses: docker/login-action@v3
        with:
          username: ${{ secrets.DOCKERHUB_USERNAME }}
          password: ${{ secrets.DOCKERHUB_TOKEN }}
      - uses: docker/build-push-action@v5
        with:
          context: services/api-gateway
          push: true
          tags: myregistry.com/api-gateway:${{ github.sha }}
          cache-from: type=registry,ref=myregistry.com/api-gateway:buildcache
          cache-to: type=registry,ref=myregistry.com/api-gateway:buildcache,mode=max

  build-product-catalog:
    needs: detect-changes
    if: needs.detect-changes.outputs.product-catalog == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: docker/setup-buildx-action@v3
      - uses: docker/login-action@v3
        with:
          username: ${{ secrets.DOCKERHUB_USERNAME }}
          password: ${{ secrets.DOCKERHUB_TOKEN }}
      - uses: docker/build-push-action@v5
        with:
          context: services/product-catalog
          push: true
          tags: myregistry.com/product-catalog:${{ github.sha }}
          cache-from: type=registry,ref=myregistry.com/product-catalog:buildcache
          cache-to: type=registry,ref=myregistry.com/product-catalog:buildcache,mode=max

  build-order-service:
    needs: detect-changes
    if: needs.detect-changes.outputs.order-service == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: docker/setup-buildx-action@v3
      - uses: docker/login-action@v3
        with:
          username: ${{ secrets.DOCKERHUB_USERNAME }}
          password: ${{ secrets.DOCKERHUB_TOKEN }}
      - uses: docker/build-push-action@v5
        with:
          context: services/order-service
          push: true
          tags: myregistry.com/order-service:${{ github.sha }}
          cache-from: type=registry,ref=myregistry.com/order-service:buildcache
          cache-to: type=registry,ref=myregistry.com/order-service:buildcache,mode=max

  build-recommendation-ml:
    needs: detect-changes
    if: needs.detect-changes.outputs.recommendation-ml == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: docker/setup-buildx-action@v3
      - uses: docker/login-action@v3
        with:
          username: ${{ secrets.DOCKERHUB_USERNAME }}
          password: ${{ secrets.DOCKERHUB_TOKEN }}
      - uses: docker/build-push-action@v5
        with:
          context: services/recommendation-ml
          push: true
          tags: myregistry.com/recommendation-ml:${{ github.sha }}
          cache-from: type=registry,ref=myregistry.com/recommendation-ml:buildcache
          cache-to: type=registry,ref=myregistry.com/recommendation-ml:buildcache,mode=max

  build-notification-svc:
    needs: detect-changes
    if: needs.detect-changes.outputs.notification-svc == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: docker/setup-buildx-action@v3
      - uses: docker/login-action@v3
        with:
          username: ${{ secrets.DOCKERHUB_USERNAME }}
          password: ${{ secrets.DOCKERHUB_TOKEN }}
      - uses: docker/build-push-action@v5
        with:
          context: services/notification-svc
          push: true
          tags: myregistry.com/notification-svc:${{ github.sha }}
          cache-from: type=registry,ref=myregistry.com/notification-svc:buildcache
          cache-to: type=registry,ref=myregistry.com/notification-svc:buildcache,mode=max
```

Key design decisions:

1. **Path-based filtering.** Each job only runs when files in its service
   directory change. If only `product-catalog` changes, the other four
   services are skipped entirely.

2. **Per-service cache tags.** Each service has its own `buildcache` tag
   in the registry. This prevents cache pollution between services.

3. **Parallel execution.** All five `build-*` jobs run in parallel (they
   only depend on `detect-changes`, not on each other).

4. **`mode=max`**. Caches all intermediate layers, including multi-stage
   build intermediates. This is critical for multi-stage Dockerfiles where
   the dependency stage should be cached independently.

## Part C: Estimated Build Times

| Service | Cold Build | Warm (code change) | Warm (dep change) |
|---------|-----------|-------------------|-------------------|
| api-gateway | 40s | 8s | 25s |
| product-catalog | 120s | 10s | 60s |
| order-service | 90s | 12s | 45s |
| recommendation-ml | 400s | 15s | 200s |
| notification-svc | 35s | 6s | 20s |
| **Total (parallel)** | **400s** | **15s** | **200s** |

Reasoning:

- **Cold builds** are slightly faster due to slim/alpine images and cache
  mounts (pip/npm/go downloads are cached even on first build with warm
  cache mounts).

- **Warm code changes** are dramatically faster because only the `COPY`
  of source code and the build step rebuild. Dependency installation is
  fully cached.

- **Warm dependency changes** are moderately faster because cache mounts
  mean only new/changed packages are downloaded, not all packages.

- **Parallel execution** means total time equals the slowest service, not
  the sum of all services.

## Part D: Makefile

```makefile
# Makefile for ecommerce platform
# Requires: Docker with BuildKit enabled

DOCKER_BUILDKIT=1
export DOCKER_BUILDKIT

REGISTRY=myregistry.com
SERVICES=api-gateway product-catalog order-service recommendation-ml notification-svc

# Build all services with caching
.PHONY: build
build: $(addprefix build-,$(SERVICES))

# Build a single service
# Usage: make build-api-gateway
build-%:
	docker build \
		--cache-from $(REGISTRY)/$*:buildcache \
		--cache-to $(REGISTRY)/$*:buildcache \
		-t $(REGISTRY)/$*:latest \
		services/$*

# Build all services (alias)
.PHONY: build-all
build-all: build

# Force full rebuild (no cache)
.PHONY: rebuild
rebuild: $(addprefix rebuild-,$(SERVICES))

rebuild-%:
	docker build --no-cache \
		-t $(REGISTRY)/$*:latest \
		services/$*

# Rebuild only dependency layers (keep other cache)
.PHONY: rebuild-deps
rebuild-deps: $(addprefix rebuild-deps-,$(SERVICES))

rebuild-deps-%:
	docker build --no-cache-filter=deps \
		--cache-from $(REGISTRY)/$*:buildcache \
		-t $(REGISTRY)/$*:latest \
		services/$*

# Push all images
.PHONY: push
push: $(addprefix push-,$(SERVICES))

push-%:
	docker push $(REGISTRY)/$*:latest

# Show image sizes
.PHONY: sizes
sizes:
	@echo "=== Image Sizes ==="
	@for svc in $(SERVICES); do \
		docker images --format "{{.Repository}}:{{.Tag}}\t{{.Size}}" \
			| grep $$svc:latest || echo "$$svc: not built"; \
	done

# Clean all images
.PHONY: clean
clean: $(addprefix clean-,$(SERVICES))

clean-%:
	docker rmi $(REGISTRY)/$*:latest 2>/dev/null || true
```

Usage:

```bash
# Build everything (cached)
make build

# Build just the Go gateway
make build-api-gateway

# Force full rebuild of everything
make rebuild

# Rebuild only dependency layers (e.g., after adding a new pip package)
make rebuild-deps

# Rebuild only recommendation-ml dependencies
make rebuild-deps-recommendation-ml

# Check image sizes
make sizes
```

## Part E: Cache Invalidation Scenarios

### Scenario 1: Developer changes one line in product-catalog/routes.py

```
api-gateway:        NO BUILD (path filter skips it)
product-catalog:    Rebuilds: COPY . . → build step (10s)
order-service:      NO BUILD (path filter skips it)
recommendation-ml:  NO BUILD (path filter skips it)
notification-svc:   NO BUILD (path filter skips it)
Total CI time:      ~10 seconds
```

### Scenario 2: Team upgrades Go from 1.21 to 1.22

```
api-gateway:        FROM golang:1.22-alpine → ALL layers rebuild (40s)
product-catalog:    NO BUILD (Python, unaffected)
order-service:      NO BUILD (Node.js, unaffected)
recommendation-ml:  NO BUILD (Python, unaffected)
notification-svc:   FROM golang:1.22-alpine → ALL layers rebuild (35s)
Total CI time:      ~40 seconds (parallel: max of 40s, 35s)
```

This is a full cache invalidation for Go services because the base image
changed. Every layer after `FROM` rebuilds. This is expected -- Go version
upgrades are rare (quarterly) and require full rebuilds.

### Scenario 3: New package added to order-service/package-lock.json

```
api-gateway:        NO BUILD
product-catalog:    NO BUILD
order-service:      COPY package-lock.json → npm ci → build (45s)
                    But: npm cache mount has previous packages
                    Only new package is downloaded
recommendation-ml:  NO BUILD
notification-svc:   NO BUILD
Total CI time:      ~45 seconds
```

The cache mount at `/root/.npm` means npm does not re-download previously
fetched packages. Only the new package is downloaded. Without the cache
mount, this would take 90 seconds (full npm ci).

### Scenario 4: Base image node:20-alpine gets a security patch

```
api-gateway:        NO BUILD (Go, unaffected)
product-catalog:    NO BUILD (Python, unaffected)
order-service:      FROM node:20-alpine → ALL layers rebuild (90s)
                    The FROM layer hash changes because the image digest changed
recommendation-ml:  NO BUILD (Python, unaffected)
notification-svc:   NO BUILD (Go, unaffected)
Total CI time:      ~90 seconds
```

This is unavoidable. When the base image changes, every layer that depends
on it must rebuild. The mitigation is that this only happens for security
patches (infrequent) and only affects one service.

### Scenario 5: Developer runs make rebuild-deps on recommendation-ml

```
api-gateway:        NO BUILD
product-catalog:    NO BUILD
order-service:      NO BUILD
recommendation-ml:  --no-cache-filter=deps → rebuilds the deps stage only
                    The model and final stages use cache where possible
notification-svc:   NO BUILD
Total time:         ~200 seconds (PyTorch reinstall, but from pip cache mount)
```

The `--no-cache-filter=deps` flag tells Docker to rebuild only the stage
named `deps`. The `model` and final stages can still use cached layers
if their inputs have not changed. The pip cache mount means PyTorch
packages are not re-downloaded, only reinstalled.

## Common Mistakes

1. **Sharing cache tags across services.** If all services write to the same
   `buildcache` tag, they overwrite each other's cache. Use per-service tags.

2. **Not using `mode=max` in CI.** Without it, intermediate stages are not
   cached. In multi-stage builds, this means the dependency stage rebuilds
   on every CI run even if dependencies have not changed.

3. **Building all services even when only one changed.** Without path filters,
   every push rebuilds all five services. This wastes CI minutes and slows
   down the pipeline.

4. **Using `--no-cache` in CI.** This defeats the purpose of remote cache.
   Use `--no-cache` only for debugging or when you suspect cache corruption.

5. **Not pinning base image digests.** Using `node:20-alpine` without a
   digest means the cache invalidates whenever Alpine releases a new version.
   For maximum cache stability, pin to a specific digest:
   `node:20-alpine@sha256:abc123...`. This trades freshness for stability.
