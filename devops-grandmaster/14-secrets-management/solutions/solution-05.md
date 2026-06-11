# Solution 05: Secrets Management Strategy with Vault

## Part A: Design the Secrets Architecture

### Secret classification table

| Secret Type | Example | Rotation Frequency | Access Pattern | Storage Method |
|-------------|---------|-------------------|----------------|----------------|
| **Database credentials** | PostgreSQL username/password | Every 30 days | Long-lived connection pool | Vault KV v2 or dynamic database secrets engine |
| **Third-party API keys** | Stripe, Twilio, AWS keys | Every 90 days or on compromise | Per-request or cached | Vault KV v2 |
| **TLS certificates** | Service mesh mTLS certs | Every 90 days (auto-renewed) | Loaded at startup, cached in memory | Vault PKI secrets engine |
| **Encryption keys** | Application-level data encryption | Every 365 days | Per-operation | Vault Transit secrets engine |
| **Internal service tokens** | Service-to-service auth tokens | Every 24 hours | Per-request | Vault AppRole or Kubernetes auth |

### Auth flow: Kubernetes auth method

```
+------------------+     +------------------+     +------------------+
|   Kubernetes     |     |   Vault          |     |   Vault Policy   |
|   Pod            |     |   Server         |     |                  |
|                  |     |                  |     |  path: secret/   |
|  1. Read SA token|     |  2. Verify token |     |    data/app/*    |
|     from         |---->|     with K8s API |---->|    cap: [read]   |
|     /var/run/    |     |  3. Check role   |     +------------------+
|     secrets/     |     |     binding      |
|     kubernetes/  |     |  4. Issue Vault  |
|     serviceacct/ |     |     token with   |
|     token        |     |     policy scope |
+------------------+     +------------------+
```

**How it works:**

1. The pod reads its Kubernetes ServiceAccount token from the standard mount path (`/var/run/secrets/kubernetes.io/serviceaccount/token`).
2. The pod sends this token to Vault's Kubernetes auth login endpoint (`/auth/kubernetes/login`).
3. Vault validates the token by calling the Kubernetes TokenReview API, confirming the token is valid and belongs to the expected ServiceAccount.
4. Vault maps the ServiceAccount to a pre-configured role, which is bound to a Vault policy. Vault issues a short-lived Vault token scoped to that policy.
5. The pod uses the Vault token to read secrets. The policy ensures it can only access secrets it is authorized for.

**Why this is secure:** No static Vault tokens are embedded in pods or secrets. The ServiceAccount token is automatically rotated by Kubernetes. The Vault token is scoped to the minimum required access. Every access is auditable in Vault's audit log.

### Secret delivery approaches

**Approach A: Init container**

```
+------------------+
|   Pod            |
|                  |
|  +-----------+   |     +------------------+
|  | Init      |---+---->| Vault Server     |
|  | Container |         |                  |
|  | (curl/CLI)|<--------| Return secret    |
|  +-----------+         +------------------+
|       |
|       | Write to emptyDir
|       v
|  +-----------+
|  | emptyDir  |
|  | /secrets/ |
|  +-----------+
|       ^
|       | Read from emptyDir
|  +-----------+
|  | App       |
|  | Container |
|  +-----------+
+------------------+
```

The init container runs before the app container. It authenticates to Vault, reads the secret, writes it to a shared emptyDir volume, and exits. The app container starts after the init container completes and reads the secret from the volume.

**Pros:** Simple, one-time operation, app does not need Vault client libraries.
**Cons:** Secret is fetched once at startup. If the secret expires or is rotated, the pod must be restarted.

**Approach B: Sidecar proxy**

```
+------------------+
|   Pod            |
|                  |
|  +-----------+   |     +------------------+
|  | Sidecar   |---+---->| Vault Server     |
|  | Container |         |                  |
|  | (agent)   |<--------| Refresh secret   |
|  +-----------+         | before expiry    |
|       |                +------------------+
|       | Write to emptyDir (refreshed)
|       v
|  +-----------+
|  | emptyDir  |
|  | /secrets/ |
|  +-----------+
|       ^
|       | Read from emptyDir
|  +-----------+
|  | App       |
|  | Container |
|  +-----------+
+------------------+
```

