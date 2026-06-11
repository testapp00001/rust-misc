# Solution 03: Write a Complete Post-Mortem

## Exercise Recap

Write a complete post-mortem document for the following incident: A
payment processing service experienced a 23-minute outage during peak
traffic (Black Friday) caused by a connection pool exhaustion after a
dependency returned slow responses. The root cause was a missing circuit
breaker and no connection pool monitoring.

## Complete Post-Mortem Document

---

### Post-Mortem: Payment Service Outage During Black Friday

#### Metadata

| Field | Value |
|-------|-------|
| **Date of Incident** | 2025-11-28 |
| **Duration** | 23 minutes (14:12 UTC -- 14:35 UTC) |
| **Severity** | SEV-1 (Critical -- revenue-impacting) |
| **Author** | Jane Martinez, Senior SRE |
| **Reviewers** | Alex Chen (Payments Lead), Sam Okafor (Platform SRE), Priya Kapoor (Engineering Manager) |
| **Status** | Final |
| **Post-Mortem Date** | 2025-12-02 |
| **Incident Commander** | Raj Patel |
| **Affected Services** | payment-service, checkout-api, order-service |
| **Affected Customers** | All customers attempting to complete purchases |
| **Revenue Impact** | Estimated $340,000 in abandoned carts |

#### Executive Summary

On November 28, 2025 (Black Friday), the payment processing service
experienced a 23-minute outage during peak traffic hours. The outage
was caused by connection pool exhaustion in the payment service, which
occurred after the upstream fraud detection service began responding
slowly (p99 latency increased from 200ms to 12 seconds). Because the
payment service had no circuit breaker and no connection pool monitoring,
slow fraud detection responses held open database connections, exhausting
the pool and causing all new payment requests to fail.

The incident was detected by a customer support escalation (not by
monitoring) and was resolved by manually restarting the payment service
pods. Total revenue impact is estimated at $340,000 in abandoned carts
during the 23-minute window.

#### Impact

**Customer Impact:**

- All customers attempting to check out between 14:12 UTC and 14:35 UTC
  received a generic error message and could not complete their purchase.
- Customers who had items in their carts during the outage experienced
  cart expiration due to high demand, losing their reserved inventory.
- Estimated 12,000 failed checkout attempts during the outage window.

**Business Impact:**

- Estimated $340,000 in abandoned carts (calculated from average cart
  value during the hour before the outage, multiplied by failed attempts).
- Customer support received 847 tickets related to the outage within
  the first hour after resolution.
- Social media mentions of checkout failures spiked 340% during the
  outage window.

**Technical Impact:**

- payment-service: 100% error rate for 23 minutes
- checkout-api: 100% error rate for 23 minutes (downstream of payment)
- order-service: Elevated error rate (78%) for 23 minutes
- No data loss or corruption occurred
- No security implications

#### Timeline

All times in UTC.

| Time | Event |
|------|-------|
| 13:45 | Black Friday traffic begins ramping up. All services nominal. |
| 13:58 | Fraud detection service p99 latency begins increasing (from 200ms to 800ms). No alerts triggered -- latency thresholds set at 5s. |
| 14:05 | Fraud detection service p99 latency reaches 3 seconds. Still within alerting threshold. Payment service connection pool utilization at 60% (normal). |
| 14:10 | Fraud detection service p99 latency reaches 8 seconds. Payment service connection pool utilization at 85%. |
| 14:12 | Payment service connection pool fully exhausted (100/100 connections in use). New payment requests begin failing with "connection pool exhausted" errors. **OUTAGE BEGINS.** |
| 14:14 | checkout-api begins returning 500 errors to all customers. |
| 14:18 | Customer support receives first "can't check out" tickets. |
| 14:22 | Customer support escalates to on-call engineer (Maria Gonzalez). |
| 14:24 | Maria begins investigating. Sees payment-service errors in logs. |
| 14:28 | Maria identifies connection pool exhaustion as the proximate cause. |
| 14:30 | Maria restarts all payment-service pods (rolling restart). |
| 14:35 | All pods restarted. Connection pool resets. Payment processing resumes. **OUTAGE ENDS.** |
| 14:40 | Fraud detection service latency returns to normal (200ms). Root cause of the latency spike: a batch job was running against the fraud detection database, causing lock contention. |

