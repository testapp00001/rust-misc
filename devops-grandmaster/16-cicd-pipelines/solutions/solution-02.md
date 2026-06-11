# Solution 02: GitHub Actions Pipeline for Docker

## Part A: Create the Workflow File

```yaml
name: CI Pipeline

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Build Docker image
        uses: docker/build-push-action@v5
        with:
          context: .
          push: false
          tags: myapp:${{ github.sha }}
```

### Why This Works

The workflow triggers on both pushes to `main` and pull requests targeting `main`, ensuring that every change is validated before it reaches the main branch. Docker Buildx is required for advanced features like multi-platform builds and caching. The `push: false` ensures the image is built but not pushed during the build stage -- push happens later after tests pass.

### Common Mistakes

- **Using `push: true` in the build step.** The image should not be pushed until it passes tests. Keep build and push as separate steps.
- **Not setting up Buildx.** Without Buildx, the `docker/build-push-action` will not work, and you lose access to caching and multi-platform builds.
- **Hardcoding the tag to `latest`.** Using `${{ github.sha }}` provides traceability -- you can always identify which commit produced which image.

## Part B: Add the Test Stage

```yaml
jobs:
  build-and-test:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Build Docker image
        uses: docker/build-push-action@v5
        with:
          context: .
          push: false
          load: true
          tags: myapp:test

      - name: Run linting
        run: docker run --rm myapp:test npm run lint

      - name: Run tests
        run: docker run --rm myapp:test npm test
```

### Why This Works

The `load: true` parameter is critical -- it loads the built image into the local Docker daemon so that `docker run` can use it. Without `load: true`, the image exists only in the Buildx cache and is not available for `docker run`. Each `docker run` command executes in a fresh container, ensuring test isolation. If `npm test` or `npm run lint` returns a non-zero exit code, the step fails and the workflow stops.

### Common Mistake

- **Forgetting `load: true`.** Without this, `docker run myapp:test` fails with "image not found" because Buildx builds to a cache by default, not the local daemon.
- **Running tests outside the container.** The whole point of container CI is to test in the same environment that will run in production. Running `npm test` on the runner (outside the container) tests a different environment.

## Part C: Push to GitHub Container Registry

```yaml
jobs:
  build-and-test:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Log in to GitHub Container Registry
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Generate image metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ghcr.io/${{ github.repository }}
          tags: |
            type=sha,prefix=
            type=ref,event=branch

      - name: Build Docker image
        uses: docker/build-push-action@v5
        with:
          context: .
          load: true
          tags: myapp:test

      - name: Run linting
        run: docker run --rm myapp:test npm run lint

      - name: Run tests
        run: docker run --rm myapp:test npm test

      - name: Build and push image
        if: github.event_name == 'push'
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
```

### Why This Works

The `GITHUB_TOKEN` is automatically available in every workflow run -- no manual secret setup required. The `packages: write` permission allows the workflow to push to ghcr.io. The `docker/metadata-action` generates tags automatically: a SHA tag for traceability and a branch name tag for convenience. The `if: github.event_name == 'push'` condition ensures images are only pushed on pushes to `main`, not on pull requests. This prevents unreviewed code from reaching the registry.

### Common Mistakes

- **Pushing on pull requests.** PRs are unmerged code. Pushing images from PRs pollutes the registry with images that may never be merged.
- **Not setting `packages: write` permission.** Without this, the `GITHUB_TOKEN` cannot push to ghcr.io and the push step fails with a 403 error.
- **Using `latest` as the only tag.** Always include the SHA tag. `latest` is mutable and can point to different images over time.

## Part D: Add Caching

```yaml
jobs:
  build-and-test:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Log in to GitHub Container Registry
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Generate image metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ghcr.io/${{ github.repository }}
          tags: |
            type=sha,prefix=
            type=ref,event=branch

      - name: Build Docker image (with cache)
        uses: docker/build-push-action@v5
        with:
          context: .
          load: true
          tags: myapp:test
          cache-from: type=gha
          cache-to: type=gha,mode=max

      - name: Run linting
        run: docker run --rm myapp:test npm run lint

      - name: Run tests
        run: docker run --rm myapp:test npm test

      - name: Build and push image
        if: github.event_name == 'push'
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

### Why This Works

`cache-from: type=gha` tells Buildx to look for cached layers in the GitHub Actions cache. `cache-to: type=gha,mode=max` tells Buildx to store all intermediate layers (not just the final ones). The `mode=max` is important -- without it, only the layers of the final stage are cached, and multi-stage builds lose the benefit of caching earlier stages. On the second run, if `package.json` has not changed, the `npm ci` layer is served from cache, saving minutes of network and build time.

### Common Mistake

- **Using `mode=min` (the default).** This only caches the final stage's layers. For multi-stage Dockerfiles, the builder stage is rebuilt every time, negating much of the caching benefit.
- **Not using `gha` cache type.** Other cache backends (inline, registry) work but require more configuration. The `gha` type uses the GitHub Actions cache automatically with no additional setup.

## Complete Workflow File

For reference, here is the complete workflow combining all parts:

```yaml
name: CI Pipeline

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build-test-push:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Log in to GitHub Container Registry
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Generate image metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ghcr.io/${{ github.repository }}
          tags: |
            type=sha,prefix=
            type=ref,event=branch

      - name: Build Docker image
        uses: docker/build-push-action@v5
        with:
          context: .
          load: true
          tags: myapp:test
          cache-from: type=gha
          cache-to: type=gha,mode=max

      - name: Run linting
        run: docker run --rm myapp:test npm run lint

      - name: Run tests
        run: docker run --rm myapp:test npm test

      - name: Build and push image
        if: github.event_name == 'push'
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

## Key Takeaway

A GitHub Actions workflow for Docker follows a clear sequence: checkout, setup Buildx, build, test, push. The `load: true` flag makes the image available for local testing. The `if:` condition controls when images are pushed. The `gha` cache backend with `mode=max` caches all layers across multi-stage builds. Each step has a specific purpose, and the order ensures that only tested, validated images reach the registry.
