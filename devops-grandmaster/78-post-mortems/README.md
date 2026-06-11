# Module 78: Post-Mortems — Blameless Analysis and Continuous Improvement

> **Previous Module:** [77 - Runbooks & Playbooks](../77-runbooks-and-playbooks/README.md)
> **Next Module:** [79 - Change Management](../79-change-management/README.md)

## The Problem

The incident is resolved. Everyone goes back to work. Two weeks later, the same incident happens again. Why? Because nobody analyzed what went wrong, nobody identified the systemic cause, and nobody implemented preventive measures. The incident was "fixed" but the underlying problem remained.

The purpose of a post-mortem is not to assign blame — it is to **learn from failure and prevent recurrence**.

## The Naive Way

```bash
# "We fixed it, let's move on"
# Or: "It was Bob's fault, he deployed the bad code"
# Or: "We'll have a quick chat about it" (no documentation, no follow-up)
```

**Why this fails:**
- Blame creates fear, fear hides problems
- No documentation means no institutional learning
- No follow-up means the same incidents repeat
- No metrics means you cannot measure improvement
- "Quick chat" produces no actionable outcomes

## The Right Way

### Blameless Post-Mortem Culture

```
BLAMELESS MEANS:
  - No individual is blamed for the incident
  - Focus on systems and processes, not people
  - Assume everyone made the best decisions with the information they had
  - Ask "why did the system allow this?" not "why did you do this?"
  - Create psychological safety to report problems

WHY BLAMELESS:
  - People who fear blame hide problems
  - Hidden problems become bigger problems
  - Blame does not prevent recurrence
  - Systemic fixes prevent recurrence
```

### Post-Mortem Template

```markdown
# Post-Mortem: [Incident Title]

## Metadata
**Incident ID:** INC-20260115-031700
**Date:** 2026-01-15
**Duration:** 3 hours 47 minutes (03:17 - 07:04)
**Severity:** P1
**Incident Commander:** @engineer-1
**Author:** @engineer-2
**Status:** Draft / Under Review / Final

## Executive Summary
On January 15, 2026, at 03:17 UTC, the production PostgreSQL database ran out of
disk space, causing a complete outage of the API and web application. The outage
lasted 3 hours and 47 minutes, affecting approximately 50,000 users and resulting
in an estimated $150,000 in lost revenue.

## Impact
- **Users affected:** ~50,000 (all users during outage window)
- **Revenue impact:** ~$150,000 (estimated based on hourly GMV)
- **Data loss:** None
- **SLA breach:** Yes (99.9% SLA = 43.8 min/month; actual: 227 min)
- **Downstream services:** Payment processing, order management, user authentication

## Timeline
| Time (UTC) | Event |
|------------|-------|
| 02:30 | Disk usage reaches 90% (warning threshold) |
| 02:45 | Alert fires but is auto-resolved (disk temporarily drops below 90%) |
| 03:17 | Disk usage reaches 100%. PostgreSQL stops accepting writes. |
| 03:17 | PagerDuty alert fires: "Database disk full" |
| 03:19 | On-call engineer @engineer-1 acknowledges alert |
| 03:22 | IC creates incident channel #incident-20260115-database |
| 03:25 | Status page updated: Investigating |
| 03:30 | Root cause identified: audit_logs table grew to 200GB |
| 03:35 | Decision: Failover to replica (has more disk) |
| 03:40 | Failover initiated |
| 03:45 | Failover complete, replica promoted to primary |
| 03:50 | API starts accepting requests again |
| 04:00 | Status page updated: Monitoring |
| 04:15 | Verification complete, service fully restored |
| 04:15 | Status page updated: Resolved |
| 07:04 | Disk cleanup on old primary complete (old audit logs archived) |

## Root Cause Analysis

### 5 Whys
1. **Why did the database go down?**
   Disk space was exhausted.

2. **Why was disk space exhausted?**
   The audit_logs table grew to 200GB.

3. **Why did audit_logs grow so large?**
   Log retention policy was set to "forever" — no automatic cleanup.

4. **Why was there no automatic cleanup?**
   The retention policy was never implemented when the feature was built.

5. **Why was it never implemented?**
   The original requirement specified "retain all audit logs for compliance"
   but did not specify a retention period or archival strategy.

### Root Cause
The audit logging feature was implemented without a data retention strategy.
Logs accumulated indefinitely, eventually exhausting disk space. The monitoring
alert at 90% was configured but was not actionable (no runbook, no auto-remediation).

## Contributing Factors
1. **No data retention policy:** Audit logs had no expiration or archival.
2. **Alert fatigue:** The 90% warning alert had fired and auto-resolved multiple
   times in the past, leading to desensitization.
3. **No runbook:** No documented procedure for "disk full" scenario.
4. **Insufficient monitoring:** No trend-based alerting to predict disk exhaustion.
5. **No capacity planning:** No regular review of disk usage trends.

## What Went Well
- Incident was detected quickly (within 2 minutes of impact)
- On-call engineer responded within SLA (< 5 minutes)
- Root cause was identified within 13 minutes
- Failover to replica was successful with no data loss
- Communication was timely (status page updated 4 times)

## What Went Poorly
- No runbook existed for disk full scenario
- Alert was not actionable (fired multiple times before without resolution)
- No automated remediation (could have archived old logs automatically)
- Post-failover verification took 25 minutes (manual checks)
- No capacity forecasting to predict the issue

## Action Items
| # | Action | Owner | Priority | Due Date | Status |
|---|--------|-------|----------|----------|--------|
| 1 | Implement audit log retention policy (archive logs > 90 days) | @dba-team | P1 | 2026-01-22 | In Progress |
| 2 | Create runbook for disk full scenario | @platform-team | P1 | 2026-01-19 | Done |
| 3 | Add trend-based disk usage alerting (predict exhaustion) | @sre-team | P2 | 2026-01-29 | In Progress |
| 4 | Implement automated disk cleanup for known large tables | @dba-team | P2 | 2026-02-05 | Not Started |
| 5 | Review all alerts for actionability | @sre-team | P3 | 2026-02-12 | Not Started |
| 6 | Establish quarterly capacity planning review | @engineering-manager | P3 | 2026-02-28 | Not Started |

## Lessons Learned
1. **Data without retention is a ticking time bomb.** Every data-generating
   feature must have a retention strategy from day one.
2. **Alerts must be actionable.** An alert that fires without a clear response
   creates noise and fatigue.
3. **Failover is not a fix.** It is a stopgap. The root cause (disk space) must
   still be addressed.
4. **Capacity planning is not optional.** Regular review of resource trends
   prevents surprises.

## Appendix
- Incident Slack channel: #incident-20260115-database
- Grafana dashboard: https://grafana.internal/d/database-metrics
- PagerDuty incident: https://pagerduty.com/incidents/PXXXXXX
```

