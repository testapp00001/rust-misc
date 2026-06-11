# Cheatsheet: Image Caching

## How Layer Caching Works

```
Dockerfile instruction → Creates a layer → Layer is cached
Rebuild: If input unchanged → Use cache. If changed → Rebuild this + ALL after.
```

## Cache Invalidation Rules

| Change | Layers Affected |
|--------|----------------|
| `COPY` file changed | That `COPY` + everything after |
| `RUN` command changed | That `RUN` + everything after |
| Base image updated | Everything (full rebuild) |
| `ARG` value changed | From that `ARG` onward |

## Optimal Dockerfile Order

```dockerfile
FROM python:3.12-slim       # 1. Base (rarely changes)
WORKDIR /app                # 2. Static instructions first
COPY requirements.txt .     # 3. Dependencies (changes rarely)
RUN pip install -r .        # 4. Install deps (cached if #3 unchanged)
COPY . .                    # 5. Source code (changes often)
CMD ["python", "app.py"]    # 6. Runtime config
```

**Rule: Least-changing → Most-changing**

## BuildKit Cache Features

```bash
# Enable BuildKit
DOCKER_BUILDKIT=1 docker build .

# Cache mount for package managers
RUN --mount=type=cache,target=/root/.cache/pip \
    pip install -r requirements.txt

# Cache mount for apt
RUN --mount=type=cache,target=/var/cache/apt \
    apt-get update && apt-get install -y curl
```

## Cache Mount Syntax

```dockerfile
# pip cache
RUN --mount=type=cache,target=/root/.cache/pip pip install -r requirements.txt

# npm cache
RUN --mount=type=cache,target=/root/.npm npm ci

# Go build cache
RUN --mount=type=cache,target=/root/.cache/go-build go build .

# Go module cache
RUN --mount=type=cache,target=/go/pkg/mod go mod download
```

## Debugging Cache

```bash
# See what's cached/not cached
docker build --progress=plain .

# Force full rebuild
docker build --no-cache .

# Check layer sizes
docker history myapp:latest

# Use dive to inspect layers
dive myapp:latest
```

## CI/CD Cache Strategy

```bash
# Pull previous image for cache
docker pull myapp:latest || true
docker build --cache-from myapp:latest -t myapp:latest .
docker push myapp:latest
```

## Common Pitfalls

| Pitfall | Fix |
|---------|-----|
| `COPY . .` before `RUN pip install` | Copy `requirements.txt` first |
| `RUN apt-get update` in separate layer | Combine: `RUN apt-get update && apt-get install -y ...` |
| Using `ADD` for remote URLs | Use `RUN curl` + `RUN tar` instead |
| Not using `--mount=type=cache` | Add cache mounts for package managers |
