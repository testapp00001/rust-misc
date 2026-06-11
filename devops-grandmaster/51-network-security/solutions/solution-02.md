# Solution 02: Build a Firewall Ruleset from Requirements

## Part A: Traffic Policy Matrix

The principle is deny-all by default, then explicitly allow only what is needed.

| # | Source | Dest Port | Protocol | Interface | Action | Justification |
|---|--------|-----------|----------|-----------|--------|---------------|
| 1 | 10.0.0.0/24 (admin) | 22 | TCP | eth1 | ALLOW | SSH for administration |
| 2 | 0.0.0.0/0 (public) | 80 | TCP | eth0 | ALLOW | HTTP (redirect to HTTPS) |
| 3 | 0.0.0.0/0 (public) | 443 | TCP | eth0 | ALLOW | HTTPS web traffic |
| 4 | 10.0.2.0/24 (app) | 5432 | TCP | eth1 | ALLOW | PostgreSQL for app servers |
| 5 | localhost | 3000 | TCP | lo | ALLOW | Node.js behind Nginx reverse proxy |
| 6 | 10.0.3.10 (monitoring) | 9090 | TCP | eth1 | ALLOW | Prometheus scraping |
| 7 | ALL | ALL | ESTABLISHED,RELATED | any | ALLOW | Return traffic for outbound connections |
| 8 | ALL | ALL | ALL | lo | ALLOW | Loopback (local inter-process) |
| 9 | ALL | ALL | ALL | any | DROP | Default deny (policy) |

**Default policies:**
- INPUT: DROP
- FORWARD: DROP
- OUTPUT: ACCEPT (server needs outbound for updates)

### Why This Works
Starting with DROP and adding specific ALLOW rules means any traffic not explicitly permitted is blocked. Each ALLOW rule has a specific source, preventing unauthorized access. The ESTABLISHED/RELATED rule allows return traffic for outbound connections (apt updates, API calls) without opening inbound ports.

### Common Mistakes
- Setting default policy to ACCEPT and trying to block specific ports (blacklist approach)
- Forgetting ESTABLISHED/RELATED (breaks outbound connections)
- Using `0.0.0.0/0` for SSH (should be admin subnet only)
- Not specifying the interface for rules (allows bypass via other interfaces)

## Part B: iptables Rules

```bash
#!/bin/bash
# firewall.sh - Production firewall rules for 10.0.1.50

# ============================================================
# Flush existing rules
# ============================================================
iptables -F
iptables -X
iptables -Z

# ============================================================
# Set default policies
# ============================================================
iptables -P INPUT DROP
iptables -P FORWARD DROP
iptables -P OUTPUT ACCEPT

# ============================================================
# Allow loopback traffic
# ============================================================
iptables -A INPUT -i lo -j ACCEPT
iptables -A OUTPUT -o lo -j ACCEPT

# ============================================================
# Allow established and related connections
# ============================================================
iptables -A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT

# ============================================================
# Allow ICMP (ping) for diagnostics
# ============================================================
iptables -A INPUT -p icmp --icmp-type echo-request -j ACCEPT

# ============================================================
# SSH from admin subnet only (with rate limiting)
# ============================================================
iptables -A INPUT -i eth1 -s 10.0.0.0/24 -p tcp --dport 22 \
  -m conntrack --ctstate NEW \
  -m recent --set --name SSH
iptables -A INPUT -i eth1 -s 10.0.0.0/24 -p tcp --dport 22 \
  -m conntrack --ctstate NEW \
  -m recent --update --seconds 60 --hitcount 4 --name SSH \
  -j DROP
iptables -A INPUT -i eth1 -s 10.0.0.0/24 -p tcp --dport 22 \
  -m conntrack --ctstate NEW -j ACCEPT

# ============================================================
# HTTP and HTTPS from public (eth0)
# ============================================================
iptables -A INPUT -i eth0 -p tcp --dport 80 \
  -m conntrack --ctstate NEW -j ACCEPT
iptables -A INPUT -i eth0 -p tcp --dport 443 \
  -m conntrack --ctstate NEW -j ACCEPT

# ============================================================
# PostgreSQL from app subnet only
# ============================================================
iptables -A INPUT -i eth1 -s 10.0.2.0/24 -p tcp --dport 5432 \
  -m conntrack --ctstate NEW -j ACCEPT

# ============================================================
# Node.js from localhost only (Nginx reverse proxy)
# ============================================================
iptables -A INPUT -i lo -p tcp --dport 3000 \
  -m conntrack --ctstate NEW -j ACCEPT

# ============================================================
# Prometheus from monitoring server only
# ============================================================
iptables -A INPUT -i eth1 -s 10.0.3.10 -p tcp --dport 9090 \
  -m conntrack --ctstate NEW -j ACCEPT

# ============================================================
# Logging chain for dropped packets
# ============================================================
iptables -N LOG_DROP
iptables -A LOG_DROP -m limit --limit 5/min --limit-burst 10 \
  -j LOG --log-prefix "IPTABLES-DROP: " --log-level 4
iptables -A LOG_DROP -j DROP

# Send remaining INPUT traffic to LOG_DROP
iptables -A INPUT -j LOG_DROP

echo "Firewall rules applied successfully."
```

