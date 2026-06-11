# Exercise 01: Change Management Principles for Infrastructure

**Type:** Conceptual
**Duration:** 30-45 minutes

## Objective

Demonstrate a thorough understanding of change management principles as applied
to infrastructure operations. You will classify changes by risk, describe the
lifecycle of a Request for Change (RFC), and identify the controls that prevent
unplanned outages.

## Instructions

### Part A -- Change Classification

For each scenario below, classify the change as **Standard**, **Normal**, or
**Emergency** and justify your classification in 1-2 sentences.

1. Applying a routine OS security patch to 50 web servers during a scheduled
   maintenance window.
2. Replacing a failed load balancer at 03:00 AM because the primary has crashed
   and traffic is dropping.
3. Migrating a production database from MySQL 5.7 to MySQL 8.0, requiring schema
   changes and a 15-minute downtime.
4. Adding a new DNS record for an internal service.
5. Scaling a Kubernetes cluster from 10 to 25 nodes in response to an unexpected
   traffic surge.

### Part B -- RFC Lifecycle

Draw or describe the lifecycle of a Normal change from initiation to
post-implementation review. Your lifecycle must include at minimum:

- Who can submit an RFC
- What information must be present in the RFC
- The role of a Change Advisory Board (CAB)
- Approval and rejection paths
- Implementation and verification steps
- Post-implementation review (PIR) requirements

### Part C -- Risk Assessment Matrix

Build a risk assessment matrix with the following dimensions:

| | Low Impact | Medium Impact | High Impact |
|---|---|---|---|
| **High Likelihood** | ? | ? | ? |
| **Medium Likelihood** | ? | ? | ? |
| **Low Likelihood** | ? | ? | ? |

For each cell, provide:
- The risk level (e.g., Critical, High, Medium, Low)
- The required approval level (e.g., Team Lead, CAB, Emergency CAB)
- One example infrastructure change that would fall in that cell

### Part D -- Controls and Safeguards

List and explain five controls or safeguards that an organization should have in
place to govern infrastructure changes. For each control, describe:

- What risk it mitigates
- How it works in practice
- What happens if the control is bypassed

## Success Criteria

- [ ] All five scenarios are correctly classified with clear justification
- [ ] RFC lifecycle covers all seven required elements
- [ ] Risk matrix is complete (9 cells) with risk levels, approval levels, and examples
- [ ] Five controls are described with risk mitigated, mechanism, and bypass consequences
- [ ] Answers demonstrate understanding of ITIL-aligned change management

## Hints

<details>
<summary>Hint 1 -- Change Types</summary>

Standard changes are low-risk, pre-approved, and follow a documented procedure.
Normal changes follow the full RFC lifecycle. Emergency changes bypass normal
CAB scheduling but still require documentation after the fact.

</details>

<details>
<summary>Hint 2 -- RFC Content</summary>

Think about what a reviewer would need to know: what is changing, why, what
could go wrong, how to verify it worked, and how to roll back if it doesn't.

</details>

<details>
<summary>Hint 3 -- Controls</summary>

Consider controls at different stages: before the change (approval, testing),
during the change (monitoring, staged rollout), and after the change
(verification, PIR).

</details>

## Deliverables

Submit your answers in a single Markdown file named
`exercise-01-answers.md`. Organize it with the same four parts (A through D)
as the instructions.
