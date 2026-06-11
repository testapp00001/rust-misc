# Solution 01: Change Management Principles for Infrastructure

## Part A -- Change Classification

### 1. Routine OS security patch on 50 web servers

**Classification: Standard**

This is a pre-approved, repeatable change with low risk. Security patches follow
a documented procedure that has been executed many times. The maintenance window
is already scheduled, and the change type is well-understood. Standard changes
do not require individual CAB approval because the risk has been assessed and
accepted in advance.

### 2. Replacing a failed load balancer at 03:00 AM

**Classification: Emergency**

The load balancer has already failed and traffic is actively dropping. This is
a restore-of-service situation that cannot wait for the next CAB meeting. The
change follows the emergency path: implement immediately to restore service,
then document the change and conduct a PIR within 48 hours. Emergency changes
still require authorization, but from an on-call manager rather than the CAB.

### 3. Database migration from MySQL 5.7 to 8.0

**Classification: Normal**

This is a significant change with medium-to-high risk: schema modifications are
required and 15 minutes of downtime is expected. It does not fit the Standard
category because it is not routine or pre-approved. It requires a full RFC with
impact analysis, rollback plan, test results, and CAB approval before
implementation.

### 4. Adding a new DNS record for an internal service

**Classification: Standard**

Adding a DNS record is a low-risk, routine operational task. It follows a
well-documented procedure, has minimal blast radius (internal only), and is
easily reversible. This is pre-approved as a standard change.

### 5. Scaling Kubernetes cluster from 10 to 25 nodes

**Classification: Emergency (likely) or Normal (context-dependent)**

If triggered by an unexpected traffic surge causing active degradation, this is
an Emergency change -- the capacity increase is needed immediately to restore
service levels. If the scaling is planned ahead of an anticipated event (e.g.,
Black Friday), it would be a Normal change with a full RFC. The classification
depends on whether the system is currently under distress.

---

## Part B -- RFC Lifecycle

### 1. Initiation (Submission)

**Who can submit:** Any engineer, project manager, or team lead who has
identified a need for change. The submitter is typically the change owner or
someone directly affected by the proposed change.

**Required RFC content:**
- **Change description:** What exactly is being changed (systems, components,
  configurations)
- **Business justification:** Why the change is needed (security, performance,
  compliance, feature)
- **Impact analysis:** Which systems and users are affected, including
  dependencies
- **Risk assessment:** Likelihood and impact of failure, with risk level
  classification
- **Implementation plan:** Step-by-step procedure for applying the change
- **Rollback plan:** How to revert to the previous state if the change fails
- **Test evidence:** Results from staging or pre-production testing
- **Schedule:** Proposed implementation date, time, and duration
- **Communication plan:** Who needs to be notified and when

### 2. Review and Assessment

The change is reviewed for completeness. A Change Manager verifies that all
required fields are populated and that the risk assessment is reasonable. If
the RFC is incomplete, it is returned to the submitter with specific feedback.

### 3. CAB Evaluation

The Change Advisory Board meets (typically weekly) to evaluate Normal changes.
The CAB:
- Reviews the risk assessment and impact analysis
- Questions the submitter if clarification is needed
- Evaluates the rollback plan's adequacy
- Checks for conflicts with other scheduled changes
- Votes to approve, reject, or request modifications

**Approval path:** If the CAB approves, the RFC moves to "Scheduled" status
with an authorized implementation window.

**Rejection path:** If rejected, the CAB documents the reason. The submitter
may revise and resubmit. If modifications are requested, the RFC returns to
"Draft" status.

### 4. Scheduling

Once approved, the change is placed on the forward change schedule. The
schedule avoids conflicts (e.g., two major changes on the same infrastructure
on the same day) and ensures resources are available.

### 5. Implementation

During the authorized window:
- The implementer follows the documented procedure
- A change coordinator monitors progress and communicates status
- If issues arise, the implementer evaluates whether to continue or invoke
  the rollback plan
- All steps are logged with timestamps

### 6. Verification

After implementation:
- Automated health checks confirm the system is operational
- Smoke tests validate the change achieved its intended effect
- Monitoring dashboards are reviewed for anomalies
- If verification fails, the rollback plan is executed

### 7. Post-Implementation Review (PIR)

Within 48 hours of implementation:
- The team reviews what went well and what didn't
- Any unplanned issues are documented
- The rollback plan effectiveness is evaluated
- Lessons learned are captured and shared
- If the change was unsuccessful, the PIR feeds into the next RFC cycle
- The RFC is closed with a final status (Successful, Unsuccessful, Rolled Back)

---

## Part C -- Risk Assessment Matrix

