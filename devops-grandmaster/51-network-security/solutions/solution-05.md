# Solution 05: From Flat Network to Zero-Trust Segmentation

## Part A: Kubernetes NetworkPolicies

### Default Deny (All Namespaces)

```yaml
# 00-default-deny.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-all
  namespace: finledger
spec:
  podSelector: {}
  policyTypes:
    - Ingress
    - Egress
---
# Allow DNS resolution for all pods
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-dns
  namespace: finledger
spec:
  podSelector: {}
  policyTypes:
    - Egress
  egress:
    - to:
        - namespaceSelector: {}
          podSelector:
            matchLabels:
              k8s-app: kube-dns
      ports:
        - protocol: UDP
          port: 53
        - protocol: TCP
          port: 53
```

### Frontend Policy

```yaml
# 01-frontend.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: frontend-policy
  namespace: finledger
spec:
  podSelector:
    matchLabels:
      app: frontend
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - namespaceSelector:
            matchLabels:
              kubernetes.io/metadata.name: ingress-nginx
      ports:
        - protocol: TCP
          port: 80
  egress:
    - to:
        - podSelector:
            matchLabels:
              app: api-gateway
      ports:
        - protocol: TCP
          port: 8000
```

### API Gateway Policy

```yaml
# 02-api-gateway.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: api-gateway-policy
  namespace: finledger
spec:
  podSelector:
    matchLabels:
      app: api-gateway
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - podSelector:
            matchLabels:
              app: frontend
      ports:
        - protocol: TCP
          port: 8000
  egress:
    - to:
        - podSelector:
            matchLabels:
              app: account-service
      ports:
        - protocol: TCP
          port: 8001
    - to:
        - podSelector:
            matchLabels:
              app: transaction-service
      ports:
        - protocol: TCP
          port: 8002
    - to:
        - podSelector:
            matchLabels:
              app: notification-service
      ports:
        - protocol: TCP
          port: 8003
```

### Account Service Policy

```yaml
# 03-account-service.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: account-service-policy
  namespace: finledger
spec:
  podSelector:
    matchLabels:
      app: account-service
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - podSelector:
            matchLabels:
              app: api-gateway
        - podSelector:
            matchLabels:
              app: transaction-service
      ports:
        - protocol: TCP
          port: 8001
  egress:
    - to:
        - podSelector:
            matchLabels:
              app: postgresql
      ports:
        - protocol: TCP
          port: 5432
    - to:
        - podSelector:
            matchLabels:
              app: redis
      ports:
        - protocol: TCP
          port: 6379
```

### Transaction Service Policy

```yaml
# 04-transaction-service.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: transaction-service-policy
  namespace: finledger
spec:
  podSelector:
    matchLabels:
      app: transaction-service
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - podSelector:
            matchLabels:
              app: api-gateway
      ports:
        - protocol: TCP
          port: 8002
  egress:
    - to:
        - podSelector:
            matchLabels:
              app: account-service
      ports:
        - protocol: TCP
          port: 8001
    - to:
        - podSelector:
            matchLabels:
              app: postgresql
      ports:
        - protocol: TCP
          port: 5432
    - to:
        - podSelector:
            matchLabels:
              app: vault
      ports:
        - protocol: TCP
          port: 8200
```

### Notification Service Policy

```yaml
# 05-notification-service.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: notification-service-policy
  namespace: finledger
spec:
  podSelector:
    matchLabels:
      app: notification-service
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - podSelector:
            matchLabels:
              app: api-gateway
        - podSelector:
            matchLabels:
              app: transaction-service
      ports:
        - protocol: TCP
          port: 8003
  egress:
    - to:
        - podSelector:
            matchLabels:
              app: redis
      ports:
        - protocol: TCP
          port: 6379
```

### Data Tier Policies (PostgreSQL, Redis)

