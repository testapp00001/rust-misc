# Solution 01: Service Discovery Fundamentals

## Part A: Why Hardcoded Addresses Fail

**1. Deployment / Rolling Update**
When a service is redeployed, it gets a new container with a new IP address. All services that depend on it still point to the old IP. Impact: immediate connection failures until all dependent configs are updated and redeployed. Service discovery prevents this by resolving the service name to the current IP at runtime.

**2. Auto-Scaling**
When a new instance is added to handle load, no other service knows its address. Impact: new instances sit idle while existing instances are overloaded. Service discovery automatically includes new instances in the resolution pool.

**3. Host Failure**
If the host running a service crashes, the IP is unreachable. Hardcoded configs will keep trying the dead IP until someone manually intervenes. Impact: extended outage. Service discovery detects the failure (via health checks) and removes the dead instance from the pool.

**4. Load Balancing**
Hardcoded addresses point to a single instance. Impact: that instance gets all the traffic while other instances are underutilized. Service discovery returns multiple healthy instances, enabling load distribution.

**5. Environment Promotion**
Moving from staging to production requires changing every config file. Impact: human error during config changes leads to pointing at the wrong environment. Service discovery uses environment-specific registries, so the same code works in any environment.

**6. Maintenance / Draining**
When taking a host offline for maintenance, you must manually find and update every service that references IPs on that host. Impact: maintenance is slow and error-prone. Service discovery automatically removes drained instances.

### Common Mistakes to Avoid

- Only listing "the IP changes" as the problem. The real issue is the coupling between service identity and network location.
- Not mentioning health checks. Discovery without health checking still returns dead instances.

---

## Part B: Discovery Mechanism Comparison

| Aspect | DNS-Based | Client-Side | Server-Side |
|--------|-----------|-------------|-------------|
| **Who resolves the address?** | DNS server (e.g., CoreDNS, Consul DNS) | The calling service itself | A proxy or load balancer (e.g., Nginx, Envoy) |
| **Extra infrastructure needed?** | Minimal (built into Swarm/K8s) | Service registry (Consul, etcd) + client library per language | Service registry + proxy/load balancer cluster |
| **Load balancing approach** | DNS round-robin (crude, no health awareness) | Client-side algorithm (round-robin, weighted, least-connections) | Proxy-side algorithm (any algorithm the proxy supports) |
| **Health checking** | Limited (depends on orchestrator) | Client can check health before calling | Proxy checks health before routing |
| **Client complexity** | Zero (standard DNS lookup) | High (must include discovery client library) | Zero (standard HTTP/TCP to proxy) |

### Common Mistakes to Avoid

- Saying DNS-based discovery has health checking. Standard DNS does not check health -- the orchestrator must handle this externally.
- Saying client-side discovery is "simple." It requires a client library in every language used by your services.

---

## Part C: Choosing the Right Approach

**Scenario 1: Simple Docker Swarm deployment with 5 services.**
**Recommendation: DNS-based discovery.** Swarm provides built-in DNS resolution on overlay networks. No additional infrastructure is needed. Services discover each other by name (`postgres:5432`). This is the simplest approach and sufficient for a small, single-language deployment.

**Scenario 2: Polyglot microservices architecture with 50 services.**
**Recommendation: Server-side discovery.** With services in Python, Go, Java, and Node.js, implementing a client-side discovery library for each language is expensive. A proxy (Nginx + Consul Template, or Envoy) centralizes routing logic. Any language can call the proxy using standard HTTP. Health checking and load balancing are handled centrally.

**Scenario 3: High-frequency trading system.**
**Recommendation: Client-side discovery.** Every millisecond matters. Server-side discovery adds a proxy hop (0.5-2ms). Client-side discovery lets the caller connect directly to the target with no intermediary. The caller can also implement application-aware routing (e.g., prefer instances with warm caches, route to the nearest datacenter).

### Common Mistakes to Avoid

- Recommending the same approach for all scenarios. The trade-offs are real and significant.
- Not considering operational cost. Server-side discovery requires operating a proxy cluster. Client-side discovery requires maintaining client libraries.

---

## Key Takeaway

Service discovery is the mechanism by which services find each other in a dynamic environment. DNS-based discovery is simplest but limited in load balancing and health checking. Client-side discovery gives maximum control but couples discovery logic to every service. Server-side discovery centralizes routing but adds a hop. The right choice depends on your architecture's complexity, language diversity, and latency requirements.
