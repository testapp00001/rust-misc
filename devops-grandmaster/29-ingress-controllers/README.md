# Module 29: Ingress Controllers — HTTP Routing and TLS Termination

**Previous:** [Module 28: Services & Networking](../28-services-and-networking/README.md)

---

## The Problem

You have services exposing your applications internally (ClusterIP) and externally (NodePort/LoadBalancer). But for a production web application, you need:

- `api.example.com` routes to your API service
- `app.example.com` routes to your frontend service
- `/admin` routes to the admin service
- All traffic uses HTTPS with valid TLS certificates
- Automatic certificate renewal before expiration
- Rate limiting, authentication, and other HTTP-level features

A LoadBalancer service gives you one external IP per service. If you have 10 microservices, you need 10 external IPs and 10 load balancers. This is expensive and unmanageable. You need a single entry point that routes HTTP traffic based on hostnames and paths.

---

## The Naive Way

Create a LoadBalancer service for each application:

```yaml
# Frontend load balancer
apiVersion: v1
kind: Service
metadata:
  name: frontend-lb
spec:
  type: LoadBalancer
  selector:
    app: frontend
  ports:
    - port: 443
      targetPort: 80

---
# API load balancer
apiVersion: v1
kind: Service
metadata:
  name: api-lb
spec:
  type: LoadBalancer
  selector:
    app: api
  ports:
    - port: 443
      targetPort: 8080

---
# Admin load balancer
apiVersion: v1
kind: Service
metadata:
  name: admin-lb
spec:
  type: LoadBalancer
  selector:
    app: admin
  ports:
    - port: 443
      targetPort: 8080
```

**What goes wrong:**

- 3 external IPs and 3 cloud load balancers = 3x cost
- TLS certificates must be managed separately for each load balancer
- No HTTP routing (path-based, host-based)
- No centralized logging, rate limiting, or authentication
- DNS must point to multiple IPs

---

## The Right Way

### Ingress Resource vs Ingress Controller

**Ingress Resource** is a Kubernetes API object that defines routing rules:

```yaml
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: app-ingress
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
spec:
  ingressClassName: nginx
  rules:
    - host: app.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: frontend-service
                port:
                  number: 80
          - path: /api
            pathType: Prefix
            backend:
              service:
                name: api-service
                port:
                  number: 8080
```

**Ingress Controller** is the actual software that implements the Ingress resource. It watches for Ingress objects and configures a reverse proxy (Nginx, Traefik, HAProxy, etc.).

```
Internet → Ingress Controller (Nginx/Traefik) → Service → Pods
```

**You must install an Ingress Controller.** Kubernetes does not include one by default.

### Path-Based Routing

Route traffic based on URL path:

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: path-routing
spec:
  ingressClassName: nginx
  rules:
    - host: myapp.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: frontend
                port:
                  number: 80
          - path: /api
            pathType: Prefix
            backend:
              service:
                name: api
                port:
                  number: 8080
          - path: /admin
            pathType: Prefix
            backend:
              service:
                name: admin
                port:
                  number: 8080
```

**Path types:**
- `Prefix` — matches based on URL path prefix (`/api` matches `/api/v1`, `/api/users`)
- `Exact` — matches exactly (`/api` only matches `/api`, not `/api/v1`)
- `ImplementationSpecific` — depends on the ingress controller

### Host-Based Routing

Route traffic based on hostname:

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: host-routing
spec:
  ingressClassName: nginx
  rules:
    - host: web.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: frontend
                port:
                  number: 80
    - host: api.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: api
                port:
                  number: 8080
    - host: admin.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: admin
                port:
                  number: 8080
```

### TLS/SSL Termination

Terminate HTTPS at the ingress controller and forward plain HTTP to backend services:

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: tls-ingress
spec:
  ingressClassName: nginx
  tls:
    - hosts:
        - app.example.com
        - api.example.com
      secretName: tls-secret    # Kubernetes Secret with cert and key
  rules:
    - host: app.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: frontend
                port:
                  number: 80
    - host: api.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: api
                port:
                  number: 8080
```

**TLS Secret:**

```bash
# Create a TLS secret from certificate files
kubectl create secret tls tls-secret \
  --cert=tls.crt \
  --key=tls.key
```

```yaml
# Or as YAML
apiVersion: v1
kind: Secret
metadata:
  name: tls-secret
type: kubernetes.io/tls
data:
  tls.crt: <base64-encoded-certificate>
  tls.key: <base64-encoded-private-key>