The sidecar runs alongside the app container. It authenticates to Vault, reads the secret, writes it to a shared volume, and monitors the lease TTL. Before the lease expires, it refreshes the secret and updates the file.

**Pros:** Handles secret rotation and expiry automatically. No pod restart needed.
**Cons:** More complex, app must handle file changes (or restart on change), adds a container to every pod.

### Audit and access control with Vault policies

```hcl
# Policy for the "app" service
path "secret/data/app/*" {
  capabilities = ["read"]
}

path "secret/metadata/app/*" {
  capabilities = ["read", "list"]
}

# Policy for the "worker" service
path "secret/data/worker/*" {
  capabilities = ["read"]
}

path "secret/metadata/worker/*" {
  capabilities = ["read", "list"]
}
```

Each service gets a dedicated policy that restricts access to its own secret path. The `data` path is for reading secret values; the `metadata` path is for listing versions and metadata. No service can read another service's secrets. Vault's audit log records every access attempt, including the authenticated identity, the path accessed, and the timestamp.

### Common mistakes

- Giving all services the same policy with a wildcard path (`secret/data/*`). This violates least privilege and means a compromised service can read every secret in the cluster.
- Not enabling audit logging. Without it, you cannot detect unauthorized access or investigate incidents.
- Using the root token for application access. The root token should be used only for initial setup and sealed in a secure location. Application access should use role-based tokens with scoped policies.
- Forgetting to rotate the Vault unseal keys. In production, Vault should be initialized with key shares and a threshold, and the unseal keys should be distributed among trusted operators.

---

## Part B: Deploy Vault in Dev Mode

### Docker Compose

```yaml
version: '3.8'

services:
  vault:
    image: hashicorp/vault:latest
    cap_add:
      - IPC_LOCK
    environment:
      - VAULT_DEV_ROOT_TOKEN_ID=myroot
      - VAULT_DEV_LISTEN_ADDRESS=0.0.0.0:8200
    ports:
      - "8200:8200"
    healthcheck:
      test: ["CMD", "vault", "status", "-address=http://localhost:8200"]
      interval: 5s
      timeout: 3s
      retries: 10
      start_period: 5s
```

### Why this works

- `VAULT_DEV_ROOT_TOKEN_ID=myroot` starts Vault in dev mode with a known root token. This is convenient for testing but must never be used in production.
- `VAULT_DEV_LISTEN_ADDRESS=0.0.0.0:8200` makes Vault listen on all interfaces, not just localhost. This is necessary for other containers to reach it.
- `IPC_LOCK` capability allows Vault to lock memory (mlock), preventing secrets from being swapped to disk. In dev mode this is optional but good practice.
- The health check uses `vault status` to verify that Vault is unsealed and responding. The `start_period` gives Vault time to initialize before health checks begin.

### Common mistakes

- Using dev mode in production. Dev mode stores everything in memory, uses a single root token, and has no persistence. All secrets are lost when the container stops.
- Not setting `VAULT_DEV_LISTEN_ADDRESS` to `0.0.0.0`. The default (`127.0.0.1`) only allows connections from inside the container.
- Exposing port 8200 to the public internet. In production, Vault should be behind a reverse proxy or accessible only from within the cluster network.

---

## Part C: Configure Vault Policies and Auth

