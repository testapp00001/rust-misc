# Solution 02: Default-Deny NetworkPolicies with Selective Allow

---

## Task 1-2: Setup and Verification

The setup commands from the exercise create the namespace and deployments
correctly. The key verification is confirming open connectivity before applying
policies.

---

## Task 3: Default-Deny

### deny-all.yaml

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-all
  namespace: webapp
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
```

**Why it works:** `podSelector: {}` selects every Pod in the namespace. With
empty `ingress` and `egress` arrays under `policyTypes`, all inbound and
outbound traffic is denied. This activates the NetworkPolicy engine for every
Pod in the namespace.

---

## Task 4: Allow DNS

### allow-dns.yaml

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-dns
  namespace: webapp
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

**Why it works:** This selects all Pods and allows egress to the `kube-system`
namespace on port 53 (both UDP and TCP). The label
`kubernetes.io/metadata.name: kube-system` is automatically applied by
Kubernetes to every namespace. This is more reliable than manually labeling
kube-system.

UDP port 53 is used for standard DNS queries. TCP port 53 is used for DNS
responses larger than 512 bytes (DNSSEC, large SRV records).

---

## Task 5: Allow Ingress to Frontend

### allow-ingress-to-frontend.yaml

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-ingress-to-frontend
  namespace: webapp
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

**Why it works:** This selects only `app=frontend` Pods and allows ingress from
the `ingress-nginx` namespace on TCP port 3000. Other Pods in the `webapp`
namespace are not affected by this policy (they are still governed by the
deny-all).

The `kubernetes.io/metadata.name` label ensures the policy targets the correct
namespace even if its labels change.

---

## Task 6: Allow Frontend to API

### allow-frontend-to-api.yaml

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-frontend-to-api
  namespace: webapp
spec:
  podSelector:
    matchLabels:
      app: api
  policyTypes:
  - Ingress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: frontend
    ports:
    - protocol: TCP
      port: 8080
```

**Why it works:** This selects `app=api` Pods and allows ingress from Pods with
label `app=frontend` on TCP port 8080. Since both Pods are in the same
namespace, a `podSelector` alone is sufficient -- no `namespaceSelector` needed.

**Important:** This only handles ingress on the API side. The frontend also
needs an egress rule to send traffic out (see Task 8 notes).

---

## Task 7: Allow API to Database

### allow-api-to-database.yaml

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-api-to-database
  namespace: webapp
spec:
  podSelector:
    matchLabels:
      app: database
  policyTypes:
  - Ingress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: api
    ports:
    - protocol: TCP
      port: 5432
```

**Why it works:** Same pattern as Task 6. Selects `app=database` and allows
ingress from `app=api` on TCP port 5432.

---

## Task 8: Egress Policies for Source Pods

The ingress policies above control what the destination allows. But the
default-deny in Task 3 also blocks **egress** from all Pods. Source Pods cannot
send traffic unless you add egress rules.

### allow-frontend-egress.yaml

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-frontend-egress
  namespace: webapp
spec:
  podSelector:
    matchLabels:
      app: frontend
  policyTypes:
  - Egress
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: api
    ports:
    - protocol: TCP
      port: 8080
```

### allow-api-egress.yaml

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-api-egress
  namespace: webapp
spec:
  podSelector:
    matchLabels:
      app: api
  policyTypes:
  - Egress
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: database
    ports:
    - protocol: TCP
      port: 5432
```

**Why they work:** Each egress policy selects the source Pod and allows it to
send traffic only to the specific destination and port. Frontend can only send
to API on 8080. API can only send to database on 5432.

---

## Verification Commands

```bash
# Frontend -> API (should work)
kubectl exec -n webapp deploy/frontend -- curl -s --max-time 3 http://api.webapp.svc.cluster.local:8080

# API -> Database (should work)
kubectl exec -n webapp deploy/api -- curl -s --max-time 3 http://database.webapp.svc.cluster.local:5432

# Frontend -> Database (should timeout)
kubectl exec -n webapp deploy/frontend -- curl -s --max-time 3 http://database.webapp.svc.cluster.local:5432

# Database -> Frontend (should timeout)
kubectl exec -n webapp deploy/database -- curl -s --max-time 3 http://frontend.webapp.svc.cluster.local:3000

# DNS works for all
kubectl exec -n webapp deploy/frontend -- nslookup kubernetes.default
kubectl exec -n webapp deploy/api -- nslookup kubernetes.default
kubectl exec -n webapp deploy/database -- nslookup kubernetes.default
```

---

## Complete Policy Summary

| Policy | Namespace | Selects | Allows |
|--------|-----------|---------|--------|
| `default-deny-all` | webapp | All Pods | Nothing (deny all ingress + egress) |
| `allow-dns` | webapp | All Pods | Egress to kube-system on port 53 |
| `allow-ingress-to-frontend` | webapp | app=frontend | Ingress from ingress-nginx on port 3000 |
| `allow-frontend-to-api` | webapp | app=api | Ingress from app=frontend on port 8080 |
| `allow-api-to-database` | webapp | app=database | Ingress from app=api on port 5432 |
| `allow-frontend-egress` | webapp | app=frontend | Egress to app=api on port 8080 |
| `allow-api-egress` | webapp | app=api | Egress to app=database on port 5432 |

---

## Why It Works

The zero-trust pattern has three layers:

1. **Deny all** -- establishes the baseline. No traffic flows.
2. **Allow DNS** -- restores the minimum connectivity needed for service
   discovery. Without this, nothing works.
3. **Selective allow** -- each policy opens exactly one traffic flow, on one
   port, between two specific Pod labels.

The policies are **additive**. The deny-all selects all Pods with `podSelector: {}`.
Each subsequent policy also selects Pods (by specific label) and adds allowed
flows. A Pod with multiple policies selecting it allows the union of all
permitted flows.

Egress policies are often forgotten. The deny-all blocks both ingress AND
egress. Even if the destination allows ingress, the source cannot send without
an egress rule. Both sides must have matching policies.

---

## Common Mistakes

1. **Forgetting egress policies.** The most common mistake. Ingress policies on
   the destination are necessary but not sufficient. Source Pods also need
   egress rules.

2. **Using `podSelector: {}` on allow policies.** If you use `podSelector: {}`
   on an allow policy, it applies to all Pods. Use specific labels to scope
   each policy to the intended service.

3. **Not allowing DNS early.** Apply the DNS allow policy immediately after
   deny-all. Without DNS, service discovery fails and all inter-service
   communication breaks even if the policies are correct.

4. **Confusing same-namespace and cross-namespace selectors.** Within the same
   namespace, `podSelector` alone is sufficient. Across namespaces, you need
   `namespaceSelector` (and optionally `podSelector` within it).

5. **Testing with `curl` to the wrong Service name.** Verify your DNS names
   with `nslookup` first. If DNS is broken, `curl` timeouts look like
   NetworkPolicy failures.
