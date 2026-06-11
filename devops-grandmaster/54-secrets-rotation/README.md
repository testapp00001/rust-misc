# 54 - Secrets Rotation

**Previous:** [53 - WAF Rules](../53-waf-rules/README.md) | **Next:** [55 - Vulnerability Scanning](../55-vulnerability-scanning/README.md)

---

## Problem

Your database password was set three years ago. It has been copied into .env files, CI/CD variables, Kubernetes secrets, developer laptops, deployment scripts, and a Confluence page that someone forgot to delete. An intern who left two years ago still has a copy on their personal machine. A contractor's laptop was stolen. A CI/CD pipeline log accidentally printed the password.

You have no way to know how many copies of this secret exist, who has access, or whether it has been compromised. And you cannot rotate it because you do not know which services depend on it, and changing it might break production at 3 AM.

Secrets rotation is the practice of automatically and regularly changing credentials -- passwords, API keys, tokens, certificates -- so that any leaked secret has a limited window of usefulness to an attacker.

---

## Naive Way

```bash
# Set it once and forget it
export DB_PASSWORD="SuperSecret123!"
echo "DB_PASSWORD=SuperSecret123!" >> .env

# "We'll rotate it manually if there's a breach"
# (You won't know there's been a breach for 200+ days on average)

# Hardcoded in application config
DATABASE_URL="postgres://admin:SuperSecret123!@prod-db:5432/myapp"
API_KEY="sk-prod-abc123def456ghi789"  # In source code
```

**Why this fails:**
- Secrets are embedded in code, config files, and environment variables
- No automated rotation means secrets live forever
- No audit trail of who accessed what secret when
- Manual rotation is terrifying because you do not know all consumers
- One leaked credential grants persistent, undetected access

---

## Right Way

### HashiCorp Vault for Centralized Secrets Management

```hcl
# vault-config.hcl - Enable database secrets engine
resource "vault_mount" "database" {
  path = "database"
  type = "database"
}

resource "vault_database_secret_backend_connection" "postgres" {
  backend       = vault_mount.database.path
  name          = "postgres"
  allowed_roles = ["app-readwrite", "app-readonly"]

  postgresql {
    connection_url = "postgresql://{{username}}:{{password}}@prod-db:5432/myapp?sslmode=verify-full"
    allowed_roles  = ["app-readwrite", "app-readonly"]
  }

  root_rotation_statements = [
    "ALTER ROLE \"{{name}}\" WITH PASSWORD '{{password}}';"
  ]
}

resource "vault_database_secret_backend_role" "app_readwrite" {
  backend             = vault_mount.database.path
  name                = "app-readwrite"
  db_name             = vault_database_secret_backend_connection.postgres.name
  default_ttl         = 3600      # 1 hour
  max_ttl             = 86400     # 24 hours

  creation_statements = [
    <<-SQL
      CREATE ROLE "{{name}}" WITH LOGIN PASSWORD '{{password}}'
      VALID UNTIL '{{expiration}}';
      GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO "{{name}}";
      GRANT USAGE ON ALL SEQUENCES IN SCHEMA public TO "{{name}}";
      ALTER DEFAULT PRIVILEGES IN SCHEMA public
        GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO "{{name}}";
    SQL
  ]

  revocation_statements = [
    "REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA public FROM \"{{name}}\";",
    "DROP ROLE IF EXISTS \"{{name}}\";"
  ]
}
```

### Application Integration with Vault

