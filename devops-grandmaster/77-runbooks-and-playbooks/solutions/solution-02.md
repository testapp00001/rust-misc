# Solution 02: Write a Pod CrashLoopBackOff Runbook

## Problem Statement

Write a complete, production-ready runbook for diagnosing and resolving
Kubernetes pods stuck in `CrashLoopBackOff` state. The runbook must follow
the standard template and be usable by any on-call engineer.

## Complete Runbook

```markdown
# RUNBOOK: Pod in CrashLoopBackOff

| Field            | Value                                      |
|------------------|--------------------------------------------|
| **Runbook ID**   | RB-K8S-001                                 |
| **Severity**     | P2 - High                                  |
| **Owner**        | Platform Engineering                       |
| **Last Updated** | 2026-06-11                                 |
| **Review Cycle** | Quarterly                                  |
| **Tags**         | kubernetes, pod, crashloop, restart, k8s   |

---

## 1. Overview

A pod in `CrashLoopBackOff` status means Kubernetes started the container,
but the container process exited (crashed), and Kubernetes is repeatedly
restarting it with exponential backoff delays (10s, 20s, 40s, ... up to 5m).

This runbook covers diagnosis and resolution of CrashLoopBackOff pods,
including common root causes such as application bugs, misconfigured
environment variables, missing secrets, failed health checks, resource
exhaustion, and dependency unavailability.

---

## 2. Symptoms

The following symptoms indicate a CrashLoopBackOff situation:

- `kubectl get pods` shows pod status as `CrashLoopBackOff`
- Pod `RESTARTS` count is incrementing
- `kubectl get events` shows `BackOff` events for the pod
- Application logs show the container exiting shortly after start
- Monitoring dashboards show gaps in metrics from the affected service
- Alert: `KubePodCrashLooping` fires in Alertmanager

```
$ kubectl get pods -n production
NAME                         READY   STATUS             RESTARTS      AGE
payment-api-7d9f4b6c8-x2k9l 0/1     CrashLoopBackOff   12 (47s ago)  4h
```

---

## 3. Impact

- **Service degradation**: The affected pod is not serving traffic. If all
  replicas are in CrashLoopBackOff, the service is fully down.
- **Resource waste**: Repeated container starts consume node CPU, memory,
  and disk (for image pulls and container logs).
- **Cascading failures**: Downstream services that depend on this pod may
  experience timeouts or errors.
- **Alert fatigue**: Frequent restarts can trigger repeated alerts.

**Determine impact scope:**

```bash
# Check how many replicas are affected
kubectl get pods -n <namespace> -l app=<app-name> --field-selector=status.phase!=Running

# Check if any healthy replicas remain
kubectl get pods -n <namespace> -l app=<app-name> -o wide | grep Running
```

---

## 4. Prerequisites

Before executing this runbook, ensure you have:

- [ ] `kubectl` access to the affected cluster
- [ ] Namespace read/write permissions for the target namespace
- [ ] Access to the container registry (to check image availability)
- [ ] Access to monitoring dashboards (Grafana, Prometheus)
- [ ] Access to centralized logging (ELK, Loki, CloudWatch Logs)
- [ ] Knowledge of the application's expected startup behavior

---

## 5. Diagnosis

### Step 1: Gather Basic Pod Information

```bash
# Get pod status details
kubectl describe pod <pod-name> -n <namespace>
```

**What to look for in the output:**

```
# In the "State" section:
State:          Waiting
  Reason:       CrashLoopBackOff

# In the "Last State" section:
Last State:     Terminated
  Reason:       Error
  Exit Code:    1          <-- Non-zero means the application crashed
  Started:      Wed, 11 Jun 2026 10:15:00 +0000
  Finished:     Wed, 11 Jun 2026 10:15:03 +0000  <-- Crashed after 3 seconds

# In the "Events" section:
Events:
  Type     Reason     Age                  From     Message
  ----     ------     ----                 ----     -------
  Warning  BackOff    2m (x15 over 5m)    kubelet  Back-off restarting failed container
```

### Step 2: Check Container Logs

```bash
# Current container logs (if the container is running)
kubectl logs <pod-name> -n <namespace>

# Previous container's logs (the one that crashed)
kubectl logs <pod-name> -n <namespace> --previous

# If the pod has multiple containers
kubectl logs <pod-name> -n <namespace> -c <container-name> --previous

# Tail logs in real-time (watch for the next crash)
kubectl logs <pod-name> -n <namespace> -f --tail=100
```

**Common log patterns and their meanings:**

```
Pattern: "panic: <error>"
Cause:   Application bug or unhandled edge case

Pattern: "Error: Cannot find module '<module>'"
Cause:   Missing dependency in the container image

Pattern: "FATAL: password authentication failed for user '<user>'"
Cause:   Wrong database credentials

Pattern: "dial tcp <ip>:<port>: connect: connection refused"
Cause:   Dependency service is unreachable

Pattern: "OOMKilled" (check Last State exit code 137)
Cause:   Container exceeded its memory limit
```

### Step 3: Check Resource Usage and Limits

```bash
# Check if the pod was OOMKilled
kubectl get pod <pod-name> -n <namespace> -o jsonpath='{.status.containerStatuses[0].lastState.terminated.reason}'

