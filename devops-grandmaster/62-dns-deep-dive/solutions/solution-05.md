# Solution 05: Production DNS Architecture

## Part A: Architecture Design

```
Production DNS Architecture
════════════════════════════

                    Internet Users
                   (US, EU, APAC)
                         │
            ┌────────────┼────────────┐
            │            │            │
            ▼            ▼            ▼
   ┌─────────────┐ ┌──────────┐ ┌──────────────┐
   │ Cloudflare  │ │ Route 53 │ │  Monitoring  │
   │ (Primary)   │ │(Secondary│ │  (Datadog /  │
   │ Anycast NS  │ │ Anycast) │ │  ThousandEyes│
   │ 300+ PoPs   │ │ 12 PoPs  │ │  )           │
   └──────┬──────┘ └────┬─────┘ └──────────────┘
          │             │
          │    Zone Transfer (AXFR/IXFR)
          │    or API-based sync
          │             │
          └──────┬──────┘
                 │
         ┌───────┴───────┐
         │  Zone Data    │
         │               │
         │  Primary NS   │
         │  (Cloudflare) │
         │               │
         │  - Geo routing│
         │  - Health checks
         │  - DNSSEC     │
         │  - DDoS protect│
         └───────────────┘
```

### Provider Strategy: Multi-Provider

**Primary: Cloudflare DNS**
- 300+ anycast points of presence globally
- Built-in DDoS protection
- Free tier handles 50M+ queries/day
- Native DNSSEC support
- Geographic routing and load balancing

**Secondary: AWS Route 53**
- 12 global edge locations
- Health check integration with AWS services
- Failover if Cloudflare experiences an outage
- Cost: $0.40 per million queries

**Why Multi-Provider:**
A single provider outage should not take down DNS. By using two providers
with independent infrastructure, a provider-level failure affects only
50% of queries (the other provider continues serving). Clients retry
automatically on timeout.

### Nameserver Placement

```
Cloudflare (Primary):
  ns1.cloudflare.com  → Anycast (300+ locations)
  ns2.cloudflare.com  → Anycast (300+ locations)

Route 53 (Secondary):
  ns-1.awsdns-01.com  → Anycast (12 locations)
  ns-2.awsdns-02.net  → Anycast (12 locations)
```

The parent zone (`.com`) lists all four nameservers. Resolvers choose
based on RTT (round-trip time), so users automatically get the nearest
server.

### Zone Structure

```
company.com                    → Primary zone (Cloudflare)
  ├── www                      → A record with failover
  ├── api                      → A record with geo-routing
  ├── admin                    → A record (internal, split-horizon)
  ├── mail                     → MX record
  ├── *.app                    → Wildcard CNAME for customer subdomains
  ├── _dmarc                   → TXT record
  └── _acme-challenge          → TXT record (automated)

_internal.company.com          → Internal zone (private DNS)
  ├── admin.internal           → A record (RFC 1918 address)
  ├── db.internal              → A record
  └── redis.internal           → A record
```

Split-horizon DNS separates internal and external resolution. Internal
services resolve to private IPs; external services resolve to public IPs.

## Part B: Record Design

