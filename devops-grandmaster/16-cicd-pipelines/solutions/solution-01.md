# Solution 01: CI vs CD and Deployment Pipelines

## Part A: Define the Three CI/CD Concepts

### 1. Continuous Integration (CI)

Continuous Integration is a practice where every developer commits code to a shared repository multiple times per day. Each commit triggers an automated build and test sequence that validates the change integrates correctly with the existing codebase. The process is triggered by a `git push` or merge request, runs automated compilation and testing in an isolated environment, and ends with a pass/fail status reported back to the developer. If done manually, a developer might forget to run the full test suite before merging, allowing a subtle regression to slip into the main branch and break other developers' work hours or days later.

### 2. Continuous Delivery (CDelivery)

Continuous Delivery extends CI by ensuring that every change passing all automated tests is packaged into a release artifact and made ready for deployment to any environment at the push of a button. The process is triggered after CI passes, runs additional steps like artifact packaging, staging deployment, and integration testing, and ends with the artifact in a "deployable" state awaiting manual approval for production. If done manually, the release process might involve a multi-hour "release day" ritual where someone manually builds, packages, and promotes artifacts, introducing delays and human error between "code is ready" and "code is deployed."

### 3. Continuous Deployment (CDeployment)

Continuous Deployment takes Continuous Delivery one step further by removing the manual approval gate. Every change that passes all automated tests is automatically deployed to production without human intervention. The process is triggered by a successful CI pipeline, runs the full deployment sequence including production rollout and health verification, and ends with the change live in production. If done manually, deployments would be batched into infrequent, large releases, increasing the risk per deployment and making it difficult to identify which change caused a production issue.

### Why This Works

The three concepts form a progression: CI ensures code quality, CDelivery ensures deployability, CDeployment ensures velocity. Most teams start with CI and gradually adopt CDelivery and CDeployment as their confidence in automation grows.

### Common Mistakes

- **Confusing Continuous Delivery with Continuous Deployment.** Delivery means "ready to deploy at any time" with a manual gate. Deployment means "automatically deployed." The difference is whether a human clicks a button.
- **Treating CI as "run tests on main."** CI means integrating frequently -- feature branches that live for weeks without CI are not continuous integration.

## Part B: Identify Pipeline Stages

| Stage | Category |
|-------|----------|
| `docker build -t myapp:$SHA .` | **Build** |
| `npm test` inside the container | **Test** |
| `trivy image myapp:$SHA` | **Security** |
| `docker push registry.example.com/myapp:$SHA` | **Publish** |
| `kubectl set image deployment/myapp myapp=registry.example.com/myapp:$SHA` | **Deploy** |
| `curl -f http://staging.example.com/health` | **Verify** |
| `docker compose run --rm app npm run lint` | **Test** |
| `cosign sign registry.example.com/myapp:$SHA` | **Security** |
| `helm upgrade myapp ./chart --set image.tag=$SHA` | **Deploy** |
| `docker buildx build --platform linux/amd64,linux/arm64` | **Build** |

### Why This Works

The classification follows the purpose of each step: Build produces artifacts, Test validates correctness, Security checks for vulnerabilities or provenance, Publish makes artifacts available, Deploy releases to an environment, and Verify confirms the release works.

### Common Mistakes

- **Classifying `cosign sign` as Publish.** Signing is a security operation that establishes provenance and integrity. It belongs in the Security category, not Publish.
- **Classifying `helm upgrade` as Verify.** Helm upgrade modifies the cluster state -- it is a Deploy operation, even though Helm can also be used for verification.

## Part C: Pipeline Failure Scenarios

| Scenario | Stage That Catches It | Explanation |
|----------|----------------------|-------------|
| 1. Code compiles but tests fail | **Test** (unit/integration test step) | The test suite returns a non-zero exit code, which the CI runner interprets as failure. |
| 2. Image contains a Critical CVE | **Security** (Trivy or equivalent scan) | The vulnerability scanner inspects the image layers and fails on Critical severity findings. |
| 3. Staging unreachable after deploy | **Verify** (health check / smoke test) | A post-deployment health check that curls the staging endpoint and checks for HTTP 200. |
| 4. Migration drops a production table | **No existing stage catches this** | Proposed new stage: **Migration Review** or **Database Gate**. This requires a separate review step that inspects migration files for destructive operations (DROP, TRUNCATE) and requires explicit approval. Some teams use a "migration dry-run" stage that runs `--dry-run` against a copy of the production schema. |
| 5. Image crashes on arm64 | **Test** (platform-specific test stage) | A test stage that runs the container on the target architecture. With `docker buildx`, build for both platforms and test on an arm64 runner or QEMU-emulated environment. |

### Why This Works

Each scenario maps to the pipeline stage whose specific responsibility is to catch that class of problem. Scenario 4 is the key insight: not all problems fit into existing stages. Destructive database migrations require specialized tooling and review processes that are separate from standard CI/CD stages.

### Common Mistakes

- **Assuming the Test stage catches everything.** Tests catch code bugs, but they do not catch infrastructure issues, vulnerability exposures, or schema-level problems.
- **Relying on manual review for security scans.** If the scan does not fail the pipeline, developers will ignore the results. Enforce thresholds automatically.

## Part D: Map the Flow

```
git push
    |
    v
+-----------+     +-----------+
|  Lint     |     | Unit Test |
|  (Build)  |     |  (Test)   |
+-----------+     +-----------+
    |                  |
    +--------+---------+
             |
             v
       +-----------+
       | Docker    |
       | Build     |
       +-----------+
             |
             v
       +-----------+     +-----------+
       | Security  |     | Integration|
       | Scan      |     | Test      |
       +-----------+     +-----------+
             |                  |
             +--------+---------+
                      |
                      v
              +----------------+
              | All checks     |
              | pass?          |
              +----------------+
                  |         |
                Yes         No
                  |         |
                  v         v
          +-----------+  +-----------+
          | Push to   |  | Notify    |
          | Registry  |  | developer |
          +-----------+  +-----------+
                |
                v
        +-----------+
        | Deploy to |
        | Staging   |
        +-----------+
                |
                v
        +-----------+
        | Staging   |
        | Health OK?|
        +-----------+
            |       |
          Yes       No
            |       |
            v       v
    +-----------+ +-----------+
    | Deploy to | | Rollback  |
    | Prod      | | Staging   |
    +-----------+ +-----------+
            |
            v
    +-----------+
    | Prod      |
    | Health OK?|
    +-----------+
        |       |
      Yes       No
        |       |
        v       v
  +---------+ +-----------+
  | Done    | | Rollback  |
  |         | | Prod      |
  +---------+ +-----------+
```

### Why This Works

The flow demonstrates parallelism (lint and unit test run simultaneously, security scan and integration test run simultaneously), decision gates (health checks can halt the pipeline), and rollback paths at both staging and production. This is the foundation of a production-grade pipeline.

### Common Mistakes

- **No parallelism.** Running all stages sequentially doubles or triples pipeline duration. Identify independent stages and run them in parallel.
- **No rollback path.** A pipeline that deploys but cannot roll back is a one-way ratchet. Every deploy stage needs a corresponding rollback.

## Key Takeaway

CI/CD is a chain of automated stages with clear responsibilities: Build produces artifacts, Test validates correctness, Security scans for vulnerabilities, Publish makes artifacts available, Deploy releases to environments, and Verify confirms the release works. The pipeline must include parallelism for speed, decision gates for safety, and rollback paths for resilience. Understanding these concepts is prerequisite to implementing any pipeline, regardless of the CI/CD tool chosen.
