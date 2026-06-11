# Solution 05: Production Service Mesh Deployment

## Part A: Service Mesh Selection

| Criterion | Istio | Linkerd | Winner |
|-----------|-------|---------|--------|
| **Feature richness** | Extensive: traffic management, security, observability, multi-cluster, VM integration | Core: mTLS, basic traffic splitting, observability | Istio |
| **Resource overhead** | Higher: ~100MB memory per sidecar, ~0.1 CPU | Lower: ~20MB memory per sidecar, ~0.01 CPU | Linkerd |
| **Ease of operation** | Complex: many CRDs, steep learning curve | Simple: few CRDs, easy installation | Linkerd |
| **Multi-cluster support** | Native: multi-cluster mesh, cross-network | Supported: multi-cluster with gateway | Istio |
| **VM integration** | Native: WorkloadEntry, WorkloadGroup | Limited: linkerd-await on VMs | Istio |
| **Community/ecosystem** | Large: CNCF graduated, extensive documentation | Growing: CNCF graduated, smaller community | Istio |
| **gRPC support** | Native (Envoy is gRPC-aware) | Native | Tie |
| **Canary deployments** | VirtualService + DestinationRule (flexible) | TrafficSplit CRD (simpler) | Istio |
| **Authorization** | AuthorizationPolicy (rich: L7 rules) | ServerAuthorization (basic) | Istio |

### Decision: Istio

For this scenario (50 services, VM integration, multi-tenant isolation),
**Istio** is the better choice:

1. **VM integration** is a hard requirement. Istio's WorkloadEntry and
   WorkloadGroup provide native VM support. Linkerd's VM support is limited.

2. **Multi-tenant isolation** requires rich authorization policies. Istio's
   AuthorizationPolicy supports namespace-level, service-level, and
   path-level rules. Linkerd's authorization is more basic.

3. **Feature richness** matters at 50 services. Traffic management (canary,
   mirroring, fault injection) and observability (distributed tracing,
   metrics) are critical at this scale.

4. **Resource overhead** is a concern at 500 pods. Istio's ~100MB per
   sidecar = 50GB total. This is significant but manageable with proper
   resource planning.

**When to choose Linkerd:**
- Smaller deployments (< 20 services)
- No VM integration needed
- Simplicity is the top priority
- Resource constraints are severe

## Part B: Installation and Configuration

### 1. Control Plane Namespace

```bash
# Create dedicated namespace for Istio control plane
kubectl create namespace istio-system

# Install Istio with production profile
istioctl install --set profile=default \
  --set values.pilot.resources.requests.cpu=500m \
  --set values.pilot.resources.requests.memory=2Gi \
  --set values.pilot.resources.limits.cpu=1000m \
  --set values.pilot.resources.limits.memory=4Gi
```

### 2. Control Plane Resource Requirements

| Component | CPU Request | CPU Limit | Memory Request | Memory Limit |
|-----------|-------------|-----------|----------------|--------------|
| istiod | 500m | 1000m | 2Gi | 4Gi |
| Ingress gateway | 200m | 500m | 256Mi | 512Mi |
| Egress gateway | 100m | 200m | 128Mi | 256Mi |

**Why these sizes:**
- istiod manages certificates, config distribution, and webhook handling
- For 500 pods, istiod needs ~2-4GB memory for config and certificate storage
- Ingress/egress gateways handle external traffic, sized based on throughput

### 3. Enable Sidecar Injection

```bash
# Enable injection for specific namespaces
kubectl label namespace production istio-injection=enabled
kubectl label namespace staging istio-injection=enabled
# Do NOT enable for kube-system, istio-system, etc.

# Verify injection is enabled
kubectl get namespace -L istio-injection
```

### 4. VM Integration (Mesh Expansion)

```yaml
# Define VM workload in Istio
apiVersion: networking.istio.io/v1beta1
kind: WorkloadEntry
metadata:
  name: legacy-vm-1
  namespace: production
spec:
  address: 10.0.5.10
  labels:
    app: legacy-service
    version: v1
  serviceAccount: legacy-service-sa

# Register VM as a service
apiVersion: networking.istio.io/v1beta1
kind: WorkloadGroup
metadata:
  name: legacy-service
  namespace: production
spec:
  metadata:
    labels:
      app: legacy-service
  template:
    serviceAccount: legacy-service-sa
    network: vm-network
```

```bash
# On the VM: install Istio sidecar
curl -sL https://istio.io/downloadIstioctl | sh -
istioctl install --set values.global.meshID=mesh1 \
  --set values.global.multiCluster.clusterName=Kubernetes

# Generate VM bootstrap configuration
istioctl x workload entry configure \
  -f workloadgroup.yaml \
  -o vm-bootstrap/ \
  --clusterID Kubernetes
```

## Part C: Multi-Tenant Isolation

### Default Deny (per namespace)

