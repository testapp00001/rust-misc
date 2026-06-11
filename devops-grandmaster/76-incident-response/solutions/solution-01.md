# Solution 01: Severity Classification

## Part A: Complete Severity Level Definitions

Every organization needs a clear, agreed-upon severity matrix. Without it,
teams waste precious minutes debating whether something is a P1 or P2 during
an outage -- exactly when every second counts.

### Severity Level Reference Table

| Level | Name | Impact | Response Time | Update Frequency | Example |
|-------|------|--------|---------------|------------------|---------|
| P1 | Critical | Complete service outage or data loss affecting all users | 5 minutes | Every 15 minutes | Database cluster down, production data corruption |
| P2 | Major | Significant degradation affecting majority of users | 15 minutes | Every 30 minutes | Payment processing failing for 50% of requests |
| P3 | Minor | Feature degradation affecting subset of users | 1 hour | Every 2 hours | Search results slow, non-critical feature broken |
| P4 | Low | Cosmetic issues or minor bugs with workarounds | Next business day | Daily | UI alignment issue, logging error in non-critical path |

```
                        Severity Decision Flow
                        ======================

                    Is the service completely down?
                              |
              +---------------+---------------+
              |                               |
             YES                             NO
              |                               |
        Is there data loss?         Is a major feature broken
              |                     for most users?
      +-------+-------+                   |
      |               |            +------+------+
     YES             NO           |             |
      |               |           YES           NO
     P1              P1           |             |
                          Is revenue impacted?  Is a feature broken
                              |                 for a subset?
                      +-------+-------+         |
                      |               |    +----+----+
                     YES             NO   |         |
                      |               |   YES       NO
                     P2              P2   |         |
                                   P3    P3        P4
```

### Why This Works

The decision tree forces a top-down evaluation. You start with the most
impactful question (complete outage?) and work down. This prevents the
common mistake of starting at the bottom and arguing about edge cases.
The binary yes/no structure eliminates ambiguity -- each path leads to
exactly one severity level.

## Part B: Classification of 10 Scenarios

### Scenario Classification Table

| # | Scenario | Severity | Justification |
|---|----------|----------|---------------|
| 1 | Production database is completely unreachable | P1 | Complete service outage affecting 100% of users. No workaround exists. Data integrity may be at risk. |
| 2 | Payment processing fails for 30% of transactions | P2 | Direct revenue impact. Not a complete outage (70% still work), but significant business impact. |
| 3 | Search results return in 8 seconds instead of 200ms | P3 | Feature is degraded but functional. Users can still browse and purchase. Affects user experience but not core functionality. |
| 4 | Login page displays misaligned logo on mobile Safari | P4 | Cosmetic issue. Does not affect functionality. Workaround: users can still log in. |
| 5 | API rate limiter is stuck, blocking all requests | P1 | Complete service outage. Every API consumer is affected. No traffic can flow. |
| 6 | Email notifications are delayed by 45 minutes | P3 | Non-critical feature degraded. Core service works. Users can check the app directly. |
| 7 | Customer data from Account A visible to Account B | P1 | Data breach / security incident. Regardless of scope, any unauthorized data exposure is P1. Regulatory and legal implications. |
| 8 | Internal admin dashboard is down | P3 | Internal tool outage. Does not directly affect end users. Operations team has workarounds (CLI, direct DB queries). |
| 9 | CDN is serving stale content for 10% of edge locations | P2 | Partial content delivery failure. Some users see outdated information. Affects trust and could cause confusion. |
| 10 | Monitoring alerts are not firing for non-critical services | P3 | Observability gap. Services are running, but team has reduced visibility. Risk of missing future incidents increases. |

### Why This Works

Each classification considers three dimensions: **breadth** (how many users
are affected), **depth** (how severely each user is affected), and **business
impact** (revenue, reputation, compliance). A security incident like Scenario 7
is always P1 regardless of user count because the business and legal
consequences are severe.

## Part C: Escalation Paths by Severity

