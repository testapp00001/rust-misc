# Exercise 05: Ingress Strategy for Multi-Team Clusters

**Type:** Integration
**Time:** 50 minutes
**Difficulty:** Hard

## Objective

Design an Ingress strategy for a shared Kubernetes cluster where multiple teams deploy their own applications. You will implement namespace isolation, team-specific Ingress resources, shared TLS configuration, and a default backend for unmatched traffic. This exercise combines Ingress concepts with namespace isolation and RBAC from earlier modules.

## Background

Your organization runs a single Kubernetes cluster shared by three teams: the payments team, the catalog team, and the platform team. Each team owns their own namespace and deploys their own services. All teams need their applications accessible via the internet, but:

- Each team should only be able to create Ingress resources in their own namespace
- TLS certificates should be managed centrally
- Unmatched traffic (unknown hostnames/paths) should return a branded 404 page
- Each team's Ingress should follow organizational standards (security headers, rate limits)

You need to design and implement this Ingress architecture.

## Tasks

### Part A: Create Team Namespaces and Deploy Applications

**Step 1:** Create namespaces for each team:

```bash
kubectl create namespace payments
kubectl create namespace catalog
kubectl create namespace platform
```

**Step 2:** Deploy the payments service in the `payments` namespace:

```yaml
# Save as payments-app.yaml
```

Create a Deployment named `payments-api` in namespace `payments` with 2 replicas using `hashicorp/http-echo:0.2.3` with args `["-text=Payments Service v2.1", "-listen=:5678"]` and label `app: payments-api`. Create a ClusterIP Service named `payments-svc` on port 5678 in namespace `payments`.

**Step 3:** Deploy the catalog service in the `catalog` namespace:

```yaml
# Save as catalog-app.yaml
```

Create a Deployment named `catalog-api` in namespace `catalog` with 2 replicas using `hashicorp/http-echo:0.2.3` with args `["-text=Catalog Service v3.4", "-listen=:5678"]` and label `app: catalog-api`. Create a ClusterIP Service named `catalog-svc` on port 5678 in namespace `catalog`.

**Step 4:** Deploy the platform service (the default landing page) in the `platform` namespace:

```yaml
# Save as platform-app.yaml
```

Create a Deployment named `landing-page` in namespace `platform` with 2 replicas using `nginx:1.25` with label `app: landing-page`. Serve custom content:

```yaml
command: ["/bin/sh", "-c"]
args: ["echo '<h1>Welcome to Our Platform</h1><p>Service not found.</p>' > /usr/share/nginx/html/index.html && nginx -g 'daemon off;'"]
```

Create a ClusterIP Service named `landing-svc` on port 80 in namespace `platform`.

**Step 5:** Verify all deployments:

```bash
kubectl get deployments --all-namespaces | grep -E "payments|catalog|platform"
kubectl get services --all-namespaces | grep -E "payments|catalog|platform"
```

<details>
<summary>Hint -- Cross-Namespace Ingress</summary>
An Ingress resource in one namespace can reference a Service in the same namespace only. It cannot reference a Service in a different namespace. This means each team's Ingress must be in the same namespace as their Service. The Ingress controller (installed in `ingress-nginx` namespace) watches all namespaces for Ingress resources by default.
</details>

### Part B: Create Team Ingress Resources

Create an Ingress resource in each team's namespace.

**Step 1:** Payments team Ingress:

```yaml
# Save as payments-ingress.yaml
```

Create an Ingress in namespace `payments` with:
- Name: `payments-ingress`
- `ingressClassName: nginx`
- Host: `payments.example.com`
- Path `/` to `payments-svc:5678`

**Step 2:** Catalog team Ingress:

```yaml
# Save as catalog-ingress.yaml
```

Create an Ingress in namespace `catalog` with:
- Name: `catalog-ingress`
- `ingressClassName: nginx`
- Host: `catalog.example.com`
- Path `/` to `catalog-svc:5678`

**Step 3:** Apply all Ingress resources:

```bash
kubectl apply -f payments-ingress.yaml
kubectl apply -f catalog-ingress.yaml
```

**Step 4:** Verify:

```bash
kubectl get ingress --all-namespaces
kubectl describe ingress payments-ingress -n payments
kubectl describe ingress catalog-ingress -n catalog
```

Answer these questions:
1. Can the payments team's Ingress reference a Service in the `catalog` namespace?
2. How does the Ingress controller know which Ingress resource handles a given request?
3. What happens when a request arrives for `unknown.example.com`?

<details>
<summary>Hint -- Host Matching</summary>
The Ingress controller matches incoming requests to Ingress resources based on the `Host` header and the URL path. When a request arrives for `payments.example.com`, the controller looks for an Ingress with a rule for that hostname. If no Ingress matches, the request is sent to the default backend. Each team's Ingress only handles traffic for their assigned hostname.
</details>

### Part C: Implement a Default Backend

Create a default backend that returns a branded 404 page for unmatched traffic.

**Step 1:** Create a custom 404 backend:

```yaml
# Save as default-backend.yaml
```

Create a Deployment named `default-backend` in namespace `platform` with 1 replica using `nginx:1.25` with label `app: default-backend`. Serve custom 404 content:

```yaml
command: ["/bin/sh", "-c"]
args:
  - |
    echo '<h1>404 - Service Not Found</h1><p>The requested service does not exist.</p>' > /usr/share/nginx/html/index.html
    echo 'server { listen 80; location / { return 404; } }' > /etc/nginx/conf.d/default.conf
    nginx -g 'daemon off;'
```

Create a ClusterIP Service named `default-backend-svc` on port 80 in namespace `platform`.

**Step 2:** Create an Ingress that acts as the default backend:

```yaml
# Save as default-ingress.yaml
```

