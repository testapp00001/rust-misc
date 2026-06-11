# Solution 03: Escalation Policies and On-Call Rotations

## Part A: Escalation Policy Design

### Task 1: Escalation Policies

**1. Critical Application Alert (e.g., HighErrorRate on payment service)**

| Level | Who | Method | Timeout | Escalates To |
|-------|-----|--------|---------|-------------|
| 1 | Backend on-call (Dave/Eve/Frank) | Push notification (PagerDuty) | 10 min | Level 2 |
| 2 | Backend secondary on-call | Push notification + SMS | 10 min | Level 3 |
| 3 | Dave (Backend lead) + Grace (SRE lead) | Phone call | 10 min | Level 4 |
| 4 | VP of Engineering | Phone call + SMS + Email | N/A | Final |

Reasoning: The payment service is backend code, so the backend team should be first responders. The SRE lead is included at level 3 because this is a critical revenue-affecting issue that may require infrastructure-level intervention.

**2. Critical Infrastructure Alert (e.g., NodeDown in production)**

| Level | Who | Method | Timeout | Escalates To |
|-------|-----|--------|---------|-------------|
| 1 | Platform on-call (Alice/Bob/Carol) | Push notification (PagerDuty) | 10 min | Level 2 |
| 2 | Platform secondary on-call | Push notification + SMS | 10 min | Level 3 |
| 3 | Alice (Platform lead) + Grace (SRE lead) | Phone call | 10 min | Level 4 |
| 4 | VP of Engineering | Phone call + SMS + Email | N/A | Final |

Reasoning: Node issues are infrastructure problems. The platform team manages Kubernetes and nodes. Grace (SRE) at level 3 because node failures can cascade and affect all services.

**3. Database Alert (e.g., ReplicationLag)**

| Level | Who | Method | Timeout | Escalates To |
|-------|-----|--------|---------|-------------|
| 1 | Backend on-call (closest to database code) | Push notification | 10 min | Level 2 |
| 2 | Alice (Platform lead, infrastructure expertise) | Push notification + SMS | 10 min | Level 3 |
| 3 | Grace (SRE lead) + Dave (Backend lead) | Phone call | 10 min | Level 4 |
| 4 | VP of Engineering | Phone call + SMS + Email | N/A | Final |

Reasoning: Database issues can be code-related (query problems) or infrastructure-related (disk, network). The backend team is first because they know the queries. Alice at level 2 because it might be a storage or networking issue.

**4. Security Alert (e.g., UnauthorizedAPICalls)**

| Level | Who | Method | Timeout | Escalates To |
|-------|-----|--------|---------|-------------|
| 1 | Grace (SRE lead, security expertise) | Push notification + SMS | 5 min | Level 2 |
| 2 | Alice (Platform lead) + Dave (Backend lead) | Phone call | 5 min | Level 3 |
| 3 | VP of Engineering | Phone call + SMS | 5 min | Level 4 |
| 4 | CISO / CTO | Phone call + SMS + Email | N/A | Final |

Reasoning: Security alerts have shorter timeouts (5 min vs 10 min) because the blast radius can grow rapidly. Grace is first because SRE typically handles security monitoring. Escalation is faster because security incidents may require immediate action (revoke credentials, block IPs).

### Task 2: On-Call Rotations

**4-Week Rotation Schedule (Alice, Bob, Carol):**

| Week | Primary | Secondary | Notes |
|------|---------|-----------|-------|
| Week 1 | Alice | Bob | Carol is backup during US hours |
| Week 2 | Carol | Alice | Bob is backup during US hours |
| Week 3 | Bob | Carol | Alice is backup during US hours |
| Week 4 | Alice | Bob | Carol is backup during US hours (repeat) |

Rules:
- Primary and secondary rotate weekly on Monday 10:00 AM UTC.
- No person is primary two weeks in a row (Alice W1, Carol W2, Bob W3, Alice W4 -- Alice has a 2-week gap).
- If someone is on vacation, the next person in the rotation takes their slot and a temporary backup is assigned.

**Follow-the-Sun Coverage:**

```
US hours (9am-5pm PT = 5pm-1am UTC):
  Primary: US-based engineer (Alice)
  Secondary: EU-based engineer (backup from another team)

EU hours (9am-5pm CET = 8am-4pm UTC):
  Primary: EU-based engineer
  Secondary: APAC-based engineer

APAC hours (9am-5pm JST = midnight-8am UTC):
  Primary: APAC-based engineer
  Secondary: US-based engineer (Alice, end of her day)
```

