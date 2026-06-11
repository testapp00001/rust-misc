# Solution 03: Service Discovery Using DNS

## Part A: Create a Two-Tier Application

**Step 1: Backend Deployment and Service (backend.yaml):**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-server
spec:
  replicas: 2
  selector:
    matchLabels:
      app: api
  template:
    metadata:
      labels:
        app: api
    spec:
      containers:
        - name: nginx
          image: nginx:1.25
          ports:
            - containerPort: 80
---
apiVersion: v1
kind: Service
metadata:
  name: api-service
spec:
  type: ClusterIP
  selector:
    app: api
  ports:
    - port: 80
      targetPort: 80
```

**Step 2: Frontend Pod (frontend.yaml):**

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: frontend
spec:
  containers:
    - name: curl
      image: curlimages/curl:8.4.0
      command: ['sh', '-c', 'while true; do curl -s http://api-service:80; echo "---"; sleep 5; done']
```

**Step 3: Verify connectivity:**

```bash
kubectl apply -f backend.yaml
kubectl apply -f frontend.yaml
kubectl logs frontend -f
```

The output should show the nginx welcome page HTML repeated every 5 seconds, separated by `---`. This confirms that the frontend pod can reach the backend service using the DNS name `api-service`.

**Why this works:** Within the same namespace, Kubernetes DNS allows you to use just the service name (`api-service`) as the hostname. The DNS resolver appends the search domain automatically, resolving `api-service` to `api-service.default.svc.cluster.local`, which returns the ClusterIP of the service.

## Part B: Test Cross-Namespace Discovery

**Step 1: Create the namespace:**

```bash
kubectl create namespace backend-ns
```

**Step 2: Backend in backend-ns (backend-ns.yaml):**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: internal-api
  namespace: backend-ns
spec:
  replicas: 2
  selector:
    matchLabels:
      app: internal-api
  template:
    metadata:
      labels:
        app: internal-api
    spec:
      containers:
        - name: nginx
          image: nginx:1.25
          ports:
            - containerPort: 80
---
apiVersion: v1
kind: Service
metadata:
  name: internal-api
  namespace: backend-ns
spec:
  type: ClusterIP
  selector:
    app: internal-api
  ports:
    - port: 80
      targetPort: 80
```

**Step 3: Test cross-namespace access:**

```bash
kubectl apply -f backend-ns.yaml

# Full FQDN -- this works
kubectl exec frontend -- curl -s http://internal-api.backend-ns.svc.cluster.local:80

# Short name -- this fails
kubectl exec frontend -- curl -s http://internal-api:80
```

**The full FQDN works.** The short name `internal-api` does not work. This is because the `frontend` pod is in the `default` namespace, and its DNS search domain is set to `default.svc.cluster.local`. When it resolves the short name `internal-api`, DNS appends the search domain and looks up `internal-api.default.svc.cluster.local`, which does not exist. The service `internal-api` exists in `backend-ns`, not `default`. To reach it, you must use the full FQDN: `internal-api.backend-ns.svc.cluster.local`.

## Part C: Explore DNS Records

Run the following from a debug pod:

```bash
kubectl run dns-debug --image=busybox:1.36 --rm -it -- sh
```

**Resolving a ClusterIP Service:**

```sh
nslookup api-service
```

Output shows a single ClusterIP address, for example:

```
Name:      api-service
Address 1: 10.96.45.12
```

**Resolving using full FQDN:**

```sh
nslookup api-service.default.svc.cluster.local
```

Same result -- a single ClusterIP address. The short name and the FQDN resolve to the same IP.

**Resolving a cross-namespace Service:**

```sh
nslookup internal-api.backend-ns.svc.cluster.local
```

Returns the ClusterIP of the `internal-api` service in `backend-ns`.

**SRV records:**

```sh
nslookup -type=SRV _http._tcp.api-service.default.svc.cluster.local
```

SRV records return the port number and target hostname. This is useful for discovering which port a service listens on when you do not know it in advance.

**Checking /etc/resolv.conf:**

```sh
cat /etc/resolv.conf
```

Typical output:

```
nameserver 10.96.0.10
search default.svc.cluster.local svc.cluster.local cluster.local
ndots:5
```

**Answers to the questions:**

1. **What IP address does `api-service` resolve to?** It resolves to the ClusterIP assigned to the `api-service` service (e.g., `10.96.45.12`). This is a virtual IP managed by kube-proxy, not a physical pod IP.

2. **What is the search domain?** The search domain is `default.svc.cluster.local svc.cluster.local cluster.local`. The first entry is the pod's own namespace. The remaining entries provide fallback resolution for shorter suffixes.

3. **Why does the search domain explain the short-name behavior?** When you type `internal-api`, DNS appends each search domain in order: first `internal-api.default.svc.cluster.local` (not found), then `internal-api.svc.cluster.local` (not found), then `internal-api.cluster.local` (not found). None of these match the actual service at `internal-api.backend-ns.svc.cluster.local`. The search domain is scoped to the pod's own namespace, so short names only resolve services in the same namespace.

## Part D: Headless Service DNS

**Step 1: Create the headless Service (api-headless.yaml):**

```yaml
apiVersion: v1
kind: Service
metadata:
  name: api-headless
