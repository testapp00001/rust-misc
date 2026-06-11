# Module 57: Pipeline Design — Build, Test, Stage, Deploy Patterns

> **Previous Module (56):** Zero Trust Architecture
> **Phase:** CI/CD Mastery
> **Estimated Time:** 3-4 hours

---

## The Problem

You have a codebase. Developers push commits. Now you need to answer critical questions:

- Does the code compile?
- Do the tests pass?
- Is the code secure?
- Does it work in a production-like environment?
- Can we ship it to users?

Manual deployment is the enemy of velocity. A developer pushes code, then someone SSHs into a server, pulls the latest code, runs build commands, restarts services, and hopes nothing breaks. This process is slow, error-prone, and doesn't scale. When 10 developers are pushing code multiple times per day, manual deployment becomes a bottleneck that grinds the team to a halt.

**The core tension:** You want to ship fast, but you also want to ship safely. CI/CD pipelines resolve this tension by automating the path from code to production.

---

## The Naive Way

### Manual Deployment Script

```bash
#!/bin/bash
# deploy.sh — the "works on my machine" deployment

ssh production-server << 'EOF'
cd /opt/myapp
git pull origin main
cargo build --release
systemctl restart myapp
echo "Deployed!"
EOF
```

**Problems:**
- No testing before deployment
- No build artifact — builds happen on the server
- No rollback strategy
- No environment parity
- Single developer can break production
- No audit trail
- Downtime during build and restart

### Slightly Better: Script with Tests

```bash
#!/bin/bash
# deploy-with-tests.sh

cargo test || exit 1
cargo build --release
scp target/release/myapp production-server:/opt/myapp/
ssh production-server "systemctl restart myapp"
```

**Still problematic:**
- Runs on a developer machine (not reproducible)
- No parallelism
- No staging environment
- No security scanning
- No artifact versioning
- Build environment varies per developer

---

## The Right Way

### Understanding CI/CD

**Continuous Integration (CI):** Automatically build and test every code change. The moment a developer pushes, the pipeline validates the change.

**Continuous Delivery (CD):** Automatically prepare releases for deployment. Every passing build is a potential release candidate.

**Continuous Deployment (CD):** Automatically deploy every passing build to production. No human gate.

```
Code Push → Build → Test → Scan → Package → Stage → Deploy
   CI                              CD                    CD
```

### Pipeline Stages

#### Stage 1: Build

```yaml
# Compile the application, resolve dependencies
build:
  steps:
    - checkout
    - install-dependencies
    - compile
    - upload-artifact
```

#### Stage 2: Test

```yaml
# Validate correctness at multiple levels
test:
  steps:
    - unit-tests
    - integration-tests
    - code-coverage-report
```

#### Stage 3: Scan

```yaml
# Security and quality gates
scan:
  steps:
    - dependency-vulnerability-scan
    - static-code-analysis
    - container-image-scan
    - license-compliance-check
```

#### Stage 4: Push

```yaml
# Store versioned artifacts
push:
  steps:
    - tag-image-with-sha
    - push-to-container-registry
    - sign-artifact
```

#### Stage 5: Deploy

```yaml
# Promote through environments
deploy:
  steps:
    - deploy-to-staging
    - run-smoke-tests
    - manual-approval-gate
    - deploy-to-production
    - run-production-health-checks
```

### Parallel vs Sequential Stages

```yaml
# Sequential: each stage depends on the previous
build → test → scan → push → deploy

# Parallel: independent stages run simultaneously
build → test (unit | integration | e2e) → scan (sast | dependency | license) → push → deploy
```

**Rule of thumb:** Stages with no data dependencies should run in parallel.

```
         ┌── unit tests ──┐
build ───┼── integration   ├── scan ── push ── deploy
         └── e2e tests ───┘
```

### Environment Promotion

```
dev (auto) → staging (auto) → production (manual gate)
```

Each environment validates different concerns:
- **dev:** Does it compile? Do unit tests pass?
- **staging:** Does it work with real dependencies? Do integration tests pass?
- **production:** Is it healthy after deployment? Do smoke tests pass?

---

## The Production Way

### GitHub Actions: Complete CI/CD Pipeline

