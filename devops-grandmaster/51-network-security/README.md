# Module 51: Network Security

## Firewalls, VPNs, and Network Segmentation

**Previous Module:** [50 - Database Migration](../50-database-migration/README.md)
**Next Module:** [52 - DDoS Protection](../52-ddos-protection/README.md)

---

## 1. The Problem

Your application is running in production. Every port on every server is potentially exposed to the entire internet. Without network security controls, an attacker who compromises one service can freely move laterally to every other system. A single misconfigured database port exposed publicly can lead to a catastrophic breach.

The core challenges:
- Every service should not be reachable from every other service
- The internet should not be able to directly access your database
- Lateral movement between compromised hosts must be prevented
- Access to internal infrastructure must be controlled and auditable
- Network traffic must be monitored for anomalies

---

## 2. The Naive Way

```bash
# Open everything because "it works"
iptables -P INPUT ACCEPT
iptables -P FORWARD ACCEPT
iptables -P OUTPUT ACCEPT

# Or just disable the firewall entirely
systemctl stop firewalld
systemctl disable firewalld

# All services on the same flat network
# Database, app server, monitoring, everything on 10.0.0.0/16
```

Problems:
- No access control between services
- Any compromised host can reach every other host
- Database directly accessible from the internet
- No audit trail of network access
- No way to detect anomalous traffic patterns

---

## 3. The Right Way

### 3.1 Defense in Depth

Defense in depth means layering multiple security controls so that a failure in one layer does not result in a complete compromise.

```
Internet
    |
[Cloud Security Group / Firewall]
    |
[Load Balancer - Public Subnet]
    |
[Web Tier - Private Subnet]
    |
[App Tier - Private Subnet]
    |
[Data Tier - Private Subnet]
```

Each layer has its own firewall rules, and traffic must pass through every layer.

### 3.2 Firewall Rules with iptables

```bash
#!/bin/bash
# Basic iptables firewall configuration

# Flush existing rules
iptables -F
iptables -X
iptables -Z

# Default policies: drop everything
iptables -P INPUT DROP
iptables -P FORWARD DROP
iptables -P OUTPUT ACCEPT

# Allow loopback
iptables -A INPUT -i lo -j ACCEPT
iptables -A OUTPUT -o lo -j ACCEPT

# Allow established and related connections
iptables -A INPUT -m state --state ESTABLISHED,RELATED -j ACCEPT

# Allow SSH (port 22) only from management network
iptables -A INPUT -p tcp --dport 22 -s 10.0.1.0/24 -j ACCEPT

# Allow HTTP/HTTPS from anywhere
iptables -A INPUT -p tcp --dport 80 -j ACCEPT
iptables -A INPUT -p tcp --dport 443 -j ACCEPT

# Allow application port only from load balancer
iptables -A INPUT -p tcp --dport 8080 -s 10.0.0.10 -j ACCEPT

# Log and drop everything else
iptables -A INPUT -j LOG --log-prefix "IPTABLES-DROP: " --log-level 4
iptables -A INPUT -j DROP
```

### 3.3 UFW (Uncomplicated Firewall) for Simpler Setups

```bash
# Reset to defaults
ufw default deny incoming
ufw default allow outgoing

# Allow SSH from management network only
ufw allow from 10.0.1.0/24 to any port 22 proto tcp

# Allow HTTP and HTTPS
ufw allow 80/tcp
ufw allow 443/tcp

# Allow database access only from app tier
ufw allow from 10.0.3.0/24 to any port 5432 proto tcp

# Enable the firewall
ufw enable

# Check status
ufw status verbose
```

### 3.4 Cloud Security Groups (AWS Example)

```hcl
# Terraform: Security group for web tier
resource "aws_security_group" "web_tier" {
  name        = "web-tier-sg"
  description = "Security group for web tier"
  vpc_id      = aws_security_group.main.vpc_id

  ingress {
    description = "HTTP from internet"
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    description = "HTTPS from internet"
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    description = "SSH from bastion only"
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    security_groups = [aws_security_group.bastion.id]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

# Security group for database tier
resource "aws_security_group" "db_tier" {
  name        = "db-tier-sg"
  description = "Security group for database tier"
  vpc_id      = aws_security_group.main.vpc_id

  ingress {
    description     = "PostgreSQL from app tier only"
    from_port       = 5432
    to_port         = 5432
    protocol        = "tcp"
    security_groups = [aws_security_group.app_tier.id]
  }

  # No ingress from internet - completely private
}
```

### 3.5 Network Segmentation

#### VPC Design with Subnets

