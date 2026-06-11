# Solution 03: Trace a Pod Creation Request

## Part A: Events Timeline

Running `kubectl run trace-pod --image=nginx:latest --port=80` and
watching events produces this sequence:

| Step | Timestamp | Reason | Component | Message |
|------|-----------|--------|-----------|---------|
| 1 | T+0s | Scheduled | kube-scheduler | Successfully assigned default/trace-pod to kind-control-plane |
| 2 | T+1s | Pulling | kubelet | Pulling image "nginx:latest" |
| 3 | T+3s | Pulled | kubelet | Successfully pulled image "nginx:latest" |
| 4 | T+3s | Created | kubelet | Created container nginx |
| 5 | T+3s | Started | kubelet | Started container nginx |

### How to Read This Table

- **Step 1** is the scheduler's job. It watches the API server for pods
  without a `nodeName` and assigns them to a node.
- **Steps 2-5** are the kubelet's job. It watches the API server for pods
  assigned to its node and handles the entire container lifecycle.

### Filtering Events for This Pod

```bash
kubectl get events --field-selector involvedObject.name=trace-pod -o wide
```

This filters out events from other pods and namespaces, showing only
events related to `trace-pod`.

## Part B: Scheduled Node

### Which Node

```bash
kubectl get pod trace-pod -o wide
```

Output:
```
NAME        READY   STATUS    RESTARTS   AGE   IP           NODE                 NOMINATED NODE   READINESS GATES
trace-pod   1/1     Running   0          30s   10.244.0.5   kind-control-plane   <none>           <none>
```

The pod was scheduled to `kind-control-plane`.

### Why This Node

With kind (single-node cluster), there is only one node available.
The scheduler had no choice.

With a multi-node cluster, the scheduler considers:
1. **Resource availability** -- Does the node have enough CPU and memory?
2. **Taints and tolerations** -- Is the node tainted? Does the pod tolerate it?
3. **Node affinity** -- Does the pod have node selector or affinity rules?
4. **Pod affinity/anti-affinity** -- Should this pod be near or far from other pods?
5. **Topology spread** -- Should pods be spread across zones?

To check what resources are available on each node:
```bash
kubectl describe node <node-name> | grep -A 10 "Allocated resources"
```

## Part C: Pod Status Details

### Status Fields from `kubectl get pod trace-pod -o yaml`

```yaml
status:
  phase: Running
  conditions:
  - type: Initialized
    status: "True"
    lastProbeTime: null
    lastTransitionTime: "2024-01-15T10:30:00Z"
  - type: Ready
    status: "True"
    lastProbeTime: null
    lastTransitionTime: "2024-01-15T10:30:05Z"
  - type: ContainersReady
    status: "True"
    lastProbeTime: null
    lastTransitionTime: "2024-01-15T10:30:05Z"
  - type: PodScheduled
    status: "True"
    lastProbeTime: null
    lastTransitionTime: "2024-01-15T10:30:00Z"
  hostIP: 172.18.0.2
  podIP: 10.244.0.5
  containerStatuses:
  - name: nginx
    state:
      running:
        startedAt: "2024-01-15T10:30:05Z"
    ready: true
    restartCount: 0
    image: nginx:latest
    imageID: docker.io/library/nginx@sha256:abc123...
```

### Field Explanations

- **`phase: Running`** -- The pod has been bound to a node, all containers
  have been created, and at least one container is running.
- **`conditions`** -- Four conditions must all be `True` for the pod to
  be considered ready:
  - `Initialized` -- All init containers have completed
  - `PodScheduled` -- The pod has been assigned to a node
  - `ContainersReady` -- All containers are ready
  - `Ready` -- The pod is ready to serve traffic
- **`hostIP`** -- The IP address of the node where the pod is running
- **`podIP`** -- The IP address assigned to the pod within the cluster network
- **`containerStatuses`** -- The state of each container (running, waiting,
  or terminated)

### Owner References

When using `kubectl run`, the pod has no owner:
```yaml
metadata:
  ownerReferences: null
```

When a pod is created by a Deployment, it has:
```yaml
metadata:
  ownerReferences:
  - apiVersion: apps/v1
    kind: ReplicaSet
    name: web-app-6d4f5b8c9
    uid: abc123-def456
```

This chain (Deployment -> ReplicaSet -> Pod) is how Kubernetes knows
what to delete when you delete a Deployment.

## Part D: kubelet's View

### kubelet Logs for trace-pod

On a kind cluster:
```bash
docker exec kind-control-plane journalctl -u kubelet --no-pager | grep "trace-pod" | tail -20
```

