# Module 77: Runbooks & Playbooks — Documented Procedures for Every Scenario

> **Previous Module:** [76 - Incident Response](../76-incident-response/README.md)
> **Next Module:** [78 - Post-Mortems](../78-post-mortems/README.md)

## The Problem

An alert fires at 2 AM: "Database disk usage at 95%." The on-call engineer has never seen this before. They spend 30 minutes searching Slack history for how someone fixed it last time. They find a message from 6 months ago: "Just truncate the audit_logs table." They run it. It works. But they also deleted 3 months of compliance-critical audit data.

Without documented procedures, every incident becomes a research project, and tribal knowledge is lost when people leave the team.

## The Naive Way

```bash
# "The senior engineers know how to fix things"
# Or: "We have a wiki somewhere, I think"
# Or: "Just search Slack for the last time this happened"
```

**Why this fails:**
- Tribal knowledge is lost when people leave
- 2 AM brain cannot remember complex procedures
- No version control on procedures (is the wiki outdated?)
- No testing (does the procedure actually work?)
- No automation (manual steps are error-prone)

## The Right Way

### Runbook vs Playbook

```
RUNBOOK:
  What: Step-by-step procedure for a specific scenario
  Who: On-call engineer executing during an incident
  When: During an incident, under time pressure
  Example: "How to failover the database"

PLAYBOOK:
  What: Higher-level strategy for a class of incidents
  Who: Incident commander coordinating response
  When: At the start of an incident, to guide overall response
  Example: "How to handle a data breach" (detection, containment, notification, recovery)
```

### Runbook Template

```markdown
# Runbook: [Alert Name]

## Overview
**Alert:** [Prometheus alert name]
**Severity:** P1/P2/P3/P4
**Service:** [Affected service]
**Owner:** [Team responsible]
**Last Updated:** [Date]
**Last Tested:** [Date]

## Symptoms
- What the user sees: [description]
- What monitoring shows: [graphs/metrics]
- Related alerts: [other alerts that fire together]

## Impact
- Users affected: [all/some/none]
- Data at risk: [yes/no, which data]
- Revenue impact: [estimated]
- SLA impact: [minutes of downtime]

## Prerequisites
- [ ] Access to [system]
- [ ] Credentials for [service]
- [ ] Approval from [role] (if required)

## Diagnosis

### Step 1: Confirm the alert is real
```bash
# Check if the alert is still firing
kubectl get prometheusrules -n monitoring | grep [alert-name]

# Verify metrics
curl -s http://prometheus:9090/api/v1/query?query=[metric]
```

### Step 2: Identify root cause
```bash
# Check recent deployments
kubectl rollout history deployment/[service] -n production

# Check resource usage
kubectl top pods -n production -l app=[service]

# Check logs
kubectl logs -n production -l app=[service] --tail=100 --since=10m
```

### Step 3: Check related services
```bash
# Database status
kubectl exec -n production postgres-0 -- psql -U app -c "SELECT 1;"

# Redis status
kubectl exec -n production redis-0 -- redis-cli ping

# External dependencies
curl -s https://api.external-service.com/health
```

## Resolution

### Option A: [Most common resolution]
```bash
# Step-by-step commands
kubectl scale deployment [service] --replicas=10
```

### Option B: [Alternative resolution]
```bash
# Step-by-step commands
kubectl rollout undo deployment/[service]
```

### Option C: [Escalation]
If Options A and B do not work:
1. Page [senior engineer]
2. Join war room: [link]
3. Follow escalation runbook: [link]

## Verification
```bash
# Confirm resolution
kubectl get pods -n production -l app=[service]
curl -s https://[service]/health

# Check metrics are returning to normal
# [Prometheus query]
```

## Post-Resolution
- [ ] Update status page to Resolved
- [ ] Notify stakeholders
- [ ] Schedule post-mortem (if P1/P2)
- [ ] Update this runbook if new information learned

## History
| Date | Who | What Changed |
|------|-----|--------------|
| 2026-01-15 | @engineer-1 | Initial creation |
| 2026-03-20 | @engineer-2 | Added Option B |
```

### Common Runbooks

#### Runbook: Server Down

```markdown
# Runbook: Server Down

## Alert
`instance_down` — Node unreachable for > 2 minutes

## Diagnosis
```bash
# 1. Check node status
kubectl get nodes | grep -v Ready

# 2. Check node conditions
kubectl describe node [node-name] | grep -A 5 Conditions

