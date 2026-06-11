# Module 62: DNS Deep Dive — Records, Resolution, TTL, Failover

> **Previous Module**: [61 - Feature Flags](../61-feature-flags/README.md)
> **Next Module**: [63 - Load Balancing Algorithms](../63-load-balancing-algorithms/README.md)
> **Phase**: 9 — Networking

---

## The Problem

Your application is deployed, your servers are healthy, but users can't reach it. Or worse — they reach the wrong server, get stale data after a deployment, or experience outages because DNS is a single point of failure. DNS is the internet's phone book, and if you don't understand it deeply, you'll spend hours debugging issues that could be resolved in minutes.

**Real-World Scenarios:**
- You deploy a new version but users still hit the old server (TTL caching)
- Your site goes down because DNS provider has an outage
- Email delivery fails because MX records are misconfigured
- Service discovery in Kubernetes relies on DNS — when it breaks, everything breaks
- Geographic routing sends users to distant servers instead of nearby ones

---

## The Naive Way

```bash
# "Just point the domain to the IP"
# Add an A record and forget about it

# Common mistakes:
# 1. TTL set to 86400 (24 hours) — changes take a full day to propagate
# 2. No failover — single point of failure
# 3. Using A records for everything instead of appropriate record types
# 4. No monitoring of DNS health
# 5. Hardcoding IPs in application configs
```

**Why This Fails:**
- Long TTLs mean slow failover during outages
- No redundancy means one DNS failure takes down everything
- Wrong record types cause subtle bugs (e.g., using A record instead of CNAME for CDN)
- No visibility into DNS health means you discover problems from users

---

## The Right Way

### How DNS Resolution Works

```
User's Browser
    │
    ▼
Recursive Resolver (ISP/Google 8.8.8.8/Cloudflare 1.1.1.1)
    │
    ▼
Root Nameserver (.) — "Who handles .com?"
    │
    ▼
TLD Nameserver (.com) — "Who handles example.com?"
    │
    ▼
Authoritative Nameserver (example.com) — "What's the IP for www.example.com?"
    │
    ▼
Response: 93.184.216.34 (with TTL)
```

### DNS Record Types

```bash
# A Record — Maps hostname to IPv4
api.example.com.    300    IN    A    203.0.113.10

# AAAA Record — Maps hostname to IPv6
api.example.com.    300    IN    AAAA    2001:db8::1

# CNAME Record — Alias to another hostname
www.example.com.    300    IN    CNAME    example.com.
# Use case: Point www to apex, or point to CDN endpoint

# MX Record — Mail server priority
example.com.    3600    IN    MX    10 mail1.example.com.
example.com.    3600    IN    MX    20 mail2.example.com.
# Lower number = higher priority

# TXT Record — Arbitrary text (SPF, DKIM, domain verification)
example.com.    3600    IN    TXT    "v=spf1 include:_spf.google.com ~all"

# SRV Record — Service location (protocol, priority, weight, port)
_sip._tcp.example.com.    3600    IN    SRV    10 60 5060 sipserver.example.com.
# Format: priority weight port target

# NS Record — Nameserver delegation
example.com.    86400    IN    NS    ns1.example.com.
example.com.    86400    IN    NS    ns2.example.com.

# SOA Record — Start of Authority (zone metadata)
example.com.    86400    IN    SOA    ns1.example.com. admin.example.com. (
    2024010101  ; serial (YYYYMMDDNN format)
    3600        ; refresh (1 hour)
    900         ; retry (15 minutes)
    604800      ; expire (1 week)
    86400       ; minimum TTL (1 day)
)
```

### TTL and Caching Strategy

```bash
# TTL (Time To Live) — How long resolvers cache the answer
# Short TTL (60-300s): Fast changes, more queries, higher cost
# Long TTL (3600-86400s): Slower changes, fewer queries, lower cost

# Strategy: Use different TTLs for different record types
# Static services (www, api): 300-3600s
# Dynamic services (failover, canary): 60-120s
# MX, TXT records: 3600-86400s (rarely change)

# Example: Gradual TTL reduction before migration
# Day 1-5: TTL = 3600
# Day 6: TTL = 300
# Day 7: Change IP (propagates in ~5 minutes)
# Day 8: TTL = 3600 (if stable)
```

### DNS Failover

