# Module 10: Container Security — Running Containers Safely

> **Previous Module:** [09 - Volumes & Data](../09-volumes-and-data/README.md)
> **Next Module:** 11 - (Coming Soon)

---

## 1. The Problem — Containers Are Not Sandboxes

A common misconception is that containers provide strong isolation like virtual machines. They do not. Containers share the host's kernel. A container that can exploit a kernel vulnerability can escape to the host. A container running as root inside is running as root on the host's kernel. A container with all Linux capabilities enabled can do nearly anything a root process on the host can do.

**The default Docker container is insecure:**

```bash
# Default container runs as root
docker run --rm alpine whoami
# Output: root

# Default container has a writable filesystem
docker run --rm alpine sh -c 'echo "malware" > /usr/bin/real-tool'

# Default container has significant capabilities
docker run --rm alpine sh -c 'cat /proc/1/status | grep Cap'
# CapEff: 00000000a80425fb  (many capabilities enabled)

# Default container can access the network freely
docker run --rm alpine sh -c 'apk add curl && curl http://169.254.169.254/latest/meta-data/'
# Can reach the cloud metadata service — a major security risk
```

Every one of these defaults is a potential attack vector. In production, every one of them must be hardened.

---

## 2. The Naive Way — Relying on Defaults

The naive approach is to run containers with default settings and assume the container boundary is sufficient.

```bash
# Running a web app as root, with writable filesystem and full capabilities
docker run -d \
  -p 8080:80 \
  -e DATABASE_PASSWORD=supersecret \
  -v /var/run/docker.sock:/var/run/docker.sock \
  my-web-app:latest

# Why this is dangerous:
# 1. Root user — if the app is compromised, attacker has root in the container
# 2. Writable filesystem — attacker can modify binaries, install tools
# 3. Docker socket mounted — attacker can escape to the host entirely
# 4. Password in ENV — visible in docker inspect, process listing, logs
# 5. Latest tag — uncontrolled image version, no vulnerability scanning
```

**Why this fails:**
- A single application vulnerability gives the attacker root access
- The Docker socket mount is equivalent to root on the host
- Secrets in environment variables are leaked through monitoring tools, logs, and `docker inspect`
- No defense in depth — one breach compromises everything

---

## 3. The Right Way — Defense in Depth

Container security is about layers. No single measure is sufficient, but combined they create meaningful barriers.

### 3.1 Run as Non-Root User

The single most impactful security improvement. If the process inside the container does not run as root, a compromised container cannot perform privileged operations.

```dockerfile
# Dockerfile — create a non-root user
FROM node:20-alpine

# Create a dedicated user
RUN addgroup -g 1001 appgroup && \
    adduser -u 1001 -G appgroup -s /bin/sh -D appuser

# Set up the application
WORKDIR /app
COPY --chown=appuser:appgroup package*.json ./
RUN npm ci --production
COPY --chown=appuser:appgroup . .

# Switch to non-root user BEFORE CMD
USER appuser

EXPOSE 3000
CMD ["node", "server.js"]
```

```bash
# Verify the container runs as non-root
docker run --rm my-app:latest whoami
# Output: appuser

# In docker-compose
services:
  app:
    image: my-app:latest
    user: "1001:1001"  # Override if needed
    security_opt:
      - no-new-privileges:true  # Prevent privilege escalation
```

**Why `no-new-privileges` matters:** Without it, a process inside the container can use setuid binaries or other mechanisms to escalate privileges. With it, no process can gain more privileges than its parent.

### 3.2 Read-Only Filesystem

If the container's filesystem is read-only, an attacker cannot modify binaries, install tools, or write malware to disk.

```yaml
services:
  app:
    image: my-app:latest
    read_only: true
    tmpfs:
      - /tmp:size=100M,noexec,nosuid     # Writable temp space
      - /var/run:size=1M,noexec,nosuid    # PID files, sockets
    volumes:
      - app-cache:/app/cache               # Explicit writable volume
```

```bash
# Verify read-only filesystem
docker run --rm --read-only alpine sh -c 'echo test > /tmp/ok && echo "tmpfs works"'
# Output: tmpfs works

docker run --rm --read-only alpine sh -c 'echo test > /usr/bin/nope'
# Output: sh: can't create /usr/bin/nope: Read-only file system
```

