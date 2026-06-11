# Cheatsheet: Image Versioning

## Tag Strategies

| Strategy | Example | Best For |
|----------|---------|----------|
| Semantic version | v1.0.0, v1.2.3 | Releases |
| Git SHA | sha-abc1234 | CI/CD builds |
| Date | 2024-01-15 | Nightly builds |
| Branch | main, develop | Branch builds |
| latest | latest | Development only |

## Best Practices
```bash
# BAD — unpredictable
docker build -t my-app:latest .

# GOOD — semantic version
docker build -t my-app:1.0.0 .

# GOOD — git SHA
docker build -t my-app:sha-$(git rev-parse --short HEAD) .

# GOOD — multiple tags
docker build -t my-app:1.0.0 -t my-app:latest .
```

## Rollback
```bash
# Deploy specific version
kubectl set image deployment/my-app my-app=my-app:1.0.0

# Rollback to previous
kubectl rollout undo deployment/my-app

# Rollback to specific version
kubectl rollout undo deployment/my-app --to-revision=2
```

## Pin Base Images
```dockerfile
# BAD — could change
FROM python:latest

# GOOD — pinned
FROM python:3.11.4-slim-bookworm

# GOOD — pinned with digest
FROM python:3.11.4-slim-bookworm@sha256:abc123...
```
