# Exercise 03: Rollback and Scaling Operations

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective

Perform rollback and scaling operations on a Deployment without step-by-step guidance. You will create a Deployment, perform multiple updates to build revision history, scale the Deployment up and down, simulate a failed update, roll back to a specific revision, and verify the final state. This exercise tests your ability to use Deployment operations independently.

## Background

Deployments maintain a revision history that enables rollback. When an update fails -- a bad image tag, a broken configuration, or a regression -- you can roll back to any previous revision. Scaling changes the replica count without triggering a new revision. Understanding the difference between these operations and when to use each is critical for production operations.

## Tasks

### Part A: Create and Update the Deployment

Create a Deployment named `api-server` with the following spec:

- 3 replicas
- Image: `httpd:2.4` (Apache HTTP Server)
- Labels: `app: api-server`
- Container port: 80
- Resource requests: 100m CPU, 64Mi memory
- Resource limits: 200m CPU, 128Mi memory

Apply it, then perform two sequential updates:

1. Update the image to `httpd:2.4.58`
2. Update the image to `httpd:2.4.59`

After each update, wait for the rollout to complete before applying the next.

```bash
kubectl rollout status deployment api-server
```

After all three versions have been deployed, check the rollout history:

```bash
kubectl rollout history deployment api-server
```

Record the revision numbers and corresponding images.

<details>
<summary>Hint -- Checking Revision Details</summary>
Use `kubectl rollout history deployment api-server --revision=<number>` to see the pod template for a specific revision. This shows you which image was used in each revision.
</details>

### Part B: Scaling Operations

Scale the Deployment to 5 replicas:

```bash
kubectl scale deployment api-server --replicas=5
```

Verify the scale:

```bash
kubectl get deployment api-server
kubectl get pods -l app=api-server
```

Now scale back down to 2 replicas:

```bash
kubectl scale deployment api-server --replicas=2
```

Answer these questions:

1. When you scaled from 3 to 5, how many new Pods were created?
2. When you scaled from 5 to 2, how many Pods were deleted? Which Pods were chosen for deletion?
3. Did scaling create a new revision? Check with `kubectl rollout history`.
4. What is the difference between `kubectl scale` and changing `replicas` in the YAML and running `kubectl apply`?

<details>
<summary>Hint -- Scaling and Revisions</summary>
Scaling does not create a new revision because the pod template has not changed. A new revision is only created when the pod template changes (image, environment variables, resources, etc.). Both `kubectl scale` and `kubectl apply` with a changed replica count produce the same result: the Deployment's replica count is updated without creating a new revision.
</details>

### Part C: Simulate a Bad Update

Update the image to a nonexistent tag:

```bash
kubectl set image deployment api-server httpd=httpd:does-not-exist
```

Watch the rollout:

```bash
kubectl rollout status deployment api-server
```

This will either hang or report failure. In another terminal, investigate:

```bash
kubectl get pods -l app=api-server
kubectl describe pods -l app=api-server | grep -A 5 "Events"
```

Answer these questions:

1. What status do the new Pods show? Why?
2. Are the old Pods still running? Why or why not?
3. What does `kubectl rollout status` report?

<details>
<summary>Hint -- ImagePullBackOff</summary>
When the image tag does not exist, the new Pods will be stuck in `ImagePullBackOff` or `ErrImagePull`. With the default rolling update strategy, Kubernetes will not delete old Pods until new Pods are ready. Since the new Pods will never become ready, the old Pods continue serving traffic. This is the safety mechanism of rolling updates.
</details>

### Part D: Rollback

Roll back to the previous revision:

```bash
kubectl rollout undo deployment api-server
```

Verify the rollback:

```bash
kubectl rollout status deployment api-server
kubectl get deployment api-server -o jsonpath='{.spec.template.spec.containers[0].image}'
kubectl get pods -l app=api-server -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.spec.containers[0].image}{"\n"}{end}'
```

Now check the history again:

```bash
kubectl rollout history deployment api-server
```

Answer these questions:

1. What image is the Deployment running after the rollback?
2. Did the rollback create a new revision or reuse an old one?
3. How many ReplicaSets exist now?

<details>
<summary>Hint -- Rollback Mechanics</summary>
`kubectl rollout undo` rolls back to the second-to-last revision by default. It does not create a new revision entry -- it reuses the ReplicaSet from the target revision. The bad revision is effectively replaced in the history.
</details>

### Part E: Rollback to a Specific Revision

Now roll back all the way to revision 1 (the original `httpd:2.4`):

```bash
kubectl rollout undo deployment api-server --to-revision=1
```

Verify:

```bash
kubectl rollout status deployment api-server
kubectl get deployment api-server -o jsonpath='{.spec.template.spec.containers[0].image}'
kubectl rollout history deployment api-server
```

Answer these questions:

1. What image is the Deployment running now?
2. What revisions are in the history? Is revision 1 still listed?
3. If you wanted to go back to `httpd:2.4.58`, which revision number would you use?

### Part F: Clean Up

```bash
kubectl delete deployment api-server
```

## Success Criteria

- [ ] You created a Deployment and performed two sequential updates
- [ ] You scaled the Deployment up and down and verified the replica count
- [ ] You understand that scaling does not create new revisions
- [ ] You simulated a failed update and observed the Pods in ImagePullBackOff
- [ ] You rolled back to the previous revision and verified the image
- [ ] You rolled back to a specific revision and verified the image
- [ ] You can explain why old Pods continue running during a failed rolling update

## What You Should Understand After This Exercise

After completing this exercise, you should be able to perform rollbacks confidently, understand the difference between scaling and updating, know why failed updates do not cause downtime with rolling updates, and navigate the revision history to find and restore any previous version. These skills are essential for incident response and safe production operations.