#### Root Cause Analysis

**5 Whys Analysis:**

```
WHY 1: Why did payments fail?
  -> Payment service connection pool was exhausted.

WHY 2: Why was the connection pool exhausted?
  -> All 100 connections were held open waiting for fraud detection
     responses that were taking 8-12 seconds each.

WHY 3: Why did the fraud detection service respond slowly?
  -> A scheduled batch job was running against the fraud detection
     database during peak hours, causing lock contention.

WHY 4: Why did slow fraud detection responses exhaust the payment
        service connection pool?
  -> The payment service had no circuit breaker to stop sending
     requests to a degraded dependency, and no connection pool
     timeout that would release connections after a reasonable
     wait.

WHY 5: Why was there no circuit breaker or connection pool monitoring?
  -> The payment service was built before the team adopted resilience
     patterns. The original design assumed all dependencies would
     respond within their SLA. There was no architectural review
     checklist that requires circuit breakers for external
     dependencies.
```

**Root Cause:**

The payment service was designed without resilience patterns (circuit
breaker, bulkhead, connection pool monitoring) because the architectural
review process at the time of design did not require them. When a
dependency degraded, the failure cascaded through the payment service
due to resource exhaustion rather than being contained.

#### Contributing Factors

1. **No circuit breaker pattern.** The payment service sent requests
   to the fraud detection service without any mechanism to stop
   sending requests when the dependency was degraded.

2. **No connection pool monitoring.** The connection pool had no
   metrics, no alerts, and no dashboard. The team had no visibility
   into pool utilization until it was fully exhausted.

3. **Batch job during peak hours.** The fraud detection team scheduled
   a batch job during Black Friday traffic, unaware that it would
   cause lock contention on the database.

4. **Insufficient latency alerting.** The fraud detection service
   latency alert threshold was set at 5 seconds, which is 25x the
   normal p99. The alert did not fire until the service was already
   severely degraded.

5. **No dependency health checks.** The payment service did not
   perform health checks on the fraud detection service before
   sending requests.

6. **Manual incident detection.** The incident was detected by customer
   support escalation 10 minutes after the outage began, not by
   automated monitoring.

#### What Went Well

1. **Fast resolution once identified.** From the time the on-call
   engineer was paged (14:22) to resolution (14:35) was 13 minutes.
   The engineer correctly identified the root cause quickly.

2. **No data corruption.** Despite the outage, no payment data was
   lost or corrupted. Failed transactions were cleanly rejected.

3. **Customer support escalation worked.** The support team identified
   the pattern quickly and escalated appropriately.

4. **Rolling restart was effective.** The manual restart resolved the
   issue cleanly without requiring a rollback or configuration change.

5. **Post-mortem was blameless.** The team focused on systemic issues
   rather than individual blame.

#### What Went Poorly

1. **Detection took 10 minutes.** The outage started at 14:12 but was
   not detected until 14:22 (customer support escalation). Automated
   monitoring should have caught this within 1 minute.

2. **No automated mitigation.** A circuit breaker would have
   automatically isolated the degraded fraud detection service and
   prevented the connection pool exhaustion entirely.

3. **No connection pool visibility.** The team had no way to see
   connection pool utilization, making diagnosis slower than necessary.

4. **Batch job scheduling conflict.** No process existed to prevent
   high-risk batch jobs from running during peak traffic windows.

5. **Alerting thresholds too permissive.** The 5-second latency
   threshold for the fraud detection service was far too high,
   providing no early warning of degradation.

#### Action Items

