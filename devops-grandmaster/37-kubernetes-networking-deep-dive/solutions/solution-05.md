# Solution 05: Complete Network Security Policy

---

## Task 1: Traffic Flow Diagram

```
=== External Ingress ===
[ingress-nginx] --TCP:3000--> [app-frontend/web-ui]

=== Internal Service-to-Service ===
[app-frontend/web-ui] --TCP:8080--> [app-api/api-v2]
[app-api/api-v2] --TCP:8081--> [app-api/auth-service]
[app-api/api-v2] --TCP:8082--> [app-workers/*] (triggers jobs)
[app-workers/scheduler] --TCP:8080--> [app-api/api-v2]

=== Data Tier Access ===
[app-api/api-v2] --TCP:5432--> [app-data/postgres]
[app-api/api-v2] --TCP:6379--> [app-data/redis]
[app-api/auth-service] --TCP:5432--> [app-data/postgres]
[app-api/auth-service] --TCP:6379--> [app-data/redis]
[app-workers/email-worker] --TCP:5432--> [app-data/postgres]
[app-workers/report-worker] --TCP:5432--> [app-data/postgres]
[app-workers/report-worker] --TCP:9200--> [app-data/elasticsearch]

=== Monitoring ===
[monitoring/prometheus] --TCP:9090--> [all namespaces] (metrics scraping)
[monitoring/grafana] --TCP:9090--> [monitoring/prometheus]

=== Infrastructure ===
[all Pods] --UDP:53/TCP:53--> [kube-system/CoreDNS]
```

---

## Task 3: Default-Deny for All Namespaces

Apply deny-all to every namespace. Use the same structure for each:

```yaml
# policies/<namespace>/deny-all.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-all
  namespace: <NAMESPACE>
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
```

Namespaces: `ingress-nginx`, `app-frontend`, `app-api`, `app-workers`,
`app-data`, `monitoring`, `cert-manager`.

---

## Task 4: DNS Allow for All Namespaces

```yaml
# policies/<namespace>/allow-dns.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-dns
  namespace: <NAMESPACE>
spec:
  podSelector: {}
  policyTypes:
  - Egress
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: kube-system
    ports:
    - protocol: UDP
      port: 53
    - protocol: TCP
      port: 53
```

Apply to all 7 namespaces.

---

## Task 5: Ingress Policies

### app-frontend: allow ingress from ingress-nginx
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-ingress-from-nginx
  namespace: app-frontend
spec:
  podSelector:
    matchLabels:
      app: web-ui
  policyTypes:
  - Ingress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: ingress-nginx
    ports:
    - protocol: TCP
      port: 3000
```

### app-api: allow ingress from web-ui
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-ingress-from-web-ui
  namespace: app-api
spec:
  podSelector:
    matchLabels:
      app: api-v2
  policyTypes:
  - Ingress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: app-frontend
      podSelector:
        matchLabels:
          app: web-ui
    ports:
    - protocol: TCP
      port: 8080
```

### app-api: allow ingress from scheduler
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-ingress-from-scheduler
  namespace: app-api
spec:
  podSelector:
    matchLabels:
      app: api-v2
  policyTypes:
  - Ingress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: app-workers
      podSelector:
        matchLabels:
          app: scheduler
    ports:
    - protocol: TCP
      port: 8080
```

### app-workers: allow ingress from api-v2
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-ingress-from-api-v2
  namespace: app-workers
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  ingress:
  - from:
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

### app-data: allow ingress from app-api
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-ingress-from-api
  namespace: app-data
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  ingress:
  # api-v2 -> postgres
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: app-api
      podSelector:
        matchLabels:
          app: api-v2
    ports:
    - protocol: TCP
      port: 5432
  # api-v2 -> redis
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: app-api
      podSelector:
        matchLabels:
          app: api-v2
    ports:
    - protocol: TCP
      port: 6379
  # auth-service -> postgres
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: app-api
      podSelector:
        matchLabels:
          app: auth-service
    ports:
    - protocol: TCP
      port: 5432
  # auth-service -> redis
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: app-api
      podSelector:
        matchLabels:
          app: auth-service
    ports:
    - protocol: TCP
      port: 6379
```

