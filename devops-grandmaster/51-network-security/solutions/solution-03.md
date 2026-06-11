# Solution 03: Design a Production Network Architecture

## Part A: VPC and Subnet Design

### VPC CIDR Allocation

```
VPC: 10.0.0.0/16 (65,536 IPs)

Subnet Layout:
+---------------------------+------------------+------------------+----------+---------+
| Subnet Name               | CIDR             | AZ               | Public?  | Resources |
+---------------------------+------------------+------------------+----------+---------+
| public-web-1a             | 10.0.1.0/24      | us-east-1a       | Yes      | ALB, NAT GW |
| public-web-1b             | 10.0.2.0/24      | us-east-1b       | Yes      | ALB, NAT GW |
| private-app-1a            | 10.0.10.0/24     | us-east-1a       | No       | Container cluster |
| private-app-1b            | 10.0.11.0/24     | us-east-1b       | No       | Container cluster |
| private-data-1a           | 10.0.20.0/24     | us-east-1a       | No       | PostgreSQL primary, Redis |
| private-data-1b           | 10.0.21.0/24     | us-east-1b       | No       | PostgreSQL replica |
| private-mgmt-1a           | 10.0.30.0/24     | us-east-1a       | No       | Monitoring, CI/CD |
| private-mgmt-1b           | 10.0.31.0/24     | us-east-1b       | No       | VPN endpoint |
+---------------------------+------------------+------------------+----------+---------+
```

### ASCII Architecture Diagram

```
                         Internet
                            |
                    [Internet Gateway]
                            |
              +-------------+-------------+
              |                           |
    +---------+---------+     +-----------+---------+
    |  public-web-1a    |     |  public-web-1b      |
    |  10.0.1.0/24      |     |  10.0.2.0/24        |
    |                   |     |                     |
    |  ALB (internet)   |     |  ALB (internet)     |
    |  NAT Gateway      |     |  NAT Gateway        |
    +---------+---------+     +-----------+---------+
              |                           |
              +-------------+-------------+
                            |
                    [App Tier SG]
                            |
              +-------------+-------------+
              |                           |
    +---------+---------+     +-----------+---------+
    |  private-app-1a   |     |  private-app-1b     |
    |  10.0.10.0/24     |     |  10.0.11.0/24       |
    |                   |     |                     |
    |  Web servers (2)  |     |  Web servers (2)    |
    |  App servers (2)  |     |  App servers (1)    |
    +---------+---------+     +-----------+---------+
              |                           |
              +-------------+-------------+
                            |
                    [Data Tier SG]
                            |
              +-------------+-------------+
              |                           |
    +---------+---------+     +-----------+---------+
    |  private-data-1a  |     |  private-data-1b    |
    |  10.0.20.0/24     |     |  10.0.21.0/24       |
    |                   |     |                     |
    |  PostgreSQL (pri) |     |  PostgreSQL (rep)   |
    |  Redis            |     |                     |
    +-------------------+     +---------------------+

    +-------------------+     +---------------------+
    |  private-mgmt-1a  |     |  private-mgmt-1b    |
    |  10.0.30.0/24     |     |  10.0.31.0/24       |
    |                   |     |                     |
    |  Prometheus       |     |  WireGuard VPN      |
    |  Grafana          |     |  CI/CD (Jenkins)    |
    |  CI/CD runner     |     |                     |
    +-------------------+     +---------------------+
```

### Route Tables

```
Public subnets:
  10.0.0.0/16  -> local
  0.0.0.0/0    -> Internet Gateway

Private app subnets:
  10.0.0.0/16  -> local
  0.0.0.0/0    -> NAT Gateway (in public subnet)

Private data subnets:
  10.0.0.0/16  -> local
  (NO internet route -- no NAT Gateway)

Private mgmt subnets:
  10.0.0.0/16  -> local
  0.0.0.0/0    -> NAT Gateway (for updates and CI/CD)
```

### Why This Works

The /16 VPC provides 65,536 IPs, giving ample room for growth. Each tier gets its own subnet pair across two AZs for high availability. The data tier has no route to the internet, satisfying the compliance requirement. NAT Gateways in public subnets allow private subnets to reach the internet for updates without being directly accessible. The management subnet is separate from the data tier, so CI/CD and monitoring cannot directly access the database -- they go through the app tier.

### Common Mistakes

