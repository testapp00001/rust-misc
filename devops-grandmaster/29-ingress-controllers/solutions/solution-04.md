# Solution 04: Rate Limiting and Authentication at the Ingress

---

## Part A: Deploy the Application Stack

### API Backend

```yaml
# api-backend.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api
spec:
  replicas: 3
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
          args: ["-text=API OK", "-listen=:5678"]
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

### Admin Panel

```yaml
# admin-panel.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: admin
spec:
  replicas: 1
  selector:
    matchLabels:
      app: admin
  template:
    metadata:
      labels:
        app: admin
    spec:
      containers:
        - name: echo
          image: hashicorp/http-echo:0.2.3
          args: ["-text=Admin Panel", "-listen=:5678"]
          ports:
            - containerPort: 5678
---
apiVersion: v1
kind: Service
metadata:
  name: admin-svc
spec:
  selector:
    app: admin
  ports:
    - port: 5678
      targetPort: 5678
```

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
          args: ["echo '<h1>Public Frontend</h1>' > /usr/share/nginx/html/index.html && nginx -g 'daemon off;'"]
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

---

## Part B: Configure Rate Limiting

```yaml
# api-ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: api-ingress
  annotations:
    nginx.ingress.kubernetes.io/limit-rps: "5"
    nginx.ingress.kubernetes.io/limit-burst-multiplier: "3"
spec:
  ingressClassName: nginx
  rules:
    - host: api.local
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: api-svc
                port:
                  number: 5678
```

**Test results:**

```bash
for i in $(seq 1 20); do
  echo -n "Request $i: "
  curl -s -o /dev/null -w "%{http_code}" http://api.local/
  echo
done
```

Typical output:
```
Request 1: 200
Request 2: 200
...
Request 15: 200
Request 16: 429
Request 17: 429
...
Request 20: 429
```

**Answers:**

1. **At what point do requests start getting rejected?** After the burst limit is exhausted. With `limit-rps: "5"` and `limit-burst-multiplier: "3"`, the burst is 15 requests. The first 15 requests succeed (the burst), then subsequent requests within the same second get 429.

2. **What does `limit-burst-multiplier` control?** It multiplies the `limit-rps` value to determine the burst size. A multiplier of 3 with 5 rps means the burst limit is 15 requests. The burst allows temporary spikes above the sustained rate.

3. **How does the rate limiter track clients?** By client IP address. Nginx Ingress uses the `$binary_remote_addr` variable as the rate limit key. All requests from the same IP share the same rate limit bucket.

**Why this works:** The Nginx Ingress controller translates the `limit-rps` annotation into an Nginx `limit_req_zone` directive. This uses Nginx's built-in rate limiting module, which tracks request rates per IP using a shared memory zone.

---

## Part C: Configure Basic Authentication

### Create the password file:

```bash
htpasswd -c auth admin
# Enter password: secretpass
```

### Create the Secret:

```bash
kubectl create secret generic basic-auth --from-file=auth
```

### Ingress with basic auth:

```yaml
# admin-ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: admin-ingress
  annotations:
    nginx.ingress.kubernetes.io/auth-type: basic
    nginx.ingress.kubernetes.io/auth-secret: basic-auth
    nginx.ingress.kubernetes.io/auth-realm: "Admin Access Restricted"
spec:
  ingressClassName: nginx
  rules:
    - host: admin.local
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: admin-svc
                port:
                  number: 5678
```

### Test:

```bash
# Without credentials
curl -v http://admin.local/
# HTTP/1.1 401 Unauthorized
# WWW-Authenticate: Basic realm="Admin Access Restricted"

# With credentials
curl -u admin:secretpass http://admin.local/
# Returns: Admin Panel
```

**Why this works:** The `auth-type: basic` annotation tells the Nginx Ingress controller to enable HTTP Basic Authentication. The `auth-secret` annotation references a Kubernetes Secret containing an htpasswd file. The Nginx configuration includes `auth_basic` and `auth_basic_user_file` directives that check credentials against the htpasswd file.

**Important:** Basic Authentication sends credentials base64-encoded (not encrypted) with every request. Always use HTTPS in production to prevent credential interception. The Ingress controller's TLS termination ensures credentials are encrypted in transit.

---

## Part D: Add Security Headers

```yaml
# frontend-ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: frontend-ingress
  annotations:
    nginx.ingress.kubernetes.io/configuration-snippet: |
      more_set_headers "X-Frame-Options: DENY";
      more_set_headers "X-Content-Type-Options: nosniff";
      more_set_headers "X-XSS-Protection: 1; mode=block";
      more_set_headers "Referrer-Policy: strict-origin-when-cross-origin";