```yaml
# Deny all traffic in team-a-prod by default
apiVersion: security.istio.io/v1beta1
kind: AuthorizationPolicy
metadata:
  name: deny-all
  namespace: team-a-prod
spec:
  {}  # Empty spec = deny all
```

### Allow Team A Internal Communication

```yaml
# Allow services within team-a-prod to communicate
apiVersion: security.istio.io/v1beta1
kind: AuthorizationPolicy
metadata:
  name: allow-team-a-internal
  namespace: team-a-prod
spec:
  action: ALLOW
  rules:
  - from:
    - source:
        namespaces: ["team-a-prod"]
```

### Allow Access to Shared Platform Services

```yaml
# Allow team-a to call platform services
apiVersion: security.istio.io/v1beta1
kind: AuthorizationPolicy
metadata:
  name: allow-platform-access
  namespace: platform-prod
spec:
  selector:
    matchLabels:
      app: auth-service
  action: ALLOW
  rules:
  - from:
    - source:
        namespaces: ["team-a-prod", "team-b-prod"]
```

### Allow Platform to Call Any Service

```yaml
# Allow platform services to call any service (health checks)
apiVersion: security.istio.io/v1beta1
kind: AuthorizationPolicy
metadata:
  name: allow-platform-healthchecks
  namespace: team-a-prod
spec:
  action: ALLOW
  rules:
  - from:
    - source:
        namespaces: ["platform-prod"]
    to:
    - operation:
        methods: ["GET"]
        paths: ["/health", "/healthz", "/ready"]
```

### Enforce mTLS

```yaml
# Enforce mTLS for all production namespaces
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: default
  namespace: team-a-prod
spec:
  mtls:
    mode: STRICT
---
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: default
  namespace: team-b-prod
spec:
  mtls:
    mode: STRICT
---
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: default
  namespace: platform-prod
spec:
  mtls:
    mode: STRICT
```

## Part D: Migration Strategy

### Phase 1: Non-Critical Services (Week 1-2)

```
1. Enable sidecar injection for staging namespace
2. Deploy 3-5 non-critical services to staging with mesh
3. Verify:
   - mTLS is working (check PeerAuthentication status)
   - Metrics appear in Prometheus
   - Traces appear in Jaeger
   - Services can communicate normally
4. Fix any issues discovered
```

### Phase 2: Production Non-Critical (Week 3-4)

```
1. Enable sidecar injection for production namespace (non-critical services)
2. Migrate services one at a time:
   a. Add sidecar injection label to deployment
   b. Rolling restart: kubectl rollout restart deployment/service-name
   c. Monitor metrics for 24 hours
   d. Verify error rates and latency are unchanged
3. If issues found: remove sidecar injection label, restart
```

### Phase 3: Critical Services (Week 5-6)

```
1. Migrate critical services (payment, auth) with extra caution
2. Use PERMISSIVE mode initially (allows both mTLS and plaintext)
3. Monitor for 48 hours
4. Switch to STRICT mode
5. Monitor for another 48 hours
6. Enable authorization policies
```

### Phase 4: VM Integration (Week 7-8)

```
1. Install Istio sidecar on VMs
2. Register VMs as WorkloadEntry
3. Verify mTLS between Kubernetes and VMs
4. Monitor for 1 week
```

### Verification Checklist

```bash
# Check mTLS coverage
istioctl analyze -n production

# Check proxy sync status
istioctl proxy-status

# Verify mTLS is active
istioctl authn tls-check <pod-name> <service-name>

# Check metrics
curl http://prometheus:9090/api/v1/query?query=istio_requests_total

# Check traces
# Open Jaeger UI, search for service traces
```

### Rollback Procedure

```bash
# If issues found, remove sidecar injection
kubectl label namespace production istio-injection-

# Restart affected deployments
kubectl rollout restart deployment -n production

# Pods restart without sidecar
# Traffic returns to direct communication (no mesh)
# No application code changes needed
```

### Common Mistakes to Avoid

- **Enabling mesh for all namespaces at once.** Migrate incrementally.
  A mesh-wide failure takes down everything.
- **Skipping PERMISSIVE mode for critical services.** Start with PERMISSIVE
  to verify compatibility, then switch to STRICT.
- **Not monitoring sidecar resource usage.** Each sidecar consumes CPU and
  memory. At 500 pods, this is significant. Monitor and set resource limits.
- **Forgetting about init container permissions.** Sidecar injection requires
  a privileged init container (for iptables). Ensure your security policies
  allow this.
- **Not testing rollback.** Before migrating production, verify that
  removing the sidecar injection label and restarting pods successfully
  removes the mesh.

## Key Takeaway

Deploying a service mesh requires careful planning: select the right mesh
(Istio for features, Linkerd for simplicity), configure security (mTLS,
authorization), set up observability (metrics, traces), and migrate
incrementally. Multi-tenant isolation uses namespace-level authorization
policies. VM integration extends the mesh to non-Kubernetes workloads.
The key trade-off is operational complexity vs networking capabilities.
Always test rollback before migrating production.