**Holiday Coverage:**
- At least 2 weeks notice for planned time off.
- The engineer going on vacation finds their own replacement and gets approval from the team lead.
- Unplanned absences (sick days): The secondary becomes primary, and the team lead assigns a new secondary.
- Holiday periods (Christmas, New Year): Extended shifts (2 weeks instead of 1) with extra compensation.

### Task 3: Escalation Simulation

```python
# webhook-receiver/app.py
from flask import Flask, request, jsonify
from datetime import datetime
import threading
import hashlib
import json

app = Flask(__name__)

# Configuration
ESCALATION_TIMEOUT = 10  # seconds for testing (600 for production: 10 minutes)
ESCALATION_LEVELS = [
    {"level": 1, "notify": "Primary on-call", "method": "push notification"},
    {"level": 2, "notify": "Secondary on-call", "method": "push + SMS"},
    {"level": 3, "notify": "Team lead", "method": "phone call"},
    {"level": 4, "notify": "Engineering manager", "method": "phone + SMS + email"},
]

# State
active_alerts = {}  # fingerprint -> alert data
escalation_log = []


def get_fingerprint(labels):
    """Generate a fingerprint from alert labels."""
    label_str = json.dumps(labels, sort_keys=True)
    return hashlib.md5(label_str.encode()).hexdigest()[:12]


def escalate(fingerprint):
    """Called when escalation timeout expires without acknowledgment."""
    if fingerprint not in active_alerts:
        return

    alert_data = active_alerts[fingerprint]
    old_level = alert_data['level']
    new_level = old_level + 1

    if new_level > len(ESCALATION_LEVELS):
        # Already at max level, no more escalation
        return

    alert_data['level'] = new_level
    new_config = ESCALATION_LEVELS[new_level - 1]

    escalation_entry = {
        'timestamp': datetime.now().isoformat(),
        'fingerprint': fingerprint,
        'alertname': alert_data['alertname'],
        'old_level': old_level,
        'new_level': new_level,
        'notify': new_config['notify'],
        'method': new_config['method'],
    }
    escalation_log.append(escalation_entry)

    print(f"\n{'='*60}")
    print(f"ESCALATION: {alert_data['alertname']}")
    print(f"  Level {old_level} -> Level {new_level}")
    print(f"  Now notifying: {new_config['notify']}")
    print(f"  Method: {new_config['method']}")
    print(f"{'='*60}")

    # Schedule next escalation if not at max level
    if new_level < len(ESCALATION_LEVELS):
        timer = threading.Timer(ESCALATION_TIMEOUT, escalate, [fingerprint])
        alert_data['timer'] = timer
        timer.start()


@app.route('/alerts', methods=['POST'])
def handle_alert():
    data = request.json

    for alert in data.get('alerts', []):
        labels = alert.get('labels', {})
        fingerprint = get_fingerprint(labels)
        alertname = labels.get('alertname', 'unknown')
        status = alert.get('status', 'firing')

        if status == 'firing' and fingerprint not in active_alerts:
            # New alert -- start escalation at level 1
            level_config = ESCALATION_LEVELS[0]
            active_alerts[fingerprint] = {
                'alertname': alertname,
                'labels': labels,
                'annotations': alert.get('annotations', {}),
                'level': 1,
                'started_at': datetime.now().isoformat(),
                'timer': None,
            }

            escalation_entry = {
                'timestamp': datetime.now().isoformat(),
                'fingerprint': fingerprint,
                'alertname': alertname,
                'old_level': 0,
                'new_level': 1,
                'notify': level_config['notify'],
                'method': level_config['method'],
            }
            escalation_log.append(escalation_entry)

            print(f"\n{'='*60}")
            print(f"NEW ALERT: {alertname}")
            print(f"  Level 1: Notifying {level_config['notify']}")
            print(f"  Method: {level_config['method']}")
            print(f"  Escalation in {ESCALATION_TIMEOUT}s if not acknowledged")
            print(f"{'='*60}")

            # Schedule escalation
            timer = threading.Timer(ESCALATION_TIMEOUT, escalate, [fingerprint])
            active_alerts[fingerprint]['timer'] = timer
            timer.start()

        elif status == 'resolved' and fingerprint in active_alerts:
            # Alert resolved -- cancel escalation
            active_alerts[fingerprint]['timer'].cancel()
            del active_alerts[fingerprint]

            print(f"\n{'='*60}")
            print(f"RESOLVED: {alertname}")
            print(f"  Escalation cancelled")
            print(f"{'='*60}")

    return jsonify({"status": "received"})


@app.route('/acknowledge', methods=['POST'])
def acknowledge():
    data = request.json
    fingerprint = data.get('fingerprint')

    if not fingerprint:
        return jsonify({"error": "fingerprint required"}), 400

    if fingerprint not in active_alerts:
        return jsonify({"error": "alert not found"}), 404

    alert_data = active_alerts[fingerprint]
    alert_data['timer'].cancel()
    alertname = alert_data['alertname']
    level = alert_data['level']

    escalation_entry = {
        'timestamp': datetime.now().isoformat(),
        'fingerprint': fingerprint,
        'alertname': alertname,
        'action': 'acknowledged',
        'level_at_ack': level,
    }
    escalation_log.append(escalation_entry)

    del active_alerts[fingerprint]

    print(f"\n{'='*60}")
    print(f"ACKNOWLEDGED: {alertname}")
    print(f"  Acknowledged at level {level}")
    print(f"  Escalation stopped")
    print(f"{'='*60}")

    return jsonify({"status": "acknowledged", "alertname": alertname, "level": level})


@app.route('/active')
def get_active():
    result = []
    for fp, data in active_alerts.items():
        result.append({
            'fingerprint': fp,
            'alertname': data['alertname'],
            'level': data['level'],
            'started_at': data['started_at'],
            'notify': ESCALATION_LEVELS[data['level'] - 1]['notify'],
        })
    return jsonify(result)


@app.route('/escalation-log')
def get_escalation_log():
    return jsonify(escalation_log)


@app.route('/clear', methods=['POST'])
def clear():
    for fp in list(active_alerts.keys()):
        active_alerts[fp]['timer'].cancel()
    active_alerts.clear()
    escalation_log.clear()
    return jsonify({"status": "cleared"})


if __name__ == '__main__':
    app.run(host='0.0.0.0', port=9998)
```

