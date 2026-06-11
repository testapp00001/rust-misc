# Solution 04: Service Topology for Microservices

## Part A: Design the Service Topology

| Component | Image | Replicas | Labels | Service Type | Service Name | Port | Target Port | Special Config |
|-----------|-------|----------|--------|--------------|--------------|------|-------------|----------------|
| Frontend | nginx:1.25 | 2 | `app: frontend, tier: frontend` | NodePort | `frontend-service` | 80 | 80 | Accessible externally via nodePort |
| API Gateway | nginx:1.25 | 2 | `app: api-gateway, tier: gateway` | ClusterIP | `api-gateway` | 8080 | 80 | Internal only |
| Product Service | nginx:1.25 | 3 | `app: product-service, tier: backend` | ClusterIP | `product-service` | 8080 | 80 | Internal only |
| Order Service | nginx:1.25 | 2 | `app: order-service, tier: backend` | ClusterIP (headless) | `order-service` | 8080 | 80 | `clusterIP: None` for individual pod addressing |

## Part B: Deploy the Topology

**frontend.yaml:**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: frontend
spec:
  replicas: 2
  selector:
    matchLabels:
      app: frontend
  template:
    metadata:
      labels:
        app: frontend
        tier: frontend
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
  name: frontend-service
spec:
  type: NodePort
  selector:
    app: frontend
  ports:
    - port: 80
      targetPort: 80
```

**api-gateway.yaml:**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-gateway
spec:
  replicas: 2
  selector:
    matchLabels:
      app: api-gateway
  template:
    metadata:
      labels:
        app: api-gateway
        tier: gateway
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
  name: api-gateway
spec:
  type: ClusterIP
  selector:
    app: api-gateway
  ports:
    - port: 8080
      targetPort: 80
```

**product-service.yaml:**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: product-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: product-service
  template:
    metadata:
      labels:
        app: product-service
        tier: backend
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
  name: product-service
spec:
  type: ClusterIP
  selector:
    app: product-service
  ports:
    - port: 8080
      targetPort: 80
```

**order-service.yaml:**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: order-service
spec:
  replicas: 2
  selector:
    matchLabels:
      app: order-service
  template:
    metadata:
      labels:
        app: order-service
        tier: backend
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
  name: order-service
spec:
  clusterIP: None
  selector:
    app: order-service
  ports:
    - port: 8080
      targetPort: 80
```

**Verification:**

```bash
kubectl apply -f frontend.yaml
kubectl apply -f api-gateway.yaml
kubectl apply -f product-service.yaml
kubectl apply -f order-service.yaml

kubectl get deployments
kubectl get services
kubectl get endpoints
```

All four deployments should show the correct replica count. All four services should have non-empty endpoints.

## Part C: Verify the Communication Chain

**Step 1: External access to frontend:**

```bash
kubectl get service frontend-service
```

Note the nodePort (auto-assigned in the 30000-32767 range). Get the node IP:

```bash
kubectl get nodes -o wide
curl http://<node-ip>:<nodePort>
```

This should return the nginx welcome page, confirming external access.

**Step 2: Internal access from within the cluster:**

```bash
kubectl run verify --image=curlimages/curl:8.4.0 --rm -it -- sh
curl http://api-gateway:8080
curl http://product-service:8080
curl http://order-service:8080
exit
```

All three should return the nginx welcome page. Note that `api-gateway`, `product-service`, and `order-service` are all ClusterIP services, so they are only reachable from within the cluster.

**Step 3: Headless service DNS:**

```bash
kubectl run dns-check --image=busybox:1.36 --rm -it -- sh
nslookup order-service
exit
```

Output should show 2 IP addresses (one for each pod replica):

```
Name:      order-service
Address 1: 10.244.0.8
Address 2: 10.244.1.5
```

A headless service with 2 replicas resolves to 2 individual pod IPs. This is different from the other ClusterIP services, which return a single virtual IP.

## Part D: Analyze and Answer

**1. Why is the Frontend a NodePort but the API Gateway is a ClusterIP?**

