# Solution 05: Error Budget Policy for Deployment Gating

## Part A -- Policy tiers

| Tier | Budget Remaining | Deployment Policy | Required Approvals | Engineering Action |
|------|-----------------|-------------------|-------------------|-------------------|
| Green | > 25% | Deploy freely within normal process | None (standard review) | Continue normal cadence |
| Yellow | 10-25% | Deploy with caution | Tech lead + SRE on-call | Investigate root causes, prioritize reliability work |
| Red | < 10% or violated | Non-critical deploys halted | VP Engineering approval | Incident review, reliability sprint, postmortem |

**Rationale for thresholds**:

- **25%**: When 75% of the budget is consumed, there is still a comfortable
  margin, but the team should start paying attention. This is the "heads up"
  signal.
- **10%**: When 90% of the budget is consumed, any further incident could
  violate the SLO. Deployments carry risk (new code can cause errors), so
  they should be gated.
- **0%**: The SLO is violated. The team has failed to meet its reliability
  commitment. All non-critical work stops until the situation is resolved.

**Why this works**:

- The tiers are progressive: each level adds friction to the deployment
  process.
- The thresholds are based on the remaining budget, not the consumed budget,
  which is more intuitive ("you have 15% left" vs. "you consumed 85%").
- The actions are concrete: specific approval requirements, not vague
  guidelines.

**Common mistakes**:
- Making all tiers "deploy with caution" (no differentiation)
- Not specifying who can approve (leads to approval paralysis)
- Setting thresholds too tight (e.g., yellow at 50%) which causes constant
  friction even when the SLO is healthy

---

## Part B -- Policy check script

