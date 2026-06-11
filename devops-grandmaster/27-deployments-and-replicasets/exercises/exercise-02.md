# Exercise 02: Create a Deployment with Rolling Update

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Create a Deployment with a rolling update strategy, verify that it creates the expected ReplicaSet and Pods, perform an image update, and observe the rolling update process step by step. This exercise walks you through each action so you can see how the Deployment controller behaves in practice.

## Background

When you create a Deployment, the Deployment controller creates a ReplicaSet, which creates the Pods. When you update the Deployment, the controller creates a new ReplicaSet and gradually shifts Pods from the old to the new. The `maxSurge` and `maxUnavailable` parameters control how fast this shift happens. You will observe this process directly.

## Tasks

### Part A: Create the Deployment

Create a file named `web-deployment.yaml` with the following specification:

- Deployment name: `web-server`
- 3 replicas
- Image: `nginx:1.25`
- Labels: `app: web-server`
- Container port: 80
- Resource requests: 100m CPU, 64Mi memory
- Resource limits: 200m CPU, 128Mi memory
- Rolling update strategy with `maxSurge: 1` and `maxUnavailable: 0`
- Readiness probe: HTTP GET on port 80, path `/`, initial delay 5 seconds, period 3 seconds
- Liveness probe: HTTP GET on port 80, path `/`, initial delay 10 seconds, period 5 seconds

<details>
<summary>Hint -- maxUnavailable: 0</summary>
Setting `maxUnavailable: 0` means Kubernetes will never reduce the number of available Pods below the desired count during an update. Combined with `maxSurge: 1`, this means Kubernetes will first create one extra Pod, wait for it to be ready, then remove one old Pod. This is the safest rolling update configuration for maintaining full capacity.
</details>

Apply the Deployment:

```bash
kubectl apply -f web-deployment.yaml
```

### Part B: Verify the Deployment

Run the following commands and record the output:

```bash
# Check the Deployment
kubectl get deployment web-server

# Check the ReplicaSet created
kubectl get replicasets -l app=web-server

# Check the Pods created
kubectl get pods -l app=web-server
```

Answer these questions:

1. How many ReplicaSets exist? What is the name pattern?
2. How many Pods are running? What is the status of each?
3. What labels do the Pods have? Where did those labels come from?

<details>
<summary>Hint -- ReplicaSet Naming</summary>
The ReplicaSet name follows the pattern `<deployment-name>-<pod-template-hash>`. The hash is derived from the pod template spec. All Pods created by this ReplicaSet will have the same hash in their names.
</details>

### Part C: Perform a Rolling Update

Update the image to `nginx:1.26` using the `kubectl set image` command:

```bash
kubectl set image deployment web-server nginx=nginx:1.26
```

Immediately run the following command to watch the rollout:

```bash
kubectl rollout status deployment web-server
```

Then inspect the state:

```bash
# Check ReplicaSets -- you should see two now
kubectl get replicasets -l app=web-server

# Check Pods -- some may be from the new ReplicaSet
kubectl get pods -l app=web-server -o wide

# Check the rollout history
kubectl rollout history deployment web-server
```

Answer these questions:

1. How many ReplicaSets exist now? Which one has active Pods?
2. During the rollout, what was the maximum number of Pods running at any point? Why?
3. Were there any moments when zero Pods were unavailable? Why or why not?

<details>
<summary>Hint -- Rolling Update Sequence</summary>
With `maxSurge: 1` and `maxUnavailable: 0` and 3 replicas, the sequence is: create 1 new Pod (4 total), wait for it to be ready, delete 1 old Pod (3 total), create 1 new Pod (4 total), wait, delete 1 old, create 1 new, wait, delete 1 old. At no point do fewer than 3 Pods serve traffic.
</details>

### Part D: Verify the Final State

After the rollout completes, verify:

```bash
# All Pods should be running the new image
kubectl get pods -l app=web-server -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.spec.containers[0].image}{"\n"}{end}'

# The old ReplicaSet should have 0 replicas
kubectl get replicasets -l app=web-server

# The deployment should be available
kubectl describe deployment web-server | grep -A 5 "Conditions"
```

Answer these questions:

1. What image is each Pod running?
2. What is the replica count of the old ReplicaSet? Why is it still around?
3. What does the `Conditions` section of the Deployment describe?

### Part E: Clean Up

```bash
kubectl delete deployment web-server
```

Verify that the ReplicaSets and Pods are also deleted:

```bash
kubectl get replicasets -l app=web-server
kubectl get pods -l app=web-server
```

Why are the ReplicaSets and Pods gone even though you only deleted the Deployment?

<details>
<summary>Hint -- Cascading Deletion</summary>
When you delete a Deployment, Kubernetes deletes the ReplicaSets it owns, and when a ReplicaSet is deleted, the Pods it owns are also deleted. This is cascading deletion via owner references.
</details>

## Success Criteria

- [ ] Your Deployment YAML is valid and creates 3 running Pods
- [ ] You can identify the ReplicaSet and its relationship to the Deployment
- [ ] You performed a rolling update and observed the old and new ReplicaSets
- [ ] You can explain why `maxSurge: 1` and `maxUnavailable: 0` maintains full capacity
- [ ] You verified that old ReplicaSets persist after the update but are cleaned up when the Deployment is deleted
- [ ] You understand cascading deletion

## What You Should Understand After This Exercise

After completing this exercise, you should be able to create a Deployment with proper resource limits and probes, perform a rolling update, and observe how Kubernetes manages the transition between old and new ReplicaSets. You should understand why `maxUnavailable: 0` is the safest setting for production workloads and why old ReplicaSets are kept after an update completes.
