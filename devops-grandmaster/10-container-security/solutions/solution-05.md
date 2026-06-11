# Solution 05: Design a Container Security Policy for a Production Environment

## Part A: Container Security Policy Document

```markdown
# Container Security Policy

**Version:** 1.0
**Last Updated:** 2024-01-15
**Owner:** Platform Engineering Team

---

## 1. Image Requirements

### 1.1 Base Image Restrictions

**Requirement:** All production containers must use images from the approved base image list. Custom base images must be reviewed and approved by the platform team before use.

**Approved base images:**
- `node:XX-slim` or `node:XX-alpine` (Node.js applications)
- `python:XX-slim` or `python:XX-alpine` (Python applications)
- `golang:XX-alpine` (Go applications, multi-stage builds required)
- `nginx:XX-alpine` (Static content / reverse proxy)
- `postgres:XX` (Database, pinned to minor version)
- `redis:XX-alpine` (Cache, pinned to minor version)

**Rationale:** Minimal images have fewer installed packages, which means fewer potential vulnerabilities and a smaller attack surface. Alpine images are preferred where compatible.

**Verification:** `docker inspect <image> --format '{{.Config.Image}}'` should show an approved base.

### 1.2 Version Pinning

**Requirement:** All image tags must be pinned to a specific version. The `:latest` tag is prohibited in production.

```bash
# PROHIBITED
image: node:latest
image: postgres:latest

# REQUIRED
image: node:18.19-slim
image: postgres:16.2
```

**Rationale:** The `:latest` tag is a moving target. It can change between builds, breaking reproducibility and introducing untested code into production.

**Verification:** `grep -r "latest" docker-compose.yml` should return no results.

### 1.3 Vulnerability Scanning

**Requirement:** All images must be scanned with Trivy before deployment. No CRITICAL vulnerabilities are allowed. HIGH vulnerabilities must be documented and have a remediation plan within 7 days.

```bash
trivy image --exit-code 1 --severity CRITICAL my-app:v1.0.0
trivy image --severity HIGH my-app:v1.0.0  # Must be documented
```

**Rationale:** Known vulnerabilities are the most common attack vector. Scanning prevents deploying images with known exploitable flaws.

**Verification:** CI/CD pipeline must include a Trivy scan step that fails on CRITICAL findings.

### 1.4 Dockerfile Best Practices

**Requirement:** All Dockerfiles must follow these rules:

- Use a non-root user (UID 1001-1999 range).
- Order COPY instructions for optimal layer caching.
- Include a HEALTHCHECK instruction.
- Use `.dockerignore` to exclude `.git`, `.env`, `node_modules`, `__pycache__`.
- No secrets in ARG or ENV instructions.
- No `ADD` with remote URLs (use `COPY` and download in a RUN step).

**Rationale:** These practices reduce image size, improve build speed, and prevent common security mistakes.

**Verification:** `trivy config Dockerfile` should report no misconfigurations.

### 1.5 Image Signing

**Requirement:** All production images must be signed using Cosign. Unsigned images cannot be deployed to production.

```bash
cosign sign --key cosign.key myregistry.io/my-app:v1.0.0
cosign verify --key cosign.pub myregistry.io/my-app:v1.0.0
```

**Rationale:** Image signing ensures that images have not been tampered with between build and deployment. It provides a chain of trust from the build system to production.

**Verification:** Deployment pipeline must verify image signatures before deploying.

---

## 2. Runtime Requirements

### 2.1 User Requirements

**Requirement:** All containers must run as a non-root user. The user must be created in the Dockerfile with a specific UID in the range 1001-1999.

```dockerfile
RUN groupadd --gid 1001 appgroup && \
    useradd --uid 1001 --gid appgroup --shell /bin/sh --create-home appuser
USER appuser
```

```yaml
# docker-compose.yml
services:
  app:
    user: "1001:1001"
```

**Rationale:** Running as root gives an attacker full control over the container. Non-root users cannot install packages, modify system files, or exploit root-only kernel features.

**Verification:** `docker exec <container> whoami` must not return `root`.

### 2.2 Filesystem Requirements

**Requirement:** All containers must use a read-only filesystem. Writable paths must be explicitly declared as tmpfs or volumes.

```yaml
services:
  app:
    read_only: true
    tmpfs:
      - /tmp:size=100M,noexec,nosuid
      - /var/run:size=1M,noexec,nosuid
    volumes:
      - app-data:/app/data  # Only if persistent storage is needed