```rust
// src/secrets/vault.rs
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Deserialize)]
struct DatabaseCredentials {
    username: String,
    password: String,
}

pub struct SecretManager {
    client: VaultClient,
    cached_creds: Arc<RwLock<Option<(DatabaseCredentials, std::time::Instant)>>>,
}

impl SecretManager {
    pub fn new(vault_addr: &str, token: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = VaultClient::new(
            VaultClientSettingsBuilder::default()
                .address(vault_addr)
                .token(token)
                .build()?,
        )?;

        Ok(Self {
            client,
            cached_creds: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn get_database_credentials(&self) -> Result<DatabaseCredentials, Box<dyn std::error::Error>> {
        // Check cache first (with TTL safety margin)
        {
            let cache = self.cached_creds.read().await;
            if let Some((creds, cached_at)) = cache.as_ref() {
                if cached_at.elapsed() < std::time::Duration::from_secs(3300) {
                    return Ok(DatabaseCredentials {
                        username: creds.username.clone(),
                        password: creds.password.clone(),
                    });
                }
            }
        }

        // Fetch new credentials from Vault
        let creds: DatabaseCredentials = vaultrs::kv2::read(
            &self.client,
            "secret",
            "database/creds/app-readwrite",
        ).await?;

        // Update cache
        {
            let mut cache = self.cached_creds.write().await;
            *cache = Some((
                DatabaseCredentials {
                    username: creds.username.clone(),
                    password: creds.password.clone(),
                },
                std::time::Instant::now(),
            ));
        }

        Ok(creds)
    }

    pub async fn get_api_key(&self, service: &str) -> Result<String, Box<dyn std::error::Error>> {
        let secret: serde_json::Value = vaultrs::kv2::read(
            &self.client,
            "secret",
            &format!("api-keys/{}", service),
        ).await?;

        Ok(secret["key"].as_str().unwrap_or_default().to_string())
    }
}
```

### Automated Rotation with Vault Agent

```hcl
# vault-agent.hcl
pid_file = "/var/run/vault-agent.pid"

vault {
  address = "https://vault.internal:8200"
}

auto_auth {
  method "aws" {
    mount_path = "auth/aws"
    config = {
      type = "iam"
      role = "app-role"
    }
  }

  sink "file" {
    config = {
      path = "/var/run/vault-token"
    }
  }
}

template {
  source      = "/etc/vault-agent/db-creds.ctmpl"
  destination = "/etc/app/database.env"
  perms       = 0600
  command     = "systemctl reload app"
}

# /etc/vault-agent/db-creds.ctmpl:
# {{ with secret "database/creds/app-readwrite" }}
# DB_USERNAME={{ .Data.username }}
# DB_PASSWORD={{ .Data.password }}
# {{ end }}
```

### AWS Secrets Manager with Automatic Rotation

```hcl
# aws-secrets.tf

resource "aws_secretsmanager_secret" "db_password" {
  name                    = "production/database/password"
  description             = "Production database password"
  recovery_window_in_days = 7

  rotation_rules {
    automatically_after_days = 30
  }
}

resource "aws_secretsmanager_secret_version" "db_password" {
  secret_id = aws_secretsmanager_secret.db_password.id
  secret_string = jsonencode({
    username = "app_user"
    password = var.initial_db_password
  })
}

resource "aws_secretsmanager_secret_rotation" "db_password" {
  secret_id           = aws_secretsmanager_secret.db_password.id
  rotation_lambda_arn = aws_lambda_function.rotate_db_password.arn

  rotation_rules {
    automatically_after_days = 30
  }
}

resource "aws_lambda_function" "rotate_db_password" {
  filename         = "lambda/rotate-password.zip"
  function_name    = "rotate-db-password"
  role             = aws_iam_role.lambda_rotation.arn
  handler          = "index.handler"
  runtime          = "python3.11"
  timeout          = 30

  environment {
    variables = {
      SECRETS_MANAGER_ENDPOINT = "https://secretsmanager.${var.region}.amazonaws.com"
    }
  }
}
```

