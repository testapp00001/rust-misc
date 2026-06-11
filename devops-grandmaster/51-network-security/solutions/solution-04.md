# Solution 04: Audit and Harden a Misconfigured Network

## Part A: Security Findings Report

| # | Finding | Severity | Risk | File/Setting |
|---|---------|----------|------|--------------|
| 1 | INPUT default policy is ACCEPT | Critical | All traffic is allowed by default; firewall rules are meaningless because unmatched traffic is accepted | iptables `:INPUT ACCEPT` |
| 2 | FORWARD default policy is ACCEPT | High | The server can be used as a router to forward traffic between networks, enabling pivoting attacks | iptables `:FORWARD ACCEPT` |
| 3 | PostgreSQL (5432) open to all | Critical | Any host on the internet can connect to the database. Brute force or default credentials give full data access | iptables rule for 5432 |
| 4 | SSH allows root login | High | Attackers can brute-force the root account. Root login means no audit trail of which admin performed actions | sshd_config `PermitRootLogin yes` |
| 5 | SSH uses password authentication | High | Passwords can be brute-forced, phished, or stolen from credential dumps. Key-based auth is significantly stronger | sshd_config `PasswordAuthentication yes` |
| 6 | SSH allows 6 auth attempts | Medium | Gives attackers more chances per connection to guess credentials before disconnect | sshd_config `MaxAuthTries 6` |
| 7 | SSH X11 forwarding enabled | Medium | X11 is a known attack vector for keylogging and screen capture. Containers and servers do not need X11 | sshd_config `X11Forwarding yes` |
| 8 | SSH TCP forwarding enabled | Medium | Can be used to create tunnels that bypass firewall rules, effectively punching holes through your security | sshd_config `AllowTcpForwarding yes` |
| 9 | No AllowUsers/AllowGroups directive | Medium | Any valid system user can SSH in. Service accounts, nologin users, and test accounts all have SSH access | sshd_config missing directive |
| 10 | PostgreSQL bound to 0.0.0.0:5432 | Critical | The database is accessible from outside the Docker network. Combined with iptables ACCEPT, it is internet-exposed | docker-compose.yml |
| 11 | App port 3000 bound to 0.0.0.0:3000 | High | The Node.js backend API is directly accessible, bypassing Nginx reverse proxy and any WAF in front of it | docker-compose.yml |
| 12 | DB_PASSWORD in plaintext env var | Critical | Password visible in docker inspect, /proc, CI/CD logs, and shell history | docker-compose.yml `DB_PASSWORD=supersecret123` |
| 13 | POSTGRES_PASSWORD in plaintext env var | Critical | Database root password in plaintext in configuration file | docker-compose.yml `POSTGRES_PASSWORD=admin123` |
| 14 | Weak passwords | Critical | `supersecret123` and `admin123` are trivially guessable | docker-compose.yml |
| 15 | No established/related connection tracking | Medium | Every return packet for outbound connections must match an explicit allow rule or be dropped, breaking outbound connectivity | iptables missing conntrack |
| 16 | No fail2ban or brute-force protection | Medium | SSH and web endpoints have no rate limiting, enabling unlimited brute-force attempts | System-wide |
| 17 | SSH open from any source | High | No restriction on source IP for SSH; the entire internet can attempt to connect | iptables rule for port 22 |
| 18 | PostgreSQL data on bind mount | Medium | `./data:/var/lib/postgresql/data` exposes the database files to the host filesystem | docker-compose.yml |

### Why This Works

A systematic audit examines every configuration file with an attacker's mindset. The key insight is that misconfigurations compound: the ACCEPT default policy means the open database rule is irrelevant -- everything is already allowed. The plaintext credentials combined with the exposed database means any network scan discovers the database and the password is right there in the config file.

### Common Mistakes

- **Only auditing the firewall.** SSH config, application config, and Docker config are all part of the attack surface.
- **Rating severity based on CVSS alone.** A "Medium" severity finding (like TCP forwarding) becomes critical when combined with other findings.
- **Not checking Docker port bindings.** `0.0.0.0:5432:5432` exposes the container port to all network interfaces.

---

## Part B: Hardened iptables Rules