```
P1 Escalation Path (Critical):
==============================
  On-Call Engineer (0 min)
        |
        v  [5 min, no ack]
  Secondary On-Call (5 min)
        |
        v  [10 min, no ack]
  Engineering Manager (10 min)
        |
        v  [15 min, no ack]
  VP of Engineering (15 min)
        |
        v  [30 min, unresolved]
  CTO / CEO (30 min)

  Auto-actions:
  - Create incident Slack channel: #inc-<timestamp>-<short-desc>
  - Page Incident Commander
  - Update status page: "Investigating"
  - Notify all stakeholders via email

P2 Escalation Path (Major):
============================
  On-Call Engineer (0 min)
        |
        v  [15 min, no ack]
  Secondary On-Call (15 min)
        |
        v  [30 min, no ack]
  Engineering Manager (30 min)
        |
        v  [2 hours, unresolved]
  VP of Engineering (2 hours)

  Auto-actions:
  - Create incident Slack channel
  - Update status page: "Investigating"
  - Notify affected team leads

P3 Escalation Path (Minor):
============================
  On-Call Engineer (0 min)
        |
        v  [1 hour, no ack]
  Team Lead (1 hour)
        |
        v  [4 hours, unresolved]
  Engineering Manager (4 hours)

  Auto-actions:
  - Create Jira ticket
  - Notify team Slack channel

P4 Escalation Path (Low):
==========================
  Queue for next sprint planning
        |
        v  [1 week, unaddressed]
  Team Lead review

  Auto-actions:
  - Create Jira ticket with "low-priority" label
```

### Why This Works

The escalation delays are calibrated to severity. P1 has the shortest delays
(5 minutes) because every minute of full outage costs revenue and trust. P4
has no active escalation because it does not warrant waking anyone up. The
auto-actions ensure that even if a human forgets a step, the system
handles the mechanical parts (channel creation, status page, notifications).

## Part D: Communication Requirements Per Severity

| Requirement | P1 | P2 | P3 | P4 |
|-------------|----|----|----|-----|
| Incident Commander | Required | Required | Optional | Not needed |
| Dedicated Slack channel | Required | Required | Optional | Not needed |
| Status page update | Within 5 min | Within 15 min | If customer-facing | No |
| Customer email | If >15 min | If >1 hour | Only if requested | No |
| Executive notification | Immediate | Within 30 min | No | No |
| Post-incident review | Required within 48h | Required within 1 week | Optional | No |
| External vendor notification | If applicable | If applicable | No | No |
| War room / bridge call | Required | If needed | No | No |
| Timeline documentation | Every 5 min | Every 15 min | Start and end | No |

### Why This Works

Communication requirements scale with severity. A P1 demands constant
communication because stakeholders are actively losing money and trust. A P4
needs almost no communication because the impact is negligible. The specific
time bounds (5 minutes, 15 minutes) remove ambiguity about "communicate
frequently."

## Common Mistakes

1. **Starting severity debate during the incident.** The severity matrix must
   be agreed upon before incidents happen. Arguing about P1 vs P2 while the
   service is down wastes critical time. Default to the higher severity when
   in doubt -- you can always downgrade later.

2. **Confusing internal impact with customer impact.** An internal dashboard
   being down (Scenario 8) might feel urgent to the ops team, but it is a P3
   because customers are unaffected. Severity is always measured from the
   customer's perspective.

3. **Treating security incidents by user count.** A data breach affecting one
   user is still a P1. Security incidents follow different rules -- the
   potential for escalation, regulatory penalties, and reputational damage
   means they must always be treated as critical.

4. **Skipping escalation because "we're already working on it."** The
   escalation path exists for a reason. Even if the on-call engineer is
   actively debugging, the secondary should still be paged at the designated
   time. Solo debugging leads to tunnel vision.

5. **Not updating the status page for P3/P4 issues.** Even minor issues
   deserve a status page update if they are customer-facing. Customers notice
   slow search results, and a status page entry prevents duplicate support
   tickets.

## Key Takeaway

Severity classification is a decision framework, not a judgment call. Define
your matrix in advance, automate the escalation paths, and default to higher
severity when uncertain. The cost of over-escalating a P3 to P2 is a few
extra notifications; the cost of under-escalating a P1 to P2 is extended
outage time and lost revenue.
