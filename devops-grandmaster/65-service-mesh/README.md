# 65 - Service Mesh

## Istio, Linkerd, mTLS, Traffic Management

> **Previous:** [64 - CDN & Edge](../64-cdn-edge/README.md) | **Next:** [66 - Network Troubleshooting](../66-network-troubleshooting/README.md)

A service mesh is a dedicated infrastructure layer for handling service-to-service communication. It provides traffic management, security, and observability without requiring changes to application code. The mesh handles mTLS, retries, circuit breaking, canary routing, and telemetry transparently via sidecar proxies.

---

## Problem

In a microservices architecture, every service must handle cross-cutting concerns:

```
Without Service Mesh:
  Service A ──> Service B
     |              |
     ├── TLS setup  ├── TLS setup
     ├── Retries    ├── Retries
     ├── Timeout    ├── Timeout
     ├── Metrics    ├── Metrics
     ├── Tracing    ├── Tracing
     └── Auth       └── Auth

Every service implements its own:
  - TLS certificate management
  - Retry logic with backoff
  - Circuit breaking
  - Distributed tracing headers
  - Metrics collection
  - Access logging
```

Problems:
- Each service re-implements the same networking logic (often differently)
- Language-specific libraries create inconsistency
- Certificate rotation requires redeploying every service
- No unified view of traffic across the mesh
- Canary deployments require application-level logic
- Debugging requires instrumenting every service individually

---

## Naive Way: Per-Service Networking Logic

```rust
// Every service has its own networking code
use reqwest::Client;
use std::time::Duration;

async fn call_service_b(client: &Client) -> Result<String, reqwest::Error> {
    // Manual TLS configuration
    // Manual retry logic
    // Manual timeout
    // Manual circuit breaking
    // Manual tracing headers
    // Manual metrics

    let mut attempts = 0;
    let max_retries = 3;

    loop {
        attempts += 1;
        match client
            .get("https://service-b.internal:8443/api/data")
            .timeout(Duration::from_secs(5))
            .header("X-Request-ID", uuid::Uuid::new_v4().to_string())
            .header("X-Trace-ID", get_current_trace_id())
            .send()
            .await
        {
            Ok(resp) => {
                record_metric("service_b_request", "success");
                return resp.text().await;
            }
            Err(e) if attempts < max_retries && is_retryable(&e) => {
                record_metric("service_b_request", "retry");
                tokio::time::sleep(Duration::from_millis(100 * 2u64.pow(attempts))).await;
            }
            Err(e) => {
                record_metric("service_b_request", "failure");
                return Err(e);
            }
        }
    }
}
```

Problems with this approach:
- Every service in every language rewrites this logic
- Certificate management is scattered across all services
- Retries, timeouts, and circuit breakers are inconsistent
- Observability gaps between services
- Changes require redeploying all services

---

## Right Way: Sidecar Proxy Pattern

### How a Service Mesh Works

```
With Service Mesh:

  Pod A                          Pod B
  +------------------+           +------------------+
  | +---------+      |           |      +---------+ |
  | | Service |<----+|           |+---->| Service | |
  | +---------+     ||           ||     +---------+ |
  |             +---v|  mTLS    |^---+             |
  |             |Sidecar|<------>|Sidecar|          |
  |             |Proxy  |       |Proxy  |          |
  |             +-------+       +-------+          |
  +------------------+           +------------------+

  All traffic flows through sidecar proxies:
  - mTLS is automatic (proxy handles certificates)
  - Retries, timeouts, circuit breaking in proxy
  - Metrics, tracing, logging in proxy
  - Application code is unaware of the mesh
```

### Istio Architecture

