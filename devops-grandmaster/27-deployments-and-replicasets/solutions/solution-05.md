# Solution 05: Deployment Pipeline with Health Checks

## Part A: Production-Grade Deployment Manifest

`production-app.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: prod-api
  labels:
    app: prod-api
    tier: backend
    env: production
spec:
  replicas: 4
  selector:
    matchLabels:
      app: prod-api
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  minReadySeconds: 10
  progressDeadlineSeconds: 300
  revisionHistoryLimit: 5
  template:
    metadata:
      labels:
        app: prod-api
        tier: backend
        env: production
    spec:
      containers:
      - name: api
        image: nginx:1.25
        ports:
        - containerPort: 8080
        resources:
          requests:
            cpu: 250m
            memory: 128Mi
          limits:
            cpu: 500m
            memory: 256Mi
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
          failureThreshold: 3
        livenessProbe:
          httpGet:
            path: /healthz
            port: 8080
          initialDelaySeconds: 15
          periodSeconds: 10
          failureThreshold: 3
```

**Why it works:**
- `maxSurge: 1` and `maxUnavailable: 0` ensure zero downtime: one new Pod is created and confirmed ready before any old Pod is removed.
- `minReadySeconds: 10` adds an extra safety buffer: a Pod must stay ready for 10 seconds after the readiness probe passes before it counts as available. This catches Pods that pass the readiness probe but crash shortly after (e.g., during cache warmup).
- `progressDeadlineSeconds: 300` tells Kubernetes to mark the deployment as stuck if no progress is made within 5 minutes. This is critical for detecting failed deployments (bad images, resource exhaustion) and triggering alerts.
- `revisionHistoryLimit: 5` keeps 5 old ReplicaSets for rollback while preventing unbounded growth of ReplicaSet objects.
- Readiness probe (`/ready`) starts earlier (5s delay) than liveness probe (`/healthz`, 15s delay) to avoid killing Pods during startup.

**Common mistakes:**
- Setting liveness probe `initialDelaySeconds` too low. If the liveness probe fires before the app finishes initializing, Kubernetes restarts the Pod, creating a crash loop.
- Omitting `minReadySeconds`. Without it, a Pod that passes readiness but crashes 2 seconds later is counted as "available" during those 2 seconds, potentially masking issues.
- Setting `progressDeadlineSeconds` too low. If your Pods take a long time to start (e.g., large images, slow init), a low deadline causes false failure reports.

---

## Part B: Deployment Verification Script

`verify-deployment.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

DEPLOYMENT_NAME="${1:-prod-api}"
NAMESPACE="${2:-default}"
TIMEOUT="${3:-300s}"

echo "=== Deployment Verification: ${DEPLOYMENT_NAME} ==="

# Step 1: Wait for rollout to complete
echo "[1/5] Waiting for rollout to complete (timeout: ${TIMEOUT})..."
if ! kubectl rollout status deployment "${DEPLOYMENT_NAME}" \
    -n "${NAMESPACE}" --timeout="${TIMEOUT}"; then
    echo "FAIL: Rollout did not complete within ${TIMEOUT}"
    exit 1
fi
echo "PASS: Rollout completed"

# Step 2: Verify all Pods are in Running state
echo "[2/5] Verifying all Pods are Running..."
NON_RUNNING=$(kubectl get pods -l "app=${DEPLOYMENT_NAME}" \
    -n "${NAMESPACE}" \
    -o jsonpath='{range .items[?(@.status.phase!="Running")]}{.metadata.name}{"\t"}{.status.phase}{"\n"}{end}')

if [ -n "${NON_RUNNING}" ]; then
    echo "FAIL: Some Pods are not Running:"
    echo "${NON_RUNNING}"
    exit 1
fi
echo "PASS: All Pods are Running"

# Step 3: Verify all Pods are Ready
echo "[3/5] Verifying all Pods are Ready..."
DESIRED=$(kubectl get deployment "${DEPLOYMENT_NAME}" \
    -n "${NAMESPACE}" \
    -o jsonpath='{.spec.replicas}')
READY=$(kubectl get deployment "${DEPLOYMENT_NAME}" \
    -n "${NAMESPACE}" \
    -o jsonpath='{.status.readyReplicas}')

if [ "${READY}" != "${DESIRED}" ]; then
    echo "FAIL: Ready replicas (${READY:-0}) != Desired (${DESIRED})"
    exit 1
fi
echo "PASS: ${READY}/${DESIRED} Pods are Ready"

# Step 4: Verify Available condition
echo "[4/5] Verifying Available condition..."
AVAILABLE_CONDITION=$(kubectl get deployment "${DEPLOYMENT_NAME}" \
    -n "${NAMESPACE}" \
    -o jsonpath='{range .status.conditions[?(@.type=="Available")]}{.status}{end}')

if [ "${AVAILABLE_CONDITION}" != "True" ]; then
    echo "FAIL: Available condition is ${AVAILABLE_CONDITION:-unset}, expected True"
    exit 1
fi
echo "PASS: Available condition is True"

# Step 5: Verify available replicas count
echo "[5/5] Verifying available replica count..."
AVAILABLE_REPLICAS=$(kubectl get deployment "${DEPLOYMENT_NAME}" \
    -n "${NAMESPACE}" \
    -o jsonpath='{.status.availableReplicas}')

if [ "${AVAILABLE_REPLICAS}" != "${DESIRED}" ]; then
    echo "FAIL: Available replicas (${AVAILABLE_REPLICAS:-0}) != Desired (${DESIRED})"
    exit 1
fi
echo "PASS: ${AVAILABLE_REPLICAS}/${DESIRED} replicas available"

# Summary
echo ""
echo "=== Verification Summary ==="
echo "Deployment:  ${DEPLOYMENT_NAME}"
echo "Namespace:   ${NAMESPACE}"
echo "Replicas:    ${AVAILABLE_REPLICAS}/${DESIRED}"
echo "Status:      HEALTHY"
echo "All checks passed."
exit 0
```

