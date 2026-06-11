# Exercise 05: Change Management Pipeline with Approval Gates

**Type:** Integration
**Duration:** 120-180 minutes

## Objective

Design and implement a complete change management pipeline that enforces
approval gates at critical stages, maintains an audit trail, and integrates
with a CI/CD system. The pipeline must support both automated and manual
approval, enforce separation of duties, and produce a deployable artifact only
after all gates pass.

## Context

Your organization requires all production changes to pass through a structured
approval process. The security team mandates that the person who writes the code
cannot be the same person who approves production deployment. The compliance
team requires a complete audit trail of every change, including who approved it
and when.

## Instructions

### Part 1 -- Pipeline Architecture

Design a pipeline with the following stages. For each stage, define:
- What happens in the stage
- What the inputs and outputs are
- What the gate criteria are
- Who can approve (if manual)

**Stages:**

1. **Source** -- Code is committed to a feature branch and a PR is opened
2. **Build** -- Application is built, unit tests run, container image is created
3. **Security Scan** -- Image is scanned for vulnerabilities (CVEs), SAST/DAST
   runs
4. **Deploy to Staging** -- Image is deployed to a staging environment
5. **Integration Tests** -- Automated tests run against staging
6. **Change Approval** -- Manual gate: Change Advisory Board or designated
   approver reviews the change
7. **Deploy to Production** -- Image is deployed to production using a rolling
   update strategy
8. **Post-Deploy Verification** -- Automated smoke tests and health checks
   confirm production is healthy
9. **Monitoring Period** -- A soak period (e.g., 30 minutes) where metrics are
   observed before the change is considered successful

Draw this as a diagram (ASCII art is fine) and document each stage in detail.

### Part 2 -- GitHub Actions Pipeline

Implement the pipeline as a GitHub Actions workflow (`.github/workflows/
change-management.yml`). The workflow must:

1. **Trigger** on pull requests to `main` and on manual `workflow_dispatch`
2. **Enforce branch protection** -- the PR must be approved by at least one
   reviewer who is NOT the PR author
3. **Build stage:**
   - Build a Docker image tagged with the commit SHA
   - Run unit tests (use a placeholder test command)
4. **Security stage:**
   - Run Trivy or similar scanner on the built image
   - Fail the pipeline if any CRITICAL or HIGH vulnerabilities are found
5. **Staging stage:**
   - Deploy to a staging namespace using `kubectl`
   - Wait for rollout to complete
   - Run integration tests (placeholder)
6. **Approval gate:**
   - Use GitHub Environments with required reviewers
   - The environment `production-approval` must have at least one required
     reviewer who is different from the PR author
7. **Production stage:**
   - Deploy to production namespace using rolling update
   - Run smoke tests
8. **Post-deploy:**
   - Monitor for 5 minutes (configurable)
   - Check error rates and latency (use placeholder metrics)
   - If metrics degrade, automatically trigger rollback (call the rollback
     script from Exercise 04 if you built one)

### Part 3 -- Approval Gate Configuration

Create the configuration for GitHub Environments that enforces:

1. **Production approval environment:**
   - Required reviewers: at least 2 from the `platform-approvers` team
   - Wait timer: 0 minutes (no artificial delay, but manual approval required)
   - Deployment branches: only `main`
2. **Separation of duties enforcement:**
   - Document how the pipeline prevents the PR author from approving their own
     deployment
   - Handle the edge case: what if there is only one platform approver and they
     are the author?
3. **Emergency bypass:**
   - Design a mechanism for emergency changes that skips the approval gate
   - The bypass must still log who authorized the emergency and why
   - The bypass must require a specific label on the PR (e.g., `emergency`)

### Part 4 -- Audit Trail

Design and implement an audit trail system:

1. **What to log:**
   - Every pipeline start, stage transition, and completion
   - Every approval (who, when, what was approved, any comments)
   - Every deployment (image tag, environment, timestamp, deployer)
   - Every rollback (trigger, who authorized, target revision)
2. **Where to log:**
   - GitHub Actions step summaries for visibility
   - A separate audit log file (JSON format) committed to a `audit-logs/`
     branch or stored as a workflow artifact
3. **Log format:**
   - Each entry must be a JSON object with fields: `timestamp`, `event_type`,
     `actor`, `target`, `details`, `outcome`
   - Provide three example log entries for: an approval, a deployment, and a
     rollback

