# Solution 05: End-to-End Pipeline with Environment Promotion

## Part A: Pipeline Architecture

```
  push to develop                    push to main
       |                                  |
       v                                  v
  ┌─────────┐                       ┌─────────┐
  │  Build   │                       │  Build   │
  └────┬─────┘                       └────┬─────┘
       |                                  |
       v                                  ├────────────┬────────────┬────────────┐
  ┌──────────┐                            v            v            v            v
  │Unit Tests│                       ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
  └────┬─────┘                       │Unit Tests│ │Integra-  │ │Dep. Scan │ │Container │
       |                             └────┬─────┘ │tion Tests│ └────┬─────┘ │  Scan    │
       v                                  |        └────┬─────┘      |        └────┬─────┘
  ┌──────────┐                            |             |            |             |
  │ Deploy   │                            └─────────────┴────────────┴─────────────┘
  │   Dev    │                                          |
  └──────────┘                                          v
                                                 ┌──────────────┐
                                                 │ Quality Gate  │
                                                 └──────┬───────┘
                                                        |
                                                        v
                                                 ┌──────────────┐
                                                 │   Deploy     │
                                                 │  Staging     │
                                                 └──────┬───────┘
                                                        |
                                                        v
                                                 ┌──────────────┐
                                                 │ Smoke Tests  │
                                                 └──────┬───────┘
                                                        |
                                                        v
                                                 ┌──────────────┐
                                                 │   Manual     │
                                                 │  Approval    │
                                                 └──────┬───────┘
                                                        |
                                                        v
                                                 ┌──────────────┐
                                                 │   Deploy     │
                                                 │ Production   │
                                                 └──────┬───────┘
                                                        |
                                                        v
                                                 ┌──────────────┐
                                                 │  Health      │── failure --> Rollback
                                                 │  Check       │
                                                 └──────────────┘
```

## Part B: Build and Push Stage

```yaml
name: CI/CD Pipeline

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  build:
    runs-on: ubuntu-latest
    outputs:
      image-tag: ${{ steps.meta.outputs.version }}
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

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
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=sha,prefix=
            type=ref,event=branch

      - name: Build and push
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

`docker/metadata-action` automatically generates tags: a SHA tag for
traceability and a branch tag for environment targeting. GitHub Actions
cache (`type=gha`) stores Docker layers in the Actions cache, so unchanged
layers are reused across builds. The `outputs.image-tag` lets downstream
jobs reference the exact image that was built.

## Part C: Test and Scan Stage

```yaml
  unit-tests:
    runs-on: ubuntu-latest
    needs: build
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Cache
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
          POSTGRES_DB: checkout_test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
      redis:
        image: redis:7
        ports:
          - 6379:6379
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Cache
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
          DATABASE_URL: postgres://postgres:test@localhost:5432/checkout_test
          REDIS_URL: redis://localhost:6379

  dependency-scan:
    runs-on: ubuntu-latest
    needs: build
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Install cargo-audit
        run: cargo install cargo-audit
      - name: Run dependency audit
        run: cargo audit

  container-scan:
    runs-on: ubuntu-latest
    needs: build
    steps:
      - name: Scan container image
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ github.sha }}
          format: table
          exit-code: 1
          severity: CRITICAL,HIGH
