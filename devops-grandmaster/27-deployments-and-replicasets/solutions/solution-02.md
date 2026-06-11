# Solution 02: Create a Deployment with Rolling Update

## Part A: Create the Deployment

`web-deployment.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-server
spec:
  replicas: 3
  selector:
    matchLabels:
      app: web-server
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  template:
    metadata:
      labels:
        app: web-server
    spec:
      containers:
      - name: nginx
        image: nginx:1.25
        ports:
        - containerPort: 80
        resources:
          requests:
            cpu: 100m
            memory: 64Mi
          limits:
            cpu: 200m
            memory: 128Mi
        readinessProbe:
          httpGet:
            path: /
            port: 80
          initialDelaySeconds: 5
          periodSeconds: 3
        livenessProbe:
          httpGet:
            path: /
            port: 80
          initialDelaySeconds: 10
          periodSeconds: 5
```

**Why it works:**
- `selector.matchLabels` must match `template.metadata.labels` -- this is how the Deployment finds its Pods.
- `maxSurge: 1` allows 1 extra Pod above the desired count during updates (so up to 4 Pods total with 3 replicas).
- `maxUnavailable: 0` ensures no Pod is removed until a replacement is ready, maintaining full capacity.
- The readiness probe starts checking at 5 seconds (before liveness at 10 seconds) so Kubernetes knows when the Pod is ready to receive traffic before it starts checking if the Pod is alive.

**Common mistakes:**
- Forgetting `selector.matchLabels` or making it not match `template.metadata.labels`. This causes the Deployment to fail with a selector error.
- Setting `maxUnavailable: 1` instead of `0` when the requirement says "never drop below full capacity." With 3 replicas and `maxUnavailable: 1`, one Pod could be unavailable during the update.
- Putting `initialDelaySeconds` for liveness too low. If the liveness probe fires before the app is ready, Kubernetes restarts the Pod, creating a crash loop.

---

## Part B: Verify the Deployment

**1. How many ReplicaSets exist? What is the name pattern?**

1 ReplicaSet exists. The name follows the pattern `web-server-<pod-template-hash>`, for example `web-server-5d69b9ff97`. The hash is derived from the pod template specification and is deterministic -- the same template always produces the same hash.

**2. How many Pods are running? What is the status of each?**

3 Pods are running. All 3 should show `Running` status with `READY` column showing `1/1` once the readiness probe passes.

**3. What labels do the Pods have? Where did those labels come from?**

Each Pod has the label `app: web-server` (from `template.metadata.labels`). Pods also get additional labels automatically: `pod-template-hash` (the hash from the ReplicaSet name). The `app: web-server` label comes from the Deployment's pod template and is what the Service would use to route traffic.

**Common mistakes:**
- Confusing the Deployment's label selector with the Pod's labels. The Deployment selector uses `matchLabels` to find Pods; the Pods get their labels from `template.metadata.labels`.
- Not waiting for the readiness probe to pass before checking Pod status. Pods may show `Running` but `0/1` Ready for a few seconds.

---

## Part C: Perform a Rolling Update

**1. How many ReplicaSets exist now? Which one has active Pods?**

2 ReplicaSets exist. The new ReplicaSet (with a different pod-template-hash) has the active Pods (all 3 running `nginx:1.26`). The old ReplicaSet has 0 replicas.

**2. During the rollout, what was the maximum number of Pods running at any point? Why?**

4 Pods. With `maxSurge: 1` and 3 replicas, Kubernetes is allowed to create 1 extra Pod beyond the desired count. The rolling update sequence is:
1. Create 1 new Pod (4 total: 3 old + 1 new)
2. Wait for new Pod to be ready
3. Delete 1 old Pod (3 total: 2 old + 1 new)
4. Create 1 new Pod (4 total: 2 old + 2 new)
5. Wait, delete 1 old (3 total: 1 old + 2 new)
6. Create 1 new (4 total: 1 old + 3 new)
7. Wait, delete 1 old (3 total: 0 old + 3 new)

**3. Were there any moments when zero Pods were unavailable? Why or why not?**

No. `maxUnavailable: 0` guarantees that Kubernetes never reduces the available Pod count below the desired replica count. At every step, at least 3 Pods are available to serve traffic. A new Pod is always created and confirmed ready before an old Pod is removed.

**Common mistakes:**
- Expecting only 3 Pods at all times. The `maxSurge: 1` parameter explicitly allows 1 extra Pod temporarily.
- Confusing "available" with "running." A Pod is "available" only after passing the readiness probe. A Pod can be "Running" but not yet "Ready."

---

## Part D: Verify the Final State

**1. What image is each Pod running?**

All 3 Pods run `nginx:1.26`. The old ReplicaSet has 0 replicas, so no Pods from the old version remain.

**2. What is the replica count of the old ReplicaSet? Why is it still around?**

The old ReplicaSet has 0 replicas. It is kept for rollback purposes. If you need to revert to `nginx:1.25`, Kubernetes can simply scale up this existing ReplicaSet instead of creating a new one. The old ReplicaSet will be garbage collected when `revisionHistoryLimit` is exceeded or when the Deployment is deleted.

**3. What does the `Conditions` section describe?**

The Conditions section shows:
- `Available: True` -- at least the minimum number of Pods are available (the `Available` condition).
- `Progressing: True` -- the deployment is making progress toward the desired state (the `Progressing` condition).
- These conditions are what `kubectl rollout status` checks to determine if a deployment is healthy.

**Common mistakes:**
- Panicking when seeing old ReplicaSets with 0 replicas. This is normal and expected behavior.
- Not understanding that `Available: True` requires more than just Pods running -- they must also pass readiness checks and (if configured) `minReadySeconds`.

---

## Part E: Clean Up

**Why are the ReplicaSets and Pods gone even though you only deleted the Deployment?**

Kubernetes uses cascading deletion via owner references. When you delete the Deployment, the garbage collector sees that the Deployment owns the ReplicaSets and deletes them. When the ReplicaSets are deleted, the garbage collector sees that the ReplicaSets own the Pods and deletes those too. This cascading chain (Deployment -> ReplicaSet -> Pod) is automatic and is the default foreground deletion policy.

**Common mistakes:**
- Trying to manually delete ReplicaSets after deleting a Deployment. This is unnecessary -- cascading deletion handles it.
- Thinking that orphaning (leaving children alive after deleting the parent) is the default behavior. It is not; cascading deletion is the default.
