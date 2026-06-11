# Solution 03: Implement a Redis Cluster Using StatefulSets

## Complete Solution

### redis-configmap.yaml

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: redis-conf
  namespace: redis-cluster
data:
  redis.conf: |
    cluster-enabled yes
    cluster-config-file nodes.conf
    cluster-node-timeout 5000
    appendonly yes
    dir /data
    port 6379
```

### redis-headless-service.yaml

```yaml
apiVersion: v1
kind: Service
metadata:
  name: redis-cluster
  namespace: redis-cluster
spec:
  clusterIP: None
  selector:
    app: redis-cluster
  ports:
  - port: 6379
    targetPort: 6379
    name: client
  - port: 16379
    targetPort: 16379
    name: gossip
```

### redis-statefulset.yaml

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: redis-cluster
  namespace: redis-cluster
spec:
  serviceName: redis-cluster
  replicas: 6
  selector:
    matchLabels:
      app: redis-cluster
  template:
    metadata:
      labels:
        app: redis-cluster
    spec:
      containers:
      - name: redis
        image: redis:7
        command: ["redis-server", "/redis-conf/redis.conf"]
        ports:
        - containerPort: 6379
          name: client
        - containerPort: 16379
          name: gossip
        volumeMounts:
        - name: data
          mountPath: /data
        - name: conf
          mountPath: /redis-conf
        resources:
          requests:
            cpu: 100m
            memory: 128Mi
          limits:
            cpu: 250m
            memory: 256Mi
      volumes:
      - name: conf
        configMap:
          name: redis-conf
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 1Gi
```

### Cluster Formation Script

After all 6 pods are running, form the cluster:

```bash
# Wait for all pods to be ready
kubectl wait --for=condition=Ready pod -l app=redis-cluster -n redis-cluster --timeout=120s

# Form the cluster using stable DNS names
kubectl exec redis-cluster-0 -n redis-cluster -- \
  redis-cli --cluster create \
  redis-cluster-0.redis-cluster.redis-cluster.svc.cluster.local:6379 \
  redis-cluster-1.redis-cluster.redis-cluster.svc.cluster.local:6379 \
  redis-cluster-2.redis-cluster.redis-cluster.svc.cluster.local:6379 \
  redis-cluster-3.redis-cluster.redis-cluster.svc.cluster.local:6379 \
  redis-cluster-4.redis-cluster.redis-cluster.svc.cluster.local:6379 \
  redis-cluster-5.redis-cluster.redis-cluster.svc.cluster.local:6379 \
  --cluster-replicas 1 --cluster-yes
```

---

## Why This Solution Works

### 1. StatefulSet Provides Stable Node Identities

Redis Cluster nodes identify each other by hostname:port. When a node starts, it reads its `nodes.conf` file which contains the cluster topology with hostnames. If hostnames change on restart, the cluster breaks.

With StatefulSets:
```
redis-cluster-0 → always redis-cluster-0
redis-cluster-0.redis-cluster.redis-cluster.svc.cluster.local → stable DNS
```

Even if the pod restarts, it gets the same hostname and DNS name. The `nodes.conf` file remains valid.

### 2. Per-Pod Storage Preserves Cluster State

Each Redis node stores its cluster state in `nodes.conf` on the persistent volume. When a pod restarts:
- It reads `nodes.conf` from its PVC
- It remembers its role (master or replica)
- It remembers which master it replicates (if a replica)
- It reconnects to its peers using their stable DNS names

Without persistent storage, a restarted node would lose its cluster membership and need to be re-added.

### 3. Both Ports Are Required

Redis Cluster uses two ports:
- **6379**: Client connections (where your app sends commands)
- **16379**: Cluster bus (gossip protocol between nodes)

The cluster bus port is `port + 10000` by default. Nodes communicate over the bus port to:
- Exchange cluster state information
- Detect failures
- Perform failover elections

If port 16379 is not exposed, nodes cannot gossip and the cluster cannot form.

### 4. `--cluster-replicas 1` Creates Master-Replica Pairs

With 6 nodes and `--cluster-replicas 1`:
- 3 nodes become masters (redis-cluster-0, 1, 2)
- 3 nodes become replicas (redis-cluster-3, 4, 5)
- Each replica replicates one master

