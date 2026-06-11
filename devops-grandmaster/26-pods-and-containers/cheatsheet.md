# Cheatsheet: Pods & Containers

## Pod Spec (Minimal)
```yaml
apiVersion: v1
kind: Pod
metadata:
  name: my-pod
  labels:
    app: my-app
spec:
  containers:
    - name: my-app
      image: my-app:1.0
      ports:
        - containerPort: 8080
```

## Resource Requests & Limits
```yaml
resources:
  requests:
    cpu: 100m      # 0.1 CPU cores (guaranteed)
    memory: 128Mi   # 128 MB (guaranteed)
  limits:
    cpu: 500m      # 0.5 CPU max (throttled)
    memory: 256Mi   # 256 MB max (OOMKilled if exceeded)
```

## Probes
```yaml
livenessProbe:     # Is it alive? If not → restart
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 15
  periodSeconds: 20

readinessProbe:    # Is it ready? If not → no traffic
  httpGet:
    path: /ready
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 10
```

## Init Containers
```yaml
spec:
  initContainers:
    - name: init
      image: busybox
      command: ['sh', '-c', 'until nslookup db; do sleep 2; done']
  containers:
    - name: app
      image: my-app:1.0
```

## Pod Statuses

| Status | Meaning |
|--------|---------|
| Pending | Waiting (scheduling, image pull) |
| Running | At least one container running |
| Succeeded | All containers exited 0 |
| Failed | Container exited non-zero |
| Unknown | Node unreachable |

## Common kubectl Commands

```bash
kubectl get pods                    # List pods
kubectl get pods -o wide            # With node info
kubectl describe pod <name>         # Detailed info
kubectl logs <pod>                  # View logs
kubectl logs <pod> -c <container>   # Specific container
kubectl exec -it <pod> -- bash      # Shell access
kubectl delete pod <name>           # Delete pod
kubectl top pods                    # Resource usage
```