```yaml
# .github/workflows/ci-cd.yml
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
  # ──────────────────────────────────────────────
  # STAGE 1: BUILD
  # ──────────────────────────────────────────────
  build:
    runs-on: ubuntu-latest
    outputs:
      image-tag: ${{ steps.meta.outputs.tags }}
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Log in to Container Registry
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
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
            type=semver,pattern={{version}}

      - name: Build and push
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
      - name: Run unit tests
        run: cargo test --lib -- --format=json | tee test-results.json
      - name: Upload test results
        uses: actions/upload-artifact@v4
        with:
          name: unit-test-results
          path: test-results.json

  integration-tests:
    runs-on: ubuntu-latest
    needs: build
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
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Run integration tests
        run: cargo test --test integration
        env:
          DATABASE_URL: postgres://postgres:test@localhost:5432/postgres

  e2e-tests:
    runs-on: ubuntu-latest
    needs: build
    steps:
      - uses: actions/checkout@v4
      - name: Run API smoke tests
        run: |
          # Start the app using the built image
          docker run -d -p 8080:8080 ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ github.sha }}
          sleep 5
          # Run health check
          curl -f http://localhost:8080/health || exit 1
          # Run contract tests
          ./scripts/run-api-tests.sh

  # ──────────────────────────────────────────────
  # STAGE 3: SECURITY SCAN (parallel)
  # ──────────────────────────────────────────────
  dependency-scan:
    runs-on: ubuntu-latest
    needs: build
    steps:
      - uses: actions/checkout@v4
      - name: Run cargo audit
        run: |
          cargo install cargo-audit
          cargo audit

  container-scan:
    runs-on: ubuntu-latest
    needs: build
    steps:
      - name: Run Trivy vulnerability scanner
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ github.sha }}
          format: table
          exit-code: 1
          severity: CRITICAL,HIGH

  sast:
    runs-on: ubuntu-latest
    needs: build
    steps:
      - uses: actions/checkout@v4
      - name: Run Semgrep
        uses: semgrep/semgrep-action@v1
        with:
          config: p/rust

  # ──────────────────────────────────────────────
  # STAGE 4: GATE — All checks must pass
  # ──────────────────────────────────────────────
  quality-gate:
    runs-on: ubuntu-latest
    needs: [unit-tests, integration-tests, e2e-tests, dependency-scan, container-scan, sast]
    steps:
      - name: All checks passed
        run: echo "All quality gates passed — ready for deployment"

  # ──────────────────────────────────────────────
  # STAGE 5: DEPLOY TO STAGING
  # ──────────────────────────────────────────────
  deploy-staging:
    runs-on: ubuntu-latest
    needs: quality-gate
    if: github.ref == 'refs/heads/main'
    environment: staging
    steps:
      - name: Deploy to staging
        run: |
          kubectl set image deployment/myapp \
            myapp=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ github.sha }} \
            --namespace=staging
          kubectl rollout status deployment/myapp --namespace=staging --timeout=300s

      - name: Run smoke tests on staging
        run: ./scripts/smoke-tests.sh https://staging.myapp.example.com

  # ──────────────────────────────────────────────
  # STAGE 6: DEPLOY TO PRODUCTION (manual gate)
  # ──────────────────────────────────────────────
  deploy-production:
    runs-on: ubuntu-latest
    needs: deploy-staging
    environment: production  # Requires manual approval
    steps:
      - name: Deploy to production
        run: |
          kubectl set image deployment/myapp \
            myapp=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ github.sha }} \
            --namespace=production
          kubectl rollout status deployment/myapp --namespace=production --timeout=300s

      - name: Verify production health
        run: ./scripts/smoke-tests.sh https://myapp.example.com

      - name: Notify team
        uses: slackapi/slack-github-action@v1
        with:
          payload: |
            {"text": "Production deployed: ${{ github.sha }}"}
```

### GitLab CI: Equivalent Pipeline

