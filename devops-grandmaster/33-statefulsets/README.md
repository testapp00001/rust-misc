# Module 33: StatefulSets — Databases and Stateful Services

## The Problem: Deployments Don't Work for Databases

Deployments are designed for stateless, interchangeable pods. Databases break this model.

```
Deployment behavior:
  pod-abc123  ← random name, no identity
  pod-def456  ← could be scaled down at any time, any order
  pod-ghi789  ← restarted in random order, gets new name

What databases need:
  db-0        ← stable name, always the primary
  db-1        ← replica, knows it's a replica
  db-2        ← replica, starts AFTER db-0 and db-1
```

### What Breaks with Deployments

```
Problem 1: Random Pod Names
  Deployment: myapp-7b4f8c6d9-x2k4n  ← clients can't find it
  Database needs: postgres-0           ← stable, predictable

Problem 2: All Pods Are Identical
  Deployment: scale up = create identical copy
  Database:   scale up = create REPLICA that syncs from PRIMARY

Problem 3: Random Scaling Order
  Deployment: scale down = kill random pod (could be the primary!)
  Database:   scale down = kill replica first, never the primary

Problem 4: Shared Storage
  Deployment: all pods can mount the same PVC
  Database:   each pod needs its OWN storage (primary has different data than replica)

Problem 5: No Ordered Startup
  Deployment: all pods start simultaneously
  Database:   primary must be ready BEFORE replicas try to connect
```

## The Naive Way: Sidecar Scripts and Workarounds

Before StatefulSets, people hacked around these problems:

```yaml
# Naive: Deployment with init containers and hostname hacks
apiVersion: apps/v1
kind: Deployment
metadata:
  name: postgres-hack
spec:
  replicas: 3
  template:
    spec:
      initContainers:
      - name: wait-for-primary
        image: busybox
        # Hope the primary is ready before we start...
        command: ["sh", "-c", "until nslookup postgres-0; do sleep 1; done"]
      containers:
      - name: postgres
        image: postgres:16
        env:
        - name: POD_NAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name    # Random name, useless for routing
```

**Why this fails:**
- Init containers can't guarantee ordering
- Pod names are random, so DNS-based discovery breaks
- No per-pod storage isolation
- Scaling down kills random pods (might kill the primary)
- Race conditions everywhere

## The Right Way: StatefulSet

A StatefulSet is a Kubernetes workload API designed specifically for stateful applications. It provides three guarantees that Deployments do not:

```
┌─────────────────────────────────────────────────────────┐
│                 StatefulSet Guarantees                    │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  1. STABLE NETWORK IDENTITY                               │
│     Pod name: {statefulset-name}-{ordinal}                │
│     DNS name: {pod-name}.{service-name}.{namespace}.svc   │
│     Example:  postgres-0.postgres.default.svc.cluster.local│
│                                                          │
│  2. ORDERED DEPLOYMENT AND SCALING                        │
│     Create:  postgres-0 → postgres-1 → postgres-2         │
│     Delete:  postgres-2 → postgres-1 → postgres-0         │
│     Each pod waits for the previous to be Ready            │
│                                                          │
│  3. STABLE, PERSISTENT STORAGE                            │
│     Each pod gets its own PVC                             │
│     PVC follows the pod (named by ordinal)                │
│     Deleting pod does NOT delete PVC                      │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### Basic StatefulSet Structure

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: postgres
spec:
  serviceName: postgres         # MUST match a headless service
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
        env:
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: password
        volumeMounts:
        - name: data
          mountPath: /var/lib/postgresql/data
  volumeClaimTemplates:         # Each pod gets its own PVC
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      storageClassName: fast-ssd
      resources:
        requests:
          storage: 20Gi
```

The `volumeClaimTemplates` section is the key difference from Deployments. It creates a **separate PVC for each pod**:

```
Pod postgres-0  →  PVC data-postgres-0  →  PV (dedicated storage)
Pod postgres-1  →  PVC data-postgres-1  →  PV (dedicated storage)
Pod postgres-2  →  PVC data-postgres-2  →  PV (dedicated storage)
```

### Headless Service: Stable DNS for StatefulSets

A StatefulSet **requires** a headless Service to provide stable DNS names.

```yaml
apiVersion: v1
kind: Service
metadata:
  name: postgres             # This name becomes the subdomain
spec:
  clusterIP: None            # HEADLESS — no virtual IP
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
```

With the headless service named `postgres` and the StatefulSet named `postgres`:

