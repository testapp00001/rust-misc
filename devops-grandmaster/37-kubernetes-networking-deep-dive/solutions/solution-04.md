# Solution 04: Multi-Tier Network Segmentation

---

## Task 2: Policy Architecture

### How many deny-all policies?
**5** -- one per namespace (dmz, gateway, services, payments, data).

### How many DNS-allow policies?
**5** -- one per namespace with workloads. Can be combined with egress
policies if you include DNS in each egress rule.

### Which namespace gets ingress vs. egress?
For each allowed flow:
- **Destination namespace** gets the ingress policy (allows the source in).
- **Source namespace** gets the egress policy (allows traffic out to the
  destination).

### Bidirectional communication?
No. All flows are unidirectional: `web -> api-gateway -> services -> data`.
The data tier never initiates connections to the application tier.

---

## Task 3: Default-Deny Policies

### deny-all-dmz.yaml
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-all
  namespace: dmz
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
```

### deny-all-gateway.yaml
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-all
  namespace: gateway
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
```

### deny-all-services.yaml
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-all
  namespace: services
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
```

### deny-all-payments.yaml
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-all
  namespace: payments
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
```

### deny-all-data.yaml
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-all
  namespace: data
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
```

---

## Task 4: DNS Allow Policies

Each namespace with workloads needs DNS egress. These are identical in
structure but target different namespaces.

### allow-dns-dmz.yaml (and similar for each namespace)
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-dns
  namespace: dmz
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

Apply the same pattern for gateway, services, payments, data namespaces (change
`namespace: dmz` to the appropriate namespace).

---

## Task 5: Ingress Policies

### allow-ingress-web.yaml (in dmz namespace)
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-ingress-to-web
  namespace: dmz
spec:
  podSelector:
    matchLabels:
      app: web
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

### allow-ingress-api-gateway.yaml (in gateway namespace)
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-ingress-to-api-gateway
  namespace: gateway
spec:
  podSelector:
    matchLabels:
      app: api-gateway
  policyTypes:
  - Ingress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: dmz
      podSelector:
        matchLabels:
          app: web
    ports:
    - protocol: TCP
      port: 8080
```

### allow-ingress-product-svc.yaml (in services namespace)
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-ingress-to-product-svc
  namespace: services
spec:
  podSelector:
    matchLabels:
      app: product-svc
  policyTypes:
  - Ingress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: gateway
      podSelector:
        matchLabels:
          app: api-gateway
    ports:
    - protocol: TCP
      port: 8081
```

### allow-ingress-order-svc.yaml (in services namespace)
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-ingress-to-order-svc
  namespace: services
spec:
  podSelector:
    matchLabels:
      app: order-svc
  policyTypes:
  - Ingress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: gateway
      podSelector:
        matchLabels:
          app: api-gateway
    ports:
    - protocol: TCP
      port: 8082
```

### allow-ingress-user-svc.yaml (in services namespace)
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-ingress-to-user-svc
  namespace: services
spec:
  podSelector:
    matchLabels:
      app: user-svc
  policyTypes:
  - Ingress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: gateway
      podSelector:
        matchLabels:
          app: api-gateway
    ports:
    - protocol: TCP
      port: 8084
```

### allow-ingress-payment-svc.yaml (in payments namespace)
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-ingress-to-payment-svc
  namespace: payments
spec:
  podSelector:
    matchLabels:
      app: payment-svc
  policyTypes:
  - Ingress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: services
      podSelector:
        matchLabels:
          app: order-svc
    ports:
    - protocol: TCP
      port: 8083
```

### allow-ingress-postgres.yaml (in data namespace)
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-ingress-to-postgres
  namespace: data
spec:
  podSelector:
    matchLabels:
      app: postgres
  policyTypes:
  - Ingress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: payments
      podSelector:
        matchLabels:
          app: payment-svc
    ports:
    - protocol: TCP
      port: 5432
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: services
      podSelector:
        matchLabels:
          app: product-svc
    ports:
    - protocol: TCP
      port: 5432
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: services
      podSelector:
        matchLabels:
          app: order-svc
    ports:
    - protocol: TCP
      port: 5432
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: services
      podSelector:
        matchLabels:
          app: user-svc
    ports:
    - protocol: TCP
      port: 5432
```

### allow-ingress-redis.yaml (in data namespace)
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-ingress-to-redis
  namespace: data
spec:
  podSelector:
    matchLabels:
      app: redis
  policyTypes:
  - Ingress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: services
      podSelector:
        matchLabels:
          app: product-svc
    ports:
    - protocol: TCP
      port: 6379
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: services
      podSelector:
        matchLabels:
          app: order-svc
    ports:
    - protocol: TCP
      port: 6379
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: services
      podSelector:
        matchLabels:
          app: user-svc
    ports:
    - protocol: TCP
      port: 6379
```

---

## Task 6: Egress Policies

### allow-egress-web.yaml (in dmz namespace)
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-egress-web
  namespace: dmz
spec:
  podSelector:
    matchLabels:
      app: web
  policyTypes:
  - Egress
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: gateway
      podSelector:
        matchLabels:
          app: api-gateway
    ports:
    - protocol: TCP
      port: 8080
```

### allow-egress-api-gateway.yaml (in gateway namespace)
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-egress-api-gateway
  namespace: gateway
spec:
  podSelector:
    matchLabels:
      app: api-gateway
  policyTypes:
  - Egress
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: services
      podSelector:
        matchLabels:
          app: product-svc
    ports:
    - protocol: TCP
      port: 8081
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: services
      podSelector:
        matchLabels:
          app: order-svc
    ports:
    - protocol: TCP
      port: 8082
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: services
      podSelector:
        matchLabels:
          app: user-svc
    ports:
    - protocol: TCP
      port: 8084
```

