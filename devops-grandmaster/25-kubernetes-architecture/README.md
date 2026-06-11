# Module 25: Kubernetes Architecture — How It All Works

## The Problem: What IS Kubernetes?

Kubernetes is a **container orchestration platform**. It manages containers across multiple machines, handling:
- Deployment (where to run containers)
- Scaling (how many copies)
- Networking (how they talk)
- Storage (where data lives)
- Self-healing (what to do when things fail)

## The Big Picture

```
┌─────────────────────────────────────────────────────────┐
│                    Kubernetes Cluster                      │
│                                                           │
│  ┌─────────────────────────────────────────────────────┐ │
│  │              Control Plane (Master)                   │ │
│  │  ┌──────────┬──────────┬──────────┬──────────┐      │ │
│  │  │ API      │ Scheduler│ Controller│ etcd    │      │ │
│  │  │ Server   │          │ Manager   │         │      │ │
│  │  └──────────┴──────────┴──────────┴──────────┘      │ │
│  └─────────────────────────────────────────────────────┘ │
│                           │                                │
│  ┌────────────────────────┼────────────────────────────┐ │
│  │                        │                             │ │
│  │  ┌─────────────┐  ┌───▼───────────┐  ┌──────────┐ │ │
│  │  │  Worker      │  │  Worker       │  │  Worker  │ │ │
│  │  │  Node 1      │  │  Node 2       │  │  Node 3  │ │ │
│  │  │  ┌─────┐    │  │  ┌─────┐     │  │  ┌─────┐ │ │ │
│  │  │  │Pod A│    │  │  │Pod D│     │  │  │Pod F│ │ │ │
│  │  │  │Pod B│    │  │  │Pod E│     │  │  │Pod G│ │ │ │
│  │  │  │Pod C│    │  │  │     │     │  │  │     │ │ │ │
│  │  │  └─────┘    │  │  └─────┘     │  │  └─────┘ │ │ │
│  │  │  [kubelet]  │  │  [kubelet]   │  │  [kubelet]│ │ │
│  │  │  [kube-proxy│  │  [kube-proxy │  │  [kube-   │ │ │
│  │  │  [Container │  │  [Container  │  │  proxy    │ │ │
│  │  │   Runtime]  │  │   Runtime]   │  │  [Contain │ │ │
│  │  └─────────────┘  └──────────────┘  │   Runtime]│ │ │
│  │                                      └──────────┘ │ │
│  └───────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

## Control Plane Components

### API Server (kube-apiserver)
The front door to the cluster. Everything talks through it.

```bash
# All kubectl commands go through the API server
kubectl get pods          # → API server → etcd → response
kubectl apply -f pod.yml  # → API server → etcd → scheduler → kubelet
```

- RESTful API
- Authentication & authorization
- Rate limiting
- The only component that talks to etcd

### etcd
The brain — stores all cluster state.

```bash
# What's in etcd:
# - All Kubernetes objects (pods, services, deployments)
# - Cluster configuration
# - Node status
# - Secrets (encrypted)
```

- Distributed key-value store
- Consistent (Raft consensus algorithm)
- **Critical:** If etcd loses data, you lose everything
- Backup etcd regularly!

### Scheduler (kube-scheduler)
Decides which node runs each pod.

```
New Pod → Scheduler → Which node has:
  - Enough CPU/memory?
  - Required labels/taints?
  - Affinity/anti-affinity rules?
  → Assign pod to best node
```

### Controller Manager (kube-controller-manager)
Runs reconciliation loops — makes reality match desired state.

```
Desired: 3 replicas of my-app
Reality: 2 replicas running
Controller: "Need 1 more!" → Creates new pod