```

### cert-manager for Automatic Certificates

**cert-manager** automates TLS certificate management using Let's Encrypt:

```bash
# Install cert-manager
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.14.0/cert-manager.yaml
```

```yaml
# ClusterIssuer for Let's Encrypt
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: admin@example.com
    privateKeySecretRef:
      name: letsencrypt-prod-key
    solvers:
      - http01:
          ingress:
            class: nginx
```

```yaml
# Ingress with automatic TLS
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: auto-tls-ingress
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
spec:
  ingressClassName: nginx
  tls:
    - hosts:
        - app.example.com
      secretName: app-tls    # cert-manager creates this automatically
  rules:
    - host: app.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: frontend
                port:
                  number: 80
```

cert-manager will:
1. Detect the annotation on the Ingress
2. Request a certificate from Let's Encrypt
3. Complete the HTTP-01 challenge
4. Store the certificate in the `app-tls` secret
5. Automatically renew before expiration

---

## The Production Way

### Nginx Ingress Controller

Install the Nginx Ingress Controller:

```bash
# Using Helm
helm repo add ingress-nginx https://kubernetes.github.io/ingress-nginx
helm repo update
helm install ingress-nginx ingress-nginx/ingress-nginx \
  --namespace ingress-nginx \
  --create-namespace

# Or using kubectl
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.9.0/deploy/static/provider/cloud/deploy.yaml
```

**Verify installation:**

```bash
kubectl get pods -n ingress-nginx
kubectl get service -n ingress-nginx
```

### Traefik Ingress Controller

```bash
# Using Helm
helm repo add traefik https://traefik.github.io/charts
helm repo update
helm install traefik traefik/traefik \
  --namespace traefik \
  --create-namespace
```

### Common Nginx Ingress Annotations

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: annotated-ingress
  annotations:
    # Rate limiting
    nginx.ingress.kubernetes.io/limit-rps: "10"
    nginx.ingress.kubernetes.io/limit-burst-multiplier: "5"

    # CORS
    nginx.ingress.kubernetes.io/enable-cors: "true"
    nginx.ingress.kubernetes.io/cors-allow-origin: "https://frontend.example.com"

    # Rewrite target
    nginx.ingress.kubernetes.io/rewrite-target: /$2

    # SSL redirect
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/force-ssl-redirect: "true"

    # Backend protocol (for gRPC)
    nginx.ingress.kubernetes.io/backend-protocol: "GRPC"

    # Proxy body size (for file uploads)
    nginx.ingress.kubernetes.io/proxy-body-size: "50m"

    # Timeouts
    nginx.ingress.kubernetes.io/proxy-connect-timeout: "10"
    nginx.ingress.kubernetes.io/proxy-read-timeout: "120"
    nginx.ingress.kubernetes.io/proxy-send-timeout: "120"

    # Basic authentication
    nginx.ingress.kubernetes.io/auth-type: basic
    nginx.ingress.kubernetes.io/auth-secret: basic-auth
    nginx.ingress.kubernetes.io/auth-realm: "Authentication Required"

    # Custom headers
    nginx.ingress.kubernetes.io/configuration-snippet: |
      more_set_headers "X-Frame-Options: DENY";
      more_set_headers "X-Content-Type-Options: nosniff";
spec:
  ingressClassName: nginx
  rules:
    - host: app.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: frontend
                port:
                  number: 80
```

### Default Backend

Configure a default backend for unmatched requests:

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: ingress-with-default
spec:
  ingressClassName: nginx
  defaultBackend:
    service:
      name: default-frontend
      port:
        number: 80
  rules:
    - host: app.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: frontend
                port:
                  number: 80
```

### Path Rewrite

Strip the path prefix before forwarding to the backend:

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: rewrite-ingress
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /$2
spec:
  ingressClassName: nginx
  rules:
    - host: app.example.com
      http:
        paths:
          - path: /api(/|$)(.*)
            pathType: ImplementationSpecific
            backend:
              service:
                name: api-service
                port:
                  number: 8080
```

Request to `/api/users` is forwarded to `api-service:8080/users`.

---

## Hands-On Lab

### Prerequisites

- A running Kubernetes cluster
- `kubectl` configured
- A cloud provider or minikube with ingress addon

### Exercise 1: Install Nginx Ingress Controller

```bash
# For minikube
minikube addons enable ingress

# For cloud providers (using kubectl)
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.9.0/deploy/static/provider/cloud/deploy.yaml

# Verify installation
kubectl get pods -n ingress-nginx
kubectl get service -n ingress-nginx
```

### Exercise 2: Create Backend Services

