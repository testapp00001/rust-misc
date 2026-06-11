# Solution 01: Anatomy of a Flat Network Breach

## Part A: Lateral Movement Paths

After compromising Web-01 (10.0.0.10), the attacker can reach every host on the 10.0.0.0/24 subnet with no filtering. Here are all identified paths:

| Target Host | Target Service/Port | Attack Technique | Impact if Exploited |
|-------------|-------------------|------------------|---------------------|
| Web-02 (.11) | SSH (22) | Brute force or stolen SSH keys from Web-01 | Second web server compromised, doubles attack surface |
| Web-02 (.11) | Nginx (80/443) | Web application vulnerabilities, same codebase | Lateral web compromise |
| App-01 (.20) | Node.js (3000) | API exploitation, SSRF, or direct service access | Business logic compromise, data manipulation |
| App-01 (.20) | SSH (22) | Credential reuse or key theft | Full application server control |
| DB-01 (.30) | PostgreSQL (5432) | Default credentials, weak auth, credential theft from App-01 | Full database access, all financial data exposed |
| DB-01 (.30) | SSH (22) | Key theft from App-01 or CI/CD | OS-level database control |
| CI/CD (.40) | Jenkins (8080) | Jenkins default creds, unauthenticated API, script console | Pipeline poisoning, backdoor in deployments |
| CI/CD (.40) | Docker API (2375) | Unauthenticated Docker API -- run arbitrary containers | Full infrastructure control, cryptomining, data exfil |
| CI/CD (.40) | SSH (22) | Credential reuse | Server compromise |
| JumpBox (.50) | SSH (22) | Key or credential theft | Access to all systems reachable from jump box |

The Docker API on CI/CD (.40) is the single most dangerous exposure -- an unauthenticated Docker API on port 2375 allows an attacker to run any container with root privileges, effectively giving them full control of the host.

### Why This Works
A flat /24 subnet means every host is one hop away at Layer 3. There is no router ACL, no firewall rule, no VLAN boundary -- just a shared broadcast domain. ARP, broadcast, and all TCP/UDP traffic flows freely between any two hosts. The attacker on Web-01 can port-scan the entire /24 in seconds and connect to any open port.

### Common Mistakes
- Assuming "internal" services are safe because they are not internet-facing
- Forgetting that Docker API on port 2375 is effectively root access
- Underestimating how quickly an attacker can enumerate all hosts on a /24

## Part B: Attack Chain

```
Attacker                    Web-01                    App-01                    DB-01
   |                          |                         |                        |
   |--- Nginx vuln --------->|                         |                        |
   |                          |                         |                        |
   |                    [Gains shell]                   |                        |
   |                    [Reads config]                  |                        |
   |                    [Finds SSH keys]                |                        |
   |                          |                         |                        |
   |                          |--- SSH (stolen key) --->|                        |
   |                          |                         |                        |
   |                          |                   [Gains shell]                  |
   |                          |                   [Reads .env]                  |
   |                          |                   [Finds DB creds]              |
   |                          |                         |                        |
   |                          |                         |--- PostgreSQL -------->|
   |                          |                         |   (creds from .env)    |
   |                          |                         |                        |
   |                          |                         |                  [DUMP ALL DATA]
```

**Step-by-step:**

1. **Web-01 Compromise:** Attacker exploits a known Nginx vulnerability (e.g., CVE in a module, or a web app behind Nginx) to gain shell access as the web server user.

2. **Credential Harvesting on Web-01:** The attacker reads `/home/deploy/.ssh/id_rsa` (private key), configuration files with IP addresses, and `/etc/hosts` entries. They also find that password authentication is enabled.

3. **Lateral Move to App-01:** Using the stolen SSH key or password reuse, the attacker SSHs to App-01 (10.0.0.20). This works because the deploy user has SSH keys distributed to all servers.

4. **Credential Harvesting on App-01:** The attacker reads environment variables (DB_HOST, DB_PASSWORD) from the running Node.js process (`/proc/<pid>/environ`) or from `.env` files on disk. They find `DB_PASSWORD=supersecret123`.

5. **Database Compromise:** Using the PostgreSQL credentials, the attacker connects to DB-01 (10.0.0.30:5432) and dumps the entire database, including financial records, user credentials, and PII.

**Total hops: 3** (Web-01 -> App-01 -> DB-01)

### Why This Works
Each host contains credentials for the next hop. In flat networks, credential sprawl is inevitable because there is no network-level isolation to compensate for application-level trust. The attacker does not need to exploit new vulnerabilities at each hop -- stolen credentials are sufficient.

### Common Mistakes
- Storing credentials in environment variables (readable via /proc)
- Using the same SSH key across all servers
- Not considering that a web server compromise directly leads to database compromise in a flat network

