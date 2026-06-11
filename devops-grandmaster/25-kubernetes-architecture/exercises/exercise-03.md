# Exercise 03: Trace a Pod Creation Request

**Type:** Independent
**Time:** 40 minutes
**Difficulty:** Medium

## Objective

Trace a Pod creation request through every component in the Kubernetes
cluster by observing real events, timestamps, and logs. This exercise
proves you understand the pod lifecycle by following it step by step.

## Scenario

Your team wants to understand exactly what happens when a Pod is created.
You will create a Pod, then use Kubernetes events and logs to reconstruct
the precise sequence of actions each component took.

## Prerequisites

A running Kubernetes cluster with kubectl access.

## Tasks

### Part A: Create a Pod and Capture Events

Create a simple Pod and immediately watch the events:

```bash
# Create the Pod
kubectl run trace-pod --image=nginx:latest --port=80

# Watch events in real-time (run this in a separate terminal)
kubectl get events --watch --sort-by='.lastTimestamp'
```

Record every event you see, in order. Each event has:
- A timestamp
- A reason (e.g., `Scheduled`, `Pulling`, `Pulled`, `Created`, `Started`)
- A message describing what happened
- An involved object

Create a table:

| Step | Timestamp | Reason | Component | Message |
|------|-----------|--------|-----------|---------|
| 1 | ? | ? | ? | ? |
| 2 | ? | ? | ? | ? |
| ... | | | | |

<details>
<summary>Hint</summary>

You should see events in roughly this order:
1. The scheduler assigns the pod to a node
2. The kubelet on that node pulls the container image
3. The kubelet creates the container
4. The container starts

Use `kubectl get events --field-selector involvedObject.name=trace-pod`
to filter events for just this pod.

</details>

### Part B: Examine the Pod's Scheduled Node

Once the pod is running, find out which node it was scheduled on and why:

```bash
# Which node?
kubectl get pod trace-pod -o wide

# Why this node?
kubectl describe pod trace-pod | grep -A 5 "Node:"
kubectl describe pod trace-pod | grep -A 10 "Conditions:"
```

Answer these questions:

1. Which node was the pod scheduled to?
2. Was there more than one node available? (Check with `kubectl get nodes`)
3. What criteria did the scheduler likely use to pick this node?

<details>
<summary>Hint</summary>

With a single-node cluster (kind or minikube), the choice is obvious --
there is only one node. If you have multiple nodes, the scheduler
considers resource availability, taints, tolerations, affinity rules,
and more. Use `kubectl describe node <name>` to see available resources.

</details>

### Part C: Inspect the Pod's Status Details

Get the full pod specification and status:

```bash
kubectl get pod trace-pod -o yaml
```

Find and record these fields:

1. `status.phase` -- What phase is the pod in?
2. `status.conditions` -- What conditions are true?
3. `status.hostIP` -- What is the node's IP?
4. `status.podIP` -- What is the pod's IP?
5. `status.containerStatuses` -- What is the container's state?
6. `metadata.ownerReferences` -- Who owns this pod?

<details>
<summary>Hint</summary>

The `phase` should be `Running`. The conditions should include
`Initialized`, `Ready`, `ContainersReady`, and `PodScheduled` -- all
`True`.

The `ownerReferences` field is interesting -- when you use
`kubectl run`, it creates a Pod directly, so there may be no owner.
Compare this to a Pod created by a Deployment, which would show a
ReplicaSet as the owner.

</details>

### Part D: Check the kubelet's View

The kubelet is the component that actually runs the pod. Check its logs:

```bash
# Find the kubelet process (if using kind, exec into the node)
# For kind:
docker exec lab-cluster-control-plane journalctl -u kubelet --no-pager | grep "trace-pod" | tail -20

# For minikube:
minikube ssh -- journalctl -u kubelet --no-pager | grep "trace-pod" | tail -20

# For a real cluster:
sudo journalctl -u kubelet --no-pager | grep "trace-pod" | tail -20
```

What do the kubelet logs tell you about how it handled this pod?

<details>
<summary>Hint</summary>

The kubelet logs will show:
- When it received the pod assignment from the API server
- When it told the container runtime to pull the image
- When it started the container
- Any health checks it configured

If you cannot access kubelet logs (common with managed clusters), skip
this step and rely on the events from Part A.

</details>

### Part E: Delete the Pod and Observe the Teardown

Delete the pod and watch the events:

```bash
kubectl delete pod trace-pod
kubectl get events --field-selector involvedObject.name=trace-pod --sort-by='.lastTimestamp'
```

Record the teardown events. What happens in what order?

<details>
<summary>Hint</summary>

The teardown sequence is:
1. API server receives the delete request
2. Pod status changes to `Terminating`
3. kubelet sends SIGTERM to the container
4. After the grace period (default 30s), kubelet sends SIGKILL
5. Container is removed
6. Pod is removed from etcd

</details>

### Part F: Compare Direct Pod vs Deployment

Create a pod via a Deployment and compare the events:

```bash
kubectl create deployment trace-deploy --image=nginx:latest --replicas=1
kubectl get events --sort-by='.lastTimestamp' | grep -i "trace-deploy"
```

What additional events do you see that were not present with the
direct `kubectl run` approach?

<details>
<summary>Hint</summary>

With a Deployment, you will see events from:
1. The Deployment controller creating a ReplicaSet
2. The ReplicaSet controller creating a Pod
3. The scheduler assigning the pod
4. The kubelet pulling and starting the container

This shows the controller manager's role -- it watches for the
Deployment object and creates the ReplicaSet, which in turn creates
the Pod.

</details>

## Success Criteria

- [ ] You recorded at least 4 events in the correct chronological order
- [ ] You can identify which component generated each event
- [ ] You know which node the pod was scheduled on and why
- [ ] You found the pod's phase, conditions, and IP addresses
- [ ] You observed the pod teardown sequence
- [ ] You can explain the difference between a direct pod and a deployment-managed pod

## What You Should Understand After This Exercise

Pod creation is a multi-step process involving multiple components.
The API server receives the request and stores it in etcd. The scheduler
watches for unscheduled pods and assigns them to nodes. The kubelet
on the assigned node pulls the image and starts the container. Each
step generates events that you can observe. When you use a Deployment,
additional controller manager steps create the ReplicaSet and Pod
objects before the scheduler even sees them.