**Handling applications that need to write:** Some applications write to specific directories at runtime (logs, cache, PID files). Use tmpfs mounts for temporary data and named volumes for persistent data. The key is that every writable location is explicit and intentional.

### 3.3 Drop Capabilities

Linux capabilities are granular permissions. Docker grants a subset by default. Drop everything, then add back only what is needed.

```yaml
services:
  app:
    image: my-app:latest
    cap_drop:
      - ALL                # Drop all capabilities
    cap_add:
      - NET_BIND_SERVICE   # Only if binding to ports < 1024
    # For most web apps, no cap_add is needed if using ports > 1024
```

```bash
# See default capabilities
docker run --rm alpine sh -c 'cat /proc/1/status | grep -i cap'
# CapEff: 00000000a80425fb

# Drop all capabilities
docker run --rm --cap-drop=ALL alpine sh -c 'cat /proc/1/status | grep -i cap'
# CapEff: 0000000000000000

# Add back only what's needed
docker run --rm --cap-drop=ALL --cap-add=NET_BIND_SERVICE alpine sh -c \
  'cat /proc/1/status | grep -i cap'
# CapEff: 0000000000000400
```

**Common capabilities and when you need them:**

```
Capability           | What it allows                    | Usually needed?
---------------------+-----------------------------------+-----------------
NET_BIND_SERVICE     | Bind to ports < 1024              | Only if using port 80/443
                     |                                   | directly (prefer > 1024)
CHOWN                | Change file ownership             | Rarely
DAC_OVERRIDE         | Bypass file permissions           | Rarely
FOWNER               | Bypass owner checks               | Rarely
SETGID/SETUID        | Change process GID/UID            | Rarely (if running non-root)
NET_RAW              | Use raw sockets (ping, ARP)       | Only for network tools
SYS_ADMIN            | Mount filesystems, namespaces,    | NEVER — this is near-root
                     | cgroups, etc.                     |
SYS_PTRACE           | Trace/debug processes             | Only for debugging tools
```

### 3.4 Security Profiles

**Seccomp (Secure Computing Mode):**

Seccomp restricts which system calls a process can make. Docker's default seccomp profile blocks about 44 of the 300+ syscalls.

```json
// custom-seccomp.json — minimal profile for a web app
{
  "defaultAction": "SCMP_ACT_ERRNO",
  "architectures": ["SCMP_ARCH_X86_64"],
  "syscalls": [
    {
      "names": [
        "accept4", "access", "bind", "brk", "chdir", "chmod",
        "clock_gettime", "close", "connect", "dup", "dup2",
        "epoll_create1", "epoll_ctl", "epoll_wait", "execve",
        "exit", "exit_group", "fchmod", "fchown", "fcntl",
        "fstat", "futex", "getdents64", "getegid", "geteuid",
        "getgid", "getpid", "getppid", "getsockname", "gettid",
        "gettimeofday", "ioctl", "listen", "lseek", "madvise",
        "mmap", "mprotect", "munmap", "nanosleep", "newfstatat",
        "openat", "pipe2", "poll", "prctl", "pread64", "pwrite64",
        "read", "readlink", "recvfrom", "recvmsg", "rename",
        "rt_sigaction", "rt_sigprocmask", "rt_sigreturn",
        "sendmsg", "sendto", "set_robust_list", "set_tid_address",
        "setsockopt", "shutdown", "sigaltstack", "socket",
        "stat", "statfs", "tgkill", "umask", "uname",
        "unlink", "wait4", "write", "writev"
      ],
      "action": "SCMP_ACT_ALLOW"
    }
  ]
}
```

```yaml
services:
  app:
    image: my-app:latest
    security_opt:
      - seccomp:custom-seccomp.json
```

**AppArmor:**

AppArmor restricts file access, network access, and capabilities at the kernel level.

