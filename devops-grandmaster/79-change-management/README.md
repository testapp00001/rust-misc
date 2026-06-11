# Module 79: Change Management — Rolling Updates, Maintenance Windows

> **Previous Module:** [78 - Post-Mortems](../78-post-mortems/README.md)
> **Next Module:** [80 - Cost Optimization](../80-cost-optimization/README.md)

## The Problem

A developer pushes a database migration at 2 PM on Friday. It locks a critical table for 30 minutes. Orders cannot be processed. Customers call support. The CEO asks why a deployment happened during peak business hours. The developer did not know it was peak hours — nobody told them.

Change management is not bureaucracy — it is **the discipline of making changes safely**. The goal is to ensure that every change to production is planned, reviewed, communicated, and reversible.

## The Naive Way

```bash
# "Just deploy whenever you want"
kubectl apply -f new-version.yaml
# "YOLO deployment"
git push origin main --force
# Hope nothing breaks
```

**Why this fails:**
- No review process (bugs reach production)
- No communication (nobody knows when changes happen)
- No rollback plan (if it breaks, you are stuck)
- No change window (changes happen during peak hours)
- No risk assessment (high-risk changes treated same as low-risk)

## The Right Way

### Change Management Process

```
CHANGE REQUEST LIFECYCLE:

1. REQUEST
   Developer submits change request
   - What: Description of change
   - Why: Business justification
   - Risk: Low / Medium / High
   - Rollback: How to undo if it fails
   - Window: When to deploy

2. REVIEW
   Change Advisory Board (CAB) reviews
   - Technical review (code, tests, security)
   - Risk assessment
   - Rollback plan verification
   - Schedule approval

3. APPROVE
   CAB approves or rejects
   - Low risk: Auto-approved (CI/CD)
   - Medium risk: Team lead approval
   - High risk: CAB approval required

4. EXECUTE
   Change is deployed
   - During maintenance window
   - With monitoring active
   - With rollback plan ready

5. VERIFY
   Post-deployment verification
   - Health checks pass
   - No error rate increase
   - Performance baseline maintained

6. CLOSE
   Change is documented
   - Success/failure recorded
   - Lessons learned captured
   - Runbooks updated if needed
```

### Risk Assessment Matrix

```
CHANGE RISK ASSESSMENT:

                    Low Impact    Medium Impact    High Impact
                  +-----------+----------------+--------------+
Low Likelihood    |   Low     |     Low        |   Medium     |
of Failure        |           |                |              |
                  +-----------+----------------+--------------+
Medium Likelihood |   Low     |     Medium     |   High       |
of Failure        |           |                |              |
                  +-----------+----------------+--------------+
High Likelihood   |   Medium  |     High       |   Critical   |
of Failure        |           |                |              |
                  +-----------+----------------+--------------+

Examples:
  Low Risk:
    - Config change (feature flag)
    - Documentation update
    - Log level change

  Medium Risk:
    - Application deployment (rolling update)
    - Database index creation
    - Dependency version update

  High Risk:
    - Database schema migration
    - Infrastructure change (VPC, subnet)
    - Security policy change

  Critical Risk:
    - Database major version upgrade
    - Multi-region failover
    - Encryption key rotation
```

### Rolling Updates

```yaml
# rolling-update.yaml — Kubernetes rolling update strategy
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web
  namespace: production
spec:
  replicas: 10
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1      # At most 1 pod unavailable during update
      maxSurge: 2            # At most 2 extra pods during update
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
          image: web:v2.0.0
          ports:
            - containerPort: 8080
          # Readiness probe: only receive traffic when ready
          readinessProbe:
            httpGet:
              path: /ready
              port: 8080
            initialDelaySeconds: 10
            periodSeconds: 5
            failureThreshold: 3
          # Liveness probe: restart if stuck
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 30
            periodSeconds: 10
            failureThreshold: 3
          resources:
            requests:
              cpu: 500m
              memory: 512Mi
            limits:
              cpu: 1
              memory: 1Gi

  # Rollback if new version is unhealthy
  minReadySeconds: 30        # Pod must be ready for 30s before proceeding
  progressDeadlineSeconds: 600  # Fail if update takes > 10 minutes
```

