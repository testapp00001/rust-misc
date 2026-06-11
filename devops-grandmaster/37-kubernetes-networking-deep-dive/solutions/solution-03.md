# Solution 03: Debug Cross-Namespace Connectivity

---

## Task 2: Diagnosis -- All Problems Identified

### Problem 1: Egress deny-all in shop-backend

**What is broken:** The `orders` Pod in `shop-backend` cannot send any traffic
outside its namespace -- DNS, payments, frontend, everything times out.

**Which resource:** `deny-all-egress` NetworkPolicy in `shop-backend`.

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: deny-all-egress
  namespace: shop-backend
spec:
  podSelector: {}
  policyTypes:
  - Egress
```

**Why it breaks:** `podSelector: {}` selects all Pods. `policyTypes: [Egress]`
with no `egress` rules means all outbound traffic is denied. The orders Pod
cannot reach DNS, cannot reach payments, cannot reach anything.

**Diagnosis command:**
```bash
kubectl describe networkpolicy deny-all-egress -n shop-backend
# Shows: Spec: all egress traffic denied
```

**The fix:** Add egress rules for DNS and the required destinations.

---

### Problem 2: Wrong DNS port in shop-backend

**What is broken:** DNS resolution fails from shop-backend Pods even after
fixing the egress deny-all.

**Which resource:** `wrong-dns-policy` NetworkPolicy in `shop-backend`.

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: wrong-dns-policy
  namespace: shop-backend
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
      port: 5353  # <-- WRONG: CoreDNS listens on 53, not 5353
```

**Why it breaks:** CoreDNS listens on UDP port 53 (standard DNS). The policy
allows port 5353. DNS queries to port 53 are blocked by the deny-all egress.
The policy exists but is ineffective because it targets the wrong port.

**Diagnosis command:**
```bash
kubectl get svc -n kube-system kube-dns
# Shows: kube-dns   ClusterIP   10.96.0.10   <none>   53/UDP,53/TCP

kubectl describe networkpolicy wrong-dns-policy -n shop-backend
# Shows: port 5353 UDP -- does not match CoreDNS port 53
```

**The fix:** Change the port from 5353 to 53, or create a new correct policy.

---

### Problem 3: Ingress deny-all in shop-payments

**What is broken:** No Pod from any namespace can reach the `payments` service.
Even if orders has correct egress rules, the payments Pods block all inbound
traffic.

**Which resource:** `deny-all-ingress` NetworkPolicy in `shop-payments`.

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: deny-all-ingress
  namespace: shop-payments
spec:
  podSelector: {}
  policyTypes:
  - Ingress
```

**Why it breaks:** All Pods in shop-payments have ingress denied. No
exceptions exist. Even orders with correct egress cannot reach payments because
the destination blocks the connection.

**Diagnosis command:**
```bash
kubectl get networkpolicy -n shop-payments
# Shows: deny-all-ingress -- blocks everything

# From orders Pod, connection times out:
kubectl exec -n shop-backend deploy/orders -- curl -s --max-time 3 http://payments.shop-payments.svc.cluster.local:8080
# Timeout -- destination is blocking
```

**The fix:** Add an ingress policy in shop-payments that allows traffic from
orders.

---

### Problem 4: Frontend policy too restrictive

**What is broken:** The `allow-only-frontend-ingress` policy in `shop-frontend`
only allows ingress from `ingress-nginx`. The `orders` service in `shop-backend`
cannot reach `frontend`.

**Which resource:** `allow-only-frontend-ingress` NetworkPolicy in
`shop-frontend`.

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-only-frontend-ingress
  namespace: shop-frontend
spec:
  podSelector:
    matchLabels:
      app: frontend
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

**Why it breaks:** The policy only allows traffic from `ingress-nginx`. Orders
in `shop-backend` is not in the `ingress-nginx` namespace, so its traffic is
blocked.

**The fix:** Add orders as an allowed source, or add a second ingress rule.

---

## Task 3: Fixes

### fix-1-egress-backend.yaml

Replace the broken egress deny-all and wrong DNS policy with correct versions:

```yaml
# Fix the deny-all: keep it, but add egress rules
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-backend-egress
  namespace: shop-backend