```
                    +-------------------+
                    |  Control Plane    |
                    |  (istiod)         |
                    +--------+----------+
                             |
              +--------------+--------------+
              |              |              |
    +---------v----+  +------v------+  +---v-----------+
    | Pilot        |  | Citadel     |  | Galley        |
    | (Config)     |  | (CA/Certs)  |  | (Validation)  |
    +--------------+  +-------------+  +---------------+
              |
              | xDS API (config push)
              |
    +---------+---------+---------+
    |         |         |         |
  +---+    +---+    +---+    +---+
  |Pod|    |Pod|    |Pod|    |Pod|
  |+--+|   |+--+|   |+--+|   |+--+|
  |Side|   |Side|   |Side|   |Side|
  |car |   |car |   |car |   |car |
  +----+   +----+   +----+   +----+
```

### Installing Istio

```bash
# Download Istio
curl -L https://istio.io/downloadIstio | ISTIO_VERSION=1.20.0 sh -
cd istio-1.20.0
export PATH=$PWD/bin:$PATH

# Install with demo profile (includes observability tools)
istioctl install --set profile=demo -y

# Verify installation
istioctl verify-install

# Enable sidecar injection for a namespace
kubectl label namespace production istio-injection=enabled

# Deploy your application -- sidecars are injected automatically
kubectl apply -f myapp-deployment.yaml

# Verify sidecar injection
kubectl get pods -n production -o jsonpath='{.items[*].spec.containers[*].name}'
# Should show: myapp istio-proxy
```

### Istio Traffic Management

#### Virtual Service (Routing Rules)

```yaml
# virtual-service.yaml
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: myapp
  namespace: production
spec:
  hosts:
    - myapp
  http:
    # Route 1: Canary -- 10% to v2, 90% to v1
    - match:
        - headers:
            x-canary:
              exact: "true"
      route:
        - destination:
            host: myapp
            subset: v2
          weight: 100

    - route:
        - destination:
            host: myapp
            subset: v1
          weight: 90
        - destination:
            host: myapp
            subset: v2
          weight: 10
      retries:
        attempts: 3
        perTryTimeout: 2s
        retryOn: "5xx,reset,connect-failure"
      timeout: 10s
```

#### Destination Rule (Traffic Policy)

```yaml
# destination-rule.yaml
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: myapp
  namespace: production
spec:
  host: myapp
  trafficPolicy:
    connectionPool:
      tcp:
        maxConnections: 100
      http:
        h2UpgradePolicy: DEFAULT
        http1MaxPendingRequests: 100
        http2MaxRequests: 1000
    loadBalancer:
      simple: LEAST_REQUEST
    outlierDetection:
      consecutive5xxErrors: 5
      interval: 30s
      baseEjectionTime: 30s
      maxEjectionPercent: 50
  subsets:
    - name: v1
      labels:
        version: v1
    - name: v2
      labels:
        version: v2
```

### Automatic mTLS with Istio

```yaml
# PeerAuthentication -- require mTLS for all traffic
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: default
  namespace: production
spec:
  mtls:
    mode: STRICT  # All traffic must be mTLS
---
# AuthorizationPolicy -- who can talk to whom
apiVersion: security.istio.io/v1beta1
kind: AuthorizationPolicy
metadata:
  name: api-service-policy
  namespace: production
spec:
  selector:
    matchLabels:
      app: api-service
  rules:
    - from:
        - source:
            principals:
              - "cluster.local/ns/production/sa/web-frontend"
              - "cluster.local/ns/production/sa/mobile-backend"
      to:
        - operation:
            methods: ["GET", "POST"]
            paths: ["/api/*"]
    - from:
        - source:
            principals:
              - "cluster.local/ns/production/sa/admin-service"
      to:
        - operation:
            methods: ["GET", "POST", "PUT", "DELETE"]
            paths: ["/api/*", "/admin/*"]
```

### Linkerd (Lightweight Alternative)

```bash
# Install Linkerd CLI
curl -fsL https://run.linkerd.io/install | sh
export PATH=$HOME/.linkerd2/bin:$PATH

# Install Linkerd control plane
linkerd install --crds | kubectl apply -f -
linkerd install | kubectl apply -f -

# Verify
linkerd check

# Inject sidecars into existing deployments
kubectl get deploy -o yaml | linkerd inject - | kubectl apply -f -

# View the mesh
linkerd viz dashboard &

# Check mTLS status
linkerd edges -n production
# Shows which connections are secured with mTLS
```