### Task 4: AlertManager Integration

Add the escalation receiver to your AlertManager configuration:

```yaml
receivers:
  - name: 'escalation-webhook'
    webhook_configs:
      - url: 'http://escalation-receiver:9998/alerts'
        send_resolved: true

route:
  routes:
    - match:
        severity: critical
      receiver: 'escalation-webhook'
```

### Task 5: Testing the Escalation Flow

```bash
# Start the stack
docker-compose up -d --build

# Send a critical alert
curl -X POST http://localhost:9093/api/v2/alerts \
  -H "Content-Type: application/json" \
  -d '[{
    "labels": {
      "alertname": "HighErrorRate",
      "severity": "critical",
      "service": "payment"
    },
    "annotations": { "summary": "Payment service error rate critical" }
  }]'

# Check active alerts (should show level 1)
sleep 2
curl http://localhost:9998/active | python -m json.tool

# Wait for escalation (10 seconds for testing)
sleep 12

# Should now be at level 2
curl http://localhost:9998/active | python -m json.tool

# Acknowledge
curl -X POST http://localhost:9998/acknowledge \
  -H "Content-Type: application/json" \
  -d '{"fingerprint": "<fingerprint-from-active>"}'

# Verify escalation stopped
curl http://localhost:9998/active | python -m json.tool
# Should be empty

# Check escalation log
curl http://localhost:9998/escalation-log | python -m json.tool
```

## Part B: Analysis

### Task 6: On-Call Best Practices

**1. Why should rotations be at least 1 week?**

Daily rotations mean the on-call person never has time to learn the patterns of the current week's incidents. They spend their first day learning what is happening and their last day about to hand off. One week is long enough to:
- Understand the current incident context
- Follow up on issues from previous days
- Learn the common failure patterns
- Feel ownership of the on-call shift

Daily rotations also create excessive handoff overhead. Every day requires a formal handoff meeting, which wastes time.

**2. What is a shadow shift?**