### app-data: allow ingress from app-workers
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-ingress-from-workers
  namespace: app-data
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  ingress:
  # email-worker -> postgres
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: app-workers
      podSelector:
        matchLabels:
          app: email-worker
    ports:
    - protocol: TCP
      port: 5432
  # report-worker -> postgres
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: app-workers
      podSelector:
        matchLabels:
          app: report-worker
    ports:
    - protocol: TCP
      port: 5432
  # report-worker -> elasticsearch
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: app-workers
      podSelector:
        matchLabels:
          app: report-worker
    ports:
    - protocol: TCP
      port: 9200
```

### monitoring: allow ingress from grafana to prometheus
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-grafana-to-prometheus
  namespace: monitoring
spec:
  podSelector:
    matchLabels:
      app: prometheus
  policyTypes:
  - Ingress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: grafana
    ports:
    - protocol: TCP
      port: 9090
```

### All target namespaces: allow ingress from prometheus (metrics scraping)
Apply to app-frontend, app-api, app-workers, app-data:
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-prometheus-scrape
  namespace: <NAMESPACE>
spec:
  podSelector: {}
  policyTypes:
  - Ingress
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

---

## Task 6: Egress Policies

### app-frontend: web-ui egress to api-v2
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-egress-web-ui
  namespace: app-frontend
spec:
  podSelector:
    matchLabels:
      app: web-ui
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

### app-api: api-v2 egress
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-egress-api-v2
  namespace: app-api
spec:
  podSelector:
    matchLabels:
      app: api-v2
  policyTypes:
  - Egress
  egress:
  # -> auth-service
  - to:
    - podSelector:
        matchLabels:
          app: auth-service
    ports:
    - protocol: TCP
      port: 8081
  # -> postgres
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: app-data
      podSelector:
        matchLabels:
          app: postgres
    ports:
    - protocol: TCP
      port: 5432
  # -> redis
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: app-data
      podSelector:
        matchLabels:
          app: redis
    ports:
    - protocol: TCP
      port: 6379
  # -> app-workers (trigger jobs)
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: app-workers
    ports:
    - protocol: TCP
      port: 8080
```

### app-api: auth-service egress
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-egress-auth-service
  namespace: app-api
spec:
  podSelector:
    matchLabels:
      app: auth-service
  policyTypes:
  - Egress
  egress:
  # -> postgres
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: app-data
      podSelector:
        matchLabels:
          app: postgres
    ports:
    - protocol: TCP
      port: 5432
  # -> redis
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: app-data
      podSelector:
        matchLabels:
          app: redis
    ports:
    - protocol: TCP
      port: 6379
```

### app-workers: email-worker egress
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-egress-email-worker
  namespace: app-workers
spec:
  podSelector:
    matchLabels:
      app: email-worker
  policyTypes:
  - Egress
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: app-data
      podSelector:
        matchLabels:
          app: postgres
    ports:
    - protocol: TCP
      port: 5432
```

### app-workers: report-worker egress
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-egress-report-worker
  namespace: app-workers
spec:
  podSelector:
    matchLabels:
      app: report-worker
  policyTypes:
  - Egress
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: app-data
      podSelector:
        matchLabels:
          app: postgres
    ports:
    - protocol: TCP
      port: 5432
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: app-data
      podSelector:
        matchLabels:
          app: elasticsearch
    ports:
    - protocol: TCP
      port: 9200
```

### app-workers: scheduler egress
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-egress-scheduler
  namespace: app-workers
spec:
  podSelector:
    matchLabels:
      app: scheduler
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

### monitoring: prometheus egress
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-egress-prometheus
  namespace: monitoring
spec:
  podSelector:
    matchLabels:
      app: prometheus
  policyTypes:
  - Egress
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: app-frontend
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: app-api
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: app-workers
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: app-data
    ports:
    - protocol: TCP
      port: 9090
```

### monitoring: grafana egress to prometheus
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-egress-grafana
  namespace: monitoring
spec:
  podSelector:
    matchLabels:
      app: grafana
  policyTypes:
  - Egress
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: prometheus
    ports:
    - protocol: TCP
      port: 9090
```

---

## Task 8: eBPF Observability

### 8a. Hubble Visualization

```bash
# Enable Hubble (if using Cilium)
cilium hubble enable

# Observe all flows in the cluster
hubble observe

# Observe flows for the data tier only
hubble observe --namespace app-data

# Observe dropped/denied flows (critical for debugging)
hubble observe --verdict DROPPED

# Observe flows for a specific Pod
hubble observe --pod app-api/api-v2-xxxxx --follow

# Export flow data for analysis
hubble observe --output json | jq '.flow'
```