```
# /etc/apparmor.d/docker-custom
#include <tunables/global>

profile docker-custom flags=(attach_disconnected) {
  #include <abstractions/base>

  # Deny write to most of the filesystem
  deny /etc/** w,
  deny /usr/** w,
  deny /bin/** w,
  deny /sbin/** w,

  # Allow read to app directory
  /app/** r,

  # Allow write to tmp
  /tmp/** rw,

  # Allow network access
  network inet stream,
  network inet dgram,

  # Deny raw sockets
  deny network raw,
}
```

```yaml
services:
  app:
    image: my-app:latest
    security_opt:
      - apparmor:docker-custom
```

---

## 4. The Production Way — Comprehensive Security Posture

### 4.1 Image Scanning

Scan images for known vulnerabilities before deploying them.

**Trivy:**

```bash
# Install Trivy
# macOS: brew install trivy
# Linux: apt-get install trivy / yum install trivy

# Scan an image
trivy image postgres:16

# Scan with severity filter
trivy image --severity HIGH,CRITICAL nginx:latest

# Scan and fail on critical vulnerabilities (CI/CD)
trivy image --exit-code 1 --severity CRITICAL my-app:latest

# Scan a Dockerfile before building
trivy config Dockerfile

# Scan in CI/CD pipeline
trivy image \
  --format json \
  --output trivy-report.json \
  --severity HIGH,CRITICAL \
  my-app:latest
```

**Docker Scout (built into Docker Desktop):**

```bash
# Enable Scout
docker scout quickview my-app:latest

# Detailed vulnerability report
docker scout cves my-app:latest

# Compare two image versions
docker scout compare --to my-app:v1.0 my-app:v1.1
```

**Snyk:**

```bash
# Install and authenticate
npm install -g snyk
snyk auth

# Scan a container image
snyk container test my-app:latest

# Monitor (track vulnerabilities over time)
snyk container monitor my-app:latest
```

### 4.2 Image Signing with Docker Content Trust

Docker Content Trust (DCT) ensures images are signed by trusted publishers and have not been tampered with.

```bash
# Enable Content Trust
export DOCKER_CONTENT_TRUST=1

# Push a signed image
docker push myregistry/my-app:v1.0
# Docker will prompt for a signing key passphrase

# Pull only signed images
docker pull myregistry/my-app:v1.0
# Fails if the image is not signed

# Disable for specific commands (use sparingly)
DOCKER_CONTENT_TRUST=0 docker pull unsigned-image:latest
```

**In docker-compose with Content Trust:**

```yaml
services:
  app:
    image: myregistry/my-app:v1.0
    # With DOCKER_CONTENT_TRUST=1, compose will only pull signed images
```

**Cosign (Sigstore) — the modern alternative:**

```bash
# Install cosign
# brew install cosign / go install github.com/sigstore/cosign/v2/cmd/cosign@latest

# Generate a key pair
cosign generate-key-pair

# Sign an image
cosign sign --key cosign.key myregistry/my-app:v1.0

# Verify a signature
cosign verify --key cosign.pub myregistry/my-app:v1.0

# Sign with keyless (OIDC-based, for CI/CD)
cosign sign myregistry/my-app:v1.0
# Uses Fulcio for short-lived certificates
```

### 4.3 Secrets Management

**Never store secrets in environment variables.** They are visible in `docker inspect`, `/proc/*/environ`, process listings, and logs.

```bash
# BAD — secrets in ENV
docker run -e DB_PASSWORD=supersecret my-app
docker inspect <container> | grep DB_PASSWORD  # Visible!

# GOOD — Docker secrets (Swarm mode)
echo "supersecret" | docker secret create db_password -
docker service create --secret db_password my-app
# Secret is mounted as /run/secrets/db_password inside the container

# GOOD — File-based secrets in Compose
services:
  app:
    image: my-app:latest
    secrets:
      - db_password
      - api_key
    environment:
      DB_PASSWORD_FILE: /run/secrets/db_password
      API_KEY_FILE: /run/secrets/api_key

secrets:
  db_password:
    file: ./secrets/db_password.txt
  api_key:
    file: ./secrets/api_key.txt
```

**Application code to read secrets from files:**

```python
# Python example
import os

def read_secret(name):
    """Read secret from file, falling back to ENV (for local dev only)."""
    file_path = os.environ.get(f"{name}_FILE")
    if file_path and os.path.exists(file_path):
        with open(file_path) as f:
            return f.read().strip()
    return os.environ.get(name)

db_password = read_secret("DB_PASSWORD")
```