```python
# lambda/rotate-password/index.py - Secrets Manager rotation Lambda
import json
import boto3
import secrets
import string
import psycopg2

def handler(event, context):
    service_client = boto3.client('secretsmanager')
    arn = event['SecretId']
    token = event['ClientRequestToken']
    step = event['Step']

    if step == "createSecret":
        create_secret(service_client, arn, token)
    elif step == "setSecret":
        set_secret(service_client, arn, token)
    elif step == "testSecret":
        test_secret(service_client, arn, token)
    elif step == "finishSecret":
        finish_secret(service_client, arn, token)
    else:
        raise ValueError(f"Invalid step: {step}")

def create_secret(service_client, arn, token):
    current = service_client.get_secret_value(SecretId=arn, VersionStage="AWSCURRENT")
    current_dict = json.loads(current['SecretString'])

    alphabet = string.ascii_letters + string.digits + "!@#$%^&*"
    new_password = ''.join(secrets.choice(alphabet) for _ in range(32))

    new_secret = {
        'host': current_dict['host'],
        'port': current_dict['port'],
        'dbname': current_dict['dbname'],
        'username': current_dict['username'],
        'password': new_password
    }

    service_client.put_secret_value(
        SecretId=arn,
        ClientRequestToken=token,
        SecretString=json.dumps(new_secret),
        VersionStages=['AWSPENDING']
    )

def set_secret(service_client, arn, token):
    pending = service_client.get_secret_value(
        SecretId=arn, VersionId=token, VersionStage="AWSPENDING"
    )
    pending_dict = json.loads(pending['SecretString'])

    conn = psycopg2.connect(
        host=pending_dict['host'],
        port=pending_dict['port'],
        dbname=pending_dict['dbname'],
        user=pending_dict['username'],
        password=pending_dict.get('old_password', ''),
        sslmode='require'
    )
    conn.autocommit = True
    cur = conn.cursor()
    cur.execute(
        f"ALTER USER {pending_dict['username']} WITH PASSWORD %s",
        (pending_dict['password'],)
    )
    conn.close()

def test_secret(service_client, arn, token):
    pending = service_client.get_secret_value(
        SecretId=arn, VersionId=token, VersionStage="AWSPENDING"
    )
    pending_dict = json.loads(pending['SecretString'])

    conn = psycopg2.connect(
        host=pending_dict['host'],
        port=pending_dict['port'],
        dbname=pending_dict['dbname'],
        user=pending_dict['username'],
        password=pending_dict['password'],
        sslmode='require'
    )
    cur = conn.cursor()
    cur.execute("SELECT 1")
    assert cur.fetchone()[0] == 1
    conn.close()

def finish_secret(service_client, arn, token):
    metadata = service_client.describe_secret(SecretId=arn)

    current_version = None
    for version, stages in metadata['VersionIdsToStages'].items():
        if "AWSCURRENT" in stages:
            current_version = version
            break

    if current_version == token:
        return

    service_client.update_secret_version_stage(
        SecretId=arn,
        VersionStage="AWSCURRENT",
        MoveToVersionId=token,
        RemoveFromVersionId=current_version
    )
```

### Kubernetes External Secrets Operator

```yaml
# external-secret.yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: app-database-credentials
  namespace: production
spec:
  refreshInterval: 1h

  secretStoreRef:
    name: vault-backend
    kind: ClusterSecretStore

  target:
    name: app-db-credentials
    creationPolicy: Owner
    template:
      type: Opaque
      data:
        DATABASE_URL: "postgresql://{{ .username }}:{{ .password }}@prod-db:5432/myapp"

  data:
    - secretKey: username
      remoteRef:
        key: database/creds/app-readwrite
        property: username

    - secretKey: password
      remoteRef:
        key: database/creds/app-readwrite
        property: password
---
apiVersion: external-secrets.io/v1beta1
kind: ClusterSecretStore
metadata:
  name: vault-backend
spec:
  provider:
    vault:
      server: "https://vault.internal:8200"
      path: "secret"
      version: "v2"
      auth:
        kubernetes:
          mountPath: "kubernetes"
          role: "app-role"
          serviceAccountRef:
            name: "vault-auth"
```

---

## Production Way

### Complete Secrets Lifecycle Management

