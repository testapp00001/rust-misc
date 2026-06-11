# Exercise 05: End-to-End Pipeline with Environment Promotion

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Design and implement a complete CI/CD pipeline that builds, tests, scans,
packages, and promotes a Rust microservice through dev, staging, and
production environments with appropriate gates at each stage. This exercise
combines everything from the previous exercises into a production-grade
pipeline.

## Scenario

You are the platform engineer for an e-commerce company. The `checkout-service`
is a Rust HTTP API that handles payment processing. It has the following
requirements:

- **Dev:** Auto-deploy on every push to `develop` branch. Run unit tests only.
- **Staging:** Auto-deploy on every push to `main` branch. Run unit, integration,
  and end-to-end tests. Scan for vulnerabilities.
- **Production:** Deploy only after manual approval. Run smoke tests after
  deployment. Automatic rollback if health checks fail.

The service connects to PostgreSQL and Redis. It exposes a `/health` endpoint
that returns `200 OK` when healthy. The team uses GitHub Container Registry.

## Tasks

### Part A: Design the Pipeline Architecture

Draw an ASCII diagram showing the complete pipeline flow from code push to
production. Include all environments, gates, and parallel/sequential
relationships. Label which branch triggers which path.

<details>
<summary>Hint</summary>

There are two main paths:
1. `develop` branch: build -> test -> scan -> deploy dev
2. `main` branch: build -> test + scan (parallel) -> quality gate -> deploy staging -> smoke test -> manual approval -> deploy production -> production health check

The build stage can be shared between both paths if you use conditional logic.

</details>

### Part B: Write the Build and Push Stage

Create the build job that:
- Builds a Docker image with multi-stage Dockerfile
- Tags it with the commit SHA and branch name
- Pushes to GitHub Container Registry
- Uses build cache for fast rebuilds
- Outputs the image tag for downstream jobs

<details>
<summary>Hint</summary>

Use `docker/metadata-action` for tagging and `docker/build-push-action` with
`cache-from: type=gha` and `cache-to: type=gha,mode=max` for GitHub Actions
cache integration.

</details>

### Part C: Write the Test and Scan Stage

Create parallel jobs for:
- Unit tests (`cargo test --lib`)
- Integration tests (with PostgreSQL and Redis service containers)
- Dependency vulnerability scan (`cargo audit`)
- Container image scan (Trivy)

All four jobs should run in parallel after the build stage.

<details>
<summary>Hint</summary>

Each test job needs the built image or source code. For integration tests,
use the `services` key:

```yaml
services:
  postgres:
    image: postgres:16
    env:
      POSTGRES_PASSWORD: test
    ports:
      - 5432:5432
  redis:
    image: redis:7
    ports:
      - 6379:6379
```

</details>

### Part D: Write the Deployment Stage

Create the deployment jobs that:
1. Deploy to staging automatically on main branch
2. Run smoke tests against staging (curl the /health endpoint and a test API call)
3. Wait for manual approval via GitHub environment protection rules
4. Deploy to production
5. Run production health checks
6. Roll back automatically if health checks fail

<details>
<summary>Hint 1</summary>

For rollback, capture the current image tag before deploying. If the new
deployment fails health checks, re-deploy the previous image:

```yaml
- name: Capture current image
  id: current
  run: |
    CURRENT=$(kubectl get deployment checkout-service -o jsonpath='{.spec.template.spec.containers[0].image}')
    echo "image=$CURRENT" >> "$GITHUB_OUTPUT"

- name: Rollback on failure
  if: failure()
  run: |
    kubectl set image deployment/checkout-service \
      checkout-service=${{ steps.current.outputs.image }}
```

</details>

<details>
<summary>Hint 2</summary>

For the production environment, use GitHub's `environment` key with a
`required_reviewers` list configured in the repository settings. This
creates a manual gate without any custom code.

</details>

## Success Criteria

- [ ] ASCII diagram shows all environments, gates, and parallel/sequential flows
- [ ] Build stage uses Docker layer caching and proper image tagging
- [ ] Four test/scan jobs run in parallel with correct service containers
- [ ] Staging deployment is automatic on main branch pushes
- [ ] Production deployment requires manual approval
- [ ] Production health checks trigger automatic rollback on failure
- [ ] The complete pipeline from push to production-ready is under 10 minutes

## What You Should Understand After This Exercise

A production-grade CI/CD pipeline is an environment promotion system, not
just a build script. Each environment validates different concerns: dev
validates compilation, staging validates integration, and production
validates real-world health. The gates between environments are what prevent
bad code from reaching users.
