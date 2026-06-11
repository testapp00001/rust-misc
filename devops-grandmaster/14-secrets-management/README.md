# Module 14: Secrets Management — Handling Passwords, API Keys, Certificates

> **Previous Module (13):** Environment Variables
> **Previous Limitation:** Environment variables are visible via `docker inspect` and `/proc/<pid>/environ`. They are not suitable for secrets.
> **This Module Solves That:** Proper secret storage, injection, rotation, and auditing.

---

## 1. The Problem

Your application needs credentials to function:

- Database passwords
- API keys (Stripe, AWS, Twilio)
- TLS/SSL certificates
- OAuth client secrets
- Encryption keys
- SSH keys for deployment

In the previous module, we used environment variables for configuration. But secrets are different from configuration. Configuration is `LOG_LEVEL=info` — it's fine if anyone sees it. Secrets are `DB_PASSWORD=p@ssw0rd!` — exposure means compromise.

**The core problem:** How do you get secrets into your containers without exposing them to anyone who can inspect the container, read image layers, or access the orchestrator's API?

---

## 2. The Naive Way

### Baking secrets into the Docker image

```dockerfile
FROM node:18-alpine
COPY . .
RUN echo "DB_PASSWORD=secret123" > /app/.env
RUN npm install
CMD ["node", "server.js"]
```

### Why it fails catastrophically

1. **Secrets persist in image layers.** Every `RUN` instruction creates a layer. Even if you `RUN rm /app/.env`, the secret is still in the previous layer:

```bash
docker history myapp --no-trunc
# Shows: RUN echo "DB_PASSWORD=secret123" > /app/.env
```

2. **Anyone with the image has the secrets.** Push to Docker Hub? Everyone can pull and extract your secrets.

3. **No rotation.** Changing a password means rebuilding and redeploying.

4. **No audit trail.** You cannot track who accessed the secret or when.

### Passing secrets as build args

```dockerfile
ARG DB_PASSWORD
RUN echo "DATABASE_URL=postgres://admin:${DB_PASSWORD}@db/prod" > /app/.env
```

```bash
docker build --build-arg DB_PASSWORD=secret123 -t myapp .
```

**Still broken.** Build args are visible in `docker history`:

```bash
docker history myapp
# ARG DB_PASSWORD
# RUN echo "DATABASE_URL=postgres://admin:secret123@db/prod" > /app/.env
```

---

## 3. The Right Way

### Method 1: Docker Secrets (Swarm Mode)

Docker Swarm has built-in secret management. Secrets are:

- Encrypted at rest (AES-256-GCM)
- Encrypted in transit (TLS)
- Mounted as files in `/run/secrets/` (tmpfs — never touches disk)
- Not visible in `docker inspect`

**Create a secret:**

```bash
# From stdin
echo "super-secret-password" | docker secret create db_password -

# From file
docker secret create db_password ./password.txt
```

**Use in a Docker Swarm service:**

```yaml
version: '3.8'

services:
  app:
    image: myapp:1.0.0
    secrets:
      - db_password
      - api_key
    environment:
      - DB_PASSWORD_FILE=/run/secrets/db_password
      - API_KEY_FILE=/run/secrets/api_key

secrets:
  db_password:
    external: true
  api_key:
    external: true
```

**Application reads the secret from file:**

```javascript
const fs = require('fs');

function readSecret(filePath) {
  try {
    return fs.readFileSync(filePath, 'utf8').trim();
  } catch (err) {
    throw new Error(`Failed to read secret from ${filePath}: ${err.message}`);
  }
}

const dbPassword = readSecret(process.env.DB_PASSWORD_FILE);
const apiKey = readSecret(process.env.API_KEY_FILE);
```

```rust
use std::fs;

fn read_secret(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    Ok(content.trim().to_string())
}

fn main() {
    let db_password = read_secret("/run/secrets/db_password")
        .expect("Failed to read database password");
    let api_key = read_secret("/run/secrets/api_key")
        .expect("Failed to read API key");
}
```

### Method 2: File-based secrets (Docker Compose)

For local development without Swarm, mount secret files as read-only volumes:

```yaml
version: '3.8'

services:
  app:
    image: myapp:1.0.0
    volumes:
      - ./secrets/db_password.txt:/run/secrets/db_password:ro
      - ./secrets/api_key.txt:/run/secrets/api_key:ro
    environment:
      - DB_PASSWORD_FILE=/run/secrets/db_password
      - API_KEY_FILE=/run/secrets/api_key
```

