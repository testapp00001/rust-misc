# 56 - Zero Trust Architecture

**Previous:** [55 - Vulnerability Scanning](../55-vulnerability-scanning/README.md) | **Next:** [57 - Pipeline Design](../57-pipeline-design/README.md)

---

## Problem

Your network perimeter is "secure" -- firewalls, VPNs, intrusion detection systems. But an attacker phishes a developer's credentials, connects to the VPN, and now has unrestricted access to every internal service. The database server, the internal APIs, the admin panel -- all trust any request from within the network. The attacker exfiltrates your entire user database in 6 hours, and nobody notices for 3 months.

Traditional security assumes that everything inside the network is trusted. This is the "castle and moat" model -- once you cross the drawbridge, you can go anywhere. Zero Trust Architecture eliminates this assumption. **Never trust, always verify.** Every request, regardless of origin, must be authenticated, authorized, and encrypted.

---

## Naive Way

```yaml
# Traditional network: everything inside the firewall is trusted
network:
  firewall:
    external_rules:
      - allow: 443 from internet to web-servers
      - deny: all from internet
    internal_rules:
      - allow: all from app-subnet to db-subnet
      - allow: all from dev-subnet to all  # Developers can access everything
      - allow: all from monitoring to all
  vpn:
    # Once on VPN, you can reach anything
    split_tunnel: false
    access: "all internal resources"

# No service-to-service authentication
# No encryption between internal services
# No per-request authorization
# Lateral movement is trivial
```

**Why this fails:**
- VPN access grants access to everything on the internal network
- No service-to-service authentication means any compromised service can impersonate any other
- No encryption between internal services means traffic can be sniffed
- Flat network topology enables lateral movement
- One compromised credential breaches the entire network

---

## Right Way

### Core Zero Trust Principles

```
1. Verify Explicitly
   - Every request must be authenticated and authorized
   - No implicit trust based on network location
   - Use strong authentication (mTLS, JWT, OAuth2)

2. Use Least Privilege Access
   - Grant minimum permissions needed for each task
   - Just-in-time and just-enough-access (JIT/JEA)
   - Time-bound access that expires automatically

3. Assume Breach
   - Segment access to minimize blast radius
   - Encrypt all traffic (east-west and north-south)
   - Continuously monitor and verify
```

### Mutual TLS (mTLS) Between Services

Every service presents a certificate and verifies the certificate of the service it communicates with. Neither side trusts the other based on network location alone.

```rust
// src/tls/mutual_tls.rs
use rustls::{ClientConfig, ServerConfig, Certificate, PrivateKey};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

pub fn create_mtls_server_config(
    cert_path: &str,
    key_path: &str,
    ca_path: &str,
) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    // Load server certificate and private key
    let cert_file = std::fs::File::open(cert_path)?;
    let mut cert_reader = std::io::BufReader::new(cert_file);
    let certs: Vec<Certificate> = rustls::internal::pemfile::certs(&mut cert_reader)?;

    let key_file = std::fs::File::open(key_path)?;
    let mut key_reader = std::io::BufReader::new(key_file);
    let key = rustls::internal::pemfile::pkcs8_private_keys(&mut key_reader)?
        .into_iter()
        .next()
        .ok_or("No private key found")?;

    // Load CA certificate for client verification
    let ca_file = std::fs::File::open(ca_path)?;
    let mut ca_reader = std::io::BufReader::new(ca_file);
    let ca_certs: Vec<Certificate> = rustls::internal::pemfile::certs(&mut ca_reader)?;

    let mut root_store = rustls::RootCertStore::empty();
    for ca_cert in ca_certs {
        root_store.add(&ca_cert)?;
    }

    // Configure to REQUIRE client certificates (mTLS)
    let client_verifier = rustls::AllowAnyAuthenticatedClient::new(root_store);

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_client_cert_verifier(client_verifier)
        .with_single_cert(certs, key)?;

    Ok(config)
}

pub fn create_mtls_client_config(
    cert_path: &str,
    key_path: &str,
    ca_path: &str,
) -> Result<ClientConfig, Box<dyn std::error::Error>> {
    // Load client certificate for presenting to server
    let cert_file = std::fs::File::open(cert_path)?;
    let mut cert_reader = std::io::BufReader::new(cert_file);
    let certs: Vec<Certificate> = rustls::internal::pemfile::certs(&mut cert_reader)?;

    let key_file = std::fs::File::open(key_path)?;
    let mut key_reader = std::io::BufReader::new(key_file);
    let key = rustls::internal::pemfile::pkcs8_private_keys(&mut key_reader)?
        .into_iter()
        .next()
        .ok_or("No private key found")?;

    // Load server CA for verification
    let ca_file = std::fs::File::open(ca_path)?;
    let mut ca_reader = std::io::BufReader::new(ca_file);
    let mut root_store = rustls::RootCertStore::empty();
    for cert in rustls::internal::pemfile::certs(&mut ca_reader)? {
        root_store.add(&cert)?;
    }

    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_client_auth_cert(certs, key)?;

    Ok(config)
}

pub async fn start_mtls_server(
    listener: TcpListener,
    config: Arc<ServerConfig>,
) -> Result<(), Box<dyn std::error::Error>> {
    let acceptor = TlsAcceptor::from(config);

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let acceptor = acceptor.clone();

        tokio::spawn(async move {
            match acceptor.accept(stream).await {
                Ok(tls_stream) {
                    // Peer is authenticated via mTLS certificate
                    println!("Authenticated connection from {}", peer_addr);
                    // Handle the connection
                    handle_connection(tls_stream).await;
                }
                Err(e) => {
                    eprintln!("mTLS handshake failed from {}: {}", peer_addr, e);
                    // Reject unauthenticated connections
                }
            }
        });
    }
}
```