### Part 5 -- Change Freeze Mechanism

Implement a mechanism to enforce change freezes (e.g., during holidays or
major sales events):

1. A configuration file (`change-freeze.json`) that defines freeze periods:

```json
{
  "freeze_periods": [
    {
      "name": "Year-End Freeze",
      "start": "2026-12-20T00:00:00Z",
      "end": "2027-01-03T00:00:00Z",
      "allowed_labels": ["emergency"],
      "message": "Year-end change freeze is in effect. Only emergency changes are permitted."
    }
  ]
}
```

2. A script (`check-freeze.sh`) that:
   - Reads the freeze configuration
   - Checks if the current time falls within any freeze period
   - Checks if the PR has an allowed label (e.g., `emergency`)
   - Exits 0 if the change is allowed, exits 1 if blocked
   - Prints the freeze message if blocked

3. Integration into the pipeline: the freeze check runs as the first step of
   the pipeline and blocks all non-emergency changes during freeze periods.

## Success Criteria

- [ ] Pipeline architecture diagram shows all 9 stages with gates and approval points
- [ ] GitHub Actions workflow file is valid YAML and implements all stages
- [ ] Approval gate uses GitHub Environments with required reviewers
- [ ] Separation of duties is enforced and edge cases are documented
- [ ] Emergency bypass mechanism exists with logging requirements
- [ ] Audit trail captures approvals, deployments, and rollbacks in JSON format
- [ ] Three example audit log entries are provided
- [ ] Change freeze configuration and checker script are implemented
- [ ] Freeze checker correctly blocks changes during freeze periods
- [ ] Freeze checker allows emergency-labeled changes during freezes
- [ ] Pipeline fails gracefully at any stage with clear error messages

## Hints

<details>
<summary>Hint 1 -- GitHub Environments</summary>

GitHub Environments are configured in the repository settings (Settings >
Environments). In the workflow, reference them with:

```yaml
jobs:
  deploy-production:
    environment:
      name: production-approval
    runs-on: ubuntu-latest
    steps:
      - run: echo "Deploying to production"
```

Required reviewers are configured in the UI, not in the YAML.

</details>

<details>
<summary>Hint 2 -- Preventing Self-Approval</summary>

GitHub's environment protection rules already prevent the workflow trigger
actor from being the sole approver if they are listed as a required reviewer.
Document this behavior and the edge case when the team is small.

</details>

<details>
<summary>Hint 3 -- Freeze Check Date Comparison</summary>

Use `jq` and `date` for date comparison in the freeze checker:

```bash
CURRENT_EPOCH=$(date +%s)
START_EPOCH=$(date -d "$start" +%s)
END_EPOCH=$(date -d "$end" +%s)

if [ "$CURRENT_EPOCH" -ge "$START_EPOCH" ] && [ "$CURRENT_EPOCH" -le "$END_EPOCH" ]; then
    echo "Freeze active"
fi
```

</details>

<details>
<summary>Hint 4 -- Audit Log Structure</summary>

Use `jq` to create properly formatted JSON entries:

```bash
jq -n \
  --arg ts "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
  --arg event "deployment" \
  --arg actor "$GITHUB_ACTOR" \
  --arg target "$IMAGE_TAG" \
  --arg outcome "success" \
  '{timestamp: $ts, event_type: $event, actor: $actor, target: $target, outcome: $outcome}'
```

</details>

<details>
<summary>Hint 5 -- Pipeline Error Handling</summary>

Use `if: failure()` and `if: always()` conditions in GitHub Actions to run
cleanup or notification steps even when earlier steps fail:

```yaml
- name: Notify on failure
  if: failure()
  run: echo "Pipeline failed at stage ${{ steps.current_stage.outputs.name }}"
```

</details>

## Deliverables

1. `pipeline-architecture.md` -- Architecture diagram and stage documentation
2. `.github/workflows/change-management.yml` -- GitHub Actions workflow
3. `environments.md` -- GitHub Environment configuration documentation
4. `audit-log-format.md` -- Audit log format specification with three examples
5. `change-freeze.json` -- Freeze period configuration
6. `check-freeze.sh` -- Freeze checker script
7. `emergency-bypass.md` -- Documentation of the emergency bypass procedure
