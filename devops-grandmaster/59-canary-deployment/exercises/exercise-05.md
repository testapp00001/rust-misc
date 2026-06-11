# Exercise 05: Full Canary Pipeline with Automated Rollback

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Build a complete CI/CD pipeline that implements canary deployment with
automated metric analysis, progressive traffic splitting, and automatic
rollback. This exercise combines pipeline design (Module 57), deployment
strategies (Module 58), and canary patterns into a production-grade system.

## Scenario

You are building the deployment pipeline for `api-service`, a critical
API that serves 50,000 requests per second. The pipeline must:

1. Build and test the new version
2. Deploy the canary at 5% traffic
3. Analyze metrics for 5 minutes
4. If healthy, progressively increase to 25%, 50%, 100%
5. If unhealthy at any step, roll back within 30 seconds
6. Notify the team on Slack at each stage

## Tasks

### Part A: Design the Pipeline Architecture

Draw an ASCII diagram showing the complete pipeline from code push to
full production, including the canary progression and rollback paths.

<details>
<summary>Hint</summary>

The pipeline has these major phases:
1. CI: build -> test -> scan -> push image
2. CD: deploy canary (5%) -> analyze -> promote (25%) -> analyze -> promote
   (50%) -> analyze -> promote (100%)
3. Each analysis step has a rollback path
4. Notifications at each stage transition

</details>

### Part B: Write the Argo Rollouts Configuration

Create the complete Argo Rollouts configuration including:
- Rollout resource with canary steps
- AnalysisTemplate for error rate and latency
- Notification triggers for Slack

<details>
<summary>Hint 1</summary>

Use `setWeight` and `pause` steps with inline `analysis`:

```yaml
steps:
  - setWeight: 5
  - pause: {duration: 2m}
  - analysis:
      templates: [{templateName: canary-analysis}]
      args:
        - name: service-name
          value: api-service
  - setWeight: 25
  - pause: {duration: 3m}
  - analysis:
      templates: [{templateName: canary-analysis}]
  - setWeight: 50
  - pause: {duration: 3m}
  - analysis:
      templates: [{templateName: canary-analysis}]
  - setWeight: 100
```

</details>

<details>
<summary>Hint 2</summary>

For notifications, use Argo Rollouts' notification system:

```yaml
metadata:
  annotations:
    notifications.argoproj.io/subscribe.on-promoted.slack: deployments
    notifications.argoproj.io/subscribe.on-aborted.slack: deployments
```

</details>

### Part C: Write the CI/CD Pipeline Job

Write the GitHub Actions job that:
1. Updates the Rollout with the new image
2. Waits for the canary to complete (or fail)
3. Reports the final status

<details>
<summary>Hint</summary>

Use `kubectl argo rollouts set image` to trigger the canary, then
`kubectl argo rollouts status` to wait for completion:

```yaml
- name: Update canary image
  run: |
    kubectl argo rollouts set image api-service \
      api-service=ghcr.io/${{ github.repository }}:${{ github.sha }}

- name: Wait for canary analysis
  run: |
    kubectl argo rollouts status api-service --timeout=1800s
```

The timeout should be long enough for all analysis steps to complete
(5 weight steps x ~5 minutes each = ~25 minutes).

</details>

### Part D: Design the Rollback Strategy

Design the rollback strategy for each failure mode:
1. Analysis fails at 5% weight
2. Analysis fails at 50% weight
3. The Rollout controller crashes during analysis
4. Prometheus is down during analysis

<details>
<summary>Hint</summary>

1. Analysis at 5%: Argo Rollouts automatically sets weight to 0 (full
   rollback). Only 5% of users were affected.
2. Analysis at 50%: Argo Rollouts automatically sets weight to 0. 50%
   of users were affected -- this is why you analyze at each step.
3. Controller crash: The Rollout has a `progressDeadlineSeconds` that
   triggers automatic abort if the rollout does not progress within
   the deadline.
4. Prometheus down: The analysis metric returns an error, which counts
   as a failure. After `failureLimit` is reached, the canary is aborted.
   Consider adding a fallback metric or increasing `failureLimit` for
   infrastructure issues.

</details>

## Success Criteria

- [ ] ASCII diagram shows complete pipeline with canary progression and rollback
- [ ] Rollout resource defines 4 weight steps with analysis at each
- [ ] Analysis template checks error rate and latency with appropriate thresholds
- [ ] CI/CD job triggers canary and waits for completion with timeout
- [ ] Rollback strategy handles all 4 failure modes

## What You Should Understand After This Exercise

A production canary pipeline is a feedback loop: deploy a small slice,
measure, decide, repeat. Each step is gated by metric analysis. If any
step fails, the blast radius is limited to the current traffic weight.
The pipeline automates the entire process, including rollback, so
deployments are safe by default.
