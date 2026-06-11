# Exercise 05: Complete Network Security Policy

**Type:** Integration
**Time:** 75 minutes
**Objective:** Design, implement, test, and document a complete production-ready
network security policy set for a Kubernetes cluster, integrating NetworkPolicies,
DNS configuration, and eBPF-based observability.

---

## Background

You are the platform engineer responsible for hardening a Kubernetes cluster that
runs a SaaS application. The cluster has the following namespaces and services:

| Namespace | Services | Sensitivity |
|-----------|----------|-------------|
| `ingress-nginx` | Ingress controller | Infrastructure |
| `app-frontend` | `web-ui` (port 3000), `static-assets` (port 8080) | Public-facing |
| `app-api` | `api-v2` (port 8080), `auth-service` (port 8081) | Business logic |
| `app-workers` | `email-worker`, `report-worker`, `scheduler` | Background jobs |
| `app-data` | `postgres` (port 5432), `redis` (port 6379), `elasticsearch` (port 9200) | Data tier |
| `monitoring` | `prometheus` (port 9090), `grafana` (port 3000), `alertmanager` (port 9093) | Observability |
| `cert-manager` | Certificate management | Infrastructure |

Your deliverable is a complete, tested, documented network security policy set
that implements zero-trust networking.

---

## Tasks

### Task 1: Traffic Flow Diagram

Create an ASCII traffic flow diagram showing every allowed communication path.
Use this format:

```
[ingress-nginx] --TCP:3000--> [app-frontend/web-ui]
[app-frontend/web-ui] --TCP:8080--> [app-api/api-v2]
...
```

Document every flow. Group them by:
- External ingress flows
- Internal service-to-service flows
- Data tier access flows
- Monitoring/observability flows
- Infrastructure flows (cert-manager, DNS)

### Task 2: Deploy the Application Stack

Create all namespaces and deploy placeholder services.

```bash
# Create namespaces with labels
for ns in ingress-nginx app-frontend app-api app-workers app-data monitoring cert-manager; do
  kubectl create namespace $ns
  kubectl label namespace $ns name=$ns
done

# Tag namespaces by tier
kubectl label namespace app-frontend tier=public
kubectl label namespace app-api tier=application
kubectl label namespace app-workers tier=application
kubectl label namespace app-data tier=data
kubectl label namespace monitoring tier=observability

# Deploy services (using nginx as placeholder)
# app-frontend
kubectl create deployment web-ui --image=nginx:alpine -n app-frontend
kubectl expose deployment web-ui --port=3000 --target-port=80 -n app-frontend
kubectl label deployment web-ui app=web-ui -n app-frontend

kubectl create deployment static-assets --image=nginx:alpine -n app-frontend
kubectl expose deployment static-assets --port=8080 --target-port=80 -n app-frontend
kubectl label deployment static-assets app=static-assets -n app-frontend

# app-api
kubectl create deployment api-v2 --image=nginx:alpine -n app-api
kubectl expose deployment api-v2 --port=8080 --target-port=80 -n app-api
kubectl label deployment api-v2 app=api-v2 -n app-api

kubectl create deployment auth-service --image=nginx:alpine -n app-api
kubectl expose deployment auth-service --port=8081 --target-port=80 -n app-api
kubectl label deployment auth-service app=auth-service -n app-api

# app-workers
for worker in email-worker report-worker scheduler; do
  kubectl create deployment $worker --image=nginx:alpine -n app-workers
  kubectl label deployment $worker app=$worker -n app-workers
done

# app-data
kubectl create deployment postgres --image=nginx:alpine -n app-data
kubectl expose deployment postgres --port=5432 --target-port=80 -n app-data
kubectl label deployment postgres app=postgres -n app-data

kubectl create deployment redis --image=nginx:alpine -n app-data
kubectl expose deployment redis --port=6379 --target-port=80 -n app-data
kubectl label deployment redis app=redis -n app-data

kubectl create deployment elasticsearch --image=nginx:alpine -n app-data
kubectl expose deployment elasticsearch --port=9200 --target-port=80 -n app-data
kubectl label deployment elasticsearch app=elasticsearch -n app-data

# monitoring
kubectl create deployment prometheus --image=nginx:alpine -n monitoring
kubectl expose deployment prometheus --port=9090 --target-port=80 -n monitoring
kubectl label deployment prometheus app=prometheus -n monitoring

kubectl create deployment grafana --image=nginx:alpine -n monitoring
kubectl expose deployment grafana --port=3000 --target-port=80 -n monitoring
kubectl label deployment grafana app=grafana -n monitoring

kubectl create deployment alertmanager --image=nginx:alpine -n monitoring
kubectl expose deployment alertmanager --port=9093 --target-port=80 -n monitoring
kubectl label deployment alertmanager app=alertmanager -n monitoring
```

### Task 3: Implement Default-Deny for All Namespaces

Apply deny-all ingress AND egress to every namespace. This is the foundation of
zero-trust.