# Check current resource usage (if metrics-server is installed)
kubectl top pod <pod-name> -n <namespace>

# Check the configured resource limits
kubectl get pod <pod-name> -n <namespace> -o jsonpath='{.spec.containers[0].resources}' | jq .
```

### Step 4: Check Environment and Dependencies

```bash
# Verify ConfigMaps and Secrets are mounted
kubectl get pod <pod-name> -n <namespace> -o jsonpath='{.spec.containers[0].envFrom}' | jq .

# Check if referenced Secrets exist
kubectl get secrets -n <namespace>

# Check if referenced ConfigMaps exist
kubectl get configmaps -n <namespace>

# Verify the container image exists and is pullable
kubectl get pod <pod-name> -n <namespace> -o jsonpath='{.spec.containers[0].image}'
```

### Step 5: Check Health Check Configuration

```bash
# Inspect liveness and readiness probes
kubectl get pod <pod-name> -n <namespace> -o json | jq '.spec.containers[0].livenessProbe, .spec.containers[0].readinessProbe'

# Check if the probe endpoint is reachable from within the cluster
kubectl exec -it <healthy-pod> -n <namespace> -- curl -v http://<service>:<port>/healthz
```

### Step 6: Check Events and Node Status

```bash
# Cluster-wide events related to the pod
kubectl get events -n <namespace> --field-selector involvedObject.name=<pod-name> --sort-by='.lastTimestamp'

# Check node conditions (disk pressure, memory pressure)
kubectl describe node <node-name> | grep -A5 "Conditions:"
```

### Diagnosis Decision Tree

```
Pod is CrashLoopBackOff
        |
        v
Check exit code in "Last State"
        |
   +----+----+----+----+
   |         |         |
  Exit 0  Exit 1   Exit 137
   |         |         |
   v         v         v
App exits  App crash  OOMKilled
normally   |         |
   |       Check app  Increase memory
   v       logs for   limit or fix
Check       error     memory leak
liveness    message
probe or         |
startup          v
script      +----+----+
            |         |
         Missing    Code
         dependency  error
            |         |
            v         v
         Fix env    Fix bug
         or config  or rollback
```

---

## 6. Resolution

### Option A: Fix the Root Cause (Preferred)

Apply this option when the root cause is identified and can be fixed quickly.

**Example: Fix a missing environment variable**

```bash
# Patch the deployment with the correct env var
kubectl set env deployment/<deployment-name> -n <namespace> \
  DATABASE_URL=postgresql://user:pass@db-host:5432/mydb

# Or edit the deployment directly
kubectl edit deployment/<deployment-name> -n <namespace>
```

**Example: Fix insufficient memory limits**

```bash
# Increase memory limit
kubectl patch deployment <deployment-name> -n <namespace> --type='json' -p='[
  {"op": "replace", "path": "/spec/template/spec/containers/0/resources/limits/memory", "value": "512Mi"},
  {"op": "replace", "path": "/spec/template/spec/containers/0/resources/requests/memory", "value": "256Mi"}
]'
```

**Example: Fix a ConfigMap reference**

```bash
# Create the missing ConfigMap
kubectl create configmap <config-name> -n <namespace> \
  --from-literal=key1=value1 \
  --from-literal=key2=value2

# Restart the pods to pick up the ConfigMap
kubectl rollout restart deployment/<deployment-name> -n <namespace>
```

### Option B: Rollback to Previous Working Version

Apply this option when a recent deployment introduced the crash.

```bash
# Check rollout history
kubectl rollout history deployment/<deployment-name> -n <namespace>

# Rollback to the previous revision
kubectl rollout undo deployment/<deployment-name> -n <namespace>

# Rollback to a specific revision
kubectl rollout undo deployment/<deployment-name> -n <namespace> --to-revision=<N>

# Watch the rollback progress
kubectl rollout status deployment/<deployment-name> -n <namespace> --timeout=120s
```

### Option C: Increase CrashLoopBackOff Tolerance (Temporary)

Apply this option as a **temporary** measure while investigating, to prevent
the exponential backoff from delaying recovery.

```bash
# Delete the crashing pod to reset the backoff timer
kubectl delete pod <pod-name> -n <namespace>

# This allows Kubernetes to immediately attempt a fresh restart,
# giving you a clean window to observe startup behavior
```

**Warning**: This does not fix the root cause. The pod will re-enter
CrashLoopBackOff if the underlying issue persists.

---

## 7. Verification

After applying the resolution, verify the fix:

```bash
# 1. Confirm pod is Running
kubectl get pods -n <namespace> -l app=<app-name> -w
# Expected: STATUS = Running, READY = 1/1, RESTARTS not increasing

# 2. Confirm all replicas are healthy
kubectl get deployment/<deployment-name> -n <namespace>
# Expected: READY matches DESIRED (e.g., 3/3)

# 3. Check recent events for errors
kubectl get events -n <namespace> --sort-by='.lastTimestamp' | tail -20

