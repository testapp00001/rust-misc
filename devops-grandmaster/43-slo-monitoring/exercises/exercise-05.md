# Exercise 05: Error Budget Policy for Deployment Gating

## Objective

Design an error budget policy that connects SLO monitoring to real engineering
decisions, specifically gating deployments based on remaining error budget.
This closes the loop between observability and engineering workflow.

## Background

An error budget policy is a document (and automation) that defines what
happens when the error budget is healthy, at risk, or exhausted. It turns
the abstract concept of "SLO compliance" into concrete actions:

- Deploy freely when budget is healthy
- Require extra approvals when budget is at risk
- Halt non-critical deployments when budget is exhausted

## Instructions

### Part A -- Define the policy tiers

Design a three-tier error budget policy. For each tier, define:

1. **Trigger condition** (budget remaining percentage or burn rate)
2. **Deployment policy** (what is allowed)
3. **Required approvals** (who must sign off)
4. **Engineering action** (what the team must do)

| Tier | Budget Remaining | Deployment Policy | Approvals | Action |
|------|-----------------|-------------------|-----------|--------|
| Green | ? | ? | ? | ? |
| Yellow | ? | ? | ? | ? |
| Red | ? | ? | ? | ? |

Fill in the table with reasonable values for a production API service.

### Part B -- Implement the policy in code

Write a Python script (`check_budget_policy.py`) that:

1. Queries a Prometheus endpoint for the current error budget
2. Determines which tier the service is in
3. Outputs a JSON document with the policy decision

The script should accept these arguments:

```bash
python check_budget_policy.py \
  --prometheus-url http://prometheus:9090 \
  --service my-api \
  --slo-target 99.9 \
  --window-days 30
```

Expected output:

```json
{
  "service": "my-api",
  "slo_target": 99.9,
  "budget_remaining_pct": 45.2,
  "budget_remaining_minutes": 19.5,
  "tier": "green",
  "deployment_allowed": true,
  "required_approvals": [],
  "recommended_action": "Continue normal deployment cadence",
  "checked_at": "2024-01-15T10:30:00Z"
}
```

### Part C -- CI/CD integration

Design the CI/CD pipeline integration for the error budget policy. You are
using GitHub Actions.

Write the GitHub Actions workflow that:

1. Runs the budget check before deploying to production
2. Blocks the deployment if the tier is "red"
3. Requires an extra approval comment if the tier is "yellow"
4. Proceeds automatically if the tier is "green"

Write the workflow YAML file.

### Part D -- Exception handling

Real-world policies need exceptions. Design the exception process for these
scenarios:

1. **Security patch**: A critical CVE needs an immediate production deploy,
   but the error budget is exhausted. What do you do?

2. **Feature freeze**: The error budget has been in the red tier for 5 days.
   What additional actions should the team take beyond blocking deploys?

3. **False positive**: The error budget shows exhaustion, but it was caused
   by a monitoring infrastructure failure (Prometheus had gaps), not actual
   user-facing errors. How do you handle this?

For each scenario, describe:
- The immediate action
- The process to follow
- How to prevent it in the future

### Part E -- Policy as code with OPA

Write an OPA (Open Policy Agent) Rego policy that encodes your deployment
gating rules. The policy should:

1. Accept input with `budget_remaining_pct`, `deployment_type`, and
   `has_approval`
2. Allow or deny the deployment
3. Return a reason for the decision

```rego
package slo.deployment_policy

# Your policy here
```

Deployment types to consider:
- `hotfix` (always allowed, even in red)
- `feature` (blocked in red)
- `experiment` (blocked in yellow and red)
- `infrastructure` (requires approval in yellow, blocked in red)

## Success Criteria

- [ ] Policy tiers have clear, measurable trigger conditions
- [ ] Python script correctly queries Prometheus and produces valid JSON
- [ ] GitHub Actions workflow gates deployments based on budget tier
- [ ] Exception handling covers security, operational, and monitoring failure cases
- [ ] OPA Rego policy correctly encodes all tier/deployment-type combinations

## Hints

<details>
<summary>Hint 1: Prometheus query for budget</summary>

```promql
# Error budget consumed percentage
(
  1 - (
    (1 - avg_over_time(job:http_errors:ratio5m[30d]))
    /
    (1 - 0.999)
  )
) * 100
```

When this returns 0, the budget is fully consumed. When it returns 100,
the budget is completely unused.

</details>

<details>
<summary>Hint 2: GitHub Actions conditional deployment</summary>

Use `if:` conditions and the `needs:` keyword:

```yaml
jobs:
  check-budget:
    runs-on: ubuntu-latest
    outputs:
      tier: ${{ steps.check.outputs.tier }}
    steps:
      - id: check
        run: |
          TIER=$(python check_budget_policy.py ...)
          echo "tier=$TIER" >> $GITHUB_OUTPUT

  deploy:
    needs: check-budget
    if: needs.check-budget.outputs.tier != 'red'
    runs-on: ubuntu-latest
    steps:
      # deploy steps
```

</details>

<details>
<summary>Hint 3: OPA policy pattern</summary>

```rego
default allow = false

allow {
    input.budget_remaining_pct > 25
}

allow {
    input.budget_remaining_pct > 10
    input.deployment_type == "hotfix"
}

deny_reason = "Error budget exhausted" {
    not allow
}
```

</details>

<details>
<summary>Hint 4: Python Prometheus query</summary>

Use the `requests` library to query the Prometheus HTTP API:

```python
import requests

def get_error_budget(prometheus_url, service, slo_target, window_days):
    query = f'''
    (1 - (avg_over_time(
        job:http_errors:ratio5m{{job="{service}"}}[{window_days}d]
    ) / {1 - slo_target/100})) * 100
    '''
    response = requests.get(
        f"{prometheus_url}/api/v1/query",
        params={"query": query}
    )
    return float(response.json()["data"]["result"][0]["value"][1])
```

</details>