- **Placing databases in public subnets.** The data tier must have no internet gateway route.
- **Using only one AZ.** A single AZ failure takes down the entire application.
- **Not sizing the CIDR block.** A /24 VPC runs out of IPs quickly when containerized workloads scale.
- **Forgetting NAT Gateway for private subnets.** Without NAT, private instances cannot download updates.

---

## Part B: Security Groups

### ALB Security Group

```yaml
Name: sg-alb
Description: Application Load Balancer - internet-facing
Inbound:
  - Source: 0.0.0.0/0
    Port: 443
    Protocol: TCP
    Description: HTTPS from internet
  - Source: 0.0.0.0/0
    Port: 80
    Protocol: TCP
    Description: HTTP (redirect to HTTPS)
Outbound:
  - Destination: sg-app-servers
    Port: 3000
    Protocol: TCP
    Description: Forward to app servers
```

### App Server Security Group

```yaml
Name: sg-app-servers
Description: Application servers - private tier
Inbound:
  - Source: sg-alb
    Port: 3000
    Protocol: TCP
    Description: Traffic from ALB
  - Source: sg-mgmt
    Port: 22
    Protocol: TCP
    Description: SSH from management
Outbound:
  - Destination: sg-database
    Port: 5432
    Protocol: TCP
    Description: PostgreSQL
  - Destination: sg-database
    Port: 6379
    Protocol: TCP
    Description: Redis
  - Destination: 0.0.0.0/0
    Port: 443
    Protocol: TCP
    Description: External API calls (HTTPS)
```

### Database Security Group

```yaml
Name: sg-database
Description: Database tier - most restricted
Inbound:
  - Source: sg-app-servers
    Port: 5432
    Protocol: TCP
    Description: PostgreSQL from app servers
  - Source: sg-app-servers
    Port: 6379
    Protocol: TCP
    Description: Redis from app servers
  - Source: sg-mgmt
    Port: 5432
    Protocol: TCP
    Description: DB admin from management (emergency)
Outbound:
  - (none -- database does not initiate outbound connections)
```

### Monitoring Security Group

```yaml
Name: sg-monitoring
Description: Prometheus/Grafana - monitoring infrastructure
Inbound:
  - Source: sg-mgmt
    Port: 443
    Protocol: TCP
    Description: Grafana dashboard (via VPN)
  - Source: sg-app-servers
    Port: 9090
    Protocol: TCP
    Description: Prometheus metrics scrape
Outbound:
  - Destination: 0.0.0.0/0
    Port: 443
    Protocol: TCP
    Description: Alertmanager webhook notifications
```

### CI/CD Security Group

```yaml
Name: sg-cicd
Description: CI/CD pipeline server
Inbound:
  - Source: sg-mgmt
    Port: 22
    Protocol: TCP
    Description: SSH from mgmt subnet
  - Source: sg-mgmt
    Port: 443
    Protocol: TCP
    Description: Jenkins/GitLab UI
Outbound:
  - Destination: sg-app-servers
    Port: 22
    Protocol: TCP
    Description: Deploy to app servers
  - Destination: 0.0.0.0/0
    Port: 443
    Protocol: TCP
    Description: Pull container images, push to registry
```

### VPN Security Group

```yaml
Name: sg-vpn
Description: WireGuard VPN endpoint
Inbound:
  - Source: 0.0.0.0/0
    Port: 51820
    Protocol: UDP
    Description: WireGuard from internet
Outbound:
  - Destination: sg-monitoring
    Port: 443
    Protocol: TCP
    Description: Grafana access for VPN users
  - Destination: sg-app-servers
    Port: 22
    Protocol: TCP
    Description: SSH for engineers
```

### Why This Works

Every security group uses security group references (sg-xxxx) rather than CIDR ranges. This means if you add more app servers, the database security group automatically allows them without rule changes. The database has no outbound rules -- it should not initiate connections. The VPN has minimal outbound access, only to the services engineers need.

### Common Mistakes

- **Using CIDR ranges instead of security group references.** CIDR ranges must be manually updated when scaling.
- **Giving the database outbound internet access.** Databases should not initiate outbound connections.
- **Opening SSH to 0.0.0.0/0.** SSH should only come through VPN or bastion.
- **Not restricting egress.** Outbound rules prevent compromised instances from exfiltrating data.

---

## Part C: VPN and Remote Access

### Technology Choice: WireGuard

