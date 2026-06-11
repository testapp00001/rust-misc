# Exercise 04: Multi-Environment Deployment Pipeline

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Design and implement a GitHub Actions pipeline that deploys a containerized application through three environments (dev, staging, production) with environment-specific configurations, manual approval gates, and automatic rollback on failure.

## Scenario

Your application must be deployed to three environments in sequence:

```
dev  -->  staging  -->  production
```

The rules are:

- **Dev:** Deployed automatically on every push to `main`. No approval needed.
- **Staging:** Deployed automatically after dev succeeds. Runs integration tests.
- **Production:** Deployed only after staging succeeds AND a designated approver approves. Has automatic rollback if health checks fail.

Each environment has its own Kubernetes cluster, its own container registry prefix, and its own set of environment variables.

## Tasks

### Part A: Define Environment-Specific Configuration

Create the configuration structure for three environments. For each environment, define:

1. The Kubernetes namespace.
2. The container registry prefix.
3. The number of replicas.
4. The resource limits (CPU and memory).
5. Any environment-specific feature flags or variables.

Write this as a reusable matrix or configuration block that the pipeline can reference.

| Setting | Dev | Staging | Production |
|---------|-----|---------|------------|
| Namespace | `dev` | `staging` | `production` |
| Registry | `ghcr.io/org/dev/` | `ghcr.io/org/staging/` | `ghcr.io/org/prod/` |
| Replicas | 1 | 2 | 3 |
| CPU Limit | 250m | 500m | 1000m |
| Memory Limit | 256Mi | 512Mi | 1Gi |
| Feature Flags | `DEBUG=true` | `DEBUG=false` | `DEBUG=false` |

Implement this using GitHub Actions environments and workflow-level variables.

<details>
<summary>Hint</summary>

GitHub Actions supports `environment:` at the job level. You can define environment variables and secrets per environment in the repository settings. Use an `env:` block or `vars.*` context to reference them.

</details>

### Part B: Build the Three-Stage Deployment Pipeline

Write the GitHub Actions workflow with three deployment jobs:

1. **deploy-dev:** Runs after build and test succeed. Deploys to the dev environment.
2. **deploy-staging:** Runs after deploy-dev succeeds. Deploys to staging and runs integration tests.
3. **deploy-production:** Runs after deploy-staging succeeds AND receives manual approval.

Each job must:

- Use `needs:` to enforce ordering.
- Reference the correct GitHub Actions environment.
- Deploy using `kubectl set image` or `helm upgrade`.
- Report deployment status.

<details>
<summary>Hint</summary>

Use `needs: [deploy-dev]` for the staging job. For production approval, configure the `production` environment in GitHub with "Required reviewers." The workflow will pause until an approver clicks "Approve and deploy" in the GitHub UI.

</details>

### Part C: Add Integration Tests in Staging

After deploying to staging, the pipeline must run a suite of integration tests against the staging environment:

1. Wait for the staging deployment to be ready (pods in `Running` state).
2. Run API health check against the staging URL.
3. Run a smoke test suite (at least 3 test scenarios).
4. If any test fails, block the production deployment.

Write the integration test step and the readiness check.

<details>
<summary>Hint</summary>

Use `kubectl rollout status` to wait for readiness. Use `curl` with retry logic for the health check. The smoke tests can be a separate test script that runs against the staging endpoint.

</details>

### Part D: Implement Automatic Rollback in Production

If the production deployment fails its health check, the pipeline must automatically roll back:

1. Deploy to production.
2. Run a health check (HTTP 200 on `/health` within 60 seconds).
3. If the health check fails, execute `kubectl rollout undo`.
4. Send a notification (Slack webhook or GitHub issue) about the failed deployment and rollback.

Write the deployment, health check, rollback, and notification steps.

<details>
<summary>Hint</summary>

Use a `continue-on-error: true` step for the health check, then a conditional step that checks the outcome and runs `kubectl rollout undo`. For notifications, use `curl` to post to a Slack webhook URL stored as a secret.

</details>

## Success Criteria

- [ ] The pipeline defines three distinct environments with separate configurations.
- [ ] Dev deploys automatically on push to main.
- [ ] Staging deploys automatically after dev and runs integration tests.
- [ ] Production requires manual approval before deploying.
- [ ] Integration tests in staging gate the production deployment.
- [ ] Production health checks run within 60 seconds of deployment.
- [ ] Failed production health checks trigger automatic rollback via `kubectl rollout undo`.
- [ ] Failed deployments send a notification.
- [ ] The complete workflow is valid YAML and follows GitHub Actions syntax.

## What You Should Understand After This Exercise

Multi-environment pipelines are not just "the same deploy script run three times." Each environment has different configuration, different risk tolerance, and different gates. Dev is fast and automatic. Staging validates the release candidate. Production has human approval and automatic rollback. GitHub Actions environments provide the infrastructure for approval gates, environment-specific secrets, and deployment history. The pipeline enforces promotion discipline: code cannot skip an environment.
