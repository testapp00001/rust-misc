# Module 76: Incident Response — On-Call, Escalation, Communication

> **Previous Module:** [75 - Multi-Cloud & Hybrid](../75-multi-cloud-and-hybrid/README.md)
> **Next Module:** [77 - Runbooks & Playbooks](../77-runbooks-and-playbooks/README.md)

## The Problem

It is 3:17 AM. PagerDuty fires. The production database is down. The on-call engineer wakes up, opens their laptop, and has no idea where to start. There is no runbook. The escalation path is unclear. Customers are seeing errors, but nobody has communicated with them. By the time the incident is resolved 4 hours later, the company has lost $200K in revenue and the trust of 10,000 users.

The problem is not that incidents happen — they are inevitable. The problem is that **your organization is not prepared to handle them**.

## The Naive Way

```bash
# "We'll just Slack the engineering team and hope someone sees it"
# #general channel: "Hey, is the site down for anyone else?"
# 45 minutes later: "Yeah, looks like the database is having issues"
# 2 hours later: "I think I found the problem..."
```

**Why this fails:**
- No clear ownership (who is responsible?)
- No escalation path (what if the on-call is unavailable?)
- No communication (customers are in the dark)
- No timeline (nobody knows what happened when)
- No post-incident learning (same incidents repeat)

## The Right Way

### Incident Severity Levels

```
SEVERITY DEFINITIONS:

P1 - Critical (SEV1)
  Impact: Complete service outage, data loss, security breach
  Response: Immediate (5 minutes)
  Escalation: VP Engineering, CTO
  Communication: All-hands, status page, customer email
  Examples: Production database down, data breach, payment processing failure

P2 - Major (SEV2)
  Impact: Significant degradation, many users affected
  Response: 15 minutes
  Escalation: Engineering Manager
  Communication: Status page, internal Slack
  Examples: API latency > 5s, 50% error rate, single-region outage

P3 - Minor (SEV3)
  Impact: Limited degradation, few users affected
  Response: 1 hour
  Escalation: Team Lead
  Communication: Internal Slack
  Examples: Non-critical feature broken, intermittent errors

P4 - Low (SEV4)
  Impact: Cosmetic issues, no user impact
  Response: Next business day
  Escalation: None
  Communication: Jira ticket
  Examples: UI typo, logging issue, minor performance regression
```

### On-Call Rotation

```yaml
# pagerduty-schedule.yaml — On-call rotation configuration
# PagerDuty API call to set up rotation

schedules:
  primary:
    name: "Primary On-Call"
    timezone: "America/New_York"
    rotation:
      type: "weekly"
      start: "2026-01-05T09:00:00"  # Monday 9 AM
      duration: 604800               # 1 week (seconds)
    layers:
      - name: "Primary"
        users:
          - "engineer-1"
          - "engineer-2"
          - "engineer-3"
          - "engineer-4"
        restrictions:
          - type: "daily"
            start_time: "09:00"
            end_time: "21:00"
      - name: "Secondary"
        users:
          - "senior-engineer-1"
          - "senior-engineer-2"
        restrictions:
          - type: "daily"
            start_time: "21:00"
            end_time: "09:00"

  escalation:
    name: "Escalation Policy"
    rules:
      - delay: 0
        targets:
          - type: "user"
            id: "primary-oncall"
      - delay: 15       # 15 minutes
        targets:
          - type: "user"
            id: "secondary-oncall"
      - delay: 30       # 30 minutes
        targets:
          - type: "user"
            id: "engineering-manager"
      - delay: 60       # 1 hour
        targets:
          - type: "user"
            id: "vp-engineering"
```

### Incident Commander Role

```
INCIDENT COMMANDER (IC) RESPONSIBILITIES:

1. ASSESS
   - Determine severity level
   - Identify affected services
   - Estimate user impact

2. MOBILIZE
   - Page relevant responders
   - Create war room (Slack channel, Zoom call)
   - Assign roles (IC, Communications, Technical Lead)

3. COORDINATE
   - Drive the investigation
   - Manage communications
   - Make decisions (rollback, failover, etc.)
   - Keep timeline updated

4. COMMUNICATE
   - Internal: Slack updates every 15 minutes
   - External: Status page updates per severity
   - Stakeholders: Direct communication for P1/P2

5. RESOLVE
   - Confirm resolution
   - Verify monitoring
   - Stand down responders
   - Schedule post-mortem
```

