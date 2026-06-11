# Exercise 01: Service Mesh Concepts

**Type:** Conceptual
**Time:** 20 min
**Difficulty:** Easy

## Objective

Understand what a service mesh is, why it exists, and how it differs from traditional networking approaches in microservices architectures.

## Scenario

Your company has 30 microservices running in Kubernetes. The team is struggling with these problems:

```
Current issues:
├── No encryption between services (plaintext HTTP)
├── No visibility into which services call which
├── Deploying a new version requires manual load balancer changes
├── Debugging failures requires checking logs across 30 services
├── No rate limiting between services
└── Circuit breaking is implemented differently in each service (3 languages)
```

## Tasks

### Part A: Service Mesh Fundamentals

Answer these questions about service mesh architecture:

1. What is a service mesh, and what problem does it solve?
2. What is a "sidecar proxy" and how does it work?
3. What is the data plane vs the control plane?
4. How does a service mesh differ from an API gateway?

<details>
<summary>Hint</summary>

A service mesh adds networking capabilities (encryption, routing, observability) at the infrastructure level, without changing application code. Sidecar proxies are deployed alongside each service pod. The data plane handles traffic; the control plane configures the proxies.

</details>

### Part B: Sidecar Injection

Explain what happens when a service mesh is enabled for a namespace:

1. What changes in the pod specification?
2. How does traffic flow from Service A to Service B with sidecar proxies?
3. What happens to the original application container's network traffic?

Draw a diagram showing the traffic flow with and without a service mesh.

<details>
<summary>Hint</summary>

With a service mesh, every pod gets an additional container (the sidecar proxy). All inbound and outbound traffic passes through this proxy. The application container thinks it is talking directly to other services, but the proxy intercepts and manages all connections.

</details>

### Part C: Feature Mapping

Map each problem from the scenario to a service mesh feature:

| Problem | Service Mesh Feature | How It Helps |
|---------|---------------------|--------------|
| No encryption between services | | |
| No visibility into service calls | | |
| Manual load balancer changes | | |
| Debugging across 30 services | | |
| No rate limiting | | |
| Inconsistent circuit breaking | | |

<details>
<summary>Hint</summary>

Think about: mTLS for encryption, distributed tracing for visibility, traffic routing for deployments, metrics collection for debugging, rate limiting policies, and circuit breaker configuration at the mesh level.

</details>

## Success Criteria

- [ ] You can explain what a service mesh is and why it exists
- [ ] You understand the sidecar proxy pattern
- [ ] You can describe the data plane and control plane
- [ ] You can map service mesh features to real-world problems

## What You Should Understand After This Exercise

A service mesh is an infrastructure layer that handles service-to-service communication. It uses sidecar proxies to intercept all traffic, providing encryption (mTLS), traffic management (routing, splitting), observability (metrics, traces), and resilience (circuit breaking, retries) without changing application code. The key benefit is that networking concerns are decoupled from business logic.