### Why This Works
- **conntrack:** Stateful tracking means established connections are allowed without explicit per-port rules for return traffic.
- **Interface binding:** Rules specify `eth0` or `eth1` to prevent traffic from bypassing rules via unexpected interfaces.
- **Rate limiting on SSH:** The `recent` module limits SSH connection attempts to 3 per 60 seconds, mitigating brute force.
- **LOG_DROP chain:** Separating logging into its own chain with rate limiting prevents log flooding while still capturing dropped packets.
- **Rule order:** Specific allows come before the LOG_DROP catch-all.

### Common Mistakes
- Forgetting to flush rules before applying (appends to existing rules)
- Not using conntrack (breaks return traffic for outbound connections)
- Logging every dropped packet without rate limiting (fills disk)
- Not binding rules to specific interfaces (allows traffic bypass)
- Applying rules before setting default policy (locks you out if SSH rule has a bug)

## Part C: UFW Commands

```bash
#!/bin/bash
# ufw-firewall.sh - UFW equivalent of the iptables ruleset

# ============================================================
# Reset to clean state
# ============================================================
ufw --force reset

# ============================================================
# Set default policies
# ============================================================
ufw default deny incoming
ufw default allow outgoing

# ============================================================
# Allow SSH from admin subnet only
# ============================================================
ufw allow from 10.0.0.0/24 to any port 22 proto tcp comment "SSH from admin subnet"

# ============================================================
# Allow HTTP and HTTPS from anywhere
# ============================================================
ufw allow 80/tcp comment "HTTP"
ufw allow 443/tcp comment "HTTPS"

# ============================================================
# Allow PostgreSQL from app subnet only
# ============================================================
ufw allow from 10.0.2.0/24 to any port 5432 proto tcp comment "PostgreSQL from app tier"

# ============================================================
# Allow Node.js from localhost only
# ============================================================
ufw allow from 127.0.0.1 to any port 3000 proto tcp comment "Node.js localhost only"

# ============================================================
# Allow Prometheus from monitoring server only
# ============================================================
ufw allow from 10.0.3.10 to any port 9090 proto tcp comment "Prometheus from monitoring"

# ============================================================
# Enable logging (medium level)
# ============================================================
ufw logging medium

# ============================================================
# Enable UFW
# ============================================================
ufw --force enable

# ============================================================
# Show status
# ============================================================
ufw status verbose
```

**UFW Application Profile (optional, for cleaner management):**

```ini
# /etc/ufw/applications.d/custom-apps
[SSH-Restricted]
title=SSH (Restricted)
description=SSH access from admin subnet only
ports=22/tcp

[WebServer]
title=Web Server
description=Nginx HTTP/HTTPS
ports=80,443/tcp

[PostgreSQL-Restricted]
title=PostgreSQL (Restricted)
description=Database access from app tier only
ports=5432/tcp

[Prometheus]
title=Prometheus
description=Metrics scraping
ports=9090/tcp
```

```bash
# Using application profiles:
ufw allow from 10.0.0.0/24 to any app SSH-Restricted
ufw allow WebServer
ufw allow from 10.0.2.0/24 to any app PostgreSQL-Restricted
```

### Why This Works
UFW provides a human-readable interface to iptables. The `comment` option documents each rule's purpose. Application profiles group related ports and make rules easier to manage. UFW handles conntrack and established connections automatically.

### Common Mistakes
- Forgetting `ufw default deny incoming` (default is ALLOW on some systems)
- Not enabling UFW after adding rules (rules exist but are inactive)
- Using `ufw allow 22` without source restriction (opens SSH to the world)

## Part D: Persistent Firewall Script