```python
#!/usr/bin/env python3
"""check_budget_policy.py

Queries Prometheus for the current error budget status and determines
the deployment policy tier.
"""

import argparse
import json
import sys
from datetime import datetime, timezone

import requests


def get_error_budget_remaining(
    prometheus_url: str,
    service: str,
    slo_target: float,
    window_days: int,
) -> float:
    """Query Prometheus for the remaining error budget as a percentage.

    Args:
        prometheus_url: Base URL of the Prometheus server.
        service: The job/service name to query.
        slo_target: The SLO target as a percentage (e.g., 99.9).
        window_days: The SLO measurement window in days.

    Returns:
        Remaining error budget as a percentage (0-100).
    """
    allowed_error_rate = 1 - (slo_target / 100)

    # PromQL query for budget remaining
    query = (
        f'clamp_min('
        f'  (1 - ('
        f'    avg_over_time('
        f'      job:http_errors:ratio5m{{job="{service}"}}'
        f'      [{window_days}d]'
        f'    ) / {allowed_error_rate}'
        f'  )) * 100,'
        f'  0'
        f')'
    )

    response = requests.get(
        f"{prometheus_url}/api/v1/query",
        params={"query": query},
        timeout=30,
    )
    response.raise_for_status()

    data = response.json()
    if not data["data"]["result"]:
        raise ValueError(
            f"No data returned for service '{service}'. "
            f"Check that the recording rule job:http_errors:ratio5m exists "
            f"and the job label matches."
        )

    return float(data["data"]["result"][0]["value"][1])


def determine_tier(budget_remaining_pct: float) -> dict:
    """Determine the deployment policy tier based on budget remaining.

    Args:
        budget_remaining_pct: Remaining error budget as a percentage (0-100).

    Returns:
        Dictionary with tier information.
    """
    if budget_remaining_pct > 25:
        return {
            "tier": "green",
            "deployment_allowed": True,
            "required_approvals": [],
            "recommended_action": "Continue normal deployment cadence.",
        }
    elif budget_remaining_pct > 10:
        return {
            "tier": "yellow",
            "deployment_allowed": True,
            "required_approvals": ["tech_lead", "sre_oncall"],
            "recommended_action": (
                "Deploy with caution. Investigate root causes of recent "
                "errors. Prioritize reliability improvements."
            ),
        }
    else:
        return {
            "tier": "red",
            "deployment_allowed": False,
            "required_approvals": ["vp_engineering"],
            "recommended_action": (
                "Non-critical deployments are halted. Conduct incident "
                "review. Begin reliability sprint. Schedule postmortem."
            ),
        }


def build_policy_output(
    service: str,
    slo_target: float,
    window_days: int,
    budget_remaining_pct: float,
    tier_info: dict,
) -> dict:
    """Build the complete policy output document."""
    # Calculate remaining minutes
    total_window_minutes = window_days * 24 * 60
    allowed_error_rate = 1 - (slo_target / 100)
    total_budget_minutes = total_window_minutes * allowed_error_rate
    remaining_minutes = total_budget_minutes * (budget_remaining_pct / 100)

    return {
        "service": service,
        "slo_target": slo_target,
        "window_days": window_days,
        "budget_remaining_pct": round(budget_remaining_pct, 2),
        "budget_remaining_minutes": round(remaining_minutes, 2),
        "total_budget_minutes": round(total_budget_minutes, 2),
        **tier_info,
        "checked_at": datetime.now(timezone.utc).isoformat(),
    }


def main():
    parser = argparse.ArgumentParser(
        description="Check error budget and determine deployment policy tier."
    )
    parser.add_argument(
        "--prometheus-url",
        required=True,
        help="Prometheus server URL (e.g., http://prometheus:9090)",
    )
    parser.add_argument(
        "--service",
        required=True,
        help="Service/job name to check",
    )
    parser.add_argument(
        "--slo-target",
        type=float,
        required=True,
        help="SLO target as percentage (e.g., 99.9)",
    )
    parser.add_argument(
        "--window-days",
        type=int,
        default=30,
        help="SLO measurement window in days (default: 30)",
    )
    parser.add_argument(
        "--fail-on-red",
        action="store_true",
        help="Exit with code 1 if tier is red (for CI/CD gating)",
    )

    args = parser.parse_args()

    try:
        budget_remaining = get_error_budget_remaining(
            prometheus_url=args.prometheus_url,
            service=args.service,
            slo_target=args.slo_target,
            window_days=args.window_days,
        )
    except (requests.RequestException, ValueError) as e:
        print(json.dumps({"error": str(e)}), file=sys.stderr)
        sys.exit(2)

    tier_info = determine_tier(budget_remaining)
    output = build_policy_output(
        service=args.service,
        slo_target=args.slo_target,
        window_days=args.window_days,
        budget_remaining_pct=budget_remaining,
        tier_info=tier_info,
    )

    print(json.dumps(output, indent=2))

    if args.fail_on_red and tier_info["tier"] == "red":
        sys.exit(1)


if __name__ == "__main__":
    main()
```

**Requirements** (`requirements.txt`):

```
requests>=2.28.0
```

**Usage examples**:

```bash
# Basic usage
python check_budget_policy.py \
  --prometheus-url http://prometheus:9090 \
  --service my-api \
  --slo-target 99.9

# With CI/CD gating
python check_budget_policy.py \
  --prometheus-url http://prometheus:9090 \
  --service my-api \
  --slo-target 99.9 \
  --fail-on-red

# Custom window
python check_budget_policy.py \
  --prometheus-url http://prometheus:9090 \
  --service my-api \
  --slo-target 99.95 \
  --window-days 28
```

**Why this works**:

- The Prometheus HTTP API `/api/v1/query` endpoint returns instant query
  results.
- The `clamp_min(..., 0)` in the PromQL prevents negative values.
- The script separates concerns: querying, tier determination, and output
  formatting.
- The `--fail-on-red` flag enables simple CI/CD integration without needing
  to parse JSON.

**Common mistakes**:
- Not handling the case where Prometheus returns no results (service not
  found)
- Hardcoding the SLO target in the query instead of parameterizing it
- Not setting a timeout on the HTTP request (can hang indefinitely)

---

## Part C -- CI/CD integration (GitHub Actions)

