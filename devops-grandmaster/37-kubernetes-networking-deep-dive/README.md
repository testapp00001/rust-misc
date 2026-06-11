# Lesson 37: Kubernetes Networking Deep Dive

**Previous:** [Kustomize](../36-kustomize/README.md) | **Next:** [Kubernetes Security](../38-kubernetes-security/README.md)

---

## The Problem

Kubernetes networking is fundamentally different from traditional infrastructure. Every Pod gets its own IP address, every Pod can communicate with every other Pod without NAT, and Services provide stable endpoints for unstable backends. Understanding how this works is critical because networking bugs cause the hardest outages -- silent failures, intermittent drops, and mysterious timeouts that no log can explain.

You need to answer: How do containers talk to each other? How do you restrict traffic? How does DNS resolve service names to Pod IPs?

---

## The Naive Way

Ignore networking entirely. Rely on defaults and hope it works.

```yaml
# A typical developer deployment -- no network policies, no DNS knowledge
apiVersion: apps/v1
kind: Deployment
metadata:
  name: backend
spec:
  replicas: 3
  selector:
    matchLabels:
      app: backend
  template:
    metadata:
      labels:
        app: backend
    spec:
      containers:
      - name: backend
        image: myapp/backend:v1
        ports:
        - containerPort: 8080
```

This works for small clusters. Every Pod can reach every other Pod. You never think about DNS, CNI plugins, or network policies. Then a security audit happens and you realize any compromised Pod can talk to your database.

---

## The Right Way

Understand the Kubernetes networking model and apply NetworkPolicies to restrict traffic flow.

### Kubernetes Networking Model

Kubernetes requires three things from the network:

1. **Every Pod gets its own IP address** -- no port sharing between Pods on the same node
2. **Pod-to-Pod communication works without NAT** -- any Pod can reach any other Pod directly
3. **Agents on a node can communicate with all Pods on that node** -- kubelet needs this

These rules create a flat network. There is no built-in network segmentation. This is by design -- Kubernetes chose simplicity and delegated segmentation to CNI plugins and NetworkPolicies.

### CNI Plugins

The Container Network Interface (CNI) is a plugin architecture that implements the networking model. Each plugin handles IP allocation, routing, and sometimes encryption.

**Flannel** -- The simplest option. Uses VXLAN overlays to create a flat Layer 3 network. No NetworkPolicy support.

```yaml
# Flannel is deployed as a DaemonSet -- check it with:
# kubectl get pods -n kube-flannel
```

**Calico** -- Supports NetworkPolicy, BGP routing, and VXLAN. The most common choice for production clusters that need network segmentation.

```yaml
# Calico provides a default deny policy
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-ingress
spec:
  podSelector: {}
  policyTypes:
  - Ingress
```

**Cilium** -- Built on eBPF. Operates at Layer 3, 4, and 7. Supports NetworkPolicy, observability via Hubble, and service mesh capabilities without sidecars.

```yaml
# Cilium can enforce L7 policies -- filter by HTTP method, path, headers
apiVersion: cilium.io/v2
kind: CiliumNetworkPolicy
metadata:
  name: allow-api-only
spec:
  endpointSelector:
    matchLabels:
      app: backend
  ingress:
  - fromEndpoints:
    - matchLabels:
        app: frontend
    toPorts:
    - ports:
      - port: "8080"
      rules:
        http:
        - method: GET
          path: "/api/v1/.*"
```

### NetworkPolicy

NetworkPolicies are Kubernetes-native resources that control traffic flow. They are additive -- if any policy selects a Pod, then all unmatched traffic is denied.

```yaml
# Deny all ingress traffic to Pods with label app=database
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: deny-all-to-database
  namespace: production
spec:
  podSelector:
    matchLabels:
      app: database
  policyTypes:
  - Ingress
  # No ingress rules = all ingress denied

---
# Allow only backend Pods to reach the database on port 5432
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-backend-to-database
  namespace: production
spec:
  podSelector:
    matchLabels:
      app: database
  policyTypes:
  - Ingress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: backend
    ports:
    - protocol: TCP
      port: 5432
```

Egress policies restrict outbound traffic -- critical for preventing data exfiltration.

```yaml
# Restrict database Pods to only talk to specific CIDR ranges
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: restrict-database-egress
  namespace: production
spec:
  podSelector:
    matchLabels:
      app: database
  policyTypes:
  - Egress
  egress:
  - to:
    - ipBlock:
        cidr: 10.0.0.0/8
    ports:
    - protocol: UDP
      port: 53
  - to:
    - podSelector:
        matchLabels:
          app: backend
    ports:
    - protocol: TCP
      port: 5432
```

### CoreDNS and Service Discovery

CoreDNS runs as a Deployment in `kube-system`. It resolves Service names to ClusterIPs.

```
# DNS resolution order within a Pod:
# 1. backend.production.svc.cluster.local  (FQDN)
# 2. backend.production.svc                (without cluster domain)
# 3. backend.production                     (without svc suffix)
# 4. backend                                (within same namespace)
```

```yaml
# CoreDNS ConfigMap -- check with: kubectl get cm coredns -n kube-system -o yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: coredns
  namespace: kube-system
data:
  Corefile: |
    .:53 {
        errors
        health
        ready
        kubernetes cluster.local in-addr.arpa ip6.arpa {
           pods insecure
           fallthrough in-addr.arpa ip6.arpa
           ttl 30
        }
        forward . /etc/resolv.conf
        cache 30
        loop
        reload
        loadbalance
    }
```

Verify DNS resolution:

