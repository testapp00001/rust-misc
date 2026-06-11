# Solution 01: ConfigMap vs Secret -- Know the Difference

---

## Task 1: Classification Table

| Item | Where | Why |
|------|-------|-----|
| Database hostname | **ConfigMap** | Non-sensitive. The hostname is discoverable and does not grant access by itself. |
| Database port | **ConfigMap** | Non-sensitive. Port numbers are standard knowledge (5432 for PostgreSQL, 3306 for MySQL). |
| Log level | **ConfigMap** | Non-sensitive. A tuning knob with no security implications. |
| Database password | **Secret** | Sensitive. Grants direct access to the database. Exposure enables data exfiltration. |
| TLS certificate | **Secret** | Sensitive (partially). The certificate itself is public, but it is always paired with a private key. Kubernetes TLS Secrets bundle both. |
| API key | **Secret** | Sensitive. Grants access to a third-party service. Exposure allows impersonation and billing abuse. |
| Feature flag: dark mode | **ConfigMap** | Non-sensitive. A UI toggle with no security impact. |
| JWT signing secret | **Secret** | Sensitive. Anyone with this key can forge valid JWTs and impersonate any user. |

---

## Task 2: Storage Differences

### 1. How does Kubernetes encode data internally in a ConfigMap?

ConfigMap values are stored as **plain text** strings in etcd. There is no
encoding. The `data` field is a map of string keys to string values.

```yaml
data:
  LOG_LEVEL: "info"    # Stored as-is
```

### 2. How does Kubernetes encode data internally in a Secret?

Secret values are stored as **base64-encoded** strings. Each value in the
`data` field must be a valid base64 string. Kubernetes decodes it when
injecting into a Pod (as an env var or file).

```yaml
data:
  DB_PASSWORD: czNjdXIzUEBzcyE=    # base64 of "s3cur3P@ss!"
```

Alternatively, the `stringData` field accepts plain text, and Kubernetes
encodes it before storing.

### 3. Is base64 encoding the same as encryption?

**No.** Base64 is a reversible encoding, not encryption:

- **Encoding** transforms data into a different format for transport. No key
  is needed. Anyone can decode it.
- **Encryption** transforms data using a key and algorithm. Decryption
  requires the key.

```bash
# Encoding: trivially reversible
echo "s3cur3P@ss!" | base64          # czNjdXIzUEBzcyE=
echo "czNjdXIzUEBzcyE=" | base64 -d  # s3cur3P@ss!

# Encryption: requires a key
openssl enc -aes-256-cbc -salt -in plain.txt -out encrypted.bin -pass pass:mykey
```

### 4. What must you configure to encrypt Secrets at rest in etcd?

You must create an `EncryptionConfiguration` resource for the kube-apiserver
that specifies an encryption provider (e.g., `aescbc`, `secretbox`, or an
external KMS provider). The apiserver must be started with the
`--encryption-provider-config` flag pointing to this file.

```yaml
apiVersion: apiserver.config.k8s.io/v1
kind: EncryptionConfiguration
resources:
  - resources:
      - secrets
    providers:
      - aescbc:
          keys:
            - name: key1
              secret: <base64-encoded-32-byte-key>
      - identity: {}    # Fallback for reading unencrypted secrets
```

After applying this, new or updated secrets are encrypted. Existing secrets
are encrypted on the next write.

---

## Task 3: Update Behavior

### 1. Does Pod A pick up the change?

**No.** When a ConfigMap is consumed as an environment variable, Kubernetes
injects the values into the container's environment at startup. The
container process reads them once. Changing the ConfigMap has no effect on
the running process. The Pod must be restarted.

### 2. Does Pod B pick up the change?

**Yes.** When a ConfigMap is mounted as a volume, Kubernetes creates a
projected volume backed by files on the node. The kubelet periodically syncs
the ConfigMap contents to these files (default interval: up to 60 seconds).
After the sync, the files at the mount path reflect the updated values.

### 3. What command forces Pod A to see the new value?

```bash
kubectl rollout restart deployment <deployment-name>
```

This terminates the existing Pods and creates new ones. The new Pods read
the updated ConfigMap at startup.

### 4. Roughly how long does it take for Pod B to see the new value?

**Up to 60 seconds** by default. The kubelet syncs mounted ConfigMap
contents based on its `--sync-frequency` flag (default 1 minute). In
practice, the update often appears within 30-60 seconds. Kubernetes also
supports a `Watch` detection strategy that can be faster.

---

## Common Mistakes

1. **Assuming base64 means secure.** Base64 is visible to anyone with
   `kubectl get secret -o yaml`. It is a transport encoding, not a security
   measure. Always enable encryption at rest for production clusters.

2. **Using env vars for secrets that rotate frequently.** If credentials
   change regularly (e.g., every 24 hours), use volume mounts instead.
   Volume-mounted secrets are updated automatically; env vars require a Pod
   restart.

3. **Confusing ConfigMap update with Pod update.** Updating a ConfigMap does
   not automatically restart Pods. Only volume-mounted consumers see the
   change (after the sync interval). Env var consumers require explicit
   restart.

4. **Not enabling encryption at rest.** Even Secrets are stored in plain
   text in etcd by default. A compromised etcd backup exposes all secrets.
   Encryption at rest (via `EncryptionConfiguration` or a KMS provider) is
   a baseline security requirement.

5. **Over-classifying as Secret.** Putting non-sensitive config in Secrets
   adds operational overhead (RBAC, encryption, rotation) for no security
   benefit. Classify by actual sensitivity, not by caution.