```yaml
name: Deploy with SLO Gate

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  PROMETHEUS_URL: http://prometheus.monitoring.svc.cluster.local:9090
  SERVICE_NAME: my-api
  SLO_TARGET: 99.9

jobs:
  # ── Step 1: Check error budget ──
  check-budget:
    runs-on: ubuntu-latest
    outputs:
      tier: ${{ steps.budget.outputs.tier }}
      budget_remaining: ${{ steps.budget.outputs.budget_remaining }}
      deployment_allowed: ${{ steps.budget.outputs.deployment_allowed }}
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: "3.11"

      - name: Install dependencies
        run: pip install requests

      - name: Check error budget
        id: budget
        run: |
          RESULT=$(python check_budget_policy.py \
            --prometheus-url "$PROMETHEUS_URL" \
            --service "$SERVICE_NAME" \
            --slo-target "$SLO_TARGET")

          TIER=$(echo "$RESULT" | jq -r '.tier')
          BUDGET=$(echo "$RESULT" | jq -r '.budget_remaining_pct')
          ALLOWED=$(echo "$RESULT" | jq -r '.deployment_allowed')

          echo "tier=$TIER" >> "$GITHUB_OUTPUT"
          echo "budget_remaining=$BUDGET" >> "$GITHUB_OUTPUT"
          echo "deployment_allowed=$ALLOWED" >> "$GITHUB_OUTPUT"

          echo "### SLO Budget Check" >> "$GITHUB_STEP_SUMMARY"
          echo "| Field | Value |" >> "$GITHUB_STEP_SUMMARY"
          echo "|-------|-------|" >> "$GITHUB_STEP_SUMMARY"
          echo "| Service | $SERVICE_NAME |" >> "$GITHUB_STEP_SUMMARY"
          echo "| Budget Remaining | ${BUDGET}% |" >> "$GITHUB_STEP_SUMMARY"
          echo "| Tier | ${TIER} |" >> "$GITHUB_STEP_SUMMARY"
          echo "| Deploy Allowed | ${ALLOWED} |" >> "$GITHUB_STEP_SUMMARY"

      - name: Fail if budget exhausted
        if: steps.budget.outputs.tier == 'red'
        run: |
          echo "::error::Error budget exhausted (tier: red). Non-critical deployments are blocked."
          echo "See the error budget policy for exception procedures."
          exit 1

  # ── Step 2: Run tests ──
  test:
    needs: check-budget
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Run tests
        run: |
          echo "Running test suite..."
          # Your test commands here

  # ── Step 3: Deploy (only if budget allows) ──
  deploy:
    needs: [check-budget, test]
    if: needs.check-budget.outputs.tier != 'red'
    runs-on: ubuntu-latest
    environment:
      name: production
      url: https://api.example.com
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Check for yellow tier approval
        if: needs.check-budget.outputs.tier == 'yellow'
        run: |
          echo "::warning::Error budget is in yellow tier (${needs.check-budget.outputs.budget_remaining}% remaining)."
          echo "This deployment requires tech lead and SRE on-call approval."
          echo "The environment protection rules will enforce this."

      - name: Deploy to production
        run: |
          echo "Deploying to production..."
          echo "Budget tier: ${{ needs.check-budget.outputs.tier }}"
          echo "Budget remaining: ${{ needs.check-budget.outputs.budget_remaining }}%"
          # Your deployment commands here
```

**Environment protection rules** (configured in GitHub repo settings):

```yaml
# In GitHub > Settings > Environments > production
# Add required reviewers for the "yellow" tier:
# - tech-lead-user
# - sre-oncall-user
# These reviewers are only required when the workflow requests them.
```

**Why this works**:

- The `check-budget` job runs first and outputs the tier to subsequent jobs.
- The `deploy` job uses `if:` to skip when the tier is red.
- The yellow tier uses GitHub environment protection rules to require
  approvals without blocking the workflow entirely.
- The `GITHUB_STEP_SUMMARY` provides a visual summary in the Actions UI.

**Common mistakes**:
- Not using `needs:` to create the dependency chain
- Forgetting to set `outputs:` on the check-budget job
- Not configuring environment protection rules (approvals are not enforced)
- Hardcoding the Prometheus URL instead of using secrets/variables