```bash
# Health-check based DNS failover (Route 53 example)
# Primary: us-east-1 (health check passes → serve this)
# Secondary: us-west-2 (failover when primary is down)

# Route 53 Health Check Configuration
aws route53 create-health-check \
  --caller-reference "primary-$(date +%s)" \
  --health-check-config '{
    "IPAddress": "203.0.113.10",
    "Port": 443,
    "Type": "HTTPS",
    "ResourcePath": "/health",
    "RequestInterval": 10,
    "FailureThreshold": 3,
    "EnableSNI": true
  }'

# Failover routing policy
# Record 1 (Primary):
#   Name: api.example.com, Type: A, Value: 203.0.113.10
#   Routing Policy: Failover, Primary
#   Health Check: primary-health-check-id

# Record 2 (Secondary):
#   Name: api.example.com, Type: A, Value: 203.0.113.20
#   Routing Policy: Failover, Secondary
```

### GeoDNS

```bash
# Route users to nearest server based on their location
# Route 53 Geolocation Routing

# US users → US servers
aws route53 change-resource-record-sets \
  --hosted-zone-id Z1234567890 \
  --change-batch '{
    "Changes": [{
      "Action": "CREATE",
      "ResourceRecordSet": {
        "Name": "api.example.com",
        "Type": "A",
        "SetIdentifier": "us-servers",
        "GeoLocation": { "ContinentCode": "NA" },
        "TTL": 300,
        "ResourceRecords": [{ "Value": "203.0.113.10" }]
      }
    }]
  }'

# EU users → EU servers
# Default → US servers (fallback)
```

### DNSSEC

```bash
# DNSSEC — DNS Security Extensions
# Prevents DNS spoofing/cache poisoning by signing records

# Enable DNSSEC on Route 53
aws route53 get-dnssec --hosted-zone-id Z1234567890

# Signs zone with ZSK (Zone Signing Key) and KSK (Key Signing Key)
# DS record at registrar points to your KSK
# Resolvers verify signatures before accepting answers

# Check DNSSEC validation
dig +dnssec example.com
# Look for: ad (authenticated data) flag
# Look for: RRSIG records in answer
```

---

## The Production Way

### Internal DNS in Kubernetes (CoreDNS)

```yaml
# CoreDNS is the default DNS server in Kubernetes
# Every pod gets DNS resolution automatically

# CoreDNS ConfigMap
apiVersion: v1
kind: ConfigMap
metadata:
  name: coredns
  namespace: kube-system
data:
  Corefile: |
    .:53 {
        errors
        health {
            lameduck 5s
        }
        ready
        kubernetes cluster.local in-addr.arpa ip6.arpa {
            pods insecure
            fallthrough in-addr.arpa ip6.arpa
            ttl 30
        }
        prometheus :9153
        forward . /etc/resolv.conf {
            max_concurrent 1000
        }
        cache 30
        loop
        reload
        loadbalance
    }

# Service discovery via DNS
# Service: my-service.namespace.svc.cluster.local
# Pod: 10-244-0-5.namespace.pod.cluster.local

# Example: Application connects to database
# DATABASE_URL=postgres://db-service.production.svc.cluster.local:5432/mydb
```

### Service Discovery via DNS

```yaml
# Headless Service — Returns individual pod IPs (for StatefulSets)
apiVersion: v1
kind: Service
metadata:
  name: postgres
  namespace: production
spec:
  clusterIP: None  # Headless!
  selector:
    app: postgres
  ports:
    - port: 5432

# DNS entries created:
# postgres.production.svc.cluster.local → [pod1-ip, pod2-ip, pod3-ip]
# postgres-0.postgres.production.svc.cluster.local → pod1-ip
# postgres-1.postgres.production.svc.cluster.local → pod2-ip
# postgres-2.postgres.production.svc.cluster.local → pod3-ip

# StatefulSet pods get stable DNS names
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: postgres
spec:
  serviceName: "postgres"
  replicas: 3
  selector:
    matchLabels:
      app: postgres
  template:
    spec:
      containers:
        - name: postgres
          image: postgres:15
```

### DNS Monitoring and Alerting

```bash
# Monitor CoreDNS metrics
# Key metrics:
# coredns_dns_requests_total — Total DNS queries
# coredns_dns_responses_total — Total responses (by RCODE)
# coredns_dns_request_duration_seconds — Query latency
# coredns_cache_hits_total — Cache hit rate

# Prometheus alert for DNS failures
# Alert: DNS failure rate > 1%
- alert: HighDNSFailureRate
  expr: |
    sum(rate(coredns_dns_responses_total{rcode="SERVFAIL"}[5m]))
    / sum(rate(coredns_dns_responses_total[5m])) > 0.01
  for: 5m
  labels:
    severity: critical
  annotations:
    summary: "DNS failure rate above 1%"

# Grafana dashboard queries
# Query rate: sum(rate(coredns_dns_requests_total[5m]))
# Latency p99: histogram_quantile(0.99, rate(coredns_dns_request_duration_seconds_bucket[5m]))
# Cache hit ratio: sum(rate(coredns_cache_hits_total[5m])) / sum(rate(coredns_dns_requests_total[5m]))
```