spec:
  ingressClassName: nginx
  rules:
    - host: frontend.local
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: frontend-svc
                port:
                  number: 80
```

### Verify:

```bash
curl -I http://frontend.local/
```

Expected headers:
```
X-Frame-Options: DENY
X-Content-Type-Options: nosniff
X-XSS-Protection: 1; mode=block
Referrer-Policy: strict-origin-when-cross-origin
```

**Answers:**

1. **X-Frame-Options: DENY** protects against clickjacking attacks. It prevents the page from being embedded in an iframe on another site. Attackers use iframes to trick users into clicking on hidden elements.

2. **X-Content-Type-Options: nosniff** protects against MIME type sniffing. It tells the browser to trust the `Content-Type` header and not guess the content type. This prevents attacks where a file disguised as an image is executed as JavaScript.

3. **Why set headers at the Ingress layer?** It ensures consistent enforcement across all backends. Developers do not need to remember to add headers in every application. Security policies can be updated without redeploying applications. It is a defense-in-depth approach.

---

## Part E: Combine All Policies

Since Nginx Ingress annotations apply to the entire Ingress resource, different annotations for different paths require separate Ingress resources.

### Frontend Ingress (security headers):

```yaml
# combined-frontend.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: combined-frontend
  annotations:
    nginx.ingress.kubernetes.io/configuration-snippet: |
      more_set_headers "X-Frame-Options: DENY";
      more_set_headers "X-Content-Type-Options: nosniff";
spec:
  ingressClassName: nginx
  rules:
    - host: app.local
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: frontend-svc
                port:
                  number: 80
```

### API Ingress (rate limiting):

```yaml
# combined-api.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: combined-api
  annotations:
    nginx.ingress.kubernetes.io/limit-rps: "5"
    nginx.ingress.kubernetes.io/limit-burst-multiplier: "3"
spec:
  ingressClassName: nginx
  rules:
    - host: app.local
      http:
        paths:
          - path: /api
            pathType: Prefix
            backend:
              service:
                name: api-svc
                port:
                  number: 5678
```

### Admin Ingress (basic auth):

```yaml
# combined-admin.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: combined-admin
  annotations:
    nginx.ingress.kubernetes.io/auth-type: basic
    nginx.ingress.kubernetes.io/auth-secret: basic-auth
    nginx.ingress.kubernetes.io/auth-realm: "Admin Access Restricted"
spec:
  ingressClassName: nginx
  rules:
    - host: app.local
      http:
        paths:
          - path: /admin
            pathType: Prefix
            backend:
              service:
                name: admin-svc
                port:
                  number: 5678
```

**Answers:**

1. **Can you apply different annotations to different paths within a single Ingress?** No. Nginx Ingress annotations apply to the entire Ingress resource. There is no way to scope annotations to individual paths.

2. **What is the recommended approach?** Create separate Ingress resources for each path group that needs different annotations. Use the same `host` in all of them. The Nginx Ingress controller merges Ingress resources with the same host into a single Nginx server block.

3. **How does Nginx Ingress handle conflicting annotations?** If two Ingress resources for the same host have conflicting annotations, the behavior is undefined (last-write-wins or first-write-wins depending on the controller version). Avoid conflicts by ensuring each path appears in only one Ingress resource.

---

## Common Mistakes

1. **Expecting per-path annotations in a single Ingress.** This is a common misconception. Annotations are metadata on the Ingress object, not on individual paths. Separate Ingress resources are the standard solution.

2. **Using basic auth without HTTPS.** Basic auth sends credentials base64-encoded in every request. Without TLS, anyone on the network can intercept them. Always combine basic auth with TLS termination.

3. **Not testing rate limiting with burst.** The burst allows temporary spikes. If you test with a steady rate below `limit-rps`, you will never see 429 responses. Test with rapid-fire requests to exhaust the burst.

4. **Forgetting the `auth` key in the Secret.** The `auth-secret` annotation expects a Secret with a key named `auth`. If you create the Secret from a file named `passwords`, the key will be `passwords`, not `auth`, and authentication will fail.

5. **Not understanding the rate limit key.** Rate limiting is per client IP. If all your test requests come from the same IP (e.g., localhost), they share one rate limit bucket. In production, each client IP gets its own bucket.
