# Solution 01: Ingress vs Service LoadBalancer

---

## Part A: Comparison Table

| Characteristic | LoadBalancer Service | Ingress |
|----------------|---------------------|---------|
| OSI layer | Layer 4 (TCP/UDP) | Layer 7 (HTTP/HTTPS) |
| External IPs required for 5 services | 5 (one per service) | 1 (the Ingress controller) |
| Path-based routing | No | Yes (`/api`, `/web`, `/admin`) |
| Host-based routing | No | Yes (`api.example.com`, `web.example.com`) |
| TLS termination | No (pass-through only) | Yes (at the Ingress controller) |
| Cloud provider dependency | Yes (provisions cloud LB) | Yes (one cloud LB for the controller) |
| Rate limiting | No | Yes (via annotations) |
| Real-world use case | TCP databases, gRPC, single-service exposure | Web applications, APIs, multi-service HTTP routing |

---

## Part B: Scenario-Based Selection

**Scenario 1:** A single gRPC service that needs external access.

**Answer: LoadBalancer Service.** gRPC uses HTTP/2, which some Ingress controllers support, but a LoadBalancer Service is simpler for a single service. It provides a raw TCP/UDP connection that works with any protocol. There is no need for path-based or host-based routing when there is only one service.

**Scenario 2:** A web app with `/`, `/api`, and `/admin` as separate Deployments.

**Answer: Ingress.** Path-based routing is exactly what Ingress is designed for. A LoadBalancer Service cannot route based on URL path -- it forwards all traffic to one backend. With Ingress, a single external IP serves all three paths, each routed to the correct backend Service.

**Scenario 3:** A TCP-based database proxy on port 5432.

**Answer: LoadBalancer Service.** Ingress is designed for HTTP/HTTPS traffic. A database proxy uses raw TCP, which Ingress does not natively support. A LoadBalancer Service provides a Layer 4 connection that works with any TCP-based protocol.

**Scenario 4:** Ten microservices, each needing a unique hostname with HTTPS.

**Answer: Ingress.** Host-based routing lets a single Ingress controller direct traffic to 10 different Services based on the `Host` header. TLS termination is handled centrally at the Ingress controller. With LoadBalancer Services, you would need 10 external IPs and 10 separate TLS configurations.

---

## Part C: Architecture Diagram

**LoadBalancer approach:**

```
Internet
  |
  v
[Cloud LB 1] ---> [frontend-svc] ---> [Pod, Pod]
[Cloud LB 2] ---> [api-svc]     ---> [Pod, Pod]
[Cloud LB 3] ---> [admin-svc]   ---> [Pod, Pod]
```

**Ingress approach:**

```
Internet
  |
  v
[Cloud LB (1 IP)] ---> [Ingress Controller (Nginx/Traefik)]
                            |
                            +-- Host: myapp.com, Path: /      --> [frontend-svc] --> [Pod, Pod]
                            +-- Host: myapp.com, Path: /api    --> [api-svc]      --> [Pod, Pod]
                            +-- Host: myapp.com, Path: /admin  --> [admin-svc]    --> [Pod]
```

**1. How many external IPs are provisioned in the Ingress approach?**

One. The Ingress controller itself is exposed via a single LoadBalancer Service. All Ingress-based traffic enters through that one IP.

**2. Where does TLS termination happen in each approach?**

- LoadBalancer approach: TLS must be terminated at each LoadBalancer Service (or passed through to the backend Pods). Each service manages its own certificates.
- Ingress approach: TLS is terminated at the Ingress controller. The controller holds the certificates and forwards plain HTTP to backend Services.

**3. What happens if the Ingress controller Pod crashes?**

All Ingress-based traffic stops. No requests can be routed to any backend Service until the controller Pod restarts. This is a single point of failure, which is why production deployments run multiple replicas of the Ingress controller with a PodDisruptionBudget.

---

## Common Mistakes

1. **Using Ingress for non-HTTP protocols.** Ingress is HTTP/HTTPS only. For TCP/UDP services (databases, custom protocols), use a LoadBalancer Service or a service mesh.

2. **Assuming Ingress eliminates all LoadBalancer Services.** The Ingress controller itself is exposed via a LoadBalancer Service. You still need one cloud load balancer -- you just do not need one per application.

3. **Forgetting that Ingress requires a controller.** Unlike Services, creating an Ingress resource does nothing by itself. You must install an Ingress controller (Nginx, Traefik, HAProxy, etc.) for the Ingress to take effect.

4. **Confusing the Ingress resource with the Ingress controller.** The Ingress resource is a Kubernetes API object that defines routing rules. The Ingress controller is the software that reads those rules and configures a reverse proxy. They are separate things.

5. **Overlooking Ingress controller high availability.** Running a single replica of the Ingress controller means a single point of failure. Production deployments should run at least 2 replicas with anti-affinity rules and a PodDisruptionBudget.