```
VPC: 10.0.0.0/16
    |
    +-- Public Subnet A (10.0.1.0/24) - Load balancers, bastion hosts
    |       Route: 0.0.0.0/0 -> Internet Gateway
    |
    +-- Private Subnet A (10.0.2.0/24) - Application servers
    |       Route: 0.0.0.0/0 -> NAT Gateway
    |
    +-- Private Subnet B (10.0.3.0/24) - Databases
    |       Route: No internet route (fully isolated)
    |
    +-- Management Subnet (10.0.10.0/24) - VPN, monitoring
            Route: 0.0.0.0/0 -> NAT Gateway
```

```hcl
# Terraform: VPC with segmented subnets
resource "aws_vpc" "main" {
  cidr_block           = "10.0.0.0/16"
  enable_dns_hostnames = true
  enable_dns_support   = true

  tags = {
    Name = "production-vpc"
  }
}

# Public subnet for load balancers
resource "aws_subnet" "public_a" {
  vpc_id                  = aws_vpc.main.id
  cidr_block              = "10.0.1.0/24"
  availability_zone       = "us-east-1a"
  map_public_ip_on_launch = true

  tags = { Name = "public-a" }
}

# Private subnet for applications
resource "aws_subnet" "private_app_a" {
  vpc_id            = aws_vpc.main.id
  cidr_block        = "10.0.2.0/24"
  availability_zone = "us-east-1a"

  tags = { Name = "private-app-a" }
}

# Private subnet for databases (no internet access)
resource "aws_subnet" "private_db_a" {
  vpc_id            = aws_vpc.main.id
  cidr_block        = "10.0.3.0/24"
  availability_zone = "us-east-1a"

  tags = { Name = "private-db-a" }
}
```

### 3.6 Bastion Hosts

A bastion host is a hardened server that serves as the single entry point for SSH access to internal infrastructure.

```hcl
# Bastion host - only SSH entry point
resource "aws_instance" "bastion" {
  ami                    = "ami-0abcdef1234567890"
  instance_type          = "t3.micro"
  subnet_id              = aws_subnet.public_a.id
  vpc_security_group_ids = [aws_security_group.bastion.id]
  key_name               = "bastion-key"

  # Hardened configuration
  user_data = <<-EOF
    #!/bin/bash
    # Only allow key-based authentication
    sed -i 's/PasswordAuthentication yes/PasswordAuthentication no/' /etc/ssh/sshd_config
    # Disable root login
    sed -i 's/PermitRootLogin yes/PermitRootLogin no/' /etc/ssh/sshd_config
    # Enable audit logging
    echo "session required pam_loginuid.so" >> /etc/pam.d/sshd
    systemctl restart sshd
  EOF
}

resource "aws_security_group" "bastion" {
  name   = "bastion-sg"
  vpc_id = aws_vpc.main.id

  ingress {
    description = "SSH from allowed IPs only"
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["203.0.113.0/24"]  # Office IP range
  }

  egress {
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["10.0.0.0/16"]  # Only to VPC
  }
}
```

SSH ProxyJump configuration:
```
# ~/.ssh/config
Host bastion
    HostName bastion.example.com
    User ec2-user
    IdentityFile ~/.ssh/bastion-key.pem

Host internal-*
    ProxyJump bastion
    User ec2-user
    IdentityFile ~/.ssh/internal-key.pem

Host internal-app-01
    HostName 10.0.2.10

Host internal-db-01
    HostName 10.0.3.10
```

### 3.7 VPN for Access Control

#### WireGuard VPN Setup

```ini
# /etc/wireguard/wg0.conf (VPN Server)
[Interface]
Address = 10.100.0.1/24
ListenPort = 51820
PrivateKey = <server-private-key>
PostUp = iptables -A FORWARD -i wg0 -j ACCEPT; iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE
PostDown = iptables -D FORWARD -i wg0 -j ACCEPT; iptables -t nat -D POSTROUTING -o eth0 -j MASQUERADE

# Peer: Developer laptop
[Peer]
PublicKey = <developer-public-key>
AllowedIPs = 10.100.0.2/32

# Peer: CI/CD server
[Peer]
PublicKey = <cicd-public-key>
AllowedIPs = 10.100.0.3/32
```

```ini
# Developer's client configuration
[Interface]
Address = 10.100.0.2/24
PrivateKey = <developer-private-key>
DNS = 10.0.0.2

[Peer]
PublicKey = <server-public-key>
Endpoint = vpn.example.com:51820
AllowedIPs = 10.0.0.0/16, 10.100.0.0/24
PersistentKeepalive = 25
```

### 3.8 Zero-Trust Networking Principles

Zero-trust networking assumes no implicit trust based on network location:

