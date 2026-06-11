# Exercise 02: Build a CI/CD Pipeline from Scratch

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Write a complete GitHub Actions CI/CD pipeline for a Rust web service that
includes build, test, security scan, and deployment stages. This exercise
walks you through each stage so you understand the mechanics before designing
pipelines independently.

## Scenario

You have a Rust web service with the following structure:

```
my-service/
  Cargo.toml
  src/
    main.rs
    lib.rs
  tests/
    integration_test.rs
  Dockerfile
```

The team uses GitHub Container Registry (ghcr.io) for images and deploys to
a Kubernetes cluster. You need a pipeline that:

1. Builds the Rust project on every push and PR
2. Runs unit and integration tests in parallel
3. Scans for dependency vulnerabilities
4. Builds and pushes a Docker image (only on main branch)
5. Deploys to staging (only on main branch)
6. Waits for manual approval before production deployment

## Tasks

### Part A: Write the Build Job

Create the `build` job that compiles the Rust project and uploads the binary
as an artifact. The job should cache Cargo dependencies for faster builds.

<details>
<summary>Hint</summary>

Use `actions/cache` with a key based on `Cargo.lock`. The cache path should
include `~/.cargo/registry` and `~/.cargo/git` and `target/`.

```yaml
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

</details>

### Part B: Write the Test Jobs

Create two parallel test jobs: `unit-tests` and `integration-tests`. The
integration tests need a PostgreSQL service container.

<details>
<summary>Hint</summary>

For the integration test job, use the `services` key to start PostgreSQL:

```yaml
services:
  postgres:
    image: postgres:16
    env:
      POSTGRES_PASSWORD: test
    ports:
      - 5432:5432
    options: >-
      --health-cmd pg_isready
      --health-interval 10s
      --health-timeout 5s
      --health-retries 5
```

Both test jobs should have `needs: build` so they run after the build succeeds.

</details>

### Part C: Write the Security Scan Job

Add a `dependency-scan` job that checks for known vulnerabilities in Rust
dependencies. This should run in parallel with the test jobs.

<details>
<summary>Hint</summary>

Use `cargo-audit` to scan dependencies:

```yaml
- name: Install cargo-audit
  run: cargo install cargo-audit
- name: Run audit
  run: cargo audit
```

This job also needs `needs: build` since it shares the build cache.

</details>

### Part D: Write the Quality Gate and Deploy Jobs

Add a `quality-gate` job that requires all test and scan jobs to pass. Then
add `deploy-staging` and `deploy-production` jobs. Production should require
manual approval via a GitHub environment protection rule.

<details>
<summary>Hint 1</summary>

The quality gate uses `needs` to list all jobs that must succeed:

```yaml
quality-gate:
  runs-on: ubuntu-latest
  needs: [unit-tests, integration-tests, dependency-scan]
  steps:
    - name: All checks passed
      run: echo "Ready for deployment"
```

</details>

<details>
<summary>Hint 2</summary>

For manual approval, use GitHub's environment protection rules. Define a
`production` environment in your repository settings that requires reviewer
approval. Then reference it in the job:

```yaml
deploy-production:
  runs-on: ubuntu-latest
  needs: deploy-staging
  environment: production
  steps:
    - name: Deploy
      run: kubectl apply -f k8s/
```

</details>

## Success Criteria

- [ ] Build job compiles Rust and caches dependencies
- [ ] Unit and integration test jobs run in parallel after build
- [ ] Integration test job has a PostgreSQL service container
- [ ] Dependency scan job runs in parallel with tests
- [ ] Quality gate blocks deployment unless all checks pass
- [ ] Staging deployment runs automatically on main branch pushes
- [ ] Production deployment requires manual approval

## What You Should Understand After This Exercise

A well-designed pipeline has independent stages running in parallel to save
time, a quality gate that acts as a single checkpoint, and environment-based
promotion that separates "can we deploy?" from "should we deploy?"