```bash
#!/bin/bash
# hardened-firewall.sh

# Flush existing rules
iptables -F
iptables -X
iptables -t nat -F
iptables -t nat -X

# ============================================================
# Default policies: DROP everything
# ============================================================
iptables -P INPUT DROP
iptables -P FORWARD DROP
iptables -P OUTPUT ACCEPT

# ============================================================
# Loopback: allow all local inter-process traffic
# ============================================================
iptables -A INPUT -i lo -j ACCEPT
iptables -A OUTPUT -o lo -j ACCEPT

# ============================================================
# Established/Related: allow return traffic
# ============================================================
iptables -A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT

# ============================================================
# ICMP: allow ping for diagnostics
# ============================================================
iptables -A INPUT -p icmp --icmp-type echo-request -j ACCEPT

# ============================================================
# SSH: ops subnet only, with rate limiting
# ============================================================
iptables -A INPUT -i eth1 -s 10.0.1.0/24 -p tcp --dport 22 \
  -m conntrack --ctstate NEW \
  -m recent --set --name SSH
iptables -A INPUT -i eth1 -s 10.0.1.0/24 -p tcp --dport 22 \
  -m conntrack --ctstate NEW \
  -m recent --update --seconds 60 --hitcount 4 --name SSH \
  -j DROP
iptables -A INPUT -i eth1 -s 10.0.1.0/24 -p tcp --dport 22 \
  -m conntrack --ctstate NEW -j ACCEPT

# ============================================================
# HTTPS: public (eth0 only)
# ============================================================
iptables -A INPUT -i eth0 -p tcp --dport 443 \
  -m conntrack --ctstate NEW -j ACCEPT

# ============================================================
# HTTP: public (redirect to HTTPS)
# ============================================================
iptables -A INPUT -i eth0 -p tcp --dport 80 \
  -m conntrack --ctstate NEW -j ACCEPT

# ============================================================
# PostgreSQL: Docker bridge network only
# ============================================================
# Docker bridge is typically 172.17.0.0/16 or a custom range
iptables -A INPUT -s 172.17.0.0/16 -p tcp --dport 5432 \
  -m conntrack --ctstate NEW -j ACCEPT

# ============================================================
# Node.js (port 3000): localhost only (Nginx reverse proxy)
# ============================================================
iptables -A INPUT -i lo -p tcp --dport 3000 \
  -m conntrack --ctstate NEW -j ACCEPT

# ============================================================
# Block external access to app port
# ============================================================
iptables -A INPUT -i eth0 -p tcp --dport 3000 -j DROP

# ============================================================
# Logging chain for dropped packets
# ============================================================
iptables -N LOG_DROP 2>/dev/null || iptables -F LOG_DROP
iptables -A LOG_DROP -m limit --limit 5/min --limit-burst 10 \
  -j LOG --log-prefix "IPTABLES-DROP: " --log-level 4
iptables -A LOG_DROP -j DROP

iptables -A INPUT -j LOG_DROP

echo "Hardened firewall rules applied."
```

### Why This Works

- **Default DROP:** Any traffic not explicitly allowed is blocked. This is the single most important change.
- **conntrack:** Established connections are tracked, so return traffic for outbound connections (apt updates, API calls) is automatically allowed.
- **SSH rate limiting:** The `recent` module allows only 3 new SSH connections per 60 seconds per source, mitigating brute force.
- **Interface binding:** SSH and HTTPS rules specify `eth1` (internal) and `eth0` (public) to prevent bypass.
- **Docker bridge restriction:** PostgreSQL only accepts connections from the Docker bridge network (172.17.0.0/16), not from external interfaces.
- **App port blocked externally:** Port 3000 is explicitly blocked on eth0, forcing traffic through Nginx on port 443.

### Common Mistakes

- **Not setting OUTPUT policy.** We keep OUTPUT ACCEPT because the server needs outbound access for updates and API calls.
- **Forgetting Docker's iptables manipulation.** Docker adds its own iptables rules. Use `DOCKER-USER` chain for rules that should apply to container traffic.
- **Not testing after applying.** Always test SSH connectivity before closing your current session.

---

## Part C: Hardened SSH Configuration

```bash
# /etc/ssh/sshd_config -- Hardened

# ============================================================
# Network
# ============================================================
Port 22
AddressFamily inet
ListenAddress 10.0.0.50

# ============================================================
# Authentication
# ============================================================
PermitRootLogin no
PasswordAuthentication no
PubkeyAuthentication yes
AuthenticationMethods publickey
MaxAuthTries 3
MaxSessions 3
LoginGraceTime 30

# ============================================================
# Access Control
# ============================================================
AllowGroups ops-team
# Alternatively: AllowUsers deploy admin1 admin2

# ============================================================
# Session
# ============================================================
ClientAliveInterval 300
ClientAliveCountMax 2

# ============================================================
# Forwarding and Tunneling
# ============================================================
X11Forwarding no
AllowTcpForwarding no
AllowAgentForwarding no
PermitTunnel no

# ============================================================
# Logging
# ============================================================
LogLevel VERBOSE
SyslogFacility AUTH

# ============================================================
# Security
# ============================================================
PermitEmptyPasswords no
StrictModes yes
MaxStartups 3:50:10
# 3 concurrent unauthenticated connections, 
# refuse at 50, drop at 10

Banner /etc/ssh/banner.txt
```

