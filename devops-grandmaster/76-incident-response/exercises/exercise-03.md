# Exercise 03: Incident Communication Templates

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective

Write clear, professional incident communication templates for each stage of an incident
lifecycle (Investigating, Identified, Monitoring, Resolved). Practice writing status page
updates, Slack messages, and internal escalation communications that are accurate, concise,
and calibrated to the audience.

## Prerequisites

- Completion of Exercises 01 and 02.
- Understanding of the incident lifecycle stages.
- Familiarity with Slack formatting (mrkdwn).

## Scenario

**Acme Corp** operates a multi-tenant SaaS platform. At 14:23 UTC on a Tuesday, monitoring
alerts fire indicating that the API gateway is returning HTTP 503 errors for 40% of requests.
The platform serves 2,000 enterprise customers globally. The incident is classified as P2.

```
┌─────────────────────────────────────────────────────────────────────┐
│                    INCIDENT LIFECYCLE STAGES                         │
│                                                                     │
│   DETECT ──> ASSESS ──> MOBILIZE ──> INVESTIGATE ──> RESOLVE        │
│      │          │          │             │              │            │
│      v          v          v             v              v            │
│   [Alert]   [Classify]  [Page Team]  [Root Cause]   [Fix Deploy]   │
│                                                                     │
│   ───────────────────────────────────────────────────────────────   │
│   COMMUNICATIONS:                                                   │
│                                                                     │
│   Investigating ──> Identified ──> Monitoring ──> Resolved          │
└─────────────────────────────────────────────────────────────────────┘
```

The timeline unfolds as follows:

| Time (UTC) | Event |
|------------|-------|
| 14:23 | Alert fires: API gateway 503 rate > 30% |
| 14:25 | On-call engineer Alice acknowledges |
| 14:30 | Alice confirms 503 errors are from upstream service `order-svc` |
| 14:35 | Alice pages the secondary (Bob) and opens incident channel |
| 14:50 | Root cause identified: `order-svc` deployment introduced a memory leak |
| 15:00 | Rollback of `order-svc` deployment begins |
| 15:10 | Rollback complete, 503 rate drops to 0% |
| 15:40 | Monitoring period ends, no recurrence |

## Tasks

### Part A: Investigating Stage

Write the following communications for the **Investigating** stage (at 14:30 UTC, shortly
after the alert fires):

1. A **Slack message** for the public `#incidents` channel.
2. A **status page update** for external customers.
3. An **internal Slack message** for the private `#incident-response` channel with technical
   details.

All three should clearly state that the team is aware of the issue and investigating.

<details><summary>Hint</summary>
Public communications should be vague about root cause (you do not know it yet) but specific
about symptoms and impact. Internal communications can include technical details like affected
services, error rates, and initial hypotheses. Always include a timestamp in UTC.
</details>

### Part B: Identified Stage

At 14:50 UTC, the root cause has been identified. Write:

1. A **Slack message** for `#incidents` updating customers on the root cause.
2. A **status page update** explaining what happened and what action is being taken.
3. A **Slack message** for the engineering `#platform-team` channel with technical details
   of the root cause and the remediation plan.

The root cause: the `order-svc` v2.4.1 deployment introduced a memory leak that caused
pod evictions, reducing capacity until the API gateway started returning 503 errors.

<details><summary>Hint</summary>
At this stage, you can be more specific about root cause. External customers care about
what is broken and when it will be fixed. Engineers care about the technical details:
which deployment, which commit, what the fix is. Always include an ETA if you have one,
but do not guess -- "we are working on a rollback" is better than a wrong ETA.
</details>

### Part C: Monitoring Stage

At 15:10 UTC, the rollback is complete and error rates have returned to normal. Write:

1. A **Slack message** for `#incidents` telling customers the issue is resolved but you are
   monitoring.
2. A **status page update** marking the service as recovering.
3. An **internal Slack message** confirming the rollback success and defining the monitoring
   criteria (what metrics, what thresholds, how long).

<details><summary>Hint</summary>
The monitoring stage is tricky. You are confident the fix worked, but you need to prove it
with data. Define specific criteria: "error rate below 0.1% for 30 minutes" is better than
"we are watching it." External customers want to know they can stop worrying. Internal teams
need to know when it is safe to close the incident.
</details>

### Part D: Resolved Stage

At 15:40 UTC, the monitoring period ends with no recurrence. Write:

1. A **final status page update** marking the incident as resolved.
2. A **Slack message** for `#incidents` confirming resolution.
3. An **internal Slack message** for `#incident-response` that includes a brief post-incident
   summary with:
   - Incident duration
   - Impact summary
   - Root cause
   - Remediation taken
   - Next steps (post-incident review scheduling)

<details><summary>Hint</summary>
The resolved communication is your last impression. External customers should feel confident
the issue is fully resolved. Internal teams should know what follow-up actions are expected.
Always schedule the post-incident review (PIR) in the resolved message -- do not let it
slip.
</details>

### Part E: Template Generalization

Take one of the communications you wrote above and generalize it into a reusable template
with placeholders. For example:

```
**Status Page Update Template**

**Title:** [Service Name] - [Brief Description]
**Status:** [Investigating / Identified / Monitoring / Resolved]
**Time:** [YYYY-MM-DD HH:MM UTC]

**Impact:** [Description of customer-facing impact]

**Current Status:** [What we know and what we are doing]

**Next Update:** [Time of next update or "We will update in X minutes"]
```

Create templates for at least two communication types (status page and Slack) that could be
reused for any incident at Acme Corp.

<details><summary>Hint</summary>
Good templates have two types of placeholders: required fields (must be filled in) and
optional fields (can be removed if not applicable). Add a comment for each placeholder
explaining what information should go there. Include a "Next Update" field to set
expectations with customers.
</details>

## Success Criteria

- [ ] Investigating-stage messages are accurate (no speculation about root cause).
- [ ] Identified-stage messages separate external (customer) and internal (engineering) audiences.
- [ ] Monitoring-stage messages include specific, measurable monitoring criteria.
- [ ] Resolved-stage messages include a post-incident review commitment.
- [ ] Generalized templates are reusable and include clear placeholder guidance.

## What You Should Understand After This Exercise

Incident communication is a skill, not an afterthought. The right message at the right time
builds trust with customers and keeps internal teams aligned. The wrong message -- too vague,
too technical, or premature -- erodes confidence and creates confusion. Templates help, but
every incident is unique; the template is a starting point, not a script.
