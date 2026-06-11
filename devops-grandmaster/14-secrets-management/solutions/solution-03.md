# Solution 03: Kubernetes Secrets with Encryption at Rest

## Part A: Create a Kubernetes Secret

### Imperative approach

```bash
kubectl create secret generic app-secrets \
  --from-literal=db-password='super-secret-db-pass-2024!' \
  --from-literal=api-key='sk-live-a1b2c3d4e5f6g7h8'
```

### Declarative approach

First, encode the values:

```bash
echo -n 'super-secret-db-pass-2024!' | base64
# Output: c3VwZXItc2VjcmV0LWRiLXBhc3MtMjAyNCE=

echo -n 'sk-live-a1b2c3d4e5f6g7h8' | base64
# Output: c2stbGl2ZS1hYmMxMjNkNGU1ZjZnN2g4
```

Then create the manifest:

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: app-secrets
type: Opaque
data:
  db-password: c3VwZXItc2VjcmV0LWRiLXBhc3MtMjAyNCE=
  api-key: c2stbGl2ZS1hYmMxMjNkNGU1ZjZnN2g4
```

Apply it:

```bash
kubectl apply -f secret.yaml
```

### Why this works

- The imperative approach lets `kubectl` handle the base64 encoding automatically. The `--from-literal` flag takes plaintext values.
- The declarative approach requires you to base64-encode values yourself because the YAML `data` field expects base64.
- `echo -n` is critical. Without `-n`, `echo` appends a newline, and the base64 encoding includes the newline character, causing password mismatches.
- `type: Opaque` is the default type for arbitrary key-value secrets. Other types exist (e.g., `kubernetes.io/tls` for TLS certificates) but are not needed here.

### Common mistakes

- Using `echo` without `-n` and getting a password with an extra newline. This is the single most common Kubernetes secret bug.
- Putting `stringData` instead of `data` in the YAML and then base64-encoding the values. `stringData` accepts plaintext and encodes automatically -- `data` expects pre-encoded values.
- Forgetting `kubectl create secret` is idempotent for new secrets but fails if the secret already exists. Use `kubectl create secret --dry-run=client -o yaml | kubectl apply -f -` to make it idempotent.

---

## Part B: Verify the Default (Insecure) Behavior

```bash
# Retrieve the secret in YAML format
kubectl get secret app-secrets -o yaml
```

Output:

```yaml
apiVersion: v1
data:
  api-key: c2stbGl2ZS1hYmMxMjNkNGU1ZjZnN2g4
  db-password: c3VwZXItc2VjcmV0LWRiLXBhc3MtMjAyNCE=
kind: Secret
metadata:
  name: app-secrets
  namespace: default
type: Opaque
```

Decode the values:

```bash
echo 'c3VwZXItc2VjcmV0LWRiLXBhc3MtMjAyNCE=' | base64 -d
# Output: super-secret-db-pass-2024!

echo 'c2stbGl2ZS1hYmMxMjNkNGU1ZjZnN2g4' | base64 -d
# Output: sk-live-a1b2c3d4e5f6g7h8
```

### Why base64 is not encryption

Base64 is an encoding scheme that converts binary data to ASCII text. It is reversible by anyone who knows the algorithm (which is public). It provides zero confidentiality -- it is the equivalent of writing your password in a simple substitution cipher. The reason Kubernetes uses base64 is not security; it is because YAML cannot reliably represent arbitrary binary data. The `data` field requires a text-safe encoding.

### Common mistakes

- Assuming that "encoded" means "encrypted." Base64 encoding is trivially reversible with a one-line command.
- Thinking that Kubernetes Secrets are secure by default. They are not. Without encryption at rest, anyone with access to etcd (or a backup of etcd) can read every secret in the cluster.
- Believing that RBAC protects secrets sufficiently. RBAC controls API access, but etcd data is stored on disk and accessible to anyone with filesystem access to the etcd nodes.

---

## Part C: Configure Encryption at Rest

### Generate the encryption key

```bash
head -c 32 /dev/urandom | base64
# Example output: aF3jK9mN2pQ7rS1tU5vW8xY0zA4bC6dE=
```

### EncryptionConfiguration manifest

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
              secret: aF3jK9mN2pQ7rS1tU5vW8xY0zA4bC6dE=
      - identity: {}
```

### Why this works

