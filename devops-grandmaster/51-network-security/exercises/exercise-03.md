# Exercise 03: Design a Production Network Architecture

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective
Given a realistic production scenario with web servers, application servers, databases, monitoring infrastructure, and a CI/CD pipeline, design the complete VPC/subnet/security group architecture on a cloud provider (AWS-style). You will practice translating business requirements into network topology and security controls.

## The Scenario

You are the platform engineer for "DataVault," a SaaS application with the following requirements:

**Business Requirements:**
- Public-facing web application served over HTTPS
- REST API backend processing sensitive financial data
- PostgreSQL database with automated backups
- Redis cache for session management
- Internal monitoring with Prometheus/Grafana
- CI/CD pipeline deploying containers to the cluster
- VPN access for remote engineers
- Compliance requirement: database must never be directly internet-accessible

**Scale:**
- 2 web servers behind a load balancer
- 3 application servers in a container cluster
- 1 primary database + 1 read replica
- 1 Redis instance
- 1 monitoring server
- 1 CI/CD server
- Up to 10 concurrent VPN users

## Tasks

### Part A: Design the VPC and Subnets
Design the VPC CIDR allocation and subnet layout. For each subnet, specify:
- Name, CIDR range, availability zone
- Whether it is public or private
- Which resources live in it

Draw an ASCII diagram showing the complete subnet topology across at least 2 availability zones.

<details>
<summary>Hint</summary>
Use a /16 VPC (e.g., `10.0.0.0/16`) and divide it into /20 or /24 subnets. Public subnets need an Internet Gateway route. Private subnets need a NAT Gateway for outbound internet access. Use at least 2 AZs for high availability. Common tiers: public (web/ALB), private-app (containers), private-data (databases), private-management (monitoring/CI/CD).
</details>

### Part B: Define Security Groups
Create security groups for each resource type. For each security group, specify:
- Name and description
- Inbound rules (source, port, protocol)
- Outbound rules (destination, port, protocol)

<details>
<summary>Hint</summary>
Start with the most restrictive possible. Database security group should only accept connections from the app server security group on port 5432. Web servers accept 443 from the ALB security group. Use security group references (sg-xxxxx) rather than CIDR ranges where possible for dynamic scaling.
</details>

### Part C: Design VPN and Remote Access
Design the VPN solution for remote engineer access. Specify:
- VPN technology choice (OpenVPN, WireGuard, or AWS Client VPN)
- Authentication method
- What networks the VPN provides access to
- DNS resolution through the VPN
- Split tunnel vs full tunnel trade-offs

<details>
<summary>Hint</summary>
Engineers need SSH access to servers and access to internal dashboards (Grafana). They do not need all their traffic routed through the VPN. Consider WireGuard for simplicity and performance. Use certificate-based or SSO-integrated authentication. Provide access only to the management subnet, not the data tier directly.
</details>

### Part D: Plan Network Monitoring and Logging
Design the network monitoring and logging architecture. Address:
- VPC Flow Logs configuration
- Security group rule hit logging
- Alert thresholds for anomalous traffic patterns
- Log retention and analysis pipeline

<details>
<summary>Hint</summary>
VPC Flow Logs capture metadata about every network flow (source, destination, port, bytes, accept/reject). Send them to CloudWatch or S3. Set alerts for: rejected connections from unexpected sources, data exfiltration patterns (large outbound transfers), and port scanning activity. Retain logs for at least 90 days for compliance.
</details>

### Part E: Create the Infrastructure-as-Code Skeleton
Write a Terraform (or CloudFormation) skeleton that declares your VPC, subnets, route tables, security groups, and VPN endpoint. You do not need to implement every detail, but the structure and resource relationships must be correct.

```hcl
# Write your Terraform skeleton here
```

<details>
<summary>Hint</summary>
Start with the VPC resource, then subnets, then route tables and associations, then security groups, then the VPN endpoint. Use variables for CIDR ranges and environment names. Use data sources for availability zones. Reference security group IDs in rules rather than hardcoding CIDRs.
</details>

## Success Criteria
- [ ] VPC uses a /16 CIDR with at least 6 subnets across 2 AZs
- [ ] Database subnet has no route to the Internet Gateway
- [ ] Security groups follow least-privilege (no 0.0.0.0/0 on non-public ports)
- [ ] VPN solution provides access only to management networks
- [ ] Terraform skeleton compiles without errors and reflects the design

## What You Should Understand After This Exercise
Cloud network design is about translating security requirements into network topology. Subnets create trust boundaries. Security groups act as stateful firewalls at the instance level. VPNs provide controlled remote access without exposing management interfaces to the public internet. Every design decision should be traceable to a specific requirement or threat. If you cannot explain why a subnet exists or why a security group rule is needed, you probably do not need it.
