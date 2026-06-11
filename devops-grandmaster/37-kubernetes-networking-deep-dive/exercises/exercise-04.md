# Exercise 04: Multi-Tier Network Segmentation

**Type:** Challenge
**Time:** 60 minutes
**Objective:** Design and implement a complete network segmentation strategy for
a multi-tier application with multiple namespaces, ensuring each tier can only
communicate with its direct dependencies.

---

## Background

Your company runs an e-commerce platform with these services:

| Service | Namespace | Port | Description |
|---------|-----------|------|-------------|
| `web` | `dmz` | 3000 | User-facing web application |
| `api-gateway` | `gateway` | 8080 | API gateway routing requests |
| `product-svc` | `services` | 8081 | Product catalog service |
| `order-svc` | `services` | 8082 | Order processing service |
| `payment-svc` | `payments` | 8083 | Payment processing (PCI zone) |
| `user-svc` | `services` | 8084 | User authentication service |
| `postgres` | `data` | 5432 | Primary database |
| `redis` | `data` | 6379 | Cache and session store |

The required traffic flows:

```
Internet -> web (dmz:3000)
web -> api-gateway (gateway:8080)
api-gateway -> product-svc (services:8081)
api-gateway -> order-svc (services:8082)
api-gateway -> user-svc (services:8084)
order-svc -> payment-svc (payments:8083)
payment-svc -> postgres (data:5432)
product-svc -> postgres (data:5432)
product-svc -> redis (data:6379)
order-svc -> postgres (data:5432)
order-svc -> redis (data:6379)
user-svc -> postgres (data:5432)
user-svc -> redis (data:6379)
All services -> CoreDNS (kube-system:53)
```

Anything not listed above must be denied.

---

## Tasks

### Task 1: Deploy the Application

Create all namespaces, Deployments, and Services.

```bash
# Create namespaces
kubectl create namespace dmz
kubectl create namespace gateway
kubectl create namespace services
kubectl create namespace payments
kubectl create namespace data

# Deploy each service (using nginx as placeholder)
for ns in dmz gateway services payments data; do
  kubectl label namespace $ns env=production
done

# DMZ
kubectl create deployment web --image=nginx:alpine -n dmz
kubectl expose deployment web --port=3000 --target-port=80 -n dmz
kubectl label deployment web app=web -n dmz

# Gateway
kubectl create deployment api-gateway --image=nginx:alpine -n gateway
kubectl expose deployment api-gateway --port=8080 --target-port=80 -n gateway
kubectl label deployment api-gateway app=api-gateway -n gateway

# Services
for svc in product-svc order-svc user-svc; do
  port=$(case $svc in product-svc) echo 8081;; order-svc) echo 8082;; user-svc) echo 8084;; esac)
  kubectl create deployment $svc --image=nginx:alpine -n services
  kubectl expose deployment $svc --port=$port --target-port=80 -n services
  kubectl label deployment $svc app=$svc -n services
done

# Payments
kubectl create deployment payment-svc --image=nginx:alpine -n payments
kubectl expose deployment payment-svc --port=8083 --target-port=80 -n payments
kubectl label deployment payment-svc app=payment-svc -n payments

# Data
kubectl create deployment postgres --image=nginx:alpine -n data
kubectl expose deployment postgres --port=5432 --target-port=80 -n data
kubectl label deployment postgres app=postgres -n data

kubectl create deployment redis --image=nginx:alpine -n data
kubectl expose deployment redis --port=6379 --target-port=80 -n data
kubectl label deployment redis app=redis -n data
```

### Task 2: Design the Policy Architecture

Before writing any YAML, answer these questions:

1. How many deny-all policies do you need? One per namespace, or one global
   policy?
2. How many DNS-allow policies do you need?
3. For each allowed traffic flow listed above, which namespace gets the ingress
   policy and which namespace gets the egress policy?
4. Do any services need bidirectional communication, or is all traffic
   unidirectional?

Document your answers. Then implement.

### Task 3: Implement Default-Deny

Apply deny-all policies to every namespace. Save each as
`deny-all-<namespace>.yaml`.

### Task 4: Implement DNS Allow

Allow DNS egress from every namespace that has Pods.

### Task 5: Implement Ingress Policies

For each allowed traffic flow, create an ingress policy in the **destination**
namespace. Name each policy clearly (e.g.,
`allow-api-gateway-to-product-svc.yaml`).

Pay attention to:
- The source namespace AND Pod label (use both `namespaceSelector` and
  `podSelector` in the same `from` entry).
- The exact port number.
- The `payments` namespace is PCI-sensitive -- only `order-svc` should be able
  to reach it.

### Task 6: Implement Egress Policies

For each source service that initiates connections, create egress policies. The
source services are: `web`, `api-gateway`, `product-svc`, `order-svc`,
`payment-svc`, `user-svc`.

