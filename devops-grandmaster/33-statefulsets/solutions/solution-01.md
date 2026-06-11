# Solution 01: StatefulSet vs Deployment for Stateful Workloads

## Part 1: Core Concept Questions - Answers

### 1. What are the three guarantees that a StatefulSet provides that a Deployment does not?

**Answer:**

1. **Stable Network Identity**: Each pod gets a predictable, persistent name (`statefulset-name-ordinal`) and a DNS entry that survives restarts (`pod-name.service-name.namespace.svc.cluster.local`). Deployments give pods random hash suffixes that change on every restart.

2. **Ordered Deployment and Scaling**: Pods are created sequentially (0, 1, 2) with each waiting for the previous to be Ready. Deletion happens in reverse order (2, 1, 0). Deployments create and delete pods in arbitrary order.

3. **Stable, Persistent Storage**: Each pod gets its own PVC created from `volumeClaimTemplates`. The PVC follows the pod by ordinal name and is NOT deleted when the pod is deleted. Deployments share PVCs across all replicas, and storage lifecycle is not tied to individual pods.

### 2. Why does a Deployment's random pod naming break database replication?

**Answer:**

Database replication relies on **stable, known hostnames**. A replica must connect to the primary using a consistent address.

```
With Deployment:
  Primary: mydb-7b4f8c6d9-x2k4n  ← replicas connect here
  Node restarts → new pod: mydb-7b4f8c6d9-r9m2q  ← different name!
  Replicas still trying to connect to x2k4n → CONNECTION FAILED

With StatefulSet:
  Primary: db-0  ← replicas connect here
  Node restarts → same pod: db-0  ← same name, same DNS
  Replicas reconnect to db-0 → SUCCESS
```