### allow-egress-product-svc.yaml (in services namespace)
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-egress-product-svc
  namespace: services
spec:
  podSelector:
    matchLabels:
      app: product-svc
  policyTypes:
  - Egress
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: data
      podSelector:
        matchLabels:
          app: postgres
    ports:
    - protocol: TCP
      port: 5432
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: data
      podSelector:
        matchLabels:
          app: redis
    ports:
    - protocol: TCP
      port: 6379
```

### allow-egress-order-svc.yaml (in services namespace)
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-egress-order-svc
  namespace: services
spec:
  podSelector:
    matchLabels:
      app: order-svc
  policyTypes:
  - Egress
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: payments
      podSelector:
        matchLabels:
          app: payment-svc
    ports:
    - protocol: TCP
      port: 8083
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: data
      podSelector:
        matchLabels:
          app: postgres
    ports:
    - protocol: TCP
      port: 5432
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: data
      podSelector:
        matchLabels:
          app: redis
    ports:
    - protocol: TCP
      port: 6379
```

### allow-egress-user-svc.yaml (in services namespace)
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-egress-user-svc
  namespace: services
spec:
  podSelector:
    matchLabels:
      app: user-svc
  policyTypes:
  - Egress
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: data
      podSelector:
        matchLabels:
          app: postgres
    ports:
    - protocol: TCP
      port: 5432
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: data
      podSelector:
        matchLabels:
          app: redis
    ports:
    - protocol: TCP
      port: 6379
```

### allow-egress-payment-svc.yaml (in payments namespace)
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-egress-payment-svc
  namespace: payments
spec:
  podSelector:
    matchLabels:
      app: payment-svc
  policyTypes:
  - Egress
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: data
      podSelector:
        matchLabels:
          app: postgres
    ports:
    - protocol: TCP
      port: 5432
```

**Note:** payment-svc egress only allows postgres (5432). It does NOT allow
redis, other services, or the gateway. This is the PCI constraint.

---

## Task 7: Verification

### Allowed flows (should succeed)
```bash
# web -> api-gateway
kubectl exec -n dmz deploy/web -- curl -s --max-time 3 http://api-gateway.gateway.svc.cluster.local:8080

# api-gateway -> product-svc
kubectl exec -n gateway deploy/api-gateway -- curl -s --max-time 3 http://product-svc.services.svc.cluster.local:8081

# api-gateway -> order-svc
kubectl exec -n gateway deploy/api-gateway -- curl -s --max-time 3 http://order-svc.services.svc.cluster.local:8082

# api-gateway -> user-svc
kubectl exec -n gateway deploy/api-gateway -- curl -s --max-time 3 http://user-svc.services.svc.cluster.local:8084

# order-svc -> payment-svc
kubectl exec -n services deploy/order-svc -- curl -s --max-time 3 http://payment-svc.payments.svc.cluster.local:8083

# payment-svc -> postgres
kubectl exec -n payments deploy/payment-svc -- curl -s --max-time 3 http://postgres.data.svc.cluster.local:5432

# product-svc -> postgres
kubectl exec -n services deploy/product-svc -- curl -s --max-time 3 http://postgres.data.svc.cluster.local:5432

# product-svc -> redis
kubectl exec -n services deploy/product-svc -- curl -s --max-time 3 http://redis.data.svc.cluster.local:6379
```

### Denied flows (should timeout)
```bash
# web -> postgres (direct database access from DMZ)
kubectl exec -n dmz deploy/web -- curl -s --max-time 3 http://postgres.data.svc.cluster.local:5432

# payment-svc -> api-gateway (payment cannot initiate to gateway)
kubectl exec -n payments deploy/payment-svc -- curl -s --max-time 3 http://api-gateway.gateway.svc.cluster.local:8080

# payment-svc -> redis (PCI: payment only talks to postgres)
kubectl exec -n payments deploy/payment-svc -- curl -s --max-time 3 http://redis.data.svc.cluster.local:6379

# product-svc -> payment-svc (services cannot directly reach payments)
kubectl exec -n services deploy/product-svc -- curl -s --max-time 3 http://payment-svc.payments.svc.cluster.local:8083
```

---

## Why It Works

**Layered segmentation** enforces the principle that each tier can only
communicate with its direct dependencies. The DMZ tier (web) can only reach the
gateway tier. The gateway can only reach the services tier. The services tier
can reach payments and data. The data tier accepts connections but never
initiates them.

**The PCI zone** (payments namespace) is the most restricted. Only order-svc
can reach it. It can only reach postgres -- not redis, not other services, not
the gateway. This isolates the payment processing path from the rest of the
application.

**Egress policies enforce source restrictions.** Even if the data tier allowed
ingress from everyone, the egress policies on the application tier prevent Pods
from sending traffic to unauthorized destinations. Defense in depth means both
sides must agree.

---

## Common Mistakes

1. **Forgetting egress for the data tier.** The data tier has deny-all egress.
   If postgres needs to reach an external backup service, you need an explicit
   egress rule. In this exercise, the data tier does not initiate connections,
   so no egress rules are needed beyond DNS.

2. **Not combining namespaceSelector and podSelector.** Using only
   `namespaceSelector` for the services namespace allows ALL Pods in services
   to reach the destination. Combining with `podSelector` restricts to the
   specific service.

3. **Allowing payment-svc to reach redis.** This violates the PCI requirement.
   Payment processing must only touch the database, not the cache. Audit this
   carefully.

4. **Not testing denied flows.** Testing only allowed flows gives false
   confidence. Always test that unauthorized paths are actually blocked.

5. **Creating permissive egress policies.** An egress policy that allows
   traffic to all namespaces on all ports defeats the purpose of segmentation.
   Each egress policy should be as specific as the ingress policy.