```

**Rationale:** A read-only filesystem prevents attackers from modifying binaries, installing tools, or writing malware to disk. Explicit writable paths ensure that every write location is intentional.

**Verification:** `docker exec <container> sh -c 'echo test > /usr/bin/nope'` must fail.

### 2.3 Capability Policy

**Requirement:** All containers must drop all capabilities and add back only what is explicitly needed.

```yaml
services:
  app:
    cap_drop:
      - ALL
    # cap_add: []  -- only add if documented justification exists
```

**Allowed exceptions (with justification):**
- `NET_BIND_SERVICE`: Only if the container binds to ports below 1024. Prefer using ports above 1024 with a reverse proxy.
- `CHOWN`, `SETGID`, `SETUID`, `FOWNER`, `DAC_OVERRIDE`: Only for database containers that require these for initialization.
- `SYS_PTRACE`: Only for debugging containers (never in production).

**Rationale:** Capabilities are granular kernel privileges. Dropping all and adding back the minimum follows the principle of least privilege.

**Verification:** `docker exec <container> cat /proc/1/status | grep CapEff` should show `0000000000000000` for most containers.

### 2.4 Security Profiles

**Requirement:** All production containers must have:
- `no-new-privileges: true` set.
- The default Docker seccomp profile applied (or a custom, more restrictive one).

```yaml
services:
  app:
    security_opt:
      - no-new-privileges:true
      # - seccomp:./custom-seccomp.json  # Optional: more restrictive
```

**Rationale:** `no-new-privileges` prevents processes from gaining additional privileges through setuid binaries. The seccomp profile restricts which system calls the process can make.

**Verification:** `docker inspect <container> --format '{{.HostConfig.SecurityOpt}}'` must include `no-new-privileges:true`.

### 2.5 Resource Limits

**Requirement:** All containers must have memory and CPU limits set.

```yaml
services:
  app:
    deploy:
      resources:
        limits:
          memory: 512M
          cpus: '0.5'
        reservations:
          memory: 128M
          cpus: '0.1'
```

**Rationale:** Without resource limits, a single container can consume all host resources (memory exhaustion, CPU saturation, fork bombs), causing denial of service for all other containers.

**Verification:** `docker inspect <container> --format '{{.HostConfig.Memory}}'` must not be `0`.

---

## 3. Network Requirements

### 3.1 Network Segmentation

**Requirement:** Containers must be placed on separate networks based on their role:
- `frontend`: For containers that serve external traffic.
- `backend`: For internal service-to-service communication. Must be marked `internal: true`.

```yaml
networks:
  frontend:
    driver: bridge
  backend:
    driver: bridge
    internal: true
```

**Rationale:** Network segmentation limits the blast radius of a compromise. If a web container is compromised, the attacker cannot directly reach the database if it is on a separate internal network.

**Verification:** `docker network inspect backend | grep Internal` must return `true`.

### 3.2 Port Exposure

**Requirement:** Only the application's external port should be exposed to the host. Internal service ports (databases, caches) must not be exposed.

```yaml
services:
  app:
    ports:
      - "3000:3000"  # External access
  db:
    # No ports: section -- accessible only via backend network
```

**Rationale:** Exposing database ports to the host means any process on the host (or any container with host network access) can connect to them. Internal services should only be accessible through the internal network.

**Verification:** `docker port <db-container>` should return nothing.

### 3.3 Host Network Mode

**Requirement:** The `host` network mode is prohibited. All containers must use bridge or custom networks.

**Rationale:** Host network mode removes network namespace isolation. The container shares the host's network stack, meaning it can bind to any port, access any host service, and sniff host traffic.

**Verification:** `docker inspect <container> --format '{{.HostConfig.NetworkMode}}'` must not be `host`.

---

## 4. Secrets Management

### 4.1 Secret Delivery

**Requirement:** Secrets must be delivered to containers as files mounted at `/run/secrets/`. Environment variables are prohibited for secret values.

```yaml
services:
  app:
    secrets:
      - db_password
      - api_key
    environment:
      DB_PASSWORD_FILE: /run/secrets/db_password  # Path, not value
      API_KEY_FILE: /run/secrets/api_key

secrets:
  db_password:
    file: ./secrets/db_password.txt
  api_key:
    file: ./secrets/api_key.txt