```
DNS Resolution:
  postgres-0.postgres.default.svc.cluster.local  → IP of pod postgres-0
  postgres-1.postgres.default.svc.cluster.local  → IP of pod postgres-1
  postgres-2.postgres.default.svc.cluster.local  → IP of pod postgres-2
  postgres.default.svc.cluster.local             → ALL pods (round-robin)

Pod Identity:
  Hostname inside pod: postgres-0
  Subdomain: postgres
  Full DNS: postgres-0.postgres.default.svc.cluster.local
```

Compare with a Deployment + ClusterIP Service:

```
Deployment DNS:
  myapp-7b4f8c6d9-x2k4n    ← random, changes on restart
  myapp.default.svc.cluster.local  ← no individual pod DNS

StatefulSet DNS:
  postgres-0                ← stable, survives restarts
  postgres-0.postgres.default.svc.cluster.local  ← individual pod DNS
```

### How Pod Restart Works in a StatefulSet

```
postgres-0 crashes:
  1. Kubernetes detects the failure
  2. New pod postgres-0 is created (SAME name)
  3. PVC data-postgres-0 is re-attached (SAME storage)
  4. DNS name postgres-0.postgres.default.svc.cluster.local still resolves
  5. Other pods are UNAFFECTED

With Deployment:
  1. pod-abc123 is killed
  2. New pod-xyz789 is created (DIFFERENT name, DIFFERENT storage)
  3. Clients trying to reach pod-abc123 fail
```

## The Production Way: StatefulSet Patterns

### Pattern 1: Primary-Replica Database (PostgreSQL)

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: postgres
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
        - name: POD_NAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
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
        livenessProbe:
          exec:
            command:
            - pg_isready
            - -U
            - postgres
          initialDelaySeconds: 30
          periodSeconds: 10
        resources:
          requests:
            cpu: 250m
            memory: 512Mi
          limits:
            cpu: "1"
            memory: 2Gi
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      storageClassName: fast-ssd
      resources:
        requests:
          storage: 50Gi
---
apiVersion: v1
kind: Service
metadata:
  name: postgres
spec:
  clusterIP: None
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
---
apiVersion: v1
kind: Service
metadata:
  name: postgres-read
spec:
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
```

Clients connect:
- Write traffic → `postgres-0.postgres.default.svc.cluster.local` (primary)
- Read traffic → `postgres-read.default.svc.cluster.local` (all replicas, round-robin)

### Pattern 2: Update Strategy

```yaml
spec:
  updateStrategy:
    type: RollingUpdate        # Default
    rollingUpdate:
      maxUnavailable: 1        # How many pods can be down during update
      partition: 0             # Only update pods with ordinal >= partition
```

Partitioned updates are critical for databases:

```yaml
# Canary update: only update postgres-2 first
spec:
  updateStrategy:
    type: RollingUpdate
    rollingUpdate:
      partition: 2             # Only pods with ordinal >= 2 get updated
```

```bash
# Update to new image, only postgres-2 changes
kubectl set image statefulset/postgres postgres=postgres:17

# Verify postgres-2 works, then update the rest
kubectl patch statefulset postgres -p '{"spec":{"updateStrategy":{"rollingUpdate":{"partition":0}}}}'
```

### Pattern 3: Pod Management Policy

```yaml
spec:
  podManagementPolicy: OrderedReady  # Default: create pods in order (0, 1, 2)
  # podManagementPolicy: Parallel     # Create all pods simultaneously
```

Use `Parallel` when:
- Pods don't depend on each other's startup order
- You need faster scaling
- Example: independent caches, stateless workers with local storage

Use `OrderedReady` (default) when:
- Pods have startup dependencies
- Primary must exist before replicas
- Example: databases, message queues, consensus systems

## When to Use StatefulSet vs Deployment

```
┌───────────────────────────┬─────────────────────────────────────┐
│ Use StatefulSet            │ Use Deployment                      │
├───────────────────────────┼─────────────────────────────────────┤
│ Databases                  │ Web servers                         │
│ Message queues (Kafka)     │ API gateways                        │
│ Distributed systems (Etcd) │ Static file servers                 │
│ ZooKeeper                  │ Microservices (stateless)           │
│ Elasticsearch              │ Background workers                  │
│ Redis Cluster (not single) │ Any app with external DB            │
│ MinIO (object storage)     │ Any horizontally scalable stateless │
│ Consul                     │ service                             │
├───────────────────────────┼─────────────────────────────────────┤
│ Each pod has UNIQUE role   │ All pods are IDENTICAL              │
│ Pods are NOT interchangeable│ Pods ARE interchangeable            │
│ Per-pod persistent storage │ Shared or no storage                │
│ Stable network identity    │ Random names are fine               │
│ Ordered operations needed  │ Parallel operations are fine        │
└───────────────────────────┴─────────────────────────────────────┘
```

**Rule of thumb:** If you can describe your app as "run N identical copies," use a Deployment. If each instance has a distinct role (primary, replica, shard), use a StatefulSet.

## Hands-On Lab: Deploy PostgreSQL with StatefulSet

### Lab 1: Deploy a 3-Node PostgreSQL Cluster

```bash
# Create the namespace
kubectl create namespace database