```yaml
# 06-postgresql.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: postgresql-policy
  namespace: finledger
spec:
  podSelector:
    matchLabels:
      app: postgresql
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - podSelector:
            matchLabels:
              app: account-service
        - podSelector:
            matchLabels:
              app: transaction-service
      ports:
        - protocol: TCP
          port: 5432
  egress: []  # Database should not initiate outbound connections
---
# 07-redis.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: redis-policy
  namespace: finledger
spec:
  podSelector:
    matchLabels:
      app: redis
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - podSelector:
            matchLabels:
              app: account-service
        - podSelector:
            matchLabels:
              app: notification-service
      ports:
        - protocol: TCP
          port: 6379
  egress: []
```

### Infrastructure Policies (Vault, Prometheus, Grafana)

```yaml
# 08-vault.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: vault-policy
  namespace: finledger
spec:
  podSelector:
    matchLabels:
      app: vault
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - podSelector:
            matchLabels:
              app: transaction-service
        - podSelector:
            matchLabels:
              app: account-service
      ports:
        - protocol: TCP
          port: 8200
  egress:
    - to:
        - podSelector:
            matchLabels:
              app: postgresql
      ports:
        - protocol: TCP
          port: 5432
---
# 09-prometheus.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: prometheus-policy
  namespace: finledger
spec:
  podSelector:
    matchLabels:
      app: prometheus
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - podSelector:
            matchLabels:
              app: grafana
      ports:
        - protocol: TCP
          port: 9090
  egress:
    - to:
        - podSelector: {}  # Can scrape all pods
      ports:
        - protocol: TCP
          port: 9090
        - protocol: TCP
          port: 15090  # Envoy stats
---
# 10-grafana.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: grafana-policy
  namespace: finledger
spec:
  podSelector:
    matchLabels:
      app: grafana
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - namespaceSelector:
            matchLabels:
              kubernetes.io/metadata.name: ingress-nginx
      ports:
        - protocol: TCP
          port: 3000
  egress:
    - to:
        - podSelector:
            matchLabels:
              app: prometheus
      ports:
        - protocol: TCP
          port: 9090
    - to:
        - podSelector:
            matchLabels:
              app: postgresql
      ports:
        - protocol: TCP
          port: 5432
```

### Why This Works

The NetworkPolicies implement three layers of defense:

1. **Default deny (00):** No pod can send or receive traffic unless an explicit allow rule exists. This is the foundation.
2. **Service-specific allows (01-10):** Each service has ingress rules specifying exactly which pods can call it, and egress rules specifying exactly which pods it can call.
3. **Data tier isolation (06-07):** PostgreSQL and Redis have no egress rules -- they do not initiate outbound connections. They only accept traffic from specific application services.

The communication matrix enforced by these policies:

```
Frontend      -> API Gateway (8000)
API Gateway   -> Account (8001), Transaction (8002), Notification (8003)
Account       -> PostgreSQL (5432), Redis (6379)
Transaction   -> Account (8001), PostgreSQL (5432), Vault (8200)
Notification  -> Redis (6379)
Vault         -> PostgreSQL (5432)
Prometheus    -> All pods (scrape metrics)
Grafana       -> Prometheus (9090), PostgreSQL (5432)
```

### Common Mistakes

- **Forgetting DNS egress.** Without the allow-dns rule, no pod can resolve service names, breaking all communication.
- **Using only podSelector without namespaceSelector.** The ingress controller is in a different namespace and needs namespaceSelector.
- **Not applying policies to the data tier.** PostgreSQL must have ingress rules restricting which services can connect.
- **Allowing all egress from application pods.** Application pods should only reach the specific services they need.

---

## Part B: Service Mesh with mTLS

### PeerAuthentication (Cluster-Wide mTLS)

```yaml
# peer-authentication.yaml
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: default
  namespace: finledger
spec:
  mtls:
    mode: STRICT
```

### AuthorizationPolicies

