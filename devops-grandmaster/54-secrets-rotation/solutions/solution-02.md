# Solution 02: Database Credential Rotation with Vault

## Part A: Database Secrets Engine Configuration

```bash
# Enable the database secrets engine
vault secrets enable database

# Configure the PostgreSQL connection
vault write database/config/postgres \
  plugin_name=postgresql-database-plugin \
  connection_url="postgresql://{{username}}:{{password}}@prod-db:5432/myapp?sslmode=verify-full" \
  allowed_roles="app-readwrite,app-readonly" \
  max_connection_lifetime="1h" \
  max_idle_connections=5 \
  max_open_connections=10

# Verify the connection
vault read database/config/postgres
```

HCL configuration:

```hcl
# database-engine.hcl
path "database/config/postgres" {
  capabilities = ["create", "read", "update", "delete"]
  allowed_parameters = {
    "plugin_name" = ["postgresql-database-plugin"]
    "connection_url" = ["postgresql://{{username}}:{{password}}@prod-db:5432/myapp?sslmode=verify-full"]
    "allowed_roles" = ["app-readwrite", "app-readonly"]
  }
}

path "database/roles/+" {
  capabilities = ["create", "read", "update", "delete", "list"]
}

path "database/creds/+" {
  capabilities = ["read"]
}
```

### Why This Works

The `{{username}}` and `{{password}}` template variables are replaced by Vault when it connects to PostgreSQL. Vault uses a root credential to manage dynamic credentials. The `max_connection_lifetime` ensures connections are recycled periodically, preventing stale connections.

### Common Mistakes

- **Hardcoding the root credential.** The root credential used to configure the connection should itself be stored in Vault and rotated.
- **Not setting `sslmode=verify-full`.** Without TLS verification, the connection is vulnerable to MITM attacks.
- **Using overly broad `allowed_roles`.** Only allow the roles that this connection actually needs.

---

## Part B: Role-Based Credentials

```hcl
# roles/app-readwrite.hcl
resource "vault_database_secret_backend_role" "app_readwrite" {
  backend             = vault_mount.database.path
  name                = "app-readwrite"
  db_name             = vault_database_secret_backend_connection.postgres.name
  default_ttl         = 3600    # 1 hour
  max_ttl             = 86400   # 24 hours

  creation_statements = [
    <<-SQL
      CREATE ROLE "{{name}}" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}';
      GRANT CONNECT ON DATABASE myapp TO "{{name}}";
      GRANT USAGE ON SCHEMA public TO "{{name}}";
      GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO "{{name}}";
      ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO "{{name}}";
    SQL
  ]

  revocation_statements = [
    <<-SQL
      REASSIGN OWNED BY "{{name}}" TO postgres;
      DROP OWNED BY "{{name}}";
      DROP ROLE IF EXISTS "{{name}}";
    SQL
  ]
}

# roles/app-readonly.hcl
resource "vault_database_secret_backend_role" "app_readonly" {
  backend             = vault_mount.database.path
  name                = "app-readonly"
  db_name             = vault_database_secret_backend_connection.postgres.name
  default_ttl         = 3600
  max_ttl             = 86400

  creation_statements = [
    <<-SQL
      CREATE ROLE "{{name}}" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}';
      GRANT CONNECT ON DATABASE myapp TO "{{name}}";
      GRANT USAGE ON SCHEMA public TO "{{name}}";
      GRANT SELECT ON ALL TABLES IN SCHEMA public TO "{{name}}";
      ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO "{{name}}";
    SQL
  ]

  revocation_statements = [
    <<-SQL
      REASSIGN OWNED BY "{{name}}" TO postgres;
      DROP OWNED BY "{{name}}";
      DROP ROLE IF EXISTS "{{name}}";
    SQL
  ]
}
```

Answers:

1. **When TTL expires**: Vault revokes the credentials by executing `revocation_statements`. The database role is dropped and connections using that role are terminated.

2. **When Max TTL expires**: Credentials are revoked regardless of renewal status. Max TTL is the hard limit that cannot be exceeded.

3. **Why 1-hour TTL**: Limits the exposure window if credentials are leaked. An attacker has at most 1 hour to use them. The 24-hour Max TTL allows renewal up to 24 times.

### Why This Works

The `VALID UNTIL '{{expiration}}'` clause ensures the database itself enforces expiration, even if Vault's revocation fails. This is defense in depth.

### Common Mistakes

- **Not including `VALID UNTIL`.** Without it, the database role remains valid after Vault revokes it.
- **Not handling revocation failures.** If Vault cannot reach the database, credentials remain valid until `VALID UNTIL` expires.
- **Using the same role for all services.** Each service should have its own role with appropriate permissions.

---

## Part C: Credential Renewal in the Application