**Why it works:**
- `set -euo pipefail` ensures the script exits on any error, undefined variable, or pipe failure.
- Each check is independent and provides a clear failure message.
- `kubectl rollout status --timeout` blocks until the rollout completes or times out, providing the first gate.
- `readyReplicas` vs `replicas` catches Pods that are Running but not Ready (failing readiness probe).
- The `Available` condition is the Kubernetes-native signal that Pods have been ready for at least `minReadySeconds`.
- `availableReplicas` is the final check: Pods must be Running, Ready, and have passed `minReadySeconds`.

**Common mistakes:**
- Checking only `status.phase == Running` and assuming the Pod is healthy. A Pod can be Running but not Ready (readiness probe failing).
- Not handling the case where `readyReplicas` is unset (null). When no Pods are ready, the field may be absent from the JSON. Default to "0" in comparisons.
- Forgetting `--timeout` on `kubectl rollout status`. Without it, the command blocks indefinitely if the rollout is stuck.

---

## Part C: Simulate a Deployment Pipeline

### Step 1: Initial deployment verification

```bash
kubectl rollout status deployment prod-api --timeout=60s
kubectl get deployment prod-api
kubectl get pods -l app=prod-api
```

Expected: All 4 Pods running, 4/4 ready, deployment Available.

### Step 2: Update the application

```bash
kubectl set image deployment prod-api api=nginx:1.26
kubectl rollout status deployment prod-api --timeout=120s
```

Expected: Rolling update completes. With `maxSurge: 1` and `maxUnavailable: 0`, the maximum Pod count is 5 (4 desired + 1 surge). At no point are fewer than 4 Pods available.

### Step 3: Post-deployment verification

```bash
kubectl get pods -l app=prod-api -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.spec.containers[0].image}{"\t"}{.status.phase}{"\n"}{end}'
kubectl get deployment prod-api -o jsonpath='{.status.conditions[*].type}{"\n"}'
```

Expected: All Pods show `nginx:1.26`, `Running`. Conditions include `Available` and `Progressing`.

### Step 4: Simulate a failure

```bash
kubectl set image deployment prod-api api=nginx:bad-tag-does-not-exist
kubectl rollout status deployment prod-api --timeout=60s
# This will fail -- expected
kubectl describe deployment prod-api | grep -A 10 "Conditions"
```

**1. What does `progressDeadlineSeconds: 300` do when a deployment is stuck?**

After 300 seconds (5 minutes) with no progress, Kubernetes sets the deployment condition to `Progressing: False` with reason `ProgressDeadlineExceeded`. This signals that the deployment is stuck but does NOT automatically roll back. You must take manual action (rollback) or have a controller that watches for this condition. The old Pods continue running and serving traffic during this entire period.

**2. After the bad update, is the old version still serving traffic? How can you verify?**

Yes. The old Pods running `nginx:1.26` continue to serve traffic because `maxUnavailable: 0` prevents their removal until new Pods are ready. Since the new Pods (with the bad image) will never become ready, the old Pods are never terminated. Verify with:

```bash
kubectl get pods -l app=prod-api -o wide
# Check that READY pods still show nginx:1.26
```

**3. What is the fastest way to recover?**

```bash
kubectl rollout undo deployment prod-api
```

This immediately scales down the bad ReplicaSet and scales up the previous good ReplicaSet. Since the previous ReplicaSet already exists, this is fast -- no image pull or Pod startup delay beyond what is needed for the existing good Pods.

### Step 5: Rollback and verify recovery

```bash
kubectl rollout undo deployment prod-api
kubectl rollout status deployment prod-api --timeout=60s
kubectl get deployment prod-api -o jsonpath='{.spec.template.spec.containers[0].image}'
```

Expected: Image reverts to `nginx:1.26`, rollout completes, all Pods healthy.

**Common mistakes:**
- Expecting `progressDeadlineSeconds` to trigger automatic rollback. It does not -- it only sets a condition. You need a human or an external controller to act on it.
- Trying to `kubectl apply` the original YAML to fix a bad `set image` update. While this works, `kubectl rollout undo` is faster and purpose-built for this scenario.

---

## Part D: Deployment Observability

**1. How do you see all events related to the `prod-api` Deployment?**

