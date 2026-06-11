# Exercise 03: Debug Cross-Namespace Connectivity

**Type:** Independent
**Time:** 45 minutes
**Objective:** Diagnose and fix network connectivity failures between services
in different namespaces using systematic troubleshooting techniques.

---

## Background

You are an SRE responding to an incident. The `orders` service in namespace
`shop-backend` cannot reach the `payments` service in namespace `shop-payments`.
The application logs show `connection refused` and `connection timeout` errors.
The development team insists the Services are running and the code is correct.

Your job: find the root cause and fix it.

---

## Setup

Apply the broken environment to your cluster. This creates the problem you need
to diagnose.

Save the following as `broken-environment.yaml`:

```yaml
# Namespace: shop-backend
apiVersion: v1
kind: Namespace
metadata:
  name: shop-backend
  labels:
    team: backend
    env: production
---
# Namespace: shop-payments
apiVersion: v1
kind: Namespace
metadata:
  name: shop-payments
  labels:
    team: payments
    env: production
---
# Namespace: shop-frontend
apiVersion: v1
kind: Namespace
metadata:
  name: shop-frontend
  labels:
    team: frontend
    env: production
---
# Orders service in shop-backend
apiVersion: apps/v1
kind: Deployment
metadata:
  name: orders
  namespace: shop-backend
spec:
  replicas: 2
  selector:
    matchLabels:
      app: orders
  template:
    metadata:
      labels:
        app: orders
        tier: backend
    spec:
      containers:
      - name: orders
        image: nginx:alpine
        ports:
        - containerPort: 8080
---
apiVersion: v1
kind: Service
metadata:
  name: orders
  namespace: shop-backend
spec:
  selector:
    app: orders
  ports:
  - port: 8080
    targetPort: 80
---
# Payments service in shop-payments
apiVersion: apps/v1
kind: Deployment
metadata:
  name: payments
  namespace: shop-payments
spec:
  replicas: 2
  selector:
    matchLabels:
      app: payments
  template:
    metadata:
      labels:
        app: payments
        tier: payments
    spec:
      containers:
      - name: payments
        image: nginx:alpine
        ports:
        - containerPort: 8080
---
apiVersion: v1
kind: Service
metadata:
  name: payments
  namespace: shop-payments
spec:
  selector:
    app: payments
  ports:
  - port: 8080
    targetPort: 80
---
# Frontend service in shop-frontend
apiVersion: apps/v1
kind: Deployment
metadata:
  name: frontend
  namespace: shop-frontend
spec:
  replicas: 2
  selector:
    matchLabels:
      app: frontend
  template:
    metadata:
      labels:
        app: frontend
        tier: frontend
    spec:
      containers:
      - name: frontend
        image: nginx:alpine
        ports:
        - containerPort: 3000
---
apiVersion: v1
kind: Service
metadata:
  name: frontend
  namespace: shop-frontend
spec:
  selector:
    app: frontend
  ports:
  - port: 3000
    targetPort: 80
---
# BROKEN: NetworkPolicies that break connectivity
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: deny-all-ingress
  namespace: shop-payments
spec:
  podSelector: {}
  policyTypes:
  - Ingress
---
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: deny-all-egress
  namespace: shop-backend
spec:
  podSelector: {}
  policyTypes:
  - Egress
---
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
---
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
      port: 5353
```

Apply the environment:

```bash
kubectl apply -f broken-environment.yaml
```

---

## Tasks

### Task 1: Verify the Problem

Confirm the connectivity failures from the orders Pod.

```bash
# Try to reach payments from orders
kubectl exec -n shop-backend deploy/orders -- curl -s --max-time 5 http://payments.shop-payments.svc.cluster.local:8080

# Try DNS resolution from orders
kubectl exec -n shop-backend deploy/orders -- nslookup payments.shop-payments.svc.cluster.local

# Try to reach frontend from orders (should also fail)
kubectl exec -n shop-backend deploy/orders -- curl -s --max-time 5 http://frontend.shop-frontend.svc.cluster.local:3000
```

### Task 2: Systematic Diagnosis

You must identify **all** the problems in the environment. For each problem,
document:

1. What is broken
2. Which resource causes it
3. Why it breaks connectivity
4. The fix

Use these commands to gather information:

```bash
# List all NetworkPolicies across all relevant namespaces
kubectl get networkpolicy -n shop-backend -o wide
kubectl get networkpolicy -n shop-payments -o wide
kubectl get networkpolicy -n shop-frontend -o wide

# Describe each policy
kubectl describe networkpolicy -n shop-backend
kubectl describe networkpolicy -n shop-payments
kubectl describe networkpolicy -n shop-frontend

# Verify Services and Endpoints
kubectl get svc,endpoints -n shop-payments
kubectl get svc,endpoints -n shop-backend

# Check Pod labels
kubectl get pods -n shop-backend --show-labels
kubectl get pods -n shop-payments --show-labels

# Launch a debug pod for testing
kubectl run debug --rm -it --image=nicolaka/netshoot -n shop-backend -- bash

# Inside the debug pod:
# curl -v --max-time 5 http://payments.shop-payments.svc.cluster.local:8080
# nslookup payments.shop-payments.svc.cluster.local
# nslookup kubernetes.default
```