```
+------------------+     +------------------+     +------------------+
|   Generate       |     |   Store          |     |   Distribute     |
|                  |     |                  |     |                  |
| - Cryptographic  |---->| - HashiCorp Vault|---->| - Vault Agent    |
|   RNG            |     | - AWS SM         |     | - ESO (K8s)      |
| - 32+ chars      |     | - GCP SM         |     | - Injected at    |
| - No patterns    |     | - Azure KV       |       runtime        |
+------------------+     +------------------+     +------------------+
                                                          |
                                                          v
+------------------+     +------------------+     +------------------+
|   Revoke         |     |   Monitor        |     |   Use            |
|                  |<----|                  |<----|                  |
| - Auto-revoke    |     | - Access logs    |     | - In-memory only |
|   after TTL      |     | - Anomaly detect |     | - No env vars    |
| - Revoke on      |     | - Alert on       |     | - No files       |
|   employee exit  |     |   unusual access |     | - Rotate on error|
+------------------+     +------------------+     +------------------+
```

### API Key Rotation for Third-Party Services

```rust
// src/secrets/rotating_client.rs
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

pub struct RotatingApiClient {
    client: Client,
    current_key: Arc<RwLock<(String, Instant)>>,
    secret_manager: Arc<SecretManager>,
    service_name: String,
    key_ttl: Duration,
}

impl RotatingApiClient {
    pub fn new(
        secret_manager: Arc<SecretManager>,
        service_name: &str,
        key_ttl: Duration,
    ) -> Self {
        Self {
            client: Client::new(),
            current_key: Arc::new(RwLock::new((String::new(), Instant::now() - key_ttl))),
            secret_manager,
            service_name: service_name.to_string(),
            key_ttl,
        }
    }

    async fn get_valid_key(&self) -> Result<String, Box<dyn std::error::Error>> {
        let key_data = self.current_key.read().await;

        if key_data.1.elapsed() < self.key_ttl - Duration::from_secs(300) {
            return Ok(key_data.0.clone());
        }
        drop(key_data);

        let new_key = self.secret_manager
            .get_api_key(&self.service_name)
            .await?;

        let mut key_data = self.current_key.write().await;
        *key_data = (new_key.clone(), Instant::now());

        Ok(new_key)
    }

    pub async fn request(
        &self,
        method: reqwest::Method,
        url: &str,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
        let key = self.get_valid_key().await?;

        let response = self.client
            .request(method, url)
            .header("Authorization", format!("Bearer {}", key))
            .send()
            .await?;

        if response.status() == 401 {
            let mut key_data = self.current_key.write().await;
            *key_data = (String::new(), Instant::now() - self.key_ttl);
            drop(key_data);

            let new_key = self.get_valid_key().await?;
            let response = self.client
                .request(reqwest::Method::GET, url)
                .header("Authorization", format!("Bearer {}", new_key))
                .send()
                .await?;

            return Ok(response);
        }

        Ok(response)
    }
}
```

### Rotation Audit and Compliance

```sql
CREATE TABLE secret_access_audit (
    id BIGSERIAL PRIMARY KEY,
    secret_path VARCHAR(255) NOT NULL,
    accessor VARCHAR(255) NOT NULL,
    access_type VARCHAR(50) NOT NULL,
    accessed_at TIMESTAMP DEFAULT now(),
    source_ip INET,
    success BOOLEAN NOT NULL,
    error_message TEXT
);

CREATE VIEW secret_rotation_status AS
SELECT
    secret_path,
    MAX(CASE WHEN access_type = 'rotate' THEN accessed_at END) AS last_rotated,
    NOW() - MAX(CASE WHEN access_type = 'rotate' THEN accessed_at END) AS age,
    CASE
        WHEN NOW() - MAX(CASE WHEN access_type = 'rotate' THEN accessed_at END) > INTERVAL '30 days'
        THEN 'OVERDUE'
        WHEN NOW() - MAX(CASE WHEN access_type = 'rotate' THEN accessed_at END) > INTERVAL '25 days'
        THEN 'DUE_SOON'
        ELSE 'OK'
    END AS rotation_status
FROM secret_access_audit
GROUP BY secret_path;
```

---

## Hands-On Lab

### Lab: Implement Automated Secrets Rotation with Vault

**Duration:** 90 minutes

**Prerequisites:**
- Docker installed
- Basic understanding of PostgreSQL