Additionally, replication configurations (like PostgreSQL's `primary_conninfo`) are baked into the config. If the primary's hostname changes, every replica needs to be reconfigured. With StatefulSets, the hostname is permanent.

### 3. What is a headless Service, and why is it required for StatefulSets?

**Answer:**

A headless Service is a Kubernetes Service with `clusterIP: None`. Unlike a regular Service that provides a single virtual IP (VIP) for load balancing, a headless Service returns the individual IP addresses of all selected pods.

**Why StatefulSets need it:**

StatefulSets need per-pod DNS entries, not a single load-balanced endpoint. The headless Service creates DNS A records for each pod:

```
Regular Service (ClusterIP):
  myservice.default.svc.cluster.local → 10.96.0.100 (VIP, load-balanced)

Headless Service:
  pod-0.myservice.default.svc.cluster.local → 10.244.0.5 (pod-0's IP)
  pod-1.myservice.default.svc.cluster.local → 10.244.1.5 (pod-1's IP)
  pod-2.myservice.default.svc.cluster.local → 10.244.2.5 (pod-2's IP)
```

Without the headless Service, clients cannot address individual pods by name, which breaks the stable identity guarantee.

### 4. How does `volumeClaimTemplates` differ from a regular `volumes` section with a PVC reference in a Deployment?

**Answer:**

**Deployment with a PVC reference:**
```yaml
volumes:
- name: data
  persistentVolumeClaim:
    claimName: shared-pvc   # One PVC, shared by ALL replicas
```
- All pods mount the SAME PVC
- If you have 3 replicas, they all read/write to the same storage
- This works for read-only shared data, but breaks for databases

**StatefulSet with `volumeClaimTemplates`:**
```yaml
volumeClaimTemplates:
- metadata:
    name: data
  spec:
    accessModes: ["ReadWriteOnce"]
    resources:
      requests:
        storage: 10Gi
```
- Creates a SEPARATE PVC for each pod: `data-postgres-0`, `data-postgres-1`, `data-postgres-2`
- Each pod has its own isolated storage
- PVCs are named by ordinal, so they persist and reattach on pod restart

### 5. When a StatefulSet pod is deleted and recreated, what three things remain the same?

**Answer:**

1. **Pod Name**: The new pod gets the exact same name (e.g., `postgres-0`). Not a new random name.

2. **DNS Name**: The DNS entry `postgres-0.postgres.namespace.svc.cluster.local` continues to resolve to the new pod's IP.

3. **PVC Attachment**: The same PVC (`data-postgres-0`) is re-attached to the new pod. All data written before the deletion is still there.

This is the core of StatefulSet's "pet" model: the pod has a persistent identity that survives restarts.

---

## Part 2: Scenario Matching - Answers

### Scenario A: PostgreSQL with primary and two replicas

**Workload Type: StatefulSet**

**Reasoning:**
- Primary has a specific role (accepts writes)
- Replicas connect to the primary by hostname for WAL streaming
- Each node needs its own data directory
- Ordered startup: primary must be ready before replicas connect
- Scaling down should remove replicas, never the primary

**Why not Deployment:**
- Random pod names break replication configuration
- Random scaling order could kill the primary
- Shared PVC would corrupt data

### Scenario B: Node.js API server with external database

**Workload Type: Deployment**

**Reasoning:**
- Application is stateless (no local data)
- Database is external (managed RDS)
- All pods are identical and interchangeable
- No need for stable identity or ordered operations

**Why not StatefulSet:**
- StatefulSet adds unnecessary complexity
- No benefit from stable identity (pods do not talk to each other)
- No need for per-pod storage

### Scenario C: Kafka cluster with 3 brokers

**Workload Type: StatefulSet**

**Reasoning:**
- Each broker has a unique identity (broker.id)
- Each broker stores its own partition data
- Producers and consumers need stable broker addresses
- Cluster membership requires stable identities for metadata

**Why not Deployment:**
- Kafka brokers register with ZooKeeper using their hostname
- Changing hostnames on restart would break the cluster
- Each broker's data is unique (different partitions)

### Scenario D: Redis single-instance cache

**Workload Type: Deployment (with optional PVC)**

**Reasoning:**
- Single instance has no peers to coordinate with
- Data is ephemeral (cache can be rebuilt)
- No need for stable identity (only one instance)
- Application can tolerate Redis restarts with cache miss

**Why not StatefulSet:**
- StatefulSet provides no benefit for a single instance
- No ordered operations needed (only one pod)
- No peer-to-peer coordination required

### Scenario E: etcd cluster

**Workload Type: StatefulSet**

**Reasoning:**
- Consensus protocol (Raft) requires stable member identities
- Members must find each other by hostname
- Each member has its own data store
- Cluster membership changes require careful orchestration

**Why not Deployment:**
- Random pod names would break Raft consensus
- etcd members advertise their peer URLs (hostnames)
- Changing hostnames on restart would split the cluster

### Scenario F: Static file server from ConfigMap

**Workload Type: Deployment**

**Reasoning:**
- All pods serve identical content
- Content comes from ConfigMap (not persistent storage)
- Pods are fully interchangeable
- No state to persist across restarts

**Why not StatefulSet:**
- No persistent storage needed (ConfigMap is the source)
- No unique pod identities needed
- All pods are identical

---

## Part 3: Failure Behavior - Answers

### 1. Deployment with 3 replicas, node crashes

**Answer:**

```
Before crash:
  Node A: mydb-7b4f8c6d9-x2k4n (has data)
  Node B: mydb-7b4f8c6d9-r9m2q
  Node C: mydb-7b4f8c6d9-k7j3p

After crash (Node A goes down):
  Kubernetes schedules a NEW pod: mydb-7b4f8c6d9-w8n5t
  This pod gets a NEW random name
  If using a PVC: the PVC might be stuck on Node A (RWO)
  If using hostPath: data is GONE (it was on Node A's disk)
  If using emptyDir: data is GONE
```

**Key problems:**
- New pod has a different name (clients cannot find it)
- PVC may not be accessible from a different node (RWO restriction)
- No ordered recovery (all pods are treated identically)

### 2. StatefulSet with 3 replicas, node crashes

**Answer:**

```
Before crash:
  Node A: postgres-0 (has PVC data-postgres-0)
  Node B: postgres-1 (has PVC data-postgres-1)
  Node C: postgres-2 (has PVC data-postgres-2)

After crash (Node A goes down):
  Kubernetes creates a new pod: postgres-0 (SAME name)
  PVC data-postgres-0 is re-attached (if storage is network-attached)
  DNS name postgres-0.postgres... still resolves
  postgres-1 and postgres-2 are UNAFFECTED
```

**Key advantages:**
- Same pod name (clients find it automatically)
- Same PVC (data survives if using network storage)
- Other pods continue serving (ordered operations)
- If postgres-0 was the primary, replicas detect the disconnect and wait for it to return

### 3. Scale StatefulSet from 3 to 1

**Answer:**

**Which pod remains:** postgres-0 (the lowest ordinal)

**What happens to the deleted pods:**
```
Scale down from 3 to 1:
  postgres-2 is deleted first (highest ordinal)
  postgres-1 is deleted next
  postgres-0 remains

PVCs after scale down:
  data-postgres-0  ← STILL EXISTS (bound to postgres-0)
  data-postgres-1  ← STILL EXISTS (Released, pod is gone)
  data-postgres-2  ← STILL EXISTS (Released, pod is gone)
```

**PVCs are NOT deleted** when pods are removed. This is by design: if you scale back up, the pods will reattach to their existing PVCs and recover their data.

```bash
# After scaling back to 3:
kubectl scale statefulset postgres --replicas=3
# postgres-1 reattaches to data-postgres-1 (old data intact)
# postgres-2 reattaches to data-postgres-2 (old data intact)
```

### 4. Scale Deployment from 3 to 1

**Answer:**

**Which pod remains:** Any one of the three (Kubernetes chooses based on various factors, not by ordinal since there is no ordinal).

**What happens to data:**
```
Scale down from 3 to 1:
  Two pods are killed (random selection)
  If they had local data (emptyDir, hostPath): DATA IS GONE
  If they shared a PVC: the PVC is still mounted to the remaining pod
  If each had its own PVC: the orphaned PVCs remain but are unattached
```

**Key difference from StatefulSet:**
- No guarantee which pod survives
- No guarantee which PVC reattaches on scale-up (new pods get new random names)
- If pods had emptyDir volumes, all data on killed pods is permanently lost

---

## Common Mistakes to Avoid

1. **Using Deployment for a database "because it is simpler"**
   - Mistake: Choosing Deployment to avoid StatefulSet complexity
   - Problem: Random names, random scaling, shared storage break databases
   - Solution: Use StatefulSet even if it seems more complex; the complexity exists because databases ARE more complex

2. **Forgetting the headless Service**
   - Mistake: Creating a StatefulSet without a headless Service
   - Problem: Pods have no individual DNS entries
   - Solution: Always create a headless Service with `clusterIP: None` and set `serviceName` to match

3. **Setting `serviceName` to a regular Service**
   - Mistake: Pointing `serviceName` to a ClusterIP Service
   - Problem: DNS resolution does not create per-pod entries
   - Solution: `serviceName` must reference a headless Service

4. **Manually deleting PVCs when scaling down**
   - Mistake: Deleting PVCs after scaling down "to clean up"
   - Problem: Data is permanently lost; scale-up creates empty volumes
   - Solution: Leave PVCs in place; they are intentionally preserved

5. **Expecting automatic primary failover**
   - Mistake: Assuming StatefulSet handles primary election automatically
   - Problem: StatefulSet provides identity and ordering, NOT application-level failover
   - Solution: Use operators (like CloudNativePG, Zalando Postgres Operator) for automatic failover
