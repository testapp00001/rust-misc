# Solution 05: Ingress Strategy for Multi-Team Clusters

---

## Part A: Create Team Namespaces and Deploy Applications

### Namespaces:

```bash
kubectl create namespace payments
kubectl create namespace catalog
kubectl create namespace platform
```

### Payments Application:

```yaml
# payments-app.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: payments-api
  namespace: payments
spec:
  replicas: 2
  selector:
    matchLabels:
      app: payments-api
  template:
    metadata:
      labels:
        app: payments-api
    spec:
      containers:
        - name: echo
          image: hashicorp/http-echo:0.2.3
          args: ["-text=Payments Service v2.1", "-listen=:5678"]
          ports:
            - containerPort: 5678
---
apiVersion: v1
kind: Service
metadata:
  name: payments-svc
  namespace: payments
spec:
  selector:
    app: payments-api
  ports:
    - port: 5678
      targetPort: 5678
```

### Catalog Application:

```yaml
# catalog-app.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: catalog-api
  namespace: catalog
spec:
  replicas: 2
  selector:
    matchLabels:
      app: catalog-api
  template:
    metadata:
      labels:
        app: catalog-api
    spec:
      containers:
        - name: echo
          image: hashicorp/http-echo:0.2.3
          args: ["-text=Catalog Service v3.4", "-listen=:5678"]
          ports:
            - containerPort: 5678
---
apiVersion: v1
kind: Service
metadata:
  name: catalog-svc
  namespace: catalog
spec:
  selector:
    app: catalog-api
  ports:
    - port: 5678
      targetPort: 5678
```

### Platform Application:

```yaml
# platform-app.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: landing-page
  namespace: platform
spec:
  replicas: 2
  selector:
    matchLabels:
      app: landing-page
  template:
    metadata:
      labels:
        app: landing-page
    spec:
      containers:
        - name: nginx
          image: nginx:1.25
          command: ["/bin/sh", "-c"]
          args: ["echo '<h1>Welcome to Our Platform</h1><p>Service not found.</p>' > /usr/share/nginx/html/index.html && nginx -g 'daemon off;'"]
          ports:
            - containerPort: 80
---
apiVersion: v1
kind: Service
metadata:
  name: landing-svc
  namespace: platform
spec:
  selector:
    app: landing-page
  ports:
    - port: 80
      targetPort: 80
```

**Verification:**

```bash
kubectl get deployments --all-namespaces | grep -E "payments|catalog|platform"
kubectl get services --all-namespaces | grep -E "payments|catalog|platform"
```

---

## Part B: Create Team Ingress Resources

### Payments Ingress:

```yaml
# payments-ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: payments-ingress
  namespace: payments
spec:
  ingressClassName: nginx
  rules:
    - host: payments.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: payments-svc
                port:
                  number: 5678
```

### Catalog Ingress:

```yaml
# catalog-ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: catalog-ingress
  namespace: catalog
spec:
  ingressClassName: nginx
  rules:
    - host: catalog.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: catalog-svc
                port:
                  number: 5678
```

**Answers:**

1. **Can the payments team's Ingress reference a Service in the `catalog` namespace?** No. An Ingress can only reference Services in its own namespace. This is a Kubernetes security boundary. The Ingress resource in `payments` namespace can only route to Services in `payments`.

2. **How does the Ingress controller match requests?** The controller reads the `Host` header and URL path from incoming requests. It searches all Ingress resources across all namespaces for a matching rule. The first match determines which backend Service receives the traffic.

3. **What happens for `unknown.example.com`?** If no Ingress rule matches the hostname, the request is sent to the default backend. If no default backend is configured, the Ingress controller returns its generic "default backend - 404" page.

---

## Part C: Implement a Default Backend

### Default Backend Deployment:

