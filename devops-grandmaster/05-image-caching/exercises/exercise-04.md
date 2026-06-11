# Exercise 04: BuildKit Cache Mounts for a Polyglot Project

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Implement BuildKit cache mounts for a project that uses multiple programming
languages. You will configure cache mounts for pip, npm, and Go module caches
in a single multi-stage Dockerfile, ensuring each language's package manager
reuses its cache across builds.

## Scenario

Your team maintains a platform with three components in a single repository:

```
platform/
├── gateway/              # Go API gateway
│   ├── go.mod
│   ├── go.sum
│   └── main.go
├── ml-service/           # Python ML service
│   ├── requirements.txt
│   ├── model.py
│   └── serve.py
├── dashboard/            # Node.js admin dashboard
│   ├── package.json
│   ├── package-lock.json
│   ├── tsconfig.json
│   └── src/
│       └── index.ts
├── Dockerfile            # Single Dockerfile for all services
└── docker-compose.yml
```

The current Dockerfile builds all three services but has no cache mounts:

```dockerfile
# Gateway
FROM golang:1.21 AS gateway
WORKDIR /app
COPY gateway/ .
RUN go mod download
RUN go build -o /gateway .

# ML Service
FROM python:3.11 AS ml
WORKDIR /app
COPY ml-service/ .
RUN pip install -r requirements.txt

# Dashboard
FROM node:20 AS dashboard
WORKDIR /app
COPY dashboard/ .
RUN npm ci
RUN npm run build

# Final image
FROM ubuntu:22.04
COPY --from=gateway /gateway /usr/local/bin/gateway
COPY --from=ml /app /opt/ml-service
COPY --from=dashboard /app/dist /opt/dashboard
CMD ["sh", "-c", "echo 'Run services individually'"]
```

## Tasks

### Part A: Add the BuildKit Syntax Directive

The current Dockerfile has no `# syntax=` directive. Add the correct directive
at the top of the file. Explain why this is required for cache mounts.

<details>
<summary>Hint</summary>

The `# syntax=docker/dockerfile:1` directive tells Docker to use the
latest Dockerfile frontend, which supports `--mount=type=cache`. Without
it, cache mounts are not available.

</details>

### Part B: Add Cache Mounts for Go

Add cache mounts for the Go module download cache and the Go build cache.
Identify the correct target directories for each.

<details>
<summary>Hint</summary>

Go has two caches: the module cache (`/go/pkg/mod`) and the build cache
(`/root/.cache/go-build`). Both should be mounted. The `go mod download`
step uses the module cache. The `go build` step uses both.

</details>

### Part C: Add Cache Mounts for pip

Add a cache mount for pip's download cache. Also fix the Dockerfile to copy
`requirements.txt` before copying the rest of the source code.

<details>
<summary>Hint</summary>

pip stores downloaded packages in `/root/.cache/pip`. Mount this directory
as a cache mount. Also restructure the COPY instructions so that
`requirements.txt` is copied and installed before the rest of the source.

</details>

### Part D: Add Cache Mounts for npm

Add a cache mount for npm's package cache. Also fix the Dockerfile to copy
`package.json` and `package-lock.json` before copying source code.

<details>
<summary>Hint</summary>

npm stores its cache in `/root/.npm`. Mount this directory as a cache mount.
Also use `npm ci` instead of `npm install` for deterministic builds.

</details>

### Part E: Separate Dependency Installation from Code Copying

For each of the three services, restructure the Dockerfile so that:

1. Dependency files are copied first
2. Dependencies are installed with cache mounts
3. Source code is copied after dependency installation

This ensures that a source code change does not trigger dependency reinstallation.

<details>
<summary>Hint</summary>

For Go: copy `go.mod` and `go.sum` first, then `go mod download`, then source.
For Python: copy `requirements.txt` first, then `pip install`, then source.
For Node.js: copy `package.json` and `package-lock.json` first, then `npm ci`,
then source.

</details>

### Part F: Verify Cache Behavior

Describe how you would verify that your cache mounts are working correctly.
What commands would you run? What output would you look for?

<details>
<summary>Hint</summary>

Build once (cold), then change a source file and build again (warm). Use
`time` to measure. Use `docker build --progress=plain` to see which layers
are cached. A warm build with source changes should skip dependency
installation entirely.

</details>

## Success Criteria

- [ ] Your Dockerfile has the `# syntax=docker/dockerfile:1` directive
- [ ] Go module cache is mounted at `/go/pkg/mod`
- [ ] Go build cache is mounted at `/root/.cache/go-build`
- [ ] pip cache is mounted at `/root/.cache/pip`
- [ ] npm cache is mounted at `/root/.npm`
- [ ] Each service copies dependency files before source code
- [ ] Each service installs dependencies before copying source code
- [ ] You can verify that source code changes do not trigger dependency reinstallation

## What You Should Understand After This Exercise

BuildKit cache mounts are per-language. Each package manager stores its cache
in a different directory. By mounting the correct cache directory for each
language, you ensure that package downloads are reused across builds, even
when the dependency file itself changes (e.g., adding one new package to
`requirements.txt` reuses all previously downloaded packages).
