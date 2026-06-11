# Cheatsheet: Network Security

## Defense in Depth
```
Internet → WAF → Firewall → Load Balancer → Application → Database
```

## Firewall Rules (iptables)
```bash
# Allow SSH
iptables -A INPUT -p tcp --dport 22 -j ACCEPT

# Allow HTTP/HTTPS
iptables -A INPUT -p tcp --dport 80 -j ACCEPT
iptables -A INPUT -p tcp --dport 443 -j ACCEPT

# Allow established connections
iptables -A INPUT -m state --state ESTABLISHED,RELATED -j ACCEPT

# Drop everything else
iptables -A INPUT -j DROP
```

## UFW (Simpler)
```bash
ufw default deny incoming
ufw default allow outgoing
ufw allow ssh
ufw allow 80/tcp
ufw allow 443/tcp
ufw enable
ufw status verbose
```

## Cloud Security Groups
```yaml
# AWS Security Group
SecurityGroupIngress:
  - IpProtocol: tcp
    FromPort: 443
    ToPort: 443
    CidrIp: 0.0.0.0/0
  - IpProtocol: tcp
    FromPort: 22
    ToPort: 22
    CidrIp: 10.0.0.0/8  # Internal only
```

## Network Segmentation
```
DMZ (Public)     → Web servers, load balancers
Application Tier → API servers, workers
Data Tier        → Databases, caches
Management Tier  → Monitoring, logging
```