```yaml
# 00-default-deny.yaml
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: deny-all
  namespace: finledger
spec:
  {}  # Deny all traffic in namespace
---
# 01-ingress-to-frontend.yaml
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: ingress-to-frontend
  namespace: finledger
spec:
  selector:
    matchLabels:
      app: frontend
  action: ALLOW
  rules:
    - from:
        - source:
            principals: ["cluster.local/ns/istio-system/sa/istio-ingressgateway-service-account"]
      to:
        - operation:
            methods: ["GET"]
            paths: ["/*"]
---
# 02-frontend-to-api-gateway.yaml
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: frontend-to-api-gateway
  namespace: finledger
spec:
  selector:
    matchLabels:
      app: api-gateway
  action: ALLOW
  rules:
    - from:
        - source:
            principals: ["cluster.local/ns/finledger/sa/frontend-sa"]
      to:
        - operation:
            methods: ["GET", "POST"]
            paths: ["/api/v1/*"]
---
# 03-api-gateway-to-account.yaml
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: api-gateway-to-account
  namespace: finledger
spec:
  selector:
    matchLabels:
      app: account-service
  action: ALLOW
  rules:
    - from:
        - source:
            principals:
              - "cluster.local/ns/finledger/sa/api-gateway-sa"
              - "cluster.local/ns/finledger/sa/transaction-service-sa"
      to:
        - operation:
            methods: ["GET", "POST", "PUT"]
            paths: ["/accounts/*"]
---
# 04-api-gateway-to-transaction.yaml
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: api-gateway-to-transaction
  namespace: finledger
spec:
  selector:
    matchLabels:
      app: transaction-service
  action: ALLOW
  rules:
    - from:
        - source:
            principals: ["cluster.local/ns/finledger/sa/api-gateway-sa"]
      to:
        - operation:
            methods: ["GET", "POST"]
            paths: ["/transactions/*"]
---
# 05-transaction-to-vault.yaml
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: transaction-to-vault
  namespace: finledger
spec:
  selector:
    matchLabels:
      app: vault
  action: ALLOW
  rules:
    - from:
        - source:
            principals: ["cluster.local/ns/finledger/sa/transaction-service-sa"]
      to:
        - operation:
            methods: ["GET"]
            paths: ["/v1/secret/*", "/v1/database/*"]
---
# 06-monitoring-scrape.yaml
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: prometheus-scrape
  namespace: finledger
spec:
  selector: {}  # Apply to all pods
  action: ALLOW
  rules:
    - from:
        - source:
            principals: ["cluster.local/ns/monitoring/sa/prometheus-sa"]
      to:
        - operation:
            methods: ["GET"]
            paths: ["/metrics", "/stats/prometheus"]
```

### Traffic Management

```yaml
# circuit-breaker.yaml
apiVersion: networking.istio.io/v1alpha3
kind: DestinationRule
metadata:
  name: account-service-circuit-breaker
  namespace: finledger
spec:
  host: account-service
  trafficPolicy:
    connectionPool:
      tcp:
        maxConnections: 100
      http:
        h2UpgradePolicy: DEFAULT
        http1MaxPendingRequests: 100
        http2MaxRequests: 1000
        maxRequestsPerConnection: 10
        maxRetries: 3
    outlierDetection:
      consecutive5xxErrors: 5
      interval: 30s
      baseEjectionTime: 30s
      maxEjectionPercent: 50
---
# timeouts.yaml
apiVersion: networking.istio.io/v1alpha3
kind: VirtualService
metadata:
  name: account-service-timeout
  namespace: finledger
spec:
  hosts:
    - account-service
  http:
    - timeout: 10s
      retries:
        attempts: 3
        perTryTimeout: 3s
        retryOn: 5xx,reset,connect-failure
```

### Why This Works

The service mesh configuration provides:

1. **STRICT mTLS:** Every connection between services is encrypted and mutually authenticated. A compromised pod cannot impersonate another service because it does not have that service's private key.

2. **Identity-based authorization:** AuthorizationPolicies use mTLS service account principals, not IP addresses. This is cryptographically verified and cannot be spoofed.

3. **Path and method restrictions:** Even if the API Gateway is authorized to reach the account service, it can only use specific HTTP methods on specific paths. It cannot DELETE or access admin endpoints.

