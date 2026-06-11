# Solution 01: Runbook vs Playbook -- Classification Exercise

## Problem Statement

Classify each of the following 10 scenarios as requiring either a **Runbook**
(step-by-step operational procedure) or a **Playbook** (strategic response
plan with coordination across teams). Provide justification for each choice.

## Complete Classification Table

| # | Scenario | Classification | Justification |
|---|----------|---------------|---------------|
| 1 | A single web server pod is CrashLoopBackOff in Kubernetes | **Runbook** | Well-defined, repeatable diagnostic steps (kubectl describe, logs, events). A single responder can follow the procedure independently. No cross-team coordination needed. |
| 2 | Production database is completely unreachable, affecting all customers | **Playbook** | Multi-team incident: DBAs, application engineers, network team, and customer support must coordinate. Requires communication plan, escalation paths, and parallel investigation tracks. |
| 3 | Disk usage on a monitoring server has hit 95% | **Runbook** | Deterministic procedure: identify large files, rotate logs, clean temp data, expand volume. Steps are sequential and can be executed by a single SRE. |
| 4 | A ransomware attack has been detected on internal infrastructure | **Playbook** | Security incident requiring coordination across security, infrastructure, legal, communications, and executive teams. Strategic decisions (isolate vs. contain, notify authorities) cannot be pre-scripted as rigid steps. |
| 5 | SSL certificate for the main API domain expires in 3 days | **Runbook** | Well-known renewal procedure: generate CSR, submit to CA, validate, deploy certificate, verify. Steps are repeatable and tool-specific (certbot, AWS ACM, etc.). |
| 6 | CI/CD pipeline is deploying to production and half the pods are failing health checks | **Playbook** | Requires real-time decision-making: rollback or proceed? Which services are affected? Who approves the rollback? Involves release engineering, QA, and product teams. |
| 7 | A single Redis cache node has high memory usage (85%) | **Runbook** | Standard procedure: check key patterns, identify TTLs, flush specific key groups, scale memory. Single-operator task with predictable steps. |
| 8 | An entire AWS availability zone has gone down, affecting multiple services | **Playbook** | Major incident requiring coordination across all engineering teams, infrastructure, management, and customer communication. Multi-track response: failover DNS, scale remaining AZs, communicate status. |
| 9 | Nginx configuration needs to be reloaded after a routing rule change | **Runbook** | Atomic, well-understood operation: validate config (`nginx -t`), reload (`nginx -s reload`), verify. Can be documented as a simple checklist. |
| 10 | A third-party payment provider is experiencing an outage | **Playbook** | Strategic response: assess impact on orders, decide whether to queue or reject, communicate to customers, coordinate with business teams on SLA implications, and prepare for provider recovery. |

## Visual Decision Framework

```
                        Is the incident well-understood
                        and repeatable?
                               |
                    +----------+----------+
                    |                     |
                   YES                    NO
                    |                     |
            Can a single responder       Does it require cross-team
            follow the procedure?        coordination or strategic
                    |                    decision-making?
             +------+------+                   |
             |             |                  YES
            YES            NO                  |
             |             |            PLAYBOOK
          RUNBOOK      PLAYBOOK
```

## Detailed Justifications

### Scenarios 1, 3, 5, 7, 9 -- Why These Are Runbooks

These five scenarios share critical characteristics:

- **Deterministic cause-and-effect**: The problem maps to a known solution path.
- **Single-operator executable**: One person with the right access can resolve the issue.
- **Repeatable**: The same steps work every time the scenario occurs.
- **Low ambiguity**: There is no "it depends" in the resolution path.
- **Tool-specific commands**: The procedure can be expressed as exact commands.

```
Runbook characteristics:
┌─────────────────────────────────────────────┐
│  Input: Symptom or alert                    │
│    ↓                                        │
│  Diagnosis: Follow known decision tree      │
│    ↓                                        │
│  Resolution: Execute specific commands      │
│    ↓                                        │
│  Verification: Check known success criteria │
│    ↓                                        │
│  Output: Service restored                   │
└─────────────────────────────────────────────┘
```

