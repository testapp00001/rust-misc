# Solution 04: Incident Response Simulation

## Part A: Initial Detection and Assessment

### Scenario

It is 02:47 AM. PagerDuty fires an alert: "Production Kubernetes pods in
CrashLoopBackOff state. Service: payment-api. Namespace: production. Pod
count affected: 8/10."

### Step 1: Acknowledge and Assess (02:47 - 02:52)

```bash
# Acknowledge the PagerDuty alert
# (In practice, tap "Acknowledge" in the PagerDuty mobile app)

# Set up your terminal environment
export KUBECONFIG=~/.kube/production.config
export INCIDENT_CHANNEL="#inc-20240315-payment-crashloop"

# First: check what is actually happening
kubectl get pods -n production -l app=payment-api -o wide

# Expected output:
# NAME                            READY   STATUS             RESTARTS   AGE   NODE
# payment-api-7f8b9c6d4-abc12    0/1     CrashLoopBackOff   5          12m   node-3
# payment-api-7f8b9c6d4-def34    0/1     CrashLoopBackOff   5          12m   node-1
# payment-api-7f8b9c6d4-ghi56    0/1     CrashLoopBackOff   5          12m   node-2
# payment-api-7f8b9c6d4-jkl78    0/1     CrashLoopBackOff   5          12m   node-3
# payment-api-7f8b9c6d4-mno90    0/1     CrashLoopBackOff   5          12m   node-1
# payment-api-7f8b9c6d4-pqr12    0/1     CrashLoopBackOff   5          12m   node-2
# payment-api-7f8b9c6d4-stu34    0/1     CrashLoopBackOff   5          12m   node-3
# payment-api-7f8b9c6d4-vwx56    0/1     CrashLoopBackOff   5          12m   node-1
# payment-api-7f8b9c6d4-yza78    1/1     Running            0          3h    node-2
# payment-api-7f8b9c6d4-bcd90    1/1     Running            0          3h    node-1

# 8 out of 10 pods are crashing. 2 old pods still running.
# The 8 crashing pods are from a recent deployment (12 min old).
```

**Slack update at 02:50:**
```
[02:50 UTC] INCIDENT DECLARED - P1
  Service: payment-api
  Impact: 8/10 pods in CrashLoopBackOff
  Assessment: Recent deployment appears to be the cause
  Action: Investigating pod logs
  IC: @on-call-engineer
```

### Why This Works

The first command is always `kubectl get pods` with `-o wide` to see which
nodes are affected and `-l` to filter by the relevant label. Seeing 8/10 pods
crashing while 2 old pods are still running immediately tells you this is a
bad deployment -- the 2 running pods are from the previous ReplicaSet. This
narrows the investigation significantly.

## Part B: Root Cause Investigation (02:52 - 03:05)

### Step 2: Check Pod Logs

```bash
# Get logs from a crashing pod
kubectl logs payment-api-7f8b9c6d4-abc12 -n production --tail=50

# If the pod has restarted, get logs from the previous container
kubectl logs payment-api-7f8b9c6d4-abc12 -n production --previous --tail=50

# Expected output:
# 2024-03-15T02:47:12Z INFO  Starting payment-api v2.14.3
# 2024-03-15T02:47:12Z INFO  Connecting to database...
# 2024-03-15T02:47:13Z INFO  Database connection established
# 2024-03-15T02:47:13Z INFO  Running migrations...
# 2024-03-15T02:47:14Z INFO  Migration 20240315_add_payment_index starting...
# 2024-03-15T02:47:45Z ERROR Migration 20240315_add_payment_index failed!
# 2024-03-15T02:47:45Z ERROR Error: relation "payments_old" does not exist
# 2024-03-15T02:47:45Z FATAL Application startup failed. Exiting.
```

### Step 3: Check Recent Deployments

```bash
# Check the deployment history
kubectl rollout history deployment/payment-api -n production

# Expected output:
# REVISION  CHANGE-CAUSE
# 1         Initial deployment
# 2         Scale to 8 replicas
# 3         Update to v2.14.2
# 4         Update to v2.14.3  <-- current

# Check what changed between v2.14.2 and v2.14.3
kubectl get deployment payment-api -n production -o jsonpath='{.spec.template.spec.containers[0].image}'

# Output: registry.example.com/payment-api:v2.14.3
```

### Step 4: Check Events

```bash
# Check Kubernetes events for the namespace
kubectl get events -n production --sort-by='.lastTimestamp' | tail -20

# Expected output shows:
# Warning  BackOff    pod/payment-api-7f8b9c6d4-abc12  Back-off restarting failed container
# Warning  Unhealthy  pod/payment-api-7f8b9c6d4-abc12  Readiness probe failed
# Normal   Pulling    pod/payment-api-7f8b9c6d4-abc12  Pulling image "payment-api:v2.14.3"
```

### Step 5: Verify the Issue is the Migration