```yaml
# .gitlab-ci.yml
stages:
  - build
  - test
  - scan
  - staging
  - production

variables:
  IMAGE_TAG: $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA

build:
  stage: build
  image: docker:24
  services:
    - docker:24-dind
  script:
    - docker login -u $CI_REGISTRY_USER -p $CI_REGISTRY_PASSWORD $CI_REGISTRY
    - docker build -t $IMAGE_TAG .
    - docker push $IMAGE_TAG
  cache:
    key: docker-layer-cache
    paths:
      - .docker-cache

unit-tests:
  stage: test
  image: rust:1.75
  script:
    - cargo test --lib
  cache:
    key: cargo-cache
    paths:
      - target/
      - ~/.cargo/registry/

integration-tests:
  stage: test
  image: rust:1.75
  services:
    - postgres:16
  variables:
    DATABASE_URL: "postgres://postgres:postgres@postgres/test"
  script:
    - cargo test --test integration

dependency-audit:
  stage: scan
  image: rust:1.75
  script:
    - cargo install cargo-audit
    - cargo audit

container-scan:
  stage: scan
  image:
    name: aquasec/trivy:latest
  script:
    - trivy image --exit-code 1 --severity HIGH,CRITICAL $IMAGE_TAG

deploy-staging:
  stage: staging
  environment:
    name: staging
  script:
    - kubectl set image deployment/app app=$IMAGE_TAG -n staging
  only:
    - main

deploy-production:
  stage: production
  environment:
    name: production
  script:
    - kubectl set image deployment/app app=$IMAGE_TAG -n production
  when: manual
  only:
    - main
```

### Jenkins: Pipeline as Code

```groovy
// Jenkinsfile
pipeline {
    agent any

    environment {
        REGISTRY = 'registry.example.com'
        IMAGE = "${REGISTRY}/myapp"
    }

    stages {
        stage('Build') {
            steps {
                sh 'docker build -t ${IMAGE}:${BUILD_NUMBER} .'
                sh 'docker push ${IMAGE}:${BUILD_NUMBER}'
            }
        }

        stage('Test') {
            parallel {
                stage('Unit Tests') {
                    steps {
                        sh 'cargo test --lib'
                    }
                }
                stage('Integration Tests') {
                    steps {
                        sh 'cargo test --test integration'
                    }
                }
                stage('Security Scan') {
                    steps {
                        sh 'cargo audit'
                    }
                }
            }
        }

        stage('Deploy to Staging') {
            when { branch 'main' }
            steps {
                sh "kubectl set image deployment/app app=${IMAGE}:${BUILD_NUMBER} -n staging"
            }
        }

        stage('Deploy to Production') {
            when { branch 'main' }
            input {
                message "Deploy to production?"
                ok "Deploy"
            }
            steps {
                sh "kubectl set image deployment/app app=${IMAGE}:${BUILD_NUMBER} -n production"
            }
        }
    }

    post {
        failure {
            slackSend channel: '#deploys', message: "Build FAILED: ${BUILD_URL}"
        }
        success {
            slackSend channel: '#deploys', message: "Deployed: ${IMAGE}:${BUILD_NUMBER}"
        }
    }
}
```

### Pipeline Caching Strategies

```yaml
# GitHub Actions: Rust dependency caching
- name: Cache cargo registry and build
  uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: |
      ${{ runner.os }}-cargo-

# Docker layer caching
- name: Build with cache
  uses: docker/build-push-action@v5
  with:
    context: .
    push: true
    tags: myapp:${{ github.sha }}
    cache-from: type=gha
    cache-to: type=gha,mode=max
```

### Matrix Builds

```yaml
# Test across multiple Rust versions and OS
test:
  strategy:
    matrix:
      rust-version: [stable, nightly]
      os: [ubuntu-latest, macos-latest]
      exclude:
        - rust-version: nightly
          os: macos-latest
  runs-on: ${{ matrix.os }}
  steps:
    - uses: actions/checkout@v4
    - uses: actions-rust-lang/setup-rust-toolchain@v1
      with:
        toolchain: ${{ matrix.rust-version }}
    - run: cargo test
```

### Artifact Management

```yaml
# Semantic versioning from git tags
- name: Determine version
  id: version
  run: |
    if [[ "$GITHUB_REF" == refs/tags/v* ]]; then
      VERSION=${GITHUB_REF#refs/tags/v}
    else
      VERSION="0.0.0-${GITHUB_SHA::8}"
    fi
    echo "version=$VERSION" >> $GITHUB_OUTPUT

# Sign artifacts for supply chain security
- name: Install cosign
  uses: sigstore/cosign-installer@v3

- name: Sign container image
  run: cosign sign ${{ env.REGISTRY }}/${{ env.IMAGE }}@${{ steps.build.outputs.digest }}
```

---

## Hands-On Lab

### Lab: Create a Complete CI/CD Pipeline in GitHub Actions

#### Prerequisites

- GitHub account
- A Rust project with at least one test
- Docker installed locally

#### Step 1: Create a Rust Application with Dockerfile

```bash
mkdir ci-cd-lab && cd ci-cd-lab
cargo init --name ci-cd-lab
```