- `aescbc` uses AES-CBC with PKCS#7 padding, providing strong symmetric encryption. It is the most secure built-in provider.
- The `secret` field must be a base64-encoded 32-byte key (256 bits). `head -c 32 /dev/urandom` generates 32 random bytes, and `base64` encodes them.
- `identity: {}` is a passthrough provider that reads unencrypted secrets. It must be listed last so that existing unencrypted secrets can still be read after encryption is enabled.
- The `resources` field specifies which Kubernetes resource types to encrypt. Only `secrets` needs encryption -- other resources (ConfigMaps, Deployments) are not sensitive.
- When the API server writes a secret, it tries providers in order: first `aescbc` (encrypts), then `identity` (no-op). When reading, it tries each provider until one succeeds: `aescbc` first (decrypts), then `identity` (reads unencrypted).

### Common mistakes

- Using a key shorter than 32 bytes. The `aescbc` provider requires exactly 32 bytes (256 bits). Shorter keys cause the API server to fail to start.
- Storing the encryption key in etcd itself. This creates a circular dependency -- if etcd is compromised, the key is too. Store the key in a separate, protected location (e.g., a hardware security module or a file on the control plane node with restricted permissions).
- Forgetting the `identity` provider. Without it, the API server cannot read any secrets that were created before encryption was enabled. You must encrypt existing secrets after enabling the configuration.
- Not rotating encryption keys. Over time, keys should be rotated. The process involves adding a new key, re-encrypting all secrets, and then removing the old key.

---

## Part D: Deploy a Pod That Consumes Secrets Two Ways

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: secret-consumer
spec:
  replicas: 1
  selector:
    matchLabels:
      app: secret-consumer
  template:
    metadata:
      labels:
        app: secret-consumer
    spec:
      containers:
        - name: app
          image: busybox:latest
          command:
            - sh
            - -c
            - |
              echo "=== Secret from volume mount ==="
              if [ -f /etc/secrets/db-password ]; then
                DB_PW=$(cat /etc/secrets/db-password)
                echo "db-password file exists, length: ${#DB_PW}"
              else
                echo "ERROR: db-password file not found"
              fi
              if [ -f /etc/secrets/api-key ]; then
                API_K=$(cat /etc/secrets/api-key)
                echo "api-key file exists, length: ${#API_K}"
              else
                echo "ERROR: api-key file not found"
              fi

              echo ""
              echo "=== Secret from environment variable ==="
              if [ -n "$DB_PASSWORD" ]; then
                echo "DB_PASSWORD env var exists, length: ${#DB_PASSWORD}"
              else
                echo "ERROR: DB_PASSWORD env var not set"
              fi

              echo ""
              echo "=== Verification ==="
              echo "File and env var should have the same length."
              echo "Volume mount length: ${#DB_PW}"
              echo "Env var length: ${#DB_PASSWORD}"
              if [ "${#DB_PW}" = "${#DB_PASSWORD}" ]; then
                echo "PASS: Lengths match"
              else
                echo "FAIL: Lengths differ"
              fi

              # Keep the pod running
              sleep infinity
          env:
            - name: DB_PASSWORD
              valueFrom:
                secretKeyRef:
                  name: app-secrets
                  key: db-password
          volumeMounts:
            - name: secret-volume
              mountPath: /etc/secrets
              readOnly: true
      volumes:
        - name: secret-volume
          secret:
            secretName: app-secrets
```

### Why this works

- The `env:` section with `valueFrom.secretKeyRef` injects a single key from the Secret as an environment variable. This is useful when the application expects an env var.
- The `volumes:` section with `secret` type mounts all keys from the Secret as files. Each key becomes a file at the mount path. This is useful when the application reads secrets from files.
- `readOnly: true` prevents the container from modifying the secret files.
- The `busybox` image is used because it is small and includes `sh`, `cat`, and `sleep`.
- `sleep infinity` keeps the pod running so you can exec into it for further testing.

### Common mistakes

- Using `env:` for all secrets. Environment variables are visible in `kubectl describe pod` output and in `/proc/<pid>/environ` inside the container. Volume mounts are more secure because they are only accessible to the process that reads the file.
- Forgetting `readOnly: true`. Without it, a compromised container could modify the secret files.
- Not specifying `key` in `secretKeyRef`. If the Secret has multiple keys, you must specify which one to inject.
- Using `sleep infinity` in production. This is for testing only. In production, the pod should run the actual application.

---

## Part E: Verification Script

```bash
#!/bin/bash
# verify-secrets.sh -- Verify Kubernetes secret setup
set -euo pipefail

