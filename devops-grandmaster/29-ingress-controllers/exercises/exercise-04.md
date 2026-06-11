# Exercise 04: Rate Limiting and Authentication at the Ingress

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Configure rate limiting, basic authentication, and custom headers on an Nginx Ingress resource using annotations. You will protect backend services from abuse, restrict access to authenticated users, and add security headers at the Ingress layer without modifying application code.

## Background

Your API backend is receiving too many requests from a single client, causing performance degradation. Your admin panel needs to be restricted to authorized users. Your security team requires specific HTTP headers on all responses. All of these requirements can be addressed at the Ingress layer using Nginx Ingress annotations, without changing a single line of application code. This is the power of Layer 7 routing: the Ingress controller can enforce policies before traffic reaches your Pods.

## Tasks

### Part A: Deploy the Application Stack

Create three services to work with.

**Step 1:** Create the API backend.

```yaml
# Save as api-backend.yaml
```

Create a Deployment named `api` with 3 replicas using image `hashicorp/http-echo:0.2.3` with args `["-text=API OK", "-listen=:5678"]` and label `app: api`. Create a ClusterIP Service named `api-svc` on port 5678.

**Step 2:** Create the admin panel.

```yaml
# Save as admin-panel.yaml
```

Create a Deployment named `admin` with 1 replica using image `hashicorp/http-echo:0.2.3` with args `["-text=Admin Panel", "-listen=:5678"]` and label `app: admin`. Create a ClusterIP Service named `admin-svc` on port 5678.

**Step 3:** Create the public frontend.

```yaml
# Save as frontend.yaml
```

Create a Deployment named `frontend` with 2 replicas using `nginx:1.25` with label `app: frontend`. Serve custom content:

```yaml
command: ["/bin/sh", "-c"]
args: ["echo '<h1>Public Frontend</h1>' > /usr/share/nginx/html/index.html && nginx -g 'daemon off;'"]
```

Create a ClusterIP Service named `frontend-svc` on port 80.

**Step 4:** Verify everything is running:

```bash
kubectl get deployments,services,pods
```

<details>
<summary>Hint -- Multiple Replicas for Rate Limiting</summary>
Use 3 replicas for the API backend so you can see the rate limiter in action. With multiple Pods, Kubernetes distributes requests across them, but the Ingress controller tracks request rates globally.
</details>

### Part B: Configure Rate Limiting

Protect the API backend from excessive requests using Nginx Ingress rate limiting annotations.

**Step 1:** Create an Ingress with rate limiting:

```yaml
# Save as api-ingress.yaml
```

Create an Ingress with:
- Name: `api-ingress`
- `ingressClassName: nginx`
- Host: `api.local`
- Path `/` to `api-svc:5678`
- Annotation: `nginx.ingress.kubernetes.io/limit-rps: "5"` (5 requests per second per IP)
- Annotation: `nginx.ingress.kubernetes.io/limit-burst-multiplier: "3"` (burst of 15 requests)

Apply:

```bash
kubectl apply -f api-ingress.yaml
```

**Step 2:** Test rate limiting. Send rapid requests:

```bash
# Send 20 requests as fast as possible
for i in $(seq 1 20); do
  echo -n "Request $i: "
  curl -s -o /dev/null -w "%{http_code}" http://api.local/
  echo
done
```

Record how many requests return 200 and how many return 429 (Too Many Requests).

**Step 3:** Answer these questions:
1. At what point do requests start getting rejected (429)?
2. What does the `limit-burst-multiplier` annotation control?
3. How does the rate limiter track clients? (By IP? By header? By cookie?)

<details>
<summary>Hint -- Rate Limiting Behavior</summary>
Nginx Ingress rate limiting tracks requests per client IP address. The `limit-rps` sets the sustained rate. The burst allows temporary spikes above the sustained rate. With `limit-rps: "5"` and `limit-burst-multiplier: "3"`, the burst limit is 5 x 3 = 15 requests. After the burst is exhausted, excess requests receive a 429 response.
</details>

### Part C: Configure Basic Authentication

Restrict access to the admin panel using HTTP Basic Authentication.

**Step 1:** Create a password file:

```bash
# Install htpasswd if not available
# apt-get install apache2-utils   (Debian/Ubuntu)
# yum install httpd-tools         (RHEL/CentOS)

# Create the password file
htpasswd -c auth admin
# Enter a password when prompted (e.g., "secretpass")
```

**Step 2:** Create a Kubernetes Secret from the password file:

```bash
kubectl create secret generic basic-auth --from-file=auth
```

Verify:

```bash
kubectl get secret basic-auth -o yaml
```

**Step 3:** Create an Ingress with basic auth:

```yaml
# Save as admin-ingress.yaml
```