### Why This Works

- **PermitRootLogin no:** Forces admins to log in as regular users and use `sudo`, creating an audit trail.
- **PasswordAuthentication no:** Eliminates brute-force attacks against passwords. Only key-based auth is allowed.
- **MaxAuthTries 3:** Limits per-connection auth attempts from 6 to 3.
- **AllowGroups ops-team:** Only members of the `ops-team` group can SSH in. Service accounts and test users are excluded.
- **ClientAliveInterval 300:** Server sends a keepalive every 5 minutes. If the client does not respond within `ClientAliveCountMax * ClientAliveInterval` (10 minutes), the session is terminated. This prevents abandoned sessions.
- **X11Forwarding no, AllowTcpForwarding no:** Prevents tunneling attacks where an attacker uses SSH to bypass firewall rules.
- **MaxStartups 3:50:10:** Limits concurrent unauthenticated connections, mitigating connection-flooding attacks.
- **LogLevel VERBOSE:** Logs the public key fingerprint used for authentication, aiding forensic analysis.

### Common Mistakes

- **Locking yourself out.** Always test key-based auth before disabling password auth. Keep a backup session open.
- **Setting ClientAliveInterval too low.** Aggressive timeouts disconnect users on slow networks.
- **Not creating the ops-team group.** If AllowGroups references a nonexistent group, nobody can log in.

---

## Part D: Hardened Docker Compose Configuration

```yaml
# docker-compose.yml -- Hardened
services:
  web:
    image: nginx:1.25-alpine
    read_only: true
    tmpfs:
      - /var/cache/nginx
      - /var/run
      - /tmp
    ports:
      - "127.0.0.1:80:80"
      - "0.0.0.0:443:443"
    networks:
      - frontend
      - app-network
    depends_on:
      app:
        condition: service_healthy
    security_opt:
      - no-new-privileges:true
    deploy:
      resources:
        limits:
          cpus: "0.50"
          memory: 256M
        reservations:
          cpus: "0.25"
          memory: 128M

  app:
    image: node:18-alpine
    read_only: true
    tmpfs:
      - /tmp
    ports:
      - "127.0.0.1:3000:3000"
    env_file:
      - .env.app
    networks:
      - app-network
      - backend
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost:3000/health"]
      interval: 10s
      timeout: 5s
      retries: 3
    security_opt:
      - no-new-privileges:true
    deploy:
      resources:
        limits:
          cpus: "1.00"
          memory: 512M
        reservations:
          cpus: "0.50"
          memory: 256M

  postgres:
    image: postgres:15-alpine
    # No ports exposed to host -- only accessible from Docker network
    env_file:
      - .env.db
    volumes:
      - postgres-data:/var/lib/postgresql/data
    networks:
      - backend
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U app"]
      interval: 10s
      timeout: 5s
      retries: 5
    security_opt:
      - no-new-privileges:true
    deploy:
      resources:
        limits:
          cpus: "1.00"
          memory: 1G
        reservations:
          cpus: "0.50"
          memory: 512M

networks:
  frontend:
    driver: bridge
  app-network:
    driver: bridge
    internal: true  # No external access
  backend:
    driver: bridge
    internal: true  # No external access

volumes:
  postgres-data:
    driver: local
```

### .env.app file

```bash
# .env.app (not checked into version control)
DB_HOST=postgres
DB_PORT=5432
DB_NAME=myapp
DB_USER=app
DB_PASSWORD=<generated-strong-password>
```

### .env.db file

```bash
# .env.db (not checked into version control)
POSTGRES_DB=myapp
POSTGRES_USER=app
POSTGRES_PASSWORD=<generated-strong-password>
```

### .dockerignore file

```
.env
.env.*
.git
.gitignore
*.md
tests/
node_modules/
__pycache__
*.pyc
.DS_Store
docker-compose.yml
```

### Why This Works