SECRET_NAME="app-secrets"
EXPECTED_DB_PW="super-secret-db-pass-2024!"
EXPECTED_API_KEY="sk-live-a1b2c3d4e5f6g7h8"
PASS=0
FAIL=0

echo "=== Check 1: Secret exists ==="
if kubectl get secret "$SECRET_NAME" &>/dev/null; then
    echo "PASS: Secret '$SECRET_NAME' exists"
    PASS=$((PASS + 1))
else
    echo "FAIL: Secret '$SECRET_NAME' not found"
    FAIL=$((FAIL + 1))
    exit 1
fi

echo ""
echo "=== Check 2: Secret has correct number of keys ==="
KEYS=$(kubectl get secret "$SECRET_NAME" -o jsonpath='{.data}' | jq -r 'keys | length')
if [ "$KEYS" = "2" ]; then
    echo "PASS: Secret has 2 keys"
    PASS=$((PASS + 1))
else
    echo "FAIL: Secret has $KEYS keys, expected 2"
    FAIL=$((FAIL + 1))
fi

echo ""
echo "=== Check 3: Decoded values match expected ==="
DB_PW=$(kubectl get secret "$SECRET_NAME" -o jsonpath='{.data.db-password}' | base64 -d)
API_KEY=$(kubectl get secret "$SECRET_NAME" -o jsonpath='{.data.api-key}' | base64 -d)
if [ "$DB_PW" = "$EXPECTED_DB_PW" ]; then
    echo "PASS: db-password matches"
    PASS=$((PASS + 1))
else
    echo "FAIL: db-password does not match"
    FAIL=$((FAIL + 1))
fi
if [ "$API_KEY" = "$EXPECTED_API_KEY" ]; then
    echo "PASS: api-key matches"
    PASS=$((PASS + 1))
else
    echo "FAIL: api-key does not match"
    FAIL=$((FAIL + 1))
fi

echo ""
echo "=== Check 4: Pod can access secrets ==="
POD=$(kubectl get pod -l app=secret-consumer -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)
if [ -z "$POD" ]; then
    echo "SKIP: No pod found with label app=secret-consumer"
else
    # Check volume mount
    VOL_RESULT=$(kubectl exec "$POD" -- cat /etc/secrets/db-password 2>/dev/null | tr -d '\n')
    if [ "$VOL_RESULT" = "$EXPECTED_DB_PW" ]; then
        echo "PASS: Pod can read db-password from volume mount"
        PASS=$((PASS + 1))
    else
        echo "FAIL: Pod cannot read db-password from volume mount"
        FAIL=$((FAIL + 1))
    fi

    # Check env var
    ENV_RESULT=$(kubectl exec "$POD" -- printenv DB_PASSWORD 2>/dev/null | tr -d '\n')
    if [ "$ENV_RESULT" = "$EXPECTED_DB_PW" ]; then
        echo "PASS: Pod has DB_PASSWORD env var set correctly"
        PASS=$((PASS + 1))
    else
        echo "FAIL: Pod does not have DB_PASSWORD env var or value is wrong"
        FAIL=$((FAIL + 1))
    fi
fi

echo ""
echo "=== Check 5: Secret values not in pod YAML ==="
POD_YAML=$(kubectl get pod -l app=secret-consumer -o yaml 2>/dev/null)
if echo "$POD_YAML" | grep -q "super-secret-db-pass-2024!"; then
    echo "FAIL: Plaintext secret value found in pod YAML"
    FAIL=$((FAIL + 1))
else
    echo "PASS: Plaintext secret value NOT in pod YAML"
    PASS=$((PASS + 1))
fi

echo ""
echo "==============================="
echo "Results: $PASS passed, $FAIL failed"
echo "==============================="
```

### Why this works

- Check 1 verifies the secret exists in the cluster.
- Check 2 uses `jsonpath` to extract the `data` field and counts keys with `jq`.
- Check 3 decodes the base64 values and compares them to expected plaintext.
- Check 4 uses `kubectl exec` to read the secret from inside the pod (both volume mount and env var).
- Check 5 uses `grep` to verify that the plaintext password does not appear in the pod's YAML output. The pod spec should contain only the `secretKeyRef` reference, not the value.

### Common mistakes

- Not waiting for the pod to be ready before running checks 4 and 5. Use `kubectl wait --for=condition=ready pod -l app=secret-consumer` before the script.
- Using `grep` on the decoded value without considering special characters. The `!` in the password might cause issues in some shells. Using single quotes around the expected value prevents shell expansion.
- Not handling the case where the pod does not exist. The script should skip pod checks gracefully rather than failing.
