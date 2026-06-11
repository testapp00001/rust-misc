# Solution 04: Balance Reliability and Velocity with SLOs

## Part A -- Recommended SLOs

### Team 1: Internal Developer Platform -- Recommended SLO: 99.9%

**Monthly downtime allowance:**
```
30 days * 24h * 60m = 43,200 minutes
43,200 * (1 - 0.999) = 43.2 minutes per month
```

**Why 99.9% and not higher:**

- **Why not 99.95% (21.6 min/month):** With only 4 engineers (2 on features),
  the team cannot invest enough in redundancy and automation to reliably hit
  99.95%. The current reliability is 99.5%, meaning they would need to nearly
  eliminate all downtime immediately -- an unrealistic jump. At 99.95%, the
  team would spend most of their time on reliability work, starving the
  platform of new features that the 200 engineers depend on.

- **Why not 99.99% (4.3 min/month):** This would require active-active
  redundancy, automated failover, and near-zero-downtime deployments -- all
  disproportionate for an internal service with 4 engineers. The operational
  cost would consume the entire team.

- **Why 99.9% is right:** 43 minutes of monthly downtime means internal
  engineers might experience one 30-minute disruption per month. Since they can
  wait 15--30 minutes without major business impact, this aligns with actual
  tolerance. It also gives the team error budget to deploy confidently (3--5
  deploys/week with room for occasional failures). The jump from 99.5% to 99.9%
  is achievable: it requires improving deployment practices and adding basic
  health checks, not a full architectural overhaul.

### Team 2: Consumer Mobile App -- Recommended SLO: 99.9%

**Monthly downtime allowance:**
```
43,200 * (1 - 0.999) = 43.2 minutes per month
```

**Why 99.9% and not higher:**

- **Why not 99.95% (21.6 min/month):** The team is currently at 99%, which is
  7+ hours of downtime per month. Jumping to 99.95% would require eliminating
  95% of current failures in one step. With competitive pressure and a strong
  feature backlog, dedicating enough engineers to reach 99.95% immediately would
  stall product development. Better to reach 99.9% first and iterate.

- **Why not 99.5% (3h 36m/month):** While 99.5% is closer to the current 99%,
  it is too lenient for a consumer app with 500K DAU and in-app purchase
  revenue. At 99.5%, users would encounter errors for over 3 hours per month.
  Given the 3x churn rate for users who encounter errors, this translates to
  significant revenue loss. The current 99% is a problem to solve, not a
  standard to enshrine.

- **Why 99.9% is right:** 43 minutes per month is tolerable for a consumer app.
  Users expect mobile apps to occasionally hiccup, but they do not expect hours
  of unavailability. The 3x churn penalty means every minute of downtime costs
  real revenue, making the investment worthwhile. The jump from 99% to 99.9% is
  ambitious but achievable: it typically requires fixing the top 3--5 failure
  modes (e.g., database connection exhaustion, third-party API timeouts, memory
  leaks), not a complete rewrite. With 10 engineers, the team has capacity to
  split between reliability and features.

### Team 3: Healthcare Data Pipeline -- Recommended SLO: 99.995%

**Monthly downtime allowance:**
```
43,200 * (1 - 0.99995) = 2.16 minutes per month
```

**Why 99.995% and not higher:**

- **Why not 99.999% (26 seconds/month):** 99.999% requires fully automated
   failover with zero human intervention, active-active multi-region
   architecture, and the ability to deploy without any user-visible impact.
   With 6 engineers, even on a mature system, this is extremely expensive.
   The engineering cost of the last 0.004% (from 99.995 to 99.999) is
   typically 5--10x the cost of the first 0.005% (from 99.99 to 99.995).
   Unless regulations specifically mandate five nines, this is
   over-investment.

- **Why not 99.99% (4.3 min/month):** 99.99% is the team's current
   reliability level. Setting an SLO at your current performance means you
   have no error budget -- any incident puts you in violation. This creates
   perverse incentives: the team cannot deploy changes (every deployment risks
   violating the SLO) and cannot improve the system. The SLO should be set
   slightly above current reliability to create a small but usable error
   budget.

- **Why 99.995% is right:** With 50 hospitals depending on the pipeline for
   patient care decisions, reliability matters deeply. 2.16 minutes per month
   is tight but achievable for a mature system with few feature changes. The
   6 engineers can focus almost entirely on reliability. Regulatory
   requirements likely mandate high availability; 99.995% demonstrates due
   diligence without the extreme cost of five nines. The small error budget
   (2.16 min/month) forces disciplined deployment practices -- changes must be
   thoroughly tested and rolled out carefully, which is appropriate for
   healthcare.