# Create the secret for PostgreSQL password
kubectl create secret generic postgres-secret \
  --from-literal=password=mysecretpassword \
  -n database

# Deploy the headless service
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: Service
metadata:
  name: postgres
  namespace: database
spec:
  clusterIP: None
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
EOF

# Deploy the StatefulSet
cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: postgres
  namespace: database
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
            command: ["pg_isready", "-U", "postgres"]
          initialDelaySeconds: 5
          periodSeconds: 5
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
EOF
```

### Lab 2: Observe StatefulSet Behavior

```bash
# Watch pods come up IN ORDER
kubectl get pods -n database -w
# You'll see: postgres-0 → postgres-1 → postgres-2

# Verify stable DNS names
kubectl exec postgres-0 -n database -- hostname
# Output: postgres-0

kubectl exec postgres-0 -n database -- nslookup postgres-1.postgres.database.svc.cluster.local

# Verify each pod has its own PVC
kubectl get pvc -n database
# NAME                 STATUS  VOLUME
# data-postgres-0      Bound   pvc-xxx
# data-postgres-1      Bound   pvc-yyy
# data-postgres-2      Bound   pvc-zzz

# Write data to postgres-0
kubectl exec postgres-0 -n database -- psql -U postgres -c "CREATE DATABASE testdb;"
kubectl exec postgres-0 -n database -- psql -U postgres -d testdb -c "CREATE TABLE items (id serial PRIMARY KEY, name text);"
kubectl exec postgres-0 -n database -- psql -U postgres -d testdb -c "INSERT INTO items (name) VALUES ('hello from postgres-0');"

# Delete postgres-0 pod
kubectl delete pod postgres-0 -n database

# Watch it come back with the SAME name and SAME data
kubectl get pods -n database -w
# postgres-0 will be recreated

# Verify data survives
kubectl exec postgres-0 -n database -- psql -U postgres -d testdb -c "SELECT * FROM items;"
# Output: hello from postgres-0
```

### Lab 3: Scale the StatefulSet

```bash
# Scale up to 5 replicas
kubectl scale statefulset postgres -n database --replicas=5

# Watch ordered creation
kubectl get pods -n database -w
# postgres-3 → postgres-4

# Scale down to 2 replicas
kubectl scale statefulset postgres -n database --replicas=2

# Watch ordered deletion (reverse order)
kubectl get pods -n database -w
# postgres-4 deleted → postgres-3 deleted → postgres-2 deleted

# PVCs are NOT deleted (data is preserved)
kubectl get pvc -n database
# All 5 PVCs still exist, even though pods 2-4 are gone

# Scale back up — pods reattach to their existing PVCs
kubectl scale statefulset postgres -n database --replicas=3
# postgres-2 comes back with its old data
```

### Lab 4: Clean Up

```bash
# Delete the StatefulSet
kubectl delete statefulset postgres -n database

# PVCs are still there (Retain policy by default)
kubectl get pvc -n database

# If you want to permanently delete the data
kubectl delete pvc data-postgres-0 data-postgres-1 data-postgres-2 -n database

# Delete the service and namespace
kubectl delete service postgres -n database
kubectl delete namespace database
```

## Limitation: StatefulSets Handle Pods, But Not Node-Level Tasks

StatefulSets run application workloads that need identity and storage. But some tasks need to run on **every node** in the cluster:
- Log collectors (Fluentd on every node)
- Monitoring agents (Prometheus node-exporter)
- Network plugins (CNI daemons)

And some tasks need to run **once to completion**:
- Database backups
- Batch processing
- Scheduled cleanup

StatefulSets run N replicas with identity. Neither Deployments nor StatefulSets handle "one per node" or "run once and stop."

**Next problem:** How do you run node-level agents and batch jobs?

→ **Next module:** [34-daemonsets-and-jobs](../34-daemonsets-and-jobs/) — Background tasks and node-level agents

## Checklist

- [ ] I can explain why Deployments fail for stateful applications
- [ ] I understand the three StatefulSet guarantees (identity, ordering, storage)
- [ ] I can create a headless Service for a StatefulSet
- [ ] I know how volumeClaimTemplates create per-pod PVCs
- [ ] I understand DNS naming: {pod}.{service}.{namespace}.svc.cluster.local
- [ ] I can distinguish when to use StatefulSet vs Deployment
- [ ] I can deploy a database with StatefulSet and verify data persistence
- [ ] I understand partitioned updates for safe rollouts
