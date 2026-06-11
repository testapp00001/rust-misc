# Cheatsheet: DNS Deep Dive

## Record Types

| Type | Purpose | Example |
|------|---------|---------|
| A | IPv4 address | example.com → 1.2.3.4 |
| AAAA | IPv6 address | example.com → ::1 |
| CNAME | Alias | www.example.com → example.com |
| MX | Mail server | example.com → mail.example.com |
| TXT | Text records | SPF, DKIM, verification |
| NS | Nameserver | example.com → ns1.dns.com |
| SRV | Service | _http._tcp.example.com |
| PTR | Reverse lookup | 1.2.3.4 → example.com |

## DNS Tools
```bash
# dig — DNS lookup
dig example.com
dig example.com A
dig example.com MX
dig @8.8.8.8 example.com  # Use specific DNS server
dig +trace example.com     # Full resolution path

# nslookup
nslookup example.com
nslookup -type=MX example.com

# host
host example.com
host 1.2.3.4  # Reverse lookup
```

## TTL (Time To Live)
```
Low TTL (60s)   → Fast propagation, more DNS queries
High TTL (3600s) → Slower propagation, fewer DNS queries

Best practice:
- Use high TTL for stable records
- Lower TTL before planned changes
- Use low TTL for failover
```

## DNS Failover
```yaml
# Route53 health check
HealthCheck:
  FullyQualifiedDomainName: example.com
  Port: 443
  Type: HTTPS
  ResourcePath: /health
  FailureThreshold: 3
  RequestInterval: 30
```
