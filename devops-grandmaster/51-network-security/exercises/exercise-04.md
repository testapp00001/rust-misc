# Exercise 04: Audit and Harden a Misconfigured Network

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective
Given a working but insecure network configuration, identify every misconfiguration, explain the risk it poses, and provide the corrected configuration while maintaining all legitimate functionality. This exercise simulates a real-world security audit where you must balance security hardening with operational continuity.

## The Scenario

You have inherited the following network configuration for a production Linux server. The previous engineer left, and management wants a security audit. The server runs a web application, a database, and provides SSH access to the operations team.

### Current iptables Rules

```bash
# Current iptables configuration (iptables-save output)
*filter
:INPUT ACCEPT [0:0]
:FORWARD ACCEPT [0:0]
:OUTPUT ACCEPT [0:0]

# SSH from anywhere
-A INPUT -p tcp --dport 22 -j ACCEPT

# Web server
-A INPUT -p tcp --dport 80 -j ACCEPT
-A INPUT -p tcp --dport 443 -j ACCEPT

# Database - open to all
-A INPUT -p tcp --dport 5432 -j ACCEPT

# Allow all loopback
-A INPUT -i lo -j ACCEPT

# Allow ICMP
-A INPUT -p icmp -j ACCEPT

# Log dropped
-A INPUT -j LOG --log-prefix "IPTABLES-DROP: "

COMMIT
```

### Current SSH Configuration

```bash
# /etc/ssh/sshd_config
Port 22
PermitRootLogin yes
PasswordAuthentication yes
MaxAuthTries 6
X11Forwarding yes
AllowTcpForwarding yes
PermitEmptyPasswords no
# No AllowUsers or AllowGroups directive
```

### Current Application Configuration

```yaml
# docker-compose.yml (partial)
services:
  web:
    image: nginx:latest
    ports:
      - "0.0.0.0:80:80"
      - "0.0.0.0:443:443"
    networks:
      - app-network

  app:
    image: node:18
    ports:
      - "0.0.0.0:3000:3000"
    environment:
      - DB_HOST=10.0.0.30
      - DB_PASSWORD=supersecret123
    networks:
      - app-network

  postgres:
    image: postgres:15
    ports:
      - "0.0.0.0:5432:5432"
    environment:
      - POSTGRES_PASSWORD=admin123
    volumes:
      - ./data:/var/lib/postgresql/data
    networks:
      - app-network

networks:
  app-network:
    driver: bridge
```

### Current Network Interfaces

```bash
$ ip addr show
2: eth0: <BROADCAST,MULTICAST,UP> mtu 1500
    inet 203.0.113.50/24 scope global eth0
3: eth1: <BROADCAST,MULTICAST,UP> mtu 1500
    inet 10.0.0.50/24 scope global eth1
```

## Tasks

### Part A: Identify All Misconfigurations
Review all four configuration files and create a security findings report. For each finding, document:
- The misconfiguration
- The severity (Critical, High, Medium, Low)
- The risk it poses (what an attacker could do)
- The affected file/setting

<details>
<summary>Hint</summary>
Look for: open default policies, services bound to 0.0.0.0, plaintext credentials, overly permissive SSH, missing rate limiting, no connection state tracking, database exposed to the public internet, no fail2ban or brute-force protection.
</details>

### Part B: Fix the iptables Rules
Rewrite the iptables ruleset to fix all identified issues while maintaining:
- SSH access from the operations team subnet (`10.0.1.0/24`)
- HTTPS access from the internet
- PostgreSQL access from the app container only
- Outbound internet access for package updates

<details>
<summary>Hint</summary>
Set default INPUT policy to DROP. Add established/related rule. Restrict SSH to the ops subnet. Use the Docker bridge network CIDR for PostgreSQL. Block the app port (3000) from external access. Add rate limiting for SSH. Log dropped packets on a separate chain to avoid log flooding.
</details>

### Part C: Fix the SSH Configuration
Harden the SSH configuration file. Address:
- Root login
- Authentication method
- Connection limits
- Network restrictions
- Session timeouts

<details>
<summary>Hint</summary>
Disable root login, use key-only authentication, restrict to specific users/groups, reduce MaxAuthTries, add ClientAliveInterval/ClientAliveCountMax, disable X11 forwarding and TCP forwarding unless specifically needed. Consider moving to a non-standard port as defense-in-depth.
</details>

### Part D: Fix the Docker Compose Configuration
Secure the Docker Compose file by:
- Removing unnecessary port bindings
- Using Docker secrets or environment files for credentials
- Configuring proper network isolation
- Adding resource limits
- Setting read-only filesystem where possible

<details>
<summary>Hint</summary>
PostgreSQL should not bind to 0.0.0.0:5432 -- it only needs to be accessible from the app container on the Docker network. Use `env_file` instead of inline environment variables. Add `read_only: true` to services that do not write to their filesystem. Use named volumes instead of bind mounts for data.
</details>

### Part E: Create a Verification Script
Write a bash script that verifies all security fixes are in place. The script should:
1. Check iptables rules match the expected configuration
2. Verify SSH configuration parameters
3. Test that restricted ports are not accessible from unauthorized sources
4. Validate Docker container networking
5. Report pass/fail for each check with colored output

<details>
<summary>Hint</summary>
Use `iptables -L -n` and parse with grep/awk. Use `sshd -T` to check effective SSH config. Use `nc -z` or `nmap` to test port accessibility. Use `docker network inspect` to verify container networking. Use ANSI color codes for green (pass) and red (fail) output.
</details>

## Success Criteria
- [ ] You identified at least 8 distinct misconfigurations across all files
- [ ] Your iptables ruleset uses DROP default policy with explicit allows
- [ ] SSH is restricted to key-based auth with no root login
- [ ] Docker services do not expose internal ports to the host
- [ ] Credentials are not stored in plaintext in configuration files
- [ ] Verification script runs cleanly and reports all checks as passing

## What You Should Understanding After This Exercise
Security auditing requires reading configurations with an attacker's mindset. Every "ACCEPT" rule, every "0.0.0.0" binding, every "yes" in a config file is a potential attack surface. Hardening is not about breaking functionality -- it is about ensuring that only the intended traffic reaches the intended service. The verification script is critical: it turns your security posture from "I think it is secure" to "I can prove it is secure."