Hubble provides a real-time view of all network flows, including which policies
allowed or denied them. This replaces the guesswork of `curl` testing with
actual flow visibility.

### 8b. Cilium L7 Policy

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
Any other HTTP method (POST, PUT, DELETE) or path (/api/v2/users, /admin) is
denied at the eBPF level -- before the request even reaches the application.

### 8c. Hubble Alerting

```yaml
# PrometheusRule for denied flow alerting
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: hubble-denied-flows
  namespace: monitoring
spec:
  groups:
  - name: network-policy
    rules:
    - alert: HighDeniedFlowRate
      expr: rate(hubble_flows_processed_total{verdict="DROPPED"}[5m]) > 10
      for: 2m
      labels:
        severity: warning
      annotations:
        summary: "High rate of denied network flows detected"
        description: "{{ $value }} flows/sec are being denied by NetworkPolicies"
```

---

## Task 10: Policy README

### POLICY-README.md (key sections)

```markdown
# Network Policy Documentation

## Architecture

Zero-trust networking: deny all traffic by default, explicitly allow required
flows. Policies are organized by namespace in the `policies/` directory.

## Adding a New Service

1. Add a deny-all policy to the new namespace.
2. Add a DNS-allow policy.
3. Identify which existing services need to reach the new service.
4. Add ingress policies in the new namespace for each allowed source.
5. Add egress policies in the new namespace for each destination.
6. Update egress policies on existing services that need to reach the new
   service.
7. Update ingress policies on existing services the new service needs to reach.
8. Test all allowed and denied flows.
9. Update this documentation.

## Troubleshooting

### Symptom: Connection timeout
**Likely cause:** NetworkPolicy blocking traffic.
**Diagnosis:**
1. `kubectl get networkpolicy -n <namespace> -o wide`
2. Check if any policy selects the source Pod for egress.
3. Check if any policy selects the destination Pod for ingress.
4. Verify DNS egress is allowed.

### Symptom: DNS resolution failure
**Likely cause:** Missing DNS egress policy or wrong port (53 vs 5353).
**Diagnosis:**
1. `kubectl exec <pod> -- nslookup kubernetes.default`
2. Check for egress policy allowing UDP/TCP 53 to kube-system.

### Symptom: Intermittent connectivity
**Likely cause:** Multiple Pods with different labels behind the same Service.
Some Pods match a policy, others do not.
**Diagnosis:**
1. `kubectl get pods -n <namespace> --show-labels`
2. Verify all Pods behind the Service have the expected labels.
```

---

## Why It Works

**Defense in depth** is implemented at three layers:

1. **Deny-all per namespace** -- establishes the baseline. No traffic flows
   without explicit permission.

2. **Ingress + egress pairing** -- every allowed flow has matching policies on
   both the source (egress) and destination (ingress). If either side is
   missing, the flow is blocked. This prevents compromised Pods from
   exfiltrating data even if they can receive commands.

3. **Pod-specific selectors** -- policies target specific Pod labels, not
   entire namespaces. `web-ui` can reach `api-v2` but `static-assets` cannot.
   `email-worker` can reach `postgres` but not `elasticsearch`.

**Monitoring as a first-class concern.** Prometheus scraping requires both
egress from monitoring and ingress on every target namespace. This is often
forgotten, leaving monitoring blind after a network policy rollout.

---

## Common Mistakes

1. **Applying deny-all without DNS.** Every namespace needs DNS egress
   immediately after deny-all. Without DNS, all service discovery fails and
   debugging becomes impossible.

2. **Not testing denied flows.** Testing only allowed flows creates false
   confidence. A single missing deny means the entire policy set is compromised.

3. **Using `namespaceSelector` without `podSelector` on egress.** An egress
   policy that allows traffic to an entire namespace is too permissive. Combine
   both selectors.

4. **Forgetting static-assets.** The `app-frontend` namespace has two services:
   `web-ui` and `static-assets`. Only `web-ui` should reach `api-v2`. If your
   egress policy uses `podSelector: {}`, both can.

5. **Not documenting the policy set.** Six months later, nobody knows which
   traffic is allowed. The POLICY-README.md is as important as the policies
   themselves.

6. **Ignoring monitoring.** After applying deny-all, Prometheus can no longer
   scrape metrics. You lose visibility at the exact moment you need it most.
   Always add monitoring egress and target ingress as part of the initial
   rollout.
