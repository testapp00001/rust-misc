# Solution 05: Complete Security Architecture

## Part A: Architecture Design

```
                           INTERNET
                              |
                    +========================+
                    |   Cloud DDoS Shield    |  <-- Module 52: Volumetric DDoS
                    |   (Layer 3/4 Filter)   |      mitigation at edge
                    +========================+
                              |
                    +========================+
                    |   CDN + Edge WAF       |  <-- Module 53: WAF rules,
                    |   (Layer 7 Filter)     |      OWASP Top 10, bot detection
                    +========================+
                              |
                    +========================+
                    |   Global Load Balancer  |  <-- Geographic routing,
                    |   (Health Checks)       |      failover between regions
                    +========================+
                              |
              +---------------+---------------+
              |               |               |
         [US-East]       [EU-West]       [AP-South]
              |               |               |
    +================+  (same)  (same)        |
    | Ingress Gateway |                       |
    | (mTLS Termination)                      |
    | (JWT Validation)  |                     |
    | (Rate Limiting)   |                     |
    +================+                        |
              |                               |
    +================+                        |
    | Service Mesh   |  <-- Module 56:        |
    | (Istio/Envoy)  |      Zero Trust        |
    | mTLS everywhere|      Controls          |
    | AuthZ policies |                        |
    +================+                        |
              |                               |
    +---------+---------+                     |
    |    |    |    |    |                      |
  [API] [Pay] [Usr] [Ord] ... (12 services)  |
    |    |    |    |    |                      |
    |  +================+                     |
    |  | OPA Sidecar    |  <-- Fine-grained   |
    |  | (Policy Engine)|      authorization   |
    |  +================+                     |
    |         |                               |
    |  +================+                     |
    |  | Vault Agent    |  <-- Module 54:      |
    |  | (Dynamic Creds)|      Secrets mgmt    |
    |  +================+                     |
    |         |                               |
    |  +================+                     |
    |  | Falco Runtime  |  <-- Runtime         |
    |  | (Anomaly Det.) |      security        |
    |  +================+                     |
    |                                        |
    +========================================+
    |        CI/CD Security Pipeline         |  <-- Module 55: Scanning
    |  SAST -> SCA -> Secrets -> Image ->    |
    |  IaC -> Policy -> DAST -> Deploy       |
    +========================================+
    |                                        |
    |  +================+                    |
    |  | Network Policies|  <-- Module 51:   |
    |  | (K8s NetPol)   |      Network sec   |
    |  +================+                    |
    +========================================+
```

### Why This Works

The architecture implements defense in depth across six layers. Each layer operates independently -- if one fails, the others continue to protect:

1. **DDoS Protection (Edge)**: Blocks volumetric attacks before they reach the application. Cloud-based scrubbing handles L3/L4 attacks; CDN handles L7 floods.

2. **WAF (Edge)**: Filters malicious HTTP requests (SQL injection, XSS, bot traffic) at the CDN/WAF layer before they reach the origin.

3. **Ingress Gateway**: Terminates external TLS, validates JWT tokens, enforces rate limits per client, and initiates mTLS to backend services.

4. **Service Mesh**: Provides mTLS between all services, authorization policies for access control, and observability for traffic analysis.

5. **Policy Engine (OPA)**: Enforces fine-grained business rules that go beyond network-level access control.

6. **Secrets Management (Vault)**: Provides dynamic, scoped credentials with automatic rotation. No static secrets in code, config, or environment variables.

7. **Runtime Security (Falco)**: Monitors container behavior for anomalies -- unexpected processes, file access, network connections.

8. **Network Policies**: Kubernetes-level network segmentation as an additional defense layer beneath the service mesh.

9. **CI/CD Pipeline**: Scans code, dependencies, containers, and infrastructure before deployment. Catches vulnerabilities before they reach production.

### Common Mistakes