### Per-Request Authorization

```rust
// src/authz/authorization.rs
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceIdentity {
    pub service_name: String,
    pub certificate_cn: String,
    pub namespace: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizationPolicy {
    pub source_service: String,
    pub target_service: String,
    pub allowed_methods: Vec<String>,
    pub allowed_paths: Vec<String>,
}

pub struct PolicyEngine {
    policies: Vec<AuthorizationPolicy>,
}

impl PolicyEngine {
    pub fn new(policies: Vec<AuthorizationPolicy>) -> Self {
        Self { policies }
    }

    pub fn is_authorized(
        &self,
        source: &ServiceIdentity,
        target: &str,
        method: &str,
        path: &str,
    ) -> bool {
        self.policies.iter().any(|policy| {
            policy.source_service == source.service_name
                && policy.target_service == target
                && (policy.allowed_methods.is_empty() || policy.allowed_methods.contains(&method.to_string()))
                && (policy.allowed_paths.is_empty() || policy.allowed_paths.iter().any(|p| path.starts_with(p)))
        })
    }
}

// Authorization middleware
pub async fn authorize_request(
    State(policy_engine): State<Arc<PolicyEngine>>,
    request: Request,
    next: Next,
) -> Response {
    // Extract service identity from mTLS certificate (set by TLS layer)
    let source_identity = request.extensions().get::<ServiceIdentity>();

    let identity = match source_identity {
        Some(id) => id,
        None => {
            return Response::builder()
                .status(401)
                .body("No service identity found".into())
                .unwrap();
        }
    };

    let method = request.method().as_str();
    let path = request.uri().path();
    let target = "api-service"; // Current service name

    if !policy_engine.is_authorized(identity, target, method, path) {
        return Response::builder()
            .status(403)
            .body(format!(
                "Unauthorized: {} cannot {} {}",
                identity.service_name, method, path
            ).into())
            .unwrap();
    }

    next.run(request).await
}
```

### SPIFFE/SPIRE for Service Identity

```yaml
# spire-server-config.yaml
apiVersion: spire.spiffe.io/v1alpha1
kind: ClusterSPIFFEID
metadata:
  name: production-services
spec:
  spiffeIDTemplate: "spiffe://company.com/ns/{{ .Pod.Namespace }}/sa/{{ .Pod.ServiceAccount }}"
  podSelector:
    matchLabels:
      environment: production
  workloadSelectorTemplate: "k8s:ns:{{ .Pod.Namespace }}:sa:{{ .Pod.ServiceAccount }}"
---
# Service registration
apiVersion: spire.spiffe.io/v1alpha1
kind: ClusterSPIFFEID
metadata:
  name: api-service
spec:
  spiffeIDTemplate: "spiffe://company.com/ns/production/sa/api-service"
  podSelector:
    matchLabels:
      app: api-service
  dnsNameTemplates:
    - "api-service.production.svc.cluster.local"
```

---

## Production Way

### Service Mesh with Istio (Zero Trust by Default)

```yaml
# istio-config.yaml

# PeerAuthentication: Require mTLS for all services
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: default
  namespace: production
spec:
  mtls:
    mode: STRICT  # Require mTLS for all traffic
---
# AuthorizationPolicy: Define fine-grained access control
apiVersion: security.istio.io/v1beta1
kind: AuthorizationPolicy
metadata:
  name: api-service-policy
  namespace: production
spec:
  selector:
    matchLabels:
      app: api-service
  rules:
    - from:
        - source:
            principals:
              - "cluster.local/ns/production/sa/web-frontend"
              - "cluster.local/ns/production/sa/mobile-backend"
      to:
        - operation:
            methods: ["GET", "POST"]
            paths: ["/api/v1/*"]
    - from:
        - source:
            principals:
              - "cluster.local/ns/production/sa/admin-service"
      to:
        - operation:
            methods: ["GET", "POST", "PUT", "DELETE"]
            paths: ["/api/v1/*", "/admin/*"]
---
# Deny all other traffic
apiVersion: security.istio.io/v1beta1
kind: AuthorizationPolicy
metadata:
  name: deny-all
  namespace: production
spec:
  {}  # Empty spec = deny all traffic without explicit allow
```