### Communication Templates

```markdown
# Status Page Update Templates

## Investigating
**Title:** Investigating: [Service Name] Degradation
**Body:** We are investigating reports of [symptom]. Our engineering team has been mobilized and is actively working to identify the root cause. We will provide updates every 15 minutes.
**Status:** Investigating

## Identified
**Title:** Identified: [Service Name] [Issue Description]
**Body:** We have identified the root cause as [brief description]. Our team is implementing a fix. Current estimated time to resolution: [estimate]. We apologize for the inconvenience.
**Status:** Identified

## Monitoring
**Title:** Monitoring: [Service Name] Recovery
**Body:** A fix has been implemented and we are monitoring recovery. Service is returning to normal levels. We will continue to monitor for the next 30 minutes before marking this incident as resolved.
**Status:** Monitoring

## Resolved
**Title:** Resolved: [Service Name] [Issue Description]
**Body:** This incident has been resolved. [Brief summary of what happened and what was done]. A full post-mortem will be published within 48 hours. We apologize for the impact to your service.
**Status:** Resolved
```

### Incident Slack Channel Template

```markdown
# Slack Channel: #incident-2026-01-15-database-outage

## Pinned Message
**Incident Commander:** @engineer-1
**Severity:** P1
**Status:** Investigating
**Affected Services:** Production Database, API, Web App
**War Room:** https://zoom.us/j/123456789

## Timeline
[03:17] PagerDuty alert fired: Database connection timeout
[03:19] @engineer-1 acknowledged alert, created incident channel
[03:22] IC determined P1 severity, paged DBA team
[03:25] Status page updated: Investigating
[03:30] Root cause identified: Disk space exhaustion on primary
[03:35] Decision: Failover to replica
[03:40] Failover complete, monitoring recovery
[03:55] Status page updated: Monitoring
[04:15] Status page updated: Resolved

## Current Action Items
- [x] Failover to replica
- [ ] Clean up disk space on primary
- [ ] Investigate why disk alerts were not triggered
- [ ] Schedule post-mortem for tomorrow
```

## The Production Way

### Incident Response Playbook