```dns
; Zone: company.com
; Primary DNS: Cloudflare
; Secondary DNS: Route 53

; ──────────────────────────────────────────────
; SOA Record
; ──────────────────────────────────────────────
@                   86400   IN  SOA   ns1.cloudflare.com. admin.company.com. (
                    2024011501  ; serial (YYYYMMDDNN)
                    3600        ; refresh (1 hour)
                    900         ; retry (15 minutes)
                    604800      ; expire (1 week)
                    86400       ; minimum TTL (24 hours)
                )

; ──────────────────────────────────────────────
; NS Records (delegation)
; ──────────────────────────────────────────────
@                   86400   IN  NS    ns1.cloudflare.com.
@                   86400   IN  NS    ns2.cloudflare.com.
@                   86400   IN  NS    ns-1.awsdns-01.com.
@                   86400   IN  NS    ns-2.awsdns-02.net.

; ──────────────────────────────────────────────
; Web Application (with failover)
; ──────────────────────────────────────────────
www                 60      IN  A     203.0.113.10        ; Primary (US-East)
                    60      IN  A     198.51.100.10       ; Failover (EU-West)
; Health check: https://www.company.com/health
; Failover: Automatic via DNS provider health checks

; ──────────────────────────────────────────────
; API (geographic routing)
; ──────────────────────────────────────────────
api                 60      IN  A     203.0.113.20        ; US (default)
; Geo-routing rules:
;   North America → 203.0.113.20 (US-East)
;   Europe        → 198.51.100.20 (EU-West)
;   Asia Pacific  → 192.0.2.20 (AP-South)
; Health check: https://api.company.com/health

; ──────────────────────────────────────────────
; Admin Panel (internal only - split-horizon)
; ──────────────────────────────────────────────
; External view: NO RECORD (not publicly resolvable)
; Internal view:
admin               300     IN  A     10.0.1.50           ; Private IP

; ──────────────────────────────────────────────
; Email
; ──────────────────────────────────────────────
mail                300     IN  A     203.0.113.30
@                   300     IN  MX    10 mail.company.com.
@                   300     IN  MX    20 mail-backup.company.com.

; ──────────────────────────────────────────────
; Customer Subdomains (wildcard)
; ──────────────────────────────────────────────
*.app               300     IN  CNAME app-gateway.company.com.
app-gateway         300     IN  A     203.0.113.40

; ──────────────────────────────────────────────
; Email Authentication (SPF, DKIM, DMARC)
; ──────────────────────────────────────────────
@                   300     IN  TXT   "v=spf1 include:_spf.google.com include:sendgrid.net -all"

selector1._domainkey 300    IN  TXT   "v=DKIM1; k=rsa; p=MIIBIjANBgkqh...base64key..."

_dmarc              86400   IN  TXT   "v=DMARC1; p=reject; rua=mailto:dmarc@company.com; pct=100"

; ──────────────────────────────────────────────
; Certificate Authority Authorization
; ──────────────────────────────────────────────
@                   86400   IN  CAA   0 issue "letsencrypt.org"
@                   86400   IN  CAA   0 issue "digicert.com"
@                   86400   IN  CAA   0 iodef "mailto:security@company.com"

; ──────────────────────────────────────────────
; ACME Challenge (for automated certificate renewal)
; ──────────────────────────────────────────────
; _acme-challenge records are created dynamically by certbot/ACME client

; ──────────────────────────────────────────────
; Reverse DNS (for mail server IP)
; ──────────────────────────────────────────────
; Managed by IP owner (ISP/hosting provider):
; 30.113.0.203.in-addr.arpa.  300  IN  PTR  mail.company.com.
```

### TTL Rationale

| Record | TTL | Reason |
|--------|-----|--------|
| NS, SOA | 86400 | Rarely change; maximize caching |
| A (www, api) | 60 | Need fast failover; moderate query load acceptable |
| A (mail) | 300 | Occasional changes during migrations |
| MX | 300 | Mail routing changes are planned |
| CNAME (*.app) | 300 | Customer subdomains, gateway may change |
| TXT (SPF, DKIM) | 300 | May change when adding/removing email senders |
| TXT (DMARC) | 86400 | Policy changes are rare and planned |
| CAA | 86400 | CA changes are rare and require planning |

## Part C: Security Hardening

### DNSSEC Configuration