4. **Circuit breaking:** If the account service starts returning errors, the circuit breaker ejects it from the load balancing pool, preventing cascade failures.

5. **Observability:** Istio automatically generates metrics, traces, and access logs for all traffic flowing through the mesh.

### Common Mistakes

- **Using PERMISSIVE mode in production.** PERMISSIVE allows both mTLS and plaintext. Use STRICT to enforce encryption.
- **Not creating service accounts for each service.** AuthorizationPolicies reference service account principals. Each service needs its own SA.
- **Forgetting monitoring access.** Prometheus needs to scrape `/metrics` from all pods. Add an explicit AuthorizationPolicy for this.
- **Not testing policy changes.** Use `istioctl analyze` to validate policies before applying.

---

## Part C: WireGuard VPN Access

### Server Configuration (Bastion Host)

```ini
# /etc/wireguard/wg0.conf
[Interface]
Address = 10.100.0.1/24
ListenPort = 51820
PrivateKey = <server-private-key>

# Enable IP forwarding and NAT
PostUp = iptables -A FORWARD -i wg0 -j ACCEPT; iptables -A FORWARD -o wg0 -j ACCEPT; iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE
PostDown = iptables -D FORWARD -i wg0 -j ACCEPT; iptables -D FORWARD -o wg0 -j ACCEPT; iptables -t nat -D POSTROUTING -o eth0 -j MASQUERADE

# === DevOps Engineers ===
[Peer]
# Team: devops
PublicKey = <devops-peer-public-key>
AllowedIPs = 10.100.0.2/32
# Routes: management subnet only (Grafana, Prometheus)
# Firewall rules applied via iptables PostUp

[Peer]
# Team: dba
PublicKey = <dba-peer-public-key>
AllowedIPs = 10.100.0.3/32
# Routes: data tier (PostgreSQL) + management subnet

[Peer]
# Team: developer
PublicKey = <developer-peer-public-key>
AllowedIPs = 10.100.0.4/32
# Routes: Kubernetes API server only
```

### Firewall Rules on Bastion (Per-Peer Access Control)

```bash
#!/bin/bash
# /etc/wireguard/firewall-rules.sh
# Called by PostUp in wg0.conf

# DevOps (10.100.0.2) -> Grafana (3000), Prometheus (9090)
iptables -A FORWARD -i wg0 -s 10.100.0.2 -d 10.0.30.15 -p tcp --dport 3000 -j ACCEPT
iptables -A FORWARD -i wg0 -s 10.100.0.2 -d 10.0.30.20 -p tcp --dport 9090 -j ACCEPT

# DBA (10.100.0.3) -> PostgreSQL (5432) + management
iptables -A FORWARD -i wg0 -s 10.100.0.3 -d 10.0.20.10 -p tcp --dport 5432 -j ACCEPT
iptables -A FORWARD -i wg0 -s 10.100.0.3 -d 10.0.30.15 -p tcp --dport 3000 -j ACCEPT

# Developer (10.100.0.4) -> Kubernetes API (6443)
iptables -A FORWARD -i wg0 -s 10.100.0.4 -d 10.0.10.1 -p tcp --dport 6443 -j ACCEPT

# Deny all other VPN-forwarded traffic
iptables -A FORWARD -i wg0 -j DROP
```

### Client Configuration Template (DevOps Engineer)

```ini
# devops-engineer.conf
[Interface]
PrivateKey = <client-private-key>
Address = 10.100.0.2/24
DNS = 10.100.0.1

[Peer]
PublicKey = <server-public-key>
Endpoint = vpn.finledger.io:51820
AllowedIPs = 10.0.30.0/24, 10.100.0.0/24
PersistentKeepalive = 25
```

### DNS Resolution Through VPN

