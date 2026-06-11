# Solution 04: Configuration Management Across Environments

---

## Task 1: Environment-Specific ConfigMaps

### Dev

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config-dev
  labels:
    environment: dev
data:
  DB_HOST: "localhost"
  LOG_LEVEL: "debug"
  FEATURE_FLAG_NEW_CHECKOUT: "true"
```

### Staging

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config-staging
  labels:
    environment: staging
data:
  DB_HOST: "staging-db.internal"
  LOG_LEVEL: "info"
  FEATURE_FLAG_NEW_CHECKOUT: "true"
```

### Production

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config-prod
  labels:
    environment: production
data:
  DB_HOST: "prod-db.internal"
  LOG_LEVEL: "warn"
  FEATURE_FLAG_NEW_CHECKOUT: "false"
```

### What is shared vs. what differs

| Aspect | Shared | Differs |
|--------|--------|---------|
| Key names | All three have identical keys (DB_HOST, LOG_LEVEL, FEATURE_FLAG_NEW_CHECKOUT) | -- |
| DB_HOST value | -- | Each environment points to its own database |
| LOG_LEVEL value | -- | Dev is verbose (debug), staging is standard (info), production is minimal (warn) |
| Feature flags | -- | New checkout is enabled in dev/staging for testing, disabled in production |

The structure is identical; only values change. This is the ideal scenario
for Kustomize overlays or Helm values files.

---

## Task 2: Immutable ConfigMaps for Production

### Updated Production ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config-prod
  labels:
    environment: production
immutable: true
data:
  DB_HOST: "prod-db.internal"
  LOG_LEVEL: "warn"
  FEATURE_FLAG_NEW_CHECKOUT: "false"
```

### 1. What happens if you try to `kubectl edit` an immutable ConfigMap?

The edit is rejected with an error:

```
error: configmaps "app-config-prod" is invalid:
A copy of your changes has been stored to "/tmp/kubectl-edit-xxx.yaml"
```

The API server returns a validation error because the `data` field of an
immutable resource cannot be modified.

### 2. Required workflow to change values

1. Update the YAML file with the new values and a **new ConfigMap name**
   (e.g., `app-config-prod-v2`).
2. Apply the new ConfigMap: `kubectl apply -f configmap-v2.yaml`.
3. Update the Deployment to reference the new ConfigMap name.
4. Kubernetes performs a rolling update, replacing Pods that use the old
   ConfigMap with Pods that use the new one.
5. Delete the old ConfigMap after all Pods have migrated: `kubectl delete
   configmap app-config-prod`.

### 3. Two benefits of immutable ConfigMaps

**Benefit 1: Accidental modification protection.** An operator cannot
accidentally `kubectl edit` or `kubectl patch` the ConfigMap and break
production. The API server rejects any mutation.

**Benefit 2: Performance improvement.** The kubelet does not need to watch
immutable ConfigMaps for changes. For large clusters with thousands of
ConfigMaps, this reduces API server load and kubelet memory usage. Kubernetes
documentation notes a significant reduction in kubelet overhead when many
ConfigMaps are marked immutable.

---

## Task 3: Hash-Triggered Rolling Restarts

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: payment-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: payment-service
  template:
    metadata:
      labels:
        app: payment-service
      annotations:
        # This annotation changes when the ConfigMap changes,
        # triggering a rolling update
        checksum/config: "CONFIG_HASH_PLACEHOLDER"
    spec:
      containers:
        - name: app
          image: myapp:v1
          envFrom:
            - configMapRef:
                name: app-config-prod
```

### CI/CD Pipeline Integration

In your deployment pipeline, compute the hash and substitute it:

```bash
#!/bin/bash
CONFIG_HASH=$(kubectl get configmap app-config-prod -o yaml | sha256sum | cut -d' ' -f1)

sed "s/CONFIG_HASH_PLACEHOLDER/$CONFIG_HASH/" deployment.yaml | kubectl apply -f -
```

### How it works

1. The pipeline computes a SHA-256 hash of the ConfigMap's YAML.
2. It injects the hash into the `checksum/config` annotation.
3. `kubectl apply` compares the new annotation with the running Pod template.
4. If the annotation changed (because the ConfigMap changed), Kubernetes
   detects a Pod template change and triggers a rolling update.
5. New Pods are created with the updated ConfigMap reference.
6. Old Pods are terminated after new Pods pass readiness checks.

### Alternative: kubectl with inline hash

For manual deployments without a pipeline:

```bash
CONFIG_HASH=$(kubectl get configmap app-config-prod -o json | sha256sum | cut -d' ' -f1)

