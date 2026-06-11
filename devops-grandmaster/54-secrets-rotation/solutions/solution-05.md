# Solution 05: Full Secrets Rotation Architecture

## Part A: Secrets Architecture

| Secret Type | Vault Path | Distribution | Rotation | During Rotation |
|-------------|-----------|-------------|----------|-----------------|
| DB credentials | `database/creds/{service}-{role}` | Vault Agent sidecar | Automatic, 24h TTL | Old creds valid until TTL. Connection pool rotates naturally. |
| Redis password | `secret/data/redis/{env}` | Vault Agent sidecar | Manual trigger, 30d | Dual-password in Redis ACL. 24h grace period. |
| API keys | `secret/data/api-keys/{svc}/{provider}` | Vault Agent sidecar | Manual trigger, 30d | Dual-key period. 30-day grace period. |
| TLS certificates | `pki/issue/{domain}` | cert-manager | Automatic, 90d cert | cert-manager updates Secret. Nginx reloads gracefully. |
| JWT signing keys | `transit/keys/{svc}-jwt` | Vault Transit | Automatic, 24h | New version for signing. Old versions retained for verification. |
| Encryption keys | `transit/keys/{svc}-encryption` | Vault Transit | Automatic, 90d | New version for encryption. Old versions retained for decryption. |

---

## Part B: Vault Cluster Configuration

```hcl
# vault-server.hcl
storage "raft" {
  path    = "/vault/data"
  node_id = "vault-1"
  retry_join { leader_api_addr = "https://vault-1.example.com:8200" }
  retry_join { leader_api_addr = "https://vault-2.example.com:8200" }
  retry_join { leader_api_addr = "https://vault-3.example.com:8200" }
}

listener "tcp" {
  address       = "0.0.0.0:8200"
  tls_cert_file = "/vault/tls/server.crt"
  tls_key_file  = "/vault/tls/server.key"
  tls_min_version = "tls12"
}

seal "awskms" {
  region     = "us-east-1"
  kms_key_id = "alias/vault-unseal"
}

telemetry {
  prometheus_retention_time = "30s"
  disable_hostname          = true
}

api_addr     = "https://vault.example.com:8200"
cluster_addr = "https://vault.example.com:8201"
ui = true
```

```hcl
# policies/payment-service.hcl
path "database/creds/payflow-payments-db" {
  capabilities = ["read"]
}
path "secret/data/api-keys/payment-service/stripe" {
  capabilities = ["read"]
}
path "transit/encrypt/payflow-encryption" {
  capabilities = ["update"]
}
path "transit/decrypt/payflow-encryption" {
  capabilities = ["update"]
}
path "transit/sign/payflow-jwt" {
  capabilities = ["update"]
}
path "secret/data/api-keys/user-service/*" {
  capabilities = ["deny"]
}
path "sys/*" {
  capabilities = ["deny"]
}
```

### Why This Works

Raft storage provides built-in HA without external dependencies. Three nodes provide fault tolerance. Auto-unseal with AWS KMS eliminates manual unsealing. Per-service policies enforce least privilege.

---

## Part C: Rotation Automation

```yaml
# rotation-schedule.yaml
rotations:
  database_credentials:
    method: dynamic
    ttl: 24h
    max_ttl: 72h
    trigger: automatic
    verification: "SELECT 1"

  api_keys:
    method: manual_trigger
    rotation_interval: 30d
    trigger: cronjob
    verification:
      stripe: "GET https://api.stripe.com/v1/balance"
    rollback_on_failure: true
    grace_period: 30d

  tls_certificates:
    method: cert_manager
    rotation_interval: 90d
    renew_before: 30d
    trigger: automatic
    verification: "openssl s_client -connect {domain}:443"

  jwt_signing_keys:
    method: vault_transit
    rotation_interval: 24h
    trigger: cronjob
    verification: "transit/sign and transit/verify"
    key_version_retention: 3

  encryption_keys:
    method: vault_transit
    rotation_interval: 90d
    trigger: cronjob
    verification: "transit/encrypt and transit/decrypt"
    min_decryption_version: current
    key_version_retention: 5
```

---

## Part D: Audit and Compliance

| Metric | Source | Alert Threshold | Compliance |
|--------|--------|-----------------|------------|
| Failed auth attempts | Vault audit log | > 5/min per IP | PCI DSS 10.2.4 |
| Secret access by service | Vault audit log | Anomaly detection | SOC2 CC6.1 |
| Rotation success rate | Rotation job logs | < 95% | PCI DSS 3.6.4 |
| Certificate expiration | cert-manager | < 14 days | PCI DSS 4.1 |
| Vault availability | Health endpoint | < 99.9% | SOC2 CC7.1 |
| Policy changes | Vault audit log | Any change | PCI DSS 7.1.1 |
| Root token usage | Vault audit log | Any usage | PCI DSS 7.1.2 |

### Why This Works

Vault audit logs record every operation with HMAC'd sensitive values. This satisfies PCI DSS requirement 10.2 and SOC2 CC6.1. Ship logs to SIEM for immutable audit trail.

---

## Part E: Disaster Recovery

| Failure | Impact | Recovery | RTO |
|---------|--------|----------|-----|
| Vault cluster down | No new credentials | Restart or restore from Raft snapshot | 1 hour |
| Single node down | Reduced HA | Restart node, Raft auto-recovery | 15 min |
| Data corruption | Secrets lost | Restore from Raft snapshot | 1 hour |
| KMS unavailable | Cannot auto-unseal | Manual unseal with recovery key | 30 min |
| CA compromise | All TLS certs invalid | Switch to backup CA, reissue | 2 hours |
| API key leak | Unauthorized access | Rotate immediately, audit access | 15 min |

### Common Mistakes to Avoid

1. **Single-node Vault.** Use 3-5 nodes for HA.
2. **Manual unsealing.** Use cloud KMS auto-unseal.
3. **Not testing rotation.** Test in staging before production.
4. **Not monitoring rotation.** Track success rates and failure rates.
5. **Not having a DR plan.** Test quarterly.

## Key Takeaway

Secrets management is a system, not a tool. It requires architecture (HA Vault), automation (rotation pipelines), monitoring (audit logs), and DR (snapshots and replication). The goal is to make rotation boring and routine.