```bash
kubectl get events --field-selector involvedObject.name=prod-api
```

Or for a wider view including ReplicaSet and Pod events:

```bash
kubectl get events --field-selector involvedObject.name=prod-api --sort-by='.lastTimestamp'
```

For all events in the namespace related to the deployment (including child resources):

```bash
kubectl get events --sort-by='.lastTimestamp' | grep prod-api
```

**2. How do you see which ReplicaSet is currently active?**

```bash
kubectl get replicasets -l app=prod-api
```

The active ReplicaSet is the one with non-zero replicas. Alternatively:

```bash
kubectl get deployment prod-api -o jsonpath='{.status.conditions[?(@.type=="Progressing")]}'
```

Or to see the current revision's ReplicaSet directly:

```bash
kubectl get deployment prod-api -o jsonpath='{.status.conditions[?(@.type=="Available")].message}'
```

**3. How do you see the full rollout history?**

```bash
kubectl rollout history deployment prod-api
```

**4. How do you see the details of revision 2?**

```bash
kubectl rollout history deployment prod-api --revision=2
```

**5. How do you see which node each Pod is running on?**

```bash
kubectl get pods -l app=prod-api -o wide
```

The `NODE` column shows which node each Pod is scheduled on. Alternatively:

```bash
kubectl get pods -l app=prod-api -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.spec.nodeName}{"\n"}{end}'
```

**Common mistakes:**
- Using `kubectl describe deployment` for events. This shows events for the Deployment object itself, not for the Pods. Use `kubectl get events` with `field-selector` for comprehensive event filtering.
- Not using `-o wide` when you need node information. The default `kubectl get pods` output does not include the node column.

---

## Part E: Production Checklist

| Checklist Item | Why It Matters | What Happens If Skipped |
|----------------|----------------|-------------------------|
| Resource requests and limits set | Requests ensure Pods get the CPU and memory they need (scheduler uses requests for placement). Limits prevent a single Pod from consuming all resources on a node, protecting other workloads. | Without requests, Pods may be scheduled on nodes with insufficient resources, causing OOM kills or CPU throttling. Without limits, a single misbehaving Pod can starve the entire node, affecting all other Pods. |
| Readiness probe configured | The readiness probe tells Kubernetes when a Pod is ready to accept traffic. Traffic is only sent to Pods that pass the probe. This prevents sending requests to Pods that are still starting up or have entered a degraded state. | Without a readiness probe, Pods receive traffic as soon as the container starts (before the application is initialized). This causes errors for users during startup and prevents graceful degradation when a Pod becomes unhealthy. |
| Liveness probe configured | The liveness probe detects when a Pod is stuck (deadlocked, hung) and needs to be restarted. Kubernetes restarts Pods that fail the liveness probe, enabling self-healing. | Without a liveness probe, stuck Pods are never restarted. They consume resources but do not serve traffic, reducing capacity. Users experience timeouts or errors for requests routed to the stuck Pod. |
| maxUnavailable: 0 for critical services | Setting `maxUnavailable: 0` ensures the available Pod count never drops below the desired replica count during updates. This guarantees zero downtime for critical services. | With `maxUnavailable: 1` (or higher), one or more Pods may be unavailable during updates. For critical services, this means some requests will fail or be delayed during deployments. |
| minReadySeconds > 0 | `minReadySeconds` adds a buffer after a Pod passes its readiness probe before it counts as available. This catches Pods that pass the readiness check but crash shortly after (e.g., during cache warmup, connection pooling). | Without `minReadySeconds`, a Pod that passes readiness but crashes 1 second later is briefly counted as available. This can mask startup issues and cause the Deployment controller to proceed with removing old Pods prematurely. |
| progressDeadlineSeconds set | This tells Kubernetes to report when a deployment is stuck (not making progress). It enables monitoring and alerting on failed deployments. Without it, a failed deployment can sit indefinitely without any signal. | Without `progressDeadlineSeconds`, a failed deployment (bad image, resource exhaustion) is never flagged. The old Pods keep running, but no one is alerted that the update is stuck. Recovery depends on someone manually noticing the issue. |
| revisionHistoryLimit set | This controls how many old ReplicaSets are kept for rollback. It prevents unbounded growth of ReplicaSet objects in the cluster. | Without `revisionHistoryLimit`, every update creates a new ReplicaSet and none are cleaned up. Over time, this accumulates hundreds of stale ReplicaSets, wasting API server storage and cluttering `kubectl get replicasets` output. |

**Common mistakes:**
- Setting `minReadySeconds` too high (e.g., 60s). This makes rollouts very slow because each Pod must wait the full duration before the next one is created.
- Setting `progressDeadlineSeconds` too low. If your Pods take 2 minutes to start and the deadline is 60s, every deployment reports failure even when it succeeds eventually.
- Setting `revisionHistoryLimit` to 0. This means no old ReplicaSets are kept, so `kubectl rollout undo` has nothing to roll back to.

---

## Part F: Clean Up

```bash
kubectl delete deployment prod-api
```

This cascades and deletes all ReplicaSets and Pods owned by the Deployment. The Service (if any) is not affected since it is a separate object.
