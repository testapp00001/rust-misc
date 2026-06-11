# Solution 02: Deploy a PostgreSQL StatefulSet with Persistent Storage

## Complete Solution

### headless-service.yaml

```yaml
apiVersion: v1
kind: Service
metadata:
  name: postgres
  namespace: postgres-demo
spec:
  clusterIP: None
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
    name: postgres
```

### postgres-statefulset.yaml

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: postgres
  namespace: postgres-demo
spec:
  serviceName: postgres
  replicas: 3
  selector:
    matchLabels:
      app: postgres
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
          name: postgres
        env:
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: password
        - name: PGDATA
          value: /var/lib/postgresql/data/pgdata
        volumeMounts:
        - name: data
          mountPath: /var/lib/postgresql/data
        readinessProbe:
          exec:
            command:
            - pg_isready
            - -U
            - postgres
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
        livenessProbe:
          exec:
            command:
            - pg_isready
            - -U
            - postgres
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
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

### read-service.yaml

```yaml
apiVersion: v1
kind: Service
metadata:
  name: postgres-read
  namespace: postgres-demo
spec:
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
    name: postgres
```

---

## Why This Solution Works

### 1. Headless Service Enables Stable DNS

The headless Service (`clusterIP: None`) is the foundation of StatefulSet networking. It creates DNS A records for each pod:

```
postgres-0.postgres.postgres-demo.svc.cluster.local → IP of postgres-0
postgres-1.postgres.postgres-demo.svc.cluster.local → IP of postgres-1
postgres-2.postgres.postgres-demo.svc.cluster.local → IP of postgres-2
```

The Service name `postgres` becomes the subdomain. The StatefulSet's `serviceName: postgres` tells Kubernetes to use this Service for DNS registration. These two names MUST match.

### 2. volumeClaimTemplates Creates Per-Pod Storage

The `volumeClaimTemplates` section creates a separate PVC for each pod:

```
Pod postgres-0 → PVC data-postgres-0 → PV (dedicated storage)
Pod postgres-1 → PVC data-postgres-1 → PV (dedicated storage)
Pod postgres-2 → PVC data-postgres-2 → PV (dedicated storage)
```

Each PVC is independent. postgres-0's data is completely isolated from postgres-1's data. This is essential because each PostgreSQL instance has its own data directory.

### 3. PGDATA Prevents Initialization Conflicts

Setting `PGDATA` to a subdirectory (`/var/lib/postgresql/data/pgdata`) is critical. PostgreSQL refuses to initialize if the data directory is not empty. When a PVC is mounted at `/var/lib/postgresql/data`, Kubernetes may create lost+found or other files. By using a subdirectory, PostgreSQL always finds an empty directory for initialization.

### 4. Probes Detect PostgreSQL Health

**Readiness probe** (frequent, short timeout):
- Runs `pg_isready -U postgres` every 5 seconds
- Pod is removed from Service endpoints when not ready
- Clients stop sending traffic to unhealthy pods

**Liveness probe** (less frequent, longer timeout):
- Runs every 10 seconds with a 30-second initial delay
- Restarts the pod if PostgreSQL is unresponsive
- Longer initial delay because PostgreSQL takes time to start (especially with large databases)

### 5. Two Services for Different Access Patterns

- **`postgres` (headless)**: Used by StatefulSet for DNS. Not typically used by clients directly.
- **`postgres-read` (ClusterIP)**: Load-balances across all pods. Used by read-heavy workloads.

Clients can connect to the primary directly using `postgres-0.postgres.postgres-demo.svc.cluster.local` for writes, or use `postgres-read` for reads.

---

## Verification Steps

### 1. Check Pods Come Up in Order

```bash
kubectl get pods -n postgres-demo -w
```

Expected output (sequential):
```
NAME          READY   STATUS              RESTARTS   AGE
postgres-0    0/1     ContainerCreating   0          2s
postgres-0    1/1     Running             0          5s
postgres-1    0/1     ContainerCreating   0          0s
postgres-1    1/1     Running             0          4s
postgres-2    0/1     ContainerCreating   0          0s
postgres-2    1/1     Running             0          4s
```