```bash
#!/bin/bash
# /usr/local/bin/firewall.sh
# Production firewall script with persistence, logging, and dry-run support

set -euo pipefail

DRY_RUN=false
LOG_FILE="/var/log/firewall-dropped.log"
RULES_FILE="/etc/iptables/rules.v4"

# ============================================================
# Parse arguments
# ============================================================
while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --show)
            echo "=== Current iptables rules ==="
            iptables -L -n -v --line-numbers
            echo ""
            echo "=== NAT table ==="
            iptables -t nat -L -n -v
            exit 0
            ;;
        --help)
            echo "Usage: $0 [--dry-run] [--show] [--help]"
            echo "  --dry-run   Show rules without applying"
            echo "  --show      Display current rules"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# ============================================================
# Function to run or print a command
# ============================================================
run() {
    if $DRY_RUN; then
        echo "[DRY-RUN] $*"
    else
        "$@"
    fi
}

# ============================================================
# Ensure log directory and file exist
# ============================================================
if ! $DRY_RUN; then
    mkdir -p /var/log
    touch "$LOG_FILE"
fi

# ============================================================
# Flush existing rules
# ============================================================
echo "Flushing existing rules..."
run iptables -F
run iptables -X
run iptables -Z
run iptables -t nat -F
run iptables -t nat -X

# ============================================================
# Set default policies
# ============================================================
echo "Setting default policies..."
run iptables -P INPUT DROP
run iptables -P FORWARD DROP
run iptables -P OUTPUT ACCEPT

# ============================================================
# Custom chains
# ============================================================
echo "Creating custom chains..."
run iptables -N LOG_DROP 2>/dev/null || run iptables -F LOG_DROP

# ============================================================
# Loopback
# ============================================================
echo "Allowing loopback..."
run iptables -A INPUT -i lo -j ACCEPT
run iptables -A OUTPUT -o lo -j ACCEPT

# ============================================================
# Established connections
# ============================================================
echo "Allowing established connections..."
run iptables -A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT

# ============================================================
# ICMP
# ============================================================
echo "Allowing ICMP..."
run iptables -A INPUT -p icmp --icmp-type echo-request -j ACCEPT

# ============================================================
# SSH with rate limiting
# ============================================================
echo "Allowing SSH from admin subnet (rate-limited)..."
run iptables -A INPUT -i eth1 -s 10.0.0.0/24 -p tcp --dport 22 \
  -m conntrack --ctstate NEW \
  -m recent --set --name SSH
run iptables -A INPUT -i eth1 -s 10.0.0.0/24 -p tcp --dport 22 \
  -m conntrack --ctstate NEW \
  -m recent --update --seconds 60 --hitcount 4 --name SSH \
  -j DROP
run iptables -A INPUT -i eth1 -s 10.0.0.0/24 -p tcp --dport 22 \
  -m conntrack --ctstate NEW -j ACCEPT

# ============================================================
# HTTP/HTTPS
# ============================================================
echo "Allowing HTTP/HTTPS..."
run iptables -A INPUT -i eth0 -p tcp -m multiport --dports 80,443 \
  -m conntrack --ctstate NEW -j ACCEPT

# ============================================================
# PostgreSQL
# ============================================================
echo "Allowing PostgreSQL from app subnet..."
run iptables -A INPUT -i eth1 -s 10.0.2.0/24 -p tcp --dport 5432 \
  -m conntrack --ctstate NEW -j ACCEPT

# ============================================================
# Node.js (localhost only)
# ============================================================
echo "Allowing Node.js on localhost..."
run iptables -A INPUT -i lo -p tcp --dport 3000 \
  -m conntrack --ctstate NEW -j ACCEPT

# ============================================================
# Prometheus
# ============================================================
echo "Allowing Prometheus from monitoring server..."
run iptables -A INPUT -i eth1 -s 10.0.3.10 -p tcp --dport 9090 \
  -m conntrack --ctstate NEW -j ACCEPT

# ============================================================
# Logging and final drop
# ============================================================
echo "Setting up logging..."
run iptables -A LOG_DROP -m limit --limit 5/min --limit-burst 10 \
  -j LOG --log-prefix "IPTABLES-DROP: " --log-level 4
run iptables -A LOG_DROP -j DROP
run iptables -A INPUT -j LOG_DROP

# ============================================================
# Persist rules
# ============================================================
if ! $DRY_RUN; then
    echo "Saving rules to $RULES_FILE..."
    mkdir -p "$(dirname "$RULES_FILE")"
    iptables-save > "$RULES_FILE"
    echo "Rules saved."
fi

echo "Firewall configuration complete."
if $DRY_RUN; then
    echo "(Dry run -- no changes were applied)"
fi
```