```

### Why This Works

All four jobs run in parallel because they only depend on `build`, not on
each other. The integration test job uses service containers for both
PostgreSQL and Redis with health checks to ensure they are ready before
tests run. Trivy scans the pushed image for OS and library vulnerabilities.

## Part D: Deployment Stage

```yaml
  quality-gate:
    runs-on: ubuntu-latest
    needs: [unit-tests, integration-tests, dependency-scan, container-scan]
    steps:
      - name: All checks passed
        run: echo "Quality gate passed -- ready for deployment"

  deploy-staging:
    runs-on: ubuntu-latest
    needs: quality-gate
    if: github.ref == 'refs/heads/main'
    environment: staging
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup kubeconfig
        run: |
          mkdir -p ~/.kube
          echo "${{ secrets.STAGING_KUBECONFIG }}" | base64 -d > ~/.kube/config

      - name: Deploy to staging
        run: |
          kubectl set image deployment/checkout-service \
            checkout-service=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ github.sha }}
          kubectl rollout status deployment/checkout-service --timeout=180s

      - name: Smoke test staging
        run: |
          STAGING_URL="https://staging.checkout.example.com"

          # Health check
          echo "Checking health endpoint..."
          for i in $(seq 1 12); do
            HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$STAGING_URL/health")
            if [ "$HTTP_CODE" = "200" ]; then
              echo "Health check passed"
              break
            fi
            if [ "$i" -eq 12 ]; then
              echo "Health check failed after 12 attempts"
              exit 1
            fi
            echo "Attempt $i/12 - got HTTP $HTTP_CODE, retrying..."
            sleep 10
          done

          # API contract test
          echo "Running API contract test..."
          RESPONSE=$(curl -sf "$STAGING_URL/api/v1/checkout/validate" \
            -H "Content-Type: application/json" \
            -d '{"items": [{"sku": "TEST-001", "quantity": 1}]}')
          echo "Response: $RESPONSE"

      - name: Rollback staging on failure
        if: failure()
        run: |
          kubectl rollout undo deployment/checkout-service
          kubectl rollout status deployment/checkout-service --timeout=180s

  deploy-production:
    runs-on: ubuntu-latest
    needs: deploy-staging
    if: github.ref == 'refs/heads/main'
    environment: production
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup kubeconfig
        run: |
          mkdir -p ~/.kube
          echo "${{ secrets.PRODUCTION_KUBECONFIG }}" | base64 -d > ~/.kube/config

      - name: Capture current image
        id: current
        run: |
          CURRENT=$(kubectl get deployment/checkout-service \
            -o jsonpath='{.spec.template.spec.containers[0].image}')
          echo "image=$CURRENT" >> "$GITHUB_OUTPUT"
          echo "Current image: $CURRENT"

      - name: Deploy to production
        run: |
          kubectl set image deployment/checkout-service \
            checkout-service=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ github.sha }}
          kubectl rollout status deployment/checkout-service --timeout=300s

      - name: Production health check
        id: health
        run: |
          PROD_URL="https://checkout.example.com"
          FAILURES=0

          for i in $(seq 1 12); do
            HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$PROD_URL/health")
            if [ "$HTTP_CODE" = "200" ]; then
              echo "Production health check passed"
              exit 0
            fi
            FAILURES=$((FAILURES + 1))
            echo "Attempt $i/12 - got HTTP $HTTP_CODE"
            sleep 10
          done

          echo "Production health check failed after 12 attempts"
          exit 1

      - name: Rollback production on failure
        if: failure()
        run: |
          echo "Rolling back to: ${{ steps.current.outputs.image }}"
          kubectl set image deployment/checkout-service \
            checkout-service=${{ steps.current.outputs.image }}
          kubectl rollout status deployment/checkout-service --timeout=180s
          echo "Rollback complete"
```

### Why This Works

The deployment follows the environment promotion model:
1. **Staging** deploys automatically and runs smoke tests
2. **Production** requires manual approval via GitHub environment protection
   rules (configured with `required_reviewers` in the repository settings)
3. Both environments capture the current image before deploying, enabling
   automatic rollback if health checks fail
4. Health checks use a retry loop with 10-second intervals to handle
   slow startup times

The `if: failure()` condition on rollback steps ensures they only run
when the deployment or health check fails, not on successful deployments.

## Common Mistakes to Avoid

- **Not capturing the current image before deploying.** Without this,
  rollback requires knowing the previous image tag, which is not always
  available after a failed deployment.
- **Health checks that only check pod status.** Kubernetes reports pods
  as "Running" even if the application inside is crashing. Always curl
  the application's health endpoint.
- **Too-short timeouts for health checks.** Applications with database
  migrations or cold caches may take 30-60 seconds to become healthy.
  Use retry loops with generous timeouts.
- **Not using GitHub environments for approval gates.** Implementing
  manual approval in the workflow file (e.g., with `concurrency` or
  custom scripts) is fragile. GitHub environments handle this natively
  and integrate with the PR review UI.

## Key Takeaway

Environment promotion is the backbone of safe deployment. Each environment
serves a different purpose: dev validates compilation, staging validates
integration, and production validates real-world health. The gates between
environments (quality gate, smoke tests, manual approval) are what prevent
bad code from reaching users. Automatic rollback ensures that even if a
bad deployment slips through, recovery is measured in seconds, not hours.