The assignment is:
```
redis-cluster-0 (master) ← redis-cluster-3 (replica)
redis-cluster-1 (master) ← redis-cluster-4 (replica)
redis-cluster-2 (master) ← redis-cluster-5 (replica)
```

Hash slots (0-16383) are distributed evenly across the 3 masters.

### 5. ConfigMap Provides Shared Configuration

The ConfigMap contains the Redis configuration shared by all nodes. Each node starts with:
- `cluster-enabled yes`: Enables cluster mode
- `cluster-config-file nodes.conf`: Where to store cluster topology
- `cluster-node-timeout 5000`: How long to wait before marking a node as failed
- `appendonly yes`: Enables AOF persistence for data durability

The ConfigMap is mounted as a volume (not a subPath) so that config changes can be picked up by restarting pods.

---

## Verification Steps

### 1. Check All Pods Are Running

```bash
kubectl get pods -n redis-cluster -o wide
```

Expected: 6 pods, all Running.

### 2. Check Cluster Info

```bash
kubectl exec redis-cluster-0 -n redis-cluster -- redis-cli cluster info
```

Expected:
```
cluster_enabled:1
cluster_state:ok
cluster_slots_assigned:16384
cluster_slots_ok:16384
cluster_known_nodes:6
cluster_size:3
```

### 3. Check Cluster Nodes

```bash
kubectl exec redis-cluster-0 -n redis-cluster -- redis-cli cluster nodes
```

Expected: 6 nodes listed with their roles (master/slave) and slot assignments.

### 4. Test Data Sharding

```bash
# Set a key (redis-cli -c follows redirects)
kubectl exec redis-cluster-0 -n redis-cluster -- redis-cli -c set mykey "hello"

# Get from a different node (will redirect if needed)
kubectl exec redis-cluster-1 -n redis-cluster -- redis-cli -c get mykey
```

### 5. Test Pod Restart Recovery

```bash
# Delete a master pod
kubectl delete pod redis-cluster-0 -n redis-cluster

# Check cluster status (should show a failover)
kubectl exec redis-cluster-1 -n redis-cluster -- redis-cli cluster nodes

# Wait for redis-cluster-0 to come back
kubectl wait --for=condition=Ready pod/redis-cluster-0 -n redis-cluster --timeout=60s

# Verify it rejoins the cluster
kubectl exec redis-cluster-0 -n redis-cluster -- redis-cli cluster nodes
```

### 6. Verify PVCs

```bash
kubectl get pvc -n redis-cluster
```

Expected: 6 PVCs, one for each pod.

---

## Common Mistakes to Avoid

### 1. Forgetting the Gossip Port

**Mistake:** Only exposing port 6379 in the Service and container.

**Problem:** Nodes can accept client connections but cannot communicate with each other. `CLUSTER MEET` commands fail, and the cluster cannot form.

**Solution:** Always expose both port 6379 (client) and 16379 (gossip).

### 2. Not Using Persistent Storage for nodes.conf

**Mistake:** Using an emptyDir or no volume for the data directory.

**Problem:** When a pod restarts, it loses `nodes.conf`. The node forgets it was part of a cluster and starts fresh. You must re-add it to the cluster every time.

**Solution:** Use `volumeClaimTemplates` to persist `nodes.conf` across restarts.

### 3. Using IPs Instead of DNS Names for Cluster Formation

**Mistake:** Using pod IPs in the `redis-cli --cluster create` command.

**Problem:** Pod IPs change on restart. The cluster topology stored in `nodes.conf` would contain stale IPs.

**Solution:** Use the stable DNS names provided by the StatefulSet:
```
redis-cluster-0.redis-cluster.redis-cluster.svc.cluster.local:6379
```

### 4. Running Cluster Create Before All Pods Are Ready

**Mistake:** Running `redis-cli --cluster create` before all 6 pods are running and listening.

**Problem:** Some nodes are not reachable, and the cluster formation fails or creates an incomplete cluster.

**Solution:** Wait for all pods to be ready:
```bash
kubectl wait --for=condition=Ready pod -l app=redis-cluster -n redis-cluster --timeout=120s
```

### 5. Using `redis:7-alpine` Without Cluster Support

**Mistake:** Using an Alpine-based image that may have missing cluster dependencies.

**Problem:** Some Alpine images have compatibility issues with Redis Cluster.

**Solution:** Use the standard `redis:7` image (Debian-based) for reliability.
