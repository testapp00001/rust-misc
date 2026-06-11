# Exercise 05: Production Service Mesh Deployment

**Type:** Integration
**Time:** 60 min
**Difficulty:** Hard

## Objective

Design and deploy a production-grade service mesh for a multi-team microservices platform, covering installation, security, traffic management, observability, and multi-tenancy.

## Scenario

You are the platform engineer responsible for deploying a service mesh across a Kubernetes cluster:

```
Cluster:
  - 50 microservices across 5 teams
  - 3 namespaces per team (dev, staging, prod)
  - 500 pods total in production
  - Mix of gRPC and HTTP/REST services
  - Some services run outside the mesh (legacy VMs)

Requirements:
  - mTLS everywhere (zero trust networking)
  - Canary deployments for all teams
  - Distributed tracing (Jaeger)
  - Metrics (Prometheus + Grafana)
  - Multi-tenant isolation (team A cannot see team B's traffic)
  - Legacy VM integration (services not in Kubernetes)
```

## Tasks

### Part A: Service Mesh Selection

Compare Istio and Linkerd for this scenario. Create a decision matrix:

| Criterion | Istio | Linkerd | Winner |
|-----------|-------|---------|--------|
| Feature richness | | | |
| Resource overhead | | | |
| Ease of operation | | | |
| Multi-cluster support | | | |
| VM integration | | | |
| Community/ecosystem | | | |

Which would you choose and why?

<details>
<summary>Hint</summary>

Istio is more feature-rich but heavier. Linkerd is lighter and simpler but has fewer features. For 50 services with VM integration needs, Istio's features (especially for VM workloads) may be worth the complexity.

</details>

### Part B: Installation and Configuration

Design the installation plan:

1. What namespace should the control plane run in?
2. What resources (CPU, memory) does the control plane need?
3. How do you enable sidecar injection for specific namespaces?
4. How do you handle the mesh expansion to VMs?

<details>
<summary>Hint</summary>

Istio control plane (istiod) runs in `istio-system` namespace. Sidecar injection is enabled per namespace via label `istio-injection=enabled`. VM integration uses Istio's WorkloadEntry resource. Resource requirements depend on mesh size.

</details>

### Part C: Multi-Tenant Isolation

Design network policies and authorization rules to isolate teams:

```
Teams:
  - Team A: owns services in namespace team-a-prod
  - Team B: owns services in namespace team-b-prod
  - Platform: owns shared services (auth, logging)

Requirements:
  - Team A services cannot call Team B services
  - Both teams can call shared platform services
  - Platform services can call any service (for health checks)
  - No cross-team traffic in production
```

<details>
<summary>Hint</summary>

Use Istio AuthorizationPolicy with namespace-level restrictions. Deny all by default, then allow specific service-to-service communication. Use `PeerAuthentication` for mTLS enforcement. Consider using `ServiceEntry` for external service access.

</details>

### Part D: Migration Strategy

Design a migration plan to onboard the 50 services to the mesh:

1. How do you handle the rollout order?
2. What happens to services that cannot use sidecars?
3. How do you verify the mesh is working correctly?
4. How do you rollback if something breaks?

<details>
<summary>Hint</summary>

Migrate incrementally: start with non-critical services, verify mTLS and observability work, then migrate critical services. For services that cannot use sidecars, use the mesh's VM integration or egress gateway. Verify with mTLS metrics (percentage of mTLS traffic) and trace completeness.

</details>

## Success Criteria

- [ ] You can compare Istio and Linkerd for a given scenario
- [ ] You can design a service mesh installation plan
- [ ] You can configure multi-tenant isolation with authorization policies
- [ ] You can create a migration strategy for onboarding services to the mesh

## What You Should Understand After This Exercise

Deploying a service mesh in production requires careful planning: selecting the right mesh for your needs, configuring security (mTLS, authorization), setting up observability (metrics, traces), and designing a migration strategy that does not break existing services. Multi-tenant isolation requires namespace-level policies. The key trade-off is between the operational complexity of the mesh and the networking capabilities it provides.