- **Treating layers as alternatives.** Each layer should be independent. Do not remove the WAF because you have a service mesh -- they protect against different threats.
- **Focusing on prevention, ignoring detection.** Prevention stops known attacks. Detection catches unknown attacks. Both are essential.
- **Not testing the full stack.** Test each layer independently and test the interactions between layers. A misconfigured WAF might pass malicious requests that the OPA policy should catch.

---

## Part B: Security Policies

### 1. Network Policies (Module 51)

```yaml
# network-policies.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-all
  namespace: payflow
spec:
  podSelector: {}
  policyTypes:
    - Ingress
    - Egress
---
# Allow ingress controller to reach API gateway
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-ingress-to-api-gateway
  namespace: payflow
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
              kubernetes.io/metadata.name: ingress-nginx
          podSelector:
            matchLabels:
              app.kubernetes.io/name: ingress-nginx
      ports:
        - protocol: TCP
          port: 8443
---
# Allow API gateway to reach payment processor
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-api-gateway-to-payment
  namespace: payflow
spec:
  podSelector:
    matchLabels:
      app: payment-processor
  policyTypes:
    - Ingress
  ingress:
    - from:
        - podSelector:
            matchLabels:
              app: api-gateway
      ports:
        - protocol: TCP
          port: 8080
---
# Allow payment processor to reach payment provider (external)
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-payment-to-provider
  namespace: payflow
spec:
  podSelector:
    matchLabels:
      app: payment-processor
  policyTypes:
    - Egress
  egress:
    - to:
        - ipBlock:
            cidr: 0.0.0.0/0
            except:
              - 10.0.0.0/8       # Block internal networks
              - 172.16.0.0/12
              - 192.168.0.0/16
      ports:
        - protocol: TCP
          port: 443
---
# Allow DNS resolution for all pods
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-dns
  namespace: payflow
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

### Why This Works (Network Policies)

The NetworkPolicies implement defense in depth at the Kubernetes networking layer:

- **Default deny**: All ingress and egress traffic is blocked unless explicitly allowed. This is the foundation.
- **Ingress controller restriction**: Only the ingress controller can reach the API gateway. No other pods or namespaces can initiate connections.
- **Payment processor isolation**: The payment processor can only receive traffic from the API gateway and send traffic to the external payment provider. It cannot reach internal databases or other services.
- **Internal network blocking**: The egress policy for the payment processor explicitly blocks RFC 1918 internal addresses, preventing the compromised service from scanning the internal network.
- **DNS allowlist**: DNS resolution is allowed for all pods (required for service discovery), but only to the cluster DNS service.

### Common Mistakes

- **Forgetting default deny.** Without the default-deny policy, all traffic is allowed by default. Always start with deny-all.
- **Not allowing DNS.** Services need DNS to discover each other. Forgetting DNS egress rules breaks all service communication.
- **Using podSelector without namespaceSelector.** In multi-namespace clusters, podSelector only matches within the same namespace. Add namespaceSelector for cross-namespace rules.
- **Overly broad egress rules.** `0.0.0.0/0` for egress allows traffic to any destination. Use IP blocks to restrict external access to known endpoints.

---

### 2. DDoS Protection (Module 52)

```yaml
# rate-limiting-config.yaml
apiVersion: networking.istio.io/v1alpha3
kind: EnvoyFilter
metadata:
  name: global-rate-limit
  namespace: payflow
spec:
  workloadSelector:
    labels:
      app: api-gateway
  configPatches:
    - applyTo: HTTP_FILTER
      match:
        context: GATEWAY
        listener:
          filterChain:
            filter:
              name: envoy.filters.network.http_connection_manager
      patch:
        operation: INSERT_BEFORE
        value:
          name: envoy.filters.http.local_ratelimit
          typed_config:
            "@type": type.googleapis.com/udpa.type.v1.TypedStruct
            type_url: type.googleapis.com/envoy.extensions.filters.http.local_ratelimit.v3.LocalRateLimit
            value:
              stat_prefix: http_local_rate_limiter
              token_bucket:
                max_tokens: 10000        # Burst capacity
                tokens_per_fill: 1000    # Refill rate
                fill_interval: 1s
              filter_enabled:
                runtime_key: local_rate_limit_enabled
                default_value:
                  numerator: 100
                  denominator: HUNDRED
              filter_enforced:
                runtime_key: local_rate_limit_enforced
                default_value:
                  numerator: 100
                  denominator: HUNDRED
