# Cheatsheet: Deployments & ReplicaSets

## Deployment Spec
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app
spec:
  replicas: 3
  selector:
    matchLabels:
      app: my-app
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  template:
    metadata:
      labels:
        app: my-app
    spec:
      containers:
        - name: my-app
          image: my-app:1.0.0
```

## Deployment Strategies

| Strategy | Description | Downtime |
|----------|-------------|----------|
| RollingUpdate | Gradually replace pods | Zero |
| Recreate | Kill all, then create new | Yes |

## Common Commands
```bash
kubectl apply -f deployment.yaml       # Create/Update
kubectl get deployments                # List
kubectl describe deployment my-app     # Details
kubectl scale deployment my-app --replicas=5  # Scale
kubectl rollout status deployment my-app      # Status
kubectl rollout history deployment my-app     # History
kubectl rollout undo deployment my-app        # Rollback
kubectl rollout undo deployment my-app --to-revision=2  # Rollback to specific
kubectl set image deployment/my-app my-app=my-app:2.0.0  # Update image
```

## Rollback Workflow
```bash
# 1. Update image
kubectl set image deployment/my-app my-app=my-app:2.0.0

# 2. Watch rollout
kubectl rollout status deployment/my-app

# 3. If problems, rollback
kubectl rollout undo deployment/my-app

# 4. Check history
kubectl rollout history deployment/my-app
```