---

## Production Way: Canary Deployments with Service Mesh

### Canary Deployment Flow

```
Traffic Split with Istio:

  100% ──> v1 (stable)
  
  Step 1: Deploy v2, split traffic
  90% ──> v1 (stable)
  10% ──> v2 (canary)
  
  Step 2: Monitor metrics, increase traffic
  50% ──> v1
  50% ──> v2
  
  Step 3: All good? Promote v2
  0%  ──> v1
  100% ──> v2 (now stable)
  
  Rollback at any step if metrics degrade
```

### Automated Canary with Flagger

```yaml
# flagger-canary.yaml
apiVersion: flagger.app/v1beta1
kind: Canary
metadata:
  name: myapp
  namespace: production
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: myapp
  progressDeadlineSeconds: 600
  service:
    port: 8080
    targetPort: 8080
    gateways:
      - public-gateway.istio-system.svc.cluster.local
    hosts:
      - myapp.example.com
  analysis:
    # Schedule interval
    interval: 1m
    # Max number of failed metric checks before rollback
    threshold: 5
    # Max traffic percentage routed to canary
    maxWeight: 50
    # Canary increment step
    stepWeight: 10
    metrics:
      - name: request-success-rate
        thresholdRange:
          min: 99
        interval: 1m
      - name: request-duration
        thresholdRange:
          max: 500
        interval: 30s
    webhooks:
      # Pre-rollout test
      - name: acceptance-test
        type: pre-rollout
        url: http://flagger-loadtester.test/
        timeout: 30s
        metadata:
          type: bash
          cmd: "curl -sf http://myapp-canary.production:8080/health"
      # Load test during canary
      - name: load-test
        type: rollout
        url: http://flagger-loadtester.test/
        timeout: 5s
        metadata:
          cmd: "hey -z 1m -q 10 -c 2 http://myapp-canary.production:8080/"
```

### Observability with Kiali, Prometheus, Grafana

```yaml
# Install Kiali for service mesh visualization
apiVersion: kiali.io/v1alpha1
kind: Kiali
metadata:
  name: kiali
  namespace: istio-system
spec:
  auth:
    strategy: anonymous
  deployment:
    accessible_namespaces: ["**"]
  external_services:
    prometheus:
      url: "http://prometheus.istio-system:9090"
    grafana:
      url: "http://grafana.istio-system:3000"
    tracing:
      url: "http://tracing.istio-system:16686"
```

### Service Mesh Metrics

```yaml
# Prometheus queries for service mesh monitoring
# Request rate by service
sum(rate(istio_requests_total{destination_service="myapp.production.svc.cluster.local"}[5m])) by (destination_version)

# Error rate
sum(rate(istio_requests_total{destination_service="myapp.production.svc.cluster.local", response_code=~"5.."}[5m])) /
sum(rate(istio_requests_total{destination_service="myapp.production.svc.cluster.local"}[5m]))

# P99 latency
histogram_quantile(0.99, sum(rate(istio_request_duration_milliseconds_bucket{destination_service="myapp.production.svc.cluster.local"}[5m])) by (le))

# mTLS status
istio_requests_total{connection_security_policy="mutual_tls"}
```

---

## Hands-On Lab

### Lab: Deploy a Service Mesh with Canary Releases

#### Part 1: Install Istio and Deploy Services