```bash
# Create frontend deployment and service
kubectl apply -f - <<EOF
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
          ports:
            - containerPort: 80
---
apiVersion: v1
kind: Service
metadata:
  name: frontend-service
spec:
  selector:
    app: frontend
  ports:
    - port: 80
      targetPort: 80
EOF

# Create API deployment and service
kubectl apply -f - <<EOF
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
          args:
            - "-text=API Response"
            - "-listen=:8080"
          ports:
            - containerPort: 8080
---
apiVersion: v1
kind: Service
metadata:
  name: api-service
spec:
  selector:
    app: api
  ports:
    - port: 8080
      targetPort: 8080
EOF
```

### Exercise 3: Path-Based Routing

```bash
kubectl apply -f - <<EOF
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: path-ingress
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /\$2
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
                name: frontend-service
                port:
                  number: 80
          - path: /api(/|$\)(.*)
            pathType: ImplementationSpecific
            backend:
              service:
                name: api-service
                port:
                  number: 8080
EOF

# Get the ingress IP
kubectl get ingress path-ingress

# Add to /etc/hosts (for local testing)
echo "$(minikube ip) myapp.local" | sudo tee -a /etc/hosts

# Test path routing
curl http://myapp.local/
curl http://myapp.local/api
```

### Exercise 4: Host-Based Routing

```bash
kubectl apply -f - <<EOF
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: host-ingress
spec:
  ingressClassName: nginx
  rules:
    - host: web.myapp.local
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: frontend-service
                port:
                  number: 80
    - host: api.myapp.local
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: api-service
                port:
                  number: 8080
EOF

# Add hosts entries
echo "$(minikube ip) web.myapp.local api.myapp.local" | sudo tee -a /etc/hosts

# Test host routing
curl http://web.myapp.local/
curl http://api.myapp.local/
```

### Exercise 5: TLS Termination

```bash
# Generate a self-signed certificate
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout tls.key -out tls.crt \
  -subj "/CN=myapp.local" \
  -addext "subjectAltName=DNS:myapp.local"

# Create the TLS secret
kubectl create secret tls myapp-tls --cert=tls.crt --key=tls.key

# Create ingress with TLS
kubectl apply -f - <<EOF
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: tls-ingress
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  ingressClassName: nginx
  tls:
    - hosts:
        - myapp.local
      secretName: myapp-tls
  rules:
    - host: myapp.local
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: frontend-service
                port:
                  number: 80
          - path: /api
            pathType: Prefix
            backend:
              service:
                name: api-service
                port:
                  number: 8080
EOF

# Test HTTPS (ignore self-signed cert warning)
curl -k https://myapp.local/
curl -k https://myapp.local/api
```

### Exercise 6: Ingress Annotations

```bash
kubectl apply -f - <<EOF
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: annotated-ingress
  annotations:
    nginx.ingress.kubernetes.io/limit-rps: "5"
    nginx.ingress.kubernetes.io/proxy-body-size: "10m"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/configuration-snippet: |
      more_set_headers "X-Custom-Header: my-value";
spec:
  ingressClassName: nginx
  tls:
    - hosts:
        - myapp.local
      secretName: myapp-tls
  rules:
    - host: myapp.local
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: frontend-service
                port:
                  number: 80
EOF

# Verify headers
curl -k -I https://myapp.local/
```

### Exercise 7: Inspect Ingress Internals

```bash
# View ingress details
kubectl describe ingress path-ingress

# Check ingress controller logs
kubectl logs -n ingress-nginx -l app.kubernetes.io/name=ingress-nginx

# View the generated nginx config
kubectl exec -n ingress-nginx $(kubectl get pods -n ingress-nginx -l app.kubernetes.io/name=ingress-nginx -o jsonpath='{.items[0].metadata.name}') -- cat /etc/nginx/nginx.conf

# Clean up
kubectl delete ingress --all
kubectl delete deployment frontend api
kubectl delete service frontend-service api-service
kubectl delete secret myapp-tls
```

---

## Verification Checklist

- [ ] Understand the difference between Ingress resource and Ingress controller
- [ ] Can set up path-based routing
- [ ] Can set up host-based routing
- [ ] Can configure TLS termination with a certificate
- [ ] Understand how cert-manager automates certificate management
- [ ] Know common Nginx ingress annotations
- [ ] Can inspect ingress controller logs and configuration

---

## Limitation

You have learned how to route traffic to your services and secure it with TLS. But your application needs configuration: database connection strings, API keys, feature flags, environment-specific settings. Hardcoding these in your container images means rebuilding the image for every environment. Passing them as environment variables in the Deployment YAML means they are visible to anyone with `kubectl` access.

**Next:** [Module 30: ConfigMaps & Secrets](../30-configmaps-and-secrets/README.md) — Externalize configuration from your application code.