```python
# app/vault_client.py
import hvac
import time
import threading
import logging

logger = logging.getLogger(__name__)

class VaultCredentialManager:
    def __init__(self, vault_addr: str, role_name: str, vault_token: str = None):
        self.client = hvac.Client(url=vault_addr)
        if vault_token:
            self.client.token = vault_token
        self.role_name = role_name
        self.credentials = None
        self.lock = threading.Lock()

    def get_credentials(self) -> dict:
        with self.lock:
            if self.credentials is None or self._is_expired():
                self._refresh_credentials()
            return self.credentials

    def _refresh_credentials(self):
        try:
            response = self.client.secrets.database.generate_credentials(
                name=self.role_name
            )
            self.credentials = {
                'username': response['data']['username'],
                'password': response['data']['password'],
                'expiration': response['data']['expiration'],
                'last_renewed': time.time()
            }
            logger.info(f"Credentials refreshed for role {self.role_name}")
        except Exception as e:
            logger.error(f"Failed to refresh credentials: {e}")
            if self.credentials is None:
                raise

    def _is_expired(self) -> bool:
        if self.credentials is None:
            return True
        # Refresh 5 minutes before expiration
        return time.time() > (self.credentials['expiration'] - 300)

    def start_renewal_thread(self):
        def _renewal_loop():
            while True:
                try:
                    with self.lock:
                        if self.credentials and not self._is_expired():
                            remaining = self.credentials['expiration'] - time.time()
                            sleep_time = max(remaining * 2/3, 60)
                        else:
                            sleep_time = 60
                    time.sleep(sleep_time)
                    with self.lock:
                        if self._is_expired():
                            self._refresh_credentials()
                except Exception as e:
                    logger.error(f"Renewal thread error: {e}")
                    time.sleep(60)

        thread = threading.Thread(target=_renewal_loop, daemon=True)
        thread.start()
        logger.info("Credential renewal thread started")
```

### Why This Works

The "refresh at 2/3 of TTL" rule provides a 20-minute buffer for retrying if refresh fails. The 5-minute pre-expiration check ensures credentials are always valid when requested. The background thread handles renewal automatically.

---

## Part D: Graceful Rotation Handling

| Scenario | Current State | Rotation Action | Application Behavior |
|----------|--------------|-----------------|---------------------|
| Normal rotation | Credentials valid for 10 more minutes | Request new credentials from Vault | Old credentials valid until TTL. New credentials used for new connections. |
| Active connection pool | 5 connections using old credentials | Credentials rotated | Old connections continue (old creds valid until TTL). New connections use new creds. |
| Vault unavailable | Credentials expire in 5 minutes | Cannot reach Vault | Continue using existing credentials. Log error and retry. |
| Application startup | No credentials yet | First request triggers credential fetch | Fetch from Vault. If unavailable, fail startup with clear error. |

### Why This Works

Old credentials remain valid until their TTL expires. The connection pool naturally rotates: old connections use old credentials, new connections use new credentials. As old connections are closed (idle timeout, max lifetime), they are replaced with new connections.

---

## Part E: Verification Script

```bash
#!/bin/bash
# test-rotation.sh
set -euo pipefail

echo "=== Step 1: Get initial credentials ==="
CREDS_1=$(vault read -format=json database/creds/app-readwrite)
USER_1=$(echo "$CREDS_1" | jq -r '.data.username')
PASS_1=$(echo "$CREDS_1" | jq -r '.data.password')
echo "Username: $USER_1"

echo ""
echo "=== Step 2: Connect with initial credentials ==="
PGPASSWORD="$PASS_1" psql "postgresql://$USER_1@prod-db:5432/myapp" \
  -c "SELECT current_user, now();" 2>&1

echo ""
echo "=== Step 3: Get new credentials ==="
CREDS_2=$(vault read -format=json database/creds/app-readwrite)
USER_2=$(echo "$CREDS_2" | jq -r '.data.username')
PASS_2=$(echo "$CREDS_2" | jq -r '.data.password')
echo "New username: $USER_2"

echo ""
echo "=== Step 4: Verify old credentials still work (within TTL) ==="
PGPASSWORD="$PASS_1" psql "postgresql://$USER_1@prod-db:5432/myapp" \
  -c "SELECT 'old creds still work';" 2>&1 || echo "Old credentials failed (TTL expired)"

echo ""
echo "=== Step 5: Verify new credentials work ==="
PGPASSWORD="$PASS_2" psql "postgresql://$USER_2@prod-db:5432/myapp" \
  -c "SELECT 'new creds work';" 2>&1

echo ""
echo "=== All tests passed ==="
```

### Common Mistakes to Avoid

1. **Not protecting the root credential.** The root credential Vault uses to manage dynamic credentials is the most critical secret.
2. **Not monitoring credential usage.** Track which services use credentials, how often they renew, and whether renewals fail.
3. **TTL too long.** A 24-hour TTL means a leaked credential is valid for 24 hours. Use shorter TTLs for sensitive databases.

## Key Takeaway

Dynamic database credentials eliminate static passwords: no shared credentials, no manual rotation, no leaked passwords in config files. The key to production success is handling credential rotation gracefully -- old credentials remain valid until TTL, allowing connection pools to rotate naturally.
