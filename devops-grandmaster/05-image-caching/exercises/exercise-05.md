# Exercise 05: Caching Strategy for a Multi-Service Application

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Design a complete caching strategy for a multi-service application that
includes Dockerfiles, CI/CD pipelines, and developer workflows. This exercise
combines everything from this module with concepts from
[Module 04: Multi-Stage Builds](../04-multi-stage-builds/).

## Scenario

Your company runs an e-commerce platform with five microservices:

```
ecommerce/
├── services/
│   ├── api-gateway/        # Go -- routes requests to other services
│   ├── product-catalog/    # Python/FastAPI -- product CRUD
│   ├── order-service/      # Node.js/Express -- order processing
│   ├── recommendation-ml/  # Python/PyTorch -- ML recommendations
│   └── notification-svc/   # Go -- email/SMS notifications
├── docker-compose.yml
├── .github/
│   └── workflows/
│       └── build.yml
└── Makefile
```

Current build times:

| Service | Cold Build | Warm Build (code change) | Image Size |
|---------|-----------|-------------------------|------------|
| api-gateway | 45s | 40s | 1.2GB |
| product-catalog | 180s | 175s | 2.8GB |
| order-service | 120s | 115s | 1.5GB |
| recommendation-ml | 600s | 590s | 5.2GB |
| notification-svc | 40s | 35s | 1.1GB |
| **Total** | **985s (~16 min)** | **955s (~16 min)** | **11.8GB** |

Every service has the same problem: `COPY . .` before dependency installation.

## Tasks

### Part A: Audit Each Dockerfile

For each service, identify the specific caching problems in its Dockerfile.
Write the corrected Dockerfile for each service. Each Dockerfile must:

1. Use a slim/alpine base image
2. Separate dependency files from source code
3. Use BuildKit cache mounts for the appropriate package manager
4. Use multi-stage builds to minimize final image size
5. Include a `.dockerignore`

You do not need to write the actual application code -- focus on the Dockerfile.

<details>
<summary>Hint</summary>

For Go services (api-gateway, notification-svc): use `golang:1.21-alpine`
builder stage, then `alpine:3.18` or `scratch` for the final image. Cache
`/go/pkg/mod` and `/root/.cache/go-build`.

For Python services (product-catalog, recommendation-ml): use `python:3.11-slim`,
cache `/root/.cache/pip`. For recommendation-ml, consider that PyTorch is
large -- use a separate stage for ML dependencies.

For Node.js (order-service): use `node:20-alpine`, cache `/root/.npm`. Use
`npm ci` for deterministic builds.

</details>

### Part B: Design the CI Pipeline

Design a GitHub Actions workflow that:

1. Builds all five services
2. Uses remote cache (`--cache-from` / `--cache-to`) for each service
3. Builds services in parallel (not sequentially)
4. Only rebuilds services whose source code changed (path filters)
5. Uses `mode=max` for full intermediate layer caching

Write the complete `.github/workflows/build.yml`.

<details>
<summary>Hint</summary>

Use a matrix strategy to build services in parallel. Use `paths` filters
on each job to skip services that did not change. Use
`docker/build-push-action` with `cache-from` and `cache-to` per service.

</details>

### Part C: Estimate Optimized Build Times

Fill in this table with your estimated build times after optimization:

| Service | Cold Build | Warm Build (code change) | Warm Build (dep change) |
|---------|-----------|-------------------------|------------------------|
| api-gateway | ? | ? | ? |
| product-catalog | ? | ? | ? |
| order-service | ? | ? | ? |
| recommendation-ml | ? | ? | ? |
| notification-svc | ? | ? | ? |

Explain your reasoning for each estimate.

<details>
<summary>Hint</summary>

Cold builds will be similar (maybe slightly faster with slim images and
cache mounts). Warm builds with code changes should drop dramatically --
only the `COPY . .` and build steps rebuild. Warm builds with dependency
changes should be moderate -- the package manager cache mount means only
new packages are downloaded.

</details>

### Part D: Design the Developer Workflow

Developers need fast local builds too. Design a `Makefile` that provides:

1. `make build` -- build all services with caching
2. `make build-<service>` -- build a single service
3. `make rebuild` -- force a full rebuild (no cache)
4. `make rebuild-deps` -- rebuild only dependency layers

Write the complete Makefile.

<details>
<summary>Hint</summary>

Use `DOCKER_BUILDKIT=1` as an environment variable. Use `--cache-from` with
a local tag for incremental builds. Use `--no-cache` for full rebuilds.
Use `--no-cache-filter` for dependency-only rebuilds.

</details>

### Part E: Cache Invalidation Strategy

For each of these scenarios, explain which layers are invalidated and what
the rebuild time impact is:

1. A developer changes one line in `product-catalog/routes.py`
2. The team upgrades Go from 1.21 to 1.22
3. A new package is added to `order-service/package.json`
4. The base image `node:20-alpine` gets a security patch
5. A developer runs `make rebuild-deps` on `recommendation-ml`

<details>
<summary>Hint</summary>

Scenario 1: Only the `COPY` of source code and build steps rebuild (fast).
Scenario 2: The Go base image changes, so all Go service layers rebuild (slow).
Scenario 3: `package-lock.json` changes, so `npm ci` and everything after rebuild (moderate).
Scenario 4: `FROM` layer changes, so everything rebuilds for that service (slow).
Scenario 5: `--no-cache-filter=deps` rebuilds dependency stages only.

</details>

## Success Criteria

- [ ] You have written corrected Dockerfiles for all five services
- [ ] Each Dockerfile uses a slim/alpine base image
- [ ] Each Dockerfile separates dependency installation from code copying
- [ ] Each Dockerfile uses BuildKit cache mounts for the appropriate package manager
- [ ] Each Dockerfile uses multi-stage builds for minimal final images
- [ ] Your CI pipeline builds services in parallel with path-based filters
- [ ] Your CI pipeline uses remote cache with `mode=max`
- [ ] You can estimate build times for cold, warm-code, and warm-dep scenarios
- [ ] You have a Makefile with build, rebuild, and rebuild-deps targets
- [ ] You can predict cache invalidation behavior for each scenario

## What You Should Understand After This Exercise

Caching is not just a Dockerfile concern -- it spans the entire build pipeline.
A complete caching strategy includes: Dockerfile instruction ordering, BuildKit
cache mounts, multi-stage builds, `.dockerignore`, CI remote cache, parallel
builds, and path-based rebuild filtering. Each layer of optimization compounds,
turning a 16-minute build into a 2-minute build.
