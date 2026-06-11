# Cheatsheet: Container Security

## Security Checklist

- [ ] Run as non-root user
- [ ] Use read-only filesystem
- [ ] Drop all capabilities, add only needed
- [ ] Scan images for vulnerabilities
- [ ] Use specific image tags (not latest)
- [ ] Sign images
- [ ] Use secrets management (not env vars)
- [ ] Set resource limits
- [ ] Use security profiles (AppArmor, Seccomp)
- [ ] No privileged containers

## Dockerfile Security
```dockerfile
# Use specific, minimal base image
FROM python:3.11-slim-bookworm

# Create non-root user
RUN adduser --disabled-password --gecos '' appuser

# Set read-only filesystem
RUN chmod -R 555 /app

# Switch to non-root user
USER appuser

# Don't expose unnecessary ports
EXPOSE 8080
```

## Runtime Security
```bash
# Run as non-root
docker run --user 1000:1000 my-app

# Read-only filesystem
docker run --read-only --tmpfs /tmp my-app

# Drop capabilities
docker run --cap-drop ALL --cap-add NET_BIND_SERVICE my-app

# No new privileges
docker run --security-opt no-new-privileges my-app

# Resource limits
docker run --memory 256m --cpus 0.5 my-app

# Security profile
docker run --security-opt apparmor=my-profile my-app
```

## Image Scanning
```bash
# Trivy
trivy image my-app:1.0

# Docker Scout
docker scout cves my-app:1.0
```
