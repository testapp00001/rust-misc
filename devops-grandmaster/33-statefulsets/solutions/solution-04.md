# Solution 04: Handle StatefulSet Scaling and Rolling Updates Safely

## Complete Solution

### db-headless-service.yaml

```yaml
apiVersion: v1
kind: Service
metadata:
  name: db
  namespace: update-demo
spec:
  clusterIP: None
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
    name: postgres
```

### db-statefulset.yaml

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: db
  namespace: update-demo
spec:
  serviceName: db
  replicas: 3
  selector:
    matchLabels:
      app: postgres
  updateStrategy:
    type: RollingUpdate
    rollingUpdate:
      partition: 0
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:16
        ports:
        - containerPort: 5432
        env:
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: pg-secret
              key: password
        - name: PGDATA
          value: /var/lib/postgresql/data/pgdata
        volumeMounts:
        - name: data
          mountPath: /var/lib/postgresql/data
        readinessProbe:
          exec:
            command: ["pg_isready", "-U", "postgres"]
          initialDelaySeconds: 5
          periodSeconds: 5
        livenessProbe:
          exec:
            command: ["pg_isready", "-U", "postgres"]
          initialDelaySeconds: 30
          periodSeconds: 10
        resources:
          requests:
            cpu: 100m
            memory: 256Mi
          limits:
            cpu: 500m
            memory: 512Mi
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 1Gi
```

### cache-statefulset.yaml (Parallel Pod Management)

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: cache
  namespace: update-demo
spec:
  serviceName: cache
  replicas: 3
  podManagementPolicy: Parallel
  selector:
    matchLabels:
      app: redis
  template:
    metadata:
      labels:
        app: redis
    spec:
      containers:
      - name: redis
        image: redis:7
        ports:
        - containerPort: 6379
        readinessProbe:
          exec:
            command: ["redis-cli", "ping"]
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          requests:
            cpu: 100m
            memory: 128Mi
          limits:
            cpu: 250m
            memory: 256Mi
---
apiVersion: v1
kind: Service
metadata:
  name: cache
  namespace: update-demo
spec:
  clusterIP: None
  selector:
    app: redis
  ports:
  - port: 6379
    targetPort: 6379
```

---

## Step-by-Step Walkthrough

### Step 2: Standard Rolling Update

```bash
# Update the image
kubectl set image statefulset/db postgres=postgres:17 -n update-demo

# Watch the update order
kubectl get pods -n update-demo -w
```

**Observed behavior:**
```
1. postgres-2 is deleted first (highest ordinal)
2. postgres-2 is recreated with postgres:17
3. postgres-2 must become Ready (readiness probe passes)
4. postgres-1 is deleted and recreated
5. postgres-1 must become Ready
6. postgres-0 is deleted and recreated (last, the primary)
7. postgres-0 must become Ready
```

**Why reverse order?** Updates go from highest ordinal to lowest. This means replicas update before the primary. This is deliberate: if the update introduces a bug, it affects replicas first. The primary (ordinal 0) is the most critical node and updates last.

### Step 3: Partitioned (Canary) Update

```bash
# Reset to postgres:16
kubectl set image statefulset/db postgres=postgres:16 -n update-demo
# Wait for all pods to be on postgres:16

# Step 1: Set partition to 2 (only update ordinal >= 2)
kubectl patch statefulset db -n update-demo \
  -p '{"spec":{"updateStrategy":{"rollingUpdate":{"partition":2}}}}'

# Step 2: Update image (only postgres-2 changes)
kubectl set image statefulset/db postgres=postgres:17 -n update-demo

# Step 3: Verify only postgres-2 changed
kubectl get pods -n update-demo -o custom-columns=\
  NAME:.metadata.name,IMAGE:.spec.containers[0].image
# postgres-0: postgres:16
# postgres-1: postgres:16
# postgres-2: postgres:17  ← only this changed

# Step 4: Verify postgres-2 is healthy
kubectl exec db-2 -n update-demo -- psql -U postgres -c "SELECT version();"

# Step 5: If healthy, roll out to all
kubectl patch statefulset db -n update-demo \
  -p '{"spec":{"updateStrategy":{"rollingUpdate":{"partition":0}}}}'

# Watch postgres-1 and postgres-0 update
kubectl get pods -n update-demo -w
```

