# Solution 05: External Secrets Operator -- Vault Integration

---

## Task 1: Install and Configure ESO

### Install via Helm

```bash
helm repo add external-secrets https://charts.external-secrets.io
helm repo update

helm install external-secrets external-secrets/external-secrets \
  --namespace external-secrets \
  --create-namespace \
  --set installCRDs=true
```

Verify the operator is running:

```bash
kubectl get pods -n external-secrets
```

Expected: three pods (operator, cert-controller, webhook) in `Running` state.

### Create a SecretStore

```yaml
apiVersion: external-secrets.io/v1beta1
kind: SecretStore
metadata:
  name: vault-backend
spec:
  provider:
    vault:
      server: "http://vault.default.svc:8200"
      path: "secret"
      version: "v2"
      auth:
        kubernetes:
          mountPath: "kubernetes"
          role: "payment-service"
          serviceAccountRef:
            name: default
```

Apply and verify:

```bash
kubectl apply -f secret-store.yaml
kubectl get secretstore vault-backend
```

The `STATUS` column should show `Valid`. If it shows `Invalid`, check:

1. Vault is reachable at the specified server address.
2. The Kubernetes auth method is enabled at the specified `mountPath`.
3. The `payment-service` role exists in Vault and is bound to the correct
   service account.
4. The service account token is valid and not expired.

**Why this works:** The SecretStore defines a connection to an external
provider. ESO uses the Kubernetes auth method to authenticate with Vault
by presenting the Pod's service account token. Vault validates this token
against the Kubernetes API and returns a Vault token scoped to the
configured role's policies.

---

## Task 2: Create an ExternalSecret

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: db-credentials
spec:
  refreshInterval: 1m
  secretStoreRef:
    name: vault-backend
    kind: SecretStore
  target:
    name: db-credentials
    creationPolicy: Owner
  data:
    - secretKey: DB_USER
      remoteRef:
        key: secret/data/payment
        property: DB_USER
    - secretKey: DB_PASSWORD
      remoteRef:
        key: secret/data/payment
        property: DB_PASSWORD
```

Apply and verify:

```bash
kubectl apply -f external-secret.yaml
kubectl get externalsecret db-credentials
kubectl get secret db-credentials -o yaml
```

The Kubernetes Secret `db-credentials` should contain the values from Vault.

**Why this works:**

- `refreshInterval: 1m` tells ESO to check Vault every minute for changes.
- `secretStoreRef` points to the SecretStore from Task 1.
- `target.name` is the name of the Kubernetes Secret to create.
- `target.creationPolicy: Owner` means ESO owns the Secret. It will be
  deleted if the ExternalSecret is deleted.
- `data[].secretKey` is the key in the Kubernetes Secret.
- `data[].remoteRef.key` is the path in Vault (KV v2 uses `secret/data/`).
- `data[].remoteRef.property` is the specific key within the Vault secret.

---

## Task 3: Verify Automatic Refresh

### Update the secret in Vault

```bash
vault kv put secret/payment DB_USER=app_user DB_PASSWORD=rotated-password-2026
```

### Check the Kubernetes Secret

Wait for the `refreshInterval` (1 minute), then:

```bash
kubectl get secret db-credentials -o jsonpath='{.data.DB_PASSWORD}' | base64 -d
```

Expected: `rotated-password-2026`

### Check sync status

```bash
kubectl get externalsecret db-credentials -o yaml
```

Look at `status`:

```yaml
status:
  conditions:
    - type: Ready
      status: "True"
      reason: SecretSynced
  refreshTime: "2026-06-11T12:01:00Z"
```

The `refreshTime` shows the last successful sync. The full cycle (Vault
update to K8s Secret update) takes at most the `refreshInterval` duration.

**Why this works:** ESO polls the external provider on each `refreshInterval`.
When it detects a change (by comparing the current K8s Secret data with the
Vault data), it updates the K8s Secret. The polling model is simple and
predictable, though it introduces a delay equal to the refresh interval.

---

## Task 4: Drift Detection

### Manually edit the Kubernetes Secret

```bash
kubectl patch secret db-credentials -p '{"stringData":{"DB_PASSWORD":"tampered-value"}}'
```

### Observe the revert

Within 1 minute (the `refreshInterval`), ESO detects that the K8s Secret
differs from the Vault source and overwrites it:

```bash
# Immediately after patch
kubectl get secret db-credentials -o jsonpath='{.data.DB_PASSWORD}' | base64 -d
# Output: tampered-value