```
# /etc/dnsmasq.d/finledger-internal.conf (on bastion)
# Resolve internal service names through VPN
address=/grafana.internal.finledger.io/10.0.30.15
address=/prometheus.internal.finledger.io/10.0.30.20
address=/vault.internal.finledger.io/10.0.30.25
address=/k8s-api.internal.finledger.io/10.0.10.1
```

### Why This Works

The WireGuard server on the bastion host provides three tiers of access:

1. **DevOps engineers** can reach Grafana and Prometheus dashboards but cannot access databases or the Kubernetes API directly.
2. **DBAs** can reach PostgreSQL for emergency access and Grafana for monitoring, but cannot deploy code or access the Kubernetes API.
3. **Developers** can access the Kubernetes API server for `kubectl` commands but cannot access databases or monitoring dashboards.

The `AllowedIPs` directive on each client determines which traffic routes through the VPN. The iptables rules on the bastion enforce fine-grained access control per peer IP. Split DNS ensures that internal service names resolve only when connected to the VPN.

### Common Mistakes

- **Giving all VPN users the same AllowedIPs.** This defeats the purpose of tiered access.
- **Not using PersistentKeepalive.** NAT devices drop idle connections. Keepalives maintain the tunnel.
- **Storing private keys in version control.** WireGuard configs contain private keys and must be distributed securely.
- **Not rotating keys.** Compromised keys grant persistent access. Rotate every 90 days.

---

## Part D: Vault Integration for Secrets

### Kubernetes Auth Configuration

```bash
# Enable Kubernetes auth method
vault auth enable kubernetes
vault write auth/kubernetes/config \
    kubernetes_host="https://kubernetes.default.svc:443"

# Create policies for each service
vault policy write account-service - <<EOF
path "secret/data/finledger/account-service/*" {
  capabilities = ["read"]
}
path "database/creds/finledger-accounts" {
  capabilities = ["read"]
}
EOF

vault policy write transaction-service - <<EOF
path "secret/data/finledger/transaction-service/*" {
  capabilities = ["read"]
}
path "database/creds/finledger-transactions" {
  capabilities = ["read"]
}
path "transit/encrypt/finledger-encryption" {
  capabilities = ["update"]
}
path "transit/decrypt/finledger-encryption" {
  capabilities = ["update"]
}
EOF

# Create Kubernetes auth roles
vault write auth/kubernetes/role/account-service \
    bound_service_account_names=account-service-sa \
    bound_service_account_namespaces=finledger \
    policies=account-service \
    ttl=1h

vault write auth/kubernetes/role/transaction-service \
    bound_service_account_names=transaction-service-sa \
    bound_service_account_namespaces=finledger \
    policies=transaction-service \
    ttl=1h
```

### Dynamic Database Credentials

```bash
# Enable database secrets engine
vault secrets enable database

vault write database/config/finledger-postgres \
    plugin_name=postgresql-database-plugin \
    connection_url="postgresql://{{username}}:{{password}}@postgresql:5432/finledger?sslmode=verify-full" \
    allowed_roles="finledger-accounts,finledger-transactions" \
    username="vault-admin" \
    password="<root-password>"

vault write database/roles/finledger-accounts \
    db_name=finledger-postgres \
    creation_statements="CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}'; GRANT SELECT, INSERT, UPDATE ON accounts TO \"{{name}}\";" \
    default_ttl="1h" \
    max_ttl="24h"

vault write database/roles/finledger-transactions \
    db_name=finledger-postgres \
    creation_statements="CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}'; GRANT SELECT, INSERT ON transactions TO \"{{name}}\";" \
    default_ttl="1h" \
    max_ttl="24h"
```

### Deployment with Vault Injection

