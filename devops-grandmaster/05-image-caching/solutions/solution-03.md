# Solution 03: Optimize a CI Pipeline's Build Time

## Part A: Problem Diagnosis

Five distinct problems:

1. **No BuildKit.** The `docker build` command uses the legacy builder by
   default. BuildKit is faster and supports cache mounts and advanced caching.

2. **No remote cache.** CI runners start with a clean slate. Without
   `--cache-from`, every build downloads all dependencies from scratch.

3. **Bad Dockerfile order.** `COPY . .` before `npm install` means every
   code change reinstalls all 400+ npm packages.

4. **Missing `.dockerignore`.** The build context includes `node_modules/`
   (500MB+), `.git/`, and other unnecessary files. This slows context
   transfer and can cause false cache invalidation.

5. **Using `npm install` instead of `npm ci`.** `npm install` modifies
   `package-lock.json`, making builds non-deterministic. `npm ci` installs
   exactly what the lockfile specifies.

## Part B: Fixed Dockerfile

```dockerfile
# syntax=docker/dockerfile:1

FROM node:20-alpine

WORKDIR /app

# Install dependencies first (changes rarely)
COPY package.json package-lock.json ./
RUN --mount=type=cache,target=/root/.npm \
    npm ci --only=production

# Copy TypeScript config (changes sometimes)
COPY tsconfig.json ./

# Copy source code (changes constantly)
COPY src/ ./src/

# Build TypeScript
RUN npm run build

CMD ["node", "dist/index.js"]
```

Key changes:

- `node:20` → `node:20-alpine` (1GB → 180MB)
- `COPY . .` → split into `COPY package.json package-lock.json` then `COPY src/`
- `npm install` → `npm ci` (deterministic, faster in CI)
- Added `--mount=type=cache,target=/root/.npm` for npm package cache

## Part C: .dockerignore

```
.git
.gitignore
node_modules
dist
coverage
.nyc_output
*.log
.env
.env.local
.DS_Store
.vscode
.idea
*.md
Dockerfile*
.dockerignore
.docker
tests
__tests__
*.test.ts
*.spec.ts
jest.config.*
.eslintrc*
.prettierrc*
tsconfig*.json
!tsconfig.json
```

Impact:

```
Without .dockerignore:
  Build context: 520MB (includes node_modules/, .git/)
  Context transfer time: 15 seconds

With .dockerignore:
  Build context: 2.3MB (only source code and configs)
  Context transfer time: 0.5 seconds
```

## Part D: Fixed CI Pipeline

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
          tags: myregistry.com/user-service:${{ github.sha }}
          cache-from: type=registry,ref=myregistry.com/user-service:buildcache
          cache-to: type=registry,ref=myregistry.com/user-service:buildcache,mode=max
```

Key changes:

1. **`docker/setup-buildx-action@v3`** -- Sets up BuildKit with buildx.
   Without this, `docker/build-push-action` cannot use advanced cache features.

2. **`docker/build-push-action@v5`** -- A single action that handles build,
   push, and cache. Replaces the separate `docker build` and `docker push` steps.

3. **`cache-from: type=registry,ref=...buildcache`** -- Pulls the previous
   build cache from the registry. On the first run, this is a no-op (no cache
   exists yet).

4. **`cache-to: type=registry,ref=...buildcache,mode=max`** -- Pushes the
   build cache to the registry after the build. `mode=max` caches all
   intermediate layers, not just the final image layers.

5. **`docker/login-action@v3`** -- Authenticates to the registry so push works.

## Part E: Estimated Build Times

### Scenario 1: Cold Cache (First Build)

```
FROM node:20-alpine          → Pulled from registry (10s)
COPY package.json ...        → Built (0.1s)
RUN npm ci                   → Downloads all packages (60s)
COPY tsconfig.json           → Built (0.1s)
COPY src/ ./src/             → Built (0.5s)
RUN npm run build            → Compiles TypeScript (15s)
CMD                          → Built (0.1s)
Total: ~90 seconds
```

### Scenario 2: Warm Cache, Source Code Changed

```
FROM node:20-alpine          → CACHED (pulled from remote cache)
COPY package.json ...        → CACHED
RUN npm ci                   → CACHED (package files unchanged)
COPY tsconfig.json           → CACHED
COPY src/ ./src/             → REBUILT (source code changed)
RUN npm run build            → REBUILT (previous layer changed)
CMD                          → CACHED
Total: ~20 seconds
```

### Scenario 3: Warm Cache, package-lock.json Changed

```
FROM node:20-alpine          → CACHED
COPY package.json ...        → REBUILT (package-lock.json changed)
RUN npm ci                   → REBUILT (previous layer changed)
                               But: npm cache mount has previous packages
                               Only new/changed packages are downloaded
COPY tsconfig.json           → REBUILT (cascade)
COPY src/ ./src/             → REBUILT (cascade)
RUN npm run build            → REBUILT (cascade)
CMD                          → CACHED
Total: ~90 seconds (but npm ci is faster due to cache mount)
```

### Scenario 4: Warm Cache, Nothing Changed

```
All layers → CACHED
Total: ~5 seconds (cache lookup only)
```

### Summary Table

| Scenario | Old Pipeline | Optimized Pipeline | Speedup |
|----------|-------------|-------------------|---------|
| Cold cache | 8 min | 90s | 5x |
| Code change | 8 min | 20s | 24x |
| Dep change | 8 min | 90s | 5x |
| No change | 8 min | 5s | 96x |

## Common Mistakes

1. **Forgetting `docker/setup-buildx-action`.** The `docker/build-push-action`
   requires buildx to be set up. Without it, cache features do not work and
   you may get confusing errors.

2. **Using `mode=min` instead of `mode=max`.** `mode=min` only caches the
   final image layers. In a multi-stage build, intermediate stages are not
   cached, so you lose the benefit of caching dependency installation.

3. **Not authenticating before cache operations.** `cache-from` can work
   without authentication for public images, but `cache-to` always requires
   push permissions. Forgetting `docker/login-action` causes silent failures.

4. **Using the same cache tag for all branches.** If `main` and `feature-x`
   share the same cache tag, they can overwrite each other's cache. Use
   branch-specific cache tags for active development branches.
