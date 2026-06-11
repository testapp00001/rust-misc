# Solution 02: mTLS Configuration

## Part A: TLS vs mTLS

| Aspect | TLS | mTLS |
|--------|-----|------|
| **Who is authenticated?** | Server only | Both server AND client |
| **Certificate required on** | Server | Both server and client |
| **Use case** | Browser to website (HTTPS) | Service-to-service (zero trust) |
| **Handshake steps** | Server sends cert → Client verifies | Server sends cert → Client verifies → Client sends cert → Server verifies |
| **Trust model** | Client trusts server | Mutual trust (both verify) |
| **Certificate management** | Manual (or Let's Encrypt) | Automatic (control plane issues certs) |

### Why mTLS Matters for Microservices

In a zero-trust network, you cannot assume any service is trustworthy just
because it is on the same network. mTLS ensures:
- Every service has a cryptographic identity
- Services verify each other before communicating
- All traffic is encrypted (no plaintext on the network)
- Identity is tied to the service, not the IP address

## Part B: Istio mTLS Configuration

### 1. Enable Strict mTLS for Namespace

```yaml
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: default
  namespace: production
spec:
  mtls:
    mode: STRICT
```

**What this does:** Every pod in the `production` namespace MUST use mTLS.
Plaintext connections are rejected. The sidecar proxy enforces this
automatically.

**Modes:**
- `STRICT`: Only mTLS connections accepted
- `PERMISSIVE`: Accept both mTLS and plaintext (migration mode)
- `DISABLE`: No mTLS (plaintext only)
- `UNSET`: Inherit from parent scope

### 2. Allow Only api-service to Call payment-service

```yaml
apiVersion: security.istio.io/v1beta1
kind: AuthorizationPolicy
metadata:
  name: payment-service-policy
  namespace: production
spec:
  selector:
    matchLabels:
      app: payment-service
  action: ALLOW
  rules:
  - from:
    - source:
        principals:
        - "cluster.local/ns/production/sa/api-service"
    to:
    - operation:
        methods: ["POST"]
        paths: ["/charge", "/refund", "/receipt"]
```

**What this does:**
- Selects pods with label `app: payment-service`
- Only allows requests from the `api-service` service account
- Only allows POST requests to specific paths
- All other requests are denied (default deny when AuthorizationPolicy exists)

### 3. Deny All Other Connections to payment-service

```yaml
apiVersion: security.istio.io/v1beta1
kind: AuthorizationPolicy
metadata:
  name: payment-service-deny-all
  namespace: production
spec:
  selector:
    matchLabels:
      app: payment-service
  action: DENY
  rules:
  - from:
    - source:
        notPrincipals:
        - "cluster.local/ns/production/sa/api-service"
```

**Alternative approach (recommended):** The ALLOW policy above implicitly
denies all other traffic when any AuthorizationPolicy targets a workload.
You do not need an explicit DENY rule. The ALLOW policy alone is sufficient.

### Complete Policy Set

```yaml
# 1. Enforce mTLS namespace-wide
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: default
  namespace: production
spec:
  mtls:
    mode: STRICT
---
# 2. Allow api-service → payment-service (implicit deny all others)
apiVersion: security.istio.io/v1beta1
kind: AuthorizationPolicy
metadata:
  name: allow-api-to-payment
  namespace: production
spec:
  selector:
    matchLabels:
      app: payment-service
  action: ALLOW
  rules:
  - from:
    - source:
        principals:
        - "cluster.local/ns/production/sa/api-service"
    to:
    - operation:
        methods: ["POST"]
        paths: ["/charge", "/refund", "/receipt"]
```

## Part C: Certificate Lifecycle

### 1. Who Issues Certificates?

The **control plane** acts as a Certificate Authority (CA):

```
Istio: istiod (built-in CA, or integration with external CA like Vault)
Linkerd: linkerd-identity (built-in CA)

Certificate chain:
  Root CA (self-signed, stored in Kubernetes secret)
    └── Intermediate CA (issued by Root CA)
          └── Workload certificates (issued by Intermediate CA, one per pod)
```

Each pod gets a unique certificate with:
- SPIFFE identity: `spiffe://cluster.local/ns/production/sa/api-service`
- SAN (Subject Alternative Name) matching the service account
- Short validity period (default: 24 hours)

### 2. Certificate Rotation

```
Certificate lifecycle:
  T=0h:    Certificate issued (valid for 24h)
  T=12h:   Automatic rotation triggered (50% of lifetime)
  T=12h:   New certificate issued, sidecar hot-reloads
  T=24h:   Old certificate expires (but already rotated)

Rotation is transparent:
  - No connection drops
  - No application restart
  - Sidecar proxy handles rotation via SDS (Secret Discovery Service)
```

### 3. What Happens When a Certificate Expires?

If rotation fails and a certificate expires:
- The sidecar proxy rejects new mTLS handshakes
- Existing connections continue until they close
- New connections from other services fail with TLS errors
- The pod's health check (if using mTLS) may also fail
- Kubernetes restarts the pod, which gets a new certificate

**This is why automatic rotation is critical.** Manual certificate
management at scale (500 pods, rotating daily) is impossible.

### 4. Handling Compromised Certificates

```
Compromised certificate response:

1. Identify the compromised pod
   kubectl get pods -n production -l app=compromised-service

2. Delete the pod (forces new certificate)
   kubectl delete pod compromised-service-xyz -n production

3. New pod gets fresh certificate from istiod
   - New private key generated
   - New certificate signed by CA
   - Old certificate is now invalid (different key)

4. If the CA itself is compromised:
   - Rotate the root CA (generate new root)
   - All pods get new certificates automatically
   - This is a more complex operation (CA rotation)
```

### Common Mistakes to Avoid

- **Using PERMISSIVE mode in production.** PERMISSIVE allows both mTLS and
  plaintext. An attacker can bypass mTLS by sending plaintext. Use STRICT
  in production.
- **Not monitoring certificate rotation failures.** If istiod cannot issue
  certificates (disk full, CA expiry), all mTLS connections will eventually
  fail. Monitor istiod health and certificate expiry.
- **Forgetting about external traffic.** mTLS only works within the mesh.
  External traffic (from outside the cluster) uses regular TLS or plaintext.
  Use an ingress gateway for external TLS termination.
- **Using long-lived certificates.** Short-lived certificates (24h) limit
  the window of exposure if a certificate is compromised. Do not increase
  the lifetime for convenience.

## Key Takeaway

mTLS in a service mesh provides automatic encryption and mutual authentication.
The control plane issues short-lived certificates, rotates them transparently,
and revokes them by deleting pods. Authorization policies control which
services can communicate, implementing zero-trust networking at the
infrastructure level. The key benefit is that security is automatic and
consistent -- developers do not write any TLS code.