# After refresh interval
kubectl get secret db-credentials -o jsonpath='{.data.DB_PASSWORD}' | base64 -d
# Output: rotated-password-2026
```

### CreationPolicy: Owner

When `creationPolicy: Owner` is set:

- ESO **owns** the Kubernetes Secret. The Secret has an owner reference
  pointing to the ExternalSecret.
- Manual edits are reverted on the next sync cycle.
- Deleting the ExternalSecret **deletes** the Kubernetes Secret.
- No other controller or user should manage this Secret directly.

Alternative policies:

| Policy | Behavior |
|--------|----------|
| `Owner` | ESO owns the Secret. Reverts manual changes. Deletes Secret when ExternalSecret is deleted. |
| `Merge` | ESO merges its keys into an existing Secret. Does not delete keys it does not manage. |
| `None` | ESO creates the Secret only if it does not exist. Never updates or deletes it. |

---

## Task 5: Multi-Secret Pattern

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: tls-certificates
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: vault-backend
    kind: SecretStore
  target:
    name: tls-certificates
    creationPolicy: Owner
    template:
      type: kubernetes.io/tls
  data:
    - secretKey: tls.crt
      remoteRef:
        key: secret/data/tls
        property: tls.crt
    - secretKey: tls.key
      remoteRef:
        key: secret/data/tls
        property: tls.key
```

Apply and verify:

```bash
kubectl apply -f external-secret-tls.yaml
kubectl get secret tls-certificates -o yaml
```

Verify the type:

```bash
kubectl get secret tls-certificates -o jsonpath='{.type}'
```

Expected: `kubernetes.io/tls`

**Why this works:**

- `target.template.type: kubernetes.io/tls` tells ESO to create a TLS-type
  Secret. This is the same type created by `kubectl create secret tls`.
- The keys `tls.crt` and `tls.key` are the conventional names that
  Kubernetes expects for TLS secrets (used by Ingress controllers, Istio,
  etc.).
- The `refreshInterval` is `1h` because TLS certificates rotate less
  frequently than database credentials. Longer intervals reduce Vault load.
- The Vault path `secret/data/tls` stores the PEM-encoded certificate and
  private key.

---

## Common Mistakes

1. **Using `secret/data/` prefix inconsistently.** Vault KV v2 stores data
   under `secret/data/{path}`. If you omit `data/`, you get a "secret not
   found" error. This is the most common ESO + Vault integration mistake.

2. **Confusing `SecretStore` and `ClusterSecretStore`.** A `SecretStore` is
   namespace-scoped. If your ExternalSecret is in namespace `app` but the
   SecretStore is in namespace `infra`, the ExternalSecret cannot reference
   it. Use `ClusterSecretStore` for cross-namespace access.

3. **Setting `refreshInterval` too short.** A 10-second refresh interval
   means ESO polls Vault 6 times per minute per ExternalSecret. With 50
   ExternalSecrets, that is 300 Vault requests per minute just for polling.
   Use intervals appropriate for the secret's rotation frequency.

4. **Not configuring Vault policies.** The ESO Vault role must have a policy
   that grants `read` access to the relevant secret paths. Without the
   correct policy, ESO authenticates successfully but gets a 403 on secret
   reads.

   ```hcl
   path "secret/data/payment" {
     capabilities = ["read"]
   }
   path "secret/data/tls" {
     capabilities = ["read"]
   }
   ```

5. **Forgetting `creationPolicy: Owner`.** Without this, ESO creates the
   Secret but does not own it. Manual edits are not reverted, and deleting
   the ExternalSecret leaves the orphaned Secret behind. This can lead to
   stale secrets lingering in the cluster.

6. **Not monitoring ESO health.** If the ESO operator crashes or loses Vault
   connectivity, secrets stop syncing. Monitor the ExternalSecret's
   `status.conditions` and set up alerts for `Ready=False` status:

   ```bash
   kubectl get externalsecrets -A -o json | \
     jq '.items[] | select(.status.conditions[0].status!="True") | .metadata.name'
   ```