### Canary Deployments

```yaml
# canary-deployment.yaml — Deploy to small percentage first
# Using Argo Rollouts
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: web
  namespace: production
spec:
  replicas: 10
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
          image: web:v2.0.0
          ports:
            - containerPort: 8080
  strategy:
    canary:
      steps:
        # Step 1: Deploy to 10% of pods
        - setWeight: 10
        - pause:
            duration: 5m          # Monitor for 5 minutes
        # Step 2: Check metrics
        - analysis:
            templates:
              - templateName: success-rate
        # Step 3: Deploy to 30%
        - setWeight: 30
        - pause:
            duration: 5m
        # Step 4: Deploy to 50%
        - setWeight: 50
        - pause:
            duration: 5m
        # Step 5: Deploy to 100%
        - setWeight: 100

      # Analysis template: check error rate
      analysis:
        templates:
          - templateName: success-rate
        args:
          - name: service-name
            value: web

---
apiVersion: argoproj.io/v1alpha1
kind: AnalysisTemplate
metadata:
  name: success-rate
spec:
  args:
    - name: service-name
  metrics:
    - name: success-rate
      interval: 1m
      successCondition: result[0] >= 0.99
      provider:
        prometheus:
          address: http://prometheus:9090
          query: |
            sum(rate(http_requests_total{service="{{args.service-name}}",status=~"2.."}[5m]))
            /
            sum(rate(http_requests_total{service="{{args.service-name}}"}[5m]))
```

### Maintenance Windows

```yaml
# maintenance-window.yaml — Schedule maintenance windows
# ConfigMap defining maintenance schedule
apiVersion: v1
kind: ConfigMap
metadata:
  name: maintenance-schedule
  namespace: engineering
data:
  schedule.yaml: |
    # Standard maintenance windows
    windows:
      - name: "Weekly Maintenance"
        day: "Tuesday"
        time: "02:00-06:00 UTC"
        timezone: "UTC"
        type: "standard"
        approvers: ["team-lead"]
        allowed_changes:
          - application_deployment
          - configuration_change
          - database_migration_minor

      - name: "Monthly Maintenance"
        day: "First Saturday"
        time: "00:00-08:00 UTC"
        timezone: "UTC"
        type: "extended"
        approvers: ["engineering-manager", "dba"]
        allowed_changes:
          - database_major_upgrade
          - infrastructure_change
          - security_patch

    # Change freeze periods
    freeze_periods:
      - name: "Holiday Freeze"
        start: "2026-12-20"
        end: "2027-01-03"
        reason: "Holiday period, reduced staffing"
        exceptions: ["security_patch", "critical_fix"]

      - name: "Quarter-End Freeze"
        start: "2026-03-28"
        end: "2026-04-02"
        reason: "Quarter-end financial processing"
        exceptions: ["critical_fix"]
```

### Rollback Procedures

```bash
#!/bin/bash
# rollback.sh — Automated rollback procedure
set -euo pipefail

DEPLOYMENT=$1
NAMESPACE=${2:-production}
EXPECTED_ROLLOUT_STATUS="successfully rolled out"

echo "=== Rolling back deployment: $DEPLOYMENT ==="

# Step 1: Check current status
echo "[1/5] Checking current deployment status..."
kubectl rollout status deployment/$DEPLOYMENT -n $NAMESPACE --timeout=60s || true

# Step 2: Rollback
echo "[2/5] Initiating rollback..."
kubectl rollout undo deployment/$DEPLOYMENT -n $NAMESPACE

# Step 3: Wait for rollback to complete
echo "[3/5] Waiting for rollback to complete..."
kubectl rollout status deployment/$DEPLOYMENT -n $NAMESPACE --timeout=300s

# Step 4: Verify health
echo "[4/5] Verifying service health..."
sleep 10
HEALTH=$(curl -s -o /dev/null -w "%{http_code}" https://$DEPLOYMENT/health)
if [ "$HEALTH" != "200" ]; then
  echo "ERROR: Health check failed with status $HEALTH"
  echo "MANUAL INTERVENTION REQUIRED"
  exit 1
fi

# Step 5: Notify
echo "[5/5] Rollback complete and verified."
echo "Deployment $DEPLOYMENT rolled back successfully."
echo "Please investigate the failure and create a post-mortem if needed."
```

