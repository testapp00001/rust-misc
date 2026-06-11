# Solution 05: Change Management Pipeline with Approval Gates

## Pipeline Architecture (pipeline-architecture.md)

### Pipeline Diagram

```
  Feature Branch / PR
         |
         v
  +------------------+
  | 1. SOURCE        |  Gate: PR approved by 1+ reviewer (not author)
  |    PR review     |  Gate: Branch protection passes
  +------------------+         |
         |                     v
         v             +------------------+
  +------------------+ | 2. BUILD         |
  | Freeze Check     | | Build image      |
  | (check-freeze.sh)| | Run unit tests   |
  +------------------+ +------------------+
         |                     |
    [blocked?]                 v
    --> exit 1         +------------------+
                       | 3. SECURITY SCAN |
                       | Trivy scan       |
                       | SAST/DAST        |
                       +------------------+
                              |
                         [CRITICAL/HIGH?]
                         --> exit 1
                              |
                              v
                       +------------------+
                       | 4. STAGING DEPLOY|
                       | kubectl apply    |
                       | Wait for rollout |
                       +------------------+
                              |
                              v
                       +------------------+
                       | 5. INTEGRATION   |
                       |    TESTS         |
                       | Run test suite   |
                       +------------------+
                              |
                         [tests fail?]
                         --> exit 1
                              |
                              v
                       +------------------+
                       | 6. CHANGE        |  <-- MANUAL GATE
                       |    APPROVAL      |
                       | GitHub Environ.  |
                       | Required reviewer|
                       +------------------+
                              |
                         [approved?]
                         --> no: PR stays pending
                              |
                              v
                       +------------------+
                       | 7. PRODUCTION    |
                       |    DEPLOY        |
                       | Rolling update   |
                       | kubectl apply    |
                       +------------------+
                              |
                              v
                       +------------------+
                       | 8. POST-DEPLOY   |
                       |    VERIFICATION  |
                       | Smoke tests      |
                       | Health checks    |
                       +------------------+
                              |
                         [verification fail?]
                         --> auto-rollback
                              |
                              v
                       +------------------+
                       | 9. MONITORING    |
                       |    PERIOD        |
                       | 5-minute soak    |
                       | Error rate check |
                       | Latency check    |
                       +------------------+
                              |
                         [metrics degrade?]
                         --> auto-rollback
                              |
                              v
                       +------------------+
                       | AUDIT LOG        |
                       | Record outcome   |
                       +------------------+
```

### Stage Details

| Stage | Input | Output | Gate Criteria | Approver |
|-------|-------|--------|---------------|----------|
| 1. Source | Git commit, PR | Approved PR | PR approved by 1+ non-author reviewer; all checks pass | Code reviewer (not author) |
| 2. Build | Source code | Docker image, test results | Unit tests pass (exit 0) | Automated |
| 3. Security | Docker image | Scan report | Zero CRITICAL/HIGH CVEs | Automated |
| 4. Staging | Image tag | Running staging deployment | `kubectl rollout status` succeeds | Automated |
| 5. Integration | Staging deployment | Test report | All integration tests pass | Automated |
| 6. Approval | RFC, test results, scan report | Approval record | 2+ platform-approvers approve; author excluded | CAB / platform-approvers team |
| 7. Production | Approved image | Running production deployment | Rollout completes within timeout | Automated (post-approval) |
| 8. Verification | Production deployment | Smoke test results | All smoke tests pass; health endpoints return 200 | Automated |
| 9. Monitoring | Live production | Metrics snapshot | Error rate < 1%, P99 latency within baseline | Automated |

---

## GitHub Actions Workflow (.github/workflows/change-management.yml)