```yaml
# default-backend.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: default-backend
  namespace: platform
spec:
  replicas: 1
  selector:
    matchLabels:
      app: default-backend
  template:
    metadata:
      labels:
        app: default-backend
    spec:
      containers:
        - name: nginx
          image: nginx:1.25
          command: ["/bin/sh", "-c"]
          args:
            - |
              echo '<h1>404 - Service Not Found</h1><p>The requested service does not exist.</p>' > /usr/share/nginx/html/index.html
              echo 'server { listen 80; location / { return 404; } }' > /etc/nginx/conf.d/default.conf
              nginx -g 'daemon off;'
          ports:
            - containerPort: 80
---
apiVersion: v1
kind: Service
metadata:
  name: default-backend-svc
  namespace: platform
spec:
  selector:
    app: default-backend
  ports:
    - port: 80
      targetPort: 80
```

### Default Ingress:

```yaml
# default-ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: default-ingress
  namespace: platform
spec:
  ingressClassName: nginx
  defaultBackend:
    service:
      name: default-backend-svc
      port:
        number: 80
```

**Test:**

```bash
# Known host (should route to payments service)
curl -H "Host: payments.example.com" http://<ingress-ip>/
# Returns: Payments Service v2.1

# Unknown host (should get custom 404)
curl -H "Host: unknown.example.com" http://<ingress-ip>/
# Returns: 404 - Service Not Found
```

**Why this works:** The `defaultBackend` field defines a catch-all backend. When no Ingress rule matches the incoming request's hostname and path, the Ingress controller routes the request to this backend. Without it, the controller returns its own generic 404 page.

---

## Part D: Apply Organizational Standards

### Payments Ingress with Standards:

```yaml
# payments-ingress-secure.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: payments-ingress
  namespace: payments
  annotations:
    nginx.ingress.kubernetes.io/limit-rps: "50"
    nginx.ingress.kubernetes.io/limit-burst-multiplier: "2"
    nginx.ingress.kubernetes.io/configuration-snippet: |
      more_set_headers "X-Frame-Options: DENY";
      more_set_headers "X-Content-Type-Options: nosniff";
spec:
  ingressClassName: nginx
  rules:
    - host: payments.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: payments-svc
                port:
                  number: 5678
```

### Catalog Ingress with Standards:

```yaml
# catalog-ingress-secure.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: catalog-ingress
  namespace: catalog
  annotations:
    nginx.ingress.kubernetes.io/limit-rps: "100"
    nginx.ingress.kubernetes.io/limit-burst-multiplier: "2"
    nginx.ingress.kubernetes.io/configuration-snippet: |
      more_set_headers "X-Frame-Options: DENY";
      more_set_headers "X-Content-Type-Options: nosniff";
spec:
  ingressClassName: nginx
  rules:
    - host: catalog.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: catalog-svc
                port:
                  number: 5678
```

**Verification:**

```bash
curl -I -H "Host: payments.example.com" http://<ingress-ip>/
# Should include X-Frame-Options and X-Content-Type-Options headers
```

**Answers:**

1. **Manual annotation for 10 teams?** No, this does not scale. Manual annotation is error-prone and relies on every team remembering to add the required annotations.

2. **How to enforce automatically?** Three approaches:
   - **Kyverno/OPA Gatekeeper:** Policy engines that validate Ingress resources at admission time. Reject Ingresses missing required annotations.
   - **Admission Webhook:** A custom webhook that intercepts Ingress creation and injects default annotations.
   - **IngressClass with defaults:** Some Ingress controllers support default annotations at the IngressClass level.

3. **What if a team creates an Ingress without annotations?** Without policy enforcement, the Ingress is accepted but lacks rate limiting and security headers. The application is still accessible but unprotected. This is why policy enforcement is critical in multi-team clusters.

**Example Kyverno policy:**

```yaml
apiVersion: kyverno.io/v1
kind: ClusterPolicy
metadata:
  name: require-ingress-annotations
spec:
  validationFailureAction: Enforce
  rules:
    - name: check-security-headers
      match:
        resources:
          kinds:
            - Ingress
      validate:
        message: "Ingress must include security headers in configuration-snippet"
        pattern:
          metadata:
            annotations:
              nginx.ingress.kubernetes.io/configuration-snippet: "*X-Frame-Options*"
```