**Why this works:**
- `partition: 2` tells Kubernetes: "Only update pods with ordinal >= 2"
- With 3 replicas (0, 1, 2), only pod 2 is eligible
- Setting `partition: 0` makes all pods eligible again
- The remaining pods update in reverse order (1, then 0)

### Step 5: Rollback

```bash
# Check rollout history
kubectl rollout history statefulset/db -n update-demo

# Roll back
kubectl rollout undo statefulset/db -n update-demo

# Verify all pods are back on postgres:16
kubectl get pods -n update-demo -o custom-columns=\
  NAME:.metadata.name,IMAGE:.spec.containers[0].image
```

---

## Why This Solution Works

### 1. Partition Enables Canary Deployments

The `partition` field is the key to safe database updates. Instead of updating all pods at once, you:

1. Update only the least critical pod (highest ordinal, typically a replica)
2. Monitor the canary for errors, performance degradation, or data corruption
3. If the canary is healthy, roll out to the rest
4. If the canary is bad, roll back just that one pod (set image back and reset partition)

This minimizes blast radius. At most one replica is affected during the canary phase.

### 2. Ordered Updates Protect the Primary

With `OrderedReady` pod management, updates proceed in reverse ordinal order:
```
postgres-2 → postgres-1 → postgres-0
```

This means:
- Replicas update first (less critical)
- The primary updates last (most critical)
- Each pod must be Ready before the next one updates
- If a pod fails to become Ready, the update stops (does not continue to the next pod)

### 3. Parallel Pod Management for Independent Workloads

The `cache` StatefulSet uses `podManagementPolicy: Parallel`:
- All 3 pods start simultaneously (no waiting)
- Useful for caches, independent workers, stateless services with local storage
- NOT suitable for databases where the primary must start first

### 4. Rollout History Enables Rollbacks

Kubernetes tracks each revision of the StatefulSet. `kubectl rollout undo` reverts to the previous revision. This works because:
- Each revision stores the complete pod template
- The rollback creates a new revision (does not delete history)
- PVCs are unaffected (data is preserved)

---

## Common Mistakes to Avoid

### 1. Not Monitoring the Canary

**Mistake:** Setting the partition and immediately setting it to 0 without verifying the canary pod.

**Problem:** If the new image has a bug, it rolls out to all pods before you notice.

**Solution:** Always verify the canary pod is healthy before proceeding:
```bash
# Check the canary's logs
kubectl logs db-2 -n update-demo

# Check the canary's readiness
kubectl get pod db-2 -n update-demo -o jsonpath='{.status.conditions[?(@.type=="Ready")].status}'

# Run application-level health checks
kubectl exec db-2 -n update-demo -- psql -U postgres -c "SELECT 1;"
```

### 2. Using OnDelete Instead of RollingUpdate

**Mistake:** Setting `updateStrategy.type: OnDelete`.

**Problem:** Pods are NOT updated automatically. You must manually delete each pod to trigger the update. This is error-prone and defeats the purpose of declarative configuration.

**Solution:** Use `type: RollingUpdate` for automated, ordered updates.

### 3. Partition Value Out of Range

**Mistake:** Setting `partition: 5` when you only have 3 replicas (ordinals 0, 1, 2).

**Problem:** No pods are eligible for update (no pod has ordinal >= 5). The update appears to do nothing.

**Solution:** Set partition to a value between 0 and `replicas - 1`.

### 4. Rolling Back Without Checking History

**Mistake:** Running `kubectl rollout undo` without checking what revision you are rolling back to.

**Problem:** You might roll back to a version that also has issues.

**Solution:** Always check history first:
```bash
kubectl rollout history statefulset/db -n update-demo
kubectl rollout history statefulset/db -n update-demo --revision=2
```

### 5. Forgetting to Reset Partition After Canary

**Mistake:** Setting partition to 2 for canary, then updating the image, but forgetting to set partition back to 0.

**Problem:** Only pod 2 is on the new version. Future scaling or updates are inconsistent.

**Solution:** Always reset partition to 0 after verifying the canary:
```bash
kubectl patch statefulset db -n update-demo \
  -p '{"spec":{"updateStrategy":{"rollingUpdate":{"partition":0}}}}'
```