**Why WireGuard over OpenVPN or AWS Client VPN:**
- Simpler configuration (single config file per peer)
- Better performance (kernel-space implementation, lower latency)
- Modern cryptography (ChaCha20, Curve25519)
- Smaller attack surface (~4,000 lines of code vs OpenVPN's ~100,000)

### WireGuard Server Configuration

```ini
# /etc/wireguard/wg0.conf (on bastion in private-mgmt-1b)
[Interface]
Address = 10.100.0.1/24
ListenPort = 51820
PrivateKey = <server-private-key>
PostUp = iptables -A FORWARD -i wg0 -j ACCEPT; iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE
PostDown = iptables -D FORWARD -i wg0 -j ACCEPT; iptables -t nat -D POSTROUTING -o eth0 -j MASQUERADE

# DNS resolution for internal services
# Run CoreDNS or dnsmasq that resolves *.internal.datavault.io to internal IPs

[Peer]
# DevOps Engineers group
PublicKey = <devops-team-key>
AllowedIPs = 10.100.0.2/32, 10.0.30.0/24, 10.0.31.0/24
# Provides access to: mgmt subnets only

[Peer]
# DBA group
PublicKey = <dba-team-key>
AllowedIPs = 10.100.0.3/32, 10.0.20.0/24, 10.0.21.0/24, 10.0.30.0/24
# Provides access to: data subnets + mgmt subnets
```

### Client Configuration Template

```ini
# devops-engineer.conf
[Interface]
PrivateKey = <client-private-key>
Address = 10.100.0.2/24
DNS = 10.100.0.1

[Peer]
PublicKey = <server-public-key>
Endpoint = vpn.datavault.io:51820
AllowedIPs = 10.0.30.0/24, 10.0.31.0/24
PersistentKeepalive = 25
```

### Authentication

- **Primary:** SSO integration via OIDC (e.g., Okta, Azure AD) -- users authenticate before receiving WireGuard config
- **Secondary:** Pre-shared keys per peer for additional protection against quantum computing attacks
- **Rotation:** Peer keys rotated every 90 days via automated script

### DNS Resolution Through VPN

```
Split DNS configuration:
  *.internal.datavault.io -> resolved by 10.100.0.1 (VPN DNS)
  grafana.internal.datavault.io -> 10.0.30.15 (Grafana)
  jenkins.internal.datavault.io -> 10.0.31.10 (Jenkins)
  prometheus.internal.datavault.io -> 10.0.30.20 (Prometheus)
```

### Why This Works

WireGuard provides encrypted tunnel from the engineer's laptop to the bastion host in the management subnet. The `AllowedIPs` directive on each peer acts as a routing rule and firewall -- DevOps engineers can only reach the management subnet, while DBAs can additionally reach the data tier. Split DNS ensures that internal service names resolve only when connected to the VPN.

### Common Mistakes

- **Giving VPN users access to all subnets.** Use `AllowedIPs` to restrict per role.
- **Not rotating keys.** Compromised keys grant persistent access.
- **Storing WireGuard configs in version control.** These contain private keys.
- **Not using PersistentKeepalive.** NAT traversal requires keepalives.

---

## Part D: Network Monitoring and Logging

### VPC Flow Logs

```hcl
# vpc-flow-logs.tf
resource "aws_flow_log" "main" {
  vpc_id                   = aws_vpc.main.id
  traffic_type             = "ALL"
  log_destination_type     = "cloud-watch-logs"
  log_destination          = aws_cloudwatch_log_group.flow_logs.arn
  iam_role_arn             = aws_iam_role.flow_logs.arn
  max_aggregation_interval = 60  # 60-second granularity

  tags = {
    Name        = "datavault-flow-logs"
    Environment = "production"
    Retention   = "90-days"
  }
}

resource "aws_cloudwatch_log_group" "flow_logs" {
  name              = "/aws/vpc/flow-logs"
  retention_in_days = 90
}
```

### Security Group Rule Hit Logging

```hcl
# Enable VPC-level DNS query logging
resource "aws_flow_log" "reject_only" {
  vpc_id               = aws_vpc.main.id
  traffic_type         = "REJECT"
  log_destination_type = "s3"
  log_destination      = aws_s3_bucket.flow_logs.arn

  destination_options {
    file_format        = "parquet"
    per_hour_partition  = true
  }
}
```

### Alert Thresholds

```
Alert 1: Rejected connections from unexpected sources
  Condition: REJECT flow from source NOT in known CIDR list
  Threshold: > 100 in 5 minutes
  Action: PagerDuty P3

Alert 2: Data exfiltration pattern
  Condition: Outbound bytes > 1GB to single external IP in 1 hour
  Threshold: > 1GB/hour to single destination
  Action: PagerDuty P1, auto-block via security group

Alert 3: Port scanning activity
  Condition: REJECT flows from single source to > 20 different ports
  Threshold: > 20 ports in 60 seconds
  Action: PagerDuty P2, auto-block source IP

Alert 4: Database connection anomaly
  Condition: New source IP connecting to database port 5432
  Threshold: Any new source (baseline deviation)
  Action: PagerDuty P1 (database should only accept from known app SG)

Alert 5: Off-hours administrative access
  Condition: SSH connection outside business hours (UTC)
  Threshold: Any SSH between 22:00-06:00 UTC
  Action: Slack notification + audit log
```

### Log Retention and Analysis Pipeline

```
VPC Flow Logs
    |
    +--> CloudWatch Logs (real-time, 90-day retention)
    |       |
    |       +--> CloudWatch Metric Filters
    |       |       |
    |       |       +--> SNS -> PagerDuty (alerts)
    |       |       +--> SNS -> Slack (notifications)
    |       |
    |       +--> CloudWatch Insights (ad-hoc queries)
    |
    +--> S3 (long-term, Parquet format, 1-year retention)
            |
            +--> Athena (SQL queries over historical data)
            +--> QuickSight (dashboards)
```

### Why This Works

VPC Flow Logs capture metadata about every network flow: source IP, destination IP, port, protocol, bytes transferred, and accept/reject status. Sending them to CloudWatch enables real-time alerting, while S3 provides cost-effective long-term storage for compliance. Parquet format in S3 enables efficient SQL queries via Athena. The alert thresholds are tuned to catch real threats without excessive false positives.

### Common Mistakes

- **Logging ALL traffic to CloudWatch.** This is expensive. Log ACCEPT to S3, REJECT to CloudWatch for real-time alerting.
- **Not setting retention policies.** Logs accumulate forever, increasing costs.
- **Alerting on every REJECT.** Internet-facing services receive constant port scans. Alert on patterns, not individual rejects.
- **Not correlating across services.** A VPC Flow Log REJECT + a database audit log = high-confidence attack signal.

---

## Part E: Terraform Skeleton

```hcl
# variables.tf
variable "environment" {
  description = "Environment name"
  type        = string
  default     = "production"
}

variable "vpc_cidr" {
  description = "VPC CIDR block"
  type        = string
  default     = "10.0.0.0/16"
}

variable "availability_zones" {
  description = "List of availability zones"
  type        = list(string)
  default     = ["us-east-1a", "us-east-1b"]
}

# main.tf
terraform {
  required_version = ">= 1.5"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
  backend "s3" {
    bucket = "datavault-terraform-state"
    key    = "network/terraform.tfstate"
    region = "us-east-1"
  }
}

provider "aws" {
  region = "us-east-1"
}

# vpc.tf
resource "aws_vpc" "main" {
  cidr_block           = var.vpc_cidr
  enable_dns_hostnames = true
  enable_dns_support   = true

  tags = {
    Name        = "${var.environment}-vpc"
    Environment = var.environment
  }
}

resource "aws_internet_gateway" "main" {
  vpc_id = aws_vpc.main.id
  tags   = { Name = "${var.environment}-igw" }
}

# --- Public Subnets (Web Tier) ---
resource "aws_subnet" "public" {
  count                   = length(var.availability_zones)
  vpc_id                  = aws_vpc.main.id
  cidr_block              = cidrsubnet(var.vpc_cidr, 8, count.index + 1)
  availability_zone       = var.availability_zones[count.index]
  map_public_ip_on_launch = true

  tags = {
    Name = "${var.environment}-public-${var.availability_zones[count.index]}"
    Tier = "public"
  }
}

# --- Private App Subnets ---
resource "aws_subnet" "private_app" {
  count             = length(var.availability_zones)
  vpc_id            = aws_vpc.main.id
  cidr_block        = cidrsubnet(var.vpc_cidr, 8, count.index + 10)
  availability_zone = var.availability_zones[count.index]

  tags = {
    Name = "${var.environment}-private-app-${var.availability_zones[count.index]}"
    Tier = "private-app"
  }
}

# --- Private Data Subnets ---
resource "aws_subnet" "private_data" {
  count             = length(var.availability_zones)
  vpc_id            = aws_vpc.main.id
  cidr_block        = cidrsubnet(var.vpc_cidr, 8, count.index + 20)
  availability_zone = var.availability_zones[count.index]

  tags = {
    Name = "${var.environment}-private-data-${var.availability_zones[count.index]}"
    Tier = "private-data"
  }
}

# --- Private Management Subnets ---
resource "aws_subnet" "private_mgmt" {
  count             = length(var.availability_zones)
  vpc_id            = aws_vpc.main.id
  cidr_block        = cidrsubnet(var.vpc_cidr, 8, count.index + 30)
  availability_zone = var.availability_zones[count.index]

  tags = {
    Name = "${var.environment}-private-mgmt-${var.availability_zones[count.index]}"
    Tier = "private-mgmt"
  }
}

# --- NAT Gateway (for private subnet internet access) ---
resource "aws_eip" "nat" {
  count  = length(var.availability_zones)
  domain = "vpc"
  tags   = { Name = "${var.environment}-nat-eip-${count.index}" }
}

resource "aws_nat_gateway" "main" {
  count         = length(var.availability_zones)
  allocation_id = aws_eip.nat[count.index].id
  subnet_id     = aws_subnet.public[count.index].id
  tags          = { Name = "${var.environment}-natgw-${count.index}" }
}

# --- Route Tables ---
resource "aws_route_table" "public" {
  vpc_id = aws_vpc.main.id
  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.main.id
  }
  tags = { Name = "${var.environment}-public-rt" }
}

resource "aws_route_table" "private_app" {
  count  = length(var.availability_zones)
  vpc_id = aws_vpc.main.id
  route {
    cidr_block     = "0.0.0.0/0"
    nat_gateway_id = aws_nat_gateway.main[count.index].id
  }
  tags = { Name = "${var.environment}-private-app-rt-${count.index}" }
}

resource "aws_route_table" "private_data" {
  vpc_id = aws_vpc.main.id
  # No internet route -- data tier is fully isolated
  tags = { Name = "${var.environment}-private-data-rt" }
}

resource "aws_route_table" "private_mgmt" {
  count  = length(var.availability_zones)
  vpc_id = aws_vpc.main.id
  route {
    cidr_block     = "0.0.0.0/0"
    nat_gateway_id = aws_nat_gateway.main[count.index].id
  }
  tags = { Name = "${var.environment}-private-mgmt-rt-${count.index}" }
}

# --- Route Table Associations ---
resource "aws_route_table_association" "public" {
  count          = length(var.availability_zones)
  subnet_id      = aws_subnet.public[count.index].id
  route_table_id = aws_route_table.public.id
}

resource "aws_route_table_association" "private_app" {
  count          = length(var.availability_zones)
  subnet_id      = aws_subnet.private_app[count.index].id
  route_table_id = aws_route_table.private_app[count.index].id
}

resource "aws_route_table_association" "private_data" {
  count          = length(var.availability_zones)
  subnet_id      = aws_subnet.private_data[count.index].id
  route_table_id = aws_route_table.private_data.id
}

resource "aws_route_table_association" "private_mgmt" {
  count          = length(var.availability_zones)
  subnet_id      = aws_subnet.private_mgmt[count.index].id
  route_table_id = aws_route_table.private_mgmt[count.index].id
}

# security-groups.tf
resource "aws_security_group" "alb" {
  name        = "${var.environment}-alb-sg"
  description = "ALB security group"
  vpc_id      = aws_vpc.main.id

  ingress {
    description = "HTTPS"
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    description = "HTTP (redirect)"
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port       = 3000
    to_port         = 3000
    protocol        = "tcp"
    security_groups = [aws_security_group.app.id]
  }

  tags = { Name = "${var.environment}-alb-sg" }
}

resource "aws_security_group" "app" {
  name        = "${var.environment}-app-sg"
  description = "Application servers"
  vpc_id      = aws_vpc.main.id

  ingress {
    description     = "From ALB"
    from_port       = 3000
    to_port         = 3000
    protocol        = "tcp"
    security_groups = [aws_security_group.alb.id]
  }

  ingress {
    description     = "SSH from mgmt"
    from_port       = 22
    to_port         = 22
    protocol        = "tcp"
    security_groups = [aws_security_group.mgmt.id]
  }

  egress {
    from_port       = 5432
    to_port         = 5432
    protocol        = "tcp"
    security_groups = [aws_security_group.database.id]
  }

  egress {
    from_port       = 6379
    to_port         = 6379
    protocol        = "tcp"
    security_groups = [aws_security_group.database.id]
  }

  egress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = { Name = "${var.environment}-app-sg" }
}

resource "aws_security_group" "database" {
  name        = "${var.environment}-database-sg"
  description = "Database tier - most restricted"
  vpc_id      = aws_vpc.main.id

  ingress {
    description     = "PostgreSQL from app"
    from_port       = 5432
    to_port         = 5432
    protocol        = "tcp"
    security_groups = [aws_security_group.app.id]
  }

  ingress {
    description     = "Redis from app"
    from_port       = 6379
    to_port         = 6379
    protocol        = "tcp"
    security_groups = [aws_security_group.app.id]
  }

  ingress {
    description     = "DB admin from mgmt"
    from_port       = 5432
    to_port         = 5432
    protocol        = "tcp"
    security_groups = [aws_security_group.mgmt.id]
  }

  # No egress -- database does not initiate outbound connections

  tags = { Name = "${var.environment}-database-sg" }
}

resource "aws_security_group" "mgmt" {
  name        = "${var.environment}-mgmt-sg"
  description = "Management and monitoring"
  vpc_id      = aws_vpc.main.id

  ingress {
    description = "WireGuard VPN"
    from_port   = 51820
    to_port     = 51820
    protocol    = "udp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = { Name = "${var.environment}-mgmt-sg" }
}

# vpn.tf
resource "aws_ec2_client_vpn_endpoint" "main" {
  description            = "DataVault VPN"
  client_cidr_block      = "10.100.0.0/24"
  server_certificate_arn = aws_acm_certificate.vpn.arn
  split_tunnel           = true

  authentication_options {
    type                       = "federated-authentication"
    saml_provider_arn          = aws_iam_saml_provider.vpn.arn
  }

  connection_log_options {
    enabled               = true
    cloudwatch_log_group  = aws_cloudwatch_log_group.vpn.name
  }

  tags = { Name = "${var.environment}-vpn" }
}

resource "aws_ec2_client_vpn_network_association" "mgmt" {
  count                  = length(var.availability_zones)
  client_vpn_endpoint_id = aws_ec2_client_vpn_endpoint.main.id
  subnet_id              = aws_subnet.private_mgmt[count.index].id
}

resource "aws_ec2_client_vpn_authorization_rule" "mgmt" {
  client_vpn_endpoint_id = aws_ec2_client_vpn_endpoint.main.id
  target_network_cidr    = "10.0.30.0/24"
  authorize_all_groups   = true
  description            = "Allow VPN access to management subnet"
}
```

### Why This Works

The Terraform skeleton declares the complete infrastructure with proper resource dependencies. The VPC is created first, then subnets, then route tables, then security groups, then VPN. Variables allow the same configuration to be used across environments. The `cidrsubnet` function automatically calculates subnet CIDRs from the VPC CIDR. Security group references ensure that changes to one group propagate to dependent groups.

### Common Mistakes

- **Hardcoding CIDR ranges in security groups.** Use security group references (`security_groups = [...]`) instead.
- **Not using remote state.** Local state files get out of sync when multiple engineers work on the same infrastructure.
- **Creating all subnets in one AZ.** High availability requires at least two AZs.
- **Not tagging resources.** Tags are essential for cost allocation, access control, and operational management.

---

## Common Mistakes to Avoid

- **Overcomplicating the CIDR scheme.** A /16 VPC with /24 subnets is simple and provides enough IPs for most workloads.
- **Not planning for growth.** Leave room in your CIDR allocation for future subnets and services.
- **Forgetting about egress traffic.** Security groups often focus on inbound rules while ignoring outbound, which is how data exfiltration happens.
- **Treating VPN as a replacement for zero-trust.** VPN provides network access, but services should still authenticate every request.

## Key Takeaway

Cloud network design is about translating security requirements into network topology. Every subnet represents a trust boundary. Every security group rule represents an explicit exception to the default-deny posture. The VPN provides controlled remote access without exposing management interfaces to the public internet. Infrastructure-as-code ensures the design is reproducible, auditable, and version-controlled. If you cannot explain why a subnet exists or why a security group rule is needed, you probably do not need it.
