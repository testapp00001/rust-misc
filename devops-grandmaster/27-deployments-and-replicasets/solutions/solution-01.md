# Solution 01: How Deployments Manage ReplicaSets

## Part A: The Controller Hierarchy

| Resource | Purpose | It Owns | What Happens When You Delete It |
|----------|---------|---------|--------------------------------|
| Deployment | Declaratively manages ReplicaSets to maintain a desired state for an application. Handles rolling updates, rollbacks, and scaling at a high level. | ReplicaSets | All ReplicaSets owned by the Deployment are deleted (cascading deletion). Those ReplicaSets then delete their Pods. Everything is cleaned up. |
| ReplicaSet | Maintains a stable set of replica Pods running at any given time. Ensures the specified number of Pod replicas are running. | Pods | All Pods owned by the ReplicaSet are deleted. If a Deployment owns this ReplicaSet, the Deployment controller will notice the discrepancy and recreate a ReplicaSet to restore the desired state. |
| Pod | The smallest deployable unit in Kubernetes. Runs one or more containers. | Nothing (it is a leaf resource) | The Pod is terminated and removed. If a ReplicaSet owns it, the ReplicaSet controller notices the count dropped and creates a new Pod to replace it. |

**Why it works:** Kubernetes uses owner references (stored in each resource's metadata) to track parent-child relationships. When you delete a parent, the garbage collector cascades the deletion to all children. This is why `kubectl delete deployment X` also removes all ReplicaSets and Pods -- the ownership chain is followed automatically.

**Common mistakes:**
- Confusing "deleting a Pod" with "deleting a Deployment." Deleting a single Pod triggers the ReplicaSet to recreate it, but does not affect other Pods. Deleting the Deployment removes everything.
- Thinking that deleting a ReplicaSet while a Deployment owns it is a valid way to scale down. The Deployment controller will simply recreate the ReplicaSet. You must change the Deployment spec instead.

---

## Part B: ReplicaSet Creation During Updates

**1. Before the update, how many ReplicaSets exist? Why?**

Exactly 1 ReplicaSet exists. When the Deployment was first created with image `nginx:1.25`, the Deployment controller created a single ReplicaSet whose pod template matches that specification. There is no reason for more than one ReplicaSet when no updates have occurred.

**2. During the update, how many ReplicaSets exist? Why does Kubernetes keep the old one?**

2 ReplicaSets exist -- one for `nginx:1.25` (scaling down) and one for `nginx:1.26` (scaling up). Kubernetes keeps the old ReplicaSet (scaled to 0 replicas once the rollout completes) so that it can quickly roll back if needed. If the old ReplicaSet were deleted, rollback would require rebuilding the entire ReplicaSet and its Pod template from scratch.

**3. After the update completes, how many ReplicaSets exist? What determines whether the old ReplicaSet is deleted?**

2 ReplicaSets still exist. The new ReplicaSet has all 3 replicas; the old ReplicaSet has 0 replicas. The old ReplicaSet is NOT deleted after a successful update -- it persists for rollback purposes. It is only deleted when `revisionHistoryLimit` causes garbage collection, or when the Deployment itself is deleted.

**4. If you run `kubectl rollout undo`, what happens at the ReplicaSet level?**

`kubectl rollout undo` does NOT create a new ReplicaSet. It scales up the old ReplicaSet (the one with the previous image) and scales down the current ReplicaSet. The existing ReplicaSet for the previous revision is reused. This is why rollback is fast -- the ReplicaSet already exists and just needs its replica count adjusted.

**Common mistakes:**
- Assuming that `kubectl rollout undo` creates a new ReplicaSet for the rollback. It reuses the existing one.
- Thinking the old ReplicaSet is deleted after a successful rollout. It is kept (scaled to 0) until `revisionHistoryLimit` triggers cleanup.

---

## Part C: Revision History

**Observation 1: Revisions 1, 3, and 4 are listed but revision 2 is missing. What happened?**

Revision 2 was a change that produced the same pod template as another revision (either 1 or 3), or it was garbage collected due to `revisionHistoryLimit`. The most likely explanation: revision 2 was a rollback or update that resulted in a pod template identical to revision 1. Since the ReplicaSet for that template already existed, Kubernetes reused it and recorded a new revision number, but the old revision 2 entry was eventually cleaned up. Alternatively, if `revisionHistoryLimit` was set low, revision 2's ReplicaSet was deleted and its history entry was removed.

**Observation 2: `revisionHistoryLimit: 2` with 5 updates. How many ReplicaSets exist?**

Only the current ReplicaSet and 2 previous ReplicaSets exist (3 total). The earliest 2 ReplicaSets were automatically garbage collected by the Deployment controller when the limit was exceeded. The `revisionHistoryLimit` controls how many old ReplicaSets are kept for rollback purposes. Once the limit is exceeded, the oldest ReplicaSets are deleted.

**Observation 3: Three ReplicaSets -- one with 3 replicas, two with 0. Which is active?**

The ReplicaSet with 3 replicas is the active one. The other two (with 0 replicas) are kept from previous revisions for rollback purposes. They still exist because `revisionHistoryLimit` has not forced their deletion yet. They consume minimal resources (no Pods running) but occupy API server storage.

**Common mistakes:**
- Thinking revision numbers are sequential and always present. Rollbacks can reuse ReplicaSets, and `revisionHistoryLimit` can remove old entries.
- Confusing "revision count" with "ReplicaSet count." A revision is a record in the Deployment's history; a ReplicaSet is an actual API object. They are related but not identical.

---

## Part D: Predicting Behavior

**Scenario 1: A Deployment has `replicas: 5`. A node crashes, killing 2 pods. What does the ReplicaSet do?**

The ReplicaSet controller detects that only 3 Pods are running instead of the desired 5. It immediately creates 2 new Pods to replace the ones lost in the crash. These new Pods are scheduled to other available nodes. The Deployment controller is not directly involved -- this is self-healing at the ReplicaSet level. The Deployment only gets involved if the pod template needs to change.

**Scenario 2: You change `replicas` from 3 to 7 and run `kubectl apply`. What happens?**

The Deployment controller detects the desired replica count changed from 3 to 7. It updates the ReplicaSet's replica count to 7. The ReplicaSet controller then creates 4 new Pods to reach the desired count. No new ReplicaSet is created because the pod template did not change -- only the replica count did. No new revision is recorded in the rollout history.

**Scenario 3: You change the image and replicas simultaneously. What happens?**

The Deployment controller handles both changes together as part of a single reconciliation. It creates a new ReplicaSet with the new image and desired replica count of 5, then performs a rolling update from the old ReplicaSet (3 replicas, old image) to the new ReplicaSet (5 replicas, new image). The rolling update parameters (`maxSurge`, `maxUnavailable`) govern how the transition happens. One new revision is recorded that captures both changes.

**Scenario 4: A rolling update is in progress (old RS has 2 pods, new RS has 1 pod). You run `kubectl rollout undo`. What happens?**

The `undo` command tells the Deployment controller to roll back to the previous revision (the old ReplicaSet's pod template). The controller scales the old ReplicaSet back up to 3 and scales the new ReplicaSet down to 0. The 1 pod from the new ReplicaSet is terminated, and 1 new pod is created on the old ReplicaSet (it already had 2, so it needs 1 more). The net effect is that the in-progress update is reversed, and the cluster returns to the old version.

**Common mistakes:**
- Thinking the Deployment controller recreates Pods when a node crashes. It does not -- the ReplicaSet controller handles that.
- Assuming simultaneous image and replica changes are handled sequentially (first scale, then update). They are reconciled together in a single rolling update.
- Believing `kubectl rollout undo` during an in-progress update will fail or cause issues. It is designed to handle this safely.
