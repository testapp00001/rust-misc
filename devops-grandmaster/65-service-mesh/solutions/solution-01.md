# Solution 01: Service Mesh Concepts

## Part A: Service Mesh Fundamentals

### 1. What is a service mesh?

A **service mesh** is a dedicated infrastructure layer for handling
service-to-service communication. It provides networking capabilities --
encryption, traffic management, observability, and resilience -- without
requiring changes to application code.

**Problem it solves:** In a microservices architecture, each service must
implement networking logic (retries, timeouts, TLS, metrics). This leads
to duplicated code, inconsistent behavior, and coupling between business
logic and networking concerns. A service mesh externalizes these concerns
into the infrastructure.

### 2. What is a sidecar proxy?

A **sidecar proxy** is a separate process deployed alongside each service
container in the same pod. It intercepts all inbound and outbound network
traffic for the service.

```
Pod:
├── Application Container (your code)
│     └── Listens on localhost:8080
│     └── Sends requests to localhost:8080 (thinks it is direct)
│
└── Sidecar Proxy (Envoy / linkerd2-proxy)
      └── Intercepts all inbound traffic → forwards to app container
      └── Intercepts all outbound traffic → routes to destination sidecar
      └── Handles mTLS, retries, timeouts, metrics
```

The application is unaware of the sidecar. It sends requests to
`localhost:8080` and receives responses, but the sidecar handles all
networking in between.

### 3. Data Plane vs Control Plane

| Component | Role | Example |
|-----------|------|---------|
| **Data Plane** | Handles actual traffic (request forwarding, encryption, load balancing) | Envoy proxies (sidecars) |
| **Control Plane** | Configures the data plane (policies, certificates, routing rules) | Istio istiod, Linkerd control plane |

```
Control Plane (istiod)
  │
  │  Configures proxies
  │  Issues certificates
  │  Collects metrics
  │
  ├──→ Sidecar Proxy A (data plane)
  ├──→ Sidecar Proxy B (data plane)
  └──→ Sidecar Proxy C (data plane)
```

The control plane does NOT handle traffic. It only manages configuration.
The data plane handles all traffic but is dumb (follows control plane
instructions).

### 4. Service Mesh vs API Gateway

| Aspect | Service Mesh | API Gateway |
|--------|-------------|-------------|
| **Scope** | Service-to-service (internal) | Client-to-service (north-south) |
| **Traffic direction** | East-west (between services) | North-south (external to internal) |
| **Deployment** | One proxy per service pod | One proxy per entry point |
| **Features** | mTLS, circuit breaking, retries, distributed tracing | Rate limiting, auth, API routing, request transformation |
| **Examples** | Istio, Linkerd, Consul Connect | Kong, Ambassador, AWS API Gateway |

A service mesh and API gateway are complementary:
- API gateway handles external traffic entering the cluster
- Service mesh handles internal traffic between services

## Part B: Sidecar Injection

### 1. Pod Specification Changes

```yaml
# Before service mesh
apiVersion: v1
kind: Pod
metadata:
  name: my-app
spec:
  containers:
  - name: my-app
    image: my-app:v1

# After service mesh (sidecar injected)
apiVersion: v1
kind: Pod
metadata:
  name: my-app
  annotations:
    sidecar.istio.io/inject: "true"
spec:
  containers:
  - name: my-app
    image: my-app:v1
  - name: istio-proxy          # ← Added by mesh
    image: envoy:v1.28
    ports:
    - containerPort: 15001     # ← Inbound capture
    - containerPort: 15006     # ← Outbound capture
```

### 2. Traffic Flow Diagram

```
Without Service Mesh:
  Service A ──── HTTP ────→ Service B
  (plaintext, no auth, no metrics)

With Service Mesh:
  Service A → Sidecar A → mTLS → Sidecar B → Service B
  (encrypted, authenticated, measured, retry-capable)

Detailed flow:
  1. Service A sends HTTP request to "service-b:8080"
  2. iptables rules redirect to Sidecar A (port 15006)
  3. Sidecar A applies policies (timeout, retry)
  4. Sidecar A establishes mTLS connection to Sidecar B
  5. Sidecar B verifies Sidecar A's certificate
  6. Sidecar B forwards request to Service B (localhost:8080)
  7. Service B responds to Sidecar B
  8. Response flows back through Sidecar B → Sidecar A → Service A
```

### 3. What Happens to Application Network Traffic

The sidecar uses **iptables** rules to intercept all traffic:

```bash
# iptables rules (simplified, injected by init container)
iptables -t nat -A PREROUTING -j ISTIO_INBOUND
iptables -t nat -A OUTPUT -j ISTIO_OUTPUT

# Inbound: redirect all traffic to port 15006 (sidecar inbound)
iptables -t nat -A ISTIO_INBOUND -p tcp -j REDIRECT --to-ports 15006

# Outbound: redirect all traffic to port 15001 (sidecar outbound)
iptables -t nat -A ISTIO_OUTPUT -p tcp -j REDIRECT --to-ports 15001
```

The application is unaware. It sends to `service-b:8080` and receives
a response from `service-b:8080`. The sidecar transparently handles
encryption, routing, and observability.

## Part C: Feature Mapping

| Problem | Service Mesh Feature | How It Helps |
|---------|---------------------|--------------|
| **No encryption** | mTLS (mutual TLS) | Automatic encryption and authentication between all services. Certificates provisioned and rotated by control plane. |
| **No visibility** | Distributed tracing + metrics | Sidecar proxies emit metrics (request rate, latency, errors) and propagate trace headers for end-to-end tracing. |
| **Manual load balancer changes** | Traffic routing (VirtualService) | Declarative traffic rules: canary, A/B testing, header-based routing. Changes apply without restarting services. |
| **Debugging across 30 services** | Distributed tracing (Jaeger/Zipkin) | End-to-end traces show exactly where time is spent in a request chain. Service map shows dependencies. |
| **No rate limiting** | Rate limiting policies | Mesh-level rate limiting without application code. Configured declaratively. |
| **Inconsistent circuit breaking** | Circuit breaker (outlier detection) | Consistent circuit breaking across all services regardless of language. Configured once at the mesh level. |

### Common Mistakes to Avoid

- **Thinking a service mesh replaces an API gateway.** They serve different
  purposes. Use both: API gateway for north-south, service mesh for east-west.
- **Deploying a service mesh for 5 services.** The operational overhead is
  not justified for small architectures. Service meshes shine at 15+ services.
- **Not planning for sidecar resource overhead.** Each sidecar consumes
  50-100MB memory and some CPU. At 500 pods, that is 25-50GB of extra memory.
- **Enabling all features at once.** Start with mTLS and observability.
  Add traffic management and policies incrementally.

## Key Takeaway

A service mesh externalizes networking concerns (encryption, routing,
observability, resilience) from application code into infrastructure.
Sidecar proxies intercept all traffic transparently. The control plane
manages configuration and certificates. The key benefit is consistency:
networking behavior is uniform across all services regardless of language
or framework.