## The Production Way

### Change Request Template

```yaml
# change-request.yaml — Formal change request
apiVersion: v1
kind: ConfigMap
metadata:
  name: change-request-2026-001
  namespace: engineering
data:
  request.yaml: |
    id: CR-2026-001
    title: "Deploy API v2.5.0 with new payment integration"
    requester: engineer-1
    date: 2026-01-15
    status: approved

    description: |
      Deploy API version 2.5.0 which includes:
      - New Stripe payment integration
      - Bug fix for order total calculation
      - Performance improvement for product search

    risk_assessment:
      likelihood: low
      impact: medium
      overall_risk: medium
      justification: |
        Well-tested in staging. Stripe integration tested with
        sandbox. Rollback plan verified.

    rollback_plan:
      method: kubectl rollout undo
      estimated_time: 2 minutes
      verification: Health check + manual test order

    testing:
      unit_tests: passed
      integration_tests: passed
      staging_deployment: verified
      load_test: passed (5000 RPS sustained)

    schedule:
      window: "2026-01-16 02:00 UTC"
      duration: "30 minutes"
      approvers: ["team-lead", "qa-lead"]

    communication:
      slack_channel: "#deployments"
      status_page: false
      stakeholders: ["product-team", "support-team"]
```

### Deployment Automation with Safety Checks

```python
# safe_deploy.py — Deployment with safety checks
import subprocess
import time
import sys

class SafeDeployment:
    def __init__(self, deployment: str, namespace: str = "production"):
        self.deployment = deployment
        self.namespace = namespace

    def deploy(self, image: str, dry_run: bool = False) -> bool:
        """Deploy with safety checks."""
        print(f"=== Safe Deployment: {self.deployment} ===")

        # Pre-flight checks
        if not self._pre_flight_checks():
            print("Pre-flight checks FAILED. Aborting.")
            return False

        if dry_run:
            print("[DRY RUN] Would deploy image:", image)
            return True

        # Deploy
        print(f"[1/6] Deploying {image}...")
        subprocess.run([
            "kubectl", "set", "image",
            f"deployment/{self.deployment}",
            f"{self.deployment}={image}",
            "-n", self.namespace
        ], check=True)

        # Wait for rollout
        print("[2/6] Waiting for rollout...")
        result = subprocess.run([
            "kubectl", "rollout", "status",
            f"deployment/{self.deployment}",
            "-n", self.namespace,
            "--timeout=300s"
        ], capture_output=True, text=True)

        if result.returncode != 0:
            print("Rollout FAILED. Initiating rollback...")
            self._rollback()
            return False

        # Post-deploy verification
        print("[3/6] Running health checks...")
        if not self._health_check():
            print("Health check FAILED. Initiating rollback...")
            self._rollback()
            return False

        print("[4/6] Checking error rate...")
        if not self._check_error_rate():
            print("Error rate elevated. Initiating rollback...")
            self._rollback()
            return False

        print("[5/6] Monitoring for 5 minutes...")
        time.sleep(300)

        print("[6/6] Final verification...")
        if not self._health_check():
            print("Post-monitoring health check FAILED. Initiating rollback...")
            self._rollback()
            return False

        print("Deployment SUCCESSFUL.")
        return True

    def _pre_flight_checks(self) -> bool:
        """Run pre-deployment checks."""
        checks = [
            self._check_image_exists(),
            self._check_resource_limits(),
            self._check_maintenance_window(),
        ]
        return all(checks)

    def _check_image_exists(self) -> bool:
        print("  Checking image exists...")
        return True

    def _check_resource_limits(self) -> bool:
        print("  Checking resource limits...")
        return True

    def _check_maintenance_window(self) -> bool:
        print("  Checking maintenance window...")
        return True

    def _health_check(self) -> bool:
        print("  Running health check...")
        # curl -s https://service/health
        return True

    def _check_error_rate(self) -> bool:
        print("  Checking error rate...")
        # Query Prometheus for error rate
        return True

    def _rollback(self):
        print("ROLLING BACK...")
        subprocess.run([
            "kubectl", "rollout", "undo",
            f"deployment/{self.deployment}",
            "-n", self.namespace
        ], check=True)
        subprocess.run([
            "kubectl", "rollout", "status",
            f"deployment/{self.deployment}",
            "-n", self.namespace,
            "--timeout=300s"
        ], check=True)
        print("Rollback complete.")
```

