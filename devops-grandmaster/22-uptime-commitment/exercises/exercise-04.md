# Exercise 04: Design SLOs Balancing Reliability and Velocity

**Type:** Challenge
**Objective:** Set SLOs that are ambitious enough to satisfy users but not so
aggressive that they prevent the team from shipping features.

## Background

Setting SLOs too high leads to over-engineering: excessive redundancy, slow
deployment processes, and engineers spending most of their time on reliability
work instead of product features. Setting them too low leads to user
dissatisfaction and churn. The art is finding the right balance.

## Scenario

You are a Staff SRE advising three different product teams. Each team needs
SLOs, but they have different constraints. Your job is to recommend SLOs and
justify them.

### Team 1: Internal Developer Platform

- **Users:** 200 internal engineers who use the platform for CI/CD, deployments,
  and observability.
- **Current reliability:** ~99.5% availability.
- **Pain point:** Engineers lose productivity when the platform is down, but they
  can usually wait 15-30 minutes without major business impact.
- **Engineering capacity:** 4 engineers, half of whom are also building new
  platform features.
- **Question:** Should they target 99.9%, 99.95%, or 99.99%?

### Team 2: Consumer Mobile App

- **Users:** 500,000 daily active users. Revenue comes from in-app purchases.
- **Current reliability:** ~99% availability.
- **Pain point:** Users who encounter errors churn at a rate 3x higher than
  those who do not.
- **Engineering capacity:** 10 engineers, strong feature backlog with
  competitive pressure.
- **Question:** Should they target 99.5%, 99.9%, or 99.95%?

### Team 3: Healthcare Data Pipeline

- **Users:** 50 hospitals that receive patient data through the pipeline.
- **Current reliability:** ~99.99% availability.
- **Pain point:** Delayed data can affect patient care decisions. Regulatory
  requirements mandate certain uptime levels.
- **Engineering capacity:** 6 engineers, but the system is mature with few new
  feature requests.
- **Question:** Should they target 99.99%, 99.995%, or 99.999%?

## Instructions

### Part A -- Recommend SLOs

For each team, recommend a specific availability SLO. For each recommendation:

1. State the SLO target.
2. Calculate the allowed downtime per month.
3. Explain **why this target and not higher or lower**. Address:
   - What happens if the SLO is too high (over-investment in reliability).
   - What happens if the SLO is too low (user/business impact).
   - How the team's current reliability affects feasibility.

### Part B -- SLO Roadmap

For each team, create a 6-month roadmap:
- **Month 1-2:** What SLO do you start with?
- **Month 3-4:** What changes, if anything?
- **Month 5-6:** What is the target SLO?

Explain what triggers each change (e.g., "after achieving X for two consecutive
months, increase the target").

### Part C -- Define Non-Availability SLOs

Availability alone is not enough. For each team, define one additional SLO from
one of these categories:

- **Latency** (e.g., p99 response time)
- **Freshness** (e.g., data is no more than X minutes stale)
- **Correctness** (e.g., X% of operations produce the right result)
- **Throughput** (e.g., system handles X concurrent users)

Explain why this SLO matters more than another category for that specific team.

### Part D -- Anti-Patterns

Identify one anti-pattern for each team that would lead to SLOs being
counterproductive. Examples of anti-patterns:
- Setting SLOs the team cannot measure
- SLOs that incentivize the wrong behavior
- SLOs that are ignored because they are not tied to consequences

For each anti-pattern, describe a concrete mitigation.

## Success Criteria

- [ ] Each team has a specific availability SLO with a calculated downtime
      allowance.
- [ ] Each recommendation justifies why the target is not higher or lower.
- [ ] The roadmap shows a progression over 6 months with clear triggers.
- [ ] At least one non-availability SLO is defined per team.
- [ ] Anti-patterns are specific to each team's situation, not generic.

## Hints

<details>
<summary>Hint 1 -- The 99.X% decision framework</summary>

Ask these questions:
- What would users do if the service was down for 5 minutes? 30 minutes? 2
  hours?
- Is there a competitor users would switch to?
- Are there regulatory or contractual requirements?
- How much engineering time can the team dedicate to reliability vs. features?

The answers tell you where the inflection point is.

</details>

<details>
<summary>Hint 2 -- Internal vs. external services</summary>

Internal services can often tolerate lower SLOs because users are employees who
can be communicated with directly. However, if the internal service blocks
external-facing teams, the SLO should reflect that downstream impact.

</details>

<details>
<summary>Hint 3 -- Current reliability constrains your starting point</summary>

If a team is at 99.5% today, jumping to 99.99% requires a massive investment.
Start by committing to your current reliability level (e.g., 99.5% SLO) and
then incrementally improve. An SLO you cannot meet is worse than no SLO at all.

</details>