```rust
// Rust example
use std::fs;

fn read_secret(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    if let Ok(file_path) = std::env::var(format!("{}_FILE", name)) {
        return Ok(fs::read_to_string(&file_path)?.trim().to_string());
    }
    std::env::var(name).map_err(|e| e.into())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_password = read_secret("DB_PASSWORD")?;
    println!("Connected with password of length: {}", db_password.len());
    Ok(())
}
```

**HashiCorp Vault integration (production-grade):**

```yaml
services:
  app:
    image: my-app:latest
    environment:
      VAULT_ADDR: "http://vault:8200"
      VAULT_ROLE: "my-app"
    # Application authenticates to Vault and retrieves secrets at startup
    # Secrets are never stored on disk or in environment variables
```

### 4.4 Network Policies

Restrict which containers can communicate with each other.

```yaml
services:
  app:
    image: my-app:latest
    networks:
      - frontend
      - backend
    # Can reach: internet (via frontend), database (via backend)
    # Cannot reach: other app containers directly

  database:
    image: postgres:16
    networks:
      - backend
    # Only accessible from backend network

  redis:
    image: redis:7-alpine
    networks:
      - backend
    # Only accessible from backend network

networks:
  frontend:
    driver: bridge
  backend:
    driver: bridge
    internal: true  # No external access — containers on this network
                    # cannot reach the internet
```

### 4.5 Full Production Security Configuration

```yaml
version: "3.8"

services:
  app:
    image: myregistry/my-app:v1.2.3  # Pinned version, not :latest
    user: "1001:1001"
    read_only: true
    security_opt:
      - no-new-privileges:true
      - seccomp:./seccomp-profile.json
      - apparmor:docker-custom
    cap_drop:
      - ALL
    tmpfs:
      - /tmp:size=50M,noexec,nosuid
    networks:
      - frontend
      - backend
    secrets:
      - db_password
      - api_key
    environment:
      DB_PASSWORD_FILE: /run/secrets/db_password
      API_KEY_FILE: /run/secrets/api_key
    deploy:
      resources:
        limits:
          memory: 512M
          cpus: '0.5'
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost:3000/health"]
      interval: 30s
      timeout: 5s
      retries: 3
    logging:
      driver: json-file
      options:
        max-size: "10m"
        max-file: "3"

  database:
    image: postgres:16  # Pinned to minor version
    user: "999:999"     # postgres user
    read_only: true
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    cap_add:
      - CHOWN
      - DAC_OVERRIDE
      - FOWNER
      - SETGID
      - SETUID
    tmpfs:
      - /tmp:size=50M,noexec,nosuid
      - /var/run/postgresql:size=1M
    volumes:
      - pg-data:/var/lib/postgresql/data
    networks:
      - backend
    secrets:
      - db_password
    environment:
      POSTGRES_PASSWORD_FILE: /run/secrets/db_password
    shm_size: '256mb'
    deploy:
      resources:
        limits:
          memory: 1G
          cpus: '1.0'

networks:
  frontend:
    driver: bridge
  backend:
    driver: bridge
    internal: true

volumes:
  pg-data:
    driver: local

secrets:
  db_password:
    file: ./secrets/db_password.txt
  api_key:
    file: ./secrets/api_key.txt
```

---

## 5. Hands-On Lab — Hardening Containers Step by Step

### Lab 5.1: Running as Non-Root

```bash
# 1. Build an image that runs as root (the default)
cat > /tmp/Dockerfile.root << 'EOF'
FROM alpine:3.19
RUN echo "I am running" > /tmp/status.txt
CMD ["sh", "-c", "while true; do sleep 1; done"]
EOF

docker build -t lab-root -f /tmp/Dockerfile.root /tmp

# 2. Run it and check the user
docker run --rm lab-root whoami
# Output: root

# 3. Build an image that runs as non-root
cat > /tmp/Dockerfile.nonroot << 'EOF'
FROM alpine:3.19
RUN adduser -D -u 1001 appuser
USER appuser
CMD ["sh", "-c", "while true; do sleep 1; done"]
EOF

docker build -t lab-nonroot -f /tmp/Dockerfile.nonroot /tmp

# 4. Run it and check the user
docker run --rm lab-nonroot whoami
# Output: appuser

# 5. Demonstrate the difference in capability
docker run --rm lab-root sh -c 'cat /proc/1/status | grep CapEff'
# Full capabilities

docker run --rm lab-nonroot sh -c 'cat /proc/1/status | grep CapEff'
# Reduced capabilities

# 6. Clean up
docker rmi lab-root lab-nonroot
rm /tmp/Dockerfile.root /tmp/Dockerfile.nonroot
```