```bash
# Check if the migration references a table that does not exist
kubectl exec -it deployment/payment-api -n production -- cat /app/migrations/20240315_add_payment_index.sql 2>/dev/null || \
  echo "Cannot exec into crashing pod, checking configmap instead"

# Alternative: check the migration file from the configmap or secret
kubectl get configmap payment-api-migrations -n production -o yaml | grep -A 5 "20240315"

# Check database state
kubectl exec -it deployment/postgres-primary -n production -- \
  psql -U payment_user -d payment_db -c "\dt payments*"

# Expected output:
#         List of relations
#  Schema |     Name      | Type
# --------+---------------+------
#  public | payments      | table
#  public | payments_v2   | table
#
# Note: payments_old does NOT exist -- this is the root cause
```

**Slack update at 03:00:**
```
[03:00 UTC] ROOT CAUSE IDENTIFIED
  Migration 20240315_add_payment_index references table "payments_old"
  which was dropped in a previous cleanup. The migration was never tested
  against the current database schema.
  Impact: All new pods fail to start due to fatal migration error
  Old pods (v2.14.2) are still serving traffic but at reduced capacity
  Plan: Roll back deployment to v2.14.2
```

### Why This Works

The investigation follows a systematic path: logs first (what is the error?),
then deployment history (what changed?), then events (what is Kubernetes
doing?), then verification (confirming the hypothesis). Each step narrows
the problem space. The `--previous` flag on `kubectl logs` is crucial -- when
a pod is in CrashLoopBackOff, the current container may not have logs yet,
but the previous crashed container does.

## Part C: Resolution (03:05 - 03:15)

### Step 6: Roll Back the Deployment

```bash
# Roll back to the previous working revision (v2.14.2)
kubectl rollout undo deployment/payment-api -n production

# Monitor the rollback progress
kubectl rollout status deployment/payment-api -n production --timeout=120s

# Expected output:
# Waiting for deployment "payment-api" rollout to finish: 4 out of 10 updated replicas are available
# Waiting for deployment "payment-api" rollout to finish: 6 out of 10 updated replicas are available
# Waiting for deployment "payment-api" rollout to finish: 8 out of 10 updated replicas are available
# Waiting for deployment "payment-api" rollout to finish: 9 out of 10 updated replicas are available
# deployment "payment-api" successfully rolled out

# Verify all pods are running
kubectl get pods -n production -l app=payment-api

# Expected output:
# NAME                            READY   STATUS    RESTARTS   AGE
# payment-api-6a7b8c9d0-xyz12    1/1     Running   0          45s
# payment-api-6a7b8c9d0-uvw34    1/1     Running   0          45s
# ... (all 10 pods running)

# Verify the image version is correct
kubectl get deployment payment-api -n production \
  -o jsonpath='{.spec.template.spec.containers[0].image}'
# Output: registry.example.com/payment-api:v2.14.2
```

### Step 7: Verify Service Health

```bash
# Check that the service is responding
kubectl get endpoints payment-api -n production

# Test the health endpoint
kubectl run curl-test --image=curlimages/curl --rm -it --restart=Never -- \
  curl -s https://payment-api.production.svc.cluster.local/health

# Expected output:
# {"status":"healthy","version":"2.14.2","uptime":15}

# Check error rates from inside the cluster
kubectl run curl-test --image=curlimages/curl --rm -it --restart=Never -- \
  curl -s https://payment-api.production.svc.cluster.local/metrics | grep error_rate

# Expected: error_rate 0.001 (back to normal)
```

### Step 8: Scale Down Old Pods Gracefully

```bash
# The old pods from v2.14.3 should already be terminated by the rollback
# Verify no old ReplicaSets have running pods
kubectl get replicasets -n production -l app=payment-api

# Expected: Only the v2.14.2 ReplicaSet has 10 ready replicas
# Old ReplicaSets show 0/0

# Clean up old ReplicaSets (optional, Kubernetes does this automatically)
kubectl delete replicaset -n production -l app=payment-api \
  --field-selector=status.replicas=0
```

**Slack update at 03:12:**
```
[03:12 UTC] RESOLUTION IN PROGRESS
  Rollback to v2.14.2 completed. All 10 pods running and healthy.
  Error rate returned to baseline (0.1%).
  Monitoring for 30 minutes before declaring resolution.
  Next update: 03:42 UTC
```

### Why This Works

`kubectl rollout undo` is the fastest way to revert a bad deployment. It uses
the ReplicaSet history that Kubernetes maintains automatically. The
`--timeout` flag prevents the command from hanging indefinitely. Verifying
the service health after rollback is critical -- sometimes the rollback
itself can cause issues (e.g., if the bad deployment modified shared state).

## Part D: Post-Incident Checklist

