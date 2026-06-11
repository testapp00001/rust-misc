# Exercise 03: Escalation Policies and On-Call Rotations

## Type: Independent

## Objective

Design escalation policies that ensure no alert goes unhandled, create on-call rotation schedules that prevent burnout, and implement a PagerDuty-style escalation system using a webhook receiver that simulates escalation levels.

## Prerequisites

- Completion of Exercises 01 and 02
- Docker and Docker Compose
- Understanding of AlertManager routing

## Part A: Escalation Policy Design

### Task 1: Design an Escalation Policy

You are the platform team lead for a company with the following team structure:

| Team | Members | Expertise |
|------|---------|-----------|
| Platform | Alice (lead), Bob, Carol | Kubernetes, networking, infrastructure |
| Backend | Dave (lead), Eve, Frank | Application code, APIs, databases |
| SRE | Grace (lead), Heidi | Monitoring, incident response, capacity |

Design a 4-level escalation policy for each of the following alert categories. For each level, specify who gets notified, how (push notification, phone call, SMS), and the timeout before escalating.

1. **Critical Application Alert** (e.g., HighErrorRate on the payment service)
2. **Critical Infrastructure Alert** (e.g., NodeDown in production)
3. **Database Alert** (e.g., ReplicationLag exceeding threshold)
4. **Security Alert** (e.g., UnauthorizedAPICalls spike)

Present your policies in a table format:

```
Level | Who | Method | Timeout | Escalates To
1     | ??? | ???    | ???m    | Level 2
2     | ??? | ???    | ???m    | Level 3
3     | ??? | ???    | ???m    | Level 4
4     | ??? | ???    | N/A     | (final)
```

### Task 2: Design On-Call Rotations

Design on-call rotation schedules for the Platform team (Alice, Bob, Carol) with these constraints:

1. Primary and secondary on-call at all times (24/7)
2. Weekly rotation (switch every Monday at 10:00 AM UTC)
3. No person should be primary two weeks in a row
4. Follow-the-sun coverage: US (9am-5pm PT), EU (9am-5pm CET), APAC (9am-5pm JST)
5. Holiday coverage plan (what happens when someone is on vacation?)

Present your rotation as a 4-week schedule showing primary, secondary, and business-hours backup for each week.

### Task 3: Escalation Simulation

Build a webhook receiver that simulates a PagerDuty-style escalation system. The receiver should:

1. Accept alerts from AlertManager via webhooks
2. Track the escalation level for each alert (starting at level 1)
3. If no acknowledgment is received within a configurable timeout (default: 10 minutes), escalate to the next level
4. Log each escalation event with timestamp, alert name, previous level, new level, and who is being notified
5. Provide an endpoint to acknowledge an alert: `POST /acknowledge` with `{"fingerprint": "abc123"}`
6. Provide an endpoint to list active alerts and their current escalation level: `GET /active`
7. Provide an endpoint to list the escalation log: `GET /escalation-log`

Use the following escalation levels:

| Level | Notify | Method | Timeout |
|-------|--------|--------|---------|
| 1 | Primary on-call | Push notification | 10 min |
| 2 | Secondary on-call | Push + SMS | 10 min |
| 3 | Team lead | Phone call | 10 min |
| 4 | Engineering manager | Phone + SMS + Email | N/A |

### Task 4: Integrate with AlertManager

Update your AlertManager configuration from Exercise 02 to send critical alerts to the escalation webhook receiver. The webhook receiver URL should be `http://escalation-receiver:9998/alerts`.

Create a `docker-compose.yml` that runs the escalation receiver alongside Prometheus and AlertManager.

### Task 5: Test the Escalation Flow

1. Start the stack.
2. Send a critical alert to AlertManager.
3. Verify the alert appears at escalation level 1.
4. Wait for the escalation timeout (or reduce it to 30 seconds for testing).
5. Verify the alert escalates to level 2.
6. Send an acknowledgment.
7. Verify the alert stops escalating.
8. Send another alert, let it escalate through all 4 levels, and verify the final state.

## Part B: Analysis

### Task 6: On-Call Best Practices

Answer the following questions:

1. Why should on-call rotations be at least 1 week long? What is wrong with daily rotations?
2. What is a "shadow shift" and when should you use one?
3. Why is it important to have both a primary and secondary on-call? What scenarios does this protect against?
4. How should you handle a situation where the on-call person does not respond at all (all 4 escalation levels are exhausted)?
5. What metrics should you track to measure on-call health? List at least 4.