| # | Action Item | Owner | Priority | Due Date | Status |
|---|------------|-------|----------|----------|--------|
| 1 | Implement circuit breaker in payment service for fraud detection dependency | Alex Chen | P0 | 2025-12-15 | In Progress |
| 2 | Add connection pool metrics and alerts (utilization > 70%, wait time > 1s) | Maria Gonzalez | P0 | 2025-12-10 | In Progress |
| 3 | Lower fraud detection latency alert threshold to 1 second p99 | Sam Okafor | P0 | 2025-12-05 | Done |
| 4 | Implement batch job scheduling policy: no non-critical batch jobs during peak traffic (09:00-21:00 on high-traffic days) | Priya Kapoor | P1 | 2025-12-20 | Not Started |
| 5 | Add automated payment service health check that triggers page on > 5% error rate | Maria Gonzalez | P0 | 2025-12-10 | In Progress |
| 6 | Conduct architectural review of all services for missing resilience patterns (circuit breaker, bulkhead, timeout) | Alex Chen | P1 | 2026-01-15 | Not Started |
| 7 | Add dependency health dashboard showing latency and error rates for all cross-service calls | Sam Okafor | P1 | 2026-01-05 | Not Started |
| 8 | Implement connection pool auto-scaling based on utilization metrics | Alex Chen | P2 | 2026-02-01 | Not Started |

#### Lessons Learned

1. **Resilience patterns are not optional.** Any service that depends
   on an external service must implement circuit breakers, timeouts,
   and bulkheads. This should be a non-negotiable architectural
   requirement enforced by review checklists.

2. **Resource pools need monitoring.** Connection pools, thread pools,
   and other finite resources must have utilization metrics, alerts,
   and dashboards. Exhaustion of these resources is a common failure
   mode that is easily detectable with proper monitoring.

3. **Peak traffic requires peak caution.** Scheduling batch jobs,
   deployments, or other risky operations during peak traffic windows
   dramatically increases the blast radius of any failure.

4. **Alerting thresholds should provide early warning.** An alert
   threshold of 25x the normal value provides no early warning.
   Thresholds should be set to trigger on meaningful deviation from
   normal, not on extreme values.

5. **Customer support is not a monitoring system.** Detecting incidents
   through customer complaints is the worst-case detection method.
   Automated monitoring must detect incidents before customers are
   affected.

---

## Why This Structure Works

### Metadata

The metadata table provides a single place to find all factual information
about the incident. It eliminates the need to search through the document
for dates, severity, or involved parties. Every post-mortem should be
findable and sortable by these fields.

### Executive Summary

The executive summary answers three questions in one paragraph:
- What happened?
- Why did it happen?
- What was the impact?

Executives and stakeholders who do not need the full technical detail
can stop here. Engineers who need to understand the system behavior
continue to the timeline and root cause.

### Impact

Separating customer impact, business impact, and technical impact
ensures that all stakeholders see the dimensions relevant to them.
The revenue impact number is critical for prioritizing action items --
a $340,000 impact justifies significant engineering investment in
prevention.

### Timeline

The timeline is the factual backbone of the post-mortem. It should:
- Start before the incident (show the ramp-up)
- Include all detection, diagnosis, and mitigation steps
- End after the incident (show the return to normal)
- Use precise times, not vague descriptions

### Root Cause Analysis

The 5 Whys chain provides a structured path from symptom to root cause.
Each "why" should be answerable with specific technical evidence from
the timeline.

### Contributing Factors

Contributing factors are the conditions that made the incident possible
or worse, but are not the single root cause. They represent the "Swiss
cheese holes" that all aligned.

### What Went Well / What Went Poorly

These sections create a balanced view. "What Went Well" prevents the
post-mortem from being purely negative and recognizes effective response.
"What Went Poorly" identifies areas for improvement without blaming
individuals.

### Action Items

Action items must be:
- **Specific**: "Implement circuit breaker" not "improve resilience"
- **Owned**: Every item has a single owner
- **Prioritized**: P0 for immediate, P1 for this quarter, P2 for later
- **Dated**: Every item has a due date
- **Tracked**: Status is updated until completion

### Lessons Learned

Lessons are generalized principles, not specific to this incident.
They should be applicable to future design decisions and other
services.

## Common Mistakes to Avoid

1. **Missing the "What Went Well" section.** Post-mortems that only
   focus on failures demoralize teams and miss opportunities to
   reinforce good practices.

2. **Vague action items.** "Improve monitoring" is not an action item.
   "Add connection pool utilization metric with alert at 70%" is.

3. **No due dates on action items.** Action items without due dates
   never get done. Every item needs a date and a status tracker.

4. **Skipping the executive summary.** Without it, every reader must
   parse the full document to understand the incident.

5. **Incomplete timeline.** A timeline that starts at the outage misses
   the context that explains why the outage happened. Start from the
   conditions that preceded it.
