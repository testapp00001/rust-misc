# Solution 01: ClusterIP vs NodePort vs LoadBalancer

## Part A: Comparison Table

| Characteristic | ClusterIP | NodePort | LoadBalancer |
|----------------|-----------|----------|--------------|
| Default type? | Yes -- if you omit `type`, Kubernetes creates a ClusterIP service | No | No |
| External access? | No -- only reachable from within the cluster | Yes -- accessible on `<NodeIP>:<nodePort>` from outside the cluster | Yes -- provisions an external IP address through the cloud provider |
| Internal access? | Yes -- reachable from any pod in the cluster via the ClusterIP or DNS name | Yes -- also gets a ClusterIP internally | Yes -- also gets a ClusterIP and NodePort |
| Port range | Any valid port | `nodePort` must be in range 30000-32767 (auto-assigned if not specified); `port` and `targetPort` can be any valid port | Any valid port for `port` and `targetPort`; cloud provider assigns the external port (typically 80/443) |
| Cloud provider dependency? | None -- works on any Kubernetes cluster | None -- works on any Kubernetes cluster | Yes -- requires a cloud provider (AWS, GCP, Azure) or a local provisioner like MetalLB |
| Real-world use case | Internal microservice-to-microservice communication (e.g., backend API calling a database service) | Development/testing environments on bare-metal clusters; exposing services when no cloud load balancer is available | Production web applications on cloud providers that need a stable external IP and automatic load balancing |

## Part B: Scenario-Based Selection

**Scenario 1: An internal API server that only other microservices within the cluster should access.**

**Correct choice: ClusterIP.**

ClusterIP exposes the service on an internal IP address that is only reachable from within the cluster. No external traffic can reach it, which is exactly what this scenario requires. NodePort would be wrong because it opens a port on every node, making the service accessible from outside the cluster if someone knows the node IP and port. LoadBalancer would be wrong because it provisions an external IP, unnecessarily exposing the service to the internet and adding cloud provider cost for no benefit.

**Scenario 2: A development team testing from laptops on bare-metal servers with no cloud provider.**

**Correct choice: NodePort.**

NodePort opens a static port (30000-32767) on every node in the cluster. Developers can access the service by connecting to any node's IP on that port from their laptops. ClusterIP would be wrong because it is only reachable from within the cluster -- developers cannot access it from their laptops. LoadBalancer would be wrong because it requires a cloud provider to provision an external load balancer, which is not available on bare-metal. (MetalLB could solve this, but it is an additional tool not present by default.)

**Scenario 3: A production web application on AWS EKS needing a stable external IP.**

**Correct choice: LoadBalancer.**

LoadBalancer provisions an AWS Elastic Load Balancer with a stable external IP (or DNS name), providing production-grade external access with automatic health checking and integration with AWS networking. ClusterIP would be wrong because it has no external access. NodePort would be wrong because it does not provide a stable external IP -- clients would need to know individual node IPs, and those nodes can be replaced. NodePort also exposes high ports (30000-32767), which is not standard for web traffic.

**Scenario 4: A database accessed only by a backend application in the same namespace.**

**Correct choice: ClusterIP.**

The database only needs to be reachable from within the same namespace. ClusterIP provides a stable internal IP and DNS name (`database-service`) that the backend can use. NodePort would be wrong because it unnecessarily exposes the database on every node's IP, creating a security risk. LoadBalancer would be wrong because it exposes the database to the internet, which is a severe security vulnerability and an unnecessary cost.

## Part C: Layer Model

**1. Can a ClusterIP Service route traffic based on HTTP path?**

No. A ClusterIP Service operates at Layer 4 (transport layer). It forwards TCP or UDP packets to pods based on IP address and port number only. It has no visibility into HTTP headers, paths, or hostnames. A request to `/api` and a request to `/web` are treated identically -- both are just TCP connections to the same port. For path-based routing, you need a Layer 7 (application layer) component such as an Ingress controller or service mesh.

**2. Can a LoadBalancer Service perform TLS termination?**

No. A Kubernetes LoadBalancer Service operates at Layer 4. It forwards raw TCP traffic to the backend pods without inspecting or modifying the payload. It cannot decrypt TLS traffic, read certificates, or terminate HTTPS connections. For TLS termination, you need a Layer 7 component such as an Ingress controller (which terminates TLS and forwards plain HTTP to pods) or a dedicated TLS proxy.

**3. What does it mean that all three Service types operate at Layer 4?**

It means that ClusterIP, NodePort, and LoadBalancer all work at the TCP/UDP transport layer. They make routing decisions based on source/destination IP addresses and port numbers only. They cannot inspect application-layer protocols (HTTP, gRPC, WebSocket). This has several practical implications:

- No path-based routing (`/api` vs `/web`).
- No host-based routing (`api.example.com` vs `web.example.com`).
- No header-based routing (e.g., routing based on `Authorization` headers).
- No TLS termination or certificate management.
- No request-level load balancing decisions (e.g., routing based on request content).

The load balancing is connection-level, not request-level. Once a TCP connection is established to a pod, all traffic on that connection goes to the same pod. This is why Kubernetes provides Ingress controllers and service meshes for Layer 7 routing.

### Common Mistakes to Avoid

- **Assuming ClusterIP means no access at all.** ClusterIP is accessible from any pod in the cluster, including pods in other namespaces. It is internal-only, not isolated.
- **Using NodePort for production.** NodePort exposes high-numbered ports on every node, which is not suitable for production web traffic (users expect port 80 or 443). It also bypasses any cloud provider load balancing and health checking.
- **Confusing `port` and `targetPort`.** The `port` is what other pods use to reach the service. The `targetPort` is the port on the container. They are often the same, but they do not have to be. If your container listens on 8080 but you want the service available on port 80, set `port: 80` and `targetPort: 8080`.
- **Forgetting that NodePort and LoadBalancer include ClusterIP.** A NodePort service also gets a ClusterIP. A LoadBalancer service gets both a ClusterIP and a NodePort. The types are additive, not mutually exclusive.
- **Thinking LoadBalancer works everywhere.** LoadBalancer requires cloud provider integration. On bare-metal clusters, you need an additional tool like MetalLB to provision external IPs.

## Key Takeaway

ClusterIP is the default and safest choice for internal communication. NodePort adds external access via node IPs on high ports, useful for development and bare-metal. LoadBalancer adds a cloud-provisioned external IP, the standard for production. All three operate at Layer 4 (TCP/UDP), which means they cannot perform HTTP-aware routing. For path-based routing, TLS termination, and advanced traffic management, you need an Ingress controller or service mesh operating at Layer 7.
