# Solution 03: Incident Communication Templates

## Part A: Status Page Update Templates

Status page updates are the public-facing communication during an incident.
They must be clear, honest, and timely. Customers do not expect perfection --
they expect transparency.

### Stage 1: Investigating

```
Subject: Investigating: API Response Time Degradation

Status: Investigating
Severity: Major
Started: 2024-03-15 14:32 UTC
Affected: API Services, Web Application

We are currently investigating reports of increased API response times.
Some users may experience slower page loads and timeouts when accessing
the platform.

Our engineering team has been notified and is actively investigating
the root cause. We will provide updates every 15 minutes until the
issue is resolved.

Workaround: None available at this time.

Next update: 2024-03-15 14:47 UTC
```

### Why This Works

The "Investigating" status tells customers three things: we know something is
wrong, we are working on it, and here is when you will hear from us next. The
specific next-update time creates accountability -- if the team does not update
by then, customers know something has changed (for better or worse). The
"Workaround" section is critical: even saying "None available" is better than
leaving customers guessing whether they can do anything.

### Stage 2: Identified

```
Subject: Identified: API Response Time Degradation - Database Connection Pool Exhaustion

Status: Identified
Severity: Major
Started: 2024-03-15 14:32 UTC
Identified: 2024-03-15 14:51 UTC
Affected: API Services, Web Application

Root cause identified: The API service is experiencing database connection
pool exhaustion due to a long-running query introduced in this morning's
deployment (v2.14.3). The query holds connections longer than expected,
causing the pool to saturate under normal traffic.

We are currently implementing a fix by:
1. Rolling back the deployment to v2.14.2
2. Increasing the connection pool size as an immediate mitigation

Estimated time to resolution: 30 minutes

Workaround: Retry failed requests. The issue is intermittent -- some
requests succeed while the connection pool recovers between peaks.

Next update: 2024-03-15 15:06 UTC
```

### Why This Works

The "Identified" status tells customers: we know what is wrong and we know how
to fix it. The root cause explanation is written in plain language, not
internal jargon. The numbered action items show progress. The estimated
resolution time gives customers a concrete expectation, even if it is
approximate. The workaround is now specific because the root cause is known.

### Stage 3: Monitoring

```
Subject: Monitoring: API Response Time Degradation - Fix Deployed

Status: Monitoring
Severity: Major
Started: 2024-03-15 14:32 UTC
Identified: 2024-03-15 14:51 UTC
Resolved: Monitoring
Affected: API Services, Web Application

The rollback to v2.14.2 has been completed and the connection pool size
has been increased. API response times have returned to normal levels.

We are monitoring the system to ensure stability before marking this
incident as resolved. Our team will continue to observe metrics for
the next 60 minutes.

Actions taken:
1. Rolled back deployment from v2.14.3 to v2.14.2 (completed 15:03 UTC)
2. Increased database connection pool from 20 to 50 (completed 15:08 UTC)
3. Verified API response times returned to baseline (confirmed 15:12 UTC)

Next update: 2024-03-15 15:47 UTC or sooner if status changes
```

### Why This Works

The "Monitoring" status tells customers: the fix is in, we are watching it.
The key phrase is "before marking this incident as resolved" -- this signals
that the team is being careful, not just declaring victory prematurely. The
action items with timestamps create an audit trail. The "or sooner if status
changes" clause ensures that if the problem recurs, customers will not wait
until the scheduled update time.

### Stage 4: Resolved

```
Subject: Resolved: API Response Time Degradation

Status: Resolved
Severity: Major
Started: 2024-03-15 14:32 UTC
Identified: 2024-03-15 14:51 UTC
Resolved: 2024-03-15 16:15 UTC
Duration: 1 hour 43 minutes
Affected: API Services, Web Application

This incident has been resolved. API response times have been stable
for over 60 minutes and all services are operating normally.

Summary:
A database connection pool exhaustion issue caused by a long-running
query in deployment v2.14.3 led to degraded API performance for
approximately 1 hour and 43 minutes. The issue was resolved by rolling
back the deployment and increasing the connection pool size.

Preventive measures:
1. Adding connection pool monitoring alerts (target: 1 week)
2. Implementing query performance gates in CI/CD pipeline (target: 2 weeks)
3. Conducting post-incident review (scheduled: 2024-03-17 10:00 UTC)

We apologize for the disruption and are committed to preventing similar
issues in the future.

Next update: Post-incident review summary will be published within 5
business days.
```