kubectl patch deployment payment-service -p \
  "{\"spec\":{\"template\":{\"metadata\":{\"annotations\":{\"checksum/config\":\"$CONFIG_HASH\"}}}}}"
```

This patches only the annotation, triggering a rolling restart without
changing anything else.

---

## Task 4: Secret Separation Strategy

| Concern | Approach A (Sealed Secrets / SOPS) | Approach B (External Vault) |
|---------|-------------------------------------|----------------------------|
| Secrets in Git? | Yes, but encrypted. The encrypted blob is in Git. Decrypting requires the cluster's private key. | No. Only references (SecretStore config) are in Git. Actual values live in the vault. |
| Rotation workflow | Update the encrypted manifest in Git, commit, push. CI/CD decrypts and applies. Requires a new commit for each rotation. | Update the value in Vault. ESO syncs automatically within the refresh interval. No Git commit needed. |
| Audit trail | Git history shows when encrypted values changed, but not what they changed to. Vault provides no audit trail. | Vault provides a full audit trail: who read/wrote which secret, when, from which IP. |
| Operational complexity | Low. Uses existing Git workflow. Requires Sealed Secrets controller or SOPS key management. | Moderate. Requires Vault cluster, auth configuration, ESO installation, and ongoing Vault operations. |
| Disaster recovery | Git clone contains all encrypted secrets. Restore the cluster, install Sealed Secrets controller with the same key, and secrets are available. | Vault must be restored first. If Vault is down, Kubernetes secrets cannot be synced. Requires Vault HA and backup strategy. |

### Recommendation for Production: Approach B (External Vault)

**Reasons:**

1. **Audit trail is non-negotiable for compliance.** PCI-DSS, SOC 2, and
   HIPAA require knowing who accessed secrets and when. Vault provides this
   natively. Sealed Secrets in Git provides no access audit trail.

2. **Rotation without Git commits.** With Approach A, every credential
   rotation requires a Git commit, PR review, and CI/CD pipeline run. With
   Approach B, the security team rotates credentials in Vault, and ESO
   propagates the change automatically. This decouples security operations
   from development workflows.

3. **Dynamic secrets.** Vault can generate short-lived credentials on demand
   (e.g., Vault's database secrets engine creates temporary PostgreSQL users
   with a configurable TTL). This eliminates long-lived credentials entirely.
   Approach A cannot do this.

4. **Single source of truth.** With Approach B, Vault is the authoritative
   source for all secrets across all clusters and environments. With Approach
   A, each Git repository is a source, creating fragmented secret management.

**The trade-off is operational complexity.** Vault requires dedicated
infrastructure, HA configuration, backup strategy, and team expertise.
For organizations without a secrets management platform, Approach A
(Sealed Secrets) is a pragmatic starting point that can be migrated to
Approach B later.

---

## Common Mistakes

1. **Using a single ConfigMap for all environments.** This forces you to
   manage environment-specific overrides in your Deployment YAML rather than
   in the ConfigMap. One ConfigMap per environment keeps each self-contained
   and independently manageable.

2. **Forgetting to update the Deployment when using immutable ConfigMaps.**
   Changing an immutable ConfigMap requires creating a new one and updating
   the Deployment reference. If you only create the new ConfigMap but forget
   to update the Deployment, Pods continue using the old (now-deleted)
   ConfigMap and fail.

3. **Computing the config hash from the wrong source.** The hash must be
   computed from the ConfigMap's actual content, not from the ConfigMap name.
   If you hash the name (which does not change), the annotation never changes
   and no rolling restart is triggered.

4. **Not using `stringData` when applying secrets imperatively.** When
   patching a secret, use `stringData` to provide plain text values. If you
   use `data`, you must base64-encode the values yourself, which is
   error-prone:

   ```bash
   # Error-prone
   kubectl patch secret db-creds -p '{"data":{"DB_PASSWORD":"bmV3LXBhc3M="}}'

   # Simpler
   kubectl patch secret db-creds -p '{"stringData":{"DB_PASSWORD":"new-pass"}}'
   ```

5. **Treating all secrets the same.** TLS certificates, database passwords,
   and API keys have different rotation frequencies and lifecycle
   requirements. Group secrets by rotation policy, not by application.