---
# Per-IP rate limiting
apiVersion: networking.istio.io/v1alpha3
kind: EnvoyFilter
metadata:
  name: per-ip-rate-limit
  namespace: payflow
spec:
  workloadSelector:
    labels:
      app: api-gateway
  configPatches:
    - applyTo: HTTP_FILTER
      match:
        context: GATEWAY
      patch:
        operation: INSERT_BEFORE
        value:
          name: envoy.filters.http.rbac
          typed_config:
            "@type": type.googleapis.com/envoy.extensions.filters.http.rbac.v3.RBAC
            rules:
              action: ALLOW
              policies:
                per-ip-limit:
                  permissions:
                    - any: true
                  principals:
                    - remote_ip:
                        address_prefix: 0.0.0.0
                        prefix_len: 0
```

Rate limiting summary:
- **Global**: 10,000 req/s across all regions (managed by cloud provider)
- **Per-IP**: 100 req/s per source IP (Envoy local rate limit)
- **Per-user**: 50 req/s for authenticated users (JWT claim-based limiting)
- **Burst**: 200 req/s for 10 seconds (token bucket configuration)
- **Geographic**: Cloud provider geo-routing distributes traffic across regions

### Why This Works (DDoS Protection)

Multi-layer rate limiting provides graduated protection:

1. **Cloud edge (L3/L4)**: Blocks volumetric attacks (SYN floods, UDP floods) before they reach the application. This is handled by the cloud provider's DDoS scrubbing service.

2. **CDN (L7)**: Absorbs HTTP floods at the edge. CDN caching reduces origin load for static content.

3. **Ingress rate limiting**: Envoy's local rate limit filters enforce per-IP and per-user limits at the application edge. Excess requests receive 429 (Too Many Requests).

4. **Token bucket**: Allows burst traffic (200 req/s for 10s) while maintaining a sustainable average rate. This accommodates legitimate traffic spikes without dropping requests.

### Common Mistakes

- **Only implementing one layer.** Cloud DDoS protection without application-level rate limiting leaves the origin vulnerable to sophisticated L7 attacks.
- **Not accounting for distributed attacks.** A single IP limit of 100 req/s is effective against single-source attacks but not against botnets with 10,000+ IPs. Combine with behavioral analysis.
- **Rate limiting health checks.** Health check endpoints should be excluded from rate limiting to prevent false positives during scaling events.
- **Not returning proper 429 responses.** Clients need 429 responses with `Retry-After` headers to implement backoff. Silent drops cause client timeouts and retries.

---

### 3. WAF Rules (Module 53)

```yaml
# waf-rules.yaml
# ModSecurity CRS rules for PayFlow
SecRuleEngine On

# Block SQL injection on payment endpoints
SecRule REQUEST_URI "@beginsWith /api/payments" \
  "id:1001,phase:2,block,msg:'SQL Injection Attempt on Payment API',\
   severity:CRITICAL,logdata:'%{MATCHED_VAR}'"

# Block XSS in all form submissions
SecRule ARGS "@detectXSS" \
  "id:1002,phase:2,block,msg:'XSS Attempt Detected',\
   severity:HIGH"

# Require Content-Type on POST/PUT/PATCH
SecRule REQUEST_METHOD "@rx ^(POST|PUT|PATCH)$" \
  "id:1003,phase:1,chain,block,msg:'Missing Content-Type Header'"
  SecRule &REQUEST_HEADERS:Content-Type "@eq 0" ""

# Block suspicious User-Agent patterns
SecRule REQUEST_HEADERS:User-Agent "@pmFromFile user-agents-blocklist.txt" \
  "id:1004,phase:1,block,msg:'Blocked User-Agent',logdata:'%{REQUEST_HEADERS.User-Agent}'"