### Why This Works

The "Resolved" status is the most important communication. It must include:
what happened (summary), how long it lasted (duration), what was done to fix
it (actions), and what will prevent it from happening again (preventive
measures). The commitment to a post-incident review with a specific date
shows accountability. The apology is genuine and specific -- not "we apologize
for any inconvenience" but "we apologize for the disruption."

## Part B: Slack Channel Messages

### Incident Channel Creation Message

```
============================================================
  INCIDENT DECLARED: API Response Time Degradation
============================================================

Severity: P2 (Major)
Incident Commander: @alice-chen
Technical Lead: @bob-martinez
Started: 2024-03-15 14:32 UTC

Channel: #inc-20240315-api-degradation

Status Page: https://status.example.com/incidents/12345
Bridge Call: https://meet.google.com/abc-defg-hij

Roles:
  - Incident Commander: Coordinates response, manages communication
  - Technical Lead: Drives debugging and resolution
  - Scribe: Documents timeline in this channel

Please use this channel for all incident-related communication.
Keep noise low -- react with :eyes: to acknowledge, post only updates.
============================================================
```

### Timeline Entry Messages

```
[14:32 UTC] INCIDENT DECLARED
  - Multiple customers reporting slow API responses
  - PagerDuty alert triggered: API p99 latency > 5s
  - On-call engineer @bob-martinez investigating
  - Status page updated: "Investigating"

[14:38 UTC] INITIAL ASSESSMENT
  - Confirmed: API p99 latency is 8.2s (normal: 200ms)
  - Error rate: 12% (normal: 0.1%)
  - Database CPU at 95%, connection pool exhausted
  - Affects: All API endpoints

[14:45 UTC] ROOT CAUSE IDENTIFIED
  - Deploy v2.14.3 at 10:00 UTC introduced slow query
  - Query: SELECT * FROM orders JOIN items (missing index)
  - Connection pool saturates under load, requests queue
  - @alice-chen: "Initiating rollback to v2.14.2"

[14:51 UTC] STATUS PAGE UPDATED
  - Status changed to "Identified"
  - Root cause communicated to customers
  - ETA for fix: 30 minutes

[15:03 UTC] ROLLBACK COMPLETE
  - v2.14.2 deployed to all pods
  - Connection pool size increased from 20 to 50
  - Database CPU dropping: 78% and falling

[15:12 UTC] RECOVERY CONFIRMED
  - API p99 latency: 180ms (below baseline)
  - Error rate: 0.08% (normal)
  - Database CPU: 45% (normal)
  - All systems nominal

[15:15 UTC] STATUS PAGE UPDATED
  - Status changed to "Monitoring"
  - 60-minute observation window started

[16:15 UTC] INCIDENT RESOLVED
  - 60-minute observation complete, no recurrence
  - Status page updated to "Resolved"
  - Post-incident review scheduled for 2024-03-17
  - @alice-chen: "Great work everyone. PIR on Monday."
```

### Why This Works