```yaml
# DNSSEC Signing Configuration
dnssec:
  enabled: true
  algorithm: ECDSAP256SHA256  # Modern, fast, secure
  ksk_rollover: 90d           # Key Signing Key rollover period
  zsk_rollover: 30d           # Zone Signing Key rollover period
  nsec3: true                 # Use NSEC3 (prevents zone enumeration)
  nsec3_iterations: 10
  nsec3_salt_length: 16

# DS Record (published in parent zone .com)
ds_record:
  key_tag: 12345
  algorithm: 13  # ECDSAP256SHA256
  digest_type: 2  # SHA-256
  digest: "a1b2c3d4e5f6..."
```

DNSSEC adds three record types to the zone:
- **RRSIG:** Signature for each record set
- **DNSKEY:** Public keys for verification
- **NSEC3:** Authenticated denial of existence (proves a name does NOT exist)

### Email Security Records

```dns
; SPF: Define authorized email senders
@    300  IN  TXT  "v=spf1 include:_spf.google.com include:sendgrid.net -all"
; -all = hard fail (reject mail from unauthorized senders)
; ~all = soft fail (mark as suspicious but deliver)

; DKIM: Cryptographic signature for email authentication
selector1._domainkey  300  IN  TXT  "v=DKIM1; k=rsa; p=MIIBIjANBgkqh..."
; selector1 = key selector (allows multiple keys)
; k=rsa = key algorithm
; p= = base64-encoded public key

; DMARC: Policy for handling SPF/DKIM failures
_dmarc  86400  IN  TXT  "v=DMARC1; p=reject; rua=mailto:dmarc-reports@company.com; pct=100"
; p=reject = reject messages that fail SPF and DKIM
; rua= = where to send aggregate reports
; pct=100 = apply policy to 100% of messages
```

### Rate Limiting Configuration

```yaml
# DNS Rate Limiting (at authoritative nameserver level)
rate_limiting:
  # Per-source-IP query rate
  per_source_ip:
    rate: 100        # queries per second
    burst: 200       # burst allowance
    action: TC       # set TC flag (truncated, force TCP)

  # Per-record query rate (prevent amplification)
  per_record:
    rate: 1000       # queries per second per record
    action: NXDOMAIN # return non-existent after limit

  # ANY query limiting (common in amplification attacks)
  any_query:
    enabled: true
    action: TC       # Force TCP for ANY queries (RFC 7873)
    rate: 10

  # Response rate limiting (RRL)
  rrl:
    window: 1        # seconds
    responses: 5     # identical responses per window
    slip: 2          # 1 in N responses sent (rest truncated)
```

### DNS-over-HTTPS (DoH)

```yaml
# DoH Configuration for Internal Resolver
doh:
  enabled: true
  endpoint: https://dns.company.com/dns-query
  tls_cert: /etc/ssl/dns.company.com.crt
  tls_key: /etc/ssl/dns.company.com.key
  upstream:
    - 1.1.1.1        # Cloudflare
    - 8.8.8.8        # Google
  cache_ttl_min: 60
  cache_ttl_max: 86400
```

DoH encrypts DNS queries, preventing ISP snooping and man-in-the-middle
attacks on DNS traffic. It uses standard HTTPS (port 443), making it
firewall-friendly and indistinguishable from normal web traffic.

## Part D: Monitoring and Alerting

### Metrics to Monitor

```yaml
# DNS Monitoring Dashboard
metrics:
  # Query metrics
  - name: dns_queries_per_second
    description: Total queries received per second
    source: authoritative_nameserver
    alert_threshold: > 10000  # Possible DDoS

  - name: dns_query_type_distribution
    description: Breakdown by record type (A, AAAA, MX, etc.)
    source: authoritative_nameserver

  # Response metrics
  - name: dns_response_time_p99
    description: 99th percentile response time
    source: synthetic_monitoring
    alert_threshold: > 100ms  # SLA: < 50ms p99

  - name: dns_response_code_distribution
    description: Breakdown by response code (NOERROR, NXDOMAIN, SERVFAIL)
    source: authoritative_nameserver
    alert_threshold: SERVFAIL > 1%  # Possible configuration error

  # Health check metrics
  - name: dns_health_check_status
    description: Health check pass/fail for each endpoint
    source: dns_provider
    alert_threshold: any failure  # Immediate alert

  # Security metrics
  - name: dns_amplification_indicator
    description: ANY query rate (amplification attack indicator)
    source: authoritative_nameserver
    alert_threshold: > 100/sec

  - name: dns_dnssec_validation_failures
    description: RRSIG validation failures
    source: monitoring_probe
    alert_threshold: any failure
```