```yaml
# Zero-trust policy example (NetworkPolicy in Kubernetes)
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: api-server-policy
  namespace: production
spec:
  podSelector:
    matchLabels:
      app: api-server
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - podSelector:
            matchLabels:
              app: web-frontend
      ports:
        - protocol: TCP
          port: 8080
  egress:
    - to:
        - podSelector:
            matchLabels:
              app: database
      ports:
        - protocol: TCP
          port: 5432
    - to:  # Allow DNS
        - namespaceSelector: {}
          podSelector:
            matchLabels:
              k8s-app: kube-dns
      ports:
        - protocol: UDP
          port: 53
```

### 3.9 Network Monitoring

```yaml
# docker-compose.yml for network monitoring stack
version: '3.8'
services:
  # NetFlow/sFlow collector
  ntopng:
    image: ntop/ntopng:stable
    ports:
      - "3000:3000"
    command: "--interface=eth0 --http-port=3000"
    cap_add:
      - NET_ADMIN

  # Intrusion detection system
  suricata:
    image: jasonish/suricata:latest
    network_mode: host
    cap_add:
      - NET_ADMIN
      - NET_RAW
    volumes:
      - ./suricata.yaml:/etc/suricata/suricata.yaml
      - /var/log/suricata:/var/log/suricata

  # Log aggregation for network events
  elasticsearch:
    image: docker.elastic.co/elasticsearch/elasticsearch:8.10.0
    environment:
      - discovery.type=single-node
      - xpack.security.enabled=true
    volumes:
      - esdata:/usr/share/elasticsearch/data

volumes:
  esdata:
```

---

## 4. The Production Way

### 4.1 Complete Network Architecture

```
                    Internet
                       |
              [DDoS Protection (AWS Shield)]
                       |
              [CloudFront / CDN]
                       |
              [Application Load Balancer]
              (Public Subnet - 10.0.1.0/24)
                       |
          +------------+------------+
          |            |            |
    [Web Tier A]  [Web Tier B]  [Web Tier C]
    (10.0.2.10)   (10.0.2.11)   (10.0.2.12)
    Private Subnet - 10.0.2.0/24
          |            |            |
          +------------+------------+
                       |
              [Internal Load Balancer]
                       |
          +------------+------------+
          |            |            |
    [App Tier A]  [App Tier B]  [App Tier C]
    (10.0.3.10)   (10.0.3.11)   (10.0.3.12)
    Private Subnet - 10.0.3.0/24
          |            |            |
          +------------+------------+
                       |
          +------------+------------+
          |            |            |
    [DB Primary] [DB Replica A] [DB Replica B]
    (10.0.4.10)   (10.0.4.11)   (10.0.4.12)
    Private Subnet - 10.0.4.0/24 (NO INTERNET ROUTE)

    [Bastion Host]        [VPN Gateway]
    (10.0.1.100)          (10.0.1.101)
    Public Subnet         Public Subnet
    SSH Jump Box           WireGuard
```

### 4.2 Network Flow Rules

```bash
# Production iptables rules for app server
#!/bin/bash

# Flush
iptables -F INPUT

# Default deny
iptables -P INPUT DROP

# Allow loopback
iptables -A INPUT -i lo -j ACCEPT

# Allow established connections
iptables -A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT

# Allow SSH only from bastion
iptables -A INPUT -p tcp --dport 22 -s 10.0.1.100 -j ACCEPT

# Allow application port from internal ALB only
iptables -A INPUT -p tcp --dport 8080 -s 10.0.2.0/24 -j ACCEPT

# Allow monitoring agent from monitoring server
iptables -A INPUT -p tcp --dport 9100 -s 10.0.5.10 -j ACCEPT

# Allow DNS
iptables -A INPUT -p udp --dport 53 -s 10.0.0.2 -j ACCEPT

# Log drops
iptables -A INPUT -j LOG --log-prefix "FW-DROP: " --log-level 4
iptables -A INPUT -j DROP

# Save rules
iptables-save > /etc/iptables/rules.v4
```

### 4.3 Security Group Audit Script