```bash
#!/bin/bash
# setup-vault.sh -- Configure Vault for the secrets management platform
set -euo pipefail

export VAULT_ADDR='http://localhost:8200'
export VAULT_TOKEN='myroot'

echo "=== Enabling KV v2 secrets engine ==="
vault secrets enable -path=secret kv-v2 2>/dev/null || echo "KV v2 already enabled"

echo ""
echo "=== Creating policies ==="

# App service policy
vault policy write app - <<'EOF'
path "secret/data/app/*" {
  capabilities = ["read"]
}
path "secret/metadata/app/*" {
  capabilities = ["read", "list"]
}
EOF
echo "  Created 'app' policy"

# Worker service policy
vault policy write worker - <<'EOF'
path "secret/data/worker/*" {
  capabilities = ["read"]
}
path "secret/metadata/worker/*" {
  capabilities = ["read", "list"]
}
EOF
echo "  Created 'worker' policy"

echo ""
echo "=== Storing sample secrets ==="

vault kv put secret/app/database \
  username="appuser" \
  password="super-secret-db-pass-2024!" \
  host="postgres.default.svc.cluster.local" \
  port="5432" \
  dbname="myapp"
echo "  Stored secret/app/database"

vault kv put secret/app/api-keys \
  stripe_key="sk_live_a1b2c3d4e5f6g7h8" \
  twilio_sid="AC1234567890abcdef" \
  twilio_token="auth_token_12345"
echo "  Stored secret/app/api-keys"

vault kv put secret/worker/config \
  redis_url="redis://redis.default.svc.cluster.local:6379" \
  queue_name="default" \
  processing_secret="worker-secret-key-2024"
echo "  Stored secret/worker/config"

echo ""
echo "=== Enabling Kubernetes auth method ==="
vault auth enable kubernetes 2>/dev/null || echo "Kubernetes auth already enabled"

# In a real cluster, the kubernetes_host would be
# https://kubernetes.default.svc and the CA cert and token would be
# mounted from the ServiceAccount. For local testing with Docker:
vault write auth/kubernetes/config \
  kubernetes_host="https://kubernetes.default.svc" \
  disable_local_ca_jwt=true 2>/dev/null || echo "K8s auth config skipped (no cluster access)"

echo ""
echo "=== Creating Kubernetes auth roles ==="

vault write auth/kubernetes/role/app \
  bound_service_account_names=app \
  bound_service_account_namespaces=default \
  policies=app \
  ttl=1h \
  max_ttl=4h 2>/dev/null || echo "Role 'app' creation skipped (no cluster access)"

vault write auth/kubernetes/role/worker \
  bound_service_account_names=worker \
  bound_service_account_namespaces=default \
  policies=worker \
  ttl=1h \
  max_ttl=4h 2>/dev/null || echo "Role 'worker' creation skipped (no cluster access)"

echo ""
echo "=== Verification ==="
echo "Secrets stored:"
vault kv list secret/app/ 2>/dev/null || echo "  (listing requires authenticated access)"
vault kv list secret/worker/ 2>/dev/null || echo "  (listing requires authenticated access)"

echo ""
echo "Policies:"
vault policy list

echo ""
echo "Auth methods:"
vault auth list

echo ""
echo "=== Setup complete ==="
```

### Why this works

- `vault secrets enable -path=secret kv-v2` enables the KV version 2 secrets engine. The `2>/dev/null || echo "already enabled"` pattern makes the script idempotent -- it does not fail if the engine is already enabled.
- Policies use HCL syntax. The `path` field specifies the secret path, and `capabilities` lists allowed operations. `read` allows `vault kv get`, and `list` allows `vault kv list`.
- The `app` policy can only access `secret/data/app/*` paths. Even if the app is compromised, it cannot read `secret/worker/*` or `secret/data/shared/*`.
- Kubernetes auth roles bind a Kubernetes ServiceAccount (`bound_service_account_names`) in a specific namespace (`bound_service_account_namespaces`) to a Vault policy. When a pod with the `app` ServiceAccount authenticates, it receives a token scoped to the `app` policy.
- `ttl=1h` sets the default token lifetime to one hour. `max_ttl=4h` prevents the token from being renewed beyond four hours, forcing re-authentication.

### Common mistakes

- Using `kv` (v1) instead of `kv-v2`. KV v2 supports versioning, soft delete, and metadata. KV v1 does not. Most tutorials use v1 because it is simpler, but v2 is the standard.
- Not making the script idempotent. Running the script twice should not fail. Use `2>/dev/null || echo "already enabled"` for resources that can only be created once.
- Granting `create` or `update` capabilities in the policy when only `read` is needed. Least privilege means the application can read its secrets but cannot modify them.
- Forgetting `disable_local_ca_jwt=true` when Vault cannot reach the Kubernetes API directly. In a local Docker environment, Vault cannot verify Kubernetes tokens without cluster access.

---

## Part D: Write an Init Container Pattern

### ServiceAccount

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: app
  namespace: default
