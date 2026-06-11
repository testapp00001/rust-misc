# Solution 03: Create an Error Budget Policy

## Part A -- Budget Consumption Thresholds

Monthly error budget: 0.05% of 1,000,000 jobs = **500 failed jobs** (or 21.6
minutes of downtime equivalent at current throughput).

| Level | Budget Consumed | Burn Rate | Status |
|-------|----------------|-----------|--------|
| Green | 0--50% (0--250 jobs) | On track or better | Healthy. The team is within normal operating parameters. Reliability work is balanced with feature work. |
| Yellow | 50--80% (250--400 jobs) | Elevated -- budget may be exhausted before month end | At risk. The rate of failures is trending toward budget exhaustion. The team must shift focus toward reliability. |
| Red | 80--100% (400--500 jobs) | Critical -- budget nearly or fully consumed | Emergency. The team has little to no room for further failures. All non-critical work stops until reliability is restored. |

**Why these thresholds:**

- **50% (Yellow)** is chosen because if half the budget is consumed before
  halfway through the month, the burn rate is unsustainable. It gives the team
  2--3 weeks to recover.
- **80% (Red)** is chosen because with only 20% remaining (~100 jobs), a single
  bad deployment could exhaust the budget entirely. This is the "stop the
  bleeding" threshold.
- These are absolute thresholds. A supplementary burn-rate check is recommended:
  if 50% is consumed in the first 7 days (pace = 214% by month end), escalate
  to Yellow even if the absolute number seems manageable.

## Part B -- Actions per Threshold

### Green (0--50% consumed)

| Category | Action |
|----------|--------|
| **Deployment policy** | Normal cadence: 3--5 deployments per week. Standard code review (1 approval required). Friday deployments allowed until 2pm. |
| **Incident response** | Standard on-call rotation. P1 incidents: 15-minute response SLA. P2: 1-hour response. |
| **Engineering work** | Normal backlog prioritization. Reliability improvements are planned alongside feature work. Up to 30% of sprint capacity may be allocated to tech debt. |
| **Communication** | Weekly reliability digest in team Slack channel. Monthly SLO review in team meeting. |
| **Escalation** | No escalation required. Team lead monitors error budget dashboard as part of weekly review. |

### Yellow (50--80% consumed)

| Category | Action |
|----------|--------|
| **Deployment policy** | Require 2 reviewers for all PRs. No deployments after 1pm on Fridays. All deployments must have a rollback plan documented in the PR. Canary deployments required for changes touching the data pipeline core. |
| **Incident response** | Escalation timer reduced: P1 response within 10 minutes. All incidents classified as Yellow-level require a blameless postmortem within 48 hours. On-call engineer empowered to halt deployments during active incidents. |
| **Engineering work** | At least 50% of engineering capacity shifts to reliability work. Feature work continues but deprioritized. Reliability work items are pulled from the top of the backlog (e.g., retry logic, circuit breakers, improved monitoring). |
| **Communication** | Daily error budget status posted to team Slack channel. Engineering manager notified. Stakeholders (product manager, dependent teams) informed of reduced deployment velocity. |
| **Escalation** | Engineering manager is briefed. If Yellow persists for 5 consecutive days, escalate to director of engineering. |

### Red (80--100% consumed)

| Category | Action |
|----------|--------|
| **Deployment policy** | Production deployment freeze except for critical security patches and reliability fixes. All changes require explicit approval from the SRE lead and engineering manager. No Friday deployments under any circumstances. |
| **Incident response** | All hands on deck. P1 response within 5 minutes. Incident commander assigned for every P1. All incidents require postmortems within 24 hours with action items tracked to completion. |
| **Engineering work** | 100% of engineering capacity on reliability. All feature work paused. Focus on the top 3 failure modes consuming error budget. Emergency reliability improvements may bypass normal sprint planning. |
| **Communication** | Daily standup includes error budget status as the first agenda item. VP of Engineering notified. If customer-facing impact, Customer Success team informed to prepare communications. |
| **Escalation** | VP of Engineering briefed immediately. If Red persists for 7 days, executive leadership review triggered. Consider engaging external SRE consultants if the team lacks capacity to resolve root causes. |

## Part C -- Recovery Criteria

### Red to Yellow

All of the following must be true:

1. **Budget consumption rate has stabilized.** No new significant incidents in the
   past 72 hours. The projected month-end consumption (based on trailing 3-day
   burn rate) is below 90%.

2. **Top incident root causes are addressed.** The postmortem action items from
   the incidents that drove the team into Red are either completed or have
   concrete owner-assigned timelines.

3. **Monitoring gaps are filled.** If the incidents that caused Red were
   undetected or detected late, the monitoring/alerting gaps have been patched.