---

## Part D -- Exception handling

### Scenario 1: Security patch during budget exhaustion

**Immediate action**:

Deploy the security patch. Security vulnerabilities take precedence over
error budget policy. The policy is a guideline, not an immutable law.

**Process**:

1. Document the exception in the deployment ticket:
   - Link to the CVE
   - Note the current budget status
   - Get verbal approval from the engineering manager (do not wait for
     formal approval workflow)
2. Deploy the patch with enhanced monitoring:
   - Deploy to a canary first (10% traffic)
   - Monitor error rates for 15 minutes before full rollout
   - Have rollback ready
3. After deployment:
   - Update the error budget policy document with the exception
   - Add a comment to the SLO dashboard noting the exception

**Prevention**:

- Maintain a separate SLO for security response time
- Pre-authorize security patches in the error budget policy document
- Keep security patches small and well-tested to minimize deployment risk

### Scenario 2: Red tier for 5+ days

**Immediate action**:

Escalate to engineering leadership and declare a reliability incident.

**Additional actions beyond blocking deploys**:

1. **Day 1-2 of red**: Block feature work. Only bug fixes and reliability
   improvements are allowed.
2. **Day 3-5**: Conduct a formal incident review. Identify the root cause
   of the sustained error budget consumption.
3. **Day 5+**:
   - Allocate 50% of engineering capacity to reliability work
   - Cancel or postpone upcoming feature launches
   - Schedule a postmortem with action items
   - Consider reducing the SLO target temporarily (with stakeholder approval)
     if the current target is unrealistic

**Key principle**: The error budget policy exists to force tradeoff decisions.
If the budget is exhausted for 5 days, the team has been making too many
reliability tradeoffs in favor of feature velocity. The policy must restore
balance.

### Scenario 3: Monitoring failure (false positive)

**Immediate action**:

1. Verify the monitoring failure by checking:
   - Prometheus server health and uptime
   - Recording rule evaluation status
   - Actual user-facing error rates from application logs or APM
2. If confirmed as a monitoring failure:
   - Acknowledge the false positive in the alert system
   - Deploy as normal (the budget is not actually exhausted)
   - File a ticket to fix the monitoring infrastructure

**Process**:

1. Document the monitoring failure:
   - Root cause (e.g., Prometheus disk full, recording rule syntax error)
   - Duration of the data gap
   - Impact on SLO calculations
2. Adjust the SLO calculation to exclude the monitoring failure period:
   - Add an annotation to the SLO dashboard
   - Update the recording rule to handle gaps gracefully

**Prevention**:

- Monitor the monitoring system (meta-monitoring):
  - Prometheus server uptime
  - Recording rule evaluation success rate
  - Data freshness (time since last scrape)
- Use `absent()` alerts to detect missing metrics:
  ```promql
  alert: MetricMissing
  expr: absent(job:http_errors:ratio5m)
  for: 5m
  ```
- Implement dead man's switch: a metric that is always 1, and alerts when
  it is absent:
  ```promql
  # Recording rule that always produces 1
  - record: job:slo:dead_mans_switch
    expr: 1

  # Alert when the metric disappears
  - alert: SLOMonitoringDown
    expr: absent(job:slo:dead_mans_switch)
    for: 5m
  ```

---

## Part E -- OPA Rego policy

