# Cheatsheet: Secrets Rotation

## Why Rotate Secrets
- Limit exposure if compromised
- Compliance requirements (PCI, SOC2)
- Reduce blast radius
- Best practice for security

## Rotation Strategies

| Strategy | When | Downtime |
|----------|------|----------|
| Time-based | Every 30/90 days | None with overlap |
| Event-based | On compromise | Brief |
| Manual | On demand | Depends |

## Zero-Downtime Rotation
```
1. Generate new secret
2. Update application to accept both old and new
3. Deploy with both secrets
4. Update external service to use new secret
5. Remove old secret from application
6. Deploy with new secret only
```

## HashiCorp Vault
```bash
# Start Vault
docker run -d --name vault -p 8200:8200 vault:latest

# Enable database secrets engine
vault secrets enable database

# Configure PostgreSQL
vault write database/config/postgres \
  plugin_name=postgresql-database-plugin \
  connection_url="postgresql://{{username}}:{{password}}@postgres:5432" \
  allowed_roles="readonly" \
  username="vault" \
  password="password"

# Generate credentials
vault read database/creds/readonly
```

## Automated Rotation Script
```bash
#!/bin/bash
# Rotate database password
NEW_PASSWORD=$(openssl rand -base64 32)

# Update in database
psql -c "ALTER USER appuser PASSWORD '$NEW_PASSWORD';"

# Update in secret manager
kubectl create secret generic db-creds \
  --from-literal=password="$NEW_PASSWORD" \
  --dry-run=client -o yaml | kubectl apply -f -

# Restart pods to pick up new secret
kubectl rollout restart deployment/my-app
```