Desired: 3 replicas of my-app  
Reality: 4 replicas running
Controller: "1 too many!" → Deletes a pod
```

Types of controllers:
- **ReplicaSet** — Maintains pod count
- **Deployment** — Manages ReplicaSets
- **Node** — Monitors node health
- **Service** — Manages endpoints
- **Job** — Runs tasks to completion

## Worker Node Components

### kubelet
The agent on each node. Runs pods, reports status.

```bash
# kubelet responsibilities:
# - Watch API server for pod assignments
# - Pull container images
# - Start/stop containers
# - Report node and pod status
# - Run liveness/readiness probes
```

### kube-proxy
Manages networking rules on each node.

```bash
# kube-proxy responsibilities:
# - Implement Services (load balancing)
# - iptables/IPVS rules
# - Forward traffic to correct pods
```

### Container Runtime
Actually runs containers (containerd, CRI-O).

```bash
# Kubernetes doesn't run containers directly
# It uses the Container Runtime Interface (CRI)
# Most common: containerd (used by Docker)
```

## How a Pod Gets Created

```
1. You run: kubectl apply -f pod.yaml
2. API server receives request, stores in etcd
3. Scheduler sees unassigned pod
4. Scheduler picks best node, updates etcd
5. Kubelet on that node sees new assignment
6. Kubelet tells container runtime to start containers
7. Container runtime pulls image (if needed) and starts container
8. Kubelet reports status back to API server
9. API server updates etcd
```

## Cluster Setup Options

### Managed Kubernetes (Recommended for Production)
```bash
# AWS EKS
eksctl create cluster --name my-cluster --nodes 3

# Google GKE
gcloud container clusters create my-cluster --num-nodes=3

# Azure AKS
az aks create --name my-cluster --node-count 3
```

### Self-Managed (kubeadm)
```bash
# On the master node
sudo kubeadm init --pod-network-cidr=10.244.0.0/16

# Set up kubectl
mkdir -p $HOME/.kube
sudo cp /etc/kubernetes/admin.conf $HOME/.kube/config

# Install network plugin (Calico, Flannel, Cilium)
kubectl apply -f https://docs.projectcalico.org/manifests/calico.yaml

# On worker nodes
sudo kubeadm join <master-ip>:6443 --token <token> --discovery-token-ca-cert-hash <hash>
```

### Local Development
```bash
# minikube (single node)
minikube start

# kind (Kubernetes in Docker)
kind create cluster

# k3s (lightweight)
curl -sfL https://get.k3s.io | sh -
```

## kubectl — Your Interface to Kubernetes

```bash
# Get resources
kubectl get pods
kubectl get services
kubectl get deployments
kubectl get nodes

# Describe resource (detailed info)
kubectl describe pod my-pod

# Create/Update resources
kubectl apply -f resource.yaml

# Delete resources
kubectl delete pod my-pod

# View logs
kubectl logs my-pod
kubectl logs -f my-pod  # Follow

# Execute command in pod
kubectl exec -it my-pod -- bash

# Port forward
kubectl port-forward my-pod 8080:80

# View cluster info
kubectl cluster-info
kubectl get nodes
```

## Hands-On Exercise

### Exercise 1: Set Up a Local Cluster

```bash
# Install kind (Kubernetes in Docker)
# Linux/macOS
curl -Lo ./kind https://kind.sigs.k8s.io/dl/v0.20.0/kind-linux-amd64
chmod +x ./kind
sudo mv ./kind /usr/local/bin/kind

# Create a cluster
kind create cluster --name devops-lab

# Verify
kubectl cluster-info
kubectl get nodes

# Explore
kubectl get all --all-namespaces
```

### Exercise 2: Understand the Components

```bash
# See system pods (control plane)
kubectl get pods -n kube-system

# You should see:
# - coredns (DNS)
# - etcd (state store)
# - kube-apiserver (API)
# - kube-controller-manager
# - kube-proxy
# - kube-scheduler
# - kindnet (network plugin)

# Describe each component
kubectl describe pod etcd -n kube-system
kubectl describe pod kube-apiserver -n kube-system
```

### Exercise 3: Create Your First Pod (Imperative)

```bash
# Run a simple pod
kubectl run nginx --image=nginx:latest --port=80

# See it
kubectl get pods

# Get details
kubectl describe pod nginx

# View logs
kubectl logs nginx

# Access it
kubectl port-forward nginx 8080:80
# Visit http://localhost:8080

# Delete it
kubectl delete pod nginx
```

## Limitation: You Know the Architecture, But Not How to Deploy YOUR App

You understand what makes up a cluster. But you don't know how to define and deploy your own workloads.

**Next problem:** How do you define a pod and run your application in Kubernetes?

→ **Next module:** [26-pods-and-containers](../26-pods-and-containers/) — The smallest deployable unit

## Checklist

- [ ] I can name all Control Plane components and their roles
- [ ] I can name all Worker Node components and their roles
- [ ] I understand how a pod gets created (the full lifecycle)
- [ ] I know the difference between managed and self-managed k8s
- [ ] I can set up a local cluster with kind or minikube
- [ ] I can use basic kubectl commands (get, describe, logs, exec)
