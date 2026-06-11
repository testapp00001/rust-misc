# Exercise 01: Severity Classification

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

Classify 10 real-world incident scenarios into severity levels (P1 through P4) and describe the
escalation path each classification triggers. This exercise builds the judgment muscle every
on-call engineer needs.

## Scenario

You are the on-call engineer at **Acme Corp**, a SaaS company with 50,000 paying customers.
The company runs a multi-region Kubernetes platform with PostgreSQL, Redis, and a microservice
architecture. The following incidents have just been reported.

```
┌─────────────────────────────────────────────────────────────────────┐
│                        ACME CORP SEVERITY MATRIX                   │
├──────────┬──────────────────────────────────────────────────────────┤
│  P1      │ Complete outage affecting all customers. Revenue impact.│
│  (Crit)  │ All hands on deck. Incident Commander required.         │
├──────────┼──────────────────────────────────────────────────────────┤
│  P2      │ Major feature degraded. Significant customer impact.    │
│  (High)  │ On-call team mobilized. Escalation within 15 min.      │
├──────────┼──────────────────────────────────────────────────────────┤
│  P3      │ Minor feature degraded. Workaround exists.              │
│  (Med)   │ Assigned to on-call. Business-hours follow-up.          │
├──────────┼──────────────────────────────────────────────────────────┤
│  P4      │ Cosmetic or low-impact issue. No customer impact.        │
│  (Low)   │ Logged as a ticket. Scheduled for future sprint.        │
└──────────┴──────────────────────────────────────────────────────────┘
```

## Tasks

### Part A: Classify Each Scenario

For each scenario below, assign a severity level (P1-P4) and write a one-sentence
justification.

| #  | Scenario |
|----|----------|
| S1 | The primary PostgreSQL database in `us-east-1` is unreachable. All read and write operations are failing. 100% of users in that region are affected. |
| S2 | The recommendation engine returns stale results. Users see recommendations from 6 hours ago instead of real-time. No data loss. |
| S3 | A misconfigured `HorizontalPodAutoscaler` causes the billing service to scale to 200 pods, tripling infrastructure costs. Service is functioning normally. |
| S4 | The login page CSS is broken on Safari browsers. Users can still log in, but the page looks distorted. |
| S5 | The payment processing service times out for 30% of transactions. Customers are retrying, and some are abandoning purchases. |
| S6 | A developer accidentally committed AWS credentials to a public GitHub repository. No evidence of misuse yet. |
| S7 | The CI/CD pipeline takes 45 minutes instead of the usual 12 minutes. Deployments are delayed but still complete. |
| S8 | Redis cluster in `eu-west-1` lost its replica node. The primary is serving all traffic with increased latency. No data loss. |
| S9 | The internal admin dashboard is down. Only the ops team uses it. Customer-facing services are unaffected. |
| S10 | A botched schema migration corrupted the `orders` table. Order history for the past 24 hours is unrecoverable from the primary. Backups exist from 6 hours ago. |

<details><summary>Hint</summary>
Consider three axes for each scenario: (1) breadth of impact -- how many customers or regions
are affected? (2) depth of impact -- is it a full outage, degraded performance, or cosmetic?
(3) urgency -- is there a security risk, data loss, or financial exposure that worsens with time?
</details>

### Part B: Map the Escalation Path

For each scenario you classified above, describe the escalation path. Use the following format:

```
Scenario S[N]:
  Severity: P[X]
  First Responder: [role]
  Escalate To: [role] after [time window]
  Additional Stakeholders: [list]
  Communication Channel: [Slack channel / war room / email]
  External Communication Required: [Yes/No]
```

<details><summary>Hint</summary>
P1 incidents typically escalate to the Incident Commander immediately and involve executive
communication. P2 incidents escalate within 15 minutes to the team lead. P3 and P4 incidents
may not require escalation at all.
</details>

### Part C: Severity Boundary Disputes

Two engineers disagree on the severity of Scenario S5 (payment timeouts). Engineer A says it is
P1 because revenue is directly impacted. Engineer B says it is P2 because 70% of transactions
still succeed.

Write a short decision framework (5 bullet points or fewer) that Acme Corp could adopt to
resolve severity disputes consistently.

<details><summary>Hint</summary>
Good frameworks use measurable thresholds rather than subjective judgment. Think about
percentages of affected users, revenue impact per hour, and whether the issue is getting
worse or stable.
</details>

### Part D: Retroactive Reclassification

Scenario S2 (stale recommendations) was initially classified as P3. After 2 hours, the
engineering team discovers that the stale data is causing the recommendation engine to suggest
out-of-stock products, leading to a spike in customer complaints and a 15% increase in support
tickets.

Should the severity be reclassified? If so, to what level? Write a brief justification and
describe the process for mid-incident severity changes.

<details><summary>Hint</summary>
Severity is not static. When new information changes the impact assessment, the Incident
Commander has the authority -- and the responsibility -- to reclassify. The key question is:
does the new information change the breadth, depth, or urgency of the impact?
</details>

### Part E: Build a Severity Matrix

Design a severity matrix for a fictional company called **DataStream**, a real-time analytics
platform that processes 1 million events per second for financial services clients. The matrix
should cover:

- Data accuracy issues
- Latency degradation
- Complete service outage
- Security incidents
- Cost anomalies

Present your matrix as a markdown table with columns: Severity, Criteria, Customer Impact,
Response Time, and Escalation Path.

<details><summary>Hint</summary>
Financial services clients have very low tolerance for data accuracy issues. A 0.01% data
loss might be a P1 for DataStream even though it would be a P3 for a social media app.
Context matters.
</details>

## Success Criteria

- [ ] All 10 scenarios are classified with clear justifications.
- [ ] Escalation paths are realistic and follow the severity matrix.
- [ ] The decision framework from Part C is objective and measurable.
- [ ] The reclassification in Part D follows a documented process.
- [ ] The DataStream severity matrix from Part E reflects domain-specific concerns.

## What You Should Understand After This Exercise

Severity classification is not about how hard the problem is to fix -- it is about the blast
radius on customers and the business. A well-defined severity matrix removes ambiguity, speeds
up response, and ensures the right people are mobilized for the right incidents. Context
matters: the same technical issue can be different severities for different businesses.
