# Exercise 01: Runbook vs Playbook

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

Understand the fundamental difference between a runbook and a playbook, and correctly classify
real-world scenarios into one category or the other. This distinction is critical for choosing
the right documentation approach when building your incident response library.

## Background

Runbooks and playbooks are both operational documents, but they serve different purposes:

- **Runbook**: A set of deterministic, step-by-step instructions for a specific operational
  task or incident. Runbooks are procedural -- follow steps A, B, C to resolve the issue.
  They are typically used by on-call engineers who may not have deep context about the system.

- **Playbook**: A higher-level strategic document that defines the overall approach to a class
  of incidents. Playbooks include decision trees, escalation paths, communication plans, and
  may reference multiple runbooks. They are used during complex, multi-team incidents.

```
Playbook (Strategic)                    Runbook (Tactical)
+---------------------------+          +---------------------------+
| - Incident classification |          | - Step 1: Run command X   |
| - Team roles & ownership  |          | - Step 2: Check output Y  |
| - Communication plan      |   --->   | - Step 3: If Z, do W      |
| - Escalation matrix       |          | - Step 4: Verify result   |
| - References to runbooks  |          | - Step 5: Close ticket    |
+---------------------------+          +---------------------------+
```

## Tasks

### Part A: Classify the Scenarios

For each of the 10 scenarios below, classify it as requiring either a **Runbook** or a **Playbook**.
Write down your classification and a one-sentence justification for each.

| # | Scenario |
|---|----------|
| 1 | A single pod in Kubernetes is stuck in `CrashLoopBackOff` and needs to be restarted. |
| 2 | A major cloud provider region outage is affecting multiple services, customers are impacted, and engineering, support, and executive teams need to coordinate. |
| 3 | A disk on a database server is at 95% utilization and logs need to be rotated or archived. |
| 4 | A security breach is detected -- customer data may be exposed. Legal, security, engineering, and communications teams must act in concert. |
| 5 | An SSL certificate is about to expire and needs to be renewed and deployed to the load balancer. |
| 6 | A production deployment is failing health checks and needs to be rolled back. |
| 7 | A ransomware attack is in progress across multiple systems. The incident commander must coordinate containment, investigation, recovery, and external communication. |
| 8 | A DNS record needs to be updated to point to a new server IP address. |
| 9 | A data center failover is triggered -- traffic must be rerouted, databases must be promoted, caches must be warmed, and customers must be notified. |
| 10 | A monitoring alert fires for high CPU usage on a single application server. |

<details><summary>Hint</summary>
Ask yourself: "Does this require following a fixed sequence of steps (runbook), or does it require
coordinating multiple teams with decision-making, communication, and escalation (playbook)?"
Single-system, deterministic tasks tend to be runbooks. Multi-team, strategic, ambiguous situations
tend to be playbooks.
</details>

### Part B: Explain Why the Distinction Matters

Write a short paragraph (3-5 sentences) answering this question:

> Why does it matter whether you write a runbook or a playbook for a given scenario? What problems
> arise when you confuse the two?

<details><summary>Hint</summary>
Consider what happens when:
- A complex multi-team incident has only a runbook (no coordination strategy).
- A simple, repeatable task has a full playbook (overhead and confusion for a routine operation).
</details>

### Part C: Boundary Cases

Scenarios 2 and 9 from Part A are complex incidents. For each one, identify:

1. What the **playbook** would contain (strategic layer).
2. At least two **runbooks** that the playbook would reference (tactical layer).

Present your answer in this format:

```
Scenario 2 (Cloud Provider Outage):
  Playbook covers: ...
  Runbook 1: ...
  Runbook 2: ...

Scenario 9 (Data Center Failover):
  Playbook covers: ...
  Runbook 1: ...
  Runbook 2: ...
```

<details><summary>Hint</summary>
A playbook for a cloud provider outage might cover communication cadence, team assignments,
and service priority. The runbooks it references would be specific technical procedures like
"failing over DNS" or "promoting read replicas."
</details>

## Success Criteria

- [ ] All 10 scenarios are correctly classified as Runbook or Playbook.
- [ ] Each classification includes a clear justification.
- [ ] Part B demonstrates understanding of why the distinction matters operationally.
- [ ] Part C correctly separates strategic (playbook) and tactical (runbook) concerns for complex incidents.
- [ ] You can articulate the difference in your own words without referring to notes.

## What You Should Understand After This Exercise

A runbook is a step-by-step procedure for a specific, repeatable task -- it tells you *how* to do
something. A playbook is a strategic coordination document for complex incidents -- it tells you
*what* to do, *who* is involved, and *when* to escalate. Choosing the right format prevents both
under-documenting complex incidents and over-complicating simple tasks.
