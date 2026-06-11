# Exercise 03: Create an Error Budget Policy

**Type:** Independent
**Objective:** Design a complete error budget policy that defines consequences
and actions when a team's error budget is consumed.

## Background

An error budget is the inverse of an SLO. If your SLO is 99.9% availability,
your error budget is 0.1% -- the amount of failure you can tolerate. An error
budget **policy** defines what happens when that budget is at risk of being
exhausted. Without a policy, error budgets are just numbers on a dashboard.

## Scenario

You are the SRE lead for **DataPipe**, a data processing platform with the
following characteristics:

- **SLO:** 99.95% availability, measured monthly as the ratio of successful
  data processing jobs to total submitted jobs.
- **Error budget:** 0.05% of monthly jobs may fail. With approximately 1,000,000
  jobs per month, that is 500 failed jobs.
- **Team:** 8 engineers. The team ships 3-5 deployments per week.
- **Current pain:** The team sometimes deploys risky changes on Fridays, leading
  to weekend incidents. There is no formal process for slowing down when
  reliability degrades.

## Instructions

### Part A -- Budget Consumption Thresholds

Define three threshold levels for error budget consumption within a calendar
month. For each level, specify:

| Level | Budget Consumed | Status |
|-------|----------------|--------|
| Green | ?%             | ?      |
| Yellow | ?%            | ?      |
| Red   | ?%             | ?      |

For each level, describe what it signals about the team's current reliability
posture.

### Part B -- Actions per Threshold

For **each** threshold level, define specific, actionable responses. Cover these
categories:

1. **Deployment policy** -- Can the team deploy normally? Any restrictions?
2. **Incident response** -- Does the incident response process change?
3. **Engineering work** -- Should the team shift focus? What kind of work takes
   priority?
4. **Communication** -- Who gets notified? How?
5. **Escalation** -- Does anything escalate to leadership?

### Part C -- Recovery Criteria

Define how the team transitions back from Red to Yellow, and from Yellow to
Green. What conditions must be met? Is it purely based on budget remaining, or
do other factors matter?

### Part D -- Policy Exceptions

Identify 2-3 scenarios where the error budget policy might need exceptions. For
each, describe:
- The scenario
- Who can authorize the exception
- What guardrails apply

## Success Criteria

- [ ] Three threshold levels are defined with clear numeric boundaries.
- [ ] Each threshold has specific actions across all five categories.
- [ ] Actions become progressively more restrictive as severity increases.
- [ ] Recovery criteria are concrete and measurable.
- [ ] At least two exception scenarios are identified with authorization
      requirements.
- [ ] The policy does not simply say "stop deploying" at every level -- actions
      are proportional.

## Hints

<details>
<summary>Hint 1 -- Think about rate of consumption</summary>

Do not just look at how much budget has been consumed. Look at how fast it is
being consumed. If you have used 50% of your budget in the first week, that is
very different from using 50% in the last week of the month. Consider
incorporating a "burn rate" into your thresholds.

</details>

<details>
<summary>Hint 2 -- Deployment restrictions should be specific</summary>

"Be more careful" is not an action. "Require a second reviewer on all PRs" or
"freeze deployments to production between Friday 2pm and Monday 9am" are
actions. Make your policy something an engineer can follow without ambiguity.

</details>

<details>
<summary>Hint 3 -- Not every incident is equal</summary>

A user-facing outage and a background job failure may both consume error budget,
but they may warrant different responses. Consider whether your policy
distinguishes between types of budget consumption.

</details>