- **No port bindings to 0.0.0.0 for internal services.** PostgreSQL has no `ports:` directive at all -- it is only accessible from the Docker network. Node.js binds to 127.0.0.1, so it is only accessible from localhost (Nginx).
- **env_file instead of inline environment.** Credentials are in separate `.env` files that are excluded from version control via `.dockerignore`.
- **Named volumes instead of bind mounts.** `postgres-data` is a Docker-managed volume, not a bind mount to `./data`. This prevents accidental exposure of database files on the host.
- **read_only: true.** Containers cannot write to their filesystem (except tmpfs mounts). An attacker who gains shell access cannot modify binaries or install tools.
- **no-new-privileges.** Prevents privilege escalation inside containers via setuid binaries.
- **Resource limits.** Limits CPU and memory to prevent a compromised container from consuming all host resources (cryptomining, fork bombs).
- **internal: true on backend networks.** Containers on internal networks cannot reach the internet. Only the `frontend` network (connected to Nginx) has external access.
- **Health checks.** Services report their health status, enabling orchestration tools to restart unhealthy containers.

### Common Mistakes

- **Exposing all ports to 0.0.0.0.** Internal services should bind to 127.0.0.1 or not expose ports at all.
- **Using bind mounts for data.** Bind mounts expose host filesystem paths and can lead to data loss if the host directory is deleted.
- **Not using env_file.** Inline environment variables are visible in `docker inspect` output.
- **Forgetting .dockerignore.** Without it, `.env` files, `.git` directories, and test files are copied into the image.

---

## Part E: Verification Script

```bash
#!/bin/bash
# verify-security.sh -- Verify all security hardening measures
set -euo pipefail

PASS=0
FAIL=0
WARN=0

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

pass() { echo -e "${GREEN}[PASS]${NC} $1"; ((PASS++)); }
fail() { echo -e "${RED}[FAIL]${NC} $1"; ((FAIL++)); }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; ((WARN++)); }

echo "============================================"
echo " Security Hardening Verification"
echo "============================================"
echo ""

# ============================================================
# 1. iptables checks
# ============================================================
echo "--- iptables Rules ---"

DEFAULT_INPUT=$(iptables -L INPUT -n | head -1)
if echo "$DEFAULT_INPUT" | grep -q "DROP"; then
    pass "Default INPUT policy is DROP"
else
    fail "Default INPUT policy is not DROP (found: $DEFAULT_INPUT)"
fi

DEFAULT_FORWARD=$(iptables -L FORWARD -n | head -1)
if echo "$DEFAULT_FORWARD" | grep -q "DROP"; then
    pass "Default FORWARD policy is DROP"
else
    fail "Default FORWARD policy is not DROP"
fi

if iptables -L INPUT -n | grep -q "ESTABLISHED\|RELATED"; then
    pass "Established/related connection tracking is configured"
else
    fail "No established/related connection tracking rule found"
fi

if iptables -L INPUT -n | grep -q "5432"; then
    DB_SOURCE=$(iptables -L INPUT -n | grep 5432 | awk '{print $4}')
    if [ "$DB_SOURCE" = "172.17.0.0/16" ] || [ "$DB_SOURCE" = "172.18.0.0/16" ]; then
        pass "PostgreSQL restricted to Docker bridge network ($DB_SOURCE)"
    else
        fail "PostgreSQL source is $DB_SOURCE (should be Docker bridge)"
    fi
else
    warn "No explicit PostgreSQL rule found (may be blocked by default DROP)"
fi

if iptables -L INPUT -n | grep -q "LOG_DROP"; then
    pass "LOG_DROP chain is configured for dropped packet logging"
else
    warn "No LOG_DROP chain found"
fi

echo ""

# ============================================================
# 2. SSH configuration checks
# ============================================================
echo "--- SSH Configuration ---"

SSH_CONFIG=$(sshd -T 2>/dev/null || cat /etc/ssh/sshd_config)

check_ssh() {
    local param="$1"
    local expected="$2"
    local actual=$(echo "$SSH_CONFIG" | grep -i "^${param}" | awk '{print tolower($2)}')
    if [ "$actual" = "$expected" ]; then
        pass "SSH: $param = $expected"
    else
        fail "SSH: $param = $actual (expected: $expected)"
    fi
}

check_ssh "permitrootlogin" "no"
check_ssh "passwordauthentication" "no"
check_ssh "x11forwarding" "no"
check_ssh "allowtcpforwarding" "no"
check_ssh "maxauthtries" "3"

echo ""

# ============================================================
# 3. Port accessibility checks
# ============================================================
echo "--- Port Accessibility ---"

# Test SSH from localhost (should work)
if timeout 3 bash -c "echo > /dev/tcp/127.0.0.1/22" 2>/dev/null; then
    pass "SSH accessible from localhost"
else
    warn "SSH not accessible from localhost (may be using non-standard port)"
fi

# Test PostgreSQL from localhost (should work via Docker network)
if timeout 3 bash -c "echo > /dev/tcp/127.0.0.1/5432" 2>/dev/null; then
    warn "PostgreSQL accessible from localhost (expected: Docker network only)"
else
    pass "PostgreSQL not directly accessible from localhost"
fi

# Test Node.js from external interface
EXTERNAL_IP=$(hostname -I | awk '{print $1}')
if timeout 3 bash -c "echo > /dev/tcp/$EXTERNAL_IP/3000" 2>/dev/null; then
    fail "Node.js (3000) accessible from external IP ($EXTERNAL_IP)"
else
    pass "Node.js (3000) not accessible from external IP"
fi

echo ""

# ============================================================
# 4. Docker configuration checks
# ============================================================
echo "--- Docker Configuration ---"

# Check if PostgreSQL port is bound to 0.0.0.0
if docker port postgres 5432 2>/dev/null | grep -q "0.0.0.0"; then
    fail "PostgreSQL port bound to 0.0.0.0 (should not be exposed)"
else
    pass "PostgreSQL port not bound to 0.0.0.0"
fi

# Check if Node.js port is bound to 127.0.0.1
if docker port app 3000 2>/dev/null | grep -q "127.0.0.1"; then
    pass "Node.js port bound to 127.0.0.1 (localhost only)"
else
    warn "Node.js port binding could not be verified"
fi

# Check for plaintext passwords in docker-compose
if grep -q "DB_PASSWORD=\|POSTGRES_PASSWORD=" docker-compose.yml 2>/dev/null; then
    fail "Plaintext passwords found in docker-compose.yml"
else
    pass "No plaintext passwords in docker-compose.yml"
fi

# Check for env_file usage
if docker inspect app --format '{{range .Config.Env}}{{println .}}{{end}}' 2>/dev/null | grep -q "DB_PASSWORD"; then
    warn "DB_PASSWORD visible in container environment (check if via env_file)"
else
    pass "Credentials not directly visible in container environment"
fi

echo ""

# ============================================================
# Summary
# ============================================================
echo "============================================"
echo -e " Results: ${GREEN}$PASS passed${NC}, ${RED}$FAIL failed${NC}, ${YELLOW}$WARN warnings${NC}"
echo "============================================"

if [ $FAIL -gt 0 ]; then
    echo -e "${RED}Security verification FAILED. Review the failed checks above.${NC}"
    exit 1
else
    echo -e "${GREEN}All security checks passed.${NC}"
    exit 0
fi
```