### 5 Whys Root Cause Analysis

```
5 WHYS TECHNIQUE:

Problem: The API returned 500 errors for 3 hours.

Why 1: Why did the API return 500 errors?
  Because the database was down.

Why 2: Why was the database down?
  Because the primary ran out of disk space.

Why 3: Why did it run out of disk space?
  Because the audit_logs table grew to 200GB.

Why 4: Why did the audit_logs table grow so large?
  Because there was no retention policy.

Why 5: Why was there no retention policy?
  Because the requirement said "retain all logs" without specifying a period,
  and nobody challenged the requirement during design review.

ROOT CAUSE: Incomplete requirements gathering and no design review process
that questions operational implications of feature requirements.

FIX: Add operational review to feature design process. Every data-generating
feature must specify retention, archival, and capacity requirements.
```

### Post-Mortem Metrics

```python
# postmortem_metrics.py — Track post-mortem metrics over time
from dataclasses import dataclass
from datetime import datetime
from typing import List
import json

@dataclass
class IncidentMetric:
    incident_id: str
    date: datetime
    severity: str
    detection_time_minutes: float    # Time from impact to detection
    response_time_minutes: float     # Time from detection to acknowledgment
    resolution_time_minutes: float   # Time from acknowledgment to resolution
    root_cause_category: str         # deployment, capacity, dependency, config, code
    action_items_total: int
    action_items_completed: int

class PostMortemTracker:
    def __init__(self):
        self.incidents: List[IncidentMetric] = []

    def add_incident(self, metric: IncidentMetric):
        self.incidents.append(metric)

    def generate_report(self) -> str:
        if not self.incidents:
            return "No incidents tracked."

        report = "\n=== POST-MORTEM METRICS REPORT ===\n\n"

        # MTTR (Mean Time To Resolution)
        mttr = sum(i.resolution_time_minutes for i in self.incidents) / len(self.incidents)
        report += f"MTTR (Mean Time To Resolution): {mttr:.1f} minutes\n"

        # MTTD (Mean Time To Detection)
        mttd = sum(i.detection_time_minutes for i in self.incidents) / len(self.incidents)
        report += f"MTTD (Mean Time To Detection): {mttd:.1f} minutes\n"

        # MTTR by severity
        for severity in ['P1', 'P2', 'P3', 'P4']:
            sev_incidents = [i for i in self.incidents if i.severity == severity]
            if sev_incidents:
                sev_mttr = sum(i.resolution_time_minutes for i in sev_incidents) / len(sev_incidents)
                report += f"MTTR ({severity}): {sev_mttr:.1f} minutes\n"

        # Incident frequency
        report += f"\nTotal incidents: {len(self.incidents)}\n"

        # Root cause distribution
        categories = {}
        for i in self.incidents:
            categories[i.root_cause_category] = categories.get(i.root_cause_category, 0) + 1
        report += "\nRoot Cause Distribution:\n"
        for cat, count in sorted(categories.items(), key=lambda x: -x[1]):
            report += f"  {cat}: {count} ({count/len(self.incidents)*100:.0f}%)\n"

        # Action item completion rate
        total_actions = sum(i.action_items_total for i in self.incidents)
        completed_actions = sum(i.action_items_completed for i in self.incidents)
        if total_actions > 0:
            completion_rate = completed_actions / total_actions * 100
            report += f"\nAction Item Completion Rate: {completed_actions}/{total_actions} ({completion_rate:.0f}%)\n"

        # Trend analysis
        report += "\nMonthly Trend:\n"
        monthly = {}
        for i in self.incidents:
            month = i.date.strftime('%Y-%m')
            monthly[month] = monthly.get(month, 0) + 1
        for month, count in sorted(monthly.items()):
            report += f"  {month}: {count} incidents\n"

        return report


# Usage
tracker = PostMortemTracker()

tracker.add_incident(IncidentMetric(
    incident_id="INC-20260115",
    date=datetime(2026, 1, 15),
    severity="P1",
    detection_time_minutes=2,
    response_time_minutes=3,
    resolution_time_minutes=227,
    root_cause_category="capacity",
    action_items_total=6,
    action_items_completed=4
))

tracker.add_incident(IncidentMetric(
    incident_id="INC-20260120",
    date=datetime(2026, 1, 20),
    severity="P2",
    detection_time_minutes=5,
    response_time_minutes=10,
    resolution_time_minutes=45,
    root_cause_category="deployment",
    action_items_total=3,
    action_items_completed=3
))

print(tracker.generate_report())
```

