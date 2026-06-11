# Exercise 02: Build a Canary with Argo Rollouts

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Implement a canary deployment using Argo Rollouts with automatic
progression and metric-based analysis. This exercise walks you through
the Argo Rollouts configuration that automates the canary process.

## Scenario

You are deploying `search-service` using Argo Rollouts. The canary
process should:

1. Start at 20% traffic
2. Pause for 2 minutes to observe
3. Check if error rate is below 1%
4. If healthy, increase to 40%, then 60%, then 80%, then 100%
5. If unhealthy at any step, automatically roll back

## Tasks

### Part A: Write the Rollout Resource

Create an Argo Rollouts `Rollout` resource that defines the canary
strategy with the steps described above.

<details>
<summary>Hint</summary>

The Rollout resource replaces a standard Deployment. The `strategy.canary`
field defines the steps:

```yaml
strategy:
  canary:
    steps:
      - setWeight: 20
      - pause: {duration: 2m}
      - setWeight: 40
      - pause: {duration: 2m}
      - setWeight: 60
      - pause: {duration: 2m}
      - setWeight: 80
      - pause: {duration: 2m}
```

</details>

### Part B: Write the Analysis Template

Create an `AnalysisTemplate` that queries Prometheus for the error rate
of the canary version. The analysis should fail if the error rate
exceeds 1%.

<details>
<summary>Hint</summary>

Use a Prometheus query that calculates the ratio of 5xx responses to
total responses:

```yaml
metrics:
  - name: error-rate
    interval: 1m
    successCondition: result[0] <= 0.01
    failureLimit: 3
    provider:
      prometheus:
        address: http://prometheus:9090
        query: |
          sum(rate(http_requests_total{service="search-service",
            status=~"5.."}[5m])) /
          sum(rate(http_requests_total{service="search-service"}[5m]))
```

</details>

### Part C: Integrate Analysis with Rollout

Modify the Rollout resource to include analysis at each weight step.
The analysis should run automatically after each weight increase.

<details>
<summary>Hint</summary>

Add an `analysis` step after each `setWeight`:

```yaml
steps:
  - setWeight: 20
  - pause: {duration: 2m}
  - analysis:
      templates:
        - templateName: error-rate
  - setWeight: 40
  - pause: {duration: 2m}
  - analysis:
      templates:
        - templateName: error-rate
```

</details>

### Part D: Write the Monitoring Commands

Write the `kubectl` commands to:
1. Watch the canary progress in real-time
2. Manually promote the canary to the next step
3. Abort the canary and roll back

<details>
<summary>Hint</summary>

```bash
# Watch progress
kubectl argo rollouts get rollout search-service --watch

# Promote to next step
kubectl argo rollouts promote search-service

# Abort and rollback
kubectl argo rollouts abort search-service
```

</details>

## Success Criteria

- [ ] Rollout resource defines 5 canary weight steps
- [ ] Analysis template queries Prometheus for error rate
- [ ] Analysis runs after each weight step with automatic failure handling
- [ ] You can watch, promote, and abort the canary with kubectl commands
- [ ] The canary automatically rolls back if error rate exceeds 1%

## What You Should Understand After This Exercise

Argo Rollouts automates the canary process by defining weight steps and
analysis templates. The controller manages traffic splitting, metric
analysis, and automatic promotion or rollback. You define the policy;
the controller enforces it.
