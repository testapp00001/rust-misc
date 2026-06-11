# Exercise 05: Expose Services with LoadBalancer and Ingress

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Deploy a multi-service application and expose it to external traffic using both LoadBalancer Services and an Ingress resource. You will combine concepts from this module (Services, DNS, endpoints) with concepts from previous modules (Deployments from Module 27, container configuration from earlier modules) to build a production-ready external access layer. This exercise bridges the gap between Module 28 (Services) and Module 29 (Ingress Controllers).

## Background

Your team has a microservices application with two HTTP services: a frontend web app and an API backend. Both need to be accessible from the internet. The simplest approach is to create a LoadBalancer Service for each, but this is expensive (one cloud load balancer per service) and does not support path-based routing. The better approach is to use a single LoadBalancer with an Ingress resource that routes traffic based on the URL path.

You need to implement both approaches, compare them, and understand when each is appropriate.

## Tasks

### Part A: Deploy the Application

Create two Deployments and their ClusterIP Services. These will be the backend services that receive traffic.

**Frontend:**
- Deployment name: `web-app`
- Replicas: 2
- Image: `nginx:1.25`
- Labels: `app: web-app, tier: frontend`
- The nginx should serve a custom page. Use a ConfigMap or initContainer to write `<h1>Frontend</h1>` to `/usr/share/nginx/html/index.html`.

**API:**
- Deployment name: `api-server`
- Replicas: 2
- Image: `nginx:1.25`
- Labels: `app: api-server, tier: backend`
- The nginx should serve a custom page. Write `<h1>API Server</h1>` to `/usr/share/nginx/html/index.html`.

Create ClusterIP Services for both:
- `web-app-service` on port 80
- `api-server-service` on port 80

Apply everything and verify:

```bash
kubectl get deployments
kubectl get services
kubectl get pods -l tier=frontend
kubectl get pods -l tier=backend
```

<details>
<summary>Hint -- Custom Nginx Content</summary>
The easiest way to serve custom content from nginx is to use a command override. For the frontend: `command: ["/bin/sh", "-c"]` with `args: ["echo '<h1>Frontend</h1>' > /usr/share/nginx/html/index.html && nginx -g 'daemon off;'"]`. Do the same for the API with `<h1>API Server</h1>`. This avoids needing a ConfigMap or volume mount.
</details>

### Part B: Expose with LoadBalancer Services

Create a LoadBalancer Service for each backend. This is the simplest but most expensive approach.

```yaml
# Save as loadbalancer-services.yaml
```

Create two LoadBalancer Services:
- `web-app-lb` targeting `app: web-app` on port 80
- `api-server-lb` targeting `app: api-server` on port 80

Apply and check:

```bash
kubectl apply -f loadbalancer-services.yaml
kubectl get services
```

If you are on a cloud provider, note the EXTERNAL-IPs assigned. If you are on minikube, run `minikube tunnel` in a separate terminal to provision external IPs.

Test access:

```bash
curl http://<web-app-external-ip>:80
curl http://<api-server-external-ip>:80
```

Answer these questions:
1. How many external IP addresses were provisioned?
2. What is the cost implication of this approach on a cloud provider?
3. Can you route `/api` to the API server and `/` to the frontend using just LoadBalancer Services?

<details>
<summary>Hint -- LoadBalancer Cost</summary>
On most cloud providers, each LoadBalancer Service provisions a separate cloud load balancer (e.g., AWS ELB, GCP Network LB). Each one has an hourly cost. With 10 microservices, you would pay for 10 load balancers. This is why Ingress exists -- one load balancer for many services.
</details>

### Part C: Expose with Ingress (Single Load Balancer)

Now replace the two LoadBalancer Services with a single Ingress resource that routes traffic based on URL path.

**Step 1:** Install an Ingress controller. If using minikube:

```bash
minikube addons enable ingress
```

For other clusters, install nginx-ingress:

```bash
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.9.0/deploy/static/provider/cloud/deploy.yaml
```

Wait for the ingress controller pod to be ready:

```bash
kubectl get pods -n ingress-nginx
```

**Step 2:** Delete the LoadBalancer Services from Part B:

```bash
kubectl delete -f loadbalancer-services.yaml
```

**Step 3:** Create an Ingress resource:

```yaml
# Save as app-ingress.yaml
```

Create an Ingress with:
- Name: `app-ingress`
- Rule: host `app.local` (or any hostname)
- Path `/` routes to `web-app-service:80`
- Path `/api` routes to `api-server-service:80`

Apply and verify:

```bash
kubectl apply -f app-ingress.yaml
kubectl get ingress
kubectl describe ingress app-ingress
```

Test access (you may need to add `app.local` to `/etc/hosts` pointing to the ingress controller's external IP):

```bash
curl http://app.local/
curl http://app.local/api
```

<details>
<summary>Hint -- Ingress Path Types</summary>
Use `pathType: Prefix` for path-based routing. This means `/api` will match `/api`, `/api/v1`, `/api/users`, etc. Use `pathType: Exact` if you want to match only the exact path. For most cases, Prefix is what you want.
</details>

### Part D: Compare and Reflect

Fill in the comparison table:

| Aspect | LoadBalancer per Service | Ingress |
|--------|------------------------|---------|
| Number of external IPs | | |
| Path-based routing | | |
| Host-based routing | | |
| TLS termination | | |
| Cost (cloud) | | |
| Complexity | | |
| Best for | | |

Answer these questions:

1. When would you still use a LoadBalancer Service instead of Ingress?
2. What happens to the Ingress if the ingress controller pod crashes?
3. Can an Ingress route to Services in different namespaces?
4. How would you add HTTPS/TLS to the Ingress configuration?

<details>
<summary>Hint -- TLS with Ingress</summary>
To add TLS, you create a Kubernetes Secret containing the TLS certificate and key, then reference it in the Ingress spec under `tls:`. The Ingress controller terminates TLS and forwards plain HTTP to the backend Services. Some Ingress controllers also support cert-manager for automatic certificate provisioning.
</details>

## Success Criteria

- [ ] Both Deployments are running with custom HTML content
- [ ] Both ClusterIP Services are created and have correct endpoints
- [ ] LoadBalancer Services provision external IPs and serve traffic (Part B)
- [ ] The Ingress resource routes `/` to the frontend and `/api` to the API (Part C)
- [ ] You can explain the cost and functional differences between the two approaches
- [ ] You understand when to use LoadBalancer vs Ingress in production

## What You Should Understand After This Exercise

After completing this exercise, you should understand the evolution from simple LoadBalancer Services to Ingress-based routing. LoadBalancer Services are straightforward but expensive -- one external IP per service. Ingress consolidates multiple services behind a single load balancer with HTTP-aware routing (path-based, host-based, TLS). The Ingress controller itself is deployed as a LoadBalancer Service, so you still need one external IP, but it serves all your applications. This is the standard production pattern for exposing HTTP services in Kubernetes, and it sets the stage for Module 29 which dives deep into Ingress controllers.
