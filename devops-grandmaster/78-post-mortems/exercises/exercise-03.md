# Exercise 03: Write a Complete Post-Mortem

**Type:** Independent | **Time:** 30 min | **Difficulty:** Medium

## Objective

Write a complete post-mortem document from raw incident facts and a timeline.
This exercise tests your ability to synthesize information, apply the post-mortem
template from Module 78, use blameless language (Exercise 01), and perform
root cause analysis (Exercise 02) -- all assembled into a professional document
that would be reviewed by engineering leadership.

## Background

A well-structured post-mortem is one of the most valuable artifacts an
engineering team produces. It serves multiple audiences:

- **Engineering teams** -- to understand what happened and what to fix
- **Leadership** -- to understand risk and allocate resources
- **New hires** -- to learn about the system's history and pain points
- **Future on-call engineers** -- to handle similar incidents faster

A complete post-mortem template typically includes:

1. **Header metadata** (date, severity, duration, author)
2. **Executive summary** (2-3 sentences for leadership)
3. **Impact** (who was affected, how, for how long)
4. **Timeline** (chronological events with timestamps)
5. **Root cause analysis** (5 Whys or equivalent)
6. **Contributing factors** (conditions that enabled or worsened the incident)
7. **What went well** (things that helped during the response)
8. **What went poorly** (things that hindered the response)
9. **Action items** (with owners, deadlines, and priorities)
10. **Lessons learned** (broader takeaways for the organization)

## The Incident

Read the following facts carefully. You will use them to write the post-mortem.

---

### Raw Facts (unstructured)

**Incident ID:** INC-2026-0342
**Date:** April 8, 2026
**Severity:** SEV-2 (High)
**Services affected:** Payment processing API, Checkout service
**Customers affected:** ~12,000 users attempted checkout during the incident window
**Revenue impact:** Estimated $47,000 in abandoned carts

#### Timeline of Events

- **02:15 UTC** -- Deployment of `payment-gateway-service` v2.14.0 begins via
  ArgoCD automated sync. The deployment includes a new dependency on
  `tax-calculation-service` v3.1.0.
- **02:17 UTC** -- `payment-gateway-service` pods begin rolling restart. New pods
  fail readiness checks because `tax-calculation-service` v3.1.0 is not yet
  deployed.
- **02:19 UTC** -- ArgoCD health check reports `payment-gateway-service` as
  "Degraded." No alert fires because the ArgoCD health check is not wired to
  PagerDuty.
- **02:25 UTC** -- Customers begin reporting "Payment failed" errors in the
  checkout flow. Support ticket volume spikes.
- **02:31 UTC** -- A support engineer escalates to the on-call SRE after seeing
  15+ payment failure tickets in 6 minutes.
- **02:35 UTC** -- On-call SRE acknowledges, begins investigating. Notices
  `payment-gateway-service` pods in CrashLoopBackOff.
- **02:42 UTC** -- SRE identifies that the new pods cannot connect to
  `tax-calculation-service` v3.1.0. Checks ArgoCD and discovers that the
  `tax-calculation-service` deployment is stuck in "OutOfSync" state.
- **02:48 UTC** -- SRE manually triggers ArgoCD sync for `tax-calculation-service`.
- **02:52 UTC** -- `tax-calculation-service` v3.1.0 pods become healthy.
- **02:55 UTC** -- `payment-gateway-service` pods restart successfully with the
  new dependency available.
- **03:00 UTC** -- Payment processing resumes. Error rate drops to zero.
- **03:15 UTC** -- Incident declared resolved after 15 minutes of stable metrics.

#### Additional Context

- The `payment-gateway-service` and `tax-calculation-service` are in the same
  ArgoCD ApplicationSet, but the sync order is not explicitly defined. ArgoCD
  syncs them in alphabetical order by default.
- There is no dependency graph configured in ArgoCD to enforce deployment order.
- The developer who authored the PR for `payment-gateway-service` v2.14.0 noted
  the new dependency in the PR description but did not update the deployment
  manifest to include a dependency ordering.
- The ArgoCD health check alert to PagerDuty was removed 3 months ago during
  an alert fatigue cleanup. At the time, the team decided it was too noisy.
- The readiness probe for `payment-gateway-service` checks HTTP `/healthz`,
  which fails when the `tax-calculation-service` dependency is unavailable.
- The on-call SRE had to SSH into the cluster to inspect ArgoCD state because
  the ArgoCD UI was not accessible from the SRE's laptop (VPN issue).