```
Post-Incident Checklist
=======================

[ ] 1. Confirm resolution
     - All pods running and healthy
     - Error rates at baseline
     - No customer reports of issues

[ ] 2. Update status page
     - Status: Resolved
     - Duration: 02:47 - 03:30 UTC (43 minutes)
     - Root cause: Bad database migration in v2.14.3

[ ] 3. Close incident Slack channel
     - Pin final timeline to channel
     - Do NOT delete the channel -- archive it

[ ] 4. Notify stakeholders
     - Send summary email to engineering-all@
     - Update Jira ticket INC-2024-0315-001

[ ] 5. Schedule post-incident review
     - Date: 2024-03-17 10:00 UTC
     - Attendees: IC, TL, on-call, deployment author, DBA
     - Location: https://meet.google.com/pir-20240315

[ ] 6. Create follow-up tickets
     - ENG-1234: Add migration validation to CI/CD pipeline
     - ENG-1235: Add pre-deployment schema compatibility check
     - ENG-1236: Implement canary deployment for payment-api

[ ] 7. Preserve evidence
     - Export pod logs to S3: s3://incidents/20240315/payment-api-logs/
     - Save deployment manifest diff
     - Screenshot Grafana dashboards for the incident window

[ ] 8. Update runbook
     - Add "CrashLoopBackOff due to migration failure" to payment-api runbook
     - Include the rollback command and expected recovery time

[ ] 9. Notify PagerDuty
     - Mark incident as resolved in PagerDuty
     - Add resolution note with root cause and fix

[ ] 10. Self-care
      - Log off and rest. You were up at 2:47 AM.
      - The post-incident review can wait until Monday.
```

### Why This Works

The checklist exists because humans forget things under stress. After a 43-minute
incident at 3 AM, the engineer will not remember to archive the Slack channel
or export the logs. The checklist ensures nothing falls through the cracks.
The self-care item is not a joke -- incident response is stressful, and
burnout is a real risk for on-call engineers.

## Part E: Complete Timeline Summary

```
Incident Timeline: INC-2024-0315-001
======================================

Time (UTC)  Event                              Actor
----------  -----                              -----
02:47       PagerDuty alert fires              PagerDuty
02:48       Alert acknowledged                 @on-call-engineer
02:50       Incident declared (P1)             @on-call-engineer
02:52       Pod logs reviewed                  @on-call-engineer
02:55       Deployment history checked         @on-call-engineer
02:58       Database schema verified           @on-call-engineer
03:00       Root cause identified              @on-call-engineer
03:02       IC paged for coordination          PagerDuty
03:05       Rollback initiated                 @on-call-engineer
03:08       Rollback completed                 @on-call-engineer
03:12       Health verification passed         @on-call-engineer
03:15       Status page: "Monitoring"          @ic-alice
03:30       30-min observation complete        @on-call-engineer
03:32       Status page: "Resolved"            @ic-alice
03:35       Stakeholder email sent             @ic-alice
03:40       Post-incident review scheduled     @ic-alice
03:45       Incident channel archived          @ic-alice

Total time to detect:     0 minutes (automated alert)
Total time to identify:   13 minutes (02:47 to 03:00)
Total time to resolve:    21 minutes (03:00 to 03:21, using rollback)
Total incident duration:  43 minutes (02:47 to 03:30)
```

### Why This Works

The timeline is the single most valuable artifact from an incident. It
captures what happened, when, and who did it. During the post-incident
review, this timeline will be the focal point for discussion. Entries are
written in past tense with specific times and actors, making it easy to
reconstruct the sequence of events even months later.

## Common Mistakes

1. **Trying to fix forward instead of rolling back.** When a deployment causes
   the outage, the fastest resolution is almost always a rollback. Trying to
   fix the migration, rebuild the image, and redeploy takes 30+ minutes.
   `kubectl rollout undo` takes 2 minutes. Roll back first, fix forward later.

2. **Not verifying the rollback worked.** Running `kubectl rollout undo` and
   assuming it worked is dangerous. Always verify: check pod status, check
   the image version, test the health endpoint, and monitor error rates.

3. **Skipping the observation period.** Declaring "Resolved" immediately after
   the rollback is tempting, especially at 3 AM. But if the issue was caused
   by a gradual resource leak, it might take 30 minutes to manifest again.
   The observation period catches these.

4. **Forgetting to preserve evidence.** Pod logs disappear when pods are
   deleted. Grafana dashboards lose resolution over time. Export everything
   you need for the post-incident review immediately, while the data is
   fresh.

5. **Not involving the deployment author in the PIR.** The person who wrote
   the bad migration is not the villain -- they are the most valuable person
   in the post-incident review. They can explain why the migration assumed
   `payments_old` existed and what checks were missing.

## Key Takeaway

Incident response is a process, not heroics. Follow the steps: detect,
assess, identify, resolve, verify, document. Use `kubectl` commands
systematically rather than randomly trying things. Roll back first, fix
forward later. And always, always document the timeline -- your future self
(and your post-incident review) will thank you.