```yaml
# account-service-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: account-service
  namespace: finledger
spec:
  replicas: 3
  selector:
    matchLabels:
      app: account-service
  template:
    metadata:
      labels:
        app: account-service
      annotations:
        vault.hashicorp.com/agent-inject: "true"
        vault.hashicorp.com/role: "account-service"
        vault.hashicorp.com/agent-inject-secret-db-creds: "database/creds/finledger-accounts"
        vault.hashicorp.com/agent-inject-template-db-creds: |
          {{- with secret "database/creds/finledger-accounts" -}}
          DB_HOST=postgresql
          DB_PORT=5432
          DB_NAME=finledger
          DB_USER={{ .Data.username }}
          DB_PASSWORD={{ .Data.password }}
          {{- end }}
    spec:
      serviceAccountName: account-service-sa
      containers:
        - name: account-service
          image: finledger/account-service:1.0.0
          command: ["/bin/sh", "-c"]
          args:
            - source /vault/secrets/db-creds && ./account-service
          ports:
            - containerPort: 8001
          securityContext:
            readOnlyRootFilesystem: true
            runAsNonRoot: true
            runAsUser: 1000
            allowPrivilegeEscalation: false
          resources:
            limits:
              cpu: "500m"
              memory: "256Mi"
            requests:
              cpu: "250m"
              memory: "128Mi"
```

### Why This Works

The Vault integration eliminates all static secrets from the deployment:

1. **Kubernetes auth:** Pods authenticate to Vault using their Kubernetes service account token. No static Vault tokens needed.
2. **Scoped policies:** Each service can only read its own secrets path. The account service cannot read transaction service secrets.
3. **Dynamic credentials:** Database credentials are generated on-demand with a 1-hour TTL. After expiration, the credentials are automatically revoked. No long-lived passwords to manage.
4. **Sidecar injection:** The Vault Agent sidecar authenticates to Vault, retrieves secrets, and writes them to a shared volume. The application reads secrets from a file, not environment variables.
5. **Audit logging:** Every secret access is logged in Vault's audit log, providing a complete trail of which service accessed what secret and when.

### Common Mistakes

- **Using environment variables for secrets.** Environment variables are visible in `/proc`, `docker inspect`, and crash logs.
- **Not rotating the Vault root token.** The root token should be unsealed and rotated immediately after initial setup.
- **Setting TTLs too long.** Shorter TTLs (1 hour) limit the window of exposure if credentials are compromised.
- **Not backing up Vault.** Vault's storage backend must be backed up. Loss of Vault means loss of all secrets.

---

## Part E: Complete Architecture Diagram and ADR

### Architecture Diagram

```
                          INTERNET
                             |
                   +====================+
                   |  Ingress Gateway   |  External TLS termination
                   |  (Istio Ingress)   |  JWT validation
                   +====================+
                             |
                   +====================+
                   |   Service Mesh     |  mTLS between all services
                   |   (Istio/Envoy)    |  AuthorizationPolicies
                   +====================+
                             |
            +----------------+----------------+
            |                                 |
   +========+========+             +=========+=========+
   |  Public Tier     |             |  Management Tier  |
   |  (Namespace:     |             |  (Namespace:      |
   |   finledger)     |             |   monitoring)     |
   |                  |             |                   |
   |  +-- frontend    |             |  +-- prometheus   |
   |  +-- api-gateway |             |  +-- grafana      |
   |  +-- account-svc |             +=========+=========+
   |  +-- txn-svc     |                       |
   |  +-- notif-svc   |             +=========+=========+
   |                  |             |  WireGuard VPN    |
   +========+=========+             |  (Bastion Host)   |
            |                      |                   |
            |                      |  DevOps -> Grafana|
   +========+=========+            |  DBA -> PostgreSQL|
   |  Data Tier       |            |  Dev -> K8s API   |
   |                  |            +===================+
   |  +-- postgresql  |
   |  +-- redis       |
   |  +-- vault       |
   +==================+

   Each service has:
   - Vault Agent sidecar (dynamic credentials)
   - Envoy sidecar (mTLS, metrics, tracing)
   - OPA sidecar (fine-grained authorization)
   - Falco monitoring (runtime anomaly detection)

   Network Policies enforce:
   - Default deny ingress/egress
   - Per-service allow rules by label selector
   - Data tier has no egress
   - DNS allowed for all pods
```

### Architecture Decision Record (ADR)