```bash
#!/bin/bash
# audit-security-groups.sh
# Audit AWS security groups for overly permissive rules

echo "=== Security Group Audit ==="
echo ""

# Find security groups with 0.0.0.0/0 ingress
echo "Groups allowing ingress from 0.0.0.0/0 (anywhere):"
aws ec2 describe-security-groups \
    --query 'SecurityGroups[?IpPermissions[?IpRanges[?CidrIp==`0.0.0.0/0`]]].[GroupId,GroupName,IpPermissions[?IpRanges[?CidrIp==`0.0.0.0/0`]].{Port:FromPort,Protocol:IpProtocol}]' \
    --output table

echo ""
echo "Groups with unrestricted egress (0.0.0.0/0 on all ports):"
aws ec2 describe-security-groups \
    --query 'SecurityGroups[?IpPermissionsEgress[?IpProtocol==`-1` && IpRanges[?CidrIp==`0.0.0.0/0`]]].[GroupId,GroupName]' \
    --output table

echo ""
echo "Groups allowing SSH (port 22) from anywhere:"
aws ec2 describe-security-groups \
    --filters "Name=ip-permission.from-port,Values=22" \
              "Name=ip-permission.cidr,Values=0.0.0.0/0" \
    --query 'SecurityGroups[*].[GroupId,GroupName]' \
    --output table

echo ""
echo "Groups allowing RDP (port 3389) from anywhere:"
aws ec2 describe-security-groups \
    --filters "Name=ip-permission.from-port,Values=3389" \
              "Name=ip-permission.cidr,Values=0.0.0.0/0" \
    --query 'SecurityGroups[*].[GroupId,GroupName]' \
    --output table
```

### 4.4 VPC Flow Logs Analysis

```bash
# Enable VPC Flow Logs
aws ec2 create-flow-logs \
    --resource-type VPC \
    --resource-ids vpc-0123456789abcdef0 \
    --traffic-type ALL \
    --log-destination-type cloud-watch-logs \
    --log-group-name /aws/vpc/flowlogs \
    --deliver-logs-permission-arn arn:aws:iam::123456789012:role/vpc-flow-logs-role

# Query rejected connections (potential attack attempts)
aws logs filter-log-events \
    --log-group-name /aws/vpc/flowlogs \
    --filter-pattern "REJECT" \
    --start-time $(date -d '1 hour ago' +%s000) \
    --query 'events[*].message' \
    --output text | \
    awk '{print $4}' | sort | uniq -c | sort -rn | head -20
```

---

## 5. Hands-On Lab

### Lab: Set Up Firewall Rules and Network Segmentation

**Objective:** Implement network segmentation with firewall rules for a multi-tier application.

#### Step 1: Create a Segmented Network

```bash
# Create directory structure
mkdir -p lab-network-security && cd lab-network-security

# Create docker-compose with segmented networks
cat > docker-compose.yml << 'EOF'
version: '3.8'

networks:
  public:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/24
  app:
    driver: bridge
    internal: true
    ipam:
      config:
        - subnet: 172.20.1.0/24
  data:
    driver: bridge
    internal: true
    ipam:
      config:
        - subnet: 172.20.2.0/24

services:
  # Bastion host - only entry point
  bastion:
    image: alpine:latest
    command: >
      sh -c "
        apk add --no-cache openssh-server &&
        ssh-keygen -A &&
        echo 'root:changeme' | chpasswd &&
        /usr/sbin/sshd -D
      "
    networks:
      public:
        ipv4_address: 172.20.0.10
      app:
        ipv4_address: 172.20.1.10
    ports:
      - "2222:22"

  # Web server - public network and app network
  web:
    image: nginx:alpine
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
    networks:
      public:
        ipv4_address: 172.20.0.20
      app:
        ipv4_address: 172.20.1.20
    ports:
      - "8080:80"
    depends_on:
      - app

  # Application server - app network and data network
  app:
    image: node:18-alpine
    command: >
      sh -c "
        mkdir -p /app &&
        cat > /app/server.js << 'SERVEREOF'
        const http = require('http');
        const { Pool } = require('pg');
        const pool = new Pool({
          host: process.env.DB_HOST || '172.20.2.30',
          port: 5432,
          user: 'app',
          password: 'secretpass',
          database: 'appdb'
        });
        const server = http.createServer(async (req, res) => {
          if (req.url === '/health') {
            res.writeHead(200);
            res.end('OK');
          } else if (req.url === '/users') {
            try {
              const result = await pool.query('SELECT * FROM users LIMIT 10');
              res.writeHead(200, {'Content-Type': 'application/json'});
              res.end(JSON.stringify(result.rows));
            } catch (err) {
              res.writeHead(500);
              res.end('Database error');
            }
          }
        });
        server.listen(3000, () => console.log('App running on :3000'));
        SERVEREOF
        npm init -y && npm install pg && node /app/server.js
      "
    environment:
      - DB_HOST=172.20.2.30
    networks:
      app:
        ipv4_address: 172.20.1.30
      data:
        ipv4_address: 172.20.2.10

  # Database - data network only (isolated)
  db:
    image: postgres:15-alpine
    environment:
      - POSTGRES_USER=app
      - POSTGRES_PASSWORD=secretpass
      - POSTGRES_DB=appdb
    volumes:
      - ./init.sql:/docker-entrypoint-initdb.d/init.sql
    networks:
      data:
        ipv4_address: 172.20.2.30
    # Note: NO port mapping to host - not directly accessible
EOF

cat > nginx.conf << 'EOF'
events {
    worker_connections 1024;
}
http {
    upstream app {
        server 172.20.1.30:3000;
    }
    server {
        listen 80;
        location / {
            proxy_pass http://app;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }
    }
}
EOF

cat > init.sql << 'EOF'
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100),
    email VARCHAR(100)
);
INSERT INTO users (name, email) VALUES
    ('Alice', 'alice@example.com'),
    ('Bob', 'bob@example.com'),
    ('Charlie', 'charlie@example.com');
EOF
```