Each egress policy should allow:
- DNS (already done in Task 4)
- Only the specific destinations and ports listed in the traffic flows

### Task 7: Verify

Run a comprehensive test. Deploy a debug Pod in each namespace and test every
allowed and denied flow.

```bash
# Launch debug pods
for ns in dmz gateway services payments data; do
  kubectl run debug-$ns --rm -it --image=nicolaka/netshoot -n $ns -- bash
done
```

For each allowed flow, verify it works:
```bash
# From dmz: web -> api-gateway
kubectl exec -n dmz deploy/web -- curl -s --max-time 3 http://api-gateway.gateway.svc.cluster.local:8080

# From gateway: api-gateway -> product-svc
kubectl exec -n gateway deploy/api-gateway -- curl -s --max-time 3 http://product-svc.services.svc.cluster.local:8081
```

For each denied flow, verify it is blocked:
```bash
# dmz -> data (should timeout -- web should not reach database directly)
kubectl exec -n dmz deploy/web -- curl -s --max-time 3 http://postgres.data.svc.cluster.local:5432

# payments -> gateway (should timeout -- payment-svc should not initiate to api-gateway)
kubectl exec -n payments deploy/payment-svc -- curl -s --max-time 3 http://api-gateway.gateway.svc.cluster.local:8080
```

---

## Success Criteria

- [ ] All 5 namespaces have deny-all policies (ingress and egress).
- [ ] All Pods can resolve DNS.
- [ ] Every allowed traffic flow from the specification works.
- [ ] Direct database access from dmz, gateway, or payments to data services
      other than through their designated paths is blocked.
- [ ] `payment-svc` can only receive connections from `order-svc`.
- [ ] `payment-svc` can only send to `postgres` (not redis, not other
      namespaces).
- [ ] No service can reach any service not listed in the allowed flows.
- [ ] Policy names clearly indicate what traffic they allow.

---

## Hints

<details>
<summary>Hint 1: Policy Count Estimate</summary>

You will need approximately:
- 5 deny-all policies (one per namespace)
- 5 DNS-allow policies (one per namespace with Pods) -- or combine DNS into
  egress policies
- 9 ingress policies (one per allowed inbound flow)
- 6 egress policies (one per source service)

Total: approximately 20-25 NetworkPolicy resources. This is normal for a
production multi-tier application.

You can combine the DNS allow with the egress deny-all by including DNS in
your egress allow rules instead of a separate policy.

</details>

<details>
<summary>Hint 2: Payment-Svc is Special</summary>

The `payments` namespace is the PCI zone. It has the most restrictive policy:

Ingress: only from `order-svc` in `services` namespace on port 8083.
Egress: only to `postgres` in `data` namespace on port 5432, plus DNS.

Payment-svc should NOT be able to reach redis, other services, or the gateway.
This is the principle of least privilege applied to a PCI-compliant service.

</details>

<details>
<summary>Hint 3: Combining Namespace and Pod Selectors</summary>

For cross-namespace policies, always combine `namespaceSelector` and
`podSelector` in the same `from` (ingress) or `to` (egress) entry:

```yaml
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

If you put them in separate entries, the policy allows:
- ANY Pod with label `app=order-svc` in ANY namespace, OR
- ANY Pod in namespace `services`

This is too permissive. Combining them in one entry means:
- Only Pods with label `app=order-svc` in namespace `services`

</details>

<details>
<summary>Hint 4: Egress Policy Structure</summary>

For a service like `api-gateway` that needs to reach multiple destinations,
you can put multiple `to` entries in a single egress policy:

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: api-gateway-egress
  namespace: gateway
spec:
  podSelector:
    matchLabels:
      app: api-gateway
  policyTypes:
  - Egress
  egress:
  # DNS
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: kube-system
    ports:
    - protocol: UDP
      port: 53
    - protocol: TCP
      port: 53
  # product-svc
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
  # order-svc
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
  # user-svc
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

</details>

<details>
<summary>Hint 5: Verifying Denied Flows</summary>

When testing denied flows, use `--max-time 3` with curl. A denied connection
will timeout (hang for 3 seconds) rather than return "connection refused."

"Connection refused" means the port is wrong or the service does not exist.
"Timeout" means the network policy is blocking the traffic -- this is the
expected behavior for denied flows.

```bash
# This should timeout (blocked by policy)
kubectl exec -n dmz deploy/web -- curl -s --max-time 3 http://postgres.data.svc.cluster.local:5432
echo $?  # Returns 28 (curl timeout exit code)

# This should return HTML (allowed)
kubectl exec -n dmz deploy/web -- curl -s --max-time 3 http://api-gateway.gateway.svc.cluster.local:8080
echo $?  # Returns 0
```

</details>