**Important:** Add `secrets/` to `.gitignore`:

```gitignore
secrets/
*.pem
*.key
.env.local
```

### Method 3: Docker Compose secrets with files

```yaml
version: '3.8'

services:
  app:
    image: myapp:1.0.0
    secrets:
      - db_password
      - api_key

secrets:
  db_password:
    file: ./secrets/db_password.txt
  api_key:
    file: ./secrets/api_key.txt
```

---

## 4. The Production Way

### External secret managers

In production, secrets live in a dedicated secret management system, not on the filesystem or in Docker.

#### HashiCorp Vault

Vault is the industry standard for secret management. It provides:

- Dynamic secrets (generate credentials on-demand)
- Automatic rotation
- Audit logging
- Multiple auth methods (token, LDAP, Kubernetes, AWS IAM)
- Secret versioning and rollback

**Architecture:**

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Application │────>│    Vault    │────>│  Database   │
│  Container   │     │   Server    │     │  (dynamic   │
│              │     │             │     │  creds)     │
└─────────────┘     └─────────────┘     └─────────────┘
                          │
                    ┌─────┴─────┐
                    │  Audit    │
                    │  Log      │
                    └───────────┘
```

**Vault integration example:**

```bash
# Start Vault in dev mode (for lab only)
docker run -d --name vault \
  -p 8200:8200 \
  -e VAULT_DEV_ROOT_TOKEN_ID=myroot \
  vault:latest

# Store a secret
export VAULT_ADDR='http://localhost:8200'
export VAULT_TOKEN='myroot'

vault kv put secret/myapp/db \
  username=admin \
  password=super-secret-password \
  host=prod-db.example.com \
  port=5432

# Application retrieves it
vault kv get -format=json secret/myapp/db
```

**Application with Vault:**

```rust
use reqwest;
use serde::Deserialize;

#[derive(Deserialize)]
struct VaultResponse {
    data: VaultData,
}

#[derive(Deserialize)]
struct VaultData {
    data: DbSecrets,
}

#[derive(Deserialize)]
struct DbSecrets {
    username: String,
    password: String,
    host: String,
    port: u16,
}

async fn get_db_secrets() -> Result<DbSecrets, Box<dyn std::error::Error>> {
    let vault_addr = std::env::var("VAULT_ADDR")?;
    let vault_token = std::env::var("VAULT_TOKEN")?;

    let client = reqwest::Client::new();
    let resp: VaultResponse = client
        .get(format!("{}/v1/secret/data/myapp/db", vault_addr))
        .header("X-Vault-Token", vault_token)
        .send()
        .await?
        .json()
        .await?;

    Ok(resp.data.data)
}
```

#### AWS Secrets Manager

```bash
# Store a secret
aws secretsmanager create-secret \
  --name myapp/production/db-password \
  --secret-string "super-secret-password"

# Retrieve it
aws secretsmanager get-secret-value \
  --secret-id myapp/production/db-password \
  --query SecretString --output text
```

**Docker Compose with AWS Secrets Manager:**

```yaml
services:
  app:
    image: myapp:1.0.0
    environment:
      - AWS_REGION=us-east-1
      - SECRET_ID=myapp/production/db-password
    entrypoint: >
      sh -c '
        export DB_PASSWORD=$$(aws secretsmanager get-secret-value \
          --secret-id $$SECRET_ID \
          --query SecretString --output text) &&
        node server.js
      '
```

#### GCP Secret Manager

```bash
# Create a secret
gcloud secrets create myapp-db-password --replication-policy="automatic"

# Add a version
echo -n "super-secret-password" | \
  gcloud secrets versions add myapp-db-password --data-file=-

# Access it
gcloud secrets versions access latest --secret=myapp-db-password
```

### Kubernetes Secrets (Preview of Phase 4)

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: myapp-secrets
type: Opaque
data:
  db-password: c3VwZXItc2VjcmV0LXBhc3N3b3Jk  # base64 encoded
  api-key: c2stbGl2ZS1hYmMxMjM=
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp
spec:
  template:
    spec:
      containers:
        - name: app
          image: myapp:1.0.0
          env:
            - name: DB_PASSWORD
              valueFrom:
                secretKeyRef:
                  name: myapp-secrets
                  key: db-password
          volumeMounts:
            - name: secrets
              mountPath: /run/secrets
              readOnly: true
      volumes:
        - name: secrets
          secret:
            secretName: myapp-secrets
```

### Secret rotation