A shadow shift is when a new team member observes an experienced on-call engineer during their shift without being the primary responder. The shadow:
- Listens in on pages and incident responses
- Asks questions about the diagnosis process
- Practices using runbooks and dashboards
- Handles a few low-severity alerts under supervision

Use shadow shifts before putting a new team member on-call for the first time. This reduces anxiety and ensures they know the tools and processes.

**3. Why primary AND secondary on-call?**

The secondary protects against:
- **Unavailability**: Primary is in a tunnel, on a plane, or in a meeting with no signal.
- **Overload**: Two simultaneous incidents that one person cannot handle.
- **Escalation**: If the primary cannot resolve the issue, the secondary can assist.
- **Fatigue**: During a long incident (3+ hours), the secondary can take over so the primary can rest.

**4. What if all 4 levels are exhausted with no response?**

This is a critical failure in the escalation chain. Actions:
1. The system should automatically page the entire engineering team (all-hands page).
2. The engineering manager should be paged via multiple channels simultaneously (phone, SMS, email, Slack).
3. If still no response, the system should escalate to the CTO/VP of Engineering.
4. Post-incident: Review why the escalation chain failed. Was it a technical failure (pages not delivering)? A process failure (wrong phone numbers)? A people failure (everyone was unreachable)?

**5. On-call health metrics:**

1. **Pages per on-call shift**: Target < 5 per week. More indicates alert noise.
2. **Mean Time to Acknowledge (MTTA)**: Target < 5 min for critical. Longer means the on-call is not hearing pages.
3. **False positive rate**: Target < 10%. High rate causes alert fatigue.
4. **Escalation rate**: Target < 5%. High rate means primary is overwhelmed or unreachable.
5. **On-call satisfaction score**: Survey after each shift. Target > 7/10. Low score indicates burnout.
6. **Incidents per week**: Target < 3. More indicates systemic reliability problems.

### Task 7: Escalation Policy Critique

**Problem 1: All notifications are email**
- Email is not urgent. Nobody checks email at 3 AM. The on-call will not see the alert until morning.
- **Fix**: Use push notifications (PagerDuty, Opsgenie) for level 1. SMS for level 2. Phone call for levels 3-4.

**Problem 2: 30-minute timeout is too long**
- A critical alert sitting unacknowledged for 30 minutes means 30 minutes of user impact with no response.
- **Fix**: 10-15 minutes maximum for critical alerts. 5 minutes for security incidents.

**Problem 3: Level 1 and Level 2 go to the same person with the same method**
- Sending the same person two emails 30 minutes apart adds no value. If they did not respond to the first email, they will not respond to the second.
- **Fix**: Level 2 should be a different person (secondary on-call) with a more urgent method (SMS).

**Problem 4: No acknowledgment mechanism**
- There is no way for the on-call to say "I am working on this." The escalation continues even if someone is actively investigating.
- **Fix**: Add an acknowledgment mechanism. Once acknowledged, stop escalating.

**Problem 5: Escalating to VP who cannot help**
- The VP of Engineering cannot debug a Kubernetes issue at 3 AM. Escalating to them adds no value for incident resolution.
- **Fix**: The final escalation level should be someone who can mobilize resources (team lead, engineering manager), not a VP. The VP should be informed during business hours via a post-incident report.

**Bonus Problem 6: No severity differentiation**
- A 2% error rate warning and a complete service outage both follow the same escalation path. The warning should not wake anyone up.
- **Fix**: Different escalation policies for different severities. Critical pages immediately; warning goes to Slack only.

## Common Mistakes

1. **Making timeouts too long**: 30 minutes is too long for a critical alert. Users are impacted for 30 minutes before anyone even looks at the problem.

2. **Same method at every level**: If push notification did not work at level 1, sending another push notification at level 2 will not work either. Escalate the urgency of the method.

3. **No acknowledgment**: Without acknowledgment, the on-call gets pages even while actively working on the issue. This is distracting and counterproductive.

4. **Escalating to people who cannot help**: Escalating to a VP who cannot debug the system wastes everyone's time. Escalate to domain experts, not executives.

5. **Forgetting the secondary on-call**: If the primary is unreachable, there is no backup. Always have a secondary.

6. **Not compensating on-call**: On-call is stressful and disruptive. If the company does not compensate (pay or time off), engineers will burn out and leave.