### Lab 5.2: Read-Only Filesystem

```bash
# 1. Try to write to a read-only container
docker run --rm --read-only alpine sh -c 'echo test > /tmp/file.txt'
# Fails: /tmp is read-only

# 2. Add tmpfs for /tmp
docker run --rm --read-only --tmpfs /tmp:size=50M alpine sh -c \
  'echo test > /tmp/file.txt && cat /tmp/file.txt'
# Works: tmpfs provides writable /tmp

# 3. Application example with read-only + tmpfs
cat > /tmp/Dockerfile.readonly << 'EOF'
FROM python:3.12-alpine
RUN adduser -D -u 1001 appuser
WORKDIR /app
RUN echo 'import tempfile; f = tempfile.NamedTemporaryFile(); print(f.name)' > app.py
USER appuser
CMD ["python", "app.py"]
EOF

docker build -t lab-readonly -f /tmp/Dockerfile.readonly /tmp

# 4. Run with read-only filesystem
docker run --rm --read-only --tmpfs /tmp:size=50M lab-readonly
# Python can create temp files in /tmp

# 5. Clean up
docker rmi lab-readonly
rm /tmp/Dockerfile.readonly
```

### Lab 5.3: Dropping Capabilities

```bash
# 1. Show default capabilities
docker run --rm alpine sh -c \
  'apk add -q libcap && capsh --print | head -10'
# Shows many capabilities

# 2. Drop all capabilities
docker run --rm --cap-drop=ALL alpine sh -c \
  'apk add -q libcap && capsh --print | head -10'
# Shows no capabilities

# 3. Demonstrate: cannot change time without capability
docker run --rm --cap-drop=ALL --cap-add=SYS_TIME alpine sh -c \
  'date -s "2025-01-01" 2>&1 || echo "Cannot change time"'

# 4. Demonstrate: cannot ping without NET_RAW
docker run --rm --cap-drop=ALL alpine sh -c \
  'apk add -q iputils && ping -c 1 8.8.8.8 2>&1 || echo "Cannot ping"'

# 5. Add back NET_RAW to allow ping
docker run --rm --cap-drop=ALL --cap-add=NET_RAW alpine sh -c \
  'apk add -q iputils && ping -c 1 8.8.8.8 2>&1'
```

### Lab 5.4: Secrets Without Environment Variables

```bash
# 1. Create a secret file
mkdir -p /tmp/lab-secrets
echo "my-super-secret-password" > /tmp/lab-secrets/db_password

# 2. BAD: secret in ENV (visible in inspect)
docker run --rm -e DB_PASSWORD=my-super-secret-password alpine env | grep DB_PASSWORD
# Output: DB_PASSWORD=my-super-secret-password

# 3. GOOD: secret as file mount
docker run --rm \
  -v /tmp/lab-secrets/db_password:/run/secrets/db_password:ro \
  alpine sh -c 'cat /run/secrets/db_password'
# Output: my-super-secret-password

# 4. Verify secret is not in ENV or inspect
docker run --rm --name secret-test \
  -v /tmp/lab-secrets/db_password:/run/secrets/db_password:ro \
  -d alpine sh -c 'sleep 30'

docker inspect secret-test | grep -i password
# No output — password is not in the container metadata

docker exec secret-test env | grep -i password
# No output — password is not in environment variables

docker rm -f secret-test
rm -rf /tmp/lab-secrets
```

### Lab 5.5: Image Scanning with Trivy