#### Step 2: Verify Network Isolation

```bash
# Start the lab
docker-compose up -d

# Test 1: Web server should be accessible from host
echo "Test 1: Web server accessible"
curl -s http://localhost:8080/health
# Expected: OK

# Test 2: Database should NOT be accessible from host
echo "Test 2: Database not accessible from host"
curl -s --max-time 3 http://localhost:5432
# Expected: Connection refused or timeout

# Test 3: Web server cannot directly access database
echo "Test 3: Web cannot reach database"
docker exec lab-network-security-web-1 \
    wget -q -O- --timeout=3 http://172.20.2.30:5432 2>&1 || echo "Blocked (expected)"

# Test 4: App server can access database
echo "Test 4: App can reach database"
docker exec lab-network-security-app-1 \
    node -e "
      const { Pool } = require('pg');
      const pool = new Pool({
        host: '172.20.2.30',
        user: 'app',
        password: 'secretpass',
        database: 'appdb'
      });
      pool.query('SELECT COUNT(*) FROM users')
        .then(r => { console.log('DB query success:', r.rows[0]); process.exit(0); })
        .catch(e => { console.error('DB query failed:', e.message); process.exit(1); });
    "

# Test 5: Full request flow
echo "Test 5: Full request flow"
curl -s http://localhost:8080/users
```

#### Step 3: Add Host-Level Firewall Rules

```bash
# Add iptables rules to app server container
docker exec lab-network-security-app-1 sh -c "
    # Install iptables
    apk add --no-cache iptables

    # Default deny incoming
    iptables -P INPUT DROP

    # Allow loopback
    iptables -A INPUT -i lo -j ACCEPT

    # Allow established connections
    iptables -A INPUT -m state --state ESTABLISHED,RELATED -j ACCEPT

    # Allow app port from web tier only
    iptables -A INPUT -p tcp --dport 3000 -s 172.20.1.20 -j ACCEPT

    # Allow SSH from bastion only
    iptables -A INPUT -p tcp --dport 22 -s 172.20.1.10 -j ACCEPT

    # Log and drop rest
    iptables -A INPUT -j LOG --log-prefix 'FW-DROP: '
    iptables -A INPUT -j DROP

    # Show rules
    iptables -L -n -v
"
```

#### Step 4: Implement Network Monitoring

```bash
# Install tcpdump on app server for monitoring
docker exec lab-network-security-app-1 sh -c "
    apk add --no-cache tcpdump
    tcpdump -i eth0 -n -c 50 &
"

# Generate traffic to see what's allowed and what's dropped
curl http://localhost:8080/users

# Check logs for dropped packets
docker exec lab-network-security-app-1 sh -c "
    dmesg | grep FW-DROP | tail -10
"
```

#### Step 5: Cleanup

```bash
docker-compose down -v
```

**Expected Results:**
- Web tier accessible from public network
- Database isolated on data network only
- App server communicates with both web and data tiers
- Firewall rules restrict traffic to expected paths only
- Monitoring captures allowed and denied connections

---

## 6. Limitation -> Next Topic

Network security controls the flow of traffic between services and prevents unauthorized access at the network layer. However, network-level controls alone cannot protect against all attack vectors.

**What network security cannot do:**
- Cannot protect against application-layer attacks (SQL injection, XSS)
- Cannot stop a DDoS attack that overwhelms your bandwidth
- Cannot prevent credential theft through phishing
- Cannot detect vulnerabilities in your application code
- Cannot automatically respond to traffic anomalies

The next layer of defense is protecting against volumetric and application-layer denial-of-service attacks.

**Next Module:** [52 - DDoS Protection](../52-ddos-protection/README.md) -- Rate limiting, traffic analysis, and DDoS mitigation strategies.
