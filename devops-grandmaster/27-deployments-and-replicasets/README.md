# Module 27: Deployments & ReplicaSets — Declarative State Management

**Previous:** [Module 26: Pods & Containers](../26-pods-and-containers/README.md)

---

## The Problem

You have learned how to create pods with YAML. Now imagine running a production application:

- You need 5 replicas of your web server for high availability
- If a pod crashes, a new one must be created automatically
- When you release a new version, you want zero-downtime rolling updates
- If a deployment goes bad, you need to rollback instantly
- You need to scale from 5 to 20 replicas during peak traffic

Managing individual pod YAML files for all of this is impossible. You need a controller that maintains your **desired state** declaratively.

---

## The Naive Way

Create individual pod YAML files and manage them manually:

```bash
# Create 5 pods manually
for i in 1 2 3 4 5; do
  kubectl apply -f pod-$i.yaml
done

# When pod-3 crashes, manually recreate it
kubectl delete pod pod-3
kubectl apply -f pod-3.yaml

# To update the image, delete all pods and recreate them
kubectl delete pods -l app=myapp
# Update image tag in each YAML
kubectl apply -f pod-1.yaml pod-2.yaml pod-3.yaml pod-4.yaml pod-5.yaml
```

**What goes wrong:**

- Downtime during updates (all pods deleted before new ones start)
- No automatic self-healing if you don't notice a crash
- Scaling requires creating or deleting YAML files
- No rollback capability
- Race conditions when updating multiple pods simultaneously

---

## The Right Way

### ReplicaSet — Maintaining Pod Count

A **ReplicaSet** ensures a specified number of pod replicas are running at any time.

```yaml
# replicaset.yaml
apiVersion: apps/v1
kind: ReplicaSet
metadata:
  name: nginx-rs
spec:
  replicas: 3
  selector:
    matchLabels:
      app: nginx
  template:
    metadata:
      labels:
        app: nginx
    spec:
      containers:
        - name: nginx
          image: nginx:1.25
          ports:
            - containerPort: 80
          resources:
            requests:
              cpu: "100m"
              memory: "64Mi"
            limits:
              cpu: "200m"
              memory: "128Mi"
```

**How ReplicaSet works:**

1. You declare `replicas: 3`
2. ReplicaSet watches for pods matching `selector.matchLabels`
3. If fewer than 3 pods exist, it creates new ones
4. If more than 3 pods exist, it deletes excess ones
5. If a pod crashes, it detects the count dropped and creates a replacement

**You rarely create ReplicaSets directly.** Deployments manage them for you.

### Deployment — Managing ReplicaSets

A **Deployment** is a higher-level controller that manages ReplicaSets and provides:

- Declarative updates (change the YAML, apply it)
- Rolling updates (zero-downtime)
- Rollback (undo a bad update)
- Pause and resume (batch multiple changes)

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nginx-deployment
  labels:
    app: nginx
spec:
  replicas: 3
  selector:
    matchLabels:
      app: nginx
  template:
    metadata:
      labels:
        app: nginx
    spec:
      containers:
        - name: nginx
          image: nginx:1.25
          ports:
            - containerPort: 80
          resources:
            requests:
              cpu: "100m"
              memory: "64Mi"
            limits:
              cpu: "200m"
              memory: "128Mi"
          livenessProbe:
            httpGet:
              path: /
              port: 80
            initialDelaySeconds: 10
            periodSeconds: 5
          readinessProbe:
            httpGet:
              path: /
              port: 80
            initialDelaySeconds: 5
            periodSeconds: 3
```

**The relationship:**

```
Deployment (nginx-deployment)
  └── ReplicaSet (nginx-deployment-7d8b49557c)
        ├── Pod (nginx-deployment-7d8b49557c-abc12)
        ├── Pod (nginx-deployment-7d8b49557c-def34)
        └── Pod (nginx-deployment-7d8b49557c-ghi56)
```

When you update the deployment (e.g., change the image), Kubernetes creates a **new** ReplicaSet and gradually shifts traffic from the old ReplicaSet to the new one.

### Rolling Updates

The default update strategy is `RollingUpdate`:

```yaml
spec:
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1        # Max extra pods above desired count
      maxUnavailable: 1   # Max pods that can be unavailable during update