```bash
# 1. Install Trivy (if not installed)
# curl -sfL https://raw.githubusercontent.com/aquasecurity/trivy/main/contrib/install.sh | sh -s -- -b /usr/local/bin

# 2. Scan a base image
trivy image --severity HIGH,CRITICAL alpine:3.19

# 3. Scan a larger image (more vulnerabilities expected)
trivy image --severity HIGH,CRITICAL node:18

# 4. Scan your own image
cat > /tmp/Dockerfile.scan << 'EOF'
FROM node:18
WORKDIR /app
COPY . .
RUN npm install
CMD ["node", "server.js"]
EOF

docker build -t lab-scan -f /tmp/Dockerfile.scan /tmp
trivy image --severity HIGH,CRITICAL lab-scan

# 5. Scan a slim variant (fewer packages = fewer vulnerabilities)
trivy image --severity HIGH,CRITICAL node:18-slim
# Fewer vulnerabilities than the full image

# 6. Clean up
docker rmi lab-scan
rm /tmp/Dockerfile.scan
```

### Lab 5.6: Complete Security Audit

```bash
# Run this against any container to check its security posture
cat > /tmp/audit-container.sh << 'SCRIPT'
#!/bin/bash
CONTAINER=$1

echo "=== Security Audit for: $CONTAINER ==="
echo ""

# 1. Check running user
USER=$(docker exec "$CONTAINER" whoami 2>/dev/null)
echo "[1] Running as: $USER"
if [ "$USER" = "root" ]; then
    echo "    WARNING: Container runs as root"
else
    echo "    OK: Running as non-root"
fi

# 2. Check capabilities
CAPS=$(docker exec "$CONTAINER" cat /proc/1/status 2>/dev/null | grep CapEff | awk '{print $2}')
echo "[2] Effective capabilities: $CAPS"
if [ "$CAPS" = "0000000000000000" ]; then
    echo "    OK: All capabilities dropped"
else
    echo "    WARNING: Capabilities are present"
fi

# 3. Check filesystem
echo "[3] Filesystem:"
docker exec "$CONTAINER" sh -c 'echo test > /tmp/test 2>&1' >/dev/null
if [ $? -eq 0 ]; then
    echo "    /tmp is writable"
    docker exec "$CONTAINER" rm /tmp/test 2>/dev/null
else
    echo "    /tmp is read-only"
fi

# 4. Check for secrets in ENV
ENV_SECRETS=$(docker inspect "$CONTAINER" --format '{{.Config.Env}}' | grep -iE 'password|secret|key|token' || true)
if [ -n "$ENV_SECRETS" ]; then
    echo "[4] WARNING: Potential secrets found in environment variables"
    echo "    $ENV_SECRETS"
else
    echo "[4] OK: No obvious secrets in environment variables"
fi

# 5. Check for Docker socket mount
SOCK=$(docker inspect "$CONTAINER" --format '{{range .Mounts}}{{.Source}} {{end}}' | grep docker.sock || true)
if [ -n "$SOCK" ]; then
    echo "[5] CRITICAL: Docker socket is mounted — full host access!"
else
    echo "[5] OK: Docker socket not mounted"
fi

# 6. Check resource limits
MEM=$(docker inspect "$CONTAINER" --format '{{.HostConfig.Memory}}')
echo "[6] Memory limit: $MEM"
if [ "$MEM" = "0" ]; then
    echo "    WARNING: No memory limit set"
else
    echo "    OK: Memory limit is $(( MEM / 1024 / 1024 )) MB"
fi

# 7. Check network mode
NET=$(docker inspect "$CONTAINER" --format '{{.HostConfig.NetworkMode}}')
echo "[7] Network mode: $NET"
if [ "$NET" = "host" ]; then
    echo "    CRITICAL: Host network mode — container shares host network stack"
else
    echo "    OK: Using isolated network"
fi

# 8. Check for privileged mode
PRIV=$(docker inspect "$CONTAINER" --format '{{.HostConfig.Privileged}}')
echo "[8] Privileged: $PRIV"
if [ "$PRIV" = "true" ]; then
    echo "    CRITICAL: Container is privileged — near-root host access"
else
    echo "    OK: Not privileged"
fi

echo ""
echo "=== Audit Complete ==="
SCRIPT
chmod +x /tmp/audit-container.sh

# Run the audit against any container
docker run --name test-container -d alpine sleep 60
/tmp/audit-container.sh test-container
docker rm -f test-container
rm /tmp/audit-container.sh
```

