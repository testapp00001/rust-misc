# Exercise 04: Diagnose a Broken Cluster

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Diagnose a Kubernetes cluster issue by examining component logs, events,
and pod states. This exercise simulates a real production incident where
pods are not running and you must determine which component is failing.

## Scenario

You arrive at work on Monday morning. The monitoring dashboard is red.
Several pods are stuck in `Pending` state and users are reporting that
the application is unreachable. Your job is to figure out what is wrong.

The following YAML files describe the current state of the cluster:

```yaml
# The application deployment
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

```yaml
# The service
apiVersion: v1
kind: Service
metadata:
  name: web-service
  namespace: production
spec:
  selector:
    app: web
  ports:
  - port: 80
    targetPort: 80
  type: ClusterIP
```

You run `kubectl get pods -n production` and see:

```
NAME                       READY   STATUS    RESTARTS   AGE
web-app-6d4f5b8c9-abc12   0/1     Pending   0          15m
web-app-6d4f5b8c9-def34   0/1     Pending   0          15m
web-app-6d4f5b8c9-ghi56   0/1     Pending   0          15m
```

## Tasks

### Part A: Gather Information

For each of the following commands, run it and record the output.
If you do not have a broken cluster, simulate the scenario using
the instructions in the hint below.

```bash
# 1. Check pod details
kubectl describe pod web-app-6d4f5b8c9-abc12 -n production

# 2. Check events
kubectl get events -n production --sort-by='.lastTimestamp'

# 3. Check node status
kubectl get nodes
kubectl describe nodes

# 4. Check scheduler logs
kubectl logs kube-scheduler -n kube-system --tail=50

# 5. Check controller manager logs
kubectl logs kube-controller-manager -n kube-system --tail=50
```

<details>
<summary>Hint</summary>

To simulate this scenario on a local cluster, you can taint the node
to repel pods:

```bash
# Create the namespace
kubectl create namespace production

# Apply the deployment
kubectl apply -f - <<EOF
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
EOF

# Simulate a node problem by adding a taint
kubectl taint nodes <node-name> broken=true:NoSchedule
```

This will cause pods to stay in `Pending` because no node accepts them.

</details>

### Part B: Identify the Root Cause

Based on the information you gathered, answer these questions:

1. What is the pod's `Status` reason? (Look at `kubectl describe pod` output)
2. What does the `Events` section at the bottom of `describe pod` show?
3. Is the issue with the scheduler, the kubelet, the API server, or something else?
4. What specific condition is preventing the pod from being scheduled?

<details>
<summary>Hint</summary>

Look for these common issues in the `describe pod` output:
- `FailedScheduling` with message about taints and tolerations
- `FailedScheduling` with message about insufficient CPU or memory
- `FailedScheduling` with message about node selector or affinity
- `Unschedulable` in the node conditions

The Events section at the bottom of `kubectl describe pod` is the most
important diagnostic output. It tells you exactly what the scheduler
decided and why.

</details>

### Part C: Fix the Problem

Write the commands to fix the issue and get the pods running. There
may be more than one valid fix -- list at least two approaches.

<details>
<summary>Hint</summary>

If the issue is a taint:
```bash
# Fix 1: Remove the taint
kubectl taint nodes <node-name> broken=true:NoSchedule-

# Fix 2: Add a toleration to the deployment
```

If the issue is insufficient resources:
```bash
# Fix 1: Reduce resource requests
# Fix 2: Add more nodes to the cluster
# Fix 3: Remove other workloads to free resources
```

</details>

### Part D: Verify the Fix

After applying your fix, verify that:

1. The pods transition from `Pending` to `Running`
2. The events show `Scheduled` and `Started` events
3. The application is accessible

Write the commands you would run to verify each of these.

<details>
<summary>Hint</summary>

```bash
# Check pod status
kubectl get pods -n production -w

# Check events
kubectl get events -n production --sort-by='.lastTimestamp'

# Check application
kubectl port-forward svc/web-service -n production 8080:80
curl http://localhost:8080
```

</details>

### Part E: Prevention

Write a monitoring command or script that would detect this problem
automatically in the future. The script should:

1. Check for pods stuck in `Pending` for more than 5 minutes
2. Output a warning with the pod name and reason
3. Exit with a non-zero status if any pods are stuck

<details>
<summary>Hint</summary>

```bash
# Find pods stuck in Pending
kubectl get pods -A --field-selector status.phase=Pending -o json

# Parse with jq to find pods pending for more than 5 minutes
# Check the metadata.creationTimestamp field
```

A simple approach:
```bash
kubectl get pods -A --field-selector status.phase=Pending -o json | \
  jq -r '.items[] | select(
    (now - (.metadata.creationTimestamp | fromdateiso8601)) > 300
  ) | "\(.metadata.namespace)/\(.metadata.name) pending since \(.metadata.creationTimestamp)"'
```

</details>

## Success Criteria

- [ ] You gathered diagnostic information from at least 4 different sources
- [ ] You identified the root cause of the scheduling failure
- [ ] You proposed at least 2 different fixes with trade-off analysis
- [ ] You verified the fix brought pods to Running state
- [ ] You wrote a monitoring script that detects stuck pods
- [ ] You can explain which component was responsible for the failure

## What You Should Understand After This Exercise

When pods are stuck in Pending, the problem is almost always at the
scheduling level. The scheduler cannot find a suitable node. The causes
range from taints and tolerations to insufficient resources to node
affinity rules. The `kubectl describe pod` output is your primary
diagnostic tool -- the Events section tells you exactly what the
scheduler decided and why. Understanding the component responsible
for each failure mode lets you diagnose issues quickly in production.