4. **Approval required.** The SRE lead and engineering manager jointly approve
   the transition. This prevents premature relaxation of controls.

### Yellow to Green

All of the following must be true:

1. **Budget consumption is below 50%** and the trailing 7-day burn rate projects
   the month will end below 60% consumption.

2. **No open P1 or P2 incidents** related to reliability degradation.

3. **At least 14 consecutive days** at the Yellow level before transitioning
   (prevents flapping between Green and Yellow).

4. **Approval required.** Team lead approves the transition back to Green.

### Important: Anti-Flapping Rule

If the team transitions from Green to Yellow and back to Green within a 7-day
window, the Yellow threshold is tightened by 10 percentage points for the
remainder of the month. This prevents a pattern of "consume budget, recover,
consume budget, recover" that exhausts the budget in bursts.

## Part D -- Policy Exceptions

### Exception 1: Critical Security Vulnerability

- **Scenario:** A critical security vulnerability (CVSS 9.0+) is discovered in
  a production dependency. A patch must be deployed immediately regardless of
  error budget status.
- **Authorization:** SRE lead + engineering manager must both approve. If both
  are unavailable, the on-call engineer may deploy with post-facto review within
  24 hours.
- **Guardrails:** The patch deployment uses a canary rollout. If the canary
  shows increased error rates above 2x the current rate, the deployment halts
  and the team evaluates alternative mitigations (e.g., WAF rules, feature
  flags) instead.

### Exception 2: Contractual or Regulatory Deadline

- **Scenario:** A regulatory compliance change (e.g., data retention policy)
  has a legal deadline that cannot be moved. The change must ship before the
  deadline even if the error budget is in Red.
- **Authorization:** VP of engineering must approve. The request must include a
  risk assessment, rollback plan, and the specific regulatory citation requiring
  the change.
- **Guardrails:** The change is deployed during the lowest-traffic window
  (typically Sunday 2--6am). Additional on-call coverage is arranged. The
  change is isolated behind a feature flag so it can be disabled without a full
  rollback.

### Exception 3: Revenue-Critical Feature with Executive Sponsorship

- **Scenario:** A major customer has a contractual commitment for a feature
  delivery date. Missing the date results in a significant revenue penalty
  (e.g., >$100K).
- **Authorization:** VP of engineering + product VP must jointly approve. A
  written risk acknowledgment is required, documenting that the deployment is
  happening against policy and accepting the risk of budget overrun.
- **Guardrails:** The feature is deployed with maximum observability (extra
  logging, synthetic monitors, dedicated on-call engineer watching dashboards).
  A pre-approved rollback path is tested before deployment. If the deployment
  causes any budget consumption, it counts against the feature team's
  reliability score, incentivizing careful engineering.

## Why This Solution Works

1. **Proportional escalation.** Actions scale with severity -- Yellow adds
   process (extra reviewers), Red freezes everything. This avoids both
   over-reacting to minor budget consumption and under-reacting to serious
   degradation.

2. **Specific and actionable.** Every action is something an engineer can
   follow without ambiguity. "Require 2 reviewers" is an action. "Be careful"
   is not.

3. **Addresses the Friday deployment problem.** The policy specifically
   restricts Friday deployments at Yellow and bans them at Red, directly
   solving the scenario's stated pain point.

4. **Recovery is not automatic.** Teams must demonstrate stability before
   relaxing controls. This prevents the cycle of "budget consumed, freeze,
   budget recovered, unfreeze, budget consumed again."

5. **Exceptions are controlled.** Every exception requires senior
   authorization and has guardrails. This prevents the policy from being
   circumvented while acknowledging that real-world constraints exist.

## Common Mistakes

1. **Binary policy (Green/Red only).** Without a Yellow level, teams go
   directly from "everything is fine" to "deployment freeze." The Yellow level
   provides a gradient that allows proportional response.

2. **Making recovery automatic.** "Red clears at the start of the next month"
   incentivizes the team to write off a bad month rather than fixing root
   causes. Recovery must be earned.

3. **Vague actions.** "Improve reliability" is not an action. "Add retry logic
   to the message queue consumer with exponential backoff" is an action. Every
   policy statement should pass the test: "Can an engineer follow this without
   asking a clarifying question?"

4. **No burn rate consideration.** Consuming 50% of the budget in 3 days is
   very different from consuming 50% in 25 days. Policies that only look at
   absolute consumption miss the trajectory.

5. **No exceptions process.** Teams will silently violate a policy that has no
   escape valve. By defining exceptions with authorization requirements, you
   ensure that policy violations are visible and deliberate rather than hidden.

6. **Treating all budget consumption equally.** A background job failure at 3am
   and a user-facing checkout outage at peak hours both consume budget, but
   they warrant different responses. Consider severity-weighted budget tracking
   as a future enhancement.
