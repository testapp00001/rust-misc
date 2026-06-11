# Exercise 05: External Secrets Operator -- Vault Integration

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Advanced

## Objective

Use the External Secrets Operator (ESO) to synchronize secrets from HashiCorp
Vault into Kubernetes Secrets, configure automatic refresh, and implement a
drift-detection strategy so that manually edited Kubernetes Secrets are
reconciled back to the vault source of truth.

---

## Background

Your organization stores all secrets in HashiCorp Vault. The security team
mandates that:

1. Secret values must never be stored in Git (not even encrypted).
2. Secrets must be rotated in Vault and automatically propagated to Kubernetes.
3. If someone manually edits a Kubernetes Secret, the operator must revert it
   to the vault value within 1 minute.
4. Applications consume secrets as mounted volumes so they pick up changes
   without restarts.

You need to set up the External Secrets Operator to bridge Vault and
Kubernetes.

---

## Tasks

### Task 1: Install and Configure ESO

Write the Helm commands and YAML manifests to:

1. Install the External Secrets Operator via Helm.
2. Create a `SecretStore` resource that connects to a Vault server at
   `http://vault.default.svc:8200` using Kubernetes auth.
3. The Vault role should be `payment-service` and the mount path should be
   `kubernetes`.

Apply all resources and verify the SecretStore reports a `Valid` status.

<details>
<summary>Hint 1: SecretStore vs ClusterSecretStore</summary>

A `SecretStore` is namespace-scoped -- it can only be referenced by
ExternalSecrets in the same namespace. A `ClusterSecretStore` is
cluster-scoped and can be referenced from any namespace. Use
`ClusterSecretStore` when multiple namespaces need access to the same
vault backend.

</details>

<details>
<summary>Hint 2: Kubernetes auth in Vault</summary>

Vault's Kubernetes auth method validates service account tokens. The ESO
service account needs permission to authenticate with Vault. The typical
flow:

1. ESO authenticates to Vault using its Kubernetes service account token.
2. Vault validates the token against the Kubernetes API.
3. Vault returns a Vault token scoped to the configured role and policies.

</details>

### Task 2: Create an ExternalSecret

Create an `ExternalSecret` resource that:

1. References the `SecretStore` from Task 1.
2. Pulls `DB_USER` and `DB_PASSWORD` from the Vault path `secret/data/payment`.
3. Creates a Kubernetes Secret named `db-credentials` in the same namespace.
4. Sets a `refreshInterval` of `1m`.

Verify the Kubernetes Secret is created and contains the expected values.

<details>
<summary>Hint 3: Vault KV v2 path structure</summary>

Vault KV v2 stores data under `secret/data/{path}`, and the actual values
are nested under `data.data` in the response. In the ExternalSecret's
`remoteRef`, use:

```yaml
remoteRef:
  key: secret/data/payment
  property: DB_PASSWORD
```

The `property` field selects a specific key from the Vault secret's data
map.

</details>

### Task 3: Verify Automatic Refresh

1. Update the secret value in Vault (change `DB_PASSWORD` to a new value).
2. Wait for the `refreshInterval` to elapse.
3. Verify the Kubernetes Secret has the new value.
4. Check the ExternalSecret's status for the `lastSyncedTime`.

How long did the full cycle take (vault update to K8s secret update)?

<details>
<summary>Hint 4: Checking sync status</summary>

```bash
kubectl get externalsecret db-credentials -o yaml
```

Look at `status.conditions` for `Ready=True` and `status.refreshTime` for
the last successful sync timestamp.

</details>

### Task 4: Drift Detection

1. Manually edit the Kubernetes Secret `db-credentials` to contain a
   different password.
2. Observe how long it takes for the operator to revert the change.
3. What happens if you set `creationPolicy: Owner` on the ExternalSecret?
   Who owns the Kubernetes Secret?

<details>
<summary>Hint 5: CreationPolicy and DeletionPolicy</summary>

- `creationPolicy: Owner` -- ESO owns the Secret. It will overwrite manual
  changes and delete the Secret when the ExternalSecret is deleted.
- `creationPolicy: Merge` -- ESO merges its data into an existing Secret
  but does not delete it.
- `deletionPolicy: Retain` -- The Secret persists even if the
  ExternalSecret is deleted.
- `deletionPolicy: Delete` -- The Secret is deleted when the
  ExternalSecret is deleted.

</details>

### Task 5: Multi-Secret Pattern

Your application also needs TLS certificates from Vault at the path
`secret/data/tls`. Create a second ExternalSecret that:

1. Pulls `tls.crt` and `tls.key` from Vault.
2. Creates a Kubernetes Secret of type `kubernetes.io/tls`.
3. Uses `refreshInterval` of `1h` (certificates rotate less frequently).

Verify the TLS Secret is created with the correct type.

<details>
<summary>Hint 6: Secret type in ExternalSecret</summary>

Use the `target.template.type` field:

```yaml
target:
  template:
    type: kubernetes.io/tls
```

The keys must be `tls.crt` and `tls.key` for TLS secrets.

</details>

---

## Success Criteria

- [ ] SecretStore is installed and reports `Valid` status.
- [ ] ExternalSecret creates a Kubernetes Secret with values from Vault.
- [ ] Secret rotation in Vault propagates to Kubernetes within the
      configured refresh interval.
- [ ] Manual edits to the Kubernetes Secret are reverted by the operator.
- [ ] A TLS-type Secret is created from a separate Vault path.

---

## Hints

<details>
<summary>Hint 7: ESO Architecture</summary>

The External Secrets Operator runs three components:

1. **Operator controller** -- reconciles ExternalSecret resources.
2. **SecretStore controller** -- validates SecretStore connectivity.
3. **Cert controller** -- manages webhook TLS certificates.

The operator watches ExternalSecret resources. On each reconciliation (every
`refreshInterval`), it reads from the external provider and updates the
target Kubernetes Secret.

</details>