```python
# incident_response.py — Automated incident response
import json
import requests
from datetime import datetime
from enum import Enum

class Severity(Enum):
    P1 = "critical"
    P2 = "major"
    P3 = "minor"
    P4 = "low"

class IncidentManager:
    def __init__(self, config: dict):
        self.pagerduty_key = config['pagerduty_api_key']
        self.slack_webhook = config['slack_webhook_url']
        self.status_page_key = config['status_page_api_key']
        self.incidents = {}

    def create_incident(self, title: str, severity: Severity,
                       affected_services: list, commander: str) -> str:
        """Create a new incident and mobilize responders."""
        incident_id = f"INC-{datetime.now().strftime('%Y%m%d-%H%M%S')}"

        incident = {
            "id": incident_id,
            "title": title,
            "severity": severity,
            "status": "investigating",
            "affected_services": affected_services,
            "commander": commander,
            "created_at": datetime.now().isoformat(),
            "timeline": [],
            "resolvers": []
        }

        self.incidents[incident_id] = incident

        # 1. Create PagerDuty incident
        self._page_responders(incident)

        # 2. Create Slack channel
        self._create_slack_channel(incident)

        # 3. Update status page
        self._update_status_page(incident, "Investigating")

        # 4. Notify stakeholders
        self._notify_stakeholders(incident)

        return incident_id

    def update_incident(self, incident_id: str, status: str = None,
                       message: str = None, action: str = None):
        """Update incident status and timeline."""
        incident = self.incidents[incident_id]

        if status:
            incident["status"] = status

        # Add to timeline
        incident["timeline"].append({
            "timestamp": datetime.now().isoformat(),
            "status": status,
            "message": message,
            "action": action
        })

        # Update status page
        if status:
            self._update_status_page(incident, status)

        # Post to Slack
        if message:
            self._post_slack_update(incident, message)

    def resolve_incident(self, incident_id: str, resolution: str):
        """Resolve incident and schedule post-mortem."""
        incident = self.incidents[incident_id]
        incident["status"] = "resolved"
        incident["resolved_at"] = datetime.now().isoformat()
        incident["resolution"] = resolution

        # Update status page
        self._update_status_page(incident, "Resolved")

        # Notify Slack
        self._post_resolution(incident)

        # Schedule post-mortem
        self._schedule_postmortem(incident)

    def _page_responders(self, incident: dict):
        """Create PagerDuty incident."""
        response = requests.post(
            "https://api.pagerduty.com/incidents",
            headers={
                "Authorization": f"Token token={self.pagerduty_key}",
                "Content-Type": "application/json"
            },
            json={
                "incident": {
                    "type": "incident",
                    "title": f"[{incident['severity'].value.upper()}] {incident['title']}",
                    "service": {
                        "id": "PXXXXXX",
                        "type": "service_reference"
                    },
                    "urgency": "high" if incident['severity'] in [Severity.P1, Severity.P2] else "low",
                    "body": {
                        "type": "incident_body",
                        "details": f"Affected services: {', '.join(incident['affected_services'])}"
                    }
                }
            }
        )

    def _create_slack_channel(self, incident: dict):
        """Create dedicated Slack channel for incident."""
        channel_name = f"incident-{incident['id'].lower()}"

        # Create channel
        requests.post(
            "https://slack.com/api/conversations.create",
            headers={"Authorization": f"Bearer {self.slack_webhook}"},
            json={"name": channel_name}
        )

        # Post initial message
        requests.post(
            "https://slack.com/api/chat.postMessage",
            headers={"Authorization": f"Bearer {self.slack_webhook}"},
            json={
                "channel": channel_name,
                "text": f"*Incident Created*\n"
                        f"*ID:* {incident['id']}\n"
                        f"*Severity:* {incident['severity'].value.upper()}\n"
                        f"*Commander:* {incident['commander']}\n"
                        f"*Affected:* {', '.join(incident['affected_services'])}"
            }
        )

    def _update_status_page(self, incident: dict, status: str):
        """Update public status page."""
        status_mapping = {
            "Investigating": "investigating",
            "Identified": "identified",
            "Monitoring": "monitoring",
            "Resolved": "resolved"
        }

        requests.post(
            "https://api.statuspage.io/v1/pages/PAGE_ID/incidents",
            headers={"Authorization": f"OAuth {self.status_page_key}"},
            json={
                "incident": {
                    "name": incident['title'],
                    "status": status_mapping.get(status, "investigating"),
                    "body": f"Status: {status}"
                }
            }
        )

    def _post_slack_update(self, incident: dict, message: str):
        """Post update to incident Slack channel."""
        channel_name = f"incident-{incident['id'].lower()}"
        requests.post(
            "https://slack.com/api/chat.postMessage",
            headers={"Authorization": f"Bearer {self.slack_webhook}"},
            json={
                "channel": channel_name,
                "text": f"*Update:* {message}"
            }
        )

    def _notify_stakeholders(self, incident: dict):
        """Notify stakeholders based on severity."""
        if incident['severity'] in [Severity.P1, Severity.P2]:
            # Notify VP Engineering
            pass

    def _post_resolution(self, incident: dict):
        """Post resolution message."""
        pass

    def _schedule_postmortem(self, incident: dict):
        """Schedule post-mortem meeting."""
        pass
```

### Monitoring and Alerting Configuration

```yaml
# alerting-rules.yaml — Prometheus alerting rules for incident detection
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: incident-alerts
  namespace: monitoring
spec:
  groups:
    - name: incident.rules
      rules:
        # P1: Complete service outage
        - alert: ServiceDown
          expr: up{job="web"} == 0
          for: 2m
          labels:
            severity: critical
            incident: "true"
          annotations:
            summary: "Service {{ $labels.instance }} is down"
            description: "Service has been down for more than 2 minutes"
            runbook: "https://wiki.internal/runbooks/service-down"

        # P1: Database connection failure
        - alert: DatabaseConnectionFailure
          expr: pg_up == 0
          for: 1m
          labels:
            severity: critical
            incident: "true"
          annotations:
            summary: "Database connection failure"
            description: "Cannot connect to PostgreSQL"

        # P2: High error rate
        - alert: HighErrorRate
          expr: |
            sum(rate(http_requests_total{status=~"5.."}[5m]))
            / sum(rate(http_requests_total[5m])) > 0.05
          for: 5m
          labels:
            severity: warning
          annotations:
            summary: "High error rate: {{ $value | humanizePercentage }}"

        # P2: High latency
        - alert: HighLatency
          expr: |
            histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m])) > 2
          for: 5m
          labels:
            severity: warning
          annotations:
            summary: "p99 latency is {{ $value }}s"

        # P3: Disk space low
        - alert: DiskSpaceLow
          expr: |
            (node_filesystem_avail_bytes / node_filesystem_size_bytes) < 0.1
          for: 10m
          labels:
            severity: warning
          annotations:
            summary: "Disk space below 10% on {{ $labels.instance }}"
```

