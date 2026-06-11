# Exercise 03: Optimize a CI Pipeline's Build Time

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective

Optimize a CI/CD pipeline that builds Docker images from scratch on every run.
You will apply layer caching, remote cache, and build context optimization to
reduce build times from 8 minutes to under 2 minutes.

## Scenario

Your team runs a GitHub Actions pipeline that builds and pushes a Node.js
microservice on every push to `main`. The current pipeline is slow:

```yaml
# .github/workflows/build.yml (CURRENT -- SLOW)
name: Build and Push

on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build image
        run: docker build -t myregistry.com/user-service:${{ github.sha }} .

      - name: Push image
        run: docker push myregistry.com/user-service:${{ github.sha }}
```

The current Dockerfile:

```dockerfile
FROM node:20

WORKDIR /app

COPY . .

RUN npm install
RUN npm run build

CMD ["node", "dist/index.js"]
```

The `.dockerignore` does not exist. The build takes **8 minutes** on every run.

## Tasks

### Part A: Diagnose the Problems

List every reason this pipeline is slow. Consider:

1. The Dockerfile itself
2. The CI workflow configuration
3. The build context
4. Cache strategy (or lack thereof)

<details>
<summary>Hint</summary>

There are at least 5 distinct problems. Think about: Dockerfile instruction
order, missing `.dockerignore`, missing BuildKit, missing remote cache,
base image size, and npm install behavior.

</details>

### Part B: Fix the Dockerfile

Rewrite the Dockerfile with proper caching. Your Dockerfile must:

1. Use a slim base image
2. Separate dependency installation from code copying
3. Use BuildKit cache mounts for npm
4. Include the `# syntax=docker/dockerfile:1` directive

<details>
<summary>Hint</summary>

Copy `package.json` and `package-lock.json` first, run `npm ci`, then copy
source code. Use `npm ci` instead of `npm install` for deterministic CI builds.

</details>

### Part C: Create the .dockerignore

Create a `.dockerignore` that reduces the build context from 500MB to under 5MB.

<details>
<summary>Hint</summary>

A typical Node.js project has `node_modules/`, `.git/`, test output, coverage
reports, and documentation that are not needed in the container.

</details>

### Part D: Fix the CI Pipeline

Rewrite the GitHub Actions workflow to:

1. Enable BuildKit
2. Use `docker buildx` for cache-aware builds
3. Use `--cache-from` and `--cache-to` with the registry
4. Use `mode=max` for full intermediate layer caching

Your workflow should pull the previous cache before building and push the
updated cache after building.

<details>
<summary>Hint</summary>

Use the `docker/setup-buildx-action` and `docker/build-push-action` GitHub
Actions. They handle BuildKit setup, cache configuration, and push in a
single step.

</details>

### Part E: Estimate the Improvement

After all optimizations, estimate the build time for these scenarios:

1. Cold cache (first build, no previous cache exists)
2. Warm cache, source code changed
3. Warm cache, only `package-lock.json` changed
4. Warm cache, nothing changed

Explain which layers are cached in each scenario.

<details>
<summary>Hint</summary>

Cold cache: full build (2-3 min). Warm cache with code change: only the
`COPY . .` and `npm run build` layers rebuild (15-30s). Warm cache with
dependency change: `npm ci` layer rebuilds (1-2 min). Nothing changed:
all layers cached (seconds).

</details>

## Success Criteria

- [ ] Your Dockerfile uses a slim base image and separates deps from code
- [ ] Your Dockerfile uses BuildKit cache mounts for npm
- [ ] Your `.dockerignore` excludes `node_modules/`, `.git/`, and other large directories
- [ ] Your CI workflow uses `docker/build-push-action` with `--cache-from` and `--cache-to`
- [ ] Your CI workflow uses `mode=max` for full layer caching
- [ ] You can estimate build times for all four scenarios above
- [ ] You understand why CI builds are different from local builds

## What You Should Understand After This Exercise

CI/CD environments start with a clean slate on every run. Without explicit
cache configuration, every build is a cold build. Remote cache (`--cache-from`
/ `--cache-to`) bridges this gap by storing build cache in a container registry,
making CI builds nearly as fast as local builds.