Save each as `policies/<namespace>/deny-all.yaml`.

### Task 4: Implement DNS Allow for All Namespaces

Every namespace with workloads needs DNS egress. Implement this efficiently.

Save as `policies/<namespace>/allow-dns.yaml`.

### Task 5: Implement Ingress Policies

Create ingress policies for each namespace based on the traffic flows from
Task 1. Save each as `policies/<namespace>/allow-ingress-<source>.yaml`.

Key requirements:
- `app-frontend`: accept traffic from `ingress-nginx` on port 3000
- `app-api`: accept traffic from `app-frontend` (web-ui only, not
  static-assets) on port 8080; accept traffic from `app-workers` (scheduler
  only) on port 8080
- `app-workers`: accept traffic from `app-api` (api-v2 only) for triggering
  jobs
- `app-data`: accept traffic from `app-api` and `app-workers` on their
  respective ports
- `monitoring`: accept scraping traffic from prometheus to all namespaces
  (this requires egress from monitoring AND ingress on monitored services)

### Task 6: Implement Egress Policies

Create egress policies for each source service. Save each as
`policies/<namespace>/allow-egress-<service>.yaml`.

Key requirements:
- `web-ui` can reach `api-v2` on port 8080
- `api-v2` can reach `auth-service` on port 8081, `postgres` on 5432, `redis`
  on 6379
- `auth-service` can reach `postgres` on 5432, `redis` on 6379
- `email-worker` can reach `postgres` on 5432 (read templates)
- `report-worker` can reach `postgres` on 5432 and `elasticsearch` on 9200
- `scheduler` can reach `api-v2` on port 8080
- `prometheus` can scrape metrics from all namespaces on port 9090
- `grafana` can reach `prometheus` on port 9090

### Task 7: Implement Monitoring Access

Prometheus needs to scrape metrics from Pods in every namespace. This requires:

1. An egress policy on `monitoring` allowing traffic to all target namespaces.
2. Ingress policies on each target namespace allowing traffic from
   `monitoring/prometheus` on the metrics port.

Decide: should you use one ingress policy per namespace, or a single policy
with multiple `from` entries?

### Task 8: Implement an eBPF Observability Strategy

Answer these questions about using eBPF (Cilium) for network observability:

1. How would you use Cilium's Hubble to visualize the traffic flows you just
   implemented?
2. How would you create a CiliumNetworkPolicy that filters at Layer 7 (HTTP)
   to allow only `GET /api/v2/products` from `web-ui` to `api-v2`?
3. How would you set up Hubble to alert on denied traffic flows?

Provide the YAML and commands.

### Task 9: Verify the Complete Policy Set

Run a comprehensive test matrix. For each allowed flow, verify connectivity.
For each denied flow, verify blocking.

```bash
# Create a test script
cat <<'EOF' > test-network-policies.sh
#!/bin/bash
PASS=0
FAIL=0

test_connectivity() {
  local ns=$1 deploy=$2 target=$3 port=$4 expect=$5
  result=$(kubectl exec -n $ns deploy/$deploy -- \
    curl -s --max-time 3 -o /dev/null -w "%{http_code}" \
    http://$target:$port 2>/dev/null)

  if [ "$expect" = "allow" ] && [ "$result" != "000" ]; then
    echo "PASS: $ns/$deploy -> $target:$port (allowed)"
    ((PASS++))
  elif [ "$expect" = "deny" ] && [ "$result" = "000" ]; then
    echo "PASS: $ns/$deploy -> $target:$port (denied)"
    ((PASS++))
  else
    echo "FAIL: $ns/$deploy -> $target:$port (expected $expect, got $result)"
    ((FAIL++))
  fi
}

# Allowed flows
test_connectivity app-frontend web-ui "api-v2.app-api.svc.cluster.local" 8080 allow
test_connectivity app-api api-v2 "postgres.app-data.svc.cluster.local" 5432 allow

# Denied flows
test_connectivity app-frontend web-ui "postgres.app-data.svc.cluster.local" 5432 deny
test_connectivity app-workers email-worker "api-v2.app-api.svc.cluster.local" 8080 deny

echo ""
echo "Results: $PASS passed, $FAIL failed"
EOF
chmod +x test-network-policies.sh
```

### Task 10: Document the Policy Set

Create a `POLICY-README.md` that includes:
1. The traffic flow diagram from Task 1.
2. A table listing every NetworkPolicy, its namespace, and what it allows.
3. Instructions for adding a new service (onboarding guide).
4. Instructions for troubleshooting connectivity issues.

---

## Success Criteria

- [ ] All 7 namespaces have deny-all policies.
- [ ] All Pods can resolve DNS.
- [ ] Every allowed traffic flow from the specification works.
- [ ] Direct access from public tier to data tier is blocked.
- [ ] `app-workers` cannot reach `app-api` directly (except scheduler to
      api-v2).