**Title:** Zero-Trust Network Architecture for FinLedger

**Status:** Accepted

**Context:** FinLedger processes sensitive financial data and must comply with PCI DSS and SOC2. The current flat network architecture allows any pod to communicate with any other pod, creating unacceptable risk of lateral movement in case of compromise.

**Decision:**

1. **Istio over Linkerd:** We chose Istio because it provides both mTLS and fine-grained AuthorizationPolicies out of the box. Linkerd is simpler but lacks AuthorizationPolicy. The operational complexity of Istio is justified by the regulatory requirements.

2. **WireGuard over OpenVPN:** WireGuard has a smaller attack surface (~4,000 lines vs ~100,000), better performance (kernel-space), and simpler configuration. It meets our requirements for remote access without the complexity of OpenVPN.

3. **HashiCorp Vault over Kubernetes Secrets:** Kubernetes Secrets are base64-encoded (not encrypted) by default. Vault provides dynamic credentials, automatic rotation, audit logging, and fine-grained access policies. The operational overhead is justified by the security posture.

4. **Network Policies as defense in depth:** Even with Istio AuthorizationPolicies, we implement Kubernetes NetworkPolicies as an independent layer. If Istio has a bug, NetworkPolicies still enforce segmentation.

**Trade-offs:**

- **Performance cost of mTLS:** mTLS adds ~1-2ms latency per request and increases CPU usage by ~5%. This is acceptable for financial transactions.
- **Operational complexity:** The mesh adds ~15 sidecar containers to the cluster, increasing resource consumption by ~20%. The team needs training on Istio debugging.
- **VPN management overhead:** WireGuard requires manual key distribution and rotation. We mitigate this with automated key rotation scripts.

**Threats Mitigated:**

- Lateral movement after pod compromise (NetworkPolicies + AuthorizationPolicies)
- Man-in-the-middle attacks between services (mTLS)
- Credential theft from environment variables (Vault injection)
- Unauthorized service-to-service communication (AuthorizationPolicies)
- Undetected compromise (Falco + Prometheus + Jaeger)

### Why This Works

Zero-trust networking is implemented through multiple independent layers, each compensating for the weaknesses of the others. Network policies control traffic at the IP level. Service mesh controls traffic at the application level with identity verification. VPN controls remote access with role-based restrictions. Vault controls secret access with scoped policies and automatic rotation. No single layer is sufficient, but together they make lateral movement extremely difficult and highly visible.

### Common Mistakes

- **Treating zero-trust as a product.** Zero-trust is a design philosophy, not a product you install. It requires changes at every layer.
- **Not monitoring the mesh.** The service mesh generates rich observability data. If you are not monitoring it, you are missing the primary benefit.
- **Over-complicating the initial deployment.** Start with default-deny and add allow rules incrementally. Do not try to deploy all policies at once.
- **Forgetting about operational runbooks.** Zero-trust changes how you debug connectivity issues. Update your runbooks to include `istioctl` commands.

---

## Common Mistakes to Avoid

- **Deploying NetworkPolicies without testing.** A wrong policy can break all service communication. Test in a staging namespace first.
- **Not allowing DNS egress.** This is the most common mistake. Every pod needs DNS to discover other services.
- **Using PERMISSIVE mTLS mode.** PERMISSIVE allows plaintext traffic, defeating the purpose. Use STRICT.
- **Forgetting about StatefulSet pod identity.** StatefulSet pods have stable identities. Use pod-specific NetworkPolicies if needed.

## Key Takeaway

Zero-trust networking is not a single product -- it is a design philosophy implemented through multiple layers of control. Network policies control who can talk to whom at the IP level. Service meshes add identity and encryption at the application level. VPNs provide controlled access to internal resources. Vault ensures that even if an attacker compromises a pod, they cannot extract long-lived credentials. Each layer compensates for the weaknesses of the others. The operational cost is real -- more moving parts, more things to debug, more YAML to maintain -- but the security posture is fundamentally stronger than a flat network where every pod trusts every other pod.