### Network Segmentation with Calico

```yaml
# calico-network-policy.yaml

# Default deny all ingress
apiVersion: projectcalico.org/v3
kind: GlobalNetworkPolicy
metadata:
  name: default-deny
spec:
  selector: all()
  types:
    - Ingress
    - Egress
  ingress:
    - action: Deny
  egress:
    - action: Deny
---
# Allow api-service to talk to database
apiVersion: projectcalico.org/v3
kind: NetworkPolicy
metadata:
  name: api-to-db
  namespace: production
spec:
  selector: app == 'database'
  ingress:
    - action: Allow
      protocol: TCP
      destination:
        ports:
          - 5432
      source:
        selector: app == 'api-service'
---
# Allow api-service to talk to redis
apiVersion: projectcalico.org/v3
kind: NetworkPolicy
metadata:
  name: api-to-redis
  namespace: production
spec:
  selector: app == 'redis'
  ingress:
    - action: Allow
      protocol: TCP
      destination:
        ports:
          - 6379
      source:
        selector: app == 'api-service'
```

### Continuous Verification with Open Policy Agent (OPA)

```rego
# policy/authz.rego - Zero trust authorization policy
package authz

default allow = false

# Allow if request has valid service identity and is authorized
allow {
    # Verify the requesting service
    input.source.principal != ""

    # Check against allowed service mappings
    service_allowed[input.source.principal][input.target.service]

    # Check method is allowed
    method_allowed[input.request.method]

    # Check path is allowed
    path_allowed(input.request.path)
}

# Service-to-service authorization matrix
service_allowed := {
    "web-frontend": {"api-service", "cdn-service"},
    "mobile-backend": {"api-service"},
    "api-service": {"database", "redis", "payment-service"},
    "admin-service": {"api-service", "database"},
}

# Allowed HTTP methods
method_allowed := {"GET", "POST", "PUT", "DELETE", "PATCH"}

# Path authorization with wildcards
path_allowed(path) {
    startswith(path, "/api/v1/")
}

path_allowed(path) {
    startswith(path, "/health")
}

path_allowed(path) {
    startswith(path, "/metrics")
}

# Rate limiting per service
rate_limit_exceeded {
    input.source.request_count > input.source.rate_limit
}
```

### Identity-Aware Proxy

```rust
// src/zero_trust/identity_proxy.rs
use axum::{
    extract::ConnectInfo,
    http::HeaderMap,
    middleware::Next,
    response::Response,
    extract::Request,
};
use std::net::SocketAddr;

#[derive(Debug)]
pub struct RequestContext {
    pub service_identity: String,
    pub user_identity: Option<String>,
    pub source_ip: std::net::IpAddr,
    pub request_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub async fn identity_aware_proxy(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Response {
    // Extract identity from mTLS certificate (set by TLS termination layer)
    let service_identity = headers
        .get("X-Service-Identity")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    // Extract user identity from JWT
    let user_identity = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|token| validate_jwt(token).ok())
        .map(|claims| claims.sub);

    // Generate request ID for tracing
    let request_id = uuid::Uuid::new_v4().to_string();

    let context = RequestContext {
        service_identity,
        user_identity,
        source_ip: addr.ip(),
        request_id,
        timestamp: chrono::Utc::now(),
    };

    // Log every request for audit trail
    log::info!(
        "Request: {} {} from {} (service={}, user={})",
        request.method(),
        request.uri(),
        context.source_ip,
        context.service_identity,
        context.user_identity.as_deref().unwrap_or("anonymous")
    );

    // Attach context to request for downstream use
    request.extensions_mut().insert(context);

    let mut response = next.run(request).await;

    // Add security headers
    let headers = response.headers_mut();
    headers.insert("X-Request-ID", request_id.parse().unwrap());
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());

    response
}
```

### Certificate Rotation Automation

```yaml
# cert-manager configuration for automatic certificate rotation
apiVersion: cert-manager.io/v1
kind: Certificate
metadata:
  name: api-service-tls
  namespace: production
spec:
  secretName: api-service-tls
  duration: 24h      # Short-lived certificates
  renewBefore: 8h    # Renew 8 hours before expiry
  isCA: false
  privateKey:
    algorithm: ECDSA
    size: 256
  usages:
    - server auth
    - client auth    # Both server and client for mTLS
  dnsNames:
    - api-service.production.svc.cluster.local
    - api-service
  issuerRef:
    name: production-ca
    kind: ClusterIssuer
---
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: production-ca
spec:
  ca:
    secretName: production-ca-key
```