## The Production Way

### Post-Mortem Review Process

```
POST-MORTEM PROCESS:

1. SCHEDULE (Within 24 hours of resolution)
   - P1/P2: Post-mortem meeting within 48 hours
   - P3/P4: Written post-mortem within 1 week

2. PREPARE (Before meeting)
   - Incident commander drafts timeline
   - Technical lead documents root cause analysis
   - All attendees review draft

3. CONDUCT (Meeting, 60-90 minutes)
   - Review timeline (15 min)
   - Discuss root cause (15 min)
   - What went well / What went poorly (15 min)
   - Generate action items (15 min)
   - Review previous action items (15 min)

4. PUBLISH (Within 24 hours of meeting)
   - Finalize post-mortem document
   - Share with engineering team
   - Add to post-mortem repository

5. FOLLOW UP (Ongoing)
   - Track action items to completion
   - Review at next post-mortem
   - Update runbooks and monitoring
```

### Blameless Language Guide

```
INSTEAD OF                          SAY
"Bob deployed the bad code"    -> "The deployment introduced a regression"
"Sarah forgot to update..."    -> "The process did not include a check for..."
"The team didn't monitor..."   -> "Monitoring was not configured for this scenario"
"Someone made a mistake"       -> "The system allowed this error to occur"
"That was a stupid decision"   -> "The decision was made with incomplete information"

FOCUS ON:
- What happened (facts)
- Why the system allowed it (processes)
- How to prevent it (improvements)

NOT ON:
- Who did it (blame)
- What they should have known (judgment)
- Why they are incompetent (character)
```

### Action Item Tracking