### DNS Poisoning Detection

```python
# dns_integrity_monitor.py
import dns.resolver
import json
import time

TRUSTED_RESOLVERS = [
    "8.8.8.8",       # Google
    "1.1.1.1",       # Cloudflare
    "9.9.9.9",       # Quad9
    "208.67.222.222" # OpenDNS
]

RECORDS_TO_MONITOR = {
    "www.company.com": "203.0.113.10",
    "api.company.com": "203.0.113.20",
    "mail.company.com": "203.0.113.30",
}

def check_consistency():
    results = {}
    for domain, expected_ip in RECORDS_TO_MONITOR.items():
        resolver_ips = []
        for resolver in TRUSTED_RESOLVERS:
            try:
                r = dns.resolver.Resolver()
                r.nameservers = [resolver]
                answers = r.resolve(domain, "A")
                ip = str(answers[0])
                resolver_ips.append({"resolver": resolver, "ip": ip})
            except Exception as e:
                resolver_ips.append({"resolver": resolver, "error": str(e)})

        # Check if all resolvers agree
        unique_ips = set(r["ip"] for r in resolver_ips if "ip" in r)
        if len(unique_ips) > 1:
            alert(f"DNS INCONSISTENCY: {domain} resolved to different IPs: {resolver_ips}")
        elif len(unique_ips) == 1 and unique_ips.pop() != expected_ip:
            alert(f"DNS HIJACKING: {domain} resolved to unexpected IP")

        results[domain] = resolver_ips
    return results
```

### SLA Measurement (99.99%)

```
99.99% uptime = 52.6 minutes of downtime per year

Measurement approach:
1. Synthetic monitoring from 10+ global locations
2. Query every 10 seconds from each location
3. Record: response code, response time, returned IP
4. Calculate: uptime = successful_queries / total_queries

SLA breach conditions:
  - Any location returns SERVFAIL for > 5 minutes
  - Response time p99 > 200ms for > 15 minutes
  - Health check failure for > 3 minutes

Alerting thresholds:
  - Warning:  99.95% (26 min/year) → investigate
  - Critical: 99.90% (52 min/year) → immediate action
  - SLA breach: 99.85% (78 min/year) → incident declared
```

### Common Mistakes to Avoid

- **Single DNS provider.** A provider outage takes down all DNS. Use at
  least two providers with independent infrastructure.
- **No DNSSEC.** Without DNSSEC, DNS responses can be spoofed. Enable
  DNSSEC and monitor for validation failures.
- **Ignoring email authentication.** Missing SPF/DKIM/DMARC records allow
  email spoofing. Implement all three and set DMARC to reject.
- **No rate limiting.** Without rate limiting, your nameservers are
  vulnerable to amplification attacks. Enable RRL and ANY query limiting.
- **Monitoring only from one location.** DNS issues can be regional.
  Monitor from multiple global locations to detect geographic problems.
- **Forgetting about CAA records.** Without CAA, any CA can issue
  certificates for your domain. CAA records restrict certificate issuance
  to authorized CAs.

## Key Takeaway

Production DNS architecture requires a holistic approach: multi-provider
redundancy for availability, geographic routing for performance, DNSSEC for
integrity, email authentication for security, and comprehensive monitoring
for observability. The 99.99% SLA target demands careful attention to every
layer -- from nameserver placement to rate limiting to alerting thresholds.
DNS is often treated as "set and forget," but it is the foundation that
every other service depends on.
