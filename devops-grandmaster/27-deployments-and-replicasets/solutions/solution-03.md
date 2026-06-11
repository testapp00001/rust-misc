# Solution 03: Rollback and Scaling Operations

## Part A: Create and Update the Deployment

The Deployment is created with `httpd:2.4`, then updated sequentially to `httpd:2.4.58` and `httpd:2.4.59`. After all three versions:

```bash
kubectl rollout history deployment api-server
```

Expected output:
```
REVISION  CHANGE-CAUSE
1         <none>
2         <none>
3         <none>
```

Revision 1 = `httpd:2.4`, Revision 2 = `httpd:2.4.58`, Revision 3 = `httpd:2.4.59`.

To verify which image corresponds to each revision:
```bash
kubectl rollout history deployment api-server --revision=1
kubectl rollout history deployment api-server --revision=2
kubectl rollout history deployment api-server --revision=3
```

**Why it works:** Each image change modifies the pod template, which causes the Deployment controller to create a new ReplicaSet with a new pod-template-hash. Each new ReplicaSet corresponds to a new revision in the rollout history.

**Common mistakes:**
- Not waiting for `kubectl rollout status` to complete between updates. If you update before the previous rollout finishes, you may get unexpected revision numbering or the previous update may be interrupted.
- Expecting `CHANGE-CAUSE` to be populated automatically. You must add `kubernetes.io/change-cause` annotation to the Deployment or use `--record` flag (deprecated in newer kubectl versions).

---

## Part B: Scaling Operations

**1. When you scaled from 3 to 5, how many new Pods were created?**

2 new Pods were created. The ReplicaSet detected that the desired count (5) exceeded the current count (3) and created 2 additional Pods.

**2. When you scaled from 5 to 2, how many Pods were deleted? Which Pods were chosen for deletion?**

3 Pods were deleted. The ReplicaSet controller chooses Pods for deletion based on several factors:
- Pods on nodes with fewer replicas are preferred (to spread load).
- Pods that have been pending longer may be preferred.
- Pods with a lower readiness score may be deleted first.
- In practice, the selection is somewhat arbitrary among healthy Pods.

**3. Did scaling create a new revision?**

No. Scaling does NOT create a new revision. You can verify with `kubectl rollout history` -- the revisions remain 1, 2, and 3. A new revision is only created when the pod template changes (image, environment variables, resource limits, etc.). Changing the replica count does not modify the pod template.

**4. What is the difference between `kubectl scale` and changing `replicas` in YAML + `kubectl apply`?**

Functionally, both produce the same result: the Deployment's replica count is updated without creating a new revision. The differences are operational:
- `kubectl scale` is imperative and immediate -- good for quick adjustments.
- `kubectl apply` with changed YAML is declarative and persistent -- the change is recorded in your source file and can be version-controlled.
- `kubectl scale` overrides whatever is in the YAML until the next `kubectl apply` re-applies the original YAML value.

**Common mistakes:**
- Expecting `kubectl scale` to create a new revision. It does not, because the pod template is unchanged.
- Thinking `kubectl scale` permanently changes the YAML file. It does not -- it only changes the live state in the cluster. The next `kubectl apply` will revert to whatever is in the file.

---

## Part C: Simulate a Bad Update

**1. What status do the new Pods show? Why?**

The new Pods show `ImagePullBackOff` or `ErrImagePull`. This happens because the image `httpd:does-not-exist` does not exist in any container registry. Kubernetes cannot pull the image, so the containers never start. After repeated failed pulls, the status transitions from `ErrImagePull` to `ImagePullBackOff` (with exponential backoff between retries).

**2. Are the old Pods still running? Why or why not?**

Yes, the old Pods running `httpd:2.4.59` continue to run and serve traffic. This is because the default rolling update strategy does not delete old Pods until new Pods are ready. Since the new Pods will never become ready (they cannot pull the image), the old Pods are never removed. This is the safety mechanism of rolling updates -- a bad update does not cause downtime.

**3. What does `kubectl rollout status` report?**

It reports that the rollout is stuck, typically with a message like "Waiting for rollout to finish: 1 out of 3 new replicas have been updated..." and may eventually time out or report failure. The exact behavior depends on `progressDeadlineSeconds` (default 600 seconds). After the deadline, the condition changes to `Progressing: False` with reason `ProgressDeadlineExceeded`.

**Common mistakes:**
- Assuming a bad image update causes downtime. With rolling updates and `maxUnavailable: 0`, old Pods keep running.
- Not realizing that Kubernetes does NOT automatically roll back a failed update. It marks the deployment as stuck but requires manual intervention (`kubectl rollout undo`).

---

## Part D: Rollback

**1. What image is the Deployment running after the rollback?**

`httpd:2.4.59` -- the version before the bad update. `kubectl rollout undo` rolls back to the second-to-last revision by default.

**2. Did the rollback create a new revision or reuse an old one?**

The rollback reuses the existing ReplicaSet from revision 3 (`httpd:2.4.59`). It does not create a new revision entry. The bad revision (the one with `httpd:does-not-exist`) is effectively replaced. Check the history:

```bash
kubectl rollout history deployment api-server
```

The revision numbers shift -- the bad image revision disappears from the history, and the previously known revision takes its place.

**3. How many ReplicaSets exist now?**

3 ReplicaSets exist (one for each distinct pod template: `httpd:2.4`, `httpd:2.4.58`, `httpd:2.4.59`). The ReplicaSet for the bad image (`httpd:does-not-exist`) may still exist briefly but will be cleaned up. The exact count depends on timing and `revisionHistoryLimit`.

**Common mistakes:**
- Thinking `kubectl rollout undo` creates a brand-new ReplicaSet. It reuses the existing one for the target revision.
- Expecting the revision numbers to be perfectly sequential after rollbacks. Rollbacks reuse ReplicaSets, which can cause gaps or shifts in the revision history.

---

## Part E: Rollback to a Specific Revision

**1. What image is the Deployment running now?**

`httpd:2.4` -- the original image from revision 1.

**2. What revisions are in the history? Is revision 1 still listed?**

The revision history now shows the updated list. After rolling back to revision 1, that ReplicaSet becomes the active one. The previous active revision is kept for potential rollback. Revision 1's ReplicaSet is reused (scaled up), and the previously active ReplicaSet is scaled down.

**3. If you wanted to go back to `httpd:2.4.58`, which revision number would you use?**

You would need to check `kubectl rollout history` to find the current revision number for the `httpd:2.4.58` image. Due to rollback mechanics, the original revision numbers may have shifted. Use:

```bash
kubectl rollout history deployment api-server --revision=<N>
```

for each revision to find the one with `httpd:2.4.58`, then run:

```bash
kubectl rollout undo deployment api-server --to-revision=<N>
```

**Common mistakes:**
- Assuming revision numbers are stable. Rollbacks can renumber revisions because they reuse ReplicaSets.
- Not checking `--revision=N` before rolling back to confirm which image corresponds to which revision number.

---

## Part F: Clean Up

```bash
kubectl delete deployment api-server
```

This cascades and deletes all ReplicaSets and Pods owned by the Deployment.
