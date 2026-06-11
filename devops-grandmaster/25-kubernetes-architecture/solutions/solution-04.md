# Solution 04: Diagnose a Broken Cluster

## Part A: Information Gathering

### 1. Pod Description

```bash
kubectl describe pod web-app-6d4f5b8c9-abc12 -n production
```

Key output in the `Events` section at the bottom:

```
Type     Reason            Age   From               Message
----     ------            ----  ----               -------
Warning  FailedScheduling  15m   default-scheduler  0/1 nodes are available:
  1 node(s) had taint {broken: true}, that the pod didn't tolerate.
```

The pod status shows:
```
Status:           Pending
Node:             <none>
```

### 2. Events

```bash
kubectl get events -n production --sort-by='.lastTimestamp'
```

Output:
```
LAST SEEN   TYPE      REASON             OBJECT                       MESSAGE
15m         Warning   FailedScheduling   pod/web-app-6d4f5b8c9-abc12  0/1 nodes are available...
15m         Warning   FailedScheduling   pod/web-app-6d4f5b8c9-def34  0/1 nodes are available...
15m         Warning   FailedScheduling   pod/web-app-6d4f5b8c9-ghi56  0/1 nodes are available...
15m         Normal    ScalingReplicaSet  deployment/web-app            Scaled up replica set...
15m         Normal    SuccessfulCreate   replicaset/web-app-6d4f5b8c9  Created pod...
```

### 3. Node Status

```bash
kubectl get nodes
```

Output:
```
NAME                 STATUS   ROLES           AGE   VERSION
kind-control-plane   Ready    control-plane   30m   v1.28.0
```

```bash
kubectl describe node kind-control-plane | grep -A 5 "Taints"
```

Output:
```
Taints:             broken=true:NoSchedule
```

### 4. Scheduler Logs

```bash
kubectl logs kube-scheduler -n kube-system --tail=50
```

Relevant lines:
```
I0115 10:30:00.123456  1 scheduler.go:456] "Unable to schedule pod" pod="production/web-app-6d4f5b8c9-abc12" err="0/1 nodes are available: 1 node(s) had taint {broken: true}, that the pod didn't tolerate."
```

### 5. Controller Manager Logs

```bash
kubectl logs kube-controller-manager -n kube-system --tail=50
```

The controller manager shows the ReplicaSet controller creating pods
but not finding issues -- it trusts the scheduler to handle placement.

## Part B: Root Cause Identification

### Pod Status Reason

The pod status is `Pending` with no `Node` assigned. The `Events` section
shows `FailedScheduling` with the message:

```
0/1 nodes are available: 1 node(s) had taint {broken: true}, that the pod didn't tolerate.
```

### What the Events Section Shows

Every few seconds, the scheduler attempts to schedule the pod and fails
with the same message. This creates repeated `FailedScheduling` events.

### Which Component Is Responsible

The **scheduler** is responsible for the failure, but it is not broken.
The scheduler is correctly refusing to place the pod on a tainted node.
The root cause is the **taint** on the node, which was added externally
(either by an administrator or an automated process).

### The Specific Condition

The node has a taint `broken=true:NoSchedule`. This taint tells the
scheduler: "Do not schedule any pods on this node unless they explicitly
tolerate this taint." Since the deployment's pod template does not define
any tolerations, the scheduler cannot place the pods anywhere.

## Part C: Fix the Problem

### Fix 1: Remove the Taint

```bash
kubectl taint nodes kind-control-plane broken=true:NoSchedule-
```

The trailing `-` removes the taint. After this, the scheduler can place
pods on the node.

### Fix 2: Add a Toleration to the Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-app
  namespace: production
spec:
  replicas: 3
  selector:
    matchLabels:
      app: web
  template:
    metadata:
      labels:
        app: web
    spec:
      tolerations:
      - key: "broken"
        operator: "Equal"
        value: "true"
        effect: "NoSchedule"
      containers:
      - name: nginx
        image: nginx:latest
        resources:
          requests:
            memory: "256Mi"
            cpu: "500m"
          limits:
            memory: "512Mi"
            cpu: "1000m"
```

### Fix 3: Add More Nodes

```bash
# Add a new worker node that does not have the taint
# For kind:
kind add worker --name lab-cluster

