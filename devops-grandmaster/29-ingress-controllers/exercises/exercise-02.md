# Exercise 02: Nginx Ingress with Path-Based Routing

**Type:** Guided
**Time:** 35 minutes
**Difficulty:** Easy-Medium

## Objective

Install the Nginx Ingress controller, deploy multiple backend services, and configure an Ingress resource that routes traffic based on URL path. You will verify that a single external IP serves traffic to different backends depending on the path requested.

## Background

Your team has three services: a frontend web app, an API backend, and a documentation site. Each is a separate Deployment. Instead of creating three LoadBalancer Services (three external IPs, three cloud load balancers), you will use one Ingress controller with path-based routing to direct traffic to the correct backend.

## Tasks

### Part A: Install the Nginx Ingress Controller

Install the Nginx Ingress controller on your cluster.

**For minikube:**

```bash
minikube addons enable ingress
```

**For other clusters (using kubectl):**

```bash
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.9.0/deploy/static/provider/cloud/deploy.yaml
```

Wait for the controller to be ready:

```bash
kubectl get pods -n ingress-nginx -w
```

Verify the controller is running. You should see a Pod in `Running` state.

<details>
<summary>Hint -- Checking Controller Health</summary>
The Ingress controller Pod should show `READY 1/1` and `STATUS Running`. If it is stuck in `ContainerCreating`, check events with `kubectl describe pod -n ingress-nginx`. On minikube, you may need to run `minikube tunnel` in a separate terminal to provision an external IP for the controller's LoadBalancer Service.
</details>

### Part B: Deploy Backend Services

Create three Deployments and their ClusterIP Services.

**Step 1:** Create the frontend.

```yaml
# Save as frontend.yaml
```

Create a Deployment named `frontend` with 2 replicas using image `nginx:1.25` with label `app: frontend`. Serve custom content by overriding the command:

```yaml
command: ["/bin/sh", "-c"]
args: ["echo '<h1>Welcome to the Frontend</h1>' > /usr/share/nginx/html/index.html && nginx -g 'daemon off;'"]
```

Create a ClusterIP Service named `frontend-svc` on port 80.

**Step 2:** Create the API backend.

```yaml
# Save as api.yaml
```

Create a Deployment named `api` with 2 replicas using image `hashicorp/http-echo:0.2.3` with args `["-text=API Response", "-listen=:5678"]` and label `app: api`. Create a ClusterIP Service named `api-svc` on port 5678.

**Step 3:** Create the docs service.

```yaml
# Save as docs.yaml
```

Create a Deployment named `docs` with 1 replica using image `nginx:1.25` with label `app: docs`. Serve custom content:

```yaml
command: ["/bin/sh", "-c"]
args: ["echo '<h1>API Documentation</h1>' > /usr/share/nginx/html/index.html && nginx -g 'daemon off;'"]
```

Create a ClusterIP Service named `docs-svc` on port 80.

**Step 4:** Verify all deployments and services are running:

```bash
kubectl get deployments
kubectl get services
kubectl get pods
```

<details>
<summary>Hint -- ClusterIP is Enough</summary>
The backend Services should be type ClusterIP (the default). They do not need NodePort or LoadBalancer. The Ingress controller will route traffic to these Services from inside the cluster.
</details>

### Part C: Create the Ingress Resource

Create an Ingress resource that routes traffic based on URL path.

```yaml
# Save as path-ingress.yaml
```

Create an Ingress with:
- Name: `path-routing`
- `ingressClassName: nginx`
- Host: `myapp.local`
- Path `/` routes to `frontend-svc:80`
- Path `/api` routes to `api-svc:5678`
- Path `/docs` routes to `docs-svc:80`
- Use `pathType: Prefix` for all paths

Apply and verify:

```bash
kubectl apply -f path-ingress.yaml
kubectl get ingress
kubectl describe ingress path-routing
```

<details>
<summary>Hint -- Ingress YAML Structure</summary>
The `rules` field contains a list of host rules. Each host rule contains `http.paths`, which is a list of path rules. Each path rule has a `path`, `pathType`, and a `backend` that references a Service name and port. The `ingressClassName` field tells Kubernetes which Ingress controller should handle this resource.
</details>

### Part D: Test Path-Based Routing

Add `myapp.local` to your `/etc/hosts` file pointing to the Ingress controller's external IP.

```bash
# Get the ingress controller's external IP
kubectl get service -n ingress-nginx
```

For minikube:
```bash
echo "$(minikube ip) myapp.local" | sudo tee -a /etc/hosts
```

For cloud clusters, use the EXTERNAL-IP from the ingress-nginx service.

Test each path:

```bash
curl http://myapp.local/
curl http://myapp.local/api
curl http://myapp.local/docs
```

Each request should return different content. Record the output for each.

<details>
<summary>Hint -- 404 or Default Backend?</summary>
If you get a 404 or "default backend" response, check that the path and pathType are correct. With `pathType: Prefix`, the path `/api` matches `/api`, `/api/v1`, `/api/users`, etc. Make sure the Service names in the Ingress match the actual Service names you created. Use `kubectl describe ingress` to see the backend mappings.
</details>

### Part E: Inspect the Generated Configuration

Look at the Nginx configuration the Ingress controller generated:

```bash
# Get the ingress controller pod name
POD=$(kubectl get pods -n ingress-nginx -l app.kubernetes.io/name=ingress-nginx -o jsonpath='{.items[0].metadata.name}')

# View the generated nginx.conf
kubectl exec -n ingress-nginx $POD -- cat /etc/nginx/nginx.conf | grep -A 20 "location /api"
```

Answer these questions:
1. How does the Ingress controller map the `/api` path to the `api-svc` Service?
2. What happens when you request `/api/v1/users`? Does it reach the API service?
3. What happens when you request `/unknown`? Which backend handles it?

<details>
<summary>Hint -- Prefix Matching</summary>
With `pathType: Prefix`, `/api` matches any path that starts with `/api`. So `/api/v1/users` also matches and is routed to the API service. The path `/unknown` does not match `/api` or `/docs`, so it matches the root path `/` and goes to the frontend.
</details>

## Success Criteria

- [ ] The Nginx Ingress controller is installed and running
- [ ] All three backend Deployments and Services are created and healthy
- [ ] The Ingress resource routes `/` to the frontend, `/api` to the API, and `/docs` to the docs
- [ ] `curl` to each path returns the expected content from the correct backend
- [ ] You understand how `pathType: Prefix` works for matching
- [ ] You can inspect the generated Nginx configuration

## What You Should Understand After This Exercise

After completing this exercise, you should understand how an Ingress controller translates Ingress resources into reverse proxy configuration. The Ingress controller is itself a Deployment exposed via a LoadBalancer Service. It watches for Ingress objects, reads the routing rules, and configures Nginx accordingly. Path-based routing lets you serve multiple applications from a single external IP, with the Ingress controller directing each request to the correct backend Service based on the URL path.