## Part B -- SLO Roadmaps

### Team 1: Internal Developer Platform

| Period | SLO Target | Rationale |
|--------|-----------|-----------|
| **Month 1--2** | 99.5% (current) | Do not commit to a target you cannot meet. Measure actual reliability for 2 months to establish a baseline. Identify top failure modes. |
| **Month 3--4** | 99.9% | After fixing the top 3 failure modes identified in months 1--2 (likely: deployment failures, missing health checks, no automatic restarts), commit to 99.9%. This is the target SLO. |
| **Month 5--6** | 99.9% (maintain) | Hold at 99.9%. If the team achieves 99.95%+ for 2 consecutive months, discuss raising the target -- but only if the team has capacity and the platform's user base has grown. |

**Trigger for change:** If 99.9% is met for 3 consecutive months with less than 50% error budget consumption, the team may optionally raise the target to 99.95% in month 7+.

### Team 2: Consumer Mobile App

| Period | SLO Target | Rationale |
|--------|-----------|-----------|
| **Month 1--2** | 99.5% | Start with a target the team can meet from their current 99%. This builds the SLO practice (measurement, dashboards, alerts) without creating impossible pressure. Focus on identifying and fixing the worst failure mode. |
| **Month 3--4** | 99.9% | After demonstrating 99.5% for 2 months, raise to 99.9%. By now the team has fixed the top failure mode, added monitoring, and established an error budget practice. The remaining failures are smaller and more numerous. |
| **Month 5--6** | 99.9% (maintain) | Hold at 99.9%. The team needs time to stabilize at this level before considering further increases. Feature velocity should recover as reliability work shifts from "emergency fixes" to "planned improvements." |

**Trigger for change:** If 99.9% is met for 3 consecutive months and the churn rate for error-experiencing users drops below 2x (from 3x), consider raising to 99.95% in month 7+.

### Team 3: Healthcare Data Pipeline

| Period | SLO Target | Rationale |
|--------|-----------|-----------|
| **Month 1--2** | 99.99% (current) | Commit to current reliability. Even though the team is already here, formalizing it as an SLO creates accountability. Use this period to build dashboards and alerting. |
| **Month 3--4** | 99.995% | After 2 months of stable 99.99% with low budget consumption, raise the target. The team has capacity (6 engineers, few feature requests) and the system is mature. Focus on eliminating the remaining failure modes (likely: dependency timeouts, resource exhaustion during peak load). |
| **Month 5--6** | 99.995% (maintain) | Hold at 99.995%. Do not push to 99.999% unless regulatory requirements demand it. The cost of the last 0.004% is disproportionate. Invest in correctness and freshness SLOs instead. |

**Trigger for change:** Only raise to 99.999% if a regulatory body or major hospital customer requires it in writing, and only after demonstrating 99.995% for 6 consecutive months.

## Part C -- Non-Availability SLOs

### Team 1: Internal Developer Platform -- Latency

```
SLI: p95 CI/CD pipeline completion time (commit to deployed artifact)
SLO: 95% of CI/CD pipeline runs complete within 10 minutes
```

**Why latency over other categories:** Internal engineers run CI/CD pipelines
dozens of times per day. A slow pipeline is a direct productivity tax. If a
pipeline takes 30 minutes, an engineer context-switches, loses flow, and wastes
time. Freshness and correctness matter less -- builds either succeed or fail,
and the data (build artifacts) is not time-sensitive in the same way. Throughput
is less critical because 200 engineers is a bounded user base.

### Team 2: Consumer Mobile App -- Latency

```
SLI: p99 API response time for all user-facing endpoints
SLO: 99% of API requests complete within 1 second
```

**Why latency over other categories:** For a consumer app, perceived speed
drives engagement. Studies show that every 100ms of added latency reduces
conversion by ~1%. Users on mobile networks are especially sensitive to latency.
Correctness matters (orders must be accurate), but the payment provider handles
transaction integrity. Freshness is less critical -- product catalog data
changes infrequently. Throughput is addressed by auto-scaling.

### Team 3: Healthcare Data Pipeline -- Freshness

```
SLI: Time between a hospital submitting patient data and it appearing in the
receiving system's query results
SLO: 99% of patient data records are queryable within 15 minutes of submission
```

**Why freshness over other categories:** In healthcare, stale data can affect
patient care decisions. A doctor reviewing lab results needs to know they are
seeing current data. Latency matters (the system should respond quickly to
queries), but the more critical question is "is the data I am seeing up to
date?" Correctness is important but is typically handled by validation at
ingestion. Throughput is bounded by the number of hospitals (50).