Typical output:
```
Jan 15 10:30:00 kind-control-plane kubelet[1234]: I0115 10:30:00.123456  1234 config.go:414] "Received pod from API server" pod="default/trace-pod"
Jan 15 10:30:00 kind-control-plane kubelet[1234]: I0115 10:30:00.234567  1234 kuberuntime_manager.go:456] "Creating container in pod" containerName="nginx" pod="default/trace-pod"
Jan 15 10:30:01 kind-control-plane kubelet[1234]: I0115 10:30:01.345678  1234 kuberuntime_manager.go:567] "Pulling image" image="nginx:latest" pod="default/trace-pod"
Jan 15 10:30:03 kind-control-plane kubelet[1234]: I0115 10:30:03.456789  1234 kuberuntime_manager.go:678] "Container started" containerID="containerd://abc123" pod="default/trace-pod"
```

### What This Tells Us

The kubelet:
1. Receives the pod spec from the API server via its watch connection
2. Calls the container runtime (containerd) to pull the image
3. Calls the container runtime to create and start the container
4. Monitors the container and reports status back to the API server

The kubelet does not make scheduling decisions -- it only executes
pods assigned to its node.

## Part E: Pod Teardown Events

```bash
kubectl delete pod trace-pod
kubectl get events --field-selector involvedObject.name=trace-pod --sort-by='.lastTimestamp'
```

Teardown events:

| Step | Reason | Message |
|------|--------|---------|
| 1 | Killing | Stopping container nginx |
| 2 | Pulled | Container image already present on machine |

### Teardown Sequence

1. `kubectl delete pod` sends a DELETE request to the API server
2. API server sets the pod's `deletionTimestamp` field
3. kubelet sees the deletion timestamp
4. kubelet sends SIGTERM to the container's main process
5. The container has `terminationGracePeriodSeconds` (default 30s) to shut down gracefully
6. If the container does not exit within the grace period, kubelet sends SIGKILL
7. Container is removed by the container runtime
8. kubelet notifies the API server
9. API server removes the pod from etcd

### Why SIGTERM Then SIGKILL

SIGTERM is a polite request to shut down. The application can catch it,
finish in-flight requests, close database connections, and exit cleanly.
SIGKILL is a forced kill -- the process cannot catch or ignore it. The
grace period gives the application time to shut down gracefully before
being forcefully killed.

## Part F: Direct Pod vs Deployment

### Additional Events with a Deployment

```bash
kubectl create deployment trace-deploy --image=nginx:latest --replicas=1
kubectl get events --sort-by='.lastTimestamp' | grep "trace-deploy"
```

Events you will see (in order):

| Step | Reason | Involved Object | Message |
|------|--------|-----------------|---------|
| 1 | ScalingReplicaSet | Deployment/trace-deploy | Scaled up replica set trace-deploy-abc123 to 1 |
| 2 | SuccessfulCreate | ReplicaSet/trace-deploy-abc123 | Created pod: trace-deploy-abc123-xyz |
| 3 | Scheduled | Pod/trace-deploy-abc123-xyz | Successfully assigned default/trace-deploy-abc123-xyz to node |
| 4 | Pulling | Pod/trace-deploy-abc123-xyz | Pulling image "nginx:latest" |
| 5 | Pulled | Pod/trace-deploy-abc123-xyz | Successfully pulled image |
| 6 | Created | Pod/trace-deploy-abc123-xyz | Created container nginx |
| 7 | Started | Pod/trace-deploy-abc123-xyz | Started container nginx |

### The Difference

With `kubectl run`:
- The pod is created directly by the kubectl command
- The API server stores it in etcd
- The scheduler assigns it to a node
- No controller manager involvement

With `kubectl create deployment`:
- The Deployment object is created
- The **Deployment controller** (in the controller manager) sees the
  Deployment and creates a ReplicaSet
- The **ReplicaSet controller** (in the controller manager) sees the
  ReplicaSet and creates a Pod
- Then the scheduler and kubelet do their normal work

This shows the controller manager's role -- it watches for objects that
specify desired state and creates the necessary child objects to achieve
that state.

### Common Mistakes

- **Confusing the event order.** Students often think the kubelet pulls
  the image before the scheduler assigns the pod. The scheduler must
  assign first -- only then does the kubelet on the target node act.
- **Missing the controller manager events.** With Deployments, two
  additional events (ScalingReplicaSet and SuccessfulCreate) happen
  before the scheduler even sees the pod. These are from the controller
  manager.
- **Not understanding the watch mechanism.** Components do not poll
  continuously. They establish a long-lived watch connection to the API
  server and receive notifications when relevant objects change. This is
  efficient and real-time.
- **Thinking the kubelet decides where to run pods.** The kubelet only
  runs pods assigned to its node. The scheduler makes the assignment
  decision. The kubelet is an executor, not a decision-maker.

## Relevant README Sections

- [How a Pod Gets Created](../README.md#how-a-pod-gets-created) -- The 9-step lifecycle
- [Scheduler (kube-scheduler)](../README.md#scheduler-kube-scheduler) -- Node selection logic
- [Controller Manager (kube-controller-manager)](../README.md#controller-manager-kube-controller-manager) -- Reconciliation loops
- [kubelet](../README.md#kubelet) -- The agent on each node