# 3. Check node events
kubectl get events --field-selector involvedObject.name=[node-name]

# 4. Check cloud provider console
# AWS: EC2 instance status checks
# GCP: Compute Engine instance health
```

## Resolution
```bash
# Option 1: Drain and reboot
kubectl drain [node-name] --ignore-daemonsets --delete-emptydir-data
# Reboot via cloud console or SSH
# Wait for node to come back
kubectl uncordon [node-name]

# Option 2: Drain and terminate (if unrecoverable)
kubectl drain [node-name] --ignore-daemonsets --delete-emptydir-data
# Terminate instance in cloud console
# Cluster autoscaler will replace it

# Option 3: Cordon (prevent new pods, investigate)
kubectl cordon [node-name]
```

## Verification
```bash
kubectl get nodes
kubectl get pods -o wide | grep [node-name]
```
```

#### Runbook: Disk Full

```markdown
# Runbook: Disk Full

## Alert
`disk_usage_critical` — Disk usage > 90%

## Diagnosis
```bash
# 1. Find which disk is full
df -h

# 2. Find what's using space
du -sh /* | sort -rh | head -10
du -sh /var/log/* | sort -rh | head -10

# 3. Check for large files
find / -type f -size +100M -exec ls -lh {} \;

# 4. Check for deleted files still held open
lsof | grep deleted
```

## Resolution
```bash
# Option 1: Clean up logs
find /var/log -name "*.log" -mtime +7 -delete
journalctl --vacuum-time=7d

# Option 2: Clean up container images
docker system prune -a --volumes

# Option 3: Clean up old Kubernetes resources
kubectl get pods --all-namespaces | grep Evicted | awk '{print $2, $1}' | xargs -L1 kubectl delete pod -n

# Option 4: Expand disk (if cloud)
# AWS: Modify EBS volume
aws ec2 modify-volume --volume-id vol-xxx --size 200
# Resize filesystem
resize2fs /dev/xvda1
```

## Verification
```bash
df -h
# Disk usage should be < 80%
```
```

#### Runbook: High CPU

```markdown
# Runbook: High CPU

## Alert
`high_cpu_usage` — CPU > 80% for > 5 minutes

## Diagnosis
```bash
# 1. Find which pods are using CPU
kubectl top pods -n production --sort-by=cpu | head -10

# 2. Check if it's a single pod or all pods
kubectl top pods -n production -l app=[service]

# 3. Check recent changes
kubectl rollout history deployment/[service]

# 4. Check for runaway processes
kubectl exec -n production [pod-name] -- top -b -n 1
```

## Resolution
```bash
# Option 1: Scale horizontally
kubectl scale deployment [service] --replicas=[current*2]

# Option 2: Scale vertically (increase limits)
kubectl set resources deployment [service] -c=[container] --limits=cpu=2

# Option 3: Rollback (if caused by recent deployment)
kubectl rollout undo deployment/[service]

# Option 4: Kill runaway process
kubectl exec -n production [pod-name] -- kill -9 [pid]
```
```

#### Runbook: Database Failover

```markdown
# Runbook: Database Failover

## Alert
`database_replication_lag` or `database_down`

## Prerequisites
- [ ] Access to database admin credentials
- [ ] Approval from DBA or Engineering Manager (for P1)
- [ ] Maintenance window scheduled (if not emergency)

## Diagnosis
```bash
# 1. Check replication status
kubectl exec -n production postgres-primary -- psql -U app -c "
  SELECT client_addr, state, sent_lsn, write_lsn, flush_lsn, replay_lsn
  FROM pg_stat_replication;"

# 2. Check primary health
kubectl exec -n production postgres-primary -- psql -U app -c "SELECT 1;"

# 3. Check replica health
kubectl exec -n production postgres-replica-0 -- psql -U app -c "SELECT 1;"
```

## Resolution

### Automatic Failover (Patroni/Stolon)
```bash
# Check Patroni status
kubectl exec -n production patroni-0 -- patronictl list

# Trigger manual failover
kubectl exec -n production patroni-0 -- patronictl failover
```

### Manual Failover
```bash
# 1. Stop writes to primary
kubectl scale deployment api --replicas=0

# 2. Promote replica
kubectl exec -n production postgres-replica-0 -- psql -U app -c "
  SELECT pg_promote();"

# 3. Update connection strings
kubectl set env deployment/api DATABASE_URL=postgres://user:pass@postgres-replica-0:5432/app

# 4. Resume traffic
kubectl scale deployment api --replicas=3
```

## Verification
```bash
# Verify new primary is accepting writes
kubectl exec -n production postgres-replica-0 -- psql -U app -c "
  INSERT INTO health_check (status) VALUES ('ok');"

# Verify application connectivity
curl -s https://api.example.com/health
```
```

### Runbook Automation

```python
# runbook_automation.py — Automate runbook steps
import subprocess
import json
from typing import Callable, List

class RunbookStep:
    def __init__(self, name: str, command: str, verify: Callable = None):
        self.name = name
        self.command = command
        self.verify = verify

class AutomatedRunbook:
    def __init__(self, name: str):
        self.name = name
        self.steps: List[RunbookStep] = []
        self.results = []

    def add_step(self, name: str, command: str, verify: Callable = None):
        self.steps.append(RunbookStep(name, command, verify))

    def execute(self, dry_run: bool = False) -> dict:
        """Execute all runbook steps."""
        print(f"=== Executing Runbook: {self.name} ===")

        for i, step in enumerate(self.steps, 1):
            print(f"\nStep {i}: {step.name}")
            print(f"Command: {step.command}")

            if dry_run:
                print("[DRY RUN] Skipping execution")
                continue

            try:
                result = subprocess.run(
                    step.command,
                    shell=True,
                    capture_output=True,
                    text=True,
                    timeout=60
                )

                if result.returncode == 0:
                    print(f"SUCCESS: {result.stdout[:200]}")
                    self.results.append({"step": step.name, "status": "success"})
                else:
                    print(f"FAILED: {result.stderr[:200]}")
                    self.results.append({"step": step.name, "status": "failed", "error": result.stderr})

                    # Ask to continue or abort
                    if not self._confirm_continue():
                        break

            except subprocess.TimeoutExpired:
                print(f"TIMEOUT: Step exceeded 60 seconds")
                self.results.append({"step": step.name, "status": "timeout"})

        return {"runbook": self.name, "results": self.results}

    def _confirm_continue(self) -> bool:
        response = input("Continue to next step? (y/n): ")
        return response.lower() == 'y'


# Example: Disk Cleanup Runbook
disk_cleanup = AutomatedRunbook("Disk Cleanup")

disk_cleanup.add_step(
    "Check disk usage",
    "df -h | grep -E '^/dev'"
)

disk_cleanup.add_step(
    "Clean old logs",
    "find /var/log -name '*.log' -mtime +7 -delete"
)

disk_cleanup.add_step(
    "Clean container images",
    "docker system prune -a --volumes -f"
)

disk_cleanup.add_step(
    "Verify cleanup",
    "df -h | grep -E '^/dev'"
)

# Execute
result = disk_cleanup.execute(dry_run=False)
print(json.dumps(result, indent=2))
```

### Keeping Runbooks Updated

```yaml
# runbook-ci.yaml — CI/CD pipeline for runbook validation
name: Runbook Validation
on:
  push:
    paths:
      - 'runbooks/**'

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Validate runbook format
        run: |
          for file in runbooks/*.md; do
            echo "Validating $file..."
            # Check required sections
            for section in "## Overview" "## Diagnosis" "## Resolution" "## Verification"; do
              if ! grep -q "$section" "$file"; then
                echo "ERROR: Missing section '$section' in $file"
                exit 1
              fi
            done
          done

      - name: Check for outdated runbooks
        run: |
          for file in runbooks/*.md; do
            last_updated=$(grep "Last Updated:" "$file" | cut -d: -f2 | xargs)
            if [ -n "$last_updated" ]; then
              days_old=$(( ($(date +%s) - $(date -d "$last_updated" +%s)) / 86400 ))
              if [ $days_old -gt 90 ]; then
                echo "WARNING: $file is $days_old days old (last updated: $last_updated)"
              fi
            fi
          done
```

## The Production Way

### Runbook Repository Structure

```
runbooks/
├── README.md                    # Index of all runbooks
├── templates/
│   └── runbook-template.md      # Template for new runbooks
├── infrastructure/
│   ├── server-down.md
│   ├── disk-full.md
│   ├── high-cpu.md
│   ├── high-memory.md
│   └── network-issues.md
├── database/
│   ├── database-down.md
│   ├── replication-lag.md
│   ├── failover.md
│   └── slow-queries.md
├── application/
│   ├── high-error-rate.md
│   ├── high-latency.md
│   ├── pod-crashloop.md
│   └── deployment-failure.md
├── security/
│   ├── data-breach.md
│   ├── ddos-attack.md
│   └── unauthorized-access.md
└── scripts/
    ├── disk-cleanup.sh
    ├── failover.sh
    └── rollback.sh
```

### Runbook Integration with Alerting

```yaml
# alertmanager-config.yaml — Link alerts to runbooks
receivers:
  - name: 'critical-alerts'
    pagerduty_configs:
      - service_key: '<key>'
    webhook_configs:
      - url: 'http://runbook-automation:8080/alert'
        send_resolved: true
```

```python
# runbook_service.py — Service that links alerts to runbooks
from fastapi import FastAPI, Request
import yaml

app = FastAPI()

with open("runbooks/index.yaml") as f:
    RUNBOOK_INDEX = yaml.safe_load(f)

@app.post("/alert")
async def handle_alert(request: Request):
    alert = await request.json()
    alert_name = alert['labels']['alertname']
    runbook = RUNBOOK_INDEX.get(alert_name)

    if not runbook:
        return {"message": f"No runbook found for {alert_name}"}

    with open(runbook['path']) as f:
        content = f.read()

    return {
        "alert": alert_name,
        "runbook": runbook['path'],
        "content": content,
        "steps": runbook.get('steps', [])
    }
```

## Hands-On Lab: Write Runbooks for Common Scenarios

### Step 1: Create Runbook Template

```bash
mkdir -p runbooks

cat > runbooks/template.md << 'EOF'
# Runbook: [Alert Name]

## Overview
**Alert:**
**Severity:**
**Service:**
**Owner:**
**Last Updated:**
**Last Tested:**

## Symptoms
- What the user sees:
- What monitoring shows:
- Related alerts:

## Impact
- Users affected:
- Data at risk:
- Revenue impact:

## Prerequisites
- [ ] Access to
- [ ] Credentials for

## Diagnosis
### Step 1: Confirm the alert
```bash
```

### Step 2: Identify root cause
```bash
```

## Resolution
### Option A:
```bash
```

### Option B:
```bash
```

## Verification
```bash
```

## Post-Resolution
- [ ] Update status page
- [ ] Notify stakeholders
- [ ] Schedule post-mortem
EOF
```

### Step 2: Write Runbooks for Three Scenarios

```bash
# 1. Pod CrashLoopBackOff
cat > runbooks/pod-crashloop.md << 'EOF'
# Runbook: Pod CrashLoopBackOff

## Overview
**Alert:** KubePodCrashLooping
**Severity:** P2
**Service:** Any
**Owner:** Platform Team
**Last Updated:** 2026-06-11
**Last Tested:** 2026-06-11

## Symptoms
- Pod status shows CrashLoopBackOff
- Pod restarts continuously
- Users may see 502/503 errors

## Diagnosis
```bash
# Check pod status
kubectl get pods -n [namespace] | grep CrashLoop

# Check pod logs
kubectl logs -n [namespace] [pod-name] --previous

# Check pod events
kubectl describe pod -n [namespace] [pod-name]

# Check resource limits
kubectl get pod -n [namespace] [pod-name] -o jsonpath='{.spec.containers[0].resources}'
```

## Resolution
```bash
# Option 1: Fix the bug and redeploy
# (Identify error in logs, fix code, push, CI/CD deploys)

# Option 2: Rollback to previous version
kubectl rollout undo deployment/[service] -n [namespace]

# Option 3: Increase resources (if OOMKilled)
kubectl set resources deployment/[service] -n [namespace] \
  -c [container] --limits=memory=1Gi

# Option 4: Debug interactively
kubectl run debug --rm -it --image=[same-image] -- /bin/sh
```

## Verification
```bash
kubectl get pods -n [namespace] -l app=[service]
kubectl logs -n [namespace] -l app=[service] --tail=10
curl https://[service]/health
```
EOF

# 2. High Error Rate
cat > runbooks/high-error-rate.md << 'EOF'
# Runbook: High Error Rate

## Overview
**Alert:** HighErrorRate
**Severity:** P2
**Service:** API/Web
**Owner:** Backend Team
**Last Updated:** 2026-06-11
**Last Tested:** 2026-06-11

## Symptoms
- Error rate > 5% for 5 minutes
- Users see 500 errors
- Status page shows degradation

## Diagnosis
```bash
# Check error rate in Prometheus
sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m]))

# Check which endpoints are failing
sum(rate(http_requests_total{status=~"5.."}[5m])) by (path)

# Check recent deployments
kubectl rollout history deployment/[service]

# Check downstream services
curl -s https://database-service/health
curl -s https://redis-service/health

# Check logs for errors
kubectl logs -n production -l app=[service] --since=10m | grep ERROR
```

## Resolution
```bash
# Option 1: Rollback recent deployment
kubectl rollout undo deployment/[service]

# Option 2: Scale up (if load-related)
kubectl scale deployment [service] --replicas=10

# Option 3: Restart pods (if state corruption)
kubectl rollout restart deployment/[service]

# Option 4: Enable circuit breaker
kubectl set env deployment/[service] CIRCUIT_BREAKER_ENABLED=true
```

## Verification
```bash
# Error rate should be < 1%
sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m]))
```
EOF

# 3. Database Replication Lag
cat > runbooks/replication-lag.md << 'EOF'
# Runbook: Database Replication Lag

## Overview
**Alert:** DatabaseReplicationLag
**Severity:** P2
**Service:** PostgreSQL
**Owner:** DBA Team
**Last Updated:** 2026-06-11
**Last Tested:** 2026-06-11

## Symptoms
- Replica is behind primary by > 60 seconds
- Read queries return stale data
- Possible data inconsistency

## Diagnosis
```bash
# Check replication lag
kubectl exec -n production postgres-primary -- psql -U app -c "
  SELECT client_addr, state, sent_lsn, replay_lsn,
         sent_lsn - replay_lsn AS lag_bytes
  FROM pg_stat_replication;"

# Check replica status
kubectl exec -n production postgres-replica -- psql -U app -c "
  SELECT now() - pg_last_xact_replay_timestamp() AS lag;"

# Check replica resource usage
kubectl top pod -n production postgres-replica

# Check for long-running queries on replica
kubectl exec -n production postgres-replica -- psql -U app -c "
  SELECT pid, now() - query_start AS duration, query
  FROM pg_stat_activity
  WHERE state = 'active'
  ORDER BY duration DESC
  LIMIT 10;"
```

## Resolution
```bash
# Option 1: Kill long-running queries
kubectl exec -n production postgres-replica -- psql -U app -c "
  SELECT pg_terminate_backend(pid)
  FROM pg_stat_activity
  WHERE state = 'active'
  AND now() - query_start > interval '5 minutes';"

# Option 2: Increase replica resources
kubectl set resources statefulset postgres-replica -n production \
  -c postgres --limits=cpu=4,memory=8Gi

# Option 3: Rebuild replica (if lag is too large)
kubectl delete pod -n production postgres-replica
# Patroni will rebuild from primary backup

# Option 4: Redirect reads to primary temporarily
kubectl set env deployment/api -n production \
  DATABASE_READ_HOST=postgres-primary
```

## Verification
```bash
# Lag should be < 1 second
kubectl exec -n production postgres-replica -- psql -U app -c "
  SELECT now() - pg_last_xact_replay_timestamp() AS lag;"
```
EOF
```

### Step 3: Test Runbooks

```bash
# Simulate each scenario and follow the runbook

# 1. Create CrashLoopBackOff
kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: crashloop-app
  namespace: default
spec:
  replicas: 1
  selector:
    matchLabels:
      app: crashloop
  template:
    metadata:
      labels:
        app: crashloop
    spec:
      containers:
        - name: app
          image: busybox
          command: ["sh", "-c", "exit 1"]
EOF

# Follow runbook: pod-crashloop.md
kubectl get pods | grep crashloop
kubectl logs -l app=crashloop --previous
kubectl rollout undo deployment/crashloop-app

# Clean up
kubectl delete deployment crashloop-app
```

### Lab Validation Checklist

- [ ] Runbook template created
- [ ] Three runbooks written (CrashLoop, Error Rate, Replication Lag)
- [ ] Each runbook has: Overview, Diagnosis, Resolution, Verification
- [ ] Runbooks tested against simulated scenarios
- [ ] Runbooks stored in version control

## Limitation -> Next Topic

Runbooks tell you how to fix things. But after the incident is resolved, you need to understand why it happened and how to prevent it from happening again. You need post-mortems.

**Next: [Module 78 — Post-Mortems](../78-post-mortems/README.md)**
