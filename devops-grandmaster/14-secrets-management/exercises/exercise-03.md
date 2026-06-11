# Exercise 03: Kubernetes Secrets with Encryption at Rest

**Type:** Independent
**Time:** 45 minutes
**Difficulty:** Medium

## Objective

Create Kubernetes Secrets, configure encryption at rest using an EncryptionConfiguration resource, and wire secrets into a pod using both environment variable injection and volume mounts. This exercise trains you to handle secrets in Kubernetes beyond the default base64-encoded-but-not-encrypted behavior.

## Scenario

Your team is deploying a web application to a Kubernetes cluster. The application needs a database password and an API key. You have learned that Kubernetes Secrets are base64-encoded by default -- which is not encryption. Anyone with access to etcd can read them. Your task is to create the secrets, enable encryption at rest, and verify that the secrets are actually encrypted in etcd.

## Tasks

### Part A: Create a Kubernetes Secret

Create a Kubernetes Secret named `app-secrets` containing two keys:

1. `db-password` with value `super-secret-db-pass-2024!`
2. `api-key` with value `sk-live-a1b2c3d4e5f6g7h8`

Use both the imperative approach (`kubectl create secret`) and the declarative approach (YAML manifest). For the YAML approach, you must base64-encode the values yourself.

<details>
<summary>Hint</summary>

For the imperative approach: `kubectl create secret generic app-secrets --from-literal=db-password='value' --from-literal=api-key='value'`. For the YAML approach, run `echo -n 'value' | base64` to encode each value. The `echo -n` is critical -- without it, a trailing newline is included in the encoding.

</details>

### Part B: Verify the Default (Insecure) Behavior

Retrieve the secret you created and decode it. Demonstrate that by default, Kubernetes Secrets are only base64-encoded, not encrypted.

1. Use `kubectl get secret` to retrieve the secret in YAML format.
2. Decode the base64 values to show the plaintext.
3. Explain why base64 is not encryption and what this means for security.

<details>
<summary>Hint</summary>

Use `kubectl get secret app-secrets -o yaml`. The values under `data:` are base64-encoded. Pipe through `base64 -d` to decode. Note that base64 is an encoding scheme, not an encryption scheme -- it provides zero confidentiality.

</details>

### Part C: Configure Encryption at Rest

Write an `EncryptionConfiguration` manifest that encrypts Kubernetes secrets at rest in etcd. The configuration should:

1. Use `aescbc` as the encryption provider (strongest built-in option).
2. Include a base64-encoded 32-byte encryption key.
3. Set the `resources` field to encrypt `secrets`.
4. List `identity` as a fallback provider for reading unencrypted secrets.

Generate the encryption key using `openssl` and show the full EncryptionConfiguration YAML.

<details>
<summary>Hint</summary>

Generate a key with `head -c 32 /dev/urandom | base64`. The EncryptionConfiguration has a `resources` list, each with a `providers` list. The first provider should be `aescbc` with a `keys` array containing one entry with `name` and `secret` fields. The last provider should be `identity` so that existing unencrypted secrets can still be read.

</details>

### Part D: Deploy a Pod That Consumes Secrets Two Ways

Write a Kubernetes Deployment manifest for an nginx pod that consumes the `app-secrets` Secret in two ways:

1. **Volume mount:** Mount the secret as a file at `/etc/secrets/` so the files `/etc/secrets/db-password` and `/etc/secrets/api-key` exist.
2. **Environment variable:** Inject `db-password` into the pod as the environment variable `DB_PASSWORD`.

The pod should run a command at startup that reads both the file and the environment variable and prints confirmation (not the actual values -- just lengths or masked values).

<details>
<summary>Hint</summary>

For volume mounts, use a `secret` volume type in `volumes:` and mount it in `volumeMounts:`. For environment variables, use `valueFrom.secretKeyRef` in the `env:` section. The startup command can use `sh -c` with a script that reads the file with `cat` and the env var with `$DB_PASSWORD`.

</details>

### Part E: Write a Verification Script

Write a shell script (`verify-secrets.sh`) that checks:

1. The secret exists in the cluster.
2. The secret has the correct number of keys.
3. The decoded values match the expected values.
4. The pod can access the secret via both the volume mount and the environment variable.
5. The secret is not visible in `kubectl get pods -o yaml` (only the reference should appear, not the value).

<details>
<summary>Hint</summary>

Use `kubectl get secret app-secrets -o jsonpath='{.data}'` to check keys. Use `kubectl exec` to run commands inside the pod. Use `grep` to check that `kubectl get pods -o yaml` does not contain the actual password string.

</details>

## Success Criteria

- [ ] A Kubernetes Secret named `app-secrets` exists with two keys.
- [ ] You can demonstrate that default secrets are only base64-encoded.
- [ ] You wrote a valid `EncryptionConfiguration` with `aescbc` provider and a 32-byte key.
- [ ] A pod consumes the secret via both volume mount and environment variable.
- [ ] The pod can read the secret values from both locations.
- [ ] The verification script passes all five checks.

## What You Should Understand After This Exercise

Kubernetes Secrets are base64-encoded by default, which provides no confidentiality. Encryption at rest using `EncryptionConfiguration` with `aescbc` is the minimum requirement for production clusters. Secrets can be consumed as files (volume mounts) or as environment variables, but volume mounts are generally preferred because environment variables are visible in `kubectl describe` output and container inspection. The encryption key itself must be protected -- if it is stored in the same etcd cluster without additional protection, you have moved the problem rather than solved it.
