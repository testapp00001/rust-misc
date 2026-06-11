# Exercise 03: Implement a Redis Cluster Using StatefulSets

## Objective

Deploy a Redis Cluster (not a single instance) using a StatefulSet. Redis Cluster requires stable node identities, per-node storage, and ordered operations to form a working cluster with sharding and replication.

## Background

Redis Cluster is a distributed Redis implementation that:
- Shards data across multiple nodes using hash slots (0-16383)
- Replicates each shard to a replica node
- Requires each node to know the identity of every other node
- Stores cluster state in a persistent configuration file

Unlike a single Redis instance, a Redis Cluster needs stable identities because nodes gossip with each other using their hostname:port pairs. If hostnames change on restart, the cluster breaks.

## Instructions

### Step 1: Create the Namespace

```bash
kubectl create namespace redis-cluster
```

### Step 2: Create a ConfigMap for Redis Configuration

Create a ConfigMap containing the Redis configuration that will be shared by all nodes. The configuration should include:

```
cluster-enabled yes
cluster-config-file nodes.conf
cluster-node-timeout 5000
appendonly yes
dir /data
```

### Step 3: Create the Headless Service

Create a headless Service named `redis-cluster` that:
- Has `clusterIP: None`
- Selects pods with label `app: redis-cluster`
- Exposes port 6379 (data) and 16379 (cluster bus)

### Step 4: Create the StatefulSet

Create a StatefulSet named `redis-cluster` with:
- `serviceName: redis-cluster`
- `replicas: 6` (3 masters + 3 replicas)
- Container using image `redis:7`
- Command that starts Redis with the config file from the ConfigMap
- A volumeMount for data at `/data`
- A volumeMount for the ConfigMap at `/redis-conf`
- A `volumeClaimTemplates` section requesting 1Gi storage
- Resource requests: 100m CPU, 128Mi memory
- Resource limits: 250m CPU, 256Mi memory

### Step 5: Create the Cluster

After all 6 pods are running, use `redis-cli --cluster create` to form the cluster:

```bash
# Get the pod IPs
kubectl get pods -n redis-cluster -o wide

# Create the cluster (use the actual IPs)
kubectl exec redis-cluster-0 -n redis-cluster -- \
  redis-cli --cluster create \
  <ip-0>:6379 <ip-1>:6379 <ip-2>:6379 \
  <ip-3>:6379 <ip-4>:6379 <ip-5>:6379 \
  --cluster-replicas 1 --cluster-yes
```

### Step 6: Verify the Cluster

1. Check cluster info and nodes
2. Write a key and verify it is sharded across nodes
3. Delete a pod and verify the cluster self-heals

## Deliverables

Create the following YAML manifests:
1. `redis-configmap.yaml` - The Redis configuration
2. `redis-headless-service.yaml` - The headless Service
3. `redis-statefulset.yaml` - The StatefulSet

## Success Criteria

- [ ] 6 Redis pods are running (redis-cluster-0 through redis-cluster-5)
- [ ] Each pod has its own PVC for data persistence
- [ ] Cluster is formed with 3 masters and 3 replicas
- [ ] `kubectl exec redis-cluster-0 -n redis-cluster -- redis-cli cluster info` shows `cluster_state:ok`
- [ ] Data written to one node can be read from another node
- [ ] Deleting a pod causes the cluster to mark that node as failed and promote a replica

## Hints

<details>
<summary>Hint 1: Redis ConfigMap Structure</summary>

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
```

The `cluster-config-file` is where Redis stores cluster topology. It must be on persistent storage.
</details>

<details>
<summary>Hint 2: StatefulSet Command</summary>

Redis needs to be started with the config file. Use a command override:

```yaml
containers:
- name: redis
  image: redis:7
  command: ["redis-server", "/redis-conf/redis.conf"]
```

The ConfigMap is mounted at `/redis-conf/` and the data volume at `/data/`.
</details>

<details>
<summary>Hint 3: Both Ports for Cluster Bus</summary>

Redis Cluster uses two ports:
- 6379: Client connections
- 16379: Cluster bus (node-to-node gossip)

Both must be exposed in the Service and the container:

```yaml
ports:
- containerPort: 6379
  name: client
- containerPort: 16379
  name: gossip
```
</details>

<details>
<summary>Hint 4: Cluster Create Command</summary>

You can automate the cluster creation with a script that resolves pod hostnames:

```bash
# Use DNS names instead of IPs
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

This uses the stable DNS names that StatefulSets provide.
</details>

<details>
<summary>Hint 5: Verifying Cluster Health</summary>

```bash
# Check cluster info
kubectl exec redis-cluster-0 -n redis-cluster -- redis-cli cluster info

# Check cluster nodes and their roles
kubectl exec redis-cluster-0 -n redis-cluster -- redis-cli cluster nodes

# Test data sharding
kubectl exec redis-cluster-0 -n redis-cluster -- redis-cli -c set testkey "hello"
kubectl exec redis-cluster-1 -n redis-cluster -- redis-cli -c get testkey
```

The `-c` flag enables cluster mode in redis-cli, which follows redirects.
</details>

## Common Issues

1. **Cluster stuck in FAIL state**: Wait a few seconds for gossip to converge after formation
2. **Node can't join cluster**: Ensure both ports (6379 and 16379) are exposed
3. **Data not persisting**: Verify the `dir` directive points to the mounted volume
4. **Cluster breaks on pod restart**: Ensure `cluster-config-file` is on persistent storage so nodes remember the cluster topology