### Task 7: Escalation Policy Critique

A team has the following escalation policy:

```
Level 1: On-call engineer (email)
  timeout: 30 minutes
Level 2: On-call engineer (email again)
  timeout: 30 minutes
Level 3: Team lead (email)
  timeout: 1 hour
Level 4: VP of Engineering (email)
```

Identify at least 5 problems with this policy and explain how to fix each one.

## Success Criteria

- [ ] You can design a 4-level escalation policy with appropriate contacts, methods, and timeouts
- [ ] You can create a rotation schedule that provides 24/7 coverage with primary and secondary
- [ ] The escalation webhook receiver correctly tracks alert levels and escalates on timeout
- [ ] Acknowledgment stops the escalation chain
- [ ] You can identify weaknesses in poorly designed escalation policies
- [ ] You understand on-call best practices including compensation, shadow shifts, and rotation length

## Hints

<details>
<summary>Hint 1: Escalation policy design principles</summary>

Good escalation policies follow these rules:
- **Level 1** should reach someone who can likely fix the issue (the domain expert).
- **Escalation timeouts** should be short enough that the alert does not sit unhandled, but long enough to account for the person being in the bathroom or on another call (10-15 minutes is typical).
- **Each level should add reach**: push notification -> SMS -> phone call. Phone calls are the most intrusive and should be reserved for higher levels.
- **The final level** should be someone with authority to mobilize additional resources (team lead, manager).
- **Never escalate to someone who cannot help**. Escalating to a VP who cannot debug Kubernetes is useless.

</details>

<details>
<summary>Hint 2: Follow-the-sun rotation structure</summary>

For a global team, create separate rotations for each timezone:

```
US hours (9am-5pm PT = 5pm-1am UTC):
  Primary: US engineer
  Secondary: EU engineer (end of their day)

EU hours (9am-5pm CET = 8am-4pm UTC):
  Primary: EU engineer
  Secondary: APAC engineer (end of their day)

APAC hours (9am-5pm JST = midnight-8am UTC):
  Primary: APAC engineer
  Secondary: US engineer (end of their day)
```

The secondary during one timezone's business hours is the primary from another timezone who is available for overlap.

</details>

<details>
<summary>Hint 3: Simulating escalation with Python threading</summary>

Use Python's `threading.Timer` to schedule escalation checks. When an alert arrives, start a timer. If the alert is not acknowledged before the timer fires, escalate and start a new timer for the next level.

```python
import threading

class EscalationTracker:
    def __init__(self, timeout=600):
        self.alerts = {}  # fingerprint -> alert_data
        self.timeout = timeout

    def start_escalation(self, fingerprint, alert):
        self.alerts[fingerprint] = {
            'alert': alert,
            'level': 1,
            'timer': threading.Timer(self.timeout, self.escalate, [fingerprint])
        }
        self.alerts[fingerprint]['timer'].start()

    def acknowledge(self, fingerprint):
        if fingerprint in self.alerts:
            self.alerts[fingerprint]['timer'].cancel()
            del self.alerts[fingerprint]

    def escalate(self, fingerprint):
        if fingerprint in self.alerts:
            self.alerts[fingerprint]['level'] += 1
            level = self.alerts[fingerprint]['level']
            if level < 4:
                self.alerts[fingerprint]['timer'] = threading.Timer(
                    self.timeout, self.escalate, [fingerprint]
                )
                self.alerts[fingerprint]['timer'].start()
```

</details>

<details>
<summary>Hint 4: Alert fingerprinting</summary>

AlertManager assigns a unique fingerprint to each alert based on its label set. The fingerprint is included in the webhook payload. Use this to track which alert is being acknowledged or escalated.

In the webhook payload, look for:
```json
{
  "alerts": [
    {
      "fingerprint": "abc123def456",
      "labels": { ... },
      "status": "firing"
    }
  ]
}
```

</details>

<details>
<summary>Hint 5: On-call health metrics</summary>

Track these metrics to measure on-call health:
1. **Pages per on-call shift**: Should be low (ideally < 5 per week).
2. **Time to acknowledge (MTTA)**: How long before someone responds. Should be < 10 minutes for critical.
3. **False positive rate**: Percentage of alerts that did not require action. Should be < 10%.
4. **Escalation rate**: How often alerts escalate past level 1. High rate means the primary is overwhelmed or unresponsive.
5. **After-hours pages**: Pages outside business hours. Track to ensure fair distribution.

</details>
