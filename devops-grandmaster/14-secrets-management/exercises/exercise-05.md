# Exercise 05: Secrets Management Strategy with Vault

**Type:** Integration
**Time:** 60 minutes
**Difficulty:** Hard

## Objective

Design and implement a complete secrets management strategy using HashiCorp Vault as the central secret store, integrated with a Kubernetes cluster. This exercise combines Docker, Kubernetes, networking, and security concepts from previous modules into a production-grade secrets architecture.

## Scenario

Your company runs a microservices platform on Kubernetes. There are five services, each needing different secrets: database credentials, API keys, TLS certificates, and encryption keys. Currently, secrets are scattered across Kubernetes Secrets, environment variables in CI/CD pipelines, and a shared password manager. The CTO has mandated a centralized secrets strategy. Your task is to design the architecture and implement a proof-of-concept using Vault.

## Architecture

```
+------------------+     +------------------+     +------------------+
|  CI/CD Pipeline  |     |   Kubernetes     |     |    Vault         |
|                  |     |   Cluster        |     |    Server        |
|  1. Build image  |     |                  |     |                  |
|  2. Deploy pod   |---->|  3. Init container|---->|  4. Authenticate |
|                  |     |     fetches secret|    |     via K8s auth |
+------------------+     |  5. App reads    |     |  6. Return secret|
                         |     secret file  |     |     (with lease) |
                         +------------------+     +------------------+
                                                       |
                                                       v
                                                 +------------------+
                                                 |  Audit Log       |
                                                 |  (who accessed   |
                                                 |   what, when)    |
                                                 +------------------+
```

## Tasks

### Part A: Design the Secrets Architecture

Before writing any code, design the architecture for your secrets management strategy. Produce a document that covers:

1. **Secret classification:** Create a table listing the types of secrets in your platform (database creds, API keys, TLS certs, encryption keys) with their rotation frequency, access pattern, and storage method.
2. **Auth flow:** Describe how a Kubernetes pod authenticates to Vault without embedding a Vault token. Research and explain the Kubernetes auth method.
3. **Secret delivery:** Describe two approaches for getting secrets from Vault into the application: (a) init container that writes to a shared volume, and (b) sidecar that proxies secret access.
4. **Audit and access control:** Describe how you would use Vault policies to ensure each service can only access its own secrets.

<details>
<summary>Hint</summary>

Vault's Kubernetes auth method lets pods authenticate using their Kubernetes ServiceAccount token. Vault verifies the token with the Kubernetes API and maps the ServiceAccount to a Vault role and policy. No static Vault token is needed. For secret delivery, an init container runs before the app, fetches the secret, writes it to an emptyDir volume, and exits. A sidecar runs alongside the app and refreshes the secret before it expires.

</details>

### Part B: Deploy Vault in Dev Mode

Write a `docker-compose.yml` or Kubernetes manifest that deploys Vault in dev mode for local testing. The deployment should:

1. Run Vault with a known root token for initial setup.
2. Expose the Vault API on a predictable address.
3. Include a health check that waits for Vault to be ready before dependent services start.

<details>
<summary>Hint</summary>

For Docker Compose: use the `hashicorp/vault:latest` image with `VAULT_DEV_ROOT_TOKEN_ID=myroot` and `VAULT_DEV_LISTEN_ADDRESS=0.0.0.0:8200`. Add `cap_add: IPC_LOCK` for production-like memory locking. For Kubernetes: use a Deployment with environment variables and a readiness probe hitting `/v1/sys/health`.

</details>

### Part C: Configure Vault Policies and Auth

Write a shell script (`setup-vault.sh`) that configures Vault for your platform. The script should:

1. Enable the KV v2 secrets engine at `secret/`.
2. Create a policy for the "app" service that can only read secrets at `secret/data/app/*`.
3. Create a policy for the "worker" service that can only read secrets at `secret/data/worker/*`.
4. Enable the Kubernetes auth method.
5. Create roles that map Kubernetes ServiceAccounts to Vault policies.
6. Store sample secrets for each service.

<details>
<summary>Hint</summary>

Use `vault secrets enable -path=secret kv-v2`. Write policies in HCL: `path "secret/data/app/*" { capabilities = ["read"] }`. Enable Kubernetes auth with `vault auth enable kubernetes` and configure it with `vault write auth/kubernetes/config kubernetes_host="https://kubernetes.default.svc"`. Create roles with `vault write auth/kubernetes/role/app bound_service_account_names=app bound_service_account_namespaces=default policies=app`.

</details>

### Part D: Write an Init Container Pattern

Write a Kubernetes Deployment manifest that uses an init container to fetch secrets from Vault before the main application starts. The deployment should:

1. Use a ServiceAccount that is mapped to a Vault role.
2. Include an init container that authenticates to Vault using the ServiceAccount token.
3. The init container reads the secret and writes it to a shared `emptyDir` volume.
4. The main container reads the secret from the volume mount.
5. The main container does not have access to Vault directly (principle of least privilege).

<details>
<summary>Hint</summary>

The init container should use `curl` or the Vault CLI. It mounts the ServiceAccount token at `/var/run/secrets/kubernetes.io/serviceaccount/token`. It sends this token to Vault's Kubernetes auth login endpoint. It then reads the secret and writes it to a file in a shared volume. The main container mounts this volume at `/run/secrets/vault/`.

</details>

### Part E: Implement Secret Rotation Awareness

Design and document a strategy for handling Vault secret leases and rotation. Your strategy must answer:

1. What happens when a Vault secret lease expires while the application is running?
2. How does the sidecar pattern solve this compared to the init container pattern?
3. Write a sidecar container definition that watches a Vault secret and refreshes it before expiry.
4. How would you integrate this with the Kubernetes Secrets rotation approach from Exercise 04?

<details>
<summary>Hint</summary>

Vault secrets have a lease duration. When it expires, Vault may revoke the secret (especially for dynamic secrets). The init container pattern only fetches the secret once -- if it expires, the app breaks. The sidecar pattern runs continuously, watches the lease TTL, and refreshes the secret before it expires. The sidecar writes the refreshed secret to the same shared volume, and the application either watches for file changes or restarts.

</details>

## Success Criteria

- [ ] You produced a secret classification table with rotation frequency and access patterns.
- [ ] You can explain Vault's Kubernetes auth method without referencing notes.
- [ ] A working `docker-compose.yml` or Kubernetes manifest deploys Vault locally.
- [ ] A `setup-vault.sh` script configures policies, auth, and sample secrets.
- [ ] A Kubernetes Deployment uses an init container to fetch secrets from Vault.
- [ ] You described the sidecar pattern for secret lease renewal with a container definition.
- [ ] You can articulate the trade-offs between init container and sidecar approaches.

## What You Should Understand After This Exercise

A production secrets management strategy is not just "use Vault." It requires designing auth flows that do not embed static tokens, policies that enforce least privilege, delivery mechanisms that handle secret lifecycle (creation, rotation, expiry, revocation), and audit logging that tracks every access. The init container pattern is simple but does not handle rotation. The sidecar pattern handles rotation but adds complexity. Kubernetes auth eliminates the need for static Vault tokens but requires careful ServiceAccount-to-policy mapping. Every secret in your platform should have a defined owner, rotation schedule, and access policy -- secrets without owners become permanent secrets, and permanent secrets are a security incident waiting to happen.