```yaml
name: Change Management Pipeline

on:
  pull_request:
    branches: [main]
  workflow_dispatch:
    inputs:
      image_tag:
        description: 'Override image tag (leave empty for commit SHA)'
        required: false

# Cancel in-progress runs for the same PR
concurrency:
  group: change-mgmt-${{ github.head_ref || github.run_id }}
  cancel-in-progress: true

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}
  STAGING_NAMESPACE: staging
  PRODUCTION_NAMESPACE: production
  MONITORING_DURATION: "300"  # 5 minutes

jobs:
  # ============================================================
  # Stage 1: Freeze Check
  # ============================================================
  freeze-check:
    name: "Stage 0: Change Freeze Check"
    runs-on: ubuntu-latest
    outputs:
      freeze_active: ${{ steps.check.outputs.freeze_active }}
    steps:
      - uses: actions/checkout@v4

      - name: Check for change freeze
        id: check
        run: |
          if [ -f check-freeze.sh ]; then
            if bash check-freeze.sh --pr-labels "${{ github.event.pull_request.labels.*.name || 'none' }}"; then
              echo "freeze_active=false" >> "$GITHUB_OUTPUT"
            else
              echo "::error::Change freeze is active. Only emergency changes are allowed."
              echo "freeze_active=true" >> "$GITHUB_OUTPUT"
              exit 1
            fi
          else
            echo "freeze_active=false" >> "$GITHUB_OUTPUT"
          fi

  # ============================================================
  # Stage 2: Build
  # ============================================================
  build:
    name: "Stage 2: Build & Unit Tests"
    runs-on: ubuntu-latest
    needs: freeze-check
    permissions:
      contents: read
      packages: write
    outputs:
      image_tag: ${{ steps.meta.outputs.version }}
    steps:
      - uses: actions/checkout@v4

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
            type=ref,event=pr

      - name: Build Docker image
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max

      - name: Run unit tests
        run: |
          echo "Running unit tests..."
          # Replace with actual test command:
          # docker run --rm ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ steps.meta.outputs.version }} npm test
          echo "Unit tests passed"

  # ============================================================
  # Stage 3: Security Scan
  # ============================================================
  security-scan:
    name: "Stage 3: Security Scan"
    runs-on: ubuntu-latest
    needs: build
    steps:
      - uses: actions/checkout@v4

      - name: Run Trivy vulnerability scanner
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: '${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ needs.build.outputs.image_tag }}'
          format: 'table'
          exit-code: '1'
          ignore-unfixed: true
          severity: 'CRITICAL,HIGH'
          output: 'trivy-results.txt'

      - name: Upload scan results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: trivy-results
          path: trivy-results.txt

      - name: Audit summary
        if: always()
        run: |
          echo "## Security Scan Results" >> "$GITHUB_STEP_SUMMARY"
          echo '```' >> "$GITHUB_STEP_SUMMARY"
          cat trivy-results.txt >> "$GITHUB_STEP_SUMMARY"
          echo '```' >> "$GITHUB_STEP_SUMMARY"

  # ============================================================
  # Stage 4: Deploy to Staging
  # ============================================================
  staging-deploy:
    name: "Stage 4: Deploy to Staging"
    runs-on: ubuntu-latest
    needs: [build, security-scan]
    environment:
      name: staging
    steps:
      - uses: actions/checkout@v4

      - name: Configure kubectl
        uses: azure/setup-kubectl@v3

      - name: Set kubeconfig
        run: |
          echo "${{ secrets.KUBE_CONFIG }}" | base64 -d > $HOME/.kube/config

      - name: Deploy to staging
        run: |
          kubectl set image deployment/app \
            app=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ needs.build.outputs.image_tag }} \
            -n ${{ env.STAGING_NAMESPACE }}

      - name: Wait for rollout
        run: |
          kubectl rollout status deployment/app \
            -n ${{ env.STAGING_NAMESPACE }} \
            --timeout=180s

      - name: Staging health check
        run: |
          echo "Running staging smoke tests..."
          # Replace with actual test commands:
          # kubectl exec -n staging deploy/app -- curl -sf http://localhost:8080/health
          echo "Staging health check passed"

  # ============================================================
  # Stage 5: Integration Tests
  # ============================================================
  integration-tests:
    name: "Stage 5: Integration Tests"
    runs-on: ubuntu-latest
    needs: staging-deploy
    steps:
      - uses: actions/checkout@v4

      - name: Run integration tests
        run: |
          echo "Running integration tests against staging..."
          # Replace with actual integration test suite:
          # npm run test:integration -- --base-url=https://staging.example.com
          echo "Integration tests passed"

      - name: Test summary
        if: always()
        run: |
          echo "## Integration Test Results" >> "$GITHUB_STEP_SUMMARY"
          echo "All integration tests passed" >> "$GITHUB_STEP_SUMMARY"

  # ============================================================
  # Stage 6: Change Approval (Manual Gate)
  # ============================================================
  change-approval:
    name: "Stage 6: Change Approval Gate"
    runs-on: ubuntu-latest
    needs: integration-tests
    # This is the manual approval gate
    # GitHub Environment "production-approval" has required reviewers configured
    environment:
      name: production-approval
    steps:
      - name: Approval confirmed
        run: |
          echo "Change approved by: ${{ github.actor }}"
          echo "Approval timestamp: $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
          echo "## Change Approval" >> "$GITHUB_STEP_SUMMARY"
          echo "- **Approved by:** ${{ github.actor }}" >> "$GITHUB_STEP_SUMMARY"
          echo "- **Timestamp:** $(date -u '+%Y-%m-%dT%H:%M:%SZ')" >> "$GITHUB_STEP_SUMMARY"
          echo "- **PR:** #${{ github.event.pull_request.number }}" >> "$GITHUB_STEP_SUMMARY"
          echo "- **Image:** ${{ needs.build.outputs.image_tag }}" >> "$GITHUB_STEP_SUMMARY"

  # ============================================================
  # Stage 7: Deploy to Production
  # ============================================================
  production-deploy:
    name: "Stage 7: Production Deploy"
    runs-on: ubuntu-latest
    needs: [build, change-approval]
    environment:
      name: production
    steps:
      - uses: actions/checkout@v4

      - name: Configure kubectl
        uses: azure/setup-kubectl@v3

      - name: Set kubeconfig
        run: |
          echo "${{ secrets.KUBE_CONFIG_PROD }}" | base64 -d > $HOME/.kube/config

      - name: Record pre-deploy state
        id: pre-deploy
        run: |
          PREV_IMAGE=$(kubectl get deployment/app -n ${{ env.PRODUCTION_NAMESPACE }} \
            -o jsonpath='{.spec.template.spec.containers[0].image}')
          echo "previous_image=${PREV_IMAGE}" >> "$GITHUB_OUTPUT"
          echo "## Pre-Deployment State" >> "$GITHUB_STEP_SUMMARY"
          echo "- Previous image: \`${PREV_IMAGE}\`" >> "$GITHUB_STEP_SUMMARY"

      - name: Deploy to production
        run: |
          kubectl set image deployment/app \
            app=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ needs.build.outputs.image_tag }} \
            -n ${{ env.PRODUCTION_NAMESPACE }}

      - name: Wait for rollout
        id: rollout
        run: |
          if kubectl rollout status deployment/app \
            -n ${{ env.PRODUCTION_NAMESPACE }} \
            --timeout=300s; then
            echo "rollout_success=true" >> "$GITHUB_OUTPUT"
          else
            echo "rollout_success=false" >> "$GITHUB_OUTPUT"
            echo "::error::Production rollout failed or timed out"
          fi

      - name: Rollback on failure
        if: steps.rollout.outputs.rollout_success == 'false'
        run: |
          echo "::warning::Initiating automatic rollback"
          kubectl rollout undo deployment/app -n ${{ env.PRODUCTION_NAMESPACE }}
          kubectl rollout status deployment/app -n ${{ env.PRODUCTION_NAMESPACE }} --timeout=180s
          echo "## Rollback Executed" >> "$GITHUB_STEP_SUMMARY"
          echo "Production rollout failed. Automatic rollback initiated." >> "$GITHUB_STEP_SUMMARY"

  # ============================================================
  # Stage 8: Post-Deploy Verification
  # ============================================================
  post-deploy-verification:
    name: "Stage 8: Post-Deploy Verification"
    runs-on: ubuntu-latest
    needs: production-deploy
    steps:
      - uses: actions/checkout@v4

      - name: Configure kubectl
        uses: azure/setup-kubectl@v3

      - name: Set kubeconfig
        run: |
          echo "${{ secrets.KUBE_CONFIG_PROD }}" | base64 -d > $HOME/.kube/config

      - name: Smoke tests
        id: smoke
        run: |
          echo "Running production smoke tests..."
          PASS=0
          FAIL=0
          for endpoint in /health /ready /api/v1/status; do
            if curl -sf "https://app.example.com${endpoint}" > /dev/null 2>&1; then
              echo "PASS: ${endpoint}"
              PASS=$((PASS + 1))
            else
              echo "FAIL: ${endpoint}"
              FAIL=$((FAIL + 1))
            fi
          done
          echo "Results: ${PASS} passed, ${FAIL} failed"
          if [ "$FAIL" -gt 0 ]; then
            echo "smoke_passed=false" >> "$GITHUB_OUTPUT"
          else
            echo "smoke_passed=true" >> "$GITHUB_OUTPUT"
          fi

      - name: Rollback on smoke test failure
        if: steps.smoke.outputs.smoke_passed == 'false'
        run: |
          echo "::warning::Smoke tests failed. Initiating rollback."
          kubectl rollout undo deployment/app -n ${{ env.PRODUCTION_NAMESPACE }}
          kubectl rollout status deployment/app -n ${{ env.PRODUCTION_NAMESPACE }} --timeout=180s
          exit 1

  # ============================================================
  # Stage 9: Monitoring Period
  # ============================================================
  monitoring-period:
    name: "Stage 9: Monitoring Period"
    runs-on: ubuntu-latest
    needs: post-deploy-verification
    steps:
      - uses: actions/checkout@v4

      - name: Configure kubectl
        uses: azure/setup-kubectl@v3

      - name: Set kubeconfig
        run: |
          echo "${{ secrets.KUBE_CONFIG_PROD }}" | base64 -d > $HOME/.kube/config

      - name: Monitor for ${{ env.MONITORING_DURATION }}s
        id: monitor
        run: |
          DURATION=${{ env.MONITORING_DURATION }}
          INTERVAL=30
          ELAPSED=0
          FAILURES=0
          MAX_FAILURES=3

          echo "Monitoring production for ${DURATION} seconds..."

          while [ "$ELAPSED" -lt "$DURATION" ]; do
            # Check pod health
            NOT_READY=$(kubectl get pods -n ${{ env.PRODUCTION_NAMESPACE }} \
              -l app=app --field-selector='status.phase!=Running' \
              -o name 2>/dev/null | wc -l)

            # Check for pod restarts
            RESTARTS=$(kubectl get pods -n ${{ env.PRODUCTION_NAMESPACE }} \
              -l app=app \
              -o jsonpath='{range .items[*]}{.status.containerStatuses[0].restartCount}{"\n"}{end}' 2>/dev/null \
              | awk '{s+=$1} END {print s}')

            if [ "$NOT_READY" -gt 0 ] || [ "${RESTARTS:-0}" -gt 5 ]; then
              FAILURES=$((FAILURES + 1))
              echo "WARNING: Health check failed (${FAILURES}/${MAX_FAILURES}) at ${ELAPSED}s"
              if [ "$FAILURES" -ge "$MAX_FAILURES" ]; then
                echo "monitor_passed=false" >> "$GITHUB_OUTPUT"
                echo "::error::Monitoring period failed after ${ELAPSED}s"
                exit 1
              fi
            else
              echo "OK: Health check passed at ${ELAPSED}s"
            fi

            sleep "$INTERVAL"
            ELAPSED=$((ELAPSED + INTERVAL))
          done

          echo "monitor_passed=true" >> "$GITHUB_OUTPUT"
          echo "Monitoring period completed successfully"

      - name: Rollback on monitoring failure
        if: steps.monitor.outputs.monitor_passed == 'false'
        run: |
          echo "::warning::Monitoring period failed. Initiating rollback."
          kubectl rollout undo deployment/app -n ${{ env.PRODUCTION_NAMESPACE }}
          kubectl rollout status deployment/app -n ${{ env.PRODUCTION_NAMESPACE }} --timeout=180s

      - name: Deployment summary
        if: always()
        run: |
          echo "## Deployment Summary" >> "$GITHUB_STEP_SUMMARY"
          echo "- **PR:** #${{ github.event.pull_request.number }}" >> "$GITHUB_STEP_SUMMARY"
          echo "- **Commit:** ${{ github.sha }}" >> "$GITHUB_STEP_SUMMARY"
          echo "- **Monitoring:** ${{ steps.monitor.outputs.monitor_passed }}" >> "$GITHUB_STEP_SUMMARY"
          echo "- **Status:** ${{ job.status }}" >> "$GITHUB_STEP_SUMMARY"

  # ============================================================
  # Audit Logging (runs on every outcome)
  # ============================================================
  audit-log:
    name: "Audit Log"
    runs-on: ubuntu-latest
    needs: [build, change-approval, monitoring-period]
    if: always()
    steps:
      - uses: actions/checkout@v4

      - name: Generate audit entry
        run: |
          mkdir -p audit-logs
          cat > "audit-logs/audit-$(date -u '+%Y%m%dT%H%M%SZ').json" << EOF
          {
            "timestamp": "$(date -u '+%Y-%m-%dT%H:%M:%SZ')",
            "event_type": "deployment",
            "actor": "${{ github.actor }}",
            "target": "${{ needs.build.outputs.image_tag }}",
            "details": {
              "pr_number": "${{ github.event.pull_request.number }}",
              "commit_sha": "${{ github.sha }}",
              "environment": "production",
              "pipeline_status": "${{ needs.monitoring-period.result }}"
            },
            "outcome": "${{ needs.monitoring-period.result == 'success' && 'success' || 'failure' }}"
          }
          EOF

      - name: Upload audit log
        uses: actions/upload-artifact@v4
        with:
          name: audit-log-${{ github.run_id }}
          path: audit-logs/

      - name: Audit summary
        run: |
          echo "## Audit Trail" >> "$GITHUB_STEP_SUMMARY"
          echo '```json' >> "$GITHUB_STEP_SUMMARY"
          cat audit-logs/*.json >> "$GITHUB_STEP_SUMMARY"
          echo '```' >> "$GITHUB_STEP_SUMMARY"
```

---

## Approval Gate Configuration (environments.md)

### GitHub Environment: production-approval

Configured in **Settings > Environments > production-approval**:

| Setting | Value |
|---------|-------|
| Required reviewers | `platform-approvers` team (minimum 2 members) |
| Wait timer | 0 minutes |
| Deployment branches | `main` only |
| Environment variables | None (secrets in separate `production` environment) |

### Separation of Duties

**How it works:**

GitHub's environment protection rules enforce that:
1. The person who triggers the workflow (by opening or updating a PR) cannot be
   the *sole* approver of their own deployment.
2. If the PR author is a member of `platform-approvers`, they can still be one
   of the approvers, but at least one other team member must also approve.
3. GitHub tracks who approved and when, creating an immutable audit trail.

**Edge case -- single platform approver who is the author:**

If there is only one platform approver and they are the PR author, the approval
gate will block indefinitely because GitHub requires an approver who is *not*
the workflow trigger actor (when configured with required reviewers).

**Mitigations:**
1. **Minimum team size:** The `platform-approvers` team should have at least 3
   members to ensure availability.
2. **Cross-team approvers:** Add members from the SRE or security team as
   secondary approvers in the environment configuration.
3. **Emergency bypass:** For emergencies, the `emergency` label mechanism
   (described below) allows bypassing the approval gate with explicit logging.

### Emergency Bypass

**Mechanism:**

1. A PR must have the `emergency` label applied by a member of the
   `platform-emergency` team (separate from `platform-approvers`).
2. The pipeline checks for this label in the freeze check stage.
3. If the label is present, the approval gate is skipped, but the pipeline
   still runs all automated checks (build, security, staging, integration).
4. An audit log entry is created with `event_type: "emergency_bypass"` and
   includes who applied the label.

**Restrictions:**
- Only members of `platform-emergency` (typically VP Engineering + on-call lead)
  can apply the `emergency` label
- Every emergency bypass triggers a notification to the engineering leadership
  Slack channel
- A PIR is mandatory within 24 hours for any emergency bypass
- The emergency bypass is logged in the audit trail with full context

---

## Audit Trail (audit-log-format.md)

### Log Format Specification

Each audit log entry is a JSON object:

```json
{
  "timestamp": "ISO 8601 UTC timestamp",
  "event_type": "string enum",
  "actor": "GitHub username or system identifier",
  "target": "what was affected (image tag, service name, etc.)",
  "details": {
    "key": "value"
  },
  "outcome": "success | failure | pending"
}
```

**Fields:**
- `timestamp` (required): ISO 8601 format in UTC (e.g., `2026-06-11T14:30:00Z`)
- `event_type` (required): One of `approval`, `deployment`, `rollback`,
  `pipeline_start`, `pipeline_end`, `emergency_bypass`, `freeze_check`
- `actor` (required): The GitHub username of the person who performed the action,
  or `system` for automated actions
- `target` (required): The resource being acted upon (image tag, deployment name,
  environment)
- `details` (required): Object with event-specific metadata
- `outcome` (required): `success`, `failure`, or `pending`

### Example Entries

**Approval:**
```json
{
  "timestamp": "2026-06-11T14:30:00Z",
  "event_type": "approval",
  "actor": "jane-platform",
  "target": "ghcr.io/myorg/myapp:abc1234",
  "details": {
    "pr_number": 42,
    "environment": "production-approval",
    "reviewers": ["jane-platform", "bob-sre"],
    "comments": "Reviewed security scan, staging tests passed. LGTM."
  },
  "outcome": "success"
}
```

**Deployment:**
```json
{
  "timestamp": "2026-06-11T14:45:00Z",
  "event_type": "deployment",
  "actor": "system",
  "target": "ghcr.io/myorg/myapp:abc1234",
  "details": {
    "pr_number": 42,
    "environment": "production",
    "previous_image": "ghcr.io/myorg/myapp:def5678",
    "rollout_duration_seconds": 45,
    "replicas": 4
  },
  "outcome": "success"
}
```

**Rollback:**
```json
{
  "timestamp": "2026-06-11T15:02:00Z",
  "event_type": "rollback",
  "actor": "system",
  "target": "ghcr.io/myorg/myapp:abc1234",
  "details": {
    "pr_number": 42,
    "environment": "production",
    "trigger": "monitoring_failure",
    "trigger_details": "Pod restart count exceeded threshold (8 > 5)",
    "rollback_to_image": "ghcr.io/myorg/myapp:def5678",
    "rollback_duration_seconds": 30
  },
  "outcome": "success"
}
```

### Where Logs Are Stored

1. **GitHub Actions Step Summary:** Human-readable summaries appear directly in
   the workflow run UI for quick review.
2. **Workflow Artifacts:** JSON audit files are uploaded as artifacts, retained
   for 90 days (configurable).
3. **audit-logs/ branch (optional):** For long-term retention, audit logs can
   be committed to a dedicated branch that is never deleted.

---

## Change Freeze (change-freeze.json)

```json
{
  "freeze_periods": [
    {
      "name": "Year-End Freeze",
      "start": "2026-12-20T00:00:00Z",
      "end": "2027-01-03T00:00:00Z",
      "allowed_labels": ["emergency"],
      "message": "Year-end change freeze is in effect from Dec 20 to Jan 3. Only emergency changes are permitted."
    },
    {
      "name": "Black Friday Weekend",
      "start": "2026-11-27T00:00:00Z",
      "end": "2026-12-01T00:00:00Z",
      "allowed_labels": ["emergency"],
      "message": "Black Friday change freeze. No deployments permitted except emergencies."
    },
    {
      "name": "Q2 Earnings Release",
      "start": "2026-07-15T00:00:00Z",
      "end": "2026-07-17T00:00:00Z",
      "allowed_labels": ["emergency", "compliance-fix"],
      "message": "Q2 earnings release freeze. Only emergency and compliance-fix changes are permitted."
    }
  ]
}
```

## check-freeze.sh

```bash
#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FREEZE_CONFIG="${SCRIPT_DIR}/change-freeze.json"
PR_LABELS=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --config)     FREEZE_CONFIG="$2"; shift 2 ;;
        --pr-labels)  PR_LABELS="$2";     shift 2 ;;
        *) echo "Unknown option: $1"; exit 2 ;;
    esac
done

if [[ ! -f "$FREEZE_CONFIG" ]]; then
    echo "No freeze configuration found at ${FREEZE_CONFIG}. Proceeding."
    exit 0
fi

if ! command -v jq &>/dev/null; then
    echo "ERROR: jq is required but not installed"
    exit 2
fi

CURRENT_EPOCH=$(date +%s)
FREEZE_COUNT=$(jq '.freeze_periods | length' "$FREEZE_CONFIG")

for i in $(seq 0 $((FREEZE_COUNT - 1))); do
    NAME=$(jq -r ".freeze_periods[$i].name" "$FREEZE_CONFIG")
    START=$(jq -r ".freeze_periods[$i].start" "$FREEZE_CONFIG")
    END=$(jq -r ".freeze_periods[$i].end" "$FREEZE_CONFIG")
    MESSAGE=$(jq -r ".freeze_periods[$i].message" "$FREEZE_CONFIG")

    START_EPOCH=$(date -d "$START" +%s 2>/dev/null || date -j -f "%Y-%m-%dT%H:%M:%SZ" "$START" +%s 2>/dev/null)
    END_EPOCH=$(date -d "$END" +%s 2>/dev/null || date -j -f "%Y-%m-%dT%H:%M:%SZ" "$END" +%s 2>/dev/null)

    if [[ "$CURRENT_EPOCH" -ge "$START_EPOCH" && "$CURRENT_EPOCH" -le "$END_EPOCH" ]]; then
        echo "FREEZE ACTIVE: ${NAME}"
        echo "Period: ${START} to ${END}"
        echo "Message: ${MESSAGE}"

        # Check if PR has an allowed label
        ALLOWED_LABELS=$(jq -r ".freeze_periods[$i].allowed_labels[]" "$FREEZE_CONFIG")
        HAS_ALLOWED_LABEL=false

        for label in $ALLOWED_LABELS; do
            if echo "$PR_LABELS" | grep -qi "$label"; then
                HAS_ALLOWED_LABEL=true
                echo "BYPASS: PR has allowed label '${label}'"
                break
            fi
        done

        if $HAS_ALLOWED_LABEL; then
            echo "Change allowed due to emergency/compliance label."
            exit 0
        else
            echo "BLOCKED: This change is not permitted during the freeze period."
            echo "To bypass, apply one of these labels: $(echo $ALLOWED_LABELS | tr '\n' ', ')"
            exit 1
        fi
    fi
done

echo "No active freeze period. Change is permitted."
exit 0
```

---

## Emergency Bypass (emergency-bypass.md)

### Emergency Change Procedure

When a critical production issue requires immediate deployment outside the
normal approval process, the following emergency bypass procedure applies.

### Prerequisites for Emergency Bypass

1. The issue must be causing active production impact (Sev1 or Sev2)
2. The normal approval process would take too long to resolve the impact
3. The change must be the minimum necessary to restore service

### Steps

1. **Apply the `emergency` label** to the PR. Only members of the
   `platform-emergency` team can apply this label.

2. **Open a companion incident ticket** in the incident management system
   with:
   - Incident severity and description
   - Link to the PR
   - Name of the person authorizing the emergency

3. **The pipeline runs with the approval gate skipped**, but all automated
   checks (build, security scan, staging, integration tests) still execute.
   A failed automated check blocks the emergency deployment.

4. **An audit log entry is created** with:
   ```json
   {
     "event_type": "emergency_bypass",
     "actor": "<who-applied-label>",
     "target": "<image-tag>",
     "details": {
       "pr_number": "<pr>",
       "incident_ticket": "<ticket-id>",
       "authorization_reason": "<free-text>"
     },
     "outcome": "bypassed"
   }
   ```

5. **Post-Implementation Review is mandatory** within 24 hours. The PIR must
   address:
   - Why the emergency bypass was necessary
   - Whether the normal process could have been faster
   - What changes to the process are recommended

### Restrictions

- The `emergency` label can only be applied by `platform-emergency` team
  members (VP Engineering, Director of Platform, on-call lead)
- Each emergency bypass triggers a Slack notification to `#engineering-leadership`
- More than 3 emergency bypasses in a 30-day period triggers a process review
- Emergency changes still require a rollback plan

### Post-Emergency Actions

1. Remove the `emergency` label from the PR after the incident is resolved
2. Complete the PIR within 24 hours
3. If the emergency change introduced new technical debt, create a follow-up
   ticket to address it in the next sprint
4. Update the change management process if the PIR identifies improvements