```bash
# Install Istio
istioctl install --set profile=demo -y

# Create namespace with sidecar injection
kubectl create namespace mesh-demo
kubectl label namespace mesh-demo istio-injection=enabled

# Deploy v1 of the application
cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp-v1
  namespace: mesh-demo
spec:
  replicas: 3
  selector:
    matchLabels:
      app: myapp
      version: v1
  template:
    metadata:
      labels:
        app: myapp
        version: v1
    spec:
      containers:
        - name: myapp
          image: myapp:v1
          ports:
            - containerPort: 8080
          env:
            - name: VERSION
              value: "v1"
EOF

# Deploy v2 (canary)
cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp-v2
  namespace: mesh-demo
spec:
  replicas: 1
  selector:
    matchLabels:
      app: myapp
      version: v2
  template:
    metadata:
      labels:
        app: myapp
        version: v2
    spec:
      containers:
        - name: myapp
          image: myapp:v2
          ports:
            - containerPort: 8080
          env:
            - name: VERSION
              value: "v2"
EOF

# Create service
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: Service
metadata:
  name: myapp
  namespace: mesh-demo
spec:
  selector:
    app: myapp
  ports:
    - port: 8080
      targetPort: 8080
EOF
```

#### Part 2: Configure Traffic Routing

```bash
# Create subsets
cat <<EOF | kubectl apply -f -
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: myapp
  namespace: mesh-demo
spec:
  host: myapp
  subsets:
    - name: v1
      labels:
        version: v1
    - name: v2
      labels:
        version: v2
EOF

# 100% to v1 initially
cat <<EOF | kubectl apply -f -
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: myapp
  namespace: mesh-demo
spec:
  hosts:
    - myapp
  http:
    - route:
        - destination:
            host: myapp
            subset: v1
          weight: 100
EOF

# Test -- all traffic to v1
for i in $(seq 1 10); do
  kubectl exec -it deploy/myapp-v1 -n mesh-demo -c myapp -- \
    curl -s http://myapp:8080/version
done
```

#### Part 3: Canary Deployment

```bash
# Shift 10% to v2
cat <<EOF | kubectl apply -f -
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: myapp
  namespace: mesh-demo
spec:
  hosts:
    - myapp
  http:
    - route:
        - destination:
            host: myapp
            subset: v1
          weight: 90
        - destination:
            host: myapp
            subset: v2
          weight: 10
      retries:
        attempts: 3
        perTryTimeout: 2s
      timeout: 10s
EOF

# Monitor the canary
# In another terminal:
kubectl -n mesh-demo port-forward svc/kiali 20001:20001
# Open http://localhost:20001 and view the graph
```

#### Part 4: Enable mTLS

```bash
# Require mTLS for all traffic in the namespace
cat <<EOF | kubectl apply -f -
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: default
  namespace: mesh-demo
spec:
  mtls:
    mode: STRICT
EOF

# Verify mTLS is active
istioctl x describe pod $(kubectl get pod -n mesh-demo -l app=myapp,version=v1 -o jsonpath='{.items[0].metadata.name}') -n mesh-demo

# Check that connections are mTLS
istioctl proxy-config listener $(kubectl get pod -n mesh-demo -l app=myapp,version=v1 -o jsonpath='{.items[0].metadata.name}') -n mesh-demo --port 8080
```

#### Part 5: Rollback Canary

```bash
# If metrics degrade, rollback to v1
cat <<EOF | kubectl apply -f -
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: myapp
  namespace: mesh-demo
spec:
  hosts:
    - myapp
  http:
    - route:
        - destination:
            host: myapp
            subset: v1
          weight: 100
EOF

echo "Rolled back to v1"
```

---

## Limitation

A service mesh adds a sidecar proxy to every pod, which increases resource consumption and adds latency to every request. The control plane (istiod) becomes a critical component that requires monitoring and high availability. When things go wrong in the mesh -- mTLS handshake failures, incorrect routing rules, certificate expiry -- troubleshooting requires deep knowledge of the mesh internals, not just application code.

---

## Next Topic

[66 - Network Troubleshooting](../66-network-troubleshooting/README.md) -- tcpdump, traceroute, packet analysis, and systematic approaches to debugging network issues in Kubernetes and service mesh environments.