---

## Part E: TLS Strategy for Multi-Team Clusters

### Generate wildcard certificate:

```bash
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout wildcard.key -out wildcard.crt \
  -subj "/CN=*.example.com" \
  -addext "subjectAltName=DNS:*.example.com"
```

### Distribute to each namespace:

```bash
kubectl create secret tls wildcard-tls --cert=wildcard.crt --key=wildcard.key -n payments
kubectl create secret tls wildcard-tls --cert=wildcard.crt --key=wildcard.key -n catalog
kubectl create secret tls wildcard-tls --cert=wildcard.crt --key=wildcard.key -n platform
```

### Update Ingress resources with TLS:

```yaml
# Add to each Ingress:
spec:
  tls:
    - hosts:
        - payments.example.com   # or catalog.example.com, etc.
      secretName: wildcard-tls
```

**Verification:**

```bash
curl -k -H "Host: payments.example.com" https://<ingress-ip>/
curl -k -H "Host: catalog.example.com" https://<ingress-ip>/
```

**Answers:**

1. **Why must the TLS Secret be in the same namespace as the Ingress?** This is a Kubernetes security boundary. A Secret in namespace A could contain credentials that namespace B should not access. By requiring Secrets to be in the same namespace, Kubernetes ensures namespace isolation. An Ingress in `payments` cannot read Secrets from `catalog`.

2. **Wildcard vs per-service certificates:**
   - **Wildcard (`*.example.com`):** One certificate covers all subdomains. Simpler management. But if the private key is compromised, all subdomains are affected. Also, wildcard certificates cannot cover multiple levels (e.g., `*.api.example.com`).
   - **Per-service:** Each service gets its own certificate. Better isolation. But more certificates to manage and renew.

3. **How to automate certificate rotation across namespaces:**
   - **cert-manager with ClusterIssuer:** cert-manager can automatically create and renew certificates in each namespace. Add the `cert-manager.io/cluster-issuer` annotation to each Ingress, and cert-manager creates the TLS Secret in that namespace.
   - **Secret replication controller:** Use a tool like `kubernetes-replicator` or `reflector` to sync a Secret from one namespace to others.
   - **CI/CD pipeline:** A CronJob or pipeline that generates certificates and distributes them to all namespaces.

4. **What if the Secret does not exist?** The Ingress controller logs an error and serves traffic on HTTP only (no HTTPS). The Ingress resource will show a warning in its status. Clients trying to connect via HTTPS will get a connection error.

---

## Common Mistakes

1. **Assuming Ingress can cross namespace boundaries.** An Ingress can only reference Services and Secrets in its own namespace. This is a fundamental Kubernetes security constraint.

2. **Not creating a default backend.** Without a default backend, unmatched traffic returns the Ingress controller's generic page, which leaks information about the controller type and version.

3. **Distributing TLS Secrets manually.** In a multi-team cluster with many namespaces, manual Secret distribution does not scale. Use cert-manager or a Secret replication tool.

4. **Not enforcing organizational standards.** Without policy enforcement, teams will forget annotations or use inconsistent values. Use Kyverno, OPA Gatekeeper, or admission webhooks to enforce standards at admission time.

5. **Running a single Ingress controller replica.** The Ingress controller is a shared component. If it crashes, all teams lose external access. Run multiple replicas with anti-affinity rules and a PodDisruptionBudget.

6. **Ignoring Ingress controller resource limits.** In a multi-team cluster, the Ingress controller handles traffic for all teams. Set appropriate CPU and memory limits to prevent one team's traffic spike from affecting others.

7. **Not using separate IngressClasses for different controllers.** If you run multiple Ingress controllers (e.g., Nginx for internal traffic, Traefik for external), use different IngressClasses so teams can choose the appropriate controller.