### DNS Troubleshooting

```bash
# dig — The Swiss army knife of DNS debugging

# Basic query
dig example.com

# Query specific record type
dig example.com MX
dig example.com TXT
dig example.com NS

# Query specific DNS server
dig @8.8.8.8 example.com
dig @1.1.1.1 example.com

# Trace full resolution path
dig +trace example.com
# Shows: Root → TLD → Authoritative

# Check TTL remaining
dig +nocmd +noall +answer example.com
# Answer: example.com. 245 IN A 93.184.216.34
# TTL = 245 seconds remaining

# Reverse DNS lookup
dig -x 93.184.216.34

# Check DNSSEC
dig +dnssec example.com

# nslookup — Simpler alternative
nslookup example.com
nslookup -type=MX example.com 8.8.8.8

# host — Quick lookup
host example.com
host -t MX example.com
```

### DNS Performance Optimization

```bash
# 1. Use DNS pre-fetching in browsers
# <link rel="dns-prefetch" href="//cdn.example.com">

# 2. Minimize DNS lookups per page
# Audit: WebPageTest, Lighthouse
# Combine services under fewer domains

# 3. Use appropriate TTLs
# Static content: 3600s
# API endpoints: 300s
# Failover services: 60s

# 4. Enable DNS caching at application level
# Node.js: Use dns-cache module
# Python: Use dnspython with caching

# 5. Use DNS over HTTPS (DoH) or DNS over TLS (DoT)
# Prevents ISP DNS manipulation
# Cloudflare: https://cloudflare-dns.com/dns-query
# Google: https://dns.google/dns-query
```

---

## Hands-On Lab

### Lab: Configure DNS Records and Failover

**Objective:** Set up a complete DNS configuration with multiple record types and health-check-based failover.

**Prerequisites:**
- A domain name (or use a free subdomain from a service like freenom)
- Access to a DNS provider (Cloudflare free tier works)
- Two servers (or use Docker containers)

#### Step 1: Set Up Authoritative DNS

```bash
# Using Cloudflare (free tier)
# 1. Sign up at cloudflare.com
# 2. Add your domain
# 3. Update nameservers at your registrar to Cloudflare's

# Verify nameserver change
dig NS yourdomain.com
# Should show Cloudflare nameservers
```

#### Step 2: Create DNS Records

```bash
# Via Cloudflare API
# Get your API key from dashboard

ZONE_ID="your-zone-id"
API_KEY="your-api-key"
EMAIL="your-email@example.com"

# Create A record for api server 1
curl -X POST "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records" \
  -H "X-Auth-Email: $EMAIL" \
  -H "X-Auth-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  --data '{
    "type": "A",
    "name": "api",
    "content": "203.0.113.10",
    "ttl": 300,
    "proxied": false
  }'

# Create CNAME for www
curl -X POST "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records" \
  -H "X-Auth-Email: $EMAIL" \
  -H "X-Auth-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  --data '{
    "type": "CNAME",
    "name": "www",
    "content": "yourdomain.com",
    "ttl": 3600,
    "proxied": true
  }'

# Create MX records for email
curl -X POST "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records" \
  -H "X-Auth-Email: $EMAIL" \
  -H "X-Auth-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  --data '{
    "type": "MX",
    "name": "yourdomain.com",
    "content": "mail.yourdomain.com",
    "priority": 10,
    "ttl": 3600
  }'

# Create TXT record for SPF
curl -X POST "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records" \
  -H "X-Auth-Email: $EMAIL" \
  -H "X-Auth-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  --data '{
    "type": "TXT",
    "name": "yourdomain.com",
    "content": "v=spf1 include:_spf.google.com ~all",
    "ttl": 3600
  }'
```

#### Step 3: Configure Health Checks

```bash
# Create health check for primary server
curl -X POST "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/healthchecks" \
  -H "X-Auth-Email: $EMAIL" \
  -H "X-Auth-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  --data '{
    "name": "api-primary-health",
    "description": "Health check for primary API server",
    "address": "203.0.113.10",
    "type": "HTTPS",
    "port": 443,
    "path": "/health",
    "interval": 60,
    "retries": 3,
    "timeout": 5,
    "method": "GET",
    "expected_codes": ["200"],
    "follow_redirects": true
  }'
```

#### Step 4: Test Failover