# For a real cluster:
kubeadm join <control-plane-ip>:6443 --token <token> --discovery-token-ca-cert-hash <hash>
```

### Trade-off Analysis

| Fix | Pros | Cons |
|-----|------|------|
| Remove taint | Simplest, immediate effect | May schedule on a broken node |
| Add toleration | Targeted, only affects this deployment | Masks the underlying node issue |
| Add nodes | Proper fix, maintains node isolation | Takes time, costs more resources |

### Recommendation

Remove the taint if the node is healthy. If the node has a real problem
(hardware failure, disk issues), add a new node and keep the taint until
the old node is repaired.

## Part D: Verify the Fix

```bash
# 1. Watch pods transition to Running
kubectl get pods -n production -w
```

Expected output:
```
NAME                       READY   STATUS    RESTARTS   AGE
web-app-6d4f5b8c9-abc12   1/1     Running   0          16m
web-app-6d4f5b8c9-def34   1/1     Running   0          16m
web-app-6d4f5b8c9-ghi56   1/1     Running   0          16m
```

```bash
# 2. Check events show Scheduled and Started
kubectl get events -n production --sort-by='.lastTimestamp' | tail -10
```

Expected output includes:
```
Normal  Scheduled  ...  Successfully assigned production/web-app-... to kind-control-plane
Normal  Pulling    ...  Pulling image "nginx:latest"
Normal  Pulled     ...  Successfully pulled image "nginx:latest"
Normal  Created    ...  Created container nginx
Normal  Started    ...  Started container nginx
```

```bash
# 3. Verify application is accessible
kubectl port-forward svc/web-service -n production 8080:80
curl http://localhost:8080
```

Expected: HTML response from nginx.

## Part E: Prevention Script

```bash
#!/bin/bash
# check-pending-pods.sh -- Detect pods stuck in Pending state

THRESHOLD_SECONDS=300  # 5 minutes
PENDING_PODS=$(kubectl get pods -A --field-selector status.phase=Pending -o json)

COUNT=$(echo "$PENDING_PODS" | jq -r '[.items[] | select(
  (now - (.metadata.creationTimestamp | fromdateiso8601)) > '$THRESHOLD_SECONDS'
)] | length')

if [ "$COUNT" -gt 0 ]; then
  echo "WARNING: $COUNT pod(s) stuck in Pending for more than $THRESHOLD_SECONDS seconds:"
  echo "$PENDING_PODS" | jq -r '.items[] | select(
    (now - (.metadata.creationTimestamp | fromdateiso8601)) > '$THRESHOLD_SECONDS'
  ) | "  - \(.metadata.namespace)/\(.metadata.name) (since \(.metadata.creationTimestamp))"'

  # Get the reason for each stuck pod
  echo ""
  echo "Scheduling failure reasons:"
  for POD in $(echo "$PENDING_PODS" | jq -r '.items[] | select(
    (now - (.metadata.creationTimestamp | fromdateiso8601)) > '$THRESHOLD_SECONDS'
  ) | "\(.metadata.namespace)/\(.metadata.name)"'); do
    NAMESPACE=$(echo "$POD" | cut -d'/' -f1)
    NAME=$(echo "$POD" | cut -d'/' -f2)
    REASON=$(kubectl describe pod "$NAME" -n "$NAMESPACE" 2>/dev/null | grep -A 2 "Events:" | tail -1)
    echo "  $POD: $REASON"
  done

  exit 1
else
  echo "OK: No pods stuck in Pending."
  exit 0
fi
```

### How to Use

```bash
chmod +x check-pending-pods.sh
./check-pending-pods.sh

# Add to cron for periodic checking
# */5 * * * * /path/to/check-pending-pods.sh >> /var/log/k8s-monitoring.log 2>&1
```

### Common Mistakes

- **Removing a taint without understanding why it was there.** Taints are
  often added for a reason (hardware failure, maintenance, security).
  Removing a taint without investigating can schedule pods on a broken node.
- **Only looking at pod status, not events.** `kubectl get pods` shows
  `Pending` but not *why*. Always use `kubectl describe pod` to see the
  events and scheduling decisions.
- **Not checking node taints.** Students often focus on resource limits
  and miss taints. `kubectl describe node | grep Taints` should be one
  of the first diagnostic commands.
- **Ignoring repeated FailedScheduling events.** Each event is the
  scheduler trying and failing. The message tells you exactly why.
  Read it carefully -- it often contains the exact taint, resource
  shortage, or constraint that is blocking scheduling.
- **Fixing symptoms instead of root cause.** Adding tolerations to pods
  masks the problem. If a node is tainted, find out why before deciding
  to schedule workloads on it.

## Relevant README Sections

- [Scheduler (kube-scheduler)](../README.md#scheduler-kube-scheduler) -- How the scheduler selects nodes
- [kubectl -- Your Interface to Kubernetes](../README.md#kubectl--your-interface-to-kubernetes) -- describe, get, logs commands
- [How a Pod Gets Created](../README.md#how-a-pod-gets-created) -- Where scheduling fits in the lifecycle