### Change Freeze Enforcement

```yaml
# change-freeze-gate.yaml — CI/CD gate that blocks deployments during freeze
name: Deploy Gate
on:
  push:
    branches: [main]

jobs:
  check-freeze:
    runs-on: ubuntu-latest
    steps:
      - name: Check change freeze
        run: |
          TODAY=$(date +%Y-%m-%d)

          # Define freeze periods
          FREEZE_START="2026-12-20"
          FREEZE_END="2027-01-03"

          if [[ "$TODAY" > "$FREEZE_START" && "$TODAY" < "$FREEZE_END" ]]; then
            echo "::error::Change freeze in effect ($FREEZE_START to $FREEZE_END)"
            echo "Deployments are blocked during the holiday freeze."
            echo "For critical security patches, request an exception."
            exit 1
          fi

          echo "No change freeze in effect. Proceeding with deployment."

  deploy:
    needs: check-freeze
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Deploy
        run: |
          kubectl set image deployment/web web=$IMAGE
```

## Hands-On Lab: Plan and Execute a Maintenance Window

### Step 1: Plan the Maintenance

```bash
# Create maintenance plan
cat > maintenance-plan.md << 'EOF'
# Maintenance Plan: Database Schema Migration

## Date: 2026-06-15 02:00-04:00 UTC
## Change: Add index to orders table for performance
## Risk: Medium (database migration, but non-destructive)
## Owner: @dba-team
## Approver: @engineering-manager

## Pre-Maintenance Checklist
- [ ] Backup verified (< 1 hour old)
- [ ] Migration tested in staging
- [ ] Rollback script prepared
- [ ] Monitoring dashboard open
- [ ] Communication sent to stakeholders
- [ ] On-call engineer available

## Steps
1. Enable maintenance mode (read-only)
2. Run migration
3. Verify index created
4. Disable maintenance mode
5. Monitor for 30 minutes
6. Close maintenance window

## Rollback
1. Enable maintenance mode
2. DROP INDEX CONCURRENTLY
3. Disable maintenance mode

## Communication
- Slack: #maintenance-20260615
- Status page: Maintenance notice posted
- Stakeholders: Product, Support, Engineering
EOF
```

### Step 2: Execute the Maintenance

```bash
# 1. Enable maintenance mode
kubectl set env deployment/api MAINTENANCE_MODE=true

# 2. Run migration (simulated)
kubectl exec -n production postgres-0 -- psql -U app -c "
  CREATE INDEX CONCURRENTLY idx_orders_created_at ON orders(created_at);"

# 3. Verify
kubectl exec -n production postgres-0 -- psql -U app -c "
  SELECT indexname FROM pg_indexes WHERE tablename = 'orders';"

# 4. Disable maintenance mode
kubectl set env deployment/api MAINTENANCE_MODE-

# 5. Monitor
watch -n 5 'curl -s -o /dev/null -w "%{http_code}" https://api/health'

# 6. Close maintenance window
echo "Maintenance complete. Monitoring for 30 minutes."
```

### Lab Validation Checklist

- [ ] Maintenance plan documented
- [ ] Risk assessment completed
- [ ] Rollback plan prepared and tested
- [ ] Communication sent before maintenance
- [ ] Maintenance executed within window
- [ ] Post-maintenance verification complete
- [ ] Maintenance window closed and documented

## Limitation -> Next Topic

You have safe change processes. But all these processes, infrastructure, and services cost money. Cloud bills are growing. Are you spending efficiently? Are you paying for resources you do not use? Can you reduce costs without sacrificing reliability?

**Next: [Module 80 — Cost Optimization](../80-cost-optimization/README.md)**