Secrets should be rotated regularly. The pattern depends on your secret manager:

**Manual rotation:**

```bash
# 1. Generate new secret
NEW_PASSWORD=$(openssl rand -base64 32)

# 2. Update the database password
psql -h prod-db -U admin -c "ALTER USER app PASSWORD '$NEW_PASSWORD';"

# 3. Update the secret store
aws secretsmanager update-secret \
  --secret-id myapp/production/db-password \
  --secret-string "$NEW_PASSWORD"

# 4. Restart the application (picks up new secret)
docker compose restart app
```

**Automatic rotation (AWS Secrets Manager):**

```bash
aws secretsmanager rotate-secret \
  --secret-id myapp/production/db-password \
  --rotation-lambda-arn arn:aws:lambda:us-east-1:123456789:function/SecretsManagerRotation \
  --rotation-rules AutomaticallyAfterDays=30
```

### Secret scanning in CI/CD

Never let secrets reach your images or repositories. Use automated scanning:

**TruffleHog (scan git history):**

```bash
# Install
pip install trufflehog

# Scan repository
trufflehog git file://. --only-verified
```

**GitLeaks (pre-commit hook):**

```bash
# Install
brew install gitleaks

# Scan
gitleaks detect --source . --verbose

# As pre-commit hook
gitleaks protect --staged --verbose
```

**Docker image scanning:**

```bash
# Scan image for embedded secrets
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
  aquasec/trivy image myapp:latest
```

**GitHub Actions secret scanning:**

```yaml
name: Secret Scan
on: [push, pull_request]

jobs:
  secret-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: gitleaks/gitleaks-action@v2
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

---

## 5. Hands-On Lab

### Lab: Implement three levels of secret management

**Objective:** Progress from naive to proper secret management using Docker Compose.

#### Step 1: Create the project

```bash
mkdir -p secrets-lab && cd secrets-lab
```

#### Step 2: Create a simple application that needs secrets

**app.py:**

```python
import os
import sys
from http.server import HTTPServer, BaseHTTPRequestHandler
import json

def read_secret(env_var, file_env_var=None):
    """Read secret from environment or file."""
    # Try file-based secret first (Docker secrets pattern)
    if file_env_var:
        file_path = os.environ.get(file_env_var)
        if file_path and os.path.exists(file_path):
            with open(file_path, 'r') as f:
                return f.read().strip()

    # Fall back to environment variable
    value = os.environ.get(env_var)
    if not value:
        print(f"FATAL: Neither {file_env_var} nor {env_var} is set", file=sys.stderr)
        sys.exit(1)
    return value

# Read secrets at startup
DB_PASSWORD = read_secret('DB_PASSWORD', 'DB_PASSWORD_FILE')
API_KEY = read_secret('API_KEY', 'API_KEY_FILE')

print(f"[STARTUP] DB password loaded: {'*' * len(DB_PASSWORD)} ({len(DB_PASSWORD)} chars)")
print(f"[STARTUP] API key loaded: {API_KEY[:8]}...{API_KEY[-4:]}")

class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        response = {
            'status': 'ok',
            'db_password_length': len(DB_PASSWORD),
            'api_key_prefix': API_KEY[:8] + '...',
            'message': 'Secrets loaded successfully'
        }
        self.wfile.write(json.dumps(response).encode())

HTTPServer(('0.0.0.0', 8080), Handler).serve_forever()
```

**Dockerfile:**

```dockerfile
FROM python:3.11-alpine
WORKDIR /app
COPY app.py .
EXPOSE 8080
CMD ["python", "app.py"]
```

#### Step 3: Level 1 — Naive (DON'T DO THIS)

**docker-compose.naive.yml:**

```yaml
version: '3.8'

services:
  app:
    build: .
    ports:
      - "8081:8080"
    environment:
      # NEVER do this — visible in docker inspect
      - DB_PASSWORD=super-secret-database-password-123!
      - API_KEY=sk_live_a1b2c3d4e5f6g7h8i9j0
```

```bash
# Start it
docker compose -f docker-compose.naive.yml up -d

# Test it
curl http://localhost:8081