## Part D -- Anti-Patterns

### Team 1: Internal Developer Platform -- Anti-Pattern: SLO Inflation Due to Internal Pressure

**The anti-pattern:** Because users are internal engineers (colleagues), there
is social pressure to set the SLO artificially high. The platform team's
engineers may feel obligated to promise 99.99% because their friends in other
teams complain loudly when CI/CD is down. This leads to an SLO the team cannot
meet, constant SLO violations that everyone ignores, and ultimately an SLO that
provides no useful signal.

**Mitigation:** Tie the SLO to a data-driven framework. Present the other
teams with the trade-off: "We can target 99.99%, but that means zero new
platform features for 6 months while we build redundancy. Or we target 99.9%
and deliver the deployment pipeline improvements you asked for." Let the
stakeholders choose. Document the decision and revisit quarterly.

### Team 2: Consumer Mobile App -- Anti-Pattern: Availability-Only Focus Ignoring Performance

**The anti-pattern:** The team sets a 99.9% availability SLO and declares
victory. The API returns 200 responses, but p99 latency is 8 seconds because
the database is overloaded. Users see a spinner for 8 seconds, assume the app
is broken, and churn anyway. The SLO is technically met (requests succeeded!)
but the business goal (reduce churn) is not achieved.

**Mitigation:** Pair the availability SLO with the latency SLO from Part C
(p99 < 1 second). Track both on the same dashboard. If either SLO is violated,
it triggers the same error budget policy. This ensures the team optimizes for
"user requests succeed AND feel fast," not just "requests succeed."

### Team 3: Healthcare Data Pipeline -- Anti-Pattern: SLO as Ceiling Instead of Floor

**The anti-pattern:** The team treats the 99.995% SLO as the maximum
reliability they need to achieve, not the minimum. They reason: "We are meeting
the SLO, so we can take more risks with deployments." They begin deploying
without canary rollouts because they have error budget to spare. A bad
deployment consumes the entire monthly budget in one incident, leaving no room
for the rest of the month.

**Mitigation:** Implement an error budget policy (Module 22, Exercise 3) that
restricts deployment practices as budget is consumed. Additionally, set a
secondary SLO for "percentage of months where the SLO is met" -- target 11 out
of 12 months per year. This incentivizes the team to treat the SLO as a floor
to protect, not a ceiling to hit and then exploit.

## Why This Solution Works

1. **Current reality constrains the starting point.** Each team's recommended
   SLO is anchored to their current reliability, not an aspirational number.
   An SLO you cannot meet is worse than no SLO at all -- it erodes trust in
   the entire framework.

2. **Roadmaps are incremental.** Each team starts where they are and steps up
   over 6 months. This builds organizational muscle (measurement, dashboards,
   error budget culture) before raising the bar.

3. **Non-availability SLOs reflect each team's specific pain.** Internal
   platform users care about speed (latency). Consumer app users care about
   responsiveness (latency). Hospital users care about data recency
   (freshness). The SLOs match the user's actual concern.

4. **Anti-patterns are specific.** Each anti-pattern is unique to the team's
   context: internal pressure for the platform team, performance blind spots
   for the consumer app, complacency for the healthcare pipeline.

## Common Mistakes

1. **Setting SLOs based on aspiration, not measurement.** "We want 99.99%"
   without knowing your current reliability is setting yourself up for failure.
   Always start with a baseline measurement period.

2. **Skipping the roadmap.** Jumping directly from 99% to 99.9% in one step
   overwhelms the team. Incremental targets build confidence and capability.

3. **One SLO per service.** Availability alone does not capture user
   experience. A service can be "available" but so slow that users abandon it.
   Always pair availability with at least one performance SLO.

4. **Identical SLOs for all teams.** An internal platform and a healthcare
   pipeline have fundamentally different reliability requirements, user
   expectations, and engineering constraints. Cookie-cutter SLOs miss these
   differences.

5. **Ignoring the cost of each "nine."** The jump from 99.9% to 99.99% is not
   10x harder -- it is typically 100x more expensive in engineering effort.
   Always calculate the downtime allowance and ask: "Is the engineering
   investment proportional to the user impact we are preventing?"

6. **No anti-pattern awareness.** Teams that define SLOs without considering
   how they could be gamed or ignored end up with SLOs that exist on paper but
   do not influence behavior. Thinking about failure modes of the SLO process
   itself is as important as thinking about failure modes of the system.
