# Cheatsheet: Secrets Management

## Problem with Environment Variables
```bash
# Secrets visible in:
docker inspect my-app          # Shows all env vars
docker exec my-app env         # Shows all env vars
/proc/1/environ               # Can be read
```

## Solutions

### 1. File-Based Secrets
```bash
# Mount secret as file
docker run -v /path/to/secret:/run/secrets/my-secret:ro my-app

# Read in application
secret = open('/run/secrets/my-secret').read().strip()
```

### 2. Docker Secrets (Swarm)
```bash
echo "my-secret" | docker secret create my-secret -
docker service create --secret my-secret my-app
# Available at /run/secrets/my-secret
```

### 3. Kubernetes Secrets
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: app-secrets
type: Opaque
data:
  PASSWORD: cGFzc3dvcmQ=
---
# Mount as volume
volumeMounts:
  - name: secrets
    mountPath: /etc/secrets
    readOnly: true
volumes:
  - name: secrets
    secret:
      secretName: app-secrets
```

### 4. External Secret Managers
```
HashiCorp Vault    → Best for complex secret management
AWS Secrets Manager → Best for AWS-native
GCP Secret Manager  → Best for GCP-native
Azure Key Vault     → Best for Azure-native
```

## Never Do This
```bash
# Don't commit secrets to git
echo "password123" > .env
git add .env  # ❌

# Don't bake into image
ENV DB_PASSWORD=secret  # ❌

# Don't pass as command line args
docker run my-app --password=secret  # ❌ (visible in ps)
```