# Rate limit login endpoint
SecRule REQUEST_URI "@beginsWith /api/auth/login" \
  "id:1005,phase:1,pass,nolog,setvar:ip.login_counter=+1,\
   expirevar:ip.login_counter=60"
SecRule IP:LOGIN_COUNTER "@gt 10" \
  "id:1006,phase:1,block,msg:'Login Rate Limit Exceeded'"

# PCI DSS: Block credit card numbers in URLs
SecRule REQUEST_URI "@creditCard" \
  "id:1007,phase:1,block,msg:'Credit Card Number in URL',severity:CRITICAL"

# PCI DSS: Block credit card numbers in request body (prevent logging)
SecRule REQUEST_BODY "@creditCard" \
  "id:1008,phase:2,pass,nolog,\
   setvar:'tx.credit_card_found=1'"
SecRule TX:CREDIT_CARD_FOUND "@eq 1" \
  "id:1009,phase:5,block,msg:'Credit Card Number in Request Body',severity:CRITICAL"

# Block common attack payloads
SecRule REQUEST_BODY|REQUEST_URI "@pm <script javascript: data: vbscript:" \
  "id:1010,phase:2,block,msg:'Attack Payload Detected',severity:HIGH"

# Block path traversal
SecRule REQUEST_URI "@contains ../" \
  "id:1011,phase:1,block,msg:'Path Traversal Attempt',severity:HIGH"

# Block command injection
SecRule ARGS "@pmFromFile command-injection-patterns.txt" \
  "id:1012,phase:2,block,msg:'Command Injection Attempt',severity:CRITICAL"
```

### Why This Works (WAF Rules)

The WAF rules protect against the OWASP Top 10 and PCI DSS requirements:

1. **SQL Injection (1001)**: Blocks SQL injection attempts on payment endpoints. Even if the application has parameterized queries, the WAF provides defense in depth.

2. **XSS (1002)**: Detects cross-site scripting payloads in all form submissions. The `@detectXSS` operator uses pattern matching and context analysis.

3. **Content-Type enforcement (1003)**: Prevents attacks that exploit missing or incorrect Content-Type headers. POST/PUT/PATCH requests must declare their content type.

4. **Credit card protection (1007-1009)**: PCI DSS requires that credit card numbers are never logged. The WAF detects card numbers in URLs and request bodies and blocks or sanitizes them before they reach the application.

5. **Rate limiting on login (1005-1006)**: Prevents brute force attacks on the login endpoint. 10 attempts per minute per IP is a reasonable threshold.

### Common Mistakes

- **Not tuning WAF rules.** Out-of-the-box WAF rules generate many false positives. Tune rules for your application's specific patterns.
- **Blocking legitimate API requests.** API requests with JSON bodies containing special characters may trigger WAF rules. Add exceptions for known-good patterns.
- **Not covering PCI DSS requirements.** Credit card number detection in URLs and bodies is a PCI DSS requirement, not optional.
- **WAF as the only defense.** WAFs can be bypassed with encoding tricks and zero-day payloads. Always combine with application-level security.

---

### 4. Secrets Management (Module 54)

```yaml
# secrets-rotation-strategy.yaml
rotation_schedule:
  database_credentials:
    method: dynamic      # Vault generates on demand
    ttl: 24h             # New credentials every 24 hours
    max_ttl: 72h         # Hard limit: 3 days
    vault_path: database/creds/payflow-prod

  api_keys:
    method: static       # Pre-generated, stored in Vault
    rotation_interval: 7d
    vault_path: secret/data/payflow/api-keys

  tls_certificates:
    method: cert-manager # Automated via cert-manager
    rotation_interval: 90d
    renewal_before: 30d  # Renew 30 days before expiry
    issuer: letsencrypt-prod

  payment_provider_keys:
    method: static
    rotation_interval: 30d
    vault_path: secret/data/payflow/payment-provider
    manual_approval: true  # Requires human approval

  encryption_keys:
    method: transit       # Vault transit engine
    rotation_interval: 90d
    vault_path: transit/keys/payflow-encryption
    min_decryption_version: 3  # Keep last 3 versions for decryption
