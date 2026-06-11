# Exercise 02: Write a Pod CrashLoop Runbook

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Create a complete, production-quality runbook for diagnosing and resolving a Kubernetes pod stuck
in `CrashLoopBackOff`. You will follow a standard runbook template and fill in each section with
realistic commands, decision logic, and verification steps.

## Scenario

You are an SRE on-call. At 03:17 AM, PagerDuty fires an alert:

```
[CRITICAL] Pod CrashLoopBackOff
  Namespace: production
  Pod: payment-service-7b4f9c6d8-xk2mn
  Restart count: 14 (last 23 minutes)
  Last exit code: 137 (OOMKilled)
```

The payment service is degraded. Customers are experiencing failed transactions.
You need a runbook that any on-call engineer can follow to diagnose and resolve this issue.

## Runbook Template

Use this template as the skeleton for your runbook. Fill in every section with concrete content.

```markdown
# Runbook: Pod CrashLoopBackOff

## Metadata
- **ID**: RB-K8S-001
- **Owner**: [team]
- **Last Reviewed**: [date]
- **Severity**: [level]
- **Alert(s)**: [linked alerts]

## Summary
[One paragraph: what is this runbook for?]

## Impact
[What is affected? Who is affected?]

## Prerequisites
[What access, tools, or context does the responder need?]

## Diagnosis
[Step-by-step investigation to identify the root cause]

### Step 1: ...
### Step 2: ...
### Step 3: ...

## Resolution
[Steps to fix the issue -- may have multiple paths based on diagnosis]

### Path A: ...
### Path B: ...
### Path C: ...

## Verification
[How to confirm the issue is resolved]

## Rollback
[What to do if the fix makes things worse]

## Escalation
[When and whom to escalate to]

## Post-Incident
[Follow-up actions after resolution]
```

## Tasks

### Part A: Metadata and Summary

Fill in the metadata section with realistic values. Write a summary that an engineer waking up
at 3 AM can read in 30 seconds and understand what this runbook covers.

<details><summary>Hint</summary>
Include the team that owns the payment-service, the alert name from Prometheus/Grafana that
triggers this runbook, and the severity level (P1/P2/P3).
</details>

### Part B: Diagnosis Steps

Write at least 5 diagnosis steps. Each step should include:
- The exact `kubectl` command to run.
- What to look for in the output.
- A decision point: "If you see X, go to Resolution Path A. If you see Y, go to Resolution Path B."

Cover these common root causes:
1. OOMKilled (memory limit exceeded)
2. Application crash (non-zero exit code, bad config, missing secret)
3. Liveness probe failure
4. Image pull error
5. Init container failure

<details><summary>Hint</summary>
Start with `kubectl describe pod <pod> -n <namespace>` to get the state and events.
Then check `kubectl logs <pod> -n <namespace> --previous` for the previous container's logs.
Use `kubectl get events -n <namespace> --field-selector involvedObject.name=<pod>` for events.
</details>

### Part C: Resolution Paths

For each root cause identified in Part B, write a resolution path with:
- The exact commands to fix the issue.
- Any preconditions (e.g., "you need cluster-admin access").
- The expected outcome after each command.

Include at least 3 distinct resolution paths:

**Path A**: OOMKilled -- increase memory limits or fix the memory leak.
**Path B**: Application crash -- fix configuration, restore missing secrets, or roll back the image.
**Path C**: Liveness probe -- adjust probe settings or fix the application startup time.

<details><summary>Hint</summary>
For OOMKilled: `kubectl set resources deployment/<name> --limits=memory=512Mi`.
For bad config: `kubectl get configmap <name> -o yaml` and compare with expected values.
For liveness probe: check `initialDelaySeconds` in the pod spec.
</details>

### Part D: Verification and Rollback

Write verification steps that prove the issue is resolved:
- How to confirm the pod is running and healthy.
- How to confirm the service is serving traffic.
- How to confirm the error is no longer firing in monitoring.

Write rollback steps for each resolution path:
- What to undo if the fix causes a new problem.
- How to restore the previous state.

<details><summary>Hint</summary>
Verification: `kubectl get pods -n production -l app=payment-service` should show Running with
Ready 1/1. Check the Prometheus alert `CrashLoopBackOff` is resolved. Test with a curl to the
service endpoint.
Rollback: `kubectl rollout undo deployment/payment-service -n production` for image changes.
For resource changes, revert the resource limits to previous values.
</details>

### Part E: Escalation and Post-Incident

Define the escalation path:
- When should the on-call engineer escalate? (e.g., "If not resolved in 15 minutes.")
- Who to escalate to? (e.g., "Escalate to the payment-service team lead.")
- What communication is required? (e.g., "Post status update in #incident channel.")

Define post-incident actions:
- What follow-up is needed after the incident is resolved?
- How should this runbook be updated if a new root cause is discovered?

## Success Criteria

- [ ] The runbook follows the template structure with all sections filled in.
- [ ] Metadata includes a realistic owner, ID, severity, and linked alert.
- [ ] Diagnosis includes at least 5 steps with exact `kubectl` commands.
- [ ] Each diagnosis step has a clear decision point leading to a resolution path.
- [ ] At least 3 resolution paths cover distinct root causes.
- [ ] Resolution steps include exact commands, preconditions, and expected outcomes.
- [ ] Verification steps confirm both pod health and service health.
- [ ] Rollback steps exist for each resolution path.
- [ ] Escalation criteria are specific and time-bound.
- [ ] A new on-call engineer could follow this runbook without additional context.

## What You Should Understand After This Exercise

A good runbook is a safety net for on-call engineers. It must be specific enough to follow under
pressure (exact commands, not vague descriptions), flexible enough to handle multiple root causes
(decision trees, not linear paths), and complete enough to cover the full lifecycle from detection
to verification to rollback. Writing runbooks is a skill that improves with practice and review.