### Why This Works

The verification script tests each hardening measure independently. It checks the iptables rules, SSH configuration, port accessibility, and Docker configuration. The colored output makes it easy to see what passed and what failed at a glance. The exit code allows integration into CI/CD pipelines -- a failed check blocks deployment.

### Common Mistakes

- **Only checking one layer.** The script checks iptables, SSH, ports, and Docker because security is defense in depth.
- **Not testing negative cases.** Verifying that port 3000 is NOT accessible from the external IP is as important as verifying that port 443 IS accessible.
- **Hardcoding expected values.** The script reads actual values from the system rather than assuming.
- **Not making the script executable.** `chmod +x verify-security.sh` before running.

---

## Common Mistakes to Avoid

- **Applying iptables rules without testing SSH first.** Always keep a backup session open when modifying firewall rules.
- **Disabling password auth before verifying key auth works.** Test with `ssh -i key.pem user@host` before setting `PasswordAuthentication no`.
- **Not persisting iptables rules.** Rules are lost on reboot without `iptables-persistent` or `netfilter-persistent`.
- **Removing the LOG_DROP chain to "reduce noise."** Dropped packet logs are essential for debugging connectivity issues and detecting attacks.

## Key Takeaway

Security auditing requires reading configurations with an attacker's mindset. Every "ACCEPT" rule, every "0.0.0.0" binding, every "yes" in a config file is a potential attack surface. Hardening is not about breaking functionality -- it is about ensuring that only the intended traffic reaches the intended service. The verification script turns your security posture from "I think it is secure" to "I can prove it is secure."