```bash
# Simulate primary server failure
# On primary server (203.0.113.10):
sudo iptables -A INPUT -p tcp --dport 443 -j DROP

# Monitor health check status
watch -n 5 'dig +short api.yourdomain.com'

# Expected behavior:
# 1. Health check fails after 3 consecutive failures (3 minutes)
# 2. DNS response changes to secondary IP
# 3. Users automatically routed to secondary server

# Verify from different locations
dig @8.8.8.8 api.yourdomain.com
dig @1.1.1.1 api.yourdomain.com
dig @8.8.4.4 api.yourdomain.com

# Restore primary server
sudo iptables -D INPUT -p tcp --dport 443 -j DROP

# Verify failback
watch -n 5 'dig +short api.yourdomain.com'
```

#### Step 5: Test TTL Behavior

```bash
# Query and observe TTL countdown
for i in {1..10}; do
  echo "--- Query $i ---"
  dig +nocmd +noall +answer api.yourdomain.com
  sleep 30
done

# TTL should decrease from 300 to ~0, then reset to 300
# This demonstrates caching behavior

# Test TTL impact on change propagation
# 1. Note current TTL (e.g., 300s)
# 2. Change the A record IP
# 3. Query immediately — old IP (cached)
# 4. Wait TTL duration — new IP
```

#### Step 6: DNS Performance Testing

```bash
# Install dnsperf
sudo apt-get install dnsperf

# Create query file
cat > queries.txt << 'EOF'
api.yourdomain.com A
www.yourdomain.com A
yourdomain.com MX
yourdomain.com TXT
EOF

# Run performance test
dnsperf -s 1.1.1.1 -d queries.txt -l 30 -c 10
# -s: DNS server
# -d: Query file
# -l: Duration in seconds
# -c: Concurrent clients

# Compare different DNS providers
dnsperf -s 8.8.8.8 -d queries.txt -l 30 -c 10
dnsperf -s 1.1.1.1 -d queries.txt -l 30 -c 10
dnsperf -s 9.9.9.9 -d queries.txt -l 30 -c 10
```

---

## Limitation → Next Topic

**What You Learned:**
- DNS resolution hierarchy (recursive → root → TLD → authoritative)
- Record types and their use cases (A, AAAA, CNAME, MX, TXT, SRV)
- TTL strategy for fast changes vs. low query volume
- DNS failover with health checks
- GeoDNS for geographic routing
- DNSSEC for security
- Internal DNS in Kubernetes (CoreDNS)
- Service discovery via DNS

**What's Still Hard:**
You understand how DNS routes users to your servers, but what happens when all those users arrive at once? DNS gets them to the right server, but it doesn't distribute load intelligently. If you have 10 servers and one is slow, DNS keeps sending traffic to it. If one server has 100 active connections and another has 10, DNS doesn't care — it just round-robins.

**Next Module:** [63 - Load Balancing Algorithms](../63-load-balancing-algorithms/README.md) — Learn how to distribute traffic across servers using intelligent algorithms that consider server health, connection count, and request characteristics.

---

## Quick Reference

### Essential Commands

```bash
# DNS lookup
dig example.com                    # Basic query
dig example.com MX                 # Specific record type
dig @8.8.8.8 example.com          # Query specific server
dig +trace example.com             # Full resolution path
dig +short example.com             # Just the answer

# nslookup (simpler)
nslookup example.com
nslookup -type=MX example.com

# Reverse lookup
dig -x 93.184.216.34

# Flush local DNS cache
sudo systemd-resolve --flush-caches  # Linux
sudo dscacheutil -flushcache          # macOS
ipconfig /flushdns                     # Windows
```

### Common DNS Issues

| Issue | Symptom | Solution |
|-------|---------|----------|
| TTL too high | Changes take hours | Lower TTL before changes |
| TTL too low | High DNS query costs | Increase TTL for stable records |
| Missing MX | Email delivery fails | Add MX records |
| Wrong CNAME | Site unreachable | Verify CNAME target exists |
| DNSSEC broken | SERVFAIL responses | Check DS record at registrar |

### DNS Provider Comparison

| Provider | Failover | GeoDNS | DNSSEC | API | Price |
|----------|----------|--------|--------|-----|-------|
| Cloudflare | Yes | Yes | Yes | Yes | Free tier |
| Route 53 | Yes | Yes | Yes | Yes | $0.50/zone |
| Google Cloud DNS | Yes | Yes | Yes | Yes | $0.20/zone |
| DigitalOcean | No | No | No | Yes | Free |

---

**Remember:** DNS is the foundation of network communication. Misconfigure it, and nothing else matters — your load balancers, CDNs, and application servers are all unreachable. Master DNS first.
