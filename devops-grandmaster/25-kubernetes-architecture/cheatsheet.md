# Cheatsheet: Kubernetes Architecture

## Control Plane Components

| Component | Role |
|-----------|------|
| API Server | Front door — all kubectl commands go here |
| etcd | State store — all cluster data |
| Scheduler | Decides which node runs each pod |
| Controller Manager | Reconciliation loops — makes reality match desired state |

## Worker Node Components

| Component | Role |
|-----------|------|
| kubelet | Agent on each node — runs pods, reports status |
| kube-proxy | Network rules — implements Services |
| Container Runtime | Actually runs containers (containerd, CRI-O) |

## Pod Lifecycle

```
Pending → Running → Succeeded/Failed
           ↓
       Unknown (node lost connection)
```

## kubectl Essentials

```bash
# Get resources
kubectl get pods/svc/deploy/nodes

# Describe (detailed info)
kubectl describe <resource> <name>

# Create/Update
kubectl apply -f resource.yaml

# Delete
kubectl delete <resource> <name>

# Logs
kubectl logs <pod> [-f] [--tail N]

# Execute
kubectl exec -it <pod> -- bash

# Port forward
kubectl port-forward <pod> 8080:80

# Cluster info
kubectl cluster-info
kubectl get nodes
```

## Local Development Clusters

| Tool | Best For |
|------|----------|
| minikube | Single-node local cluster |
| kind | K8s in Docker (CI/CD) |
| k3s | Lightweight K8s |
| Docker Desktop | Built-in K8s |
