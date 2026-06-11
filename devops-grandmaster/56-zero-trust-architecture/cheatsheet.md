# Cheatsheet: Zero Trust Architecture

## Core Principles
```
Never trust, always verify
Assume breach
Least privilege access
Verify explicitly
```

## Implementation Layers

| Layer | Technology | Purpose |
|-------|-----------|---------|
| Identity | IAM, SSO, MFA | Who are you? |
| Device | MDM, certificates | Is device trusted? |
| Network | mTLS, service mesh | Encrypted communication |
| Application | RBAC, authorization | What can you access? |
| Data | Encryption, DLP | Protect data |

## mTLS (Mutual TLS)
```yaml
# Istio mTLS
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: default
  namespace: istio-system
spec:
  mtls:
    mode: STRICT
```

## Service Mesh for Zero Trust
```yaml
# Istio AuthorizationPolicy
apiVersion: security.istio.io/v1beta1
kind: AuthorizationPolicy
metadata:
  name: api-policy
spec:
  selector:
    matchLabels:
      app: api
  rules:
    - from:
        - source:
            principals: ["cluster.local/ns/default/sa/frontend"]
      to:
        - operation:
            methods: ["GET"]
            paths: ["/api/*"]
```

## BeyondCorp Model
```
1. Device inventory and trust
2. Identity and access management
3. Access proxy (no VPN)
4. Continuous monitoring
5. Dynamic access policies
```
