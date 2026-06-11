# Exercise 03: Service Discovery Using DNS

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective

Set up a multi-service application where services discover each other using Kubernetes DNS. You will create a frontend that communicates with a backend using DNS names, test cross-namespace service discovery, and verify that DNS resolution works correctly for both regular and headless Services.

## Background

In a real application, services need to find and communicate with each other without hardcoding IP addresses. Kubernetes provides automatic DNS entries for every Service, following the pattern `<service-name>.<namespace>.svc.cluster.local`. You need to understand how this works to build service architectures where components can discover each other dynamically.

## Tasks

### Part A: Create a Two-Tier Application

Create a backend Deployment and Service, then a frontend pod that talks to the backend using its DNS name.

**Step 1:** Create the backend.

```yaml
# Save as backend.yaml
```

Create a Deployment named `api-server` with 2 replicas using image `nginx:1.25` with label `app: api`. Then create a ClusterIP Service named `api-service` on port 80 targeting port 80.

**Step 2:** Create a frontend that uses DNS to reach the backend.

```yaml
# Save as frontend.yaml
```

Create a pod named `frontend` using image `curlimages/curl:8.4.0` that runs a loop:

```sh
while true; do curl -s http://api-service:80; echo "---"; sleep 5; done
```

**Step 3:** Verify the frontend can reach the backend:

```bash
kubectl logs frontend -f
```

Record the output. The frontend should be receiving nginx responses.

<details>
<summary>Hint -- DNS Name Within Namespace</summary>
When the frontend and backend are in the same namespace (default), you can use just the Service name: `http://api-service:80`. Kubernetes DNS resolves this to the ClusterIP. You do not need the full FQDN for same-namespace access.
</details>

### Part B: Test Cross-Namespace Discovery

Now create the same backend in a different namespace and test whether you can reach it using the full DNS name.

**Step 1:** Create a new namespace:

```bash
kubectl create namespace backend-ns
```

**Step 2:** Create the backend Deployment and Service in `backend-ns`:

```yaml
# Save as backend-ns.yaml
```

Create a Deployment named `internal-api` with 2 replicas using image `nginx:1.25` with label `app: internal-api` in namespace `backend-ns`. Create a ClusterIP Service named `internal-api` on port 80 in namespace `backend-ns`.

**Step 3:** From the `frontend` pod in the default namespace, try to reach the backend in `backend-ns`:

```bash
kubectl exec frontend -- curl -s http://internal-api.backend-ns.svc.cluster.local:80
```

Does it work? Now try the short name:

```bash
kubectl exec frontend -- curl -s http://internal-api:80
```

Does this work? Explain why or why not.

<details>
<summary>Hint -- FQDN for Cross-Namespace</summary>
The short name `internal-api` resolves only within the same namespace. To reach a Service in a different namespace, you must use the full FQDN: `internal-api.backend-ns.svc.cluster.local`. This is because DNS search domains are set to the pod's own namespace.
</details>

### Part C: Explore DNS Records

Run DNS lookups from within the cluster to understand the DNS records Kubernetes creates.

```bash
kubectl run dns-debug --image=busybox:1.36 --rm -it -- sh
```

Inside the pod, run the following commands and record the results:

```sh
# Resolve a ClusterIP Service
nslookup api-service

# Resolve using full FQDN
nslookup api-service.default.svc.cluster.local

# Resolve a Service in another namespace
nslookup internal-api.backend-ns.svc.cluster.local

# Look up SRV records
nslookup -type=SRV _http._tcp.api-service.default.svc.cluster.local

# Check what DNS server the pod uses
cat /etc/resolv.conf
```

Answer these questions:
1. What IP address does `api-service` resolve to?
2. What is the search domain listed in `/etc/resolv.conf`?
3. Why does the search domain explain the short-name behavior from Part B?

<details>
<summary>Hint -- resolv.conf</summary>
The `/etc/resolv.conf` file in a Kubernetes pod typically has a `search` line that includes `<namespace>.svc.cluster.local`, `svc.cluster.local`, and `cluster.local`. This is why short names like `api-service` work -- DNS appends the search domain automatically. Since the search domain is the pod's own namespace, short names only resolve Services in the same namespace.
</details>

### Part D: Headless Service DNS

Create a headless Service for the `api-server` Deployment and compare its DNS behavior to the regular ClusterIP Service.

**Step 1:** Create the headless Service:

```yaml
# Save as api-headless.yaml
```

Create a Service named `api-headless` with `clusterIP: None`, selector `app: api`, port 80.

**Step 2:** Run DNS lookups and compare:

```bash
kubectl run dns-test2 --image=busybox:1.36 --rm -it -- sh
```

```sh
# Regular ClusterIP Service
nslookup api-service

# Headless Service
nslookup api-headless
```

Record the difference in output. How many IP addresses does each return?

<details>
<summary>Hint -- Headless DNS</summary>
A regular ClusterIP Service returns a single virtual IP. A headless Service returns the individual IP addresses of all matching pods. With 2 replicas, you should see 2 IP addresses for the headless Service lookup.
</details>

## Success Criteria

- [ ] The frontend pod successfully reaches the backend using `http://api-service:80`
- [ ] Cross-namespace access works with the full FQDN but fails with the short name
- [ ] You can explain why short names do not work across namespaces based on `/etc/resolv.conf`
- [ ] The headless Service DNS returns multiple pod IPs while the ClusterIP Service returns one virtual IP
- [ ] You understand the DNS search domain mechanism

## What You Should Understand After This Exercise

After completing this exercise, you should understand how Kubernetes DNS enables automatic service discovery. Services get DNS entries that resolve to their ClusterIP. Within the same namespace, short names work because of DNS search domains. Across namespaces, you must use the full FQDN. Headless Services return individual pod IPs instead of a single virtual IP, which is essential for StatefulSets and client-side load balancing. This DNS-based discovery is the foundation of all microservice communication in Kubernetes.