The Frontend must be accessible from the internet (user browsers need to reach it), so it needs external access via NodePort. The API Gateway only receives traffic from the Frontend, which is inside the cluster, so ClusterIP is sufficient. This addresses the security concern of **minimizing attack surface**. By making the API Gateway a ClusterIP, it is invisible to the internet. An attacker cannot directly reach the API Gateway -- they must go through the Frontend first. If the API Gateway were also a NodePort, an attacker could bypass the Frontend entirely and send requests directly to the gateway, potentially exploiting vulnerabilities or bypassing authentication that the Frontend would enforce.

**2. Why is the Order Service headless?**

The Order Service is headless because it is **stateful and maintains connections**. Each instance must be individually addressable so that clients can maintain session affinity to a specific pod. A regular ClusterIP Service returns a single virtual IP, and kube-proxy load-balances connections across pods randomly. The client cannot control which pod it connects to. With a headless Service, DNS returns all pod IPs, and the client (or a service mesh) can choose which pod to connect to. This enables:

- **Session affinity:** Route a specific user's requests to the same pod that holds their session state.
- **Sticky connections:** Maintain long-lived connections (like WebSocket) to a specific pod.
- **Stateful coordination:** Allow pods to discover each other for leader election or data replication.

**3. What happens if you use the same label (`app: backend`) for both Product Service and Order Service pods?**

Routing would **not** work correctly. Two problems would occur:

First, both Services would route to the same set of pods. If `product-service` and `order-service` both have selector `app: backend`, both Services would load-balance across all 5 pods (3 Product + 2 Order). A request intended for the Product Service could end up at an Order Service pod, and vice versa.

Second, the Deployments would conflict. If both Deployments create pods with label `app: backend`, each Deployment's selector would match all pods with that label. The Deployment controller would try to manage pods that belong to the other Deployment, causing unpredictable scaling and rollout behavior.

Labels must be unique enough to distinguish workloads. Use specific labels like `app: product-service` and `app: order-service` to ensure each Service and Deployment targets only its own pods.

**4. If the Product Service scales from 3 to 5 replicas, what changes automatically?**

Two Kubernetes objects update automatically without manual intervention:

- **Endpoints object:** The `product-service` Endpoints object is automatically updated to include the IPs of the 2 new pods. The endpoints controller watches for pod changes matching the service selector and adds/removes IPs in real time. Before scaling, the Endpoints lists 3 IPs; after scaling, it lists 5.

- **ReplicaSet:** The Deployment's ReplicaSet is updated to reflect the new desired replica count. The ReplicaSet controller creates 2 new pods to reach the target of 5 replicas.

The Service object itself does not change. Its selector still matches `app: product-service`, and the new pods have that label, so they are automatically included. This is the beauty of the selector-based model: you never need to manually register or deregister pods with a service.

### Common Mistakes to Avoid

- **Label collisions between Deployments.** If two Deployments share the same label, their pod selectors overlap. Each Deployment will try to manage the other's pods. Always use unique, specific labels.
- **Service selector does not match pod labels.** If the Service selector is `app: product-service` but the pods are labeled `app: product`, the Endpoints will be empty. The Service exists but routes to nothing.
- **Forgetting that NodePort includes ClusterIP.** The Frontend Service is NodePort, but it also has a ClusterIP. Pods inside the cluster can reach it via `http://frontend-service:80`. Do not create a separate ClusterIP service for internal access to a NodePort service.
- **Using the same port for different services and expecting routing.** Each Service has its own ClusterIP and its own port space. Two services can both use port 8080 without conflict because they have different ClusterIPs.
- **Confusing headless with "no service."** A headless Service still exists in the API server, still has a DNS entry, and still has an Endpoints object. It just does not get a virtual IP. It is a valid Service, not the absence of one.

## Key Takeaway

Designing a microservice topology requires choosing the right Service type for each component based on its access requirements. Use ClusterIP by default for internal services to minimize attack surface. Use NodePort or LoadBalancer only for components that must be externally accessible. Use headless Services when clients need to address individual pods (stateful workloads, session affinity). Consistent, unique labeling is critical -- selectors are the glue that connects Services to pods, and overlapping labels cause routing conflicts.