---

## Hands-On Lab

### Lab: Implement Zero Trust with mTLS and Service Mesh

**Duration:** 120 minutes

**Prerequisites:**
- Kubernetes cluster (minikube or kind)
- Istio installed (`istioctl install --set profile=demo`)
- Two simple HTTP services (can use httpbin and a custom app)

**Step 1: Deploy Services with Istio Sidecar Injection**

```bash
# Enable sidecar injection in namespace
kubectl create namespace production
kubectl label namespace production istio-injection=enabled

# Deploy httpbin as the "backend" service
kubectl apply -f https://raw.githubusercontent.com/istio/istio/master/samples/httpbin/httpbin.yaml -n production

# Deploy a simple client service
kubectl apply -n production -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: client
spec:
  replicas: 1
  selector:
    matchLabels:
      app: client
  template:
    metadata:
      labels:
        app: client
    spec:
      containers:
      - name: client
        image: curlimages/curl
        command: ["sleep", "infinity"]
EOF
```

**Step 2: Enable Strict mTLS**

```yaml
# Apply strict mTLS for the namespace
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: default
  namespace: production
spec:
  mtls:
    mode: STRICT
```

```bash
kubectl apply -f strict-mtls.yaml

# Verify mTLS is active
istioctl x describe pod $(kubectl get pod -l app=httpbin -n production -o jsonpath='{.items[0].metadata.name}') -n production
```

**Step 3: Test Without Authorization (Should Fail)**

```bash
# Try to access httpbin from client WITHOUT authorization policy
kubectl exec -it $(kubectl get pod -l app=client -n production -o jsonpath='{.items[0].metadata.name}') -n production -c client -- \
    curl http://httpbin.production.svc.cluster.local:8000/get

# With PeerAuthentication in STRICT mode but no AuthorizationPolicy,
# this should succeed because Istio allows all traffic by default
```

**Step 4: Apply Authorization Policy (Zero Trust)**

```yaml
# Deny all traffic by default
apiVersion: security.istio.io/v1beta1
kind: AuthorizationPolicy
metadata:
  name: deny-all
  namespace: production
spec:
  {}
---
# Allow only client -> httpbin on specific paths
apiVersion: security.istio.io/v1beta1
kind: AuthorizationPolicy
metadata:
  name: allow-client-to-httpbin
  namespace: production
spec:
  selector:
    matchLabels:
      app: httpbin
  rules:
    - from:
        - source:
            principals:
              - "cluster.local/ns/production/sa/default"
      to:
        - operation:
            methods: ["GET"]
            paths: ["/get", "/headers"]
```

```bash
kubectl apply -f authorization-policies.yaml

# Now test: /get should work
kubectl exec -it $(kubectl get pod -l app=client -n production -o jsonpath='{.items[0].metadata.name}') -n production -c client -- \
    curl http://httpbin.production.svc.cluster.local:8000/get

# /post should be denied (403)
kubectl exec -it $(kubectl get pod -l app=client -n production -o jsonpath='{.items[0].metadata.name}') -n production -c client -- \
    curl -X POST http://httpbin.production.svc.cluster.local:8000/post
```

**Step 5: Verify Certificate Rotation**

```bash
# Check the certificates in the sidecar
istioctl proxy-config secret $(kubectl get pod -l app=httpbin -n production -o jsonpath='{.items[0].metadata.name}') -n production

# Watch certificate expiration
watch -n 5 'istioctl proxy-config secret $(kubectl get pod -l app=httpbin -n production -o jsonpath='{.items[0].metadata.name}') -n production | grep -A5 "EXPIRATION"'

# Istio automatically rotates certificates before they expire
```

**Deliverable:** Document the complete request path showing how a request from the client to httpbin is authenticated (mTLS), authorized (AuthorizationPolicy), and encrypted (TLS 1.3). Show the certificate chain and explain how a compromised service would be blocked.

---

## Limitation

Zero Trust Architecture secures your infrastructure at the network and service level. Every request is verified, every service has an identity, and lateral movement is prevented. But securing the infrastructure is only half the battle.

The other half is **how you deploy changes to that infrastructure**. A zero-trust network is useless if your deployment process involves a developer manually running `kubectl apply` from their laptop at 2 AM. If the deployment pipeline does not verify that the code was reviewed, tested, scanned, and approved, a malicious or buggy change can bypass all your security controls.

The deployment pipeline itself must be zero-trust: every build must be reproducible, every artifact must be signed, every deployment must be auditable, and every change must pass through automated gates before reaching production.

---

## Next Topic

[57 - Pipeline Design](../57-pipeline-design/README.md) -- Learn how to design robust, secure, and efficient CI/CD pipelines that enforce quality gates, ensure reproducibility, and automate the entire path from code commit to production deployment.