spec:
  clusterIP: None
  selector:
    app: api
  ports:
    - port: 80
      targetPort: 80
```

**Step 2: Compare DNS behavior:**

```bash
kubectl apply -f api-headless.yaml
kubectl run dns-test2 --image=busybox:1.36 --rm -it -- sh
```

```sh
# Regular ClusterIP Service
nslookup api-service

# Headless Service
nslookup api-headless
```

**Regular ClusterIP Service output:**

```
Name:      api-service
Address 1: 10.96.45.12
```

Returns a single virtual IP (the ClusterIP).

**Headless Service output:**

```
Name:      api-headless
Address 1: 10.244.0.5
Address 2: 10.244.1.3
```

Returns the individual IP addresses of both pods (2 replicas = 2 IPs). There is no virtual IP.

**The difference:** A regular ClusterIP Service returns one virtual IP, and kube-proxy handles load balancing across pods. A headless Service returns all pod IPs directly, giving the client the full list of endpoints. The client (or client-side library) can then choose which pod to connect to, enabling client-side load balancing, sticky connections to specific pods, or StatefulSet-style addressing where each pod has a stable DNS name.

### Common Mistakes to Avoid

- **Using short names across namespaces.** The short name `api-service` only works within the same namespace. For cross-namespace communication, always use the full FQDN. This is one of the most common mistakes in multi-namespace architectures.
- **Forgetting that DNS is a cluster-level service.** The Kubernetes DNS service (CoreDNS) runs in the `kube-system` namespace. If CoreDNS is down, no service discovery works. Check `kubectl get pods -n kube-system -l k8s-app=kube-dns` if DNS resolution fails.
- **Confusing ClusterIP DNS with headless DNS.** A ClusterIP Service returns one IP (the virtual IP). A headless Service returns all pod IPs. If you need client-side load balancing or individual pod addressing, use headless. If you want kube-proxy to handle load balancing, use ClusterIP.
- **Hardcoding IPs instead of using DNS names.** Pod IPs change when pods are restarted. Service ClusterIPs are stable for the life of the Service object. Always use DNS names, never hardcode IPs.
- **Assuming SRV records always exist.** SRV records are only created for services with named ports. If you do not name your ports in the Service spec, SRV lookups will fail.

## Key Takeaway

Kubernetes DNS provides automatic service discovery. Services get DNS entries that resolve to their ClusterIP. Within the same namespace, short names work because of the DNS search domain configured in `/etc/resolv.conf`. Across namespaces, you must use the full FQDN (`service-name.namespace.svc.cluster.local`). Headless Services return individual pod IPs instead of a single virtual IP, enabling client-side load balancing and StatefulSet-style addressing. This DNS-based discovery is the foundation of all inter-service communication in Kubernetes.