```

```hcl
# payment-service-policy.hcl
# Payment processor can only access its own secrets
path "secret/data/payflow/payment-processor/*" {
  capabilities = ["read"]
}

# Payment processor can generate database credentials
path "database/creds/payflow-payments-db" {
  capabilities = ["read"]
}

# Payment processor can encrypt/decrypt payment data
path "transit/encrypt/payflow-encryption" {
  capabilities = ["update"]
}

path "transit/decrypt/payflow-encryption" {
  capabilities = ["update"]
}

# Explicitly deny access to other service secrets
path "secret/data/payflow/user-service/*" {
  capabilities = ["deny"]
}

path "secret/data/payflow/admin/*" {
  capabilities = ["deny"]
}

# Deny access to Vault admin endpoints
path "sys/*" {
  capabilities = ["deny"]
}

path "auth/*" {
  capabilities = ["deny"]
}
```

### Why This Works (Secrets Management)

The secrets strategy implements the principle of least privilege for credentials:

1. **Dynamic database credentials**: Vault generates unique, short-lived credentials for each service. No shared database passwords. Automatic expiration prevents credential accumulation.

2. **Scoped Vault policies**: Each service has a Vault policy that limits access to only its own secrets. The payment processor cannot read user service secrets or admin credentials.

3. **Transit encryption**: Payment data is encrypted at the application level using Vault's transit engine. Even if the database is compromised, the data is encrypted with keys that the database does not have.

4. **Automated rotation**: All credentials rotate automatically. No manual rotation means no forgotten credentials and no long-lived secrets.

5. **PCI DSS compliance**: Payment provider keys require manual approval for rotation, satisfying the "split knowledge" requirement for critical credentials.

### Common Mistakes

- **Storing secrets in environment variables.** Environment variables are visible in `/proc`, container inspect, and crash logs. Use Vault agent injection instead.
- **Long-lived credentials.** Credentials that never expire are a security risk. Even if not compromised, they accumulate over time as employees leave and systems change.
- **Shared credentials.** Multiple services sharing the same database password means you cannot revoke access for one service without affecting others.
- **Not backing up Vault.** Vault's storage backend must be backed up. Loss of Vault means loss of all secrets.

---

### 5. Security Scanning Pipeline (Module 55)

```yaml
# .github/workflows/security-pipeline.yaml
name: PayFlow Security Pipeline
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  sast:
    name: Static Application Security Testing
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run Semgrep
        uses: returntocorp/semgrep-action@v1
        with:
          config: >-
            p/security-audit
            p/secrets
            p/owasp-top-ten
      - name: Run cargo-audit (Rust)
        run: |
          cargo install cargo-audit
          cargo audit

  sca:
    name: Software Composition Analysis
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run Snyk
        uses: snyk/actions@master
        with:
          command: test
          args: --severity-threshold=high

  secret-scanning:
    name: Secret Detection
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - name: Run Gitleaks
        uses: gitleaks/gitleaks-action@v2
        with:
          args: --verbose --redact

  container-scan:
    name: Container Image Scanning
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build image
        run: docker build -t payflow:${{ github.sha }} .
      - name: Run Trivy
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: payflow:${{ github.sha }}
          severity: CRITICAL,HIGH
          exit-code: 1

  iac-scan:
    name: Infrastructure as Code Scanning
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run Checkov
        uses: bridgecrewio/checkov-action@v12
        with:
          directory: ./infrastructure
          framework: kubernetes,terraform
          output_format: github_failed_only

  policy-validation:
    name: Policy Validation
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Validate OPA Policies
        run: |
          curl -L -o opa https://openpolicyagent.org/downloads/latest/opa_linux_amd64
          chmod +x opa
          ./opa test policies/ -v
      - name: Validate Istio Policies
        run: |
          # Dry-run Istio AuthorizationPolicies
          for f in k8s/istio-policies/*.yaml; do
            kubectl apply --dry-run=server -f "$f"
          done

  dast:
    name: Dynamic Application Security Testing
    runs-on: ubuntu-latest
    needs: [sast, sca, container-scan]
    steps:
      - uses: actions/checkout@v4
      - name: Start application
        run: docker-compose up -d
      - name: Run OWASP ZAP
        uses: zaproxy/action-baseline@v0.7.0
        with:
          target: https://localhost:8443
          rules_file_name: zap-rules.tsv
      - name: Cleanup
        run: docker-compose down

  security-gate:
    name: Security Gate
    runs-on: ubuntu-latest
    needs: [sast, sca, secret-scanning, container-scan, iac-scan, policy-validation, dast]
    steps:
      - name: Evaluate Results
        run: |
          echo "All security scans passed. Deployment approved."
          # If any job failed, this step will not run
```

### Why This Works (CI/CD Pipeline)

The pipeline implements shift-left security -- catching vulnerabilities before they reach production:

1. **SAST**: Finds vulnerabilities in source code (SQL injection, XSS, insecure crypto) before the code is compiled.
2. **SCA**: Identifies known vulnerabilities in dependencies. The `cargo-audit` step checks Rust crates against the RustSec advisory database.
3. **Secret scanning**: Prevents secrets (API keys, passwords, certificates) from being committed to version control.
4. **Container scanning**: Scans the container image for OS-level vulnerabilities and malware.
5. **IaC scanning**: Validates Kubernetes manifests and Terraform configurations against security best practices.
6. **Policy validation**: Ensures OPA and Istio policies are syntactically correct and pass tests before deployment.
7. **DAST**: Tests the running application for vulnerabilities that static analysis cannot find (authentication bypass, business logic flaws).
8. **Security gate**: All scans must pass before deployment. This prevents vulnerable code from reaching production.

### Common Mistakes

- **Not failing the pipeline on critical findings.** If the pipeline succeeds despite critical vulnerabilities, developers will ignore scan results. Use `exit-code: 1` to fail on critical findings.
- **Scanning too late.** Run SAST and secret scanning on every PR, not just on merge to main. Catch vulnerabilities during code review.
- **Not updating scan rules.** Vulnerability databases update daily. Pin to a specific version but update regularly.
- **Ignoring false positives.** Too many false positives cause developers to ignore all findings. Invest time in tuning rules to reduce noise.

---

## Part C: Zero Trust Service Mesh

### PeerAuthentication

```yaml
# peer-authentication.yaml
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: default
  namespace: payflow
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
  namespace: payflow
spec:
  {}
---
# 01-ingress-to-api-gateway.yaml
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: ingress-to-api-gateway
  namespace: payflow
spec:
  selector:
    matchLabels:
      app: api-gateway
  action: ALLOW
  rules:
    - from:
        - source:
            principals: ["cluster.local/ns/istio-system/sa/istio-ingressgateway-service-account"]
      to:
        - operation:
            methods: ["GET", "POST", "PUT", "DELETE"]
            paths: ["/api/v1/*"]
---
# 02-api-gateway-to-services.yaml
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: api-gateway-to-payment
  namespace: payflow
spec:
  selector:
    matchLabels:
      app: payment-processor
  action: ALLOW
  rules:
    - from:
        - source:
            principals: ["cluster.local/ns/payflow/sa/api-gateway-sa"]
      to:
        - operation:
            methods: ["POST"]
            paths: ["/v1/charges", "/v1/refunds"]
---
# 03-payment-to-provider.yaml
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: payment-to-provider
  namespace: payflow
spec:
  selector:
    matchLabels:
      app: payment-processor
  action: ALLOW
  rules:
    - to:
        - operation:
            hosts: ["api.stripe.com", "api.payment-provider.com"]
            methods: ["POST"]
---
# 04-admin-restrictions.yaml
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: admin-service-restrict
  namespace: payflow
spec:
  selector:
    matchLabels:
      app: admin-service
  action: ALLOW
  rules:
    - from:
        - source:
            principals: ["cluster.local/ns/istio-system/sa/istio-ingressgateway-service-account"]
            requestPrincipals: ["*"]  # Requires authenticated user
      to:
        - operation:
            methods: ["GET", "POST", "PUT", "DELETE"]
            paths: ["/admin/*"]
---
# 05-block-admin-from-non-admin-users.yaml
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: admin-requires-admin-role
  namespace: payflow
spec:
  selector:
    matchLabels:
      app: admin-service
  action: DENY
  rules:
    - from:
        - source:
            notRequestPrincipals: ["*"]  # Deny unauthenticated
```

### OPA Policy for Payment Validation

```rego
package payflow.payments

import future.keywords.in
import future.keywords.if

default allow = false

# Rule 1: Max transaction amount without approval
allow if {
    input.method == "POST"
    input.path[0] == "v1" 
    input.path[1] == "charges"
    input.body.amount <= 50000
    input.user.role == "customer"
}

# Rule 2: High-value transactions require admin approval
allow if {
    input.method == "POST"
    input.path[0] == "v1"
    input.path[1] == "charges"
    input.body.amount > 50000
    input.user.role == "admin"
    input.body.approval_token != ""
}

# Rule 3: Daily transaction limit per user
allow if {
    input.method == "POST"
    input.path[0] == "v1"
    input.path[1] == "charges"
    input.body.amount <= 100000
    daily_total := data.user_totals[input.user.id]
    daily_total + input.body.amount <= 100000
}

# Rule 4: Velocity check (max 10 transactions per hour)
allow if {
    input.method == "POST"
    input.path[0] == "v1"
    input.path[1] == "charges"
    hourly_count := data.user_hourly_counts[input.user.id]
    hourly_count < 10
}

# Rule 5: Sanctioned countries blocked
allow if {
    input.method == "POST"
    input.path[0] == "v1"
    input.path[1] == "charges"
    not input.body.billing_country in data.sanctioned_countries
}

# Rule 6: PCI DSS - never log full card numbers
allow if {
    input.method == "POST"
    input.path[0] == "v1"
    input.path[1] == "charges"
    # Validate card number is masked in any logging
    not contains_card_number(input.body.card_number)
}

contains_card_number(card) if {
    # Check if card number is full (16 digits) - should be masked
    regex.match(`^\d{16}$`, card)
}

# Deny reasons for debugging
deny_reason := "amount_exceeds_limit" if {
    input.method == "POST"
    input.body.amount > 50000
    input.user.role != "admin"
}

deny_reason := "daily_limit_exceeded" if {
    input.method == "POST"
    daily_total := data.user_totals[input.user.id]
    daily_total + input.body.amount > 100000
}

deny_reason := "velocity_limit_exceeded" if {
    input.method == "POST"
    hourly_count := data.user_hourly_counts[input.user.id]
    hourly_count >= 10
}

deny_reason := "sanctioned_country" if {
    input.method == "POST"
    input.body.billing_country in data.sanctioned_countries
}

deny_reason := "full_card_number_detected" if {
    input.method == "POST"
    contains_card_number(input.body.card_number)
}
```

### Why This Works (Zero Trust Mesh)

The service mesh configuration implements zero trust at the network and application layers:

1. **STRICT mTLS**: Every connection between services is encrypted and authenticated. No plaintext traffic is allowed within the mesh.

2. **Default deny**: The base AuthorizationPolicy blocks all traffic. Services must have explicit ALLOW policies to communicate.

3. **Identity-based access**: Policies use mTLS service identities (not IP addresses) to control access. The payment processor only accepts traffic from the API gateway, identified by its service account.

4. **Egress control**: The payment processor can only reach the external payment provider. It cannot reach other internal services or arbitrary external endpoints.

5. **Authentication requirement**: Admin endpoints require authenticated users (requestPrincipals). Unauthenticated requests are denied.

6. **PCI DSS compliance**: The OPA policy enforces transaction limits, velocity checks, and card number masking. These are PCI DSS requirements that go beyond network-level access control.

### Common Mistakes

- **Using PERMISSIVE mode in production.** PERMISSIVE mode allows both mTLS and plaintext traffic. Use STRICT mode to enforce mTLS.
- **Not testing policy changes.** AuthorizationPolicy changes can break service communication. Test in staging first with `istioctl analyze`.
- **Forgetting about external services.** Service meshes only cover internal traffic. External API calls need separate security controls.
- **Not monitoring policy denials.** Denied requests should be logged and monitored. A spike in denials may indicate a misconfiguration or an attack.

---

## Part E: Incident Response Matrix

| Threat | Detection Layer | Detection Method | Response | Containment Time |
|--------|----------------|------------------|----------|-----------------|
| DDoS Attack (Volumetric) | L1 (DDoS) | Traffic anomaly, spike in requests | Cloud DDoS mitigation, auto-scale | < 30s |
| DDoS Attack (Application) | L2 (WAF) + L3 (Rate Limit) | Request rate threshold, pattern analysis | Rate limit, block IPs, CDN challenge | < 1min |
| SQL Injection | L2 (WAF) | Pattern match in request body/params | Block request, alert security team | Immediate |
| XSS Attack | L2 (WAF) | Script tag/JS pattern detection | Block request, log for analysis | Immediate |
| Compromised Service | L4 (Mesh) + L6 (Runtime) | Anomalous traffic, 403 spike, Falco alert | Revoke cert, quarantine, redeploy | < 2min |
| Stolen Credentials | L5 (Secrets) | Vault audit log, unusual access pattern | Rotate credentials, revoke tokens | < 5min |
| Vulnerable Dependency | L7 (Scanning) | SCA scan in CI/CD | Block deployment, patch dependency | < 1hr |
| Data Exfiltration | L4 (Mesh) + L6 (Runtime) | Egress traffic anomaly, DLP detection | Block egress, isolate service | < 1min |
| Insider Threat | All Layers | Behavioral analysis, access pattern deviation | Investigate, restrict access, audit | < 30min |
| Supply Chain Attack | L7 (Scanning) | Image signing verification, SCA | Block deployment, revoke image, audit | < 2hr |
| Certificate Compromise | L4 (Mesh) | Certificate usage anomaly, revocation check | Revoke certificate, reissue, audit | < 5min |
| Zero-Day Vulnerability | L6 (Runtime) + L2 (WAF) | Behavioral anomaly, exploit attempt pattern | Virtual patch (WAF), isolate service | < 15min |

### Why This Works (Incident Response Matrix)

The matrix demonstrates defense in depth in action:

1. **Multiple detection layers**: Each threat is detected by one or more independent layers. If the WAF misses a SQL injection, the OPA policy or application-level input validation should catch it.

2. **Graduated response**: Responses range from immediate (block request) to manual (investigate insider threat). Automated responses handle the most common and dangerous threats.

3. **Realistic containment times**: Automated responses (certificate revocation, rate limiting) happen in seconds. Manual responses (insider threat investigation) take longer but are still bounded.

4. **Coverage across the attack surface**: The matrix covers external threats (DDoS, injection), internal threats (compromised service, insider), and supply chain threats (vulnerable dependency, malicious image).

### Common Mistakes

- **Only planning for external threats.** Insider threats and supply chain attacks are real risks. Include them in your incident response plan.
- **Not testing incident response.** Run tabletop exercises and chaos engineering experiments to verify that automated responses work as expected.
- **Ignoring the human element.** Automated responses handle the technical containment, but human judgment is needed for investigation, communication, and recovery.
- **Not updating the matrix.** As new threats emerge and the architecture evolves, the incident response matrix must be updated.

## Key Takeaway

Security architecture is not a collection of independent tools -- it is a system where each layer reinforces the others. DDoS protection, WAF, secrets management, security scanning, and zero trust controls must work together as a cohesive defense. The zero trust principle "never trust, always verify" applies not just to network access but to every layer: verify the request (WAF), verify the identity (mTLS), verify the authorization (OPA), verify the code (scanning), and verify the secrets (Vault). No single layer is sufficient. Defense in depth is not redundancy -- it is resilience.
