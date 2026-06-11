# Solution 02: Nginx Ingress with Path-Based Routing

---

## Part A: Install the Nginx Ingress Controller

**For minikube:**

```bash
minikube addons enable ingress
```

**For other clusters:**

```bash
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.9.0/deploy/static/provider/cloud/deploy.yaml
```

**Verification:**

```bash
kubectl get pods -n ingress-nginx
```

Expected output shows a Pod with `READY 1/1` and `STATUS Running`. The controller runs in the `ingress-nginx` namespace.

---

## Part B: Deploy Backend Services

### Frontend

```yaml
# frontend.yaml
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
    spec:
      containers:
        - name: nginx
          image: nginx:1.25
          command: ["/bin/sh", "-c"]
          args: ["echo '<h1>Welcome to the Frontend</h1>' > /usr/share/nginx/html/index.html && nginx -g 'daemon off;'"]
          ports:
            - containerPort: 80
---
apiVersion: v1
kind: Service
metadata:
  name: frontend-svc
spec:
  selector:
    app: frontend
  ports:
    - port: 80
      targetPort: 80
```

### API Backend

```yaml
# api.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api
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
        - name: echo
          image: hashicorp/http-echo:0.2.3
          args: ["-text=API Response", "-listen=:5678"]
          ports:
            - containerPort: 5678
---
apiVersion: v1
kind: Service
metadata:
  name: api-svc
spec:
  selector:
    app: api
  ports:
    - port: 5678
      targetPort: 5678
```

### Docs Service

```yaml
# docs.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: docs
spec:
  replicas: 1
  selector:
    matchLabels:
      app: docs
  template:
    metadata:
      labels:
        app: docs
    spec:
      containers:
        - name: nginx
          image: nginx:1.25
          command: ["/bin/sh", "-c"]
          args: ["echo '<h1>API Documentation</h1>' > /usr/share/nginx/html/index.html && nginx -g 'daemon off;'"]
          ports:
            - containerPort: 80
---
apiVersion: v1
kind: Service
metadata:
  name: docs-svc
spec:
  selector:
    app: docs
  ports:
    - port: 80
      targetPort: 80
```

**Why this works:** All three Services are ClusterIP (the default type). They are only reachable from within the cluster. The Ingress controller, running inside the cluster, can route traffic to them. No NodePort or LoadBalancer is needed for the backends.

---

## Part C: Create the Ingress Resource

```yaml
# path-ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: path-routing
spec:
  ingressClassName: nginx
  rules:
    - host: myapp.local
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: frontend-svc
                port:
                  number: 80
          - path: /api
            pathType: Prefix
            backend:
              service:
                name: api-svc
                port:
                  number: 5678
          - path: /docs
            pathType: Prefix
            backend:
              service:
                name: docs-svc
                port:
                  number: 80
```

**Why this works:** The Ingress defines routing rules for host `myapp.local`. When a request arrives with that `Host` header, the Ingress controller matches the URL path against the rules. With `pathType: Prefix`, `/api` matches `/api`, `/api/v1`, `/api/users`, etc. The first matching path wins, so `/` catches everything that does not match `/api` or `/docs`.

---

## Part D: Test Path-Based Routing

**Get the external IP:**

```bash
# For minikube
minikube ip

# For cloud clusters
kubectl get service -n ingress-nginx -o jsonpath='{.items[0].status.loadBalancer.ingress[0].ip}'
```

**Add to /etc/hosts:**

```bash
echo "$(minikube ip) myapp.local" | sudo tee -a /etc/hosts
```

**Test:**

```bash
curl http://myapp.local/
# Returns: <h1>Welcome to the Frontend</h1>

curl http://myapp.local/api
# Returns: API Response

curl http://myapp.local/docs
# Returns: <h1>API Documentation</h1>
```

**Additional tests:**

```bash
# Prefix matching: /api/v1 also matches /api
curl http://myapp.local/api/v1
# Returns: API Response

# Unknown path falls through to root
curl http://myapp.local/unknown
# Returns: <h1>Welcome to the Frontend</h1>
```

---

## Part E: Inspect the Generated Configuration

```bash
POD=$(kubectl get pods -n ingress-nginx -l app.kubernetes.io/name=ingress-nginx -o jsonpath='{.items[0].metadata.name}')
kubectl exec -n ingress-nginx $POD -- cat /etc/nginx/nginx.conf | grep -A 20 "location /api"
```

**Answers:**

1. **How does the Ingress controller map `/api` to `api-svc`?** The controller generates an Nginx `location /api` block with a `proxy_pass` directive pointing to the ClusterIP and port of `api-svc`. Nginx handles the actual HTTP proxying.

2. **What happens with `/api/v1/users`?** With `pathType: Prefix`, the path `/api` matches any URL that starts with `/api`. So `/api/v1/users` matches and is routed to `api-svc`. The `http-echo` server receives the full path but returns "API Response" regardless.

3. **What happens with `/unknown`?** It does not match `/api` or `/docs`, so it falls through to the root path `/` and is routed to `frontend-svc`.

---

## Common Mistakes

1. **Using `pathType: Exact` when you want prefix matching.** `Exact` means the path must match precisely. `/api` with `Exact` would not match `/api/v1`. Use `Prefix` for path-based routing where subpaths should also match.

2. **Forgetting the `ingressClassName` field.** Without `ingressClassName: nginx`, the Ingress resource is not handled by any controller (unless there is a default IngressClass). Always specify the class explicitly.

3. **Using LoadBalancer Services for backends.** The backends should be ClusterIP Services. The Ingress controller routes to them from inside the cluster. Using LoadBalancer for backends wastes external IPs.

4. **Not adding `/etc/hosts` entries.** For local testing, the hostname must resolve to the Ingress controller's IP. Without the `/etc/hosts` entry, curl cannot reach the controller.

5. **Confusing the Ingress controller Service with backend Services.** The Ingress controller has its own LoadBalancer Service (in `ingress-nginx` namespace). The backend Services are ClusterIP. Traffic flows: Internet -> Ingress Controller LB -> Ingress Controller Pod -> Backend ClusterIP Service -> Backend Pod.