## Part C: Segmentation Controls

| Lateral Movement Path | Segmentation Control | How It Blocks the Path |
|----------------------|---------------------|----------------------|
| Web-01 -> Web-02 (SSH) | Firewall rules | Only allow SSH from management subnet, not from web tier |
| Web-01 -> App-01 (port 3000) | VLAN/subnet isolation | Web tier and app tier on separate subnets; only load balancer can reach app tier |
| Web-01 -> App-01 (SSH) | Micro-segmentation | Host-based firewall blocks SSH from web servers; only jump box can SSH |
| Web-01 -> DB-01 (PostgreSQL) | VLAN/subnet isolation | Database on isolated subnet; only app tier allowed, not web tier |
| Web-01 -> DB-01 (SSH) | Micro-segmentation | Database subnet has no SSH access from web or app tiers; only jump box |
| Web-01 -> CI/CD (Jenkins 8080) | Firewall rules | Jenkins only accessible from management subnet |
| Web-01 -> CI/CD (Docker API 2375) | VLAN/subnet isolation + disable | Docker API should be on unix socket only, never TCP; management subnet isolated |
| Web-01 -> JumpBox (SSH) | Firewall rules | Jump box only accepts SSH from VPN or bastion, not from production subnets |

### Why This Works
Defense in depth means that even if one control fails (e.g., a firewall rule is too broad), another control (e.g., VLAN isolation) still blocks the attack. Micro-segmentation adds host-level controls that work even within the same subnet.

### Common Mistakes
- Relying on a single control (e.g., only VLANs without host firewalls)
- Forgetting that SSH between production servers should be blocked -- use a jump box
- Not addressing the Docker API exposure (port 2375 should never be open to TCP)

## Part D: Segmented Architecture

```
                        Internet
                           |
                     [Firewall/Router]
                           |
                  +--------+--------+
                  |    Public DMZ    |
                  |   10.0.1.0/24   |
                  |                 |
                  | Web-01 (.10)    |
                  | Web-02 (.11)    |
                  | ALB (internal)  |
                  +--------+--------+
                           |
                    [FW: 443 only]
                           |
                  +--------+--------+
                  |   App Tier      |
                  |  10.0.2.0/24    |
                  |                 |
                  | App-01 (.10)    |
                  | App-02 (.11)    |
                  | App-03 (.12)    |
                  +--------+--------+
                           |
                    [FW: 5432 only]
                           |
                  +--------+--------+
                  |   Data Tier     |
                  |  10.0.3.0/24    |
                  |                 |
                  | DB-01 (.10)     |
                  | DB-02 (.11)     |
                  +-----------------+

                  +-----------------+
                  |  Management     |
                  | 10.0.4.0/24     |
                  |                 |
                  | JumpBox (.10)   |
                  | CI/CD (.20)     |
                  | Monitoring (.30)|
                  +-----------------+
```

**Firewall Rules Between Segments:**

```
Public DMZ -> App Tier:     ALLOW TCP 3000 (API only)
Public DMZ -> Data Tier:    DENY ALL
Public DMZ -> Management:   DENY ALL

App Tier -> Public DMZ:     DENY ALL
App Tier -> Data Tier:      ALLOW TCP 5432 (PostgreSQL)
App Tier -> Management:     ALLOW TCP 9090 (metrics push)

Data Tier -> Public DMZ:    DENY ALL
Data Tier -> App Tier:      DENY ALL
Data Tier -> Management:    DENY ALL

Management -> Public DMZ:   ALLOW TCP 22 (SSH)
Management -> App Tier:     ALLOW TCP 22 (SSH)
Management -> Data Tier:    ALLOW TCP 22, 5432 (SSH + DB admin)
```

**Within each segment:** Micro-segmentation via host-based firewalls (iptables/nftables) restricts even intra-segment traffic. For example, within the app tier, App-01 does not need to talk to App-02 -- they only talk to the data tier.

### Why This Works
Each tier is a separate subnet (broadcast domain) with a firewall between them. The attacker who compromises Web-01 can only reach port 3000 on the app tier -- they cannot SSH, cannot reach the database, and cannot reach management. Even if they compromise an app server, the database is the only thing they can reach on port 5432 -- they still cannot reach management or the public DMZ in reverse.

### Common Mistakes
- Making the management subnet able to reach everything (it should be restricted too)
- Not blocking reverse connections (app tier should not be able to SSH to web tier)
- Forgetting to segment within a tier (web-01 should not talk to web-02)

## Key Takeaway
A flat network gives an attacker who compromises one host the keys to the entire kingdom. Segmentation forces the attacker through choke points where you can inspect, log, and block traffic. The goal is not to make compromise impossible -- it is to make lateral movement so difficult and noisy that you detect and contain the breach before the attacker reaches your crown jewels.