```rego
package slo.deployment_policy

import future.keywords.in

# Default: deny all deployments
default allow = false

# ── Tier determination ──

tier = "green" {
    input.budget_remaining_pct > 25
}

tier = "yellow" {
    input.budget_remaining_pct > 10
    input.budget_remaining_pct <= 25
}

tier = "red" {
    input.budget_remaining_pct <= 10
}

# ── Allow rules (any matching rule allows the deployment) ──

# Green tier: everything allowed
allow {
    tier == "green"
}

# Yellow tier: allowed with approval, except experiments
allow {
    tier == "yellow"
    input.has_approval == true
    input.deployment_type != "experiment"
}

# Red tier: only hotfixes allowed (always)
allow {
    tier == "red"
    input.deployment_type == "hotfix"
}

# Red tier: infrastructure with VP approval
allow {
    tier == "red"
    input.deployment_type == "infrastructure"
    input.has_approval == true
    input.approver_role == "vp_engineering"
}

# ── Deny reasons ──

deny_reason = "Error budget exhausted. Only hotfixes are allowed." {
    tier == "red"
    not allow
}

deny_reason = "Yellow tier requires approval. Request tech lead and SRE on-call sign-off." {
    tier == "yellow"
    not allow
    input.deployment_type != "experiment"
}

deny_reason = "Experiments are blocked in yellow tier to protect error budget." {
    tier == "yellow"
    not allow
    input.deployment_type == "experiment"
}

deny_reason = "Unknown deployment type." {
    not allow
    not deny_reason
}

# ── Policy metadata ──

policy_version = "1.0.0"

policy_summary = {
    "tier": tier,
    "allowed": allow,
    "reason": reason,
} {
    allow
    reason := "Deployment approved."
}

policy_summary = {
    "tier": tier,
    "allowed": allow,
    "reason": reason,
} {
    not allow
    reason := deny_reason
}
```

**Test cases** (Rego test file):

```rego
package slo.deployment_policy

test_green_allows_feature {
    allow with input as {
        "budget_remaining_pct": 50,
        "deployment_type": "feature",
        "has_approval": false,
    }
}

test_yellow_blocks_experiment {
    not allow with input as {
        "budget_remaining_pct": 20,
        "deployment_type": "experiment",
        "has_approval": false,
    }
}

test_yellow_allows_approved_feature {
    allow with input as {
        "budget_remaining_pct": 20,
        "deployment_type": "feature",
        "has_approval": true,
    }
}

test_red_blocks_feature {
    not allow with input as {
        "budget_remaining_pct": 5,
        "deployment_type": "feature",
        "has_approval": false,
    }
}

test_red_allows_hotfix {
    allow with input as {
        "budget_remaining_pct": 0,
        "deployment_type": "hotfix",
        "has_approval": false,
    }
}

test_red_allows_infrastructure_with_vp_approval {
    allow with input as {
        "budget_remaining_pct": 5,
        "deployment_type": "infrastructure",
        "has_approval": true,
        "approver_role": "vp_engineering",
    }
}

test_red_blocks_infrastructure_without_approval {
    not allow with input as {
        "budget_remaining_pct": 5,
        "deployment_type": "infrastructure",
        "has_approval": false,
    }
}
```

**Integration with CI/CD**:

```bash
# Evaluate the policy using the OPA CLI
echo '{
  "budget_remaining_pct": 15,
  "deployment_type": "feature",
  "has_approval": true
}' | opa eval --data policy.rego --input - 'data.slo.deployment_policy'

# Use the decision in a script
DECISION=$(echo "$INPUT" | opa eval \
  --data policy.rego \
  --input - \
  --format raw \
  'data.slo.deployment_policy.allow')

if [ "$DECISION" = "true" ]; then
    echo "Deployment approved"
    exit 0
else
    REASON=$(echo "$INPUT" | opa eval \
      --data policy.rego \
      --input - \
      --format raw \
      'data.slo.deployment_policy.deny_reason')
    echo "Deployment denied: $REASON"
    exit 1
fi
```

**Why this works**:

- The policy uses a default-deny pattern: deployments are blocked unless
  explicitly allowed.
- Each allow rule is independent (logical OR), making it easy to add new
  exceptions.
- The `deny_reason` provides human-readable feedback for debugging and
  audit trails.
- Test cases verify the policy behavior for each tier and deployment type
  combination.
- Hotfixes are always allowed, which is critical for incident response.

**Common mistakes**:
- Not having a default deny (deployments are allowed when no rule matches)
- Making the policy too complex (keep it to 3-5 rules per tier)
- Not testing the policy (Rego policies can have subtle logic errors)
- Forgetting to version the policy (teams need to know which version is
  active)
