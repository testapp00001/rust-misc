# Solution 01: Anatomy of a Broken Pipeline

## Part A: List What Is Missing

The pipeline is missing these critical stages:

| Missing Stage | Purpose | Risk of Skipping |
|---------------|---------|------------------|
| **Unit tests** | Validate individual function correctness | Bugs ship to production undetected |
| **Integration tests** | Validate component interactions | API contracts break silently |
| **Security scanning** | Detect dependency vulnerabilities | Known CVEs ship in production images |
| **Container image scanning** | Detect OS-level vulnerabilities | Base image vulnerabilities reach production |
| **Staging deployment** | Validate in production-like environment | Integration issues only found in production |
| **Smoke tests** | Verify deployment health | Broken deployments go undetected |
| **Manual approval gate** | Human review before production | No safety net for automated failures |
| **Artifact versioning** | Reproducible deployments | Cannot identify which code is running |

### Why This Matters

Each stage exists to catch a specific category of defect. The pipeline as
written only validates that the code compiles and the Docker image can be
pushed. It catches zero correctness, security, or integration issues.

## Part B: Identify the Anti-Patterns

### Anti-Pattern 1: Using the `latest` Tag

```yaml
docker push registry.example.com/myapp:latest
```

The `latest` tag is mutable. If two commits push in quick succession, the
first deployment might pull the second commit's image. You cannot roll back
to a specific version because every version was tagged `latest`. You cannot
audit which code is running in production.

### Anti-Pattern 2: No Tests Before Deployment

The pipeline builds and deploys without running any tests. A single typo
in a configuration file could break the entire service, and the pipeline
would deploy it happily.

### Anti-Pattern 3: No Staging Environment

Deployment goes directly to production. There is no intermediate environment
to catch integration issues, performance regressions, or configuration
mismatches before users are affected.

### Anti-Pattern 4: No Rollback Strategy

```yaml
kubectl set image deployment/myapp myapp=registry.example.com/myapp:latest
```

If the new version crashes, there is no automatic rollback. The pipeline
does not check if the deployment succeeded, and `latest` makes manual
rollback ambiguous -- what do you roll back to?

### Anti-Pattern 5: No Security Scanning

The pipeline pushes code to production without checking for known
vulnerabilities in dependencies or the container image. A single
compromised dependency could expose the entire production environment.

### Anti-Pattern 6: Single Monolithic Job

Everything runs in one job sequentially. If the build takes 5 minutes and
tests take 3 minutes, the total is 8 minutes. If tests and build were
parallel (after a shared build step), the wall-clock time would be shorter.

## Part C: Draw the Correct Pipeline

```
  push to main
       |
       v
  ┌─────────┐
  │  Build   │  Compile Rust, build Docker image, push with SHA tag
  └────┬─────┘
       |
       ├────────────────┬────────────────┬────────────────┐
       v                v                v                v
  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐
  │  Unit    │   │Integra-  │   │Dependency│   │ Container│
  │  Tests   │   │tion Tests│   │  Scan    │   │  Scan    │
  └────┬─────┘   └────┬─────┘   └────┬─────┘   └────┬─────┘
       │                │              │              │
       └────────────────┴──────────────┴──────────────┘
                            |
                            v
                    ┌──────────────┐
                    │Quality Gate  │  All checks must pass
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
                    │ Smoke Tests  │  Validate staging health
                    └──────┬───────┘
                           |
                           v
                    ┌──────────────┐
                    │   Manual     │  Human approval gate
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
                    │  Production  │  Rollback if unhealthy
                    │Health Checks │
                    └──────────────┘
```

## Part D: Explain the Consequences

### `latest` Tag Consequence

Developer A pushes commit X at 10:00 AM. The pipeline starts building. At
10:01 AM, Developer B pushes commit Y. The pipeline for commit X finishes
and pushes `myapp:latest`. The pipeline for commit Y finishes and pushes
`myapp:latest`, overwriting X's image. When the deployment for commit X
runs, it pulls `myapp:latest` which is actually commit Y's code. The
deployment "succeeds" but the wrong code is running. Debugging this takes
hours because the running code does not match the commit that triggered
the deployment.

### No Tests Consequence

A developer refactors the payment calculation function and accidentally
changes `total * 0.08` (8% tax) to `total * 0.8` (80% tax). The code
compiles. The Docker image builds. The pipeline deploys it to production.
Customers are charged 80% tax for 45 minutes before someone notices.
A single unit test would have caught this in 2 seconds.

### No Staging Consequence

The service works perfectly in the developer's local environment and in
CI. But in production, it connects to a PostgreSQL 15 database while the
developer tested against PostgreSQL 16. A new SQL feature used in the
query fails silently in PostgreSQL 15, returning empty results. Orders
start disappearing. A staging environment with production-matching
databases would have caught this.

### No Rollback Consequence

The new version has a memory leak that causes the process to be OOM-killed
after 10 minutes. The pipeline deploys it successfully (the health check
passes at startup), but 10 minutes later all pods are crashing. The on-call
engineer has to manually identify the previous image tag (which was `latest`,
so they cannot), rebuild the old code, push a new image, and redeploy. The
outage lasts 45 minutes instead of the 30 seconds a rollback would take.

## Common Mistakes to Avoid

- **Treating the pipeline as "just automation."** The pipeline is your
  quality gate. Every stage you skip is a category of defect that ships
  to users.
- **Using `latest` tags.** Always tag images with commit SHA or semantic
  version. The `latest` tag is a convenience for local development, not
  for production deployments.
- **Running everything sequentially.** Independent stages (tests, scans)
  should run in parallel to reduce feedback time.
- **Deploying directly to production.** Always use a staging environment
  as a buffer. The cost of staging is far less than the cost of a
  production outage.

## Key Takeaway

A production pipeline is a series of quality gates, each designed to catch
a specific category of defect. The stages are not optional -- each one
exists because someone, somewhere, shipped a bug that stage would have
caught. Design your pipeline by asking: "What could go wrong, and which
stage catches it?"