### 2. Verify PVCs

```bash
kubectl get pvc -n postgres-demo
```

Expected:
```
NAME                 STATUS   VOLUME                                     CAPACITY   ACCESS MODES   STORAGECLASS   AGE
data-postgres-0      Bound    pvc-abc123                                 1Gi        RWO            standard       2m
data-postgres-1      Bound    pvc-def456                                 1Gi        RWO            standard       90s
data-postgres-2      Bound    pvc-ghi789                                 1Gi        RWO            standard       60s
```

### 3. Verify Hostnames

```bash
kubectl exec postgres-0 -n postgres-demo -- hostname
# Expected: postgres-0

kubectl exec postgres-1 -n postgres-demo -- hostname
# Expected: postgres-1
```

### 4. Verify DNS Resolution

```bash
kubectl exec postgres-0 -n postgres-demo -- \
  nslookup postgres-1.postgres.postgres-demo.svc.cluster.local
```

Expected: Resolves to postgres-1's ClusterIP.

### 5. Verify Data Persistence

```bash
# Create test data
kubectl exec postgres-0 -n postgres-demo -- \
  psql -U postgres -c "CREATE DATABASE testdb;"
kubectl exec postgres-0 -n postgres-demo -- \
  psql -U postgres -d testdb -c "CREATE TABLE messages (id serial PRIMARY KEY, content text);"
kubectl exec postgres-0 -n postgres-demo -- \
  psql -U postgres -d testdb -c "INSERT INTO messages (content) VALUES ('hello from postgres-0');"

# Delete the pod
kubectl delete pod postgres-0 -n postgres-demo

# Wait for it to come back
kubectl get pods -n postgres-demo -w
# Wait until postgres-0 is Running

# Verify data survived
kubectl exec postgres-0 -n postgres-demo -- \
  psql -U postgres -d testdb -c "SELECT * FROM messages;"
# Expected: hello from postgres-0
```

---

## Common Mistakes to Avoid

### 1. Missing PGDATA Subdirectory

**Mistake:** Not setting `PGDATA` or setting it to the mount path directly.

**Problem:** PostgreSQL fails to initialize because the mount directory is not empty (contains `lost+found`).

**Solution:** Always set `PGDATA` to a subdirectory:
```yaml
- name: PGDATA
  value: /var/lib/postgresql/data/pgdata
```

### 2. serviceName Does Not Match Headless Service

**Mistake:** Setting `serviceName: my-postgres` when the headless Service is named `postgres`.

**Problem:** Kubernetes cannot create per-pod DNS entries. Pods will not get stable DNS names.

**Solution:** These two names MUST be identical:
```yaml
# In StatefulSet
spec:
  serviceName: postgres    # ← must match

# In Service
metadata:
  name: postgres           # ← must match
```

### 3. Using a ClusterIP Service Instead of Headless

**Mistake:** Creating a Service without `clusterIP: None`.

**Problem:** DNS returns a single VIP, not per-pod entries. The StatefulSet's stable identity guarantee is broken.

**Solution:** Always use `clusterIP: None` for the Service referenced by `serviceName`.

### 4. No Resource Limits

**Mistake:** Omitting resource requests and limits.

**Problem:** PostgreSQL can consume all available memory, causing OOM kills of other pods. Without requests, the scheduler cannot make informed placement decisions.

**Solution:** Always set both requests and limits. PostgreSQL's `shared_buffers` and `work_mem` should be tuned to stay within the memory limit.

### 5. Readiness Probe Too Aggressive

**Mistake:** Setting `initialDelaySeconds: 0` on the readiness probe.

**Problem:** PostgreSQL takes several seconds to start. The probe marks the pod as not-ready immediately, preventing other pods from starting (in OrderedReady mode).

**Solution:** Set `initialDelaySeconds: 5` or higher for the readiness probe. For the liveness probe, use 30 seconds or more.