```

**Example with `replicas: 3`, `maxSurge: 1`, `maxUnavailable: 1`:**

```
Step 1: 3 old pods running
Step 2: Create 1 new pod (total: 4 pods, 3 old + 1 new)
Step 3: Wait for new pod to be ready
Step 4: Delete 1 old pod (total: 3 pods, 2 old + 1 new)
Step 5: Create 1 new pod (total: 4 pods, 2 old + 1 new)
Step 6: Wait for new pod to be ready
Step 7: Delete 1 old pod (total: 3 pods, 1 old + 2 new)
Step 8: Create 1 new pod (total: 4 pods, 1 old + 3 new)
Step 9: Wait for new pod to be ready
Step 10: Delete 1 old pod (total: 3 pods, 0 old + 3 new)
```

**The formula:**
- `maxSurge + maxUnavailable <= replicas` (recommended, not enforced)
- Higher `maxSurge` = faster rollout but more resource usage
- Lower `maxUnavailable` = safer but slower rollout

### Recreate Strategy

```yaml
spec:
  strategy:
    type: Recreate
```

**Behavior:** All existing pods are killed before new ones are created. This causes downtime but ensures no two versions run simultaneously. Use this when your application cannot run two versions at the same time (e.g., database schema migrations that break backward compatibility).

### Rollback

Kubernetes maintains deployment history:

```bash
# View deployment history
kubectl rollout history deployment nginx-deployment

# View details of a specific revision
kubectl rollout history deployment nginx-deployment --revision=2

# Rollback to the previous revision
kubectl rollout undo deployment nginx-deployment

# Rollback to a specific revision
kubectl rollout undo deployment nginx-deployment --to-revision=1
```

**How rollback works:** Kubernetes simply points the deployment back to the old ReplicaSet. The old ReplicaSet already exists with its pod template intact. Kubernetes performs a rolling update using the old template.

### Deployment Status

```bash
# Check deployment status
kubectl rollout status deployment nginx-deployment

# Describe deployment for detailed conditions
kubectl describe deployment nginx-deployment
```

**Deployment conditions:**
- `Available` — minimum required replicas are available
- `Progressing` — deployment is in progress (update, scale)
- `ReplicaFailure` — pod creation failed

---

## The Production Way

### Deployment Strategy Recommendations

```yaml
# For stateless web services (zero downtime)
spec:
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: "25%"         # 25% extra pods during rollout
      maxUnavailable: "25%"   # 25% can be unavailable

# For critical services (safer, slower)
spec:
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0       # Always maintain full capacity

# For stateful applications with breaking changes
spec:
  strategy:
    type: Recreate
```

### Min Ready Seconds

```yaml
spec:
  minReadySeconds: 30  # Pod must be ready for 30s before marking as available
```

This prevents Kubernetes from marking a pod as available immediately after it starts, giving your application time to warm up (load caches, establish connections, etc.).

### Progress Deadline

```yaml
spec:
  progressDeadlineSeconds: 600  # 10 minutes
```

If the deployment doesn't make progress within this time, Kubernetes marks it as failed. This prevents infinite rollback loops when a deployment is stuck.

### Revision History Limit

```yaml
spec:
  revisionHistoryLimit: 10  # Keep the last 10 ReplicaSets for rollback
```

Default is 10. Set to 0 to disable rollback (saves resources).

### Scaling

```bash
# Manual scaling
kubectl scale deployment nginx-deployment --replicas=5

# Scale with condition (only if current replicas is 3)
kubectl scale deployment nginx-deployment --replicas=5 --current-replicas=3

# Autoscaling (requires metrics-server)
kubectl autoscale deployment nginx-deployment \
  --min=2 \
  --max=10 \
  --cpu-percent=80
```

### Canary Deployments

Run two versions simultaneously with controlled traffic split:

```yaml
# Stable version (90% traffic)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: app-stable
spec:
  replicas: 9
  selector:
    matchLabels:
      app: myapp
      track: stable
  template:
    metadata:
      labels:
        app: myapp
        track: stable
    spec:
      containers:
        - name: app
          image: myapp:v1

---
# Canary version (10% traffic)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: app-canary
spec:
  replicas: 1
  selector:
    matchLabels:
      app: myapp
      track: canary
  template:
    metadata:
      labels:
        app: myapp
        track: canary
    spec:
      containers:
        - name: app
          image: myapp:v2