```yaml
# action-items.yaml — Track post-mortem action items
apiVersion: v1
kind: ConfigMap
metadata:
  name: postmortem-action-items
  namespace: engineering
data:
  items.yaml: |
    items:
      - id: AI-001
        incident: INC-20260115
        action: Implement audit log retention policy
        owner: dba-team
        priority: P1
        due_date: 2026-01-22
        status: completed
        completed_date: 2026-01-21

      - id: AI-002
        incident: INC-20260115
        action: Create runbook for disk full scenario
        owner: platform-team
        priority: P1
        due_date: 2026-01-19
        status: completed
        completed_date: 2026-01-18

      - id: AI-003
        incident: INC-20260115
        action: Add trend-based disk usage alerting
        owner: sre-team
        priority: P2
        due_date: 2026-01-29
        status: in_progress

      - id: AI-004
        incident: INC-20260120
        action: Add integration tests for deployment pipeline
        owner: backend-team
        priority: P2
        due_date: 2026-02-05
        status: not_started
```

## Hands-On Lab: Write a Post-Mortem for a Simulated Incident

### Step 1: Simulate an Incident

```bash
# Create a "production" environment
kubectl create namespace postmortem-lab

# Deploy an application
kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web
  namespace: postmortem-lab
spec:
  replicas: 3
  selector:
    matchLabels:
      app: web
  template:
    metadata:
      labels:
        app: web
    spec:
      containers:
        - name: web
          image: nginx:alpine
          ports:
            - containerPort: 80
EOF

kubectl expose deployment web -n postmortem-lab --port=80

# Inject a failure: bad deployment
kubectl set image deployment/web web=nginx:nonexistent -n postmortem-lab

# Observe the failure
kubectl get pods -n postmortem-lab
kubectl describe pods -n postmortem-lab | grep -A 5 "Events"
```

### Step 2: Resolve the Incident

```bash
# Fix the deployment
kubectl rollout undo deployment/web -n postmortem-lab

# Verify
kubectl rollout status deployment/web -n postmortem-lab
kubectl get pods -n postmortem-lab
```

### Step 3: Write the Post-Mortem

```markdown
# Post-Mortem: Bad Deployment Caused Pod CrashLoop

## Metadata
**Incident ID:** INC-POSTMORTEM-LAB
**Date:** 2026-06-11
**Duration:** 15 minutes
**Severity:** P3
**Incident Commander:** [Your name]

## Executive Summary
A deployment with a nonexistent container image caused all web pods to enter
CrashLoopBackOff state, making the service unavailable.

## Impact
- **Users affected:** All users (lab environment)
- **Duration:** 15 minutes
- **Data loss:** None

## Timeline
| Time | Event |
|------|-------|
| T+0 | Deployment triggered with image nginx:nonexistent |
| T+1 | Pods start failing with ImagePullBackOff |
| T+2 | Pods transition to CrashLoopBackOff |
| T+5 | Alert fires (simulated) |
| T+10 | Root cause identified: bad image tag |
| T+12 | Rollback initiated |
| T+15 | Service restored |

## Root Cause Analysis

### 5 Whys
1. Why were pods crashing? Image pull failed.
2. Why did image pull fail? Tag "nonexistent" does not exist.
3. Why was a nonexistent tag used? No image validation in deployment pipeline.
4. Why is there no validation? CI/CD pipeline does not verify image existence.
5. Why not? This check was never implemented.

### Root Cause
The CI/CD pipeline does not validate that container images exist before deploying.

## Action Items
| # | Action | Owner | Due |
|---|--------|-------|-----|
| 1 | Add image validation step to CI/CD pipeline | DevOps | 2026-06-18 |
| 2 | Add canary deployment for all changes | Platform | 2026-06-25 |
```

### Lab Validation Checklist

- [ ] Incident simulated and resolved
- [ ] Timeline documented with timestamps
- [ ] 5 Whys analysis completed
- [ ] Root cause identified (systemic, not individual)
- [ ] Action items defined with owners and due dates
- [ ] Post-mortem written in blameless language

## Limitation -> Next Topic

Post-mortems analyze what went wrong after the fact. But many incidents can be prevented by controlling how changes are made in the first place. Change management ensures that every modification to production is planned, reviewed, and executed safely.

**Next: [Module 79 — Change Management](../79-change-management/README.md)**