**Step 1: Start Vault in Development Mode**

```bash
docker run -d --name vault \
    -p 8200:8200 \
    -e VAULT_DEV_ROOT_TOKEN_ID="root-token" \
    -e VAULT_DEV_LISTEN_ADDRESS="0.0.0.0:8200" \
    hashicorp/vault:latest

export VAULT_ADDR='http://localhost:8200'
export VAULT_TOKEN='root-token'

vault secrets enable database

docker run -d --name postgres \
    -p 5432:5432 \
    -e POSTGRES_PASSWORD=rootpass \
    postgres:15
```

**Step 2: Configure Vault Database Connection**

```bash
vault write database/config/postgres \
    plugin_name=postgresql-database-plugin \
    connection_url="postgresql://{{username}}:{{password}}@host.docker.internal:5432/postgres?sslmode=disable" \
    allowed_roles="app-role" \
    username="postgres" \
    password="rootpass"

vault write database/roles/app-role \
    db_name=postgres \
    creation_statements="CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}'; \
        GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO \"{{name}}\";" \
    default_ttl="1h" \
    max_ttl="24h"
```

**Step 3: Generate and Test Dynamic Credentials**

```bash
# Generate new credentials
vault read database/creds/app-role

# Test with PostgreSQL
PGPASSWORD="<generated_password>" psql -h localhost -U "<generated_username>" -d postgres \
    -c "SELECT current_user, now();"

# Generate another set - note different username/password
vault read database/creds/app-role
```

**Step 4: Implement Application with Auto-Renewing Credentials**

```rust
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let vault = VaultClient::new(
        VaultClientSettingsBuilder::default()
            .address("http://localhost:8200")
            .token("root-token")
            .build()?,
    )?;

    let creds: serde_json::Value = vaultrs::database::creds::read(
        &vault, "postgres", "app-role"
    ).await?;

    let username = creds["username"].as_str().unwrap();
    let password = creds["password"].as_str().unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&format!(
            "postgres://{}:{}@localhost/postgres",
            username, password
        ))
        .await?;

    let result: (String,) = sqlx::query_as("SELECT current_user")
        .fetch_one(&pool)
        .await?;
    println!("Connected as: {}", result.0);

    // Credential renewal in background
    let vault_clone = vault.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(3300)).await;
            match vaultrs::token::renew_self(&vault_clone).await {
                Ok(_) => println!("Token renewed"),
                Err(e) => eprintln!("Token renewal failed: {}", e),
            }
        }
    });

    Ok(())
}
```

**Step 5: Observe Automatic Rotation**

```bash
# Watch credentials expire and rotate
watch -n 5 'vault list sys/leases/lookup/database/creds/app-role 2>/dev/null || echo "No active leases"'

# After TTL expires, old credentials are revoked
PGPASSWORD="<old_password>" psql -h localhost -U "<old_username>" -d postgres \
    -c "SELECT 1;" 2>&1 || echo "Credentials revoked as expected"
```

**Deliverable:** Modify the application to handle credential rotation gracefully -- when the database connection fails with an authentication error, fetch fresh credentials from Vault and reconnect without crashing.

---

## Limitation

Secrets rotation ensures that compromised credentials expire quickly. But rotation addresses the symptom of credential leakage, not the root cause. A secret that is rotated every 30 days still grants 30 days of access to anyone who obtains it.

More importantly, secrets rotation does not protect against **vulnerabilities in your dependencies**. Your application might have perfectly managed secrets, but if a library you depend on has a remote code execution vulnerability, an attacker does not need your credentials -- they exploit the vulnerability directly.

Your carefully rotated database password is useless if the `log4j` library running in your application lets an attacker execute arbitrary commands. Or if the base Docker image you use has a known rootkit. Or if a transitive dependency in your `Cargo.toml` was taken over by a malicious maintainer.

---

## Next Topic

[55 - Vulnerability Scanning](../55-vulnerability-scanning/README.md) -- Learn how to scan container images, audit dependencies, and detect known vulnerabilities before they reach production.
