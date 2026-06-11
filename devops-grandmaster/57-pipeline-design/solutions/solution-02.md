# Solution 02: Build a CI/CD Pipeline from Scratch

## Part A: Build Job

```yaml
build:
  runs-on: ubuntu-latest
  steps:
    - name: Checkout code
      uses: actions/checkout@v4

    - name: Setup Rust toolchain
      uses: actions-rust-lang/setup-rust-toolchain@v1
      with:
        toolchain: stable

    - name: Cache Cargo dependencies
      uses: actions/cache@v4
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
        restore-keys: |
          ${{ runner.os }}-cargo-

    - name: Build
      run: cargo build --release

    - name: Upload binary
      uses: actions/upload-artifact@v4
      with:
        name: binary
        path: target/release/my-service
```

### Why This Works

The cache key uses `Cargo.lock` hash, so the cache is invalidated only when
dependencies change. The `restore-keys` fallback ensures partial cache hits
when only some dependencies change. Uploading the binary as an artifact lets
downstream jobs use it without rebuilding.

## Part B: Test Jobs

```yaml
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
      run: cargo test --test integration_test
      env:
        DATABASE_URL: postgres://postgres:test@localhost:5432/testdb
```

### Why This Works

Both jobs have `needs: build`, so they run after the build succeeds but in
parallel with each other. The PostgreSQL service container uses health checks
to ensure the database is ready before tests run. Each job restores the
Cargo cache independently.

## Part C: Security Scan Job

```yaml
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
```

### Why This Works

`cargo audit` checks the `Cargo.lock` file against the RustSec Advisory
Database, which tracks known vulnerabilities in Rust crate versions. It
runs in parallel with the test jobs because it has no dependency on test
results -- it only needs the source code.

## Part D: Quality Gate and Deploy Jobs

```yaml
quality-gate:
  runs-on: ubuntu-latest
  needs: [unit-tests, integration-tests, dependency-scan]
  steps:
    - name: All checks passed
      run: echo "All quality gates passed -- ready for deployment"

deploy-staging:
  runs-on: ubuntu-latest
  needs: quality-gate
  if: github.ref == 'refs/heads/main'
  environment: staging
  steps:
    - name: Checkout code
      uses: actions/checkout@v4

    - name: Build and push image
      uses: docker/build-push-action@v5
      with:
        context: .
        push: true
        tags: ghcr.io/${{ github.repository }}:${{ github.sha }}
        cache-from: type=gha
        cache-to: type=gha,mode=max

    - name: Deploy to staging
      run: |
        kubectl set image deployment/my-service \
          my-service=ghcr.io/${{ github.repository }}:${{ github.sha }}
        kubectl rollout status deployment/my-service --timeout=120s

deploy-production:
  runs-on: ubuntu-latest
  needs: deploy-staging
  if: github.ref == 'refs/heads/main'
  environment: production
  steps:
    - name: Deploy to production
      run: |
        kubectl set image deployment/my-service \
          my-service=ghcr.io/${{ github.repository }}:${{ github.sha }}
        kubectl rollout status deployment/my-service --timeout=120s
```

### Why This Works

The `quality-gate` job acts as a single checkpoint. Deployment jobs only
run if all tests and scans pass. The `if: github.ref == 'refs/heads/main'`
condition ensures deployment only happens on main branch pushes, not on
pull requests. The `environment` key references GitHub environment
protection rules configured in the repository settings.

For the production environment, you configure `required_reviewers` in the
GitHub repository settings under Settings > Environments > production. This
creates a manual approval gate without any code -- GitHub handles it natively.

## Common Mistakes to Avoid

- **Not caching dependencies.** Without caching, `cargo build` downloads
  and compiles all dependencies from scratch on every run. This adds 3-5
  minutes to every pipeline execution.
- **Running tests sequentially.** Unit tests, integration tests, and
  dependency scans are independent. Running them in parallel cuts feedback
  time by 60-70%.
- **Forgetting the `if` condition on deploy jobs.** Without it, every PR
  would trigger a deployment attempt, which would fail or deploy broken code.
- **Using `latest` instead of SHA for images.** The SHA tag ensures every
  deployment is tied to a specific commit, making rollbacks and audits
  straightforward.

## Key Takeaway

A well-designed pipeline has three properties: parallelism (independent
stages run simultaneously), gating (a quality gate blocks bad code), and
promotion (code moves through environments with increasing confidence).
The `needs` keyword in GitHub Actions is your tool for expressing these
dependencies.