- Support engineers do not have a runbook for "payment failures" -- they
  escalate all payment issues to the SRE on-call.

---

## The Exercise

Write a complete post-mortem document for this incident. Use the template
structure below as your guide. You must include every section.

### Template Structure

```markdown
# Post-Mortem: INC-2026-0342

## Metadata

| Field | Value |
|-------|-------|
| Incident ID | |
| Date | |
| Severity | |
| Duration | |
| Author | |
| Reviewers | |

## Executive Summary

[2-3 sentences. What happened, what was the impact, what was the root cause.
Write this for a VP of Engineering who has 30 seconds.]

## Impact

### Customer Impact
[How many customers were affected? How were they affected?]

### Business Impact
[Revenue, SLA/SLO burn, support ticket volume, etc.]

### Duration of Impact
[Exact window of customer-facing impact]

## Timeline

| Time (UTC) | Event | Actor |
|-----------|-------|-------|
| [time] | [event] | [who/what] |

## Root Cause Analysis

[Perform a 5 Whys analysis. Include at least 2 branches if applicable.
Use blameless language throughout.]

## Contributing Factors

[Conditions that enabled or worsened the incident. Be specific and blameless.]

## What Went Well

[Things that helped during detection, response, or resolution.
Even bad incidents have bright spots.]

## What Went Poorly

[Things that hindered detection, response, or resolution.
Focus on systems and processes, not people.]

## Action Items

| # | Action Item | Owner | Deadline | Priority |
|---|------------|-------|----------|----------|
| 1 | [item] | [team/person] | [date] | [High/Med/Low] |

[Include at least 5 action items covering different categories:
automation, process, monitoring, documentation, architecture]

## Lessons Learned

[2-3 broader organizational takeaways. These should be insights that
apply beyond this specific incident.]

## Appendix

[Any additional data, graphs, or references.]
```

### Requirements

Your post-mortem must meet these criteria:

1. **Complete:** Every section in the template is filled in.
2. **Blameless:** No language blames individuals. Reference roles and teams.
3. **Specific:** Timelines, numbers, and names of services are exact.
4. **Actionable:** Action items have clear owners, deadlines, and priorities.
5. **Honest:** "What went poorly" honestly addresses gaps, including ones the
   team created themselves (e.g., removing the ArgoCD alert).
6. **Executive summary:** Must be readable by a non-technical leader in 30 seconds.

<details>
<summary>Hint 1: Executive Summary Structure</summary>
Use this formula: "On [date], [what happened] affecting [who] for [duration].
The root cause was [systemic issue]. Impact: [quantified impact]."
</details>

<details>
<summary>Hint 2: Action Item Categories</strong>
Consider action items in these categories:
- **Automation:** Can we prevent this class of failure automatically?
- **Monitoring:** Can we detect this faster next time?
- **Process:** What process gap allowed this to happen?
- **Documentation:** What documentation would have helped?
- **Architecture:** What architectural change would prevent cascading?
</details>

<details>
<summary>Hint 3: "What Went Well"</strong>
Even in this incident, consider: Did the SRE diagnose it relatively quickly?
Did the support team escalate effectively? Did the fix work on the first try?
</details>

---

## Self-Assessment

Rate yourself after completing this exercise:

- [ ] I included every section of the post-mortem template
- [ ] My executive summary is concise and readable by non-technical leadership
- [ ] I used blameless language throughout the entire document
- [ ] My timeline is complete and chronologically accurate
- [ ] My root cause analysis identifies systemic issues, not individuals
- [ ] My action items have clear owners, deadlines, and priorities
- [ ] My "What Went Poorly" section is honest about team-created gaps
- [ ] My lessons learned are broadly applicable beyond this incident

## Reflection Questions

1. **Balance:** Did you find it challenging to write "What Went Poorly" honestly
   without sounding blameful? How did you navigate that?

2. **Audience:** How did you adjust your writing for the executive summary vs.
   the technical sections?

3. **Completeness:** Were there facts in the raw data that you chose not to
   include in the timeline? Why? (Hint: a post-mortem timeline should include
   events relevant to the incident, not every minute of activity.)

4. **Action items:** How did you decide which action items are "High" priority
   vs. "Medium" or "Low"? What criteria did you use?

## Next Steps

Proceed to [Exercise 04: Post-Mortem Metrics Dashboard](exercise-04.md) to
build a tool that tracks and analyzes post-mortem data over time.
