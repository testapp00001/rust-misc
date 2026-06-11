# Exercise 02: Deploy a PostgreSQL StatefulSet with Persistent Storage

## Objective

Deploy a 3-node PostgreSQL StatefulSet with persistent storage, a headless Service for stable DNS, and a ClusterIP Service for client access. Verify that each pod gets its own PVC, pods have stable hostnames, and data survives pod restarts.

## Background

PostgreSQL is a classic stateful workload. A primary-replica setup requires:
- A stable hostname for the primary so replicas can connect
- Per-pod storage so each node has its own data directory
- Ordered startup so the primary is ready before replicas attempt to connect

This exercise walks you through deploying PostgreSQL as a StatefulSet step by step.

## Instructions

### Step 1: Create the Namespace and Secret

```bash
kubectl create namespace postgres-demo

kubectl create secret generic postgres-secret \
  --from-literal=password=exercise-password-123 \
  -n postgres-demo
```

### Step 2: Create the Headless Service

Create a headless Service named `postgres` in the `postgres-demo` namespace. This Service must:
- Have `clusterIP: None`
- Select pods with label `app: postgres`
- Expose port 5432

### Step 3: Create the StatefulSet

Create a StatefulSet named `postgres` in the `postgres-demo` namespace with:
- `serviceName: postgres` (must match the headless Service name)
- `replicas: 3`
- Selector matching `app: postgres`
- Container using image `postgres:16`
- Environment variable `POSTGRES_PASSWORD` from the secret
- Environment variable `PGDATA` set to `/var/lib/postgresql/data/pgdata`
- A volumeMount named `data` at `/var/lib/postgresql/data`
- A `volumeClaimTemplates` section requesting 1Gi of storage with `ReadWriteOnce` access
- A readinessProbe using `pg_isready -U postgres`
- Resource requests: 100m CPU, 256Mi memory
- Resource limits: 500m CPU, 512Mi memory

### Step 4: Create a Client Service (Optional Read Service)

Create a second Service named `postgres-read` that is NOT headless (has a ClusterIP). This Service should:
- Select pods with label `app: postgres`
- Expose port 5432
- Allow clients to connect to any available pod for reads

### Step 5: Verify the Deployment

After applying all resources, verify:
1. All 3 pods are running in order (postgres-0, then postgres-1, then postgres-2)
2. Each pod has its own PVC (data-postgres-0, data-postgres-1, data-postgres-2)
3. Each pod has a stable hostname matching its pod name
4. DNS resolution works for individual pods
5. Data written to postgres-0 survives a pod restart

## Deliverables

Create the following YAML manifests (can be in a single file or separate files):
1. `headless-service.yaml` - The headless Service
2. `postgres-statefulset.yaml` - The StatefulSet
3. `read-service.yaml` - The ClusterIP Service for reads (optional)

## Success Criteria

- [ ] StatefulSet is created with 3 replicas
- [ ] Pods come up in order: postgres-0, postgres-1, postgres-2
- [ ] Each pod has its own PVC (verify with `kubectl get pvc -n postgres-demo`)
- [ ] `kubectl exec postgres-0 -n postgres-demo -- hostname` returns `postgres-0`
- [ ] DNS resolution works: `nslookup postgres-0.postgres.postgres-demo.svc.cluster.local`
- [ ] Data written to postgres-0 survives pod deletion and recreation

## Hints

<details>
<summary>Hint 1: Headless Service Structure</summary>

A headless Service is just a regular Service with `clusterIP: None`:

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
```

The name of this Service must match the `serviceName` in the StatefulSet spec.
</details>

<details>
<summary>Hint 2: volumeClaimTemplates Syntax</summary>

The `volumeClaimTemplates` section is at the same level as `serviceName` and `replicas`, NOT inside the pod template:

```yaml
spec:
  serviceName: postgres
  replicas: 3
  # ... pod template ...
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 1Gi
```

This creates PVCs named `data-postgres-0`, `data-postgres-1`, etc.
</details>

<details>
<summary>Hint 3: PGDATA Environment Variable</summary>

The `PGDATA` variable tells PostgreSQL where to store its data files. Setting it to a subdirectory of the mount point prevents initialization errors:

```yaml
env:
- name: PGDATA
  value: /var/lib/postgresql/data/pgdata
```

Without this, PostgreSQL may fail to initialize because the mounted directory is not empty.
</details>

<details>
<summary>Hint 4: Verifying Data Persistence</summary>

To test data persistence:

```bash
# Write data to postgres-0
kubectl exec postgres-0 -n postgres-demo -- \
  psql -U postgres -c "CREATE DATABASE testdb;"
kubectl exec postgres-0 -n postgres-demo -- \
  psql -U postgres -d testdb -c "CREATE TABLE messages (id serial PRIMARY KEY, content text);"
kubectl exec postgres-0 -n postgres-demo -- \
  psql -U postgres -d testdb -c "INSERT INTO messages (content) VALUES ('hello from postgres-0');"

# Delete the pod
kubectl delete pod postgres-0 -n postgres-demo

# Wait for it to come back, then verify
kubectl exec postgres-0 -n postgres-demo -- \
  psql -U postgres -d testdb -c "SELECT * FROM messages;"
```

The data should still be there because the PVC was not deleted.
</details>

<details>
<summary>Hint 5: Checking DNS Resolution</summary>

To verify DNS works for individual pods:

```bash
# From inside a pod in the same namespace
kubectl exec postgres-0 -n postgres-demo -- \
  nslookup postgres-1.postgres.postgres-demo.svc.cluster.local

# Or use a temporary pod
kubectl run dns-test --image=busybox -n postgres-demo --rm -it -- \
  nslookup postgres-0.postgres.postgres-demo.svc.cluster.local
```

Each pod should resolve to its own IP address.
</details>

## Common Issues

1. **Pods stuck in Init/CrashLoop**: Check that the secret exists and the password is set correctly
2. **PVCs stuck in Pending**: Verify a StorageClass exists (`kubectl get storageclass`)
3. **DNS resolution fails**: Ensure the headless Service name matches the StatefulSet's `serviceName`
4. **Data not persisting**: Verify PGDATA is set to a subdirectory of the volume mount