```

Both deployments are selected by the same Service (label `app: myapp`), so traffic is distributed proportionally: 9/10 to stable, 1/10 to canary.

### Blue-Green Deployments

Run two complete environments and switch traffic:

```yaml
# Blue (current)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: app-blue
spec:
  replicas: 3
  selector:
    matchLabels:
      app: myapp
      version: blue
  template:
    metadata:
      labels:
        app: myapp
        version: blue
    spec:
      containers:
        - name: app
          image: myapp:v1

---
# Green (new)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: app-green
spec:
  replicas: 3
  selector:
    matchLabels:
      app: myapp
      version: green
  template:
    metadata:
      labels:
        app: myapp
        version: green
    spec:
      containers:
        - name: app
          image: myapp:v2
```

**Switch traffic** by updating the Service selector from `version: blue` to `version: green`.

---

## Hands-On Lab

### Exercise 1: Create a Deployment

```bash
kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nginx-deployment
spec:
  replicas: 3
  selector:
    matchLabels:
      app: nginx
  template:
    metadata:
      labels:
        app: nginx
    spec:
      containers:
        - name: nginx
          image: nginx:1.25
          ports:
            - containerPort: 80
          resources:
            requests:
              cpu: "100m"
              memory: "64Mi"
            limits:
              cpu: "200m"
              memory: "128Mi"
EOF

# Watch the deployment
kubectl get deployments -w

# Check the ReplicaSet created
kubectl get replicasets

# Check the pods
kubectl get pods -l app=nginx
```

### Exercise 2: Scale the Deployment

```bash
# Scale to 5 replicas
kubectl scale deployment nginx-deployment --replicas=5

# Watch the new pods come up
kubectl get pods -l app=nginx -w

# Verify the deployment
kubectl describe deployment nginx-deployment | grep -A 3 "Conditions"
```

### Exercise 3: Rolling Update

```bash
# Update the image version
kubectl set image deployment nginx-deployment nginx=nginx:1.26

# Watch the rollout
kubectl rollout status deployment nginx-deployment

# Check the history
kubectl rollout history deployment nginx-deployment

# Verify the new ReplicaSet
kubectl get replicasets
```

### Exercise 4: Rollback

```bash
# Simulate a bad update
kubectl set image deployment nginx-deployment nginx=nginx:bad-tag

# Watch the deployment fail
kubectl rollout status deployment nginx-deployment

# Check the events
kubectl get events --field-selector reason=Failed

# Rollback
kubectl rollout undo deployment nginx-deployment

# Verify the rollback
kubectl rollout status deployment nginx-deployment
kubectl get pods -l app=nginx -o wide
```

### Exercise 5: Deployment Strategies

```bash
# Apply a deployment with RollingUpdate strategy
kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-app
spec:
  replicas: 4
  selector:
    matchLabels:
      app: web
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  template:
    metadata:
      labels:
        app: web
    spec:
      containers:
        - name: web
          image: nginx:1.25
          ports:
            - containerPort: 80
          readinessProbe:
            httpGet:
              path: /
              port: 80
            initialDelaySeconds: 5
            periodSeconds: 3
EOF

# Update and watch the rolling update
kubectl set image deployment web-app web=nginx:1.26

# In another terminal, watch the pods
kubectl get pods -l app=web -w

# Verify no downtime during the update
# (In production, you would run continuous requests to verify)
```

### Exercise 6: Inspect Deployment Internals

```bash
# View the deployment YAML with live state
kubectl get deployment nginx-deployment -o yaml

# View the ReplicaSet
kubectl get replicasets -l app=nginx

# View the pods with node assignment
kubectl get pods -l app=nginx -o wide

# Check events for the deployment
kubectl get events --field-selector involvedObject.name=nginx-deployment

# Clean up
kubectl delete deployment nginx-deployment web-app
```

---

## Verification Checklist

- [ ] Understand why you should never manage pods directly in production
- [ ] Can create a Deployment with proper resource requests and limits
- [ ] Understand the Deployment → ReplicaSet → Pod hierarchy
- [ ] Can perform rolling updates and rollbacks
- [ ] Know the difference between RollingUpdate and Recreate strategies
- [ ] Can configure maxSurge and maxUnavailable
- [ ] Can scale deployments manually

---

## Limitation

You have learned how to manage pods declaratively with Deployments and ReplicaSets. But pods have ephemeral IP addresses. When a pod is replaced, its IP changes. How do other pods or external clients find your application? You cannot hardcode pod IPs — they change on every restart.

**Next:** [Module 28: Services & Networking](../28-services-and-networking/README.md) — Kubernetes Services provide stable endpoints for pods.