Add a simple HTTP server:

```toml
# Cargo.toml
[dependencies]
actix-web = "4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

```rust
// src/main.rs
use actix_web::{web, App, HttpServer, HttpResponse};

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status": "healthy"}))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(health))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
```

```rust
// tests/integration.rs
use actix_web::{test, web, App};
// Integration test example
#[actix_web::test]
async fn test_health_endpoint() {
    let app = test::init_service(
        App::new().route("/health", web::get().to(|| async {
            actix_web::HttpResponse::Ok().json(serde_json::json!({"status": "healthy"}))
        }))
    ).await;
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
```

Create a Dockerfile:

```dockerfile
# Dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/ci-cd-lab /usr/local/bin/
EXPOSE 8080
CMD ["ci-cd-lab"]
```

#### Step 2: Create the GitHub Actions Pipeline

```bash
mkdir -p .github/workflows
```

```yaml
# .github/workflows/ci-cd.yml
name: CI/CD Pipeline

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          components: rustfmt, clippy
      - run: cargo fmt -- --check
      - run: cargo clippy -- -D warnings

  test:
    runs-on: ubuntu-latest
    needs: lint
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - run: cargo test

  build:
    runs-on: ubuntu-latest
    needs: test
    permissions:
      contents: read
      packages: write
    outputs:
      image-tag: ${{ steps.meta.outputs.tags }}
    steps:
      - uses: actions/checkout@v4

      - uses: docker/setup-buildx-action@v3

      - uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: type=sha

      - uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          cache-from: type=gha
          cache-to: type=gha,mode=max

  deploy-staging:
    runs-on: ubuntu-latest
    needs: build
    if: github.ref == 'refs/heads/main'
    environment: staging
    steps:
      - name: Deploy to staging
        run: echo "Deploying ${{ needs.build.outputs.image-tag }} to staging"
        # Replace with actual kubectl/helm commands

  deploy-production:
    runs-on: ubuntu-latest
    needs: deploy-staging
    environment: production
    steps:
      - name: Deploy to production
        run: echo "Deploying to production"
```

#### Step 3: Push and Watch

```bash
git init
git add .
git commit -m "feat: initial CI/CD pipeline"
git remote add origin https://github.com/YOUR_USERNAME/ci-cd-lab.git
git push -u origin main
```

Navigate to GitHub > Actions tab to watch the pipeline execute.

#### Step 4: Add a Quality Gate

Add a test that intentionally fails:

```rust
#[test]
fn failing_test() {
    assert_eq!(1, 2, "This test fails to show pipeline stops");
}
```

Push and observe: the pipeline stops at the test stage. No image is built. No deployment occurs.

#### Step 5: Add Matrix Builds

```yaml
test:
  runs-on: ubuntu-latest
  needs: lint
  strategy:
    matrix:
      rust: [stable, "1.75"]
  steps:
    - uses: actions/checkout@v4
    - uses: actions-rust-lang/setup-rust-toolchain@v1
      with:
        toolchain: ${{ matrix.rust }}
    - run: cargo test
```

#### Verification Checklist

- [ ] Lint job catches formatting issues
- [ ] Test job runs unit and integration tests
- [ ] Build job creates and pushes a container image
- [ ] Staging deployment triggers automatically on main
- [ ] Production deployment requires manual approval
- [ ] Failed tests block the entire pipeline
- [ ] Matrix builds run tests on multiple Rust versions
- [ ] Docker layer caching reduces build time

---

## Key Takeaways

1. **Pipeline as Code** — Pipeline definitions live in the repository, versioned alongside application code
2. **Fail Fast** — Run cheap checks (lint, unit tests) before expensive ones (integration, e2e)
3. **Parallelism** — Independent stages should run concurrently to reduce total pipeline time
4. **Artifacts** — Build once, deploy the same artifact to every environment
5. **Gates** — Manual approval before production prevents accidental deployments
6. **Caching** — Cache dependencies and Docker layers to speed up builds
7. **Environment Promotion** — dev -> staging -> production, each validating different concerns

---

## Limitation

Pipeline design gives you the automated path from code to production. But deploying to production still means replacing the old version with the new version. If something goes wrong, users experience downtime or errors while you roll back.

**Next:** Module 58 — Blue-Green Deployment solves this by maintaining two identical environments and switching traffic atomically, enabling zero-downtime releases and instant rollback.