| | Low Impact | Medium Impact | High Impact |
|---|---|---|---|
| **High Likelihood** | **Medium** -- Team Lead approval; Example: DNS TTL change for internal service that may cause brief resolution delays | **High** -- CAB approval required; Example: OS kernel upgrade on web servers with known compatibility issues | **Critical** -- CAB + Executive approval; Example: Database engine migration with known data corruption risk in test environments |
| **Medium Likelihood** | **Low** -- Team Lead approval; Example: Adding a monitoring check to a healthy service | **Medium** -- Team Lead + Manager approval; Example: Upgrading a reverse proxy to a new minor version | **High** -- CAB approval required; Example: Network firewall rule changes affecting production traffic paths |
| **Low Likelihood** | **Low** -- Pre-approved (Standard change); Example: Adding a DNS record for a new internal microservice | **Low** -- Team Lead approval; Example: Scaling up a non-critical batch processing service | **Medium** -- CAB approval required; Example: Cloud provider region migration for a production workload |

### Rationale

- **Risk Level** = Likelihood x Impact. High likelihood with high impact is
  Critical and requires the highest level of scrutiny.
- **Approval levels** scale with risk: Standard changes are pre-approved, Low
  risk needs a Team Lead, Medium risk needs management, High and Critical need
  the CAB.
- **Examples** reflect real infrastructure scenarios where the combination of
  likelihood and impact produces that risk level.

---

## Part D -- Controls and Safeguards

### 1. Change Approval Process (Pre-Change)

**Risk mitigated:** Unauthorized or poorly planned changes reaching production.

**How it works:** Every change must be documented in an RFC and approved by the
appropriate authority before implementation. The level of approval scales with
risk. Standard changes use pre-approved procedures; Normal changes go through
the CAB; Emergency changes are authorized by an on-call manager with PIR to
follow.

**If bypassed:** Changes may be applied without adequate review, increasing the
risk of outages. Without approval, there is no accountability trail, making
root cause analysis difficult when failures occur. Compliance audits will flag
the gap.

### 2. Mandatory Testing in Non-Production Environments (Pre-Change)

**Risk mitigated:** Deploying untested changes that break production.

**How it works:** All changes must be tested in a staging or pre-production
environment that mirrors production configuration. Test results (pass/fail,
performance benchmarks, integration test output) must be attached to the RFC.
The CAB will not approve an RFC without test evidence.

**If bypassed:** Untested changes may introduce regressions, data corruption,
or performance degradation. The first time the change runs is in production,
with real users and real data at risk.

### 3. Automated Rollback Capability (During Change)

**Risk mitigated:** Extended outages caused by failed changes that are slow to
reverse.

**How it works:** Every change must have a documented and tested rollback plan.
For Kubernetes deployments, this means using rolling update strategies with
`kubectl rollout undo` capability. For database changes, this means tested
backup restoration procedures. Automated monitoring can trigger rollbacks
without human intervention.

**If bypassed:** If a change fails and there is no rollback plan, the team
must reverse-engineer a fix under pressure, significantly extending the outage
duration and increasing the risk of compounding errors.

### 4. Separation of Duties (Pre/During Change)

**Risk mitigated:** A single person making and approving their own changes,
reducing oversight.

**How it works:** The person who implements a change must be different from the
person who approves it. In CI/CD pipelines, the code author cannot be the
sole approver for production deployment. This is enforced through branch
protection rules and environment approval requirements.

**If bypassed:** A single engineer could push unauthorized or malicious changes
to production without oversight. This violates the principle of least privilege
and creates a single point of failure in the approval chain.

### 5. Post-Implementation Review and Monitoring (Post-Change)

**Risk mitigated:** Recurring failures from changes that appeared successful but
had hidden negative effects.

**How it works:** After every change, a mandatory review period (30 minutes to
24 hours depending on risk) monitors key metrics: error rates, latency
percentiles, resource utilization, and user-reported issues. A PIR meeting
captures lessons learned. Findings feed back into the change management process
to improve future risk assessments.

**If bypassed:** Without PIR, the organization cannot learn from mistakes.
Changes that caused subtle degradation (e.g., a 10% latency increase) may go
undetected until they compound into a significant issue. The same mistakes
will be repeated in future changes.

---

## Key Takeaways

1. Change management is not bureaucracy -- it is a systematic approach to
   reducing the risk of infrastructure modifications.
2. The level of process should be proportional to the risk. Low-risk changes
   need less process; high-risk changes need more.
3. Every change needs a rollback plan. If you cannot roll back, you need to
   rethink the change.
4. Approval without testing is meaningless. The CAB should require evidence
   that the change works before approving it.
5. Post-implementation review closes the feedback loop and prevents repeated
   mistakes.