**Installation:**

```bash
# Make executable
chmod +x /usr/local/bin/firewall.sh

# Run normally
sudo /usr/local/bin/firewall.sh

# Dry run (show what would happen)
sudo /usr/local/bin/firewall.sh --dry-run

# Install persistence package (Debian/Ubuntu)
sudo apt install iptables-persistent
sudo netfilter-persistent save
```

### Why This Works
- **set -euo pipefail:** Script fails fast on any error, preventing partial rule application.
- **Custom LOG_DROP chain:** Isolates logging logic, applies rate limiting (5 per minute), then drops.
- **DRY_RUN flag:** Allows testing the script without affecting the live firewall.
- **iptables-save:** Persists rules in the standard format that survives reboots.
- **2>/dev/null on chain creation:** Prevents error if chain already exists on re-run.

### Common Mistakes
- Not making the script executable (`chmod +x`)
- Forgetting to install `iptables-persistent` (rules lost on reboot)
- Not using `set -e` (script continues after failures, leaving partial rules)
- Saving rules before verifying they work (locks in bad rules)

## Part E: Test Commands

```bash
# ============================================================
# Test matrix: run from each source location
# ============================================================

# --- From admin subnet (10.0.0.x) ---

# SSH (should SUCCEED)
ssh deploy@10.0.1.50 -o ConnectTimeout=5 "echo SSH works"
# Expected: "SSH works"

# SSH brute force (should be RATE LIMITED after 4 attempts)
for i in $(seq 1 6); do
    ssh deploy@10.0.1.50 -o ConnectTimeout=2 -o BatchMode=yes 2>&1 | head -1
done
# Expected: First 3-4 attempts show "Permission denied", then "Connection refused" or timeout

# PostgreSQL (should FAIL)
nc -zv 10.0.1.50 5432 -w 3
# Expected: Connection timed out

# Prometheus (should FAIL)
curl -s --max-time 3 http://10.0.1.50:9090/metrics
# Expected: Connection timed out

# --- From app subnet (10.0.2.x) ---

# PostgreSQL (should SUCCEED)
nc -zv 10.0.1.50 5432 -w 3
# Expected: Connection to 10.0.1.50 5432 port [tcp/postgresql] succeeded!

# SSH (should FAIL)
ssh deploy@10.0.1.50 -o ConnectTimeout=3
# Expected: Connection timed out

# --- From monitoring server (10.0.3.10) ---

# Prometheus (should SUCCEED)
curl -s --max-time 3 http://10.0.1.50:9090/metrics | head -5
# Expected: Prometheus metrics output

# PostgreSQL (should FAIL)
nc -zv 10.0.1.50 5432 -w 3
# Expected: Connection timed out

# --- From public internet ---

# HTTPS (should SUCCEED)
curl -s --max-time 5 https://10.0.1.50/ | head -5
# Expected: Nginx response

# HTTP (should SUCCEED)
curl -s --max-time 5 http://10.0.1.50/ | head -5
# Expected: Nginx response (or redirect to HTTPS)

# SSH (should FAIL)
ssh deploy@10.0.1.50 -o ConnectTimeout=3
# Expected: Connection timed out

# PostgreSQL (should FAIL)
nc -zv 10.0.1.50 5432 -w 3
# Expected: Connection timed out

# Node.js direct (should FAIL)
curl -s --max-time 3 http://10.0.1.50:3000/
# Expected: Connection timed out

# --- Check dropped packet logs ---
sudo tail -20 /var/log/firewall-dropped.log
# Expected: Log entries for all failed connection attempts above
```

### Why This Works
Testing from each source location verifies that the firewall rules match the policy matrix. Each test has a clear expected outcome (success or specific failure mode). The log check confirms that dropped packets are being recorded. Running these tests after every firewall change is essential to catch regressions.

### Common Mistakes
- Only testing that allowed traffic works (must also test that blocked traffic is actually blocked)
- Not testing from the correct source IP/interface (tests from localhost bypass INPUT rules)
- Forgetting to check the logs (confirms drops are working and being recorded)

## Key Takeaway
A firewall is only as good as the policy it enforces, and a policy is only as good as the tests that verify it. Start with deny-all, add specific allows with explicit sources, test every rule from both allowed and denied perspectives, persist the rules, and log the drops. The iptables-to-UFW translation shows that the underlying security model is the same -- only the interface changes.
