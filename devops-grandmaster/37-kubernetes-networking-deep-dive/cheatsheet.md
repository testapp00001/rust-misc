# Cheatsheet: Kubernetes Networking Deep Dive

## Networking Model

```
Every Pod gets its own IP
Pod-to-Pod communication: no NAT
Services provide stable endpoints
CNI plugins implement the networking
```

## CNI Plugins Comparison

| CNI | Network Policy | Encryption | Performance |
|-----|---------------|------------|-------------|
| Calico | ✅ Full | ✅ WireGuard | High |
| Cilium | ✅ Full | ✅ WireGuard | Highest (eBPF) |
| Flannel | ❌ None | ❌ None | Medium |
| Weave | ✅ Basic | ✅ IPSec | Medium |
| Antrea | ✅ Full | ✅ WireGuard | High |

## NetworkPolicy Examples

### Default Deny All Ingress
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-ingress
spec:
  podSelector: {}
  policyTypes:
  - Ingress
```

### Allow Specific Traffic
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-frontend-to-backend
spec:
  podSelector:
    matchLabels:
      app: backend
  policyTypes:
  - Ingress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: frontend
    ports:
    - port: 8080
```

### Allow from Namespace
```yaml
ingress:
- from:
  - namespaceSelector:
      matchLabels:
        env: production
```

## DNS Resolution

```bash
# Service DNS: <service>.<namespace>.svc.cluster.local
# Pod DNS: <pod-ip-dashed>.<namespace>.pod.cluster.local

# Inside a Pod:
nslookup my-service                    # Same namespace
nslookup my-service.other-ns           # Different namespace
nslookup my-service.other-ns.svc.cluster.local  # FQDN
```

## Debugging Network Issues

```bash
# Test connectivity from a debug Pod
kubectl run debug --rm -it --image=nicolaka/netshoot -- bash

# Inside debug Pod:
curl http://my-service:8080
nslookup my-service
tcpdump -i eth0 -nn
traceroute my-service

# Check DNS resolution
kubectl exec -it my-pod -- nslookup kubernetes.default

# Check endpoints
kubectl get endpoints my-service

# Check NetworkPolicies
kubectl get networkpolicy -A
kubectl describe networkpolicy my-policy
```

## Service Types & Networking

| Type | Scope | Use Case |
|------|-------|----------|
| ClusterIP | Internal only | Default, internal services |
| NodePort | External via node IP | Development, testing |
| LoadBalancer | External via LB | Production external access |
| ExternalName | DNS CNAME | External service alias |

## CoreDNS Configuration

```yaml
# Check CoreDNS config
kubectl -n kube-system get configmap coredns -o yaml

# Custom DNS entry (CoreDNS hosts plugin)
hosts {
  10.0.0.1 my-external-service.example.com
  fallthrough
}
```
