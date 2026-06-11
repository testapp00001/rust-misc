# Module 28: Services & Networking — Stable Endpoints for Ephemeral Pods

**Previous:** [Module 27: Deployments & ReplicaSets](../27-deployments-and-replicasets/README.md)

---

## The Problem

You have a deployment running 3 replicas of your web application. Each pod has its own IP address. But:

- Pod IPs are ephemeral — they change when pods are replaced
- You need a single, stable endpoint for clients to connect to
- You need load balancing across the 3 replicas
- Internal services need to discover and talk to each other
- External traffic needs to reach your application

You cannot hardcode pod IPs in configuration files. You need a service discovery and load balancing mechanism.

---

## The Naive Way

Hardcode pod IPs or use external load balancers:

```yaml
# In your application config
database_host: 10.244.1.5    # Pod IP (will change on restart!)
api_host: 10.244.2.12        # Pod IP (will change on restart!)
```

Or run an external HAProxy/Nginx that you manually update when pods change.

**What goes wrong:**

- Pod IPs change on every restart, scale, or node failure
- Manual updates to load balancer configs are error-prone
- No automatic health checking and traffic routing
- No service discovery — every service must know every other service's IP
- External load balancers add complexity and single points of failure

---

## The Right Way

### What Is a Kubernetes Service?

A **Service** is a stable abstraction that defines a logical set of pods and a policy to access them. It provides:

- A stable virtual IP (ClusterIP) that never changes
- A DNS name that resolves to that IP
- Load balancing across healthy pods
- Automatic updates when pods are added or removed

```yaml
# service.yaml
apiVersion: v1
kind: Service
metadata:
  name: nginx-service
spec:
  selector:
    app: nginx        # Selects pods with label app=nginx
  ports:
    - protocol: TCP
      port: 80         # Service port (what clients connect to)
      targetPort: 80   # Pod port (where traffic is forwarded)
  type: ClusterIP      # Default type
```

**How it works:**

```
Client → nginx-service (10.96.0.100:80) → Pod 1 (10.244.1.5:80)
                                        → Pod 2 (10.244.2.12:80)
                                        → Pod 3 (10.244.3.8:80)
```

Kubernetes maintains an **Endpoints** object that tracks the current pod IPs matching the service selector. When pods are created or deleted, the endpoints are updated automatically.

### Service Types

#### 1. ClusterIP (Default)

Exposes the service on an internal IP address, reachable only within the cluster.

```yaml
apiVersion: v1
kind: Service
metadata:
  name: backend-service
spec:
  type: ClusterIP
  selector:
    app: backend
  ports:
    - port: 8080
      targetPort: 8080
```

**Use case:** Internal communication between services (e.g., frontend to backend, backend to database).

#### 2. NodePort

Exposes the service on a static port on each node's IP address.

```yaml
apiVersion: v1
kind: Service
metadata:
  name: web-nodeport
spec:
  type: NodePort
  selector:
    app: web
  ports:
    - port: 80           # Service port (internal)
      targetPort: 80     # Pod port
      nodePort: 30080    # Port on each node (30000-32767)
```

**Access:** `<NodeIP>:30080` from outside the cluster.

**Use case:** Development/testing, exposing services without a cloud load balancer.

#### 3. LoadBalancer

Provisions an external load balancer (cloud provider specific) that routes traffic to the service.

```yaml
apiVersion: v1
kind: Service
metadata:
  name: web-lb
spec:
  type: LoadBalancer
  selector:
    app: web
  ports:
    - port: 80
      targetPort: 80
```

**Access:** The cloud provider assigns an external IP address.

**Use case:** Production workloads on cloud providers (AWS, GCP, Azure).

#### 4. ExternalName

Maps a service to a DNS name (no proxying).

```yaml
apiVersion: v1
kind: Service
metadata:
  name: external-database
spec:
  type: ExternalName
  externalName: db.prod.example.com
```

**Use case:** Accessing external services (RDS, external APIs) with a consistent internal DNS name.

### Service Discovery via DNS

Kubernetes provides automatic DNS resolution for services:

```
<service-name>.<namespace>.svc.cluster.local
```

**Examples within the same namespace:**

```bash
# Service name resolves to ClusterIP
curl http://backend-service:8080

# Full FQDN (works from any namespace)
curl http://backend-service.default.svc.cluster.local:8080
```