Create an Ingress with:
- Name: `admin-ingress`
- `ingressClassName: nginx`
- Host: `admin.local`
- Path `/` to `admin-svc:5678`
- Annotations:
  - `nginx.ingress.kubernetes.io/auth-type: basic`
  - `nginx.ingress.kubernetes.io/auth-secret: basic-auth`
  - `nginx.ingress.kubernetes.io/auth-realm: "Admin Access Restricted"`

Apply:

```bash
kubectl apply -f admin-ingress.yaml
```

**Step 4:** Test authentication:

```bash
# Without credentials (should get 401)
curl -v http://admin.local/

# With credentials (should get 200)
curl -u admin:secretpass http://admin.local/
```

Record the response codes and headers for both requests.

<details>
<summary>Hint -- Secret Key Name</summary>
The `auth-secret` must reference a Secret that contains a key named `auth`. The value of this key is the content of an htpasswd file. If you used `kubectl create secret generic basic-auth --from-file=auth`, the key in the Secret will be `auth` (the filename). If your file is named differently, the key will match the filename.
</details>

### Part D: Add Security Headers

Add security headers to all responses from the frontend using a configuration snippet.

**Step 1:** Create an Ingress with security headers:

```yaml
# Save as frontend-ingress.yaml
```

Create an Ingress with:
- Name: `frontend-ingress`
- `ingressClassName: nginx`
- Host: `frontend.local`
- Path `/` to `frontend-svc:80`
- Annotation using `configuration-snippet`:

```yaml
nginx.ingress.kubernetes.io/configuration-snippet: |
  more_set_headers "X-Frame-Options: DENY";
  more_set_headers "X-Content-Type-Options: nosniff";
  more_set_headers "X-XSS-Protection: 1; mode=block";
  more_set_headers "Referrer-Policy: strict-origin-when-cross-origin";
```

Apply:

```bash
kubectl apply -f frontend-ingress.yaml
```

**Step 2:** Verify the headers are present:

```bash
curl -I http://frontend.local/
```

Check that all four security headers appear in the response.

**Step 3:** Answer these questions:
1. What does `X-Frame-Options: DENY` protect against?
2. What does `X-Content-Type-Options: nosniff` protect against?
3. Why is it better to set these at the Ingress layer rather than in each application?

<details>
<summary>Hint -- Header Enforcement</summary>
Setting security headers at the Ingress layer ensures they are applied consistently to all responses, regardless of the backend application. This is a defense-in-depth approach: even if a developer forgets to add headers in their application code, the Ingress controller adds them. It also means you can update security policies without redeploying applications.
</details>

### Part E: Combine All Policies

Create a single Ingress that combines rate limiting, authentication, and security headers for different paths on the same host.

```yaml
# Save as combined-ingress.yaml
```

Create an Ingress with:
- Name: `combined-ingress`
- `ingressClassName: nginx`
- Host: `app.local`
- Three paths:
  - `/` to `frontend-svc:80` with security headers
  - `/api` to `api-svc:5678` with rate limiting (5 rps, burst multiplier 3)
  - `/admin` to `admin-svc:5678` with basic auth

**Note:** Since annotations apply to the entire Ingress resource, you may need to create separate Ingress resources for different annotation sets, or use path-specific annotations if your Ingress controller supports them.

Answer these questions:
1. Can you apply different annotations to different paths within a single Ingress resource?
2. If not, what is the recommended approach?
3. How does Nginx Ingress handle conflicting annotations?

<details>
<summary>Hint -- Per-Path Annotations</summary>
Nginx Ingress annotations typically apply to the entire Ingress resource, not to individual paths. To apply different policies to different paths, create separate Ingress resources for each path (or group of paths with the same policy). Use the same host in multiple Ingress resources -- the Ingress controller merges them. This is the standard pattern for applying different middleware to different routes.
</details>

## Success Criteria

- [ ] Rate limiting on the API backend returns 429 after the burst is exhausted
- [ ] Basic auth on the admin panel returns 401 without credentials and 200 with valid credentials
- [ ] Security headers appear on all frontend responses
- [ ] You understand that annotations apply per-Ingress, not per-path
- [ ] You can explain why separate Ingress resources are needed for different annotation sets
- [ ] You understand the trade-offs of Ingress-layer policy enforcement

## What You Should Understand After This Exercise

After completing this exercise, you should understand that the Ingress layer is the right place to enforce cross-cutting HTTP policies. Rate limiting protects backends from abuse. Basic authentication restricts access to sensitive areas. Security headers protect clients from common attacks. These policies are enforced by the Ingress controller (Nginx) before traffic reaches your application Pods, which means you get consistent enforcement without modifying application code. The key limitation is that Nginx Ingress annotations apply to the entire Ingress resource, so different policies for different paths require separate Ingress resources.
