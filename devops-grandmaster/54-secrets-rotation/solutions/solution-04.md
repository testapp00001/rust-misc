# Solution 04: Zero-Downtime TLS Certificate Rotation

## Part A: Certificate Lifecycle

| Stage | Tools | Failure Mode | Mitigation |
|-------|-------|-------------|------------|
| Issuance | cert-manager, Let's Encrypt, Vault PKI | CA rate limiting, DNS timeout | Multiple CAs, retry with backoff, pre-stage certificates |
| Validation | ACME HTTP-01, DNS-01 | DNS propagation delay | Use DNS-01 for wildcards |
| Distribution | Kubernetes Secrets, volume mounts | Network partition | cert-manager built-in distribution |
| Activation | Nginx reload, Envoy SDS | Reload failure, config error | Test config before reload |
| Monitoring | Prometheus, cert-manager metrics | Missed alerts | Multiple alert channels |
| Renewal | cert-manager auto-renewal | ACME server down | Staging CA for testing |
| Rotation | Nginx reload, Envoy hot-reload | Connection drops | Graceful reload |
| Revocation | OCSP, CRL | OCSP responder down | OCSP stapling, CRL caching |

---

## Part B: cert-manager Configuration

```yaml
# cluster-issuer.yaml
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: security@example.com
    privateKeySecretRef:
      name: letsencrypt-prod-account-key
    solvers:
      - http01:
          ingress:
            class: nginx
      - selector:
          dnsZones:
            - "example.com"
        dns01:
          route53:
            region: us-east-1
```

```yaml
# certificate.yaml
apiVersion: cert-manager.io/v1
kind: Certificate
metadata:
  name: api-example-com
  namespace: production
spec:
  secretName: api-example-com-tls
  duration: 2160h    # 90 days
  renewBefore: 720h  # 30 days before expiry
  privateKey:
    algorithm: ECDSA
    size: 256
  usages:
    - server auth
    - digital signature
    - key encipherment
  dnsNames:
    - api.example.com
    - "*.api.example.com"
  issuerRef:
    name: letsencrypt-prod
    kind: ClusterIssuer
```

Answers:
1. **ACME and HTTP-01**: ACME is the protocol used by Let's Encrypt. HTTP-01 places a token at `http://your-domain/.well-known/acme-challenge/TOKEN`. The CA verifies domain ownership by requesting this URL.
2. **`renewBefore: 720h`**: Renews 30 days before expiry, providing a 30-day buffer for retries.
3. **Renewal failures**: cert-manager retries with exponential backoff. Events are recorded on the Certificate resource.

---

## Part C: Zero-Downtime Certificate Rotation

| Component | How It Reloads | Dropped Connections? |
|-----------|---------------|---------------------|
| Nginx | `nginx -s reload` (SIGHUP) | No -- new workers start, old workers finish current requests |
| HAProxy | Runtime API `reload ssl certs` | No |
| Envoy | SDS (Secret Discovery Service) | No |
| Traefik | Automatic file watching | No |
| AWS ALB | ACM auto-renewal | No |

```nginx
server {
    listen 443 ssl http2;
    server_name api.example.com;
    ssl_certificate /etc/ssl/certs/api.example.com/tls.crt;
    ssl_certificate_key /etc/ssl/certs/api.example.com/tls.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;
    ssl_stapling on;
    ssl_stapling_verify on;
    location / {
        proxy_pass http://backend;
    }
}
```

```bash
#!/bin/bash
# reload-nginx-certs.sh
set -euo pipefail

CERT_FILE="/etc/ssl/certs/api.example.com/tls.crt"
CERT_HASH_FILE="/tmp/cert-hash.txt"

# Check if certificate changed
CURRENT_HASH=$(sha256sum "$CERT_FILE" | awk '{print $1}')
if [ -f "$CERT_HASH_FILE" ]; then
    PREVIOUS_HASH=$(cat "$CERT_HASH_FILE")
    if [ "$CURRENT_HASH" = "$PREVIOUS_HASH" ]; then
        echo "Certificate unchanged, skipping reload"
        exit 0
    fi
fi

# Validate new certificate
openssl x509 -in "$CERT_FILE" -noout -checkend 86400 || {
    echo "ERROR: Certificate expires within 24 hours"
    exit 1
}

# Test Nginx config
nginx -t || exit 1

# Reload gracefully
nginx -s reload
echo "$CURRENT_HASH" > "$CERT_HASH_FILE"
echo "Certificate rotation successful"
```

### Why This Works

Nginx `reload` sends SIGHUP to the master process. New workers start with the new certificate. Old workers finish current requests and exit. Zero dropped connections. The certificate file path does not change -- only the content changes.

---

## Part D: Certificate Monitoring

```yaml
groups:
  - name: certificate-monitoring
    rules:
      - alert: CertificateExpiringSoon
        expr: (certmanager_certificate_expiration_timestamp_seconds - time()) / 86400 < 14
        for: 1h
        labels:
          severity: warning
        annotations:
          summary: "Certificate {{ $labels.name }} expires in {{ $value | humanize }} days"

      - alert: CertificateRenewalFailed
        expr: certmanager_certificate_ready_status{condition="False"} == 1
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Certificate renewal failed for {{ $labels.name }}"
```

---

## Part E: Emergency Certificate Rotation

```
EMERGENCY PROCEDURE:

1. DETECT (< 5 min): CT log monitoring, CA revocation notification
2. REVOKE (< 5 min): Submit revocation to CA, update OCSP
3. ISSUE (< 10 min): New key pair, new certificate from backup CA
4. DEPLOY (< 30 min): Update K8s Secret, reload all endpoints
5. VERIFY (< 10 min): Check all endpoints with openssl

PRE-STAGING: Maintain backup certificate from different CA in Vault.
Reduces emergency rotation to < 15 minutes.
```

### Common Mistakes to Avoid

1. **Manual certificate management.** Automate with cert-manager.
2. **Not monitoring expiration.** Alert at 14 days, not at expiration.
3. **Restarting instead of reloading.** Use `nginx -s reload` for graceful reload.
4. **Not testing the full lifecycle.** Test issuance, distribution, activation, revocation in staging.
5. **No emergency procedure.** Key compromise requires a different process than routine renewal.

## Key Takeaway

Certificate rotation must be fully automated. cert-manager handles issuance and renewal, Kubernetes distributes the certificate, and the TLS terminator reloads gracefully. Monitoring catches issues before outages, and emergency procedures handle key compromise.