```bash
# Launch a debug Pod
kubectl run debug --image=busybox:1.36 --rm -it --restart=Never -- nslookup kubernetes.default

# Expected output:
# Name:      kubernetes.default.svc.cluster.local
# Address:   10.96.0.1
```

---

## The Production Way

### CNI Selection Guide

| Feature | Flannel | Calico | Cilium |
|---------|---------|--------|--------|
| NetworkPolicy | No | Yes | Yes (L3/L4/L7) |
| Encryption | No | WireGuard | WireGuard/IPsec |
| Observability | No | Basic | Hubble (rich) |
| Performance | Good | Good | Best (eBPF) |
| Complexity | Low | Medium | High |
| Service Mesh | No | No | Yes (sidecar-free) |

Production recommendation: **Cilium** for new clusters with the budget for a learning curve. **Calico** for teams that want proven stability with NetworkPolicy support.

### Zero-Trust Network Policy Strategy

Start from deny-all, then explicitly allow required flows.

```yaml
# Step 1: Deny all traffic in the namespace
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-all
  namespace: production
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress

---
# Step 2: Allow DNS resolution for all Pods
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-dns
  namespace: production
spec:
  podSelector: {}
  policyTypes:
  - Egress
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: kube-system
    ports:
    - protocol: UDP
      port: 53
    - protocol: TCP
      port: 53

---
# Step 3: Allow ingress controller to reach frontends
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-ingress-to-frontend
  namespace: production
spec:
  podSelector:
    matchLabels:
      app: frontend
  policyTypes:
  - Ingress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: ingress-nginx
    ports:
    - protocol: TCP
      port: 3000
```

### DNS Tuning for Production

```yaml
# Increase CoreDNS cache TTL for services with stable backends
# In Corefile:
# cache 300 {
#     success 10000
#     denial 500
# }

# Add NodeLocal DNSCache to reduce CoreDNS load
# This runs a DNS cache on every node
# kubectl apply -f https://github.com/kubernetes/kubernetes/blob/master/cluster/addons/dns/nodelocaldns/nodelocaldns.yaml
```

### Monitoring CNI Health

```bash
# Check CNI plugin logs
kubectl logs -n kube-system -l k8s-app=calico-node --tail=50

# Check for IP allocation exhaustion
kubectl get ippool -o yaml | grep -A5 spec

# Verify no Pods are in CrashLoopBackOff due to CNI issues
kubectl get pods -A --field-selector=status.phase!=Running
```

---

## Hands-On Lab

### Lab: Implement Network Segmentation

**Step 1:** Create namespaces and deploy applications.

```bash
kubectl create namespace frontend
kubectl create namespace backend
kubectl create namespace database

# Deploy a simple app in each namespace
kubectl create deployment web --image=nginx -n frontend
kubectl create deployment api --image=nginx -n backend
kubectl create deployment db --image=nginx -n database

# Expose as Services
kubectl expose deployment web --port=80 -n frontend
kubectl expose deployment api --port=80 -n backend
kubectl expose deployment db --port=5432 -n database
```

**Step 2:** Verify connectivity is open by default.

```bash
# From frontend, reach the database -- this should work (bad!)
kubectl exec -n frontend deploy/web -- curl -s --max-time 3 http://db.database.svc.cluster.local:5432
```

**Step 3:** Apply a deny-all policy to the database namespace.

```yaml
# deny-all-database.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: deny-all
  namespace: database
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
```

```bash
kubectl apply -f deny-all-database.yaml

# Verify connectivity is now blocked
kubectl exec -n frontend deploy/web -- curl -s --max-time 3 http://db.database.svc.cluster.local:5432
# Should timeout
```

**Step 4:** Allow only backend namespace to reach the database.

```yaml
# allow-backend-to-db.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-backend
  namespace: database
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: backend
    ports:
    - protocol: TCP
      port: 5432
```

```bash
kubectl apply -f allow-backend-to-db.yaml

# Frontend still blocked
kubectl exec -n frontend deploy/web -- curl -s --max-time 3 http://db.database.svc.cluster.local:5432

# Backend now allowed
kubectl exec -n backend deploy/api -- curl -s --max-time 3 http://db.database.svc.cluster.local:5432
```

**Step 5:** Add DNS egress so Pods can resolve names.

```yaml
# allow-dns-database.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-dns
  namespace: database
spec:
  podSelector: {}
  policyTypes:
  - Egress
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: kube-system
    ports:
    - protocol: UDP
      port: 53
```

**Step 6:** Clean up.

```bash
kubectl delete namespace frontend backend database
```

---

## Limitation

Kubernetes networking provides the foundation for connectivity, but NetworkPolicies are not enough on their own. They operate at Layer 3/4 -- they can restrict which Pods talk to which Pods on which ports, but they cannot inspect or filter application-layer traffic (HTTP paths, gRPC methods, SQL queries). A Pod with network access can still send malicious payloads.

More critically, if your CNI plugin does not support NetworkPolicy (like Flannel), the resources you create are silently ignored. There is no error, no warning -- they just do nothing. This creates a false sense of security.

NetworkPolicy also has no egress logging by default. You cannot see which connections were denied unless your CNI provides observability (Cilium Hubble, Calico flow logs). Without this, debugging network issues in production is guesswork.

Security concerns remain: compromised Pods, lateral movement, and exfiltration require additional layers beyond network segmentation.

**Next:** [Kubernetes Security](../38-kubernetes-security/README.md) -- PodSecurityStandards, OPA/Gatekeeper, and image scanning to harden the cluster beyond network boundaries.
