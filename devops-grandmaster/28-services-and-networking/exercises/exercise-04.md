# Exercise 04: Service Topology for Microservices

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Design and implement a complete service topology for a microservices application with three tiers: a public-facing frontend, an internal API gateway, and two backend services. You must choose the correct Service type for each tier, configure proper selectors and ports, verify inter-service communication, and implement a headless service for the stateful component. This exercise simulates a real production architecture decision.

## Background

Your team is building an e-commerce platform with the following architecture:

```
Internet --> Frontend --> API Gateway --> Product Service
                                      --> Order Service
```

- The **Frontend** serves a web UI and must be accessible from outside the cluster.
- The **API Gateway** receives requests from the frontend and routes them to backend services. It should only be reachable from within the cluster.
- The **Product Service** is a stateless read-only API. It should only be reachable from the API Gateway.
- The **Order Service** is stateful and maintains connections. Each instance must be individually addressable for session affinity. It should only be reachable from the API Gateway.

You need to deploy all four components and wire them together with the correct Services.

## Tasks

### Part A: Design the Service Topology

Before writing any YAML, fill in the design table. For each component, specify the Deployment details and the Service configuration.

| Component | Image | Replicas | Labels | Service Type | Service Name | Port | Target Port | Special Config |
|-----------|-------|----------|--------|--------------|--------------|------|-------------|----------------|
| Frontend | nginx:1.25 | 2 | | | | | | |
| API Gateway | nginx:1.25 | 2 | | | | | | |
| Product Service | nginx:1.25 | 3 | | | | | | |
| Order Service | nginx:1.25 | 2 | | | | | | |

For the "Special Config" column, note any non-standard settings like headless services or named ports.

<details>
<summary>Hint -- Service Type Selection</summary>
The Frontend needs external access, so NodePort (or LoadBalancer on cloud). The API Gateway, Product Service, and Order Service are internal only, so ClusterIP. The Order Service needs individual pod addressing, which means a headless Service (clusterIP: None).
</details>

### Part B: Deploy the Topology

Create the following YAML files and apply them:

**1. Frontend Deployment and Service** (`frontend.yaml`)

Deploy the frontend with 2 replicas. Create a NodePort Service so it is accessible externally. The frontend needs to reach the API Gateway at `http://api-gateway:8080`.

**2. API Gateway Deployment and Service** (`api-gateway.yaml`)

Deploy the API Gateway with 2 replicas. Create a ClusterIP Service. The gateway needs to reach:
- `http://product-service:8080`
- `http://order-service:8080`

**3. Product Service Deployment and Service** (`product-service.yaml`)

Deploy the Product Service with 3 replicas. Create a ClusterIP Service on port 8080.

**4. Order Service Deployment and Service** (`order-service.yaml`)

Deploy the Order Service with 2 replicas. Create a headless Service (clusterIP: None) on port 8080.

Apply all files and verify:

```bash
kubectl apply -f frontend.yaml
kubectl apply -f api-gateway.yaml
kubectl apply -f product-service.yaml
kubectl apply -f order-service.yaml

# Verify all deployments
kubectl get deployments

# Verify all services
kubectl get services

# Verify all endpoints
kubectl get endpoints
```

<details>
<summary>Hint -- Label Naming Convention</summary>
Use consistent labels: `app: frontend`, `app: api-gateway`, `app: product-service`, `app: order-service`. Each Deployment's `spec.selector.matchLabels` must match its pod template labels exactly. Each Service's `spec.selector` must match the pods it routes to.
</details>

### Part C: Verify the Communication Chain

Test that the full communication chain works. Start from a temporary pod and walk through each hop.

**Step 1:** Verify the frontend is accessible externally:

```bash
# Get node IP and nodePort
kubectl get service frontend-service
curl http://<node-ip>:<nodePort>
```

**Step 2:** Verify the API Gateway is reachable from within the cluster:

```bash
kubectl run verify --image=curlimages/curl:8.4.0 --rm -it -- sh
curl http://api-gateway:8080
curl http://product-service:8080
curl http://order-service:8080
exit
```

**Step 3:** Verify the headless Service returns individual pod IPs:

```bash
kubectl run dns-check --image=busybox:1.36 --rm -it -- sh
nslookup order-service
exit
```

Record the output. How many IPs does `order-service` resolve to?

<details>
<summary>Hint -- Headless DNS Output</summary>
A headless Service with 2 replicas should return 2 IP addresses. The output will show "Name: order-service" followed by "Address: <ip1>" and "Address: <ip2>". This is different from a ClusterIP Service which returns a single virtual IP.
</details>

### Part D: Analyze and Answer

Answer these architectural questions:

1. **Why is the Frontend a NodePort but the API Gateway is a ClusterIP?** What security concern does this address?

2. **Why is the Order Service headless?** What capability does this provide that a regular ClusterIP Service does not?

3. **What would happen if you accidentally used the same label (`app: backend`) for both the Product Service and Order Service pods, but created separate Services with different names?** Would routing work correctly? Why or why not?

4. **If the Product Service Deployment scales from 3 to 5 replicas, what changes automatically?** Name the Kubernetes objects that update without manual intervention.

<details>
<summary>Hint -- Selector Overlap</summary>
If two different Services use the same selector, both Services will route to the same set of pods. If two Deployments share the same label, their pod selectors will conflict and each Deployment will try to manage the other's pods. Labels must be unique enough to distinguish workloads.
</details>

## Success Criteria

- [ ] All four Deployments are running with the correct number of replicas
- [ ] The Frontend Service is a NodePort accessible from outside the cluster
- [ ] The API Gateway Service is a ClusterIP accessible only from within the cluster
- [ ] The Product Service is a ClusterIP accessible only from within the cluster
- [ ] The Order Service is headless and resolves to individual pod IPs
- [ ] The full communication chain works: Frontend -> API Gateway -> Backend Services
- [ ] You can explain the reasoning behind each Service type choice

## What You Should Understand After This Exercise

After completing this exercise, you should be able to design a multi-tier service topology where each component has the correct exposure level. The key principles are: minimize attack surface by using ClusterIP by default, only use NodePort or LoadBalancer for components that must be externally accessible, use headless Services when clients need to address individual pods, and use consistent labeling to ensure selectors match the right pods. This is the foundation of production microservice networking in Kubernetes.