### PagerDuty Integration

```yaml
# pagerduty-integration.yaml — Kubernetes PagerDuty integration
apiVersion: v1
kind: ConfigMap
metadata:
  name: pagerduty-config
  namespace: monitoring
data:
  routing.yml: |
    # Route alerts to PagerDuty based on severity
    routes:
      - match:
          severity: critical
        receiver: pagerduty-critical
        group_wait: 10s
        group_interval: 5m
        repeat_interval: 5m

      - match:
          severity: warning
        receiver: pagerduty-warning
        group_wait: 30s
        group_interval: 15m
        repeat_interval: 1h

    receivers:
      - name: pagerduty-critical
        pagerduty_configs:
          - service_key: <PAGERDUTY_CRITICAL_KEY>
            severity: critical
            description: '{{ .CommonAnnotations.summary }}'

      - name: pagerduty-warning
        pagerduty_configs:
          - service_key: <PAGERDUTY_WARNING_KEY>
            severity: warning
            description: '{{ .CommonAnnotations.summary }}'
```

## Hands-On Lab: Simulate an Incident Response

### Step 1: Set Up the Scenario

```bash
# Create a "production" environment
kubectl create namespace incident-lab

# Deploy a web application
kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web
  namespace: incident-lab
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
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
---
apiVersion: v1
kind: Service
metadata:
  name: web
  namespace: incident-lab
spec:
  selector:
    app: web
  ports:
    - port: 80
EOF
```

### Step 2: Simulate an Incident

```bash
# Inject failure: Kill all pods
kubectl delete pods --all -n incident-lab

# Or: Corrupt the deployment
kubectl set image deployment/web web=nginx:nonexistent -n incident-lab

# Or: Resource exhaustion
kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web
  namespace: incident-lab
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
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 100m
              memory: 128Mi
          command: ["sh", "-c", "while true; do dd if=/dev/zero of=/tmp/fill bs=1M; done"]
EOF
```

### Step 3: Practice Incident Response

```bash
# 1. DETECT: Monitoring should alert
kubectl get events -n incident-lab --watch

# 2. ASSESS: Determine severity
echo "=== ASSESSMENT ==="
echo "Affected services: web"
echo "User impact: Complete outage"
echo "Severity: P1"

# 3. COMMUNICATE: Create incident channel (simulated)
echo "=== COMMUNICATION ==="
echo "Slack: #incident-$(date +%Y%m%d-%H%M%S)"
echo "Status page: Update to Investigating"

# 4. INVESTIGATE: Find root cause
echo "=== INVESTIGATION ==="
kubectl get pods -n incident-lab
kubectl describe pods -n incident-lab
kubectl logs -n incident-lab -l app=web --tail=50

# 5. RESOLVE: Fix the issue
echo "=== RESOLUTION ==="
# Rollback
kubectl rollout undo deployment/web -n incident-lab
# Or fix the deployment
kubectl set image deployment/web web=nginx:alpine -n incident-lab

# 6. VERIFY: Confirm resolution
kubectl rollout status deployment/web -n incident-lab
kubectl get pods -n incident-lab

# 7. UPDATE: Status page and stakeholders
echo "=== POST-INCIDENT ==="
echo "Status page: Update to Resolved"
echo "Schedule post-mortem"
```

### Lab Validation Checklist

- [ ] Incident detected within 5 minutes
- [ ] Severity correctly assessed
- [ ] Communication sent within 15 minutes
- [ ] Root cause identified
- [ ] Resolution implemented and verified
- [ ] Timeline documented
- [ ] Post-mortem scheduled

## Limitation -> Next Topic

You have an incident response process. But what happens when the on-call engineer opens the alert and does not know what to do? They need step-by-step instructions for every known failure scenario. They need runbooks.

**Next: [Module 77 — Runbooks & Playbooks](../77-runbooks-and-playbooks/README.md)**