Create an Ingress in namespace `platform` with:
- Name: `default-ingress`
- `ingressClassName: nginx`
- `defaultBackend` pointing to `default-backend-svc:80`

Apply and test:

```bash
kubectl apply -f default-backend.yaml
kubectl apply -f default-ingress.yaml

# Test with a known host (should route to the correct service)
curl -H "Host: payments.example.com" http://<ingress-ip>/

# Test with an unknown host (should get the 404 page)
curl -H "Host: unknown.example.com" http://<ingress-ip>/
```

<details>
<summary>Hint -- defaultBackend vs rules</summary>
The `defaultBackend` field in an Ingress resource defines where traffic goes when no other rule matches. This is useful for catching typos in hostnames or paths. Without a default backend, unmatched traffic returns the Ingress controller's generic "default backend - 404" page. A custom default backend lets you return a branded error page.
</details>

### Part D: Apply Organizational Standards

Add security headers and rate limiting to all team Ingress resources.

**Step 1:** Update the payments Ingress to include security headers and rate limiting:

```yaml
# Save as payments-ingress-secure.yaml
```

Update the payments Ingress with annotations:
- `nginx.ingress.kubernetes.io/limit-rps: "50"`
- `nginx.ingress.kubernetes.io/limit-burst-multiplier: "2"`
- `nginx.ingress.kubernetes.io/configuration-snippet` with security headers:
  - `X-Frame-Options: DENY`
  - `X-Content-Type-Options: nosniff`

**Step 2:** Do the same for the catalog Ingress with appropriate rate limits.

**Step 3:** Apply and verify:

```bash
kubectl apply -f payments-ingress-secure.yaml
kubectl apply -f catalog-ingress-secure.yaml

# Verify headers
curl -I -H "Host: payments.example.com" http://<ingress-ip>/
curl -I -H "Host: catalog.example.com" http://<ingress-ip>/
```

Answer these questions:
1. If you have 10 teams, do you need to manually add these annotations to every Ingress?
2. How could you enforce these standards automatically? (Think about admission controllers, OPA/Gatekeeper, or Kyverno.)
3. What happens if a team creates an Ingress without the required annotations?

<details>
<summary>Hint -- Policy Enforcement</summary>
In a real multi-team cluster, you would use a policy engine like Kyverno or OPA/Gatekeeper to enforce Ingress standards. These tools can validate Ingress resources at admission time and reject those that do not meet organizational requirements. This ensures consistency without relying on teams to remember to add annotations. Alternatively, you could use an IngressClass with a custom controller that injects default annotations.
</details>

### Part E: TLS Strategy for Multi-Team Clusters

Design a TLS strategy for the multi-team cluster.

**Step 1:** Generate a wildcard certificate (for demonstration):

```bash
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout wildcard.key -out wildcard.crt \
  -subj "/CN=*.example.com" \
  -addext "subjectAltName=DNS:*.example.com"
```

**Step 2:** Create the TLS Secret in each namespace (Ingress can only reference Secrets in its own namespace):

```bash
kubectl create secret tls wildcard-tls --cert=wildcard.crt --key=wildcard.key -n payments
kubectl create secret tls wildcard-tls --cert=wildcard.crt --key=wildcard.key -n catalog
kubectl create secret tls wildcard-tls --cert=wildcard.crt --key=wildcard.key -n platform
```

**Step 3:** Update each Ingress to include TLS:

Add a `tls` section to each Ingress:
- Payments: host `payments.example.com`, secretName `wildcard-tls`
- Catalog: host `catalog.example.com`, secretName `wildcard-tls`

Apply and test:

```bash
kubectl apply -f payments-ingress-secure.yaml
kubectl apply -f catalog-ingress-secure.yaml

curl -k -H "Host: payments.example.com" https://<ingress-ip>/
curl -k -H "Host: catalog.example.com" https://<ingress-ip>/
```

**Step 4:** Answer these questions:
1. Why must the TLS Secret be in the same namespace as the Ingress?
2. What are the trade-offs of a wildcard certificate vs per-service certificates?
3. How would you automate certificate rotation across all namespaces?
4. What happens if a team creates an Ingress with TLS but the Secret does not exist?

<details>
<summary>Hint -- Certificate Distribution</summary>
Kubernetes does not support cross-namespace Secret references in Ingress. The TLS Secret must be in the same namespace as the Ingress resource. For multi-namespace TLS, you either copy the Secret to each namespace (as shown above) or use cert-manager with per-namespace ClusterIssuers. cert-manager can automatically create Secrets in each namespace where an Ingress has the `cert-manager.io/cluster-issuer` annotation. For automation, use a controller like "kubernetes-replicator" or write a CronJob that syncs Secrets across namespaces.
</details>

## Success Criteria

- [ ] Three namespaces are created with their respective applications running
- [ ] Each team has an Ingress resource in their own namespace routing to their Service
- [ ] A default backend returns a branded 404 for unmatched hosts
- [ ] Security headers and rate limiting are applied to all team Ingress resources
- [ ] TLS is configured for all Ingress resources
- [ ] You can explain why Ingress resources must be in the same namespace as their Services
- [ ] You understand the trade-offs of different TLS strategies in multi-team clusters
- [ ] You can describe how policy enforcement would work in production

## What You Should Understand After This Exercise

After completing this exercise, you should understand how to design an Ingress architecture for a shared Kubernetes cluster. Each team manages their own Ingress resources in their own namespace, ensuring isolation. The Ingress controller is a shared component that watches all namespaces. TLS certificates must be distributed to each namespace (or automated with cert-manager). A default backend catches unmatched traffic. Organizational standards (security headers, rate limits) should be enforced through policy engines rather than manual processes. This architecture scales to many teams while maintaining security and consistency.