- [ ] `app-frontend/static-assets` cannot reach `app-api` (only web-ui can).
- [ ] Prometheus can scrape metrics from all namespaces.
- [ ] Grafana can reach Prometheus.
- [ ] `monitoring` cannot reach the data tier directly.
- [ ] Traffic flow diagram matches the implemented policies.
- [ ] Policy README documents the complete policy set.
- [ ] eBPF/Hubble questions are answered with working examples.

---

## Hints

<details>
<summary>Hint 1: Organizing Policy Files</summary>

Use a directory structure that mirrors the namespace layout:

```
policies/
  ingress-nginx/
    deny-all.yaml
  app-frontend/
    deny-all.yaml
    allow-dns.yaml
    allow-ingress-from-nginx.yaml
    allow-egress-to-api.yaml
  app-api/
    deny-all.yaml
    allow-dns.yaml
    allow-ingress-from-frontend.yaml
    allow-ingress-from-scheduler.yaml
    allow-egress-to-data.yaml
    allow-egress-to-auth.yaml
  app-workers/
    deny-all.yaml
    allow-dns.yaml
    allow-ingress-from-api.yaml
    allow-egress-email-worker.yaml
    allow-egress-report-worker.yaml
    allow-egress-scheduler.yaml
  app-data/
    deny-all.yaml
    allow-dns.yaml
    allow-ingress-from-api.yaml
    allow-ingress-from-workers.yaml
  monitoring/
    deny-all.yaml
    allow-dns.yaml
    allow-egress-prometheus.yaml
    allow-ingress-grafana-to-prometheus.yaml
  cert-manager/
    deny-all.yaml
    allow-dns.yaml
```

Apply with: `kubectl apply -R -f policies/`

</details>

<details>
<summary>Hint 2: Prometheus Scraping Pattern</summary>

Prometheus scraping requires:
1. Egress from `monitoring/prometheus` to each target namespace on the metrics
   port.
2. Ingress on each target namespace from `monitoring/prometheus`.

You can use `namespaceSelector` with `matchLabels` for the tier label, or
explicitly list each namespace. Explicit listing is more secure:

```yaml
# In monitoring namespace -- prometheus egress
egress:
- to:
  - namespaceSelector:
      matchLabels:
        tier: application
  - namespaceSelector:
      matchLabels:
        tier: data
  ports:
  - protocol: TCP
    port: 9090
```

For the ingress side, each namespace needs:
```yaml
ingress:
- from:
  - namespaceSelector:
      matchLabels:
        kubernetes.io/metadata.name: monitoring
    podSelector:
      matchLabels:
        app: prometheus
  ports:
  - protocol: TCP
    port: 9090
```

</details>

<details>
<summary>Hint 3: Selective Service Access</summary>

To allow `web-ui` but NOT `static-assets` to reach `app-api`:

In the egress policy for `app-frontend`:
```yaml
egress:
- to:
  - namespaceSelector:
      matchLabels:
        kubernetes.io/metadata.name: app-api
    podSelector:
      matchLabels:
        app: api-v2
  ports:
  - protocol: TCP
    port: 8080
```

But this policy is on the `app-frontend` namespace and selects ALL Pods. You
need to make it Pod-specific:

```yaml
spec:
  podSelector:
    matchLabels:
      app: web-ui  # Only web-ui, not static-assets
  policyTypes:
  - Egress
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: app-api
      podSelector:
        matchLabels:
          app: api-v2
    ports:
    - protocol: TCP
      port: 8080
```

</details>

<details>
<summary>Hint 4: Cilium L7 Policy Example</summary>

Cilium can enforce Layer 7 HTTP policies. This goes beyond standard
NetworkPolicy:

```yaml
apiVersion: cilium.io/v2
kind: CiliumNetworkPolicy
metadata:
  name: web-ui-to-api-products-only
  namespace: app-api
spec:
  endpointSelector:
    matchLabels:
      app: api-v2
  ingress:
  - fromEndpoints:
    - matchLabels:
        app: web-ui
    toPorts:
    - ports:
      - port: "8080"
      rules:
        http:
        - method: GET
          path: "/api/v2/products"
```

This allows `web-ui` to make only `GET /api/v2/products` requests to `api-v2`.
Any other HTTP method or path is denied at the eBPF level.

</details>

<details>
<summary>Hint 5: Hubble for Observability</summary>

Hubble is Cilium's observability platform. It provides:

```bash
# Install Hubble (usually comes with Cilium)
cilium hubble enable

# Observe all flows in real-time
hubble observe

# Observe denied flows only
hubble observe --verdict DROPPED

# Observe flows for a specific namespace
hubble observe --namespace app-data

# Observe flows for a specific Pod
hubble observe --pod app-api/api-v2-xxxxx

# Flow-based alerting with Hubble Relay and Prometheus
# Export Hubble metrics to Prometheus for alerting
```

Hubble exports metrics like `hubble_flows_processed_total` and
`hubble_drop_total` which Prometheus can scrape and Alertmanager can alert on.

</details>