**Examples across namespaces:**

```bash
# Service in 'production' namespace
curl http://backend-service.production.svc.cluster.local:8080

# Service in 'monitoring' namespace
curl http://prometheus.monitoring.svc.cluster.local:9090
```

**SRV records** for discovering port numbers:

```bash
# Discover all ports for a service
nslookup -type=SRV _http._tcp.backend-service.default.svc.cluster.local
```

### Headless Services

A headless service (ClusterIP: None) does not get a virtual IP. DNS returns the individual pod IPs directly.

```yaml
apiVersion: v1
kind: Service
metadata:
  name: nginx-headless
spec:
  clusterIP: None
  selector:
    app: nginx
  ports:
    - port: 80
      targetPort: 80
```

**DNS resolution:**

```bash
# Returns all pod IPs
nslookup nginx-headless.default.svc.cluster.local

# Example output:
# Name: nginx-headless.default.svc.cluster.local
# Address: 10.244.1.5
# Address: 10.244.2.12
# Address: 10.244.3.8
```

**Use cases:**
- Stateful applications (databases) that need to know individual instances
- Client-side load balancing
- Service mesh implementations

### Endpoint Objects

Kubernetes automatically creates an **Endpoints** object for each service, tracking the pod IPs:

```bash
# View endpoints for a service
kubectl get endpoints nginx-service

# Output:
# NAME            ENDPOINTS                                      AGE
# nginx-service   10.244.1.5:80,10.244.2.12:80,10.244.3.8:80   5m
```

When a pod becomes unready (fails readiness probe), its IP is removed from the endpoints. When a new pod is created, its IP is added.

### Service Without Selectors

You can manually manage endpoints for services that point to external resources:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: external-api
spec:
  type: ClusterIP
  ports:
    - port: 443
      targetPort: 443

---
apiVersion: v1
kind: Endpoints
metadata:
  name: external-api
subsets:
  - addresses:
      - ip: 203.0.113.10
      - ip: 203.0.113.11
    ports:
      - port: 443
```

---

## The Production Way

### Session Affinity

Route requests from the same client to the same pod:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: sticky-service
spec:
  selector:
    app: myapp
  sessionAffinity: ClientIP
  sessionAffinityConfig:
    clientIP:
      timeoutSeconds: 1800  # 30 minutes
  ports:
    - port: 80
      targetPort: 80
```

**Use case:** Applications that store session state locally (not recommended — use Redis instead).

### Multi-Port Services

```yaml
apiVersion: v1
kind: Service
metadata:
  name: myapp-service
spec:
  selector:
    app: myapp
  ports:
    - name: http
      port: 80
      targetPort: 8080
    - name: https
      port: 443
      targetPort: 8443
    - name: metrics
      port: 9090
      targetPort: 9090
```

**Note:** When a service has multiple ports, you must name them.

### Service Mesh Integration

Services integrate with service meshes like Istio or Linkerd:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: myapp
  labels:
    app: myapp
spec:
  selector:
    app: myapp
  ports:
    - port: 80
      targetPort: 8080
      name: http
```

Service meshes add:
- mTLS between services
- Advanced traffic management (canary, circuit breaking)
- Observability (distributed tracing, metrics)
- Retries and timeouts

### Network Policies (Service-Level)

Restrict which pods can access a service:

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: backend-policy
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
        - protocol: TCP
          port: 8080
```

Only pods with `app: frontend` can reach pods with `app: backend` on port 8080.

### External Traffic Policy

Control how external traffic is routed to pods:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: web-service
spec:
  type: NodePort
  externalTrafficPolicy: Local    # Preserve client IP, no cross-node routing
  selector:
    app: web
  ports:
    - port: 80
      targetPort: 80
      nodePort: 30080
```

**`Cluster` (default):** Traffic may be routed to pods on other nodes (extra hop, loses client IP).

**`Local`:** Traffic is only routed to pods on the receiving node (preserves client IP, may cause uneven load).

---

## Hands-On Lab

### Exercise 1: Create a ClusterIP Service

```bash
# Create a deployment
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
EOF

# Create a ClusterIP service
kubectl apply -f - <<EOF
apiVersion: v1
kind: Service
metadata:
  name: nginx-service
spec:
  selector:
    app: nginx
  ports:
    - port: 80
      targetPort: 80
  type: ClusterIP
EOF