### Task 3: Fix All Problems

Apply fixes so that:

1. `orders` in `shop-backend` can reach `payments` in `shop-payments` on port
   8080.
2. `orders` in `shop-backend` can resolve DNS.
3. `frontend` in `shop-frontend` can receive traffic from `ingress-nginx` on
   port 3000 AND from `orders` in `shop-backend` on port 3000.
4. `payments` in `shop-payments` can receive traffic from `orders` on port 8080.
5. All deny policies remain in place (you are fixing, not removing).

Save your fixes as `fix-*.yaml` files.

### Task 4: Verify All Fixes

Run the complete verification:

```bash
# 1. Orders can resolve DNS
kubectl exec -n shop-backend deploy/orders -- nslookup payments.shop-payments.svc.cluster.local

# 2. Orders can reach payments
kubectl exec -n shop-backend deploy/orders -- curl -s --max-time 5 http://payments.shop-payments.svc.cluster.local:8080

# 3. Orders can reach frontend
kubectl exec -n shop-backend deploy/orders -- curl -s --max-time 5 http://frontend.shop-frontend.svc.cluster.local:3000

# 4. Payments still cannot initiate connections to orders (deny-all-ingress on payments blocks incoming from unknown sources)
# This is intentional -- only orders should reach payments, not the other way around

# 5. Frontend can receive from ingress-nginx (test if ingress-nginx namespace exists)
# If ingress-nginx does not exist, verify the policy structure is correct
```

---

## Success Criteria

- [ ] You identified all broken resources (there are at least 3 distinct
      problems).
- [ ] DNS resolution works from shop-backend Pods.
- [ ] orders can reach payments on port 8080.
- [ ] orders can reach frontend on port 3000.
- [ ] The deny-all policies remain in place -- you added allow rules alongside
      them, not deleted them.
- [ ] Your fix files are named `fix-*.yaml` and are applied to the correct
      namespaces.

---

## Hints

<details>
<summary>Hint 1: Diagnosing Egress Problems</summary>

If a Pod cannot reach any external service, check for an egress deny-all
policy. The `shop-backend` namespace has `deny-all-egress` which blocks ALL
outbound traffic from orders Pods.

To fix: add egress rules that allow:
- DNS to kube-system (port 53 UDP/TCP)
- Traffic to shop-payments on port 8080
- Traffic to shop-frontend on port 3000

```bash
# Quick test: does the Pod have any egress policy?
kubectl get networkpolicy -n shop-backend -o yaml | grep -A5 policyTypes
```

</details>

<details>
<summary>Hint 2: Diagnosing DNS Failures</summary>

If `nslookup` fails with `;; connection timed out; no servers could be reached`,
the Pod cannot reach CoreDNS. Check:

1. Is there an egress policy blocking port 53?
2. What port does the policy allow? (The broken environment uses port 5353,
   but CoreDNS listens on port 53.)
3. Is the namespace selector correct? (`kubernetes.io/metadata.name: kube-system`
   is the standard label.)

```bash
# Check what port CoreDNS listens on
kubectl get svc -n kube-system kube-dns

# Check the broken DNS policy
kubectl describe networkpolicy wrong-dns-policy -n shop-backend
```

</details>

<details>
<summary>Hint 3: Diagnosing Ingress Problems</summary>

If the payments namespace has `deny-all-ingress`, then no Pod can reach
payments -- even if the source has correct egress rules. You need an ingress
allow rule on the payments side.

The deny-all-ingress policy selects all Pods in shop-payments with
`podSelector: {}`. To allow traffic from orders, add a new NetworkPolicy in
shop-payments that selects `app=payments` and allows ingress from
`app=orders` in `shop-backend`.

Remember: NetworkPolicies are additive. Adding an allow policy alongside the
deny-all does not remove the deny-all. It adds an exception.

</details>

<details>
<summary>Hint 4: Cross-Namespace Selectors</summary>

To allow traffic from a specific namespace, use `namespaceSelector`. To allow
traffic from specific Pods in that namespace, combine `namespaceSelector` with
`podSelector` in the same `from` entry:

```yaml
ingress:
- from:
  - namespaceSelector:
      matchLabels:
        kubernetes.io/metadata.name: shop-backend
    podSelector:
      matchLabels:
        app: orders
```

Important: if `namespaceSelector` and `podSelector` are in the SAME `from`
entry, they are ANDed. If they are in SEPARATE `from` entries, they are ORed.

AND: "Pods with label app=orders in namespace shop-backend"
OR: "Pods with label app=orders in ANY namespace" OR "ANY Pod in namespace
shop-backend"

</details>

<details>
<summary>Hint 5: Counting the Problems</summary>

There are exactly 4 problems in the broken environment:

1. `deny-all-egress` in shop-backend blocks all outbound from orders.
2. `wrong-dns-policy` in shop-backend allows DNS on port 5353 instead of 53.
3. `deny-all-ingress` in shop-payments blocks all inbound to payments.
4. The frontend policy does not allow orders to reach frontend.

Problem 2 is subtle -- the DNS policy exists but uses the wrong port. The
policy allows UDP 5353 but CoreDNS listens on UDP 53. This means DNS fails
even though a DNS policy is present.

</details>
