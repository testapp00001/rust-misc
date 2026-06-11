# Solution 01: Reverse Proxy vs Forward Proxy vs Load Balancer

## Part A: Definitions

### 1. Forward Proxy

- Sits on the **client** side.
- The **client** configures it.
- It hides **the client** from **the internet**.
- Common use cases: **content filtering**, **privacy/anonymity**, **bypassing geo-restrictions**.

### 2. Reverse Proxy

- Sits on the **server** side.
- The **server/administrator** configures it.
- It hides **the backend servers** from **the client**.
- Common use cases: **SSL termination**, **load balancing**, **caching and compression**.

### 3. Load Balancer

- Distributes **incoming requests** across **multiple backend instances**.
- Uses algorithms such as **round-robin**, **least connections**, **IP hash**.
- Can operate at Layer **4** (TCP) or Layer **7** (HTTP).

---

## Part B: Scenario Classification

### 1. Blocking employee access to social media

**Component:** Forward proxy.

**Reasoning:** The proxy sits between internal clients (employees) and the internet. It inspects outbound requests and blocks those matching social media domains. The company controls the proxy, and employees' traffic is routed through it.

### 2. Spreading requests across three identical API servers

**Component:** Load balancer.

**Reasoning:** This is the textbook use case for a load balancer. The services are identical, and the goal is distribution, not routing or hiding. A round-robin or least-connections algorithm distributes traffic evenly.

### 3. Multiple subdomains through a single IP with SSL

**Component:** Reverse proxy.

**Reasoning:** The reverse proxy terminates SSL (handling the certificate), inspects the `Host` header to determine which backend to route to, and hides all backend services behind one public IP. A load balancer alone does not handle host-based routing or SSL certificate management.

### 4. Developer routing outbound traffic through an intermediary

**Component:** Forward proxy.

**Reasoning:** The developer is a client making outbound requests. The proxy sits between the developer and the internet, masking the developer's origin. This is a classic forward proxy use case (similar to a VPN or SOCKS proxy).

### 5. 50,000 req/s to 10 identical checkout containers

**Component:** Load balancer.

**Reasoning:** The services are identical and the primary concern is distributing high-volume traffic. A Layer 7 load balancer with health checks ensures requests only go to healthy instances. A reverse proxy can do this, but the scenario emphasizes distribution, not routing logic.

### 6. Adding authentication and WAF rules without modifying services

**Component:** Reverse proxy.

**Reasoning:** The reverse proxy intercepts all inbound requests, applies cross-cutting concerns (authentication, WAF rules, rate limiting), and then forwards to backends. This is "infrastructure-level middleware" — the backends are unaware of it.

---

## Part C: Architecture Diagram

```
┌──────────────────────┐           ┌──────────────────────────────────────────────┐
│   Internal Client    │           │               Internet                       │
│  (Office Computer)   │           │                                              │
└──────────┬───────────┘           │    ┌──────────────────────┐                  │
           │                       │    │   External Client    │                  │
           │                       │    │  (Browser / Mobile)  │                  │
           ▼                       │    └──────────┬───────────┘                  │
┌──────────────────────┐           │               │                              │
│   Forward Proxy      │           │               ▼                              │
│ (Squid / Corporate)  │           │    ┌──────────────────────┐                  │
│                      │           │    │    Reverse Proxy     │                  │
│ • Filters outbound   │           │    │  (Nginx / Traefik)   │                  │
│   traffic            │           │    │                      │                  │
│ • Logs employee      │           │    │ • SSL termination    │                  │
│   activity           │           │    │ • Host-based routing │                  │
│ • Blocks categories  │           │    │ • Rate limiting      │                  │
└──────────┬───────────┘           │    └──────────┬───────────┘                  │
           │                       │               │                              │
           ▼                       │               │                              │
    ┌─────────────┐                │               ▼                              │
    │  Internet   │                │    ┌──────────────────────┐                  │
    └─────────────┘                │    │    Load Balancer     │                  │
                                   │    │   (HAProxy / Nginx)  │                  │
                                   │    │                      │                  │
                                   │    │ • Health checks      │                  │
                                   │    │ • Round-robin        │                  │
                                   │    │ • Session affinity   │                  │
                                   │    └──────────┬───────────┘                  │
                                   │               │                              │
                                   │    ┌──────────┼──────────┐                   │
                                   │    │          │          │                   │
                                   │    ▼          ▼          ▼                   │
                                   │ ┌──────┐ ┌──────┐ ┌──────┐                  │
                                   │ │ API  │ │ API  │ │ API  │                  │
                                   │ │  #1  │ │  #2  │ │  #3  │                  │
                                   │ └──────┘ └──────┘ └──────┘                  │
                                   └──────────────────────────────────────────────┘
```

**Traffic flow summary:**
- Internal client -> Forward proxy -> Internet (outbound, client-initiated)
- External client -> Reverse proxy -> Load balancer -> Backend instances (inbound, server-side)

---

## Part D: Overlap Question

### Why we still distinguish these components

Even though capabilities overlap, each component has a **primary responsibility**:

1. **Reverse proxy** focuses on **routing and request transformation**. It decides *where* a request goes based on path, hostname, headers, or cookies. It also handles cross-cutting concerns like SSL termination, compression, caching, and authentication. Its routing rules can be complex (100+ location blocks for a microservices platform).

2. **Load balancer** focuses on **distribution and availability**. It spreads traffic across *identical* instances of a single service, performs health checks, and removes unhealthy backends from the pool. Its algorithms (round-robin, least connections, weighted) are optimized for this one job.

3. Separating them provides:
   - **Fault isolation.** If the load balancer for the API fails, the frontend and other services behind the reverse proxy are unaffected.
   - **Independent scaling.** You can scale the load balancer for high-traffic services without touching the reverse proxy.
   - **Specialized optimization.** A dedicated L4 load balancer (like AWS NLB) handles raw TCP throughput better than Nginx for certain workloads.
   - **Operational clarity.** Different teams can own different layers. The platform team manages the reverse proxy; the service team manages their own load balancer.

### When to deploy a dedicated load balancer behind a reverse proxy

Deploy a separate load balancer when:

- A single service has **more instances than the reverse proxy can efficiently manage** (e.g., 50 API containers).
- You need **L4 (TCP) load balancing** for non-HTTP protocols (gRPC, databases, message queues).
- You want **per-service load balancing policies** (e.g., the checkout service uses least-connections while the product service uses round-robin).
- You are running in a cloud environment where a managed load balancer (ALB, NLB) provides health checks, auto-scaling integration, and DDoS protection that would be complex to replicate in Nginx.

---

## Common Mistakes

1. **Confusing "proxy" with "VPN."** A VPN encrypts traffic between two points. A forward proxy makes requests on behalf of clients but does not necessarily encrypt the tunnel itself.

2. **Thinking a reverse proxy IS a load balancer.** A reverse proxy *can* load balance, but it does much more (routing, SSL, caching). A load balancer *can* terminate SSL, but it does not do path-based routing or request rewriting. The overlap is real, but the primary purpose differs.

3. **Drawing traffic flow in the wrong direction.** A forward proxy is configured by the client and sends traffic *outward*. A reverse proxy is configured by the server and accepts traffic *inward*. Always label your arrows.