The Slack timeline serves two purposes: real-time coordination and
post-incident documentation. Each entry has a timestamp, a clear description,
and specific data points. The scribe role is critical -- without someone
dedicated to documenting, the team will forget details after the adrenaline
fades. The timeline entries are written in third person ("On-call engineer
investigating") so they read as a log, not a conversation.

## Part C: Stakeholder Notification Email

```
Subject: [P2 Incident] API Performance Degradation - Resolved

To: engineering-all@, product-leads@, support@
Cc: exec-team@

Team,

We experienced a P2 incident today affecting API performance. Here is
the summary:

WHAT: API response times degraded from 200ms to 8+ seconds
WHEN: March 15, 2024, 14:32 - 16:15 UTC (1 hour 43 minutes)
IMPACT: ~12% of API requests failed; all users experienced slowness
ROOT CAUSE: A database query introduced in v2.14.3 caused connection
             pool exhaustion
RESOLUTION: Rolled back to v2.14.2 and increased connection pool size

CUSTOMER IMPACT:
- 47 support tickets opened during the incident
- 3 enterprise customers requested incident reports
- Estimated revenue impact: ~$12,000 in failed transactions

PREVENTIVE MEASURES:
1. Connection pool monitoring alerts (ETA: 1 week)
2. Query performance gates in CI/CD (ETA: 2 weeks)
3. Post-incident review: March 17, 10:00 UTC (all welcome)

The full post-incident report will be shared within 5 business days.

Please direct customer inquiries to the support team. The status page
has been updated with the final resolution notice.

- Alice Chen, Incident Commander
```

### Why This Works

The stakeholder email follows the "BLUF" (Bottom Line Up Front) format:
what happened, when, how long, what was the impact, and what are we doing
about it. The executive summary at the top lets busy leaders get the key
facts in 10 seconds. The customer impact section quantifies the damage in
terms that matter to the business (support tickets, revenue). The preventive
measures have specific owners and dates, not vague promises.

## Part D: Template Customization Guide

```
Template Variables Reference
============================

{{incident_id}}      - Unique incident identifier (e.g., INC-2024-0315-001)
{{severity}}         - P1, P2, P3, or P4
{{status}}           - Investigating, Identified, Monitoring, Resolved
{{started_at}}       - ISO 8601 timestamp when incident began
{{identified_at}}    - ISO 8601 timestamp when root cause found
{{resolved_at}}      - ISO 8601 timestamp when incident resolved
{{duration}}         - Human-readable duration
{{affected_services}} - Comma-separated list of affected services
{{root_cause}}       - Plain-language description of root cause
{{workaround}}       - Available workaround or "None available"
{{next_update}}      - ISO 8601 timestamp for next scheduled update
{{ic_name}}          - Incident Commander name
{{tl_name}}          - Technical Lead name
{{pir_date}}         - Post-incident review date

Severity-Specific Adjustments:
  P1: Add executive summary, revenue impact, customer count
  P2: Add service impact details, estimated resolution time
  P3: Shorter format, fewer updates, no executive notification
  P4: Minimal format, internal tracking only
```

### Why This Works

Template variables ensure consistency across incidents. When an engineer is
debugging a production issue at 2am, they should not be crafting prose from
scratch -- they should fill in blanks. The severity-specific adjustments
prevent over-communicating for minor issues (no one needs an executive email
for a P4) and under-communicating for major ones (a P1 without revenue impact
numbers will get escalated immediately).

## Common Mistakes

1. **Writing technical jargon in customer-facing updates.** "Connection pool
   exhaustion due to missing index on orders table" means nothing to a
   customer. "A database issue is causing slower response times" is clear.
   Save the technical details for the post-incident review.

2. **Not committing to a next-update time.** "We will update you soon" is
   not a commitment. "Next update at 14:47 UTC" is. Customers can plan around
   a specific time. If you miss it, they know to be concerned.

3. **Declaring resolution too early.** The "Monitoring" stage exists for a
   reason. If you jump from "Identified" to "Resolved" without observation,
   you risk having to reopen the incident. Reopened incidents destroy trust
   far more than a longer monitoring period.

4. **Forgetting the scribe role during the incident.** Without a dedicated
   timeline documenter, the post-incident review will rely on memory and
   Slack scroll-back, both of which are unreliable under stress.

5. **Not quantifying customer impact.** "Some users were affected" is
   meaningless. "12% of API requests failed, affecting approximately 3,400
   users" is actionable. Impact quantification drives prioritization of
   preventive measures.

## Key Takeaway

Incident communication is a skill, not an afterthought. Use templates to
ensure consistency, commit to specific update times, write for your audience
(technical for engineers, plain language for customers), and always include
the next step. The goal is not to make the incident look better than it is --
it is to give stakeholders the information they need to make decisions.