# 4. Verify the application is responding
kubectl exec -it <pod-name> -n <namespace> -- curl -sf http://localhost:8080/healthz
# Expected: HTTP 200 with healthy response body

# 5. Confirm the service is receiving and processing traffic
kubectl logs <pod-name> -n <namespace> --tail=50 | grep -i "request\|response"
# Expected: Recent request/response log lines

# 6. Check monitoring for recovery
# Verify in Grafana/Prometheus:
#   - Pod restart count is stable (not increasing)
#   - Request latency is normal
#   - Error rate has returned to baseline
```

**Verification checklist:**

- [ ] Pod status is `Running` and stable for at least 5 minutes
- [ ] Restart count is not increasing
- [ ] Application health endpoint returns 200
- [ ] Service is processing traffic normally
- [ ] No new `BackOff` events in the last 5 minutes
- [ ] Monitoring dashboards show normal metrics

---

## 8. Post-Resolution

### Immediate Actions

1. **Update the incident ticket** with the root cause and resolution applied.
2. **Notify stakeholders** that the service has been restored.
3. **Document the specific error message** that caused the crash for future reference.

### Follow-Up Actions

1. **Add startup logging**: If the root cause was hard to diagnose, add more
   detailed logging to the application startup sequence.

2. **Improve health checks**: If the liveness probe was too aggressive, tune
   the `initialDelaySeconds`, `periodSeconds`, and `failureThreshold`.

3. **Add resource monitoring**: Set up alerts for approaching memory limits
   (e.g., alert at 80% of memory limit).

4. **Create a pre-deployment checklist**: If the crash was caused by a
   deployment, add validation steps to your CI/CD pipeline:
   - Validate all referenced ConfigMaps and Secrets exist
   - Run smoke tests against a staging environment
   - Check resource limits are appropriate

5. **Update this runbook**: If you discovered a new root cause pattern, add
   it to the Diagnosis section.

### Metrics to Track

| Metric | Where to Find | Why It Matters |
|--------|---------------|----------------|
| Time to detect | Alert timestamp | Measures monitoring effectiveness |
| Time to diagnose | First action to root cause identified | Measures runbook clarity |
| Time to resolve | Root cause identified to service restored | Measures resolution options |
| Recurrence rate | Number of CrashLoopBackOff alerts per week | Measures root cause fix quality |

---

## 9. Escalation

If you cannot resolve the issue within 30 minutes:

| Time Elapsed | Action |
|-------------|--------|
| 15 min | Page the application team on-call |
| 30 min | Page the platform engineering lead |
| 60 min | Declare a major incident, engage incident commander |
| 90 min | Engage engineering management |

---

## 10. Related Runbooks

- [RB-K8S-002: Pod OOMKilled](runbook-002-oomkilled.md)
- [RB-K8S-003: ImagePullBackOff](runbook-003-imagepullbackoff.md)
- [RB-K8S-004: Pod Pending - Insufficient Resources](runbook-004-pod-pending.md)
- [RB-K8S-005: Deployment Rollback](runbook-005-deployment-rollback.md)
```

## Why This Works

1. **Structured diagnosis path**: The runbook starts with the cheapest
   information-gathering steps (describe, logs) before moving to more
   expensive ones (checking dependencies, node status). This mirrors how
   an experienced engineer would investigate.

2. **Decision tree included**: Rather than just listing commands, the runbook
   provides a decision tree that maps exit codes to root cause categories.
   This prevents the common mistake of trying random fixes without diagnosis.

3. **Multiple resolution options**: Real incidents are not one-size-fits-all.
   The three resolution options (fix root cause, rollback, reset backoff)
   cover the spectrum from ideal to temporary measures.

4. **Verification is explicit**: Many runbooks skip verification. This runbook
   includes specific commands and expected outputs, preventing premature
   incident closure.

5. **Post-resolution prevents recurrence**: The follow-up actions address
   the systemic issues that allowed the crash to happen, not just the
   immediate symptom.

## Common Mistakes

1. **Skipping `--previous` on logs**: The most common mistake. When a pod is
   in CrashLoopBackOff, the *current* container is waiting to restart. The
   interesting logs are in the *previous* container. Always use `--previous`.

2. **Deleting the pod without understanding why**: Running `kubectl delete pod`
   resets the backoff timer but does nothing about the root cause. The pod
   will crash again. Only use this as a temporary measure while investigating.

3. **Ignoring exit codes**: Exit code 0 means the application thinks it shut
   down cleanly (check liveness probe). Exit code 137 means OOMKilled. Exit
   code 1 means application error. These point to completely different
   investigation paths.

4. **Not checking all replicas**: Fixing one pod while others are also
   crashing means the deployment is broken. Always check the deployment-level
   status, not just individual pods.

5. **Missing the health check trap**: A pod that starts successfully but fails
   its liveness probe will be killed by Kubernetes, appearing as
   CrashLoopBackOff even though the application code is fine. Always check
   probe configuration and timing.

6. **Closing the incident without verification**: "The pod is running" is not
   enough. Verify it stays running for at least 5 minutes and is actually
   serving traffic.