**Example -- Scenario 3 (Disk at 95%):**

```bash
# Step 1: Identify large files
du -sh /var/log/* | sort -rh | head -20

# Step 2: Check for old logs
find /var/log -name "*.gz" -mtime +30 -delete

# Step 3: Rotate current logs
logrotate -f /etc/logrotate.conf

# Step 4: Check for temp files
find /tmp -type f -mtime +7 -delete

# Step 5: Verify
df -h /var/log
```

### Scenarios 2, 4, 6, 8, 10 -- Why These Are Playbooks

These five scenarios share fundamentally different characteristics:

- **Non-deterministic**: The root cause is unknown; investigation branches in multiple directions.
- **Multi-team coordination**: No single team can resolve the issue alone.
- **Strategic decisions required**: Choices must be made based on business impact, risk tolerance, and real-time information.
- **Communication-heavy**: Stakeholders, customers, and management need updates.
- **Escalation paths**: The response changes based on severity and duration.

```
Playbook characteristics:
┌──────────────────────────────────────────────────┐
│  Trigger: Incident declared                       │
│    ↓                                             │
│  Triage: Assess severity, assign roles            │
│    ↓                                             │
│  Parallel investigation tracks:                   │
│    ├── Track A: Infrastructure team               │
│    ├── Track B: Application team                  │
│    ├── Track C: Communications / Customer support │
│    └── Track D: Security / Compliance (if needed) │
│    ↓                                             │
│  Decision points:                                 │
│    ├── Escalate?                                  │
│    ├── Rollback?                                  │
│    ├── Failover?                                  │
│    └── External communication?                    │
│    ↓                                             │
│  Resolution + Post-incident review                │
└──────────────────────────────────────────────────┘
```

**Example -- Scenario 8 (AZ Down):**

```
PLAYBOOK: Availability Zone Failure
====================================

ROLES:
  Incident Commander (IC) .............. [Name]
  Infrastructure Lead .................. [Name]
  Application Lead ..................... [Name]
  Communications Lead .................. [Name]

TIMELINE:
  T+0m   IC declares major incident, pages relevant teams
  T+5m   Infrastructure confirms AZ failure (not network partition)
  T+10m  Begin DNS failover to healthy AZs
  T+10m  Scale up services in remaining AZs
  T+15m  Communications posts status page update
  T+30m  Verify all critical services serving from healthy AZs
  T+60m  IC provides executive briefing
  T+??   Monitor for AZ recovery, plan failback

DECISION POINTS:
  - Is this a full AZ outage or partial? → Determines response scope
  - Are data stores replicated? → Determines data loss risk
  - Can remaining AZs handle full load? → Determines degradation strategy
```

## Key Takeaway

The distinction is not about complexity -- a playbook can involve simple
technical steps, and a runbook can be technically challenging. The distinction
is about **coordination and decision-making**:

- **Runbook**: "Follow these steps." The path is known.
- **Playbook**: "Make these decisions with these people." The path depends on context.

A mature organization often has playbooks that *reference* runbooks. For
example, an AZ-failure playbook might say "Execute the DNS failover runbook"
as one of its coordinated actions.

## Common Mistakes

1. **Treating major incidents as runbooks**: A database-down scenario looks
   technical, but if it affects all customers, you need cross-team coordination
   -- that is a playbook, not a runbook.

2. **Over-complicating runbooks**: Adding "notify the VP" and "update the
   status page" to a disk-cleanup runbook turns it into a poorly structured
   playbook. Keep runbooks focused on the technical procedure.

3. **Playbooks without runbooks**: A playbook that says "fix the database"
   without linking to a detailed failover runbook is useless. Playbooks should
   reference specific runbooks for technical execution.

4. **Ignoring the gray area**: Some scenarios sit on the boundary. A pod
   CrashLoop in a critical payment service might need a playbook (coordinate
   with product, consider customer impact) rather than just a runbook (restart
   the pod). Context matters.

5. **Static classification**: A scenario's classification can change based on
   scope. "One disk is full" is a runbook. "All disks across all servers are
   full simultaneously" is a playbook (something systemic is wrong).
