# Exercise 04: Automated Rotation with Vault

**Type:** Challenge
**Difficulty:** Advanced
**Estimated Time:** 60 minutes

## Objective

Integrate HashiCorp Vault's Transit and Key/Value secrets engines to build an automated
rotation pipeline where Vault manages the cryptographic lifecycle of secrets, and your
application retrieves them on demand.

## Background

HashiCorp Vault provides several mechanisms for secret management:

- **KV Secrets Engine**: Stores static key-value secrets with versioning.
- **Transit Engine**: Provides encryption-as-a-service, manages encryption keys with
  automatic rotation.
- **Dynamic Secrets**: Generates unique, short-lived credentials on demand (e.g.,
  database credentials).
- **Lease and Renewal**: Secrets have TTLs and can be renewed or revoked.

This exercise uses Docker to run a local Vault server and interacts with it via HTTP API.

## Instructions

### Step 1: Start Vault in Dev Mode

```bash
docker run --cap-add=IPC_LOCK -d \
  --name vault-dev \
  -p 8200:8200 \
  -e 'VAULT_DEV_ROOT_TOKEN_ID=myroot' \
  -e 'VAULT_DEV_LISTEN_ADDRESS=0.0.0.0:8200' \
  hashicorp/vault:1.15

export VAULT_ADDR='http://127.0.0.1:8200'
export VAULT_TOKEN='myroot'
```

Verify it is running:

```bash
curl -s $VAULT_ADDR/v1/sys/health | jq
```

### Step 2: Enable and Configure Secrets Engines

Enable the KV v2 engine and Transit engine:

```bash
# KV v2 for application config secrets
vault secrets enable -path=secret kv-v2

# Transit for encryption key management
vault secrets enable transit

# Create an encryption key
vault write transit/keys/app-data-key type=aes256-gcm96
```

### Step 3: Build the Rust Client

Create a Rust project that talks to Vault:

```bash
cargo new vault_rotator
cd vault_rotator
```

**Dependencies:**

```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
base64 = "0.21"
chrono = "0.4"
```

Implement a `VaultClient` with these methods:

1. `new(addr: &str, token: &str) -> Self` -- constructor storing address and token.

2. `write_secret(&self, path: &str, data: serde_json::Value) -> Result<()>` -- write a
   secret to the KV v2 engine.

3. `read_secret(&self, path: &str) -> Result<serde_json::Value>` -- read the latest
   version of a secret.

4. `read_secret_version(&self, path: &str, version: u32) -> Result<serde_json::Value>`
   -- read a specific version.

5. `list_secret_versions(&self, path: &str) -> Result<Vec<u32>>` -- list all versions of
   a secret.

6. `encrypt(&self, key_name: &str, plaintext: &str) -> Result<String>` -- encrypt using
   the Transit engine, return base64 ciphertext.

7. `decrypt(&self, key_name: &str, ciphertext: &str) -> Result<String>` -- decrypt using
   the Transit engine.

8. `rotate_key(&self, key_name: &str) -> Result<()>` -- rotate the Transit encryption
   key.

### Step 4: Implement Rotation Logic

Build a `VaultSecretRotator` that uses the `VaultClient`:

1. Stores application secrets (like database credentials) in KV v2 with versioning.
2. On rotation: writes a new version of the secret.
3. Consumers read the latest version.
4. Old versions remain readable for a configurable grace period (Vault handles this via
   `max_versions` and `delete_version_after`).

Build a second component `VaultEncryptionRotator`:

1. Uses the Transit engine to encrypt sensitive data at rest.
2. On rotation: calls `rotate_key` to create a new encryption key version.
3. New encryptions use the latest key version.
4. Decryption works with any version (Vault keeps old key versions).

### Step 5: Integration Test

Write a test that:

1. Writes a secret `database/password` with value `{"user": "app", "pass": "old_pass"}`.
2. Reads it back and asserts correctness.
3. Rotates: writes `{"user": "app", "pass": "new_pass"}`.
4. Reads the latest version and asserts it is the new password.
5. Reads version 1 and asserts it is still the old password.
6. Encrypts a value with the Transit engine.
7. Rotates the Transit key.
8. Decrypts the original ciphertext (still works with old key version).
9. Encrypts a new value and decrypts it (uses new key version).

### Step 6: Cleanup

```bash
docker stop vault-dev && docker rm vault-dev
```

## Success Criteria

- [ ] Vault is running and accessible on port 8200.
- [ ] `VaultClient` can read, write, and list secret versions in KV v2.
- [ ] `VaultClient` can encrypt, decrypt, and rotate keys in Transit.
- [ ] `VaultSecretRotator` writes new versions without breaking reads of old versions.
- [ ] `VaultEncryptionRotator` rotates keys while maintaining decryptability of old
      ciphertexts.
- [ ] The integration test passes end-to-end.

## Hints

<details>
<summary>Hint 1: KV v2 API paths</summary>

KV v2 uses different paths than v1. Write to `/v1/secret/data/{path}` and read from the
same path. The response wraps data in a `data.data` nested structure. List versions via
`/v1/secret/metadata/{path}`.

</details>

<details>
<summary>Hint 2: Transit encrypt API</summary>

POST to `/v1/transit/encrypt/{key_name}` with body:

```json
{ "plaintext": "<base64-encoded-plaintext>" }
```

The response contains `data.ciphertext`. For decryption, POST to
`/v1/transit/decrypt/{key_name}`.

</details>

<details>
<summary>Hint 3: Vault token header</summary>

All requests need the header `X-Vault-Token: {your_token}`.

</details>

<details>
<summary>Hint 4: Handling Vault being down</summary>

Implement retries with exponential backoff in your `VaultClient`. If Vault is unreachable,
the application should use a cached secret (the last successfully read value) rather than
crashing. This is called a "resilience pattern."

</details>