```

### Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: app
  namespace: default
spec:
  replicas: 3
  selector:
    matchLabels:
      app: myapp
  template:
    metadata:
      labels:
        app: myapp
    spec:
      serviceAccountName: app
      volumes:
        - name: vault-secrets
          emptyDir:
            medium: Memory  # tmpfs -- never touches disk
        - name: sa-token
          projected:
            sources:
              - serviceAccountToken:
                  path: token
                  expirationSeconds: 3600
                  audience: vault
      initContainers:
        - name: vault-init
          image: curlimages/curl:latest
          command:
            - sh
            - -c
            - |
              set -e

              VAULT_ADDR="${VAULT_ADDR:-http://vault.default.svc.cluster.local:8200}"
              SA_TOKEN=$(cat /var/run/secrets/vault/token)

              echo "Authenticating to Vault..."
              VAULT_TOKEN=$(curl -s \
                --request POST \
                --data "{\"role\": \"app\", \"jwt\": \"$SA_TOKEN\"}" \
                "$VAULT_ADDR/v1/auth/kubernetes/login" \
                | jq -r '.auth.client_token')

              if [ "$VAULT_TOKEN" = "null" ] || [ -z "$VAULT_TOKEN" ]; then
                echo "ERROR: Failed to authenticate to Vault"
                exit 1
              fi

              echo "Fetching database credentials..."
              DB_CREDS=$(curl -s \
                -H "X-Vault-Token: $VAULT_TOKEN" \
                "$VAULT_ADDR/v1/secret/data/app/database")

              echo "$DB_CREDS" | jq -r '.data.data.password' > /secrets/db-password
              echo "$DB_CREDS" | jq -r '.data.data.username' > /secrets/db-username
              echo "$DB_CREDS" | jq -r '.data.data.host' > /secrets/db-host

              echo "Fetching API keys..."
              API_KEYS=$(curl -s \
                -H "X-Vault-Token: $VAULT_TOKEN" \
                "$VAULT_ADDR/v1/secret/data/app/api-keys")

              echo "$API_KEYS" | jq -r '.data.data.stripe_key' > /secrets/stripe-key

              # Set restrictive permissions
              chmod 400 /secrets/*
              echo "Secrets written to /secrets/"
              ls -la /secrets/
          volumeMounts:
            - name: vault-secrets
              mountPath: /secrets
            - name: sa-token
              mountPath: /var/run/secrets/vault
              readOnly: true
      containers:
        - name: app
          image: myapp:1.0.0
          volumeMounts:
            - name: vault-secrets
              mountPath: /run/secrets/vault
              readOnly: true
          env:
            - name: DB_PASSWORD_FILE
              value: /run/secrets/vault/db-password
            - name: DB_USERNAME_FILE
              value: /run/secrets/vault/db-username
            - name: DB_HOST_FILE
              value: /run/secrets/vault/db-host
            - name: STRIPE_KEY_FILE
              value: /run/secrets/vault/stripe-key
```

### Why this works

- `serviceAccountName: app` assigns the ServiceAccount that is bound to the Vault role. The pod's identity is established by Kubernetes, and Vault trusts it.
- The `projected` volume with `serviceAccountToken` creates a token specifically for Vault (with `audience: vault`). This is more secure than using the default ServiceAccount token because the token is audience-scoped and has a defined expiration.
- The init container authenticates to Vault, fetches secrets, and writes them to the `emptyDir` volume. The `medium: Memory` setting ensures the volume is tmpfs (never written to disk).
- The app container mounts the same volume at `/run/secrets/vault` in read-only mode. It reads secrets from files, not from environment variables.
- The app container has no access to Vault. If the app is compromised, the attacker cannot use it to access other secrets. The Vault token exists only in the init container's memory and is gone after the init container exits.

### Common mistakes

- Mounting the default ServiceAccount token into the init container. The default token has no audience restriction and may be valid for longer than necessary. Use a projected token with `audience: vault` instead.
- Not setting file permissions. By default, files in emptyDir are world-readable. The init container should `chmod 400` the secret files.
- Hardcoding the Vault address. Use an environment variable or a Kubernetes service name so the address can change without modifying the manifest.
- Not handling Vault being temporarily unavailable. The init container should retry authentication with exponential backoff, or the pod will be stuck in `Init:0/1` state.

---

## Part E: Implement Secret Rotation Awareness

### What happens when a Vault secret lease expires

Vault secrets have a lease duration (TTL). When the lease expires:

- **KV v2 secrets:** The secret itself is not revoked. KV v2 secrets do not have leases -- they persist until explicitly deleted or a new version is written. However, the Vault *token* used to read the secret has a TTL and will expire.
- **Dynamic secrets (e.g., database credentials):** Vault revokes the credentials. The database user that Vault created is deleted. Any connection using those credentials will fail.
- **The Vault token:** The token used by the init container expires after its TTL (1 hour in our configuration). After expiry, the token cannot be used to read any secrets.

The init container pattern only fetches secrets once. If the Vault token expires, the pod cannot re-authenticate. If dynamic secrets are revoked, the application loses database access.

### Sidecar pattern for secret lease renewal

```yaml
containers:
  - name: vault-sidecar
    image: hashicorp/vault:latest
    command:
      - sh
      - -c
      - |
        # Authenticate to Vault
        SA_TOKEN=$(cat /var/run/secrets/vault/token)
        VAULT_TOKEN=$(vault write -field=token \
          auth/kubernetes/login \
          role=app \
          jwt="$SA_TOKEN")

        # Fetch and write initial secret
        fetch_secret() {
          vault kv get -format=json secret/app/database \
            | jq -r '.data.data.password' > /secrets/db-password
          chmod 400 /secrets/db-password
          echo "[$(date)] Secret refreshed"
        }

        fetch_secret

        # Refresh loop: re-authenticate and re-fetch before token expires
        # Token TTL is 1 hour, refresh at 45 minutes
        while true; do
          sleep 2700  # 45 minutes

          echo "[$(date)] Refreshing Vault token..."
          VAULT_TOKEN=$(vault write -field=token \
            auth/kubernetes/login \
            role=app \
            jwt="$SA_TOKEN")

          fetch_secret
        done
    volumeMounts:
      - name: vault-secrets
        mountPath: /secrets
      - name: sa-token
        mountPath: /var/run/secrets/vault
        readOnly: true
```

### How the sidecar solves the problem

The sidecar runs continuously alongside the app container. Every 45 minutes (before the 1-hour token TTL expires), it re-authenticates to Vault and refreshes the secret. The app container sees the updated file in the shared volume.

The application must detect the file change. Two approaches:

1. **File watcher (inotifywait in a wrapper):** The app's entrypoint script watches the secret file and restarts the app process when it changes.
2. **Application-level detection:** The app periodically stats the file and checks the modification time. If it changes, the app re-reads the secret and refreshes its connection pool.

### Integration with Exercise 04 rotation strategy

The Vault sidecar pattern integrates with the zero-downtime rotation approach from Exercise 04:

```
Vault rotation trigger (cron or API)
        |
        v
1. Update secret in Vault (new version written)
        |
        v
2. Sidecar detects TTL approaching or new version available
        |
        v
3. Sidecar fetches new secret, writes to shared volume
        |
        v
4. App detects file change, refreshes connection pool
        |
        v
5. Old connections drain naturally (on pool recycling)
        |
        v
6. No pod restart needed, no downtime
```

The key difference from Exercise 04 is that Vault handles the "dual password" problem: Vault's dynamic database secrets engine can create database credentials that coexist with the previous set. Both sets are valid until the old lease expires. The application transitions to the new credentials without any interruption.

### Comparison: init container vs. sidecar

| Aspect | Init Container | Sidecar |
|--------|---------------|---------|
| **Secret freshness** | Fetched once at startup | Refreshed before expiry |
| **Handles rotation** | Requires pod restart | Automatic, no restart |
| **Resource usage** | Exits after fetching | Runs continuously |
| **Complexity** | Simple | Moderate |
| **Vault token lifetime** | Short-lived (just for init) | Long-lived (refreshed) |
| **Failure mode** | Pod stuck in Init | Sidecar retries, app uses stale secret |
| **Best for** | Static secrets, TLS certs | Dynamic secrets, frequently rotated secrets |

### Common mistakes

- Using the sidecar pattern for secrets that never rotate (e.g., static configuration). The overhead of a continuously running sidecar is unnecessary.
- Not handling the race condition where the app reads the secret file while the sidecar is writing it. Use an atomic write pattern: write to a temp file, then rename it over the original.
- Ignoring the sidecar's health. If the sidecar crashes, the secret will eventually expire. Use a Kubernetes liveness probe on the sidecar to restart it.
- Not setting resource limits on the sidecar. A sidecar that leaks memory will eventually OOM and take the pod down.
