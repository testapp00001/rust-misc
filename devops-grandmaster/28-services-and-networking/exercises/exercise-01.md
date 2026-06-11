# Exercise 01: ClusterIP vs NodePort vs LoadBalancer

**Type:** Conceptual
**Time:** 20 minutes
**Difficulty:** Easy

## Objective

Demonstrate your understanding of the three primary Kubernetes Service types by comparing their behavior, use cases, networking characteristics, and trade-offs. This exercise verifies that you understand *why* you would choose one Service type over another, not just *what* the YAML looks like.

## Background

Kubernetes Services provide stable networking for ephemeral Pods. The three main Service types -- ClusterIP, NodePort, and LoadBalancer -- each solve a different networking problem. Choosing the wrong one leads to either unreachable services or unnecessary complexity. Understanding the differences is essential for designing network architectures that are correct, secure, and cost-effective.

## Tasks

### Part A: Comparison Table

Fill in the table below. For each Service type, describe the networking behavior, list the access scope, and identify at least one real-world use case.

| Characteristic | ClusterIP | NodePort | LoadBalancer |
|----------------|-----------|----------|--------------|
| Default type? | | | |
| External access? | | | |
| Internal access? | | | |
| Port range | | | |
| Cloud provider dependency? | | | |
| Real-world use case | | | |

<details>
<summary>Hint -- Access Scope</summary>
ClusterIP is only reachable from within the cluster. NodePort opens a port on every node in the cluster, making it reachable from outside if you can reach the node IPs. LoadBalancer provisions an external IP through the cloud provider's load balancing infrastructure.
</details>

### Part B: Scenario-Based Selection

For each scenario below, choose the most appropriate Service type and explain why the other two are not suitable.

**Scenario 1:** An internal API server that only other microservices within the cluster should access. No external traffic should ever reach it.

**Scenario 2:** A development team needs to test a web application from their laptops. The cluster runs on bare-metal servers in an office with no cloud provider.

**Scenario 3:** A production web application that must be accessible from the internet with a stable external IP address, running on AWS EKS.

**Scenario 4:** A database that is accessed only by a backend application running in the same namespace.

<details>
<summary>Hint -- NodePort Limitations</summary>
NodePort uses ports in the range 30000-32767. This means you cannot use standard port 80 or 443, and the URLs users type will always include a port number. It also exposes every node's IP, which may not be desirable for production.
</details>

### Part C: Layer Model

Explain which OSI network layer each Service type operates at and what that means for the type of traffic routing it can perform. Specifically answer:

1. Can a ClusterIP Service route traffic based on HTTP path (e.g., `/api` vs `/web`)?
2. Can a LoadBalancer Service perform TLS termination?
3. What does it mean that all three Service types operate at Layer 4 (TCP/UDP)?

<details>
<summary>Hint -- Layer 4 vs Layer 7</summary>
Layer 4 routing works with TCP/UDP connections -- it forwards packets without understanding the application protocol. Layer 7 routing understands HTTP headers, paths, and hostnames. Kubernetes Services are Layer 4. For Layer 7 routing, you need an Ingress controller or service mesh (covered in Module 29).
</details>

## Success Criteria

- [ ] Your comparison table is complete and accurate for all three Service types
- [ ] Each scenario answer includes the correct Service type with a clear justification
- [ ] You can explain why the rejected Service types are wrong for each scenario
- [ ] You understand the Layer 4 limitation and can explain what it means for routing decisions

## What You Should Understand After This Exercise

After completing this exercise, you should be able to look at any networking requirement and immediately know which Service type is appropriate. You should understand that ClusterIP is the default and safest choice for internal communication, NodePort is useful for development and bare-metal access, and LoadBalancer is the production standard for cloud deployments. The Layer 4 limitation is the key insight -- it explains why Ingress controllers exist and why Services alone are not enough for HTTP-aware routing.