```

**Rationale:** Environment variables are visible in `docker inspect`, `/proc/*/environ`, process listings, and logs. File-based secrets are mounted as tmpfs and are not persisted in container metadata.

**Verification:** `docker inspect <container> --format '{{.Config.Env}}'` must not contain any secret values.

### 4.2 Forbidden Practices

**Requirement:** The following are prohibited:
- Secrets in Dockerfile `ENV` or `ARG` instructions.
- Secrets in docker-compose.yml `environment:` values.
- Secrets in CI/CD pipeline variables that are logged.
- Secrets in application code or configuration files committed to version control.

**Rationale:** Any secret that appears in a build artifact, log, or version control history is permanently compromised. Even if deleted from the current version, it persists in git history.

**Verification:** `docker history <image> --no-trunc | grep -iE 'PASSWORD|SECRET|KEY|TOKEN'` must return nothing.

### 4.3 Secret Rotation

**Requirement:** Secrets must be rotated at least every 90 days. The rotation process must be documented and tested.

**Rationale:** Regular rotation limits the window of exposure if a secret is compromised. If a secret is leaked but rotated within 90 days, the attacker's window of usefulness is limited.

**Verification:** Secret metadata (creation date) must be tracked. Secrets older than 90 days must trigger an alert.

---

## 5. Monitoring and Audit

### 5.1 Logging

**Requirement:** All containers must have logging configured with size limits.

```yaml
services:
  app:
    logging:
      driver: json-file
      options:
        max-size: "10m"
        max-file: "3"
```

**Rationale:** Without log limits, a container can fill the host's disk with log output, causing denial of service. Logs must also not contain secrets.

**Verification:** `docker inspect <container> --format '{{.HostConfig.LogConfig}}'` must show size limits.

### 5.2 Compliance Audit

**Requirement:** The `audit-containers.sh` script must be run weekly against all production containers. Results must be reviewed by the platform team.

**Rationale:** Continuous auditing ensures that containers remain compliant over time. Configuration drift is common -- a container that was compliant at deployment may not be compliant after a restart or update.

**Verification:** Audit reports must be stored and reviewed. Non-compliant containers must be remediated within 48 hours.

### 5.3 Incident Response

**Requirement:** If a container is found to be compromised:
1. Isolate the container (disconnect from networks).
2. Preserve forensic evidence (container filesystem, logs).
3. Rotate all secrets that the container had access to.
4. Rebuild and redeploy from a known-good image.
5. Conduct a post-incident review within 48 hours.

**Rationale:** A clear incident response plan reduces the time to contain a breach and prevents the attacker from maintaining access through compromised secrets or images.
```

---

## Part B: Reference docker-compose.template.yml

```yaml
# =============================================================
# Golden Template: Container Security Policy-Compliant Compose
# =============================================================
# This template is the starting point for all new services.
# Customize the marked sections for your application.
# Every security setting is annotated with the policy section.

version: "3.8"

services:
  # ---------------------------------------------------------
  # APPLICATION SERVICE
  # Customize: image name, ports, secrets, health check
  # ---------------------------------------------------------
  app:
    # POLICY 1.2: Pinned version, not :latest
    image: YOUR_REGISTRY/YOUR_APP:v1.0.0
    build:
      context: .
      dockerfile: Dockerfile

    # POLICY 2.1: Non-root user
    user: "1001:1001"

    # POLICY 2.2: Read-only filesystem
    read_only: true

    # POLICY 2.3: Drop all capabilities
    cap_drop:
      - ALL
    # POLICY 2.3: Add back ONLY if documented justification exists
    # cap_add:
    #   - NET_BIND_SERVICE  # Only if binding to port < 1024

    # POLICY 2.4: Security options
    security_opt:
      - no-new-privileges:true
      # - seccomp:./seccomp-profile.json  # Optional custom profile

    # POLICY 2.2: Writable paths (explicit)
    tmpfs:
      - /tmp:size=100M,noexec,nosuid

    # POLICY 3.1: Network segmentation
    networks:
      - frontend
      - backend

    # POLICY 3.2: Only external ports exposed
    ports:
      - "3000:3000"  # Customize to your app's port

    # POLICY 4.1: Secrets as files, not ENV
    secrets:
      - db_password
      - api_key
    environment:
      DB_PASSWORD_FILE: /run/secrets/db_password
      API_KEY_FILE: /run/secrets/api_key
      NODE_ENV: production  # Non-secret config is OK in ENV

    # POLICY 2.5: Resource limits
    deploy:
      resources:
        limits:
          memory: 512M
          cpus: '0.5'
        reservations:
          memory: 128M
          cpus: '0.1'

    # Application health check (customize the URL)
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost:3000/health"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 10s

    # POLICY 5.1: Logging with limits
    logging:
      driver: json-file
      options:
        max-size: "10m"
        max-file: "3"

    restart: unless-stopped

  # ---------------------------------------------------------
  # DATABASE SERVICE
  # Customize: image version, secrets, resource limits
  # ---------------------------------------------------------
  db:
    # POLICY 1.2: Pinned to minor version
    image: postgres:16

    # POLICY 2.1: Non-root (postgres user)
    user: "999:999"

    # POLICY 2.2: Read-only filesystem
    read_only: true

    # POLICY 2.4: Security options
    security_opt:
      - no-new-privileges:true

    # POLICY 2.3: Drop all, add back minimum for postgres
    cap_drop:
      - ALL
    cap_add:
      - CHOWN
      - DAC_OVERRIDE
      - FOWNER
      - SETGID
      - SETUID

    # POLICY 2.2: Writable paths
    tmpfs:
      - /tmp:size=50M,noexec,nosuid
      - /var/run/postgresql:size=1M

    volumes:
      - pg-data:/var/lib/postgresql/data

    # POLICY 3.1: Backend network only (no external access)
    networks:
      - backend
    # POLICY 3.2: No ports exposed to host

    # POLICY 4.1: Secrets as files
    secrets:
      - db_password
    environment:
      POSTGRES_PASSWORD_FILE: /run/secrets/db_password

    shm_size: '256mb'

    # POLICY 2.5: Resource limits
    deploy:
      resources:
        limits:
          memory: 1G
          cpus: '1.0'
        reservations:
          memory: 256M
          cpus: '0.25'

    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 30s
      timeout: 5s
      retries: 3

    # POLICY 5.1: Logging with limits
    logging:
      driver: json-file
      options:
        max-size: "10m"
        max-file: "3"

    restart: unless-stopped

# ---------------------------------------------------------
# NETWORKS
# POLICY 3.1: Segmented networks
# ---------------------------------------------------------
networks:
  frontend:
    driver: bridge
  backend:
    driver: bridge
    internal: true  # No external access

# ---------------------------------------------------------
# VOLUMES
# ---------------------------------------------------------
volumes:
  pg-data:
    driver: local

# ---------------------------------------------------------
# SECRETS
# POLICY 4.1: File-based secrets
# Create a secrets/ directory with these files.
# Add secrets/ to .gitignore.
# ---------------------------------------------------------
secrets:
  db_password:
    file: ./secrets/db_password.txt
  api_key:
    file: ./secrets/api_key.txt
```

---

## Part C: Reference Dockerfile.template

```dockerfile
# =============================================================
# Golden Template: Container Security Policy-Compliant Dockerfile
# =============================================================
# This template is the starting point for all new services.
# Customize the marked sections for your application.
# Every security decision is annotated with the policy section.

# ---------------------------------------------------------
# Stage 1: Build (if needed)
# For applications that need compilation or dependency installation.
# Remove this stage for interpreted languages that do not need it.
# ---------------------------------------------------------
FROM node:18-slim AS builder

# POLICY 1.4: .dockerignore must exclude .git, .env, node_modules
# Create a .dockerignore file in your project root:
#   .git
#   .env
#   .env.*
#   node_modules
#   __pycache__
#   *.pyc
#   .DS_Store

WORKDIR /build

# Copy dependency files first (layer caching)
COPY package*.json ./

# Install all dependencies (including dev for building)
RUN npm ci

# Copy source code
COPY . .

# Build step (if applicable)
# RUN npm run build

# ---------------------------------------------------------
# Stage 2: Runtime
# Minimal image with only runtime dependencies.
# ---------------------------------------------------------
FROM node:18-slim

# POLICY 1.1: Use approved minimal base image
# POLICY 1.2: Pin to specific version (not :latest)

# Install only the runtime OS packages needed
# (Add packages here if your app needs them)
# RUN apt-get update && \
#     apt-get install -y --no-install-recommends curl && \
#     rm -rf /var/lib/apt/lists/*

# POLICY 2.1: Create non-root user with specific UID
RUN groupadd --gid 1001 appgroup && \
    useradd --uid 1001 --gid appgroup --shell /bin/sh --create-home appuser

WORKDIR /app

# Copy dependency files and install production dependencies only
COPY --from=builder --chown=appuser:appgroup package*.json ./
RUN npm ci --production && \
    npm cache clean --force

# Copy built application from builder stage
COPY --from=builder --chown=appuser:appgroup . .

# POLICY 1.4: Remove sensitive files that might have been copied
RUN rm -rf .git .env .env.* .npmrc

# POLICY 2.1: Switch to non-root user BEFORE CMD
USER appuser

EXPOSE 3000

# POLICY 1.4: Health check is required
HEALTHCHECK --interval=30s --timeout=5s --retries=3 --start-period=10s \
  CMD ["node", "-e", "require('http').get('http://localhost:3000/health', (r) => { process.exit(r.statusCode === 200 ? 0 : 1) }).on('error', () => process.exit(1))"]

# POLICY 4.2: No secrets in CMD, ENTRYPOINT, or ENV
CMD ["node", "server.js"]
```

**Required .dockerignore file:**

```
.git
.gitignore
.env
.env.*
node_modules
npm-debug.log*
__pycache__
*.pyc
.DS_Store
Thumbs.db
*.md
docker-compose*.yml
Dockerfile*
.dockerignore
secrets/
```

---

## Part D: Compliance Audit Script

```bash
#!/bin/bash
# audit-containers.sh -- Audit all running containers against the security policy
set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

TOTAL_CHECKS=0
TOTAL_PASSES=0
TOTAL_FAILURES=0
DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

pass() {
    echo -e "  ${GREEN}[PASS]${NC} $1"
    TOTAL_PASSES=$((TOTAL_PASSES + 1))
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
}

fail() {
    echo -e "  ${RED}[FAIL]${NC} $1"
    TOTAL_FAILURES=$((TOTAL_FAILURES + 1))
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
}

audit_container() {
    local container="$1"
    local container_passes=0
    local container_total=0

    echo -e "${BLUE}Container: ${container}${NC}"

    # -------------------------------------------------------
    # Check 1: Non-root user
    # -------------------------------------------------------
    container_total=$((container_total + 1))
    local user
    user=$(docker exec "$container" whoami 2>/dev/null || echo "unknown")
    if [ "$user" = "root" ]; then
        fail "Running as root"
    else
        pass "Running as: $user"
        container_passes=$((container_passes + 1))
    fi

    # -------------------------------------------------------
    # Check 2: Capabilities dropped
    # -------------------------------------------------------
    container_total=$((container_total + 1))
    local caps
    caps=$(docker exec "$container" cat /proc/1/status 2>/dev/null | grep CapEff | awk '{print $2}' || echo "unknown")
    if [ "$caps" = "0000000000000000" ]; then
        pass "All capabilities dropped"
        container_passes=$((container_passes + 1))
    elif [ "$caps" = "unknown" ]; then
        fail "Cannot read capabilities"
    else
        fail "Capabilities not fully dropped (CapEff: $caps)"
    fi

    # -------------------------------------------------------
    # Check 3: Read-only filesystem
    # -------------------------------------------------------
    container_total=$((container_total + 1))
    if docker exec "$container" sh -c 'echo test > /usr/bin/test-write 2>/dev/null' 2>/dev/null; then
        docker exec "$container" rm -f /usr/bin/test-write 2>/dev/null
        fail "Filesystem is writable"
    else
        pass "Filesystem is read-only (or /usr/bin is protected)"
        container_passes=$((container_passes + 1))
    fi

    # -------------------------------------------------------
    # Check 4: No secrets in environment variables
    # -------------------------------------------------------
    container_total=$((container_total + 1))
    local env_secrets
    env_secrets=$(docker inspect "$container" --format '{{.Config.Env}}' | grep -iE 'PASSWORD=|SECRET=|API_KEY=|TOKEN=|PRIVATE_KEY=' || true)
    if [ -n "$env_secrets" ]; then
        fail "Potential secrets in environment variables"
    else
        pass "No obvious secrets in environment variables"
        container_passes=$((container_passes + 1))
    fi

    # -------------------------------------------------------
    # Check 5: No Docker socket mount
    # -------------------------------------------------------
    container_total=$((container_total + 1))
    local sock_mount
    sock_mount=$(docker inspect "$container" --format '{{range .Mounts}}{{.Source}} {{end}}' | grep docker.sock || true)
    if [ -n "$sock_mount" ]; then
        fail "Docker socket is mounted"
    else
        pass "No Docker socket mount"
        container_passes=$((container_passes + 1))
    fi

    # -------------------------------------------------------
    # Check 6: Resource limits set
    # -------------------------------------------------------
    container_total=$((container_total + 1))
    local mem_limit
    mem_limit=$(docker inspect "$container" --format '{{.HostConfig.Memory}}')
    if [ "$mem_limit" = "0" ]; then
        fail "No memory limit set"
    else
        local mem_mb=$((mem_limit / 1024 / 1024))
        pass "Memory limit: ${mem_mb}MB"
        container_passes=$((container_passes + 1))
    fi

    # -------------------------------------------------------
    # Check 7: Isolated network (not host mode)
    # -------------------------------------------------------
    container_total=$((container_total + 1))
    local net_mode
    net_mode=$(docker inspect "$container" --format '{{.HostConfig.NetworkMode}}')
    if [ "$net_mode" = "host" ]; then
        fail "Using host network mode"
    else
        pass "Network mode: $net_mode"
        container_passes=$((container_passes + 1))
    fi

    # -------------------------------------------------------
    # Check 8: Not privileged
    # -------------------------------------------------------
    container_total=$((container_total + 1))
    local privileged
    privileged=$(docker inspect "$container" --format '{{.HostConfig.Privileged}}')
    if [ "$privileged" = "true" ]; then
        fail "Running in privileged mode"
    else
        pass "Not privileged"
        container_passes=$((container_passes + 1))
    fi

    # Container summary
    local pct=$((container_passes * 100 / container_total))
    if [ "$pct" -eq 100 ]; then
        echo -e "  Compliance: ${GREEN}${container_passes}/${container_total} (${pct}%)${NC}"
    elif [ "$pct" -ge 75 ]; then
        echo -e "  Compliance: ${YELLOW}${container_passes}/${container_total} (${pct}%)${NC}"
    else
        echo -e "  Compliance: ${RED}${container_passes}/${container_total} (${pct}%)${NC}"
    fi
    echo ""
}

# -------------------------------------------------------
# Main
# -------------------------------------------------------
echo -e "${BLUE}=== Container Security Compliance Report ===${NC}"
echo "Date: $DATE"
echo ""

# Get all running containers
CONTAINERS=$(docker ps --format '{{.Names}}' 2>/dev/null)

if [ -z "$CONTAINERS" ]; then
    echo "No running containers found."
    exit 0
fi

# Audit each container
for container in $CONTAINERS; do
    audit_container "$container"
done

# -------------------------------------------------------
# Overall summary
# -------------------------------------------------------
echo -e "${BLUE}=== Overall Summary ===${NC}"
echo "Containers audited: $(echo "$CONTAINERS" | wc -l | tr -d ' ')"
echo "Checks: $TOTAL_PASSES/$TOTAL_CHECKS passed"

if [ "$TOTAL_FAILURES" -gt 0 ]; then
    OVERALL_PCT=$((TOTAL_PASSES * 100 / TOTAL_CHECKS))
    echo -e "Result: ${RED}$TOTAL_FAILURES check(s) failed${NC} (${OVERALL_PCT}% compliance)"
    exit 1
else
    echo -e "Result: ${GREEN}All checks passed${NC} (100% compliance)"
    exit 0
fi
```

---

## Common Mistakes

- **Writing a policy without templates:** A policy document that is not backed by easy-to-use templates will be ignored. Developers will default to what is easiest. Provide templates that make compliance the path of least resistance.
- **Too many exceptions:** If every team has an exception to the policy, the policy is ineffective. Design the policy to be achievable for 90% of cases, and require a formal review process for exceptions.
- **Not automating the audit:** A manual audit is inconsistent and happens too infrequently. Automate it and run it weekly (or on every deployment).
- **Focusing only on build-time security:** Security must cover the full lifecycle: build (Dockerfile best practices), ship (image signing, scanning), and run (capabilities, network, secrets). A secure image that runs with the Docker socket mounted is not secure.
- **Ignoring operational requirements:** The policy must include logging, monitoring, and incident response. A security incident that is not detected or responded to is worse than one that never happened.