---

## 6. Limitation — What's the Next Problem?

Container security hardening addresses individual container safety, but it creates new challenges at scale:

**Configuration complexity.** Every security measure adds configuration. Seccomp profiles, AppArmor profiles, capability lists, secret mounts, network policies — maintaining these across dozens of services is error-prone. A misconfigured seccomp profile can break an application in subtle ways.

**No runtime threat detection.** All the measures above are preventive. They do not detect or respond to active attacks. A container that is running and behaving maliciously within its allowed capabilities will not be stopped.

**Container escape is still possible.** Kernel vulnerabilities (CVE-2022-0185, CVE-2022-0492, CVE-2024-0193) have demonstrated that container escape is achievable even without privileged mode. Shared kernels are an inherent risk.

**Secrets rotation is not solved.** Mounting secrets as files is better than ENV, but rotating secrets still requires restarting containers. There is no built-in mechanism for hot-reloading secrets.

**Compliance and auditing.** Security hardening does not automatically produce audit trails. You need separate tooling to prove that containers are running with the correct security posture at all times.

**The CIS Docker Benchmark** provides a comprehensive checklist for Docker security, but implementing all 6+ sections manually is impractical. Tools like Docker Bench for Security automate this:

```bash
# Run the CIS Docker Benchmark
docker run --rm --net host --pid host \
  --userns host --cap-add audit_control \
  -v /var/lib:/var/lib:ro \
  -v /var/run/docker.sock:/var/run/docker.sock:ro \
  docker/docker-bench-security
```

The next step is orchestration security: how to manage security policies across many containers, enforce compliance automatically, detect runtime threats, and respond to incidents. This leads into Kubernetes security, policy engines (OPA/Gatekeeper, Kyverno), runtime security (Falco, Sysdig), and zero-trust networking.

---

## 7. Security Checklist

Use this checklist for every container deployed to production:

```
Image Security
  [ ] Base image is from a trusted source
  [ ] Base image version is pinned (not :latest)
  [ ] Image has been scanned (Trivy/Snyk/Scout) — no HIGH/CRITICAL CVEs
  [ ] Image is signed (DCT or Cosign)
  [ ] Image is built from a minimal base (Alpine, distroless, scratch)
  [ ] No secrets baked into the image
  [ ] .dockerignore excludes sensitive files

Runtime Security
  [ ] Container runs as non-root user
  [ ] Filesystem is read-only (with explicit tmpfs/volumes for writable paths)
  [ ] All capabilities are dropped (cap_drop: ALL)
  [ ] Only required capabilities are added back
  [ ] no-new-privileges is set
  [ ] Seccomp profile is applied (custom or default)
  [ ] AppArmor profile is applied (if applicable)
  [ ] Memory and CPU limits are set
  [ ] Health checks are configured

Network Security
  [ ] Container is not using host network mode
  [ ] Internal services are on an internal network (no external access)
  [ ] Only required ports are exposed
  [ ] Network policies restrict inter-container communication

Secrets Management
  [ ] No secrets in environment variables
  [ ] No secrets in Dockerfile or image layers
  [ ] Secrets are mounted as files (Docker secrets or bind mount)
  [ ] Secret files are read-only
  [ ] Secrets are rotated on a schedule

Operational Security
  [ ] Logging is configured with size limits
  [ ] Container restart policy is set
  [ ] Resource limits prevent denial-of-service
  [ ] Docker socket is NOT mounted into containers
  [ ] Containers are not running in privileged mode
  [ ] Docker Bench for Security passes all checks
```

---

## 8. Next Topic

With containers secured and data persisted, the next challenge is managing multiple containers across multiple hosts: orchestration. Topics to explore include:

- Kubernetes security policies and Pod Security Standards
- Runtime threat detection with Falco
- Policy engines (OPA/Gatekeeper, Kyverno)
- Service mesh security (Istio, Linkerd)
- Zero-trust networking (mTLS, SPIFFE/SPIRE)
- Supply chain security (SLSA, SBOM)