# Check the service
kubectl get service nginx-service

# Check the endpoints
kubectl get endpoints nginx-service

# Test the service from within the cluster
kubectl run test-pod --image=busybox:1.36 --rm -it -- sh
# Inside the pod:
wget -qO- http://nginx-service
exit
```

### Exercise 2: Create a NodePort Service

```bash
# Create a NodePort service
kubectl apply -f - <<EOF
apiVersion: v1
kind: Service
metadata:
  name: nginx-nodeport
spec:
  selector:
    app: nginx
  ports:
    - port: 80
      targetPort: 80
      nodePort: 30080
  type: NodePort
EOF

# Check the service
kubectl get service nginx-nodeport

# Get node IPs
kubectl get nodes -o wide

# Access the service (from outside the cluster)
curl http://<node-ip>:30080
```

### Exercise 3: Test Service Discovery

```bash
# Create a frontend that talks to the backend
kubectl apply -f - <<EOF
apiVersion: v1
kind: Pod
metadata:
  name: frontend
spec:
  containers:
    - name: curl
      image: curlimages/curl:8.4.0
      command: ['sh', '-c', 'while true; do curl -s http://nginx-service; echo; sleep 5; done']
EOF

# Watch the frontend logs
kubectl logs frontend -f

# Verify DNS resolution
kubectl exec -it frontend -- nslookup nginx-service
```

### Exercise 4: Headless Service

```bash
# Create a headless service
kubectl apply -f - <<EOF
apiVersion: v1
kind: Service
metadata:
  name: nginx-headless
spec:
  clusterIP: None
  selector:
    app: nginx
  ports:
    - port: 80
      targetPort: 80
EOF

# Check the service
kubectl get service nginx-headless

# DNS resolution returns individual pod IPs
kubectl run dns-test --image=busybox:1.36 --rm -it -- sh
# Inside the pod:
nslookup nginx-headless
exit
```

### Exercise 5: Multi-Port Service

```bash
# Create a deployment with multiple ports
kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: multi-port-app
spec:
  replicas: 2
  selector:
    matchLabels:
      app: multi
  template:
    metadata:
      labels:
        app: multi
    spec:
      containers:
        - name: web
          image: nginx:1.25
          ports:
            - containerPort: 80
            - containerPort: 443
        - name: metrics
          image: prom/node-exporter:v1.7.0
          ports:
            - containerPort: 9100
EOF

# Create a multi-port service
kubectl apply -f - <<EOF
apiVersion: v1
kind: Service
metadata:
  name: multi-service
spec:
  selector:
    app: multi
  ports:
    - name: http
      port: 80
      targetPort: 80
    - name: https
      port: 443
      targetPort: 443
    - name: metrics
      port: 9100
      targetPort: 9100
  type: ClusterIP
EOF

# Verify the service
kubectl describe service multi-service
```

### Exercise 6: Service Endpoints in Action

```bash
# Watch endpoints change as pods scale
kubectl get endpoints nginx-service -w &

# Scale down
kubectl scale deployment nginx-deployment --replicas=1

# Scale up
kubectl scale deployment nginx-deployment --replicas=5

# Clean up
kubectl delete deployment nginx-deployment multi-port-app
kubectl delete service nginx-service nginx-nodeport nginx-headless multi-service
kubectl delete pod frontend
```

---

## Verification Checklist

- [ ] Understand why pod IPs are ephemeral and why services are needed
- [ ] Know the four service types: ClusterIP, NodePort, LoadBalancer, ExternalName
- [ ] Can create services with proper selectors and ports
- [ ] Understand DNS-based service discovery
- [ ] Know when to use headless services
- [ ] Can inspect endpoints to verify traffic routing
- [ ] Understand session affinity and external traffic policy

---

## Limitation

You have learned how to expose services internally (ClusterIP) and via node ports (NodePort). But for production web applications, you need:

- HTTP path-based routing (`/api` goes to backend, `/` goes to frontend)
- Host-based routing (`api.example.com` vs `web.example.com`)
- TLS/SSL termination (HTTPS)
- Automatic certificate management

NodePort and LoadBalancer services operate at Layer 4 (TCP/UDP) and cannot do HTTP routing. You need an HTTP-aware proxy.

**Next:** [Module 29: Ingress Controllers](../29-ingress-controllers/README.md) — HTTP routing, TLS termination, and path-based traffic management.