spec:
  podSelector: {}
  policyTypes:
  - Egress
  egress:
  # DNS to kube-system (port 53, not 5353)
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: kube-system
    ports:
    - protocol: UDP
      port: 53
    - protocol: TCP
      port: 53
  # Orders -> Payments
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: shop-payments
      podSelector:
        matchLabels:
          app: payments
    ports:
    - protocol: TCP
      port: 8080
  # Orders -> Frontend
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: shop-frontend
      podSelector:
        matchLabels:
          app: frontend
    ports:
    - protocol: TCP
      port: 3000
```

Apply:
```bash
kubectl apply -f fix-1-egress-backend.yaml
```

### fix-2-ingress-payments.yaml

Add an ingress allow in shop-payments for orders:

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-orders-to-payments
  namespace: shop-payments
spec:
  podSelector:
    matchLabels:
      app: payments
  policyTypes:
  - Ingress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: shop-backend
      podSelector:
        matchLabels:
          app: orders
    ports:
    - protocol: TCP
      port: 8080
```

Apply:
```bash
kubectl apply -f fix-2-ingress-payments.yaml
```

### fix-3-ingress-frontend.yaml

Add orders as an allowed source for frontend:

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-orders-to-frontend
  namespace: shop-frontend
spec:
  podSelector:
    matchLabels:
      app: frontend
  policyTypes:
  - Ingress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: shop-backend
      podSelector:
        matchLabels:
          app: orders
    ports:
    - protocol: TCP
      port: 3000
```

Apply:
```bash
kubectl apply -f fix-3-ingress-frontend.yaml
```

---

## Task 4: Verification

```bash
# 1. DNS works from shop-backend
kubectl exec -n shop-backend deploy/orders -- nslookup payments.shop-payments.svc.cluster.local
# Expected: resolves to a ClusterIP

# 2. Orders -> Payments (should work)
kubectl exec -n shop-backend deploy/orders -- curl -s --max-time 5 http://payments.shop-payments.svc.cluster.local:8080
# Expected: HTML response from nginx

# 3. Orders -> Frontend (should work)
kubectl exec -n shop-backend deploy/orders -- curl -s --max-time 5 http://frontend.shop-frontend.svc.cluster.local:3000
# Expected: HTML response from nginx

# 4. Payments cannot initiate to orders (deny-all-ingress on payments still applies)
# This is correct -- only orders should reach payments, not the reverse
```

---

## Why It Works

The four problems represent the four failure modes of NetworkPolicies:

1. **Deny-all without allow rules** -- the blunt instrument that blocks
   everything, including what you need.

2. **Wrong port number** -- the policy exists and looks correct at a glance,
   but the port does not match the actual service. This is the hardest to
   diagnose because `kubectl describe` shows the policy is present.

3. **Missing ingress on destination** -- the source has correct egress, but
   the destination denies the connection. Both sides must agree.

4. **Overly restrictive allow list** -- the policy works, but it does not
   include all legitimate sources. Adding a new source requires updating the
   policy.

The fix strategy: never delete the deny-all policies. Instead, add specific
allow policies alongside them. The deny-all remains as the safety net, and
the allow policies create explicit exceptions.

---

## Common Mistakes

1. **Deleting deny-all to "fix" connectivity.** This removes the security
   baseline. The correct approach is to add allow policies alongside the
   deny-all.

2. **Fixing only one side of the connection.** If orders has egress but
   payments has no ingress allow, the connection still fails. Both sides need
   matching policies.

3. **Not noticing the wrong port.** Port 5353 vs 53 is easy to miss. Always
   verify the actual port with `kubectl get svc` before writing NetworkPolicies.

4. **Using `podSelector: {}` on allow policies.** This allows traffic from
   ALL Pods, not just the intended source. Use specific labels.

5. **Not testing after each fix.** Apply one fix at a time and verify. If you
   apply all fixes at once and something still does not work, you cannot
   isolate which fix failed.
