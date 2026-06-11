# Exercise 01: Ingress vs Service LoadBalancer

**Type:** Conceptual
**Time:** 20 minutes
**Difficulty:** Easy

## Objective

Demonstrate your understanding of the difference between exposing services through LoadBalancer-type Services and through Ingress resources. This exercise verifies that you understand *why* Ingress exists, what problem it solves, and when each approach is appropriate.

## Background

In Module 28, you learned that a Service of type LoadBalancer provisions an external IP address and forwards traffic to backend Pods. This works well for a single service, but production clusters often run dozens of microservices that all need external access. Creating a LoadBalancer Service for each one is expensive and does not support HTTP-aware routing.

Ingress is Kubernetes' answer to this problem: a single load balancer entry point that routes traffic to multiple services based on hostnames, URL paths, and TLS configuration.

## Tasks

### Part A: Comparison Table

Fill in the table below. For each characteristic, describe how LoadBalancer Services and Ingress resources behave.

| Characteristic | LoadBalancer Service | Ingress |
|----------------|---------------------|---------|
| OSI layer | | |
| External IPs required for 5 services | | |
| Path-based routing (`/api` vs `/web`) | | |
| Host-based routing (`api.example.com` vs `web.example.com`) | | |
| TLS termination | | |
| Cloud provider dependency | | |
| Rate limiting | | |
| Real-world use case | | |

<details>
<summary>Hint -- Layer 4 vs Layer 7</summary>
LoadBalancer Services operate at Layer 4 (TCP/UDP). They forward packets without understanding HTTP. Ingress operates at Layer 7 (HTTP). It can inspect headers, paths, and hostnames to make routing decisions.
</details>

### Part B: Scenario-Based Selection

For each scenario below, choose the most appropriate approach and explain why the alternative is not suitable.

**Scenario 1:** A single gRPC service that needs external access. gRPC uses HTTP/2 but does not need path-based routing.

**Scenario 2:** A web application with a frontend at `/`, an API at `/api`, and an admin panel at `/admin`. All three are separate Deployments.

**Scenario 3:** A TCP-based database proxy that needs external access on port 5432.

**Scenario 4:** Ten microservices, each needing a unique hostname (e.g., `auth.example.com`, `payments.example.com`, `catalog.example.com`). All use HTTPS.

<details>
<summary>Hint -- Protocol Matters</summary>
Ingress is designed for HTTP/HTTPS traffic. For non-HTTP protocols (TCP, UDP, databases), a LoadBalancer Service or a service mesh is more appropriate. Some Ingress controllers support TCP/UDP proxying via custom configuration, but it is not the native Ingress API behavior.
</details>

### Part C: Architecture Diagram

Draw (on paper or in a text editor) the traffic flow for each approach. Your diagrams should show:

1. **LoadBalancer approach:** Internet -> 3 LoadBalancer Services -> 3 backend Services -> Pods
2. **Ingress approach:** Internet -> 1 LoadBalancer -> Ingress Controller -> 3 backend Services -> Pods

Answer these questions:
1. In the Ingress approach, how many external IPs are provisioned?
2. Where does TLS termination happen in each approach?
3. What happens if the Ingress controller Pod crashes?

<details>
<summary>Hint -- Single Entry Point</summary>
In the Ingress approach, the Ingress controller itself is exposed via a single LoadBalancer Service. All traffic enters through that one external IP. The Ingress controller (Nginx, Traefik, etc.) reads the Ingress resources and configures its internal routing. If the controller crashes, all Ingress-based traffic stops until the Pod restarts.
</details>

## Success Criteria

- [ ] Your comparison table is complete and accurate for all characteristics
- [ ] Each scenario answer includes the correct approach with a clear justification
- [ ] You can explain why the rejected approach is wrong for each scenario
- [ ] You understand that Ingress operates at Layer 7 while LoadBalancer Services operate at Layer 4
- [ ] You can describe the traffic flow for both approaches

## What You Should Understand After This Exercise

After completing this exercise, you should be able to look at any external access requirement and immediately know whether a LoadBalancer Service or an Ingress resource is appropriate. LoadBalancer Services are the right choice for non-HTTP protocols and simple single-service exposure. Ingress is the right choice for HTTP/HTTPS traffic that needs path-based routing, host-based routing, TLS termination, or shared access across multiple services. The key insight is that Ingress consolidates many services behind a single external IP, reducing cost and centralizing HTTP-layer features.
