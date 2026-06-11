# Exercise 02: mTLS Configuration

**Type:** Guided
**Time:** 30 min
**Difficulty:** Easy-Medium

## Objective

Understand mutual TLS (mTLS) in a service mesh context and configure mTLS policies for service-to-service communication.

## Scenario

You are securing a microservices platform with three services:

```
Services:
├── frontend (serves web UI, calls api-service)
├── api-service (handles API calls, calls payment-service)
└── payment-service (processes payments, sensitive data)

Security requirements:
- All service-to-service communication must be encrypted
- Services must verify each other's identity (mutual authentication)
- Payment-service only accepts connections from api-service
- Frontend should NOT be able to call payment-service directly
```

## Tasks

### Part A: TLS vs mTLS

Explain the difference between TLS and mTLS:

| Aspect | TLS | mTLS |
|--------|-----|------|
| Who is authenticated? | | |
| Certificate required on | | |
| Use case | | |
| Handshake steps | | |

<details>
<summary>Hint</summary>

TLS: server presents certificate, client verifies server identity. mTLS: both server AND client present certificates, both verify each other. In a service mesh, the control plane automatically provisions and rotates certificates for all services.

</details>

### Part B: Istio mTLS Configuration

Write Istio PeerAuthentication and AuthorizationPolicy resources for the scenario:

1. Enable strict mTLS for the entire namespace
2. Allow only `api-service` to call `payment-service`
3. Deny all other connections to `payment-service`

```yaml
# Your Istio configuration here
```

<details>
<summary>Hint</summary>

Use `PeerAuthentication` with `STRICT` mode to enforce mTLS. Use `AuthorizationPolicy` to define which services can call which. The `from` field specifies the source; the `to` field specifies the destination.

</details>

### Part C: Certificate Lifecycle

In a service mesh with automatic mTLS:

1. Who issues the certificates?
2. How often are they rotated?
3. What happens when a certificate expires?
4. How does the mesh handle a compromised certificate?

<details>
<summary>Hint</summary>

The control plane (Istio's istiod or Linkerd's identity service) acts as a certificate authority. It issues short-lived certificates (typically 24 hours) and automatically rotates them. If a certificate is compromised, revoking it requires removing the pod (the mesh does not support CRL/OCSP in most implementations).

</details>

## Success Criteria

- [ ] You can explain the difference between TLS and mTLS
- [ ] You can write Istio mTLS and authorization policies
- [ ] You understand automatic certificate provisioning and rotation
- [ ] You can design mTLS policies for a multi-service architecture

## What You Should Understand After This Exercise

mTLS provides both encryption and mutual authentication. In a service mesh, mTLS is automatic: the control plane issues short-lived certificates to each service, and sidecar proxies handle the TLS handshake. Authorization policies control which services can communicate, implementing the principle of least privilege at the network level.