# See the problem
docker inspect secrets-lab-app-1 | grep -A 10 "Env"
# All secrets visible in plain text!
```

#### Step 4: Level 2 — File-based secrets (Better)

**Create secret files:**

```bash
mkdir -p secrets
echo "super-secret-database-password-123!" > secrets/db_password.txt
echo "sk_live_a1b2c3d4e5f6g7h8i9j0" > secrets/api_key.txt
chmod 600 secrets/*.txt
```

**docker-compose.files.yml:**

```yaml
version: '3.8'

services:
  app:
    build: .
    ports:
      - "8082:8080"
    volumes:
      - ./secrets/db_password.txt:/run/secrets/db_password:ro
      - ./secrets/api_key.txt:/run/secrets/api_key:ro
    environment:
      - DB_PASSWORD_FILE=/run/secrets/db_password
      - API_KEY_FILE=/run/secrets/api_key
```

```bash
# Start it
docker compose -f docker-compose.files.yml up -d

# Test it
curl http://localhost:8082

# Secrets are NOT in docker inspect
docker inspect secrets-lab-app-1 | grep -A 10 "Env"
# Only shows the file paths, not the actual secrets
```

#### Step 5: Level 3 — Docker Compose secrets (Best for local dev)

**docker-compose.secrets.yml:**

```yaml
version: '3.8'

services:
  app:
    build: .
    ports:
      - "8083:8080"
    secrets:
      - db_password
      - api_key
    environment:
      - DB_PASSWORD_FILE=/run/secrets/db_password
      - API_KEY_FILE=/run/secrets/api_key

secrets:
  db_password:
    file: ./secrets/db_password.txt
  api_key:
    file: ./secrets/api_key.txt
```

```bash
# Start it
docker compose -f docker-compose.secrets.yml up -d

# Test it
curl http://localhost:8083

# Secrets are mounted as tmpfs (encrypted in memory)
docker exec secrets-lab-app-1 ls -la /run/secrets/
```

#### Step 6: Level 4 — External secret manager simulation

**docker-compose.vault.yml:**

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

  app:
    build: .
    ports:
      - "8084:8080"
    depends_on:
      - vault
    environment:
      - VAULT_ADDR=http://vault:8200
      - VAULT_TOKEN=myroot
    entrypoint: >
      sh -c '
        apk add --no-cache curl jq &&
        echo "Waiting for Vault..." &&
        sleep 2 &&
        export DB_PASSWORD=$$(curl -s http://vault:8200/v1/secret/data/myapp/db \
          -H "X-Vault-Token: myroot" | jq -r ".data.data.password") &&
        export API_KEY=$$(curl -s http://vault:8200/v1/secret/data/myapp/api \
          -H "X-Vault-Token: myroot" | jq -r ".data.data.key") &&
        python app.py
      '
```

```bash
# Start Vault
docker compose -f docker-compose.vault.yml up -d vault

# Store secrets in Vault
curl -X POST http://localhost:8200/v1/secret/data/myapp/db \
  -H "X-Vault-Token: myroot" \
  -H "Content-Type: application/json" \
  -d '{"data": {"password": "super-secret-database-password-123!"}}'

curl -X POST http://localhost:8200/v1/secret/data/myapp/api \
  -H "X-Vault-Token: myroot" \
  -H "Content-Type: application/json" \
  -d '{"data": {"key": "sk_live_a1b2c3d4e5f6g7h8i9j0"}}'

# Start the app (secrets fetched from Vault at runtime)
docker compose -f docker-compose.vault.yml up -d app

# Test it
curl http://localhost:8084
```

#### Step 7: Verify security

```bash
# Compare what docker inspect reveals for each approach
for port in 8081 8082 8083 8084; do
  echo "=== Port $port ==="
  curl -s http://localhost:$port | python -m json.tool
done

# Check docker inspect for each
for name in secrets-lab-app-1; do
  echo "=== $name ==="
  docker inspect $name | grep -E "(DB_PASSWORD|API_KEY)" || echo "No secrets visible"
done
```

#### Cleanup

```bash
docker compose -f docker-compose.naive.yml down -v
docker compose -f docker-compose.files.yml down -v
docker compose -f docker-compose.secrets.yml down -v
docker compose -f docker-compose.vault.yml down -v
rm -rf secrets/
```

---

## 6. Limitation

You now have secrets properly managed. But there is a new problem: **you have no visibility into what your containers are actually doing.**

- Is the application consuming too much memory?
- Is the database query latency increasing?
- Are containers restarting unexpectedly?
- Is network traffic normal or suspicious?

You configured your containers, but you cannot observe them. A production system without monitoring is flying blind.

---

## 7. Next Topic

**Module 15: Container Monitoring** — We will learn how to observe container health and performance using `docker stats`, cAdvisor, Prometheus, and Grafana. You will set up dashboards that show CPU, memory, network, and disk metrics for every container in your system.
