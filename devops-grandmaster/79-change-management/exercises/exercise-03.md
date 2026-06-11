# Exercise 03: Design a Maintenance Window Procedure

**Type:** Independent
**Duration:** 60-90 minutes

## Objective

Design a complete maintenance window procedure for a production web application.
Your procedure must cover scheduling, communication, pre-maintenance checks,
execution steps, post-maintenance verification, and rollback criteria. The
procedure should be detailed enough that any operations engineer could follow it.

## Context

You are the lead platform engineer at an e-commerce company. The application
runs on Kubernetes across three environments (dev, staging, production). The
production environment serves 50,000 concurrent users during peak hours (10:00
-- 22:00 UTC) and 5,000 during off-peak. You need to apply a major
infrastructure change: upgrading the Kubernetes cluster from version 1.27 to
1.29, which requires node pool upgrades and will involve temporary pod
evictions.

## Instructions

### Part A -- Maintenance Window Scheduling

Define the maintenance window by addressing:

1. **When** -- Choose a specific day and time window. Justify based on the
   traffic pattern provided in the context.
2. **Duration** -- Estimate how long the maintenance will take, including
   buffer time. Explain your reasoning.
3. **Blackout periods** -- Identify periods when maintenance must never occur
   (e.g., sales events, holidays). List at least three.
4. **Frequency** -- How often can this type of maintenance be scheduled? What
   limits the frequency?

### Part B -- Communication Plan

Create a communication plan that includes:

1. **Audience matrix** -- Who needs to be notified (internal teams, external
   customers, management)?
2. **Notification timeline** -- When does each audience receive notifications
   (e.g., 2 weeks, 1 week, 24 hours, 1 hour before)?
3. **Communication channels** -- What channels are used for each audience
   (email, Slack, status page, in-app banner)?
4. **Message templates** -- Write the actual notification messages for:
   - The initial announcement (2 weeks before)
   - The reminder (24 hours before)
   - The "maintenance started" notification
   - The "maintenance completed" notification
5. **Escalation contacts** -- Define who is called if something goes wrong,
   with primary and secondary contacts for each role.

### Part C -- Pre-Maintenance Checklist

Create a checklist of at least 10 items that must be verified before the
maintenance window opens. Categories to cover:

- Cluster health (node status, pod health, resource utilization)
- Backup verification (etcd snapshot, PV backups, database dumps)
- Change artifacts (upgrade manifests tested in staging, rollback plan ready)
- Team readiness (on-call engineer confirmed, access verified)
- External dependencies (third-party services operational, no ongoing incidents)

Each checklist item must have:
- A clear pass/fail criterion
- The person responsible for the check
- What to do if the check fails (proceed, delay, or abort)

### Part D -- Execution Runbook

Write a step-by-step runbook for the cluster upgrade. Each step must include:

1. The exact command or action
2. The expected output or result
3. How long the step should take
4. What to do if the step fails
5. A checkpoint (go/no-go decision point)

Minimum steps:
- Pre-maintenance snapshot/backup
- Drain and upgrade control plane nodes
- Upgrade worker node pools (one pool at a time)
- Verify cluster health after each pool
- Run application smoke tests
- Gradually restore traffic if load balancer was adjusted

### Part E -- Rollback Criteria and Procedure

Define:

1. **Rollback triggers** -- List at least five specific conditions that would
   trigger an immediate rollback. Each trigger must be measurable (e.g., "error
   rate exceeds 5% for 2 consecutive minutes" not "things look bad").
2. **Rollback procedure** -- Step-by-step instructions to return to the
   previous state (Kubernetes 1.27).
3. **Rollback time limit** -- At what point during the maintenance window is
   rollback no longer viable? What is the decision process after that point?
4. **Partial rollback** -- If only one node pool fails, can you roll back just
   that pool? How?

## Success Criteria

- [ ] Maintenance window is justified by traffic analysis
- [ ] Communication plan covers all audiences with specific timelines and channels
- [ ] All four message templates are written and ready to send
- [ ] Pre-maintenance checklist has 10+ items with pass/fail criteria and owners
- [ ] Execution runbook has numbered steps with commands, expected outputs, and timing
- [ ] Rollback triggers are specific and measurable
- [ ] Rollback procedure is complete and tested (at least mentally walked through)
- [ ] The document is detailed enough for another engineer to execute independently

## Hints

<details>
<summary>Hint 1 -- Window Selection</summary>

Look at the traffic pattern: 50,000 users during 10:00-22:00 UTC, 5,000 during
off-peak. The safest window is during the lowest traffic period, which is
roughly 02:00-06:00 UTC. Choose a window that starts and ends outside peak
hours with buffer time.

</details>

<details>
<summary>Hint 2 -- Checklist Organization</summary>

Organize your checklist with clear sections and checkboxes. Consider using a
table format:

```
| # | Check | Criterion | Owner | If Failed |
|---|-------|-----------|-------|-----------|
| 1 | ...   | ...       | ...   | ...       |
```

</details>

<details>
<summary>Hint 3 -- Rollback Triggers</summary>

Think about the signals that indicate the upgrade is causing problems:
- HTTP error rates (5xx responses)
- Latency percentiles (p99 exceeding threshold)
- Pod restart counts
- Node NotReady states
- Failed health checks
- Customer-reported issues

</details>

<details>
<summary>Hint 4 -- kubectl Commands</summary>

Useful commands for the runbook:
- `kubectl get nodes` -- check node versions and status
- `kubectl drain <node> --ignore-daemonsets --delete-emptydir-data` -- evict pods
- `kubectl uncordon <node>` -- mark node as schedulable
- `kubectl get pods --all-namespaces | grep -v Running` -- find unhealthy pods

</details>

## Deliverables

Submit a single file named `maintenance-window-procedure.md` containing all
five parts (A through E). The file should be formatted as a professional
runbook that could be used in a real operations environment.
