# Exercise 04: Handle StatefulSet Scaling and Rolling Updates Safely

## Objective

Implement safe scaling and rolling update strategies for a StatefulSet running a database. Practice partitioned updates, understand the difference between `OrderedReady` and `Parallel` pod management, and learn how to perform canary updates before rolling out changes to all pods.

## Background

Updating a StatefulSet running a database is dangerous. Unlike stateless web servers where you can kill and replace pods freely, database updates require careful sequencing:

- Replicas should update before the primary (so you catch issues on less critical nodes)
- Each updated node should be verified before proceeding
- If something goes wrong, you need to roll back quickly without data loss

StatefulSets provide `updateStrategy` and `partition` fields to control exactly which pods get updated and when.

## Instructions

### Step 1: Set Up the Environment

Create a namespace and deploy a 3-node PostgreSQL StatefulSet (reuse or adapt from Exercise 02):

```bash
kubectl create namespace update-demo

kubectl create secret generic pg-secret \
  --from-literal=password=update-demo-pass \
  -n update-demo
```

Deploy a StatefulSet named `db` with:
- 3 replicas
- Image: `postgres:16`
- Headless Service named `db`
- 1Gi PVCs
- Readiness and liveness probes

### Step 2: Perform a Standard Rolling Update

Update the image tag from `postgres:16` to `postgres:17` and observe the default rolling update behavior:

```bash
kubectl set image statefulset/db postgres=postgres:17 -n update-demo
```

Observe:
1. Which pod updates first?
2. Does the next pod wait for the previous one to be Ready?
3. What is the order of updates?

### Step 3: Perform a Partitioned (Canary) Update

Reset to `postgres:16`, then perform a canary update using partitions:

1. Set the partition to 2 (only update pods with ordinal >= 2)
2. Update the image to `postgres:17`
3. Verify that only postgres-2 changed while postgres-0 and postgres-1 remain on postgres:16
4. Verify postgres-2 is healthy
5. Set the partition to 0 to roll out to all pods

### Step 4: Demonstrate Parallel Pod Management

Create a second StatefulSet named `cache` that:
- Uses `podManagementPolicy: Parallel` (all pods start simultaneously)
- Runs Redis (image: `redis:7`)
- Has 3 replicas

Compare the startup behavior with the `db` StatefulSet that uses `OrderedReady`.

### Step 5: Safe Rollback

After the update to postgres:17, perform a rollback:
1. Check the rollout history
2. Roll back to postgres:16
3. Verify all pods are back on the old version

## Deliverables

1. YAML manifests for the `db` StatefulSet and its headless Service
2. YAML manifest for the `cache` StatefulSet with `Parallel` pod management
3. Screenshots or terminal output showing:
   - Ordered update behavior (pods updating one at a time)
   - Partitioned update (only ordinal >= 2 updating)
   - Parallel pod management (all pods starting at once)

## Success Criteria

- [ ] You can explain what `partition` does in a rolling update
- [ ] You can perform a canary update by setting partition to the highest ordinal
- [ ] You can verify the canary pod is healthy before rolling out to the rest
- [ ] You can roll the partition back to 0 to update all remaining pods
- [ ] You understand the difference between `OrderedReady` and `Parallel` pod management
- [ ] You can perform a rollback using `kubectl rollout undo`

## Hints

<details>
<summary>Hint 1: Partition Behavior</summary>

The `partition` field tells Kubernetes: "Only update pods whose ordinal is >= partition."

```yaml
updateStrategy:
  type: RollingUpdate
  rollingUpdate:
    partition: 2
```

With 3 replicas (ordinals 0, 1, 2):
- partition: 2 → only postgres-2 updates
- partition: 1 → postgres-1 and postgres-2 update
- partition: 0 → all pods update (default behavior)

This lets you test changes on a single pod before rolling out to the entire cluster.
</details>

<details>
<summary>Hint 2: Canary Update Workflow</summary>

The safe production update workflow:

```
1. Set partition to N-1 (highest ordinal)
2. Update the image
3. Only pod-N updates (the "canary")
4. Monitor the canary for errors, performance issues
5. If canary is healthy: set partition to 0 (roll out to all)
6. If canary is bad: roll back the image, set partition to 0
```

```bash
# Step 1: Set partition
kubectl patch statefulset db -n update-demo \
  -p '{"spec":{"updateStrategy":{"rollingUpdate":{"partition":2}}}}'

# Step 2: Update image (only postgres-2 changes)
kubectl set image statefulset/db postgres=postgres:17 -n update-demo

# Step 3: Verify postgres-2
kubectl exec db-2 -n update-demo -- psql -U postgres -c "SELECT version();"

# Step 4: Roll out to all
kubectl patch statefulset db -n update-demo \
  -p '{"spec":{"updateStrategy":{"rollingUpdate":{"partition":0}}}}'
```
</details>

<details>
<summary>Hint 3: OrderedReady vs Parallel</summary>

```yaml
# Default: pods start one at a time, in order
spec:
  podManagementPolicy: OrderedReady

# All pods start simultaneously
spec:
  podManagementPolicy: Parallel
```

**OrderedReady**: db-0 must be Ready before db-1 starts. Use for databases where the primary must be up first.

**Parallel**: All pods start at the same time. Use for stateless caches or independent workers that don't depend on each other.
</details>

<details>
<summary>Hint 4: Checking Rollout History</summary>

```bash
# View rollout history
kubectl rollout history statefulset/db -n update-demo

# Check current image on each pod
kubectl get pods -n update-demo -o custom-columns=\
  NAME:.metadata.name,IMAGE:.spec.containers[0].image,STATUS:.status.phase

# Roll back to previous revision
kubectl rollout undo statefulset/db -n update-demo
```
</details>

<details>
<summary>Hint 5: Observing Update Order</summary>

Use a watch command to see updates happen in real time:

```bash
kubectl get pods -n update-demo -w -o custom-columns=\
  NAME:.metadata.name,STATUS:.status.phase,READY:.status.conditions[?(@.type=="Ready")].status
```

With `OrderedReady`:
1. postgres-2 is deleted and recreated with new image
2. postgres-2 must become Ready
3. Then postgres-1 is deleted and recreated
4. postgres-1 must become Ready
5. Then postgres-0 is updated

Updates go in REVERSE ordinal order (highest first). This is deliberate: update replicas before the primary.
</details>

## Common Issues

1. **Update seems stuck**: A pod may be failing its readiness probe. Check pod events and logs.
2. **All pods update at once**: Verify `updateStrategy.type` is `RollingUpdate`, not `OnDelete`.
3. **Partition not working**: Ensure you patch the correct StatefulSet and the partition value is valid.
4. **Rollback doesn't change the image**: `kubectl rollout undo` may revert to the same image if only one revision exists.
