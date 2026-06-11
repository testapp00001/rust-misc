# Solution 01: DNS Record Types

## Part A: Identify Record Types

| # | Requirement | Record Type | Why |
|---|-------------|-------------|-----|
| 1 | Point `www.example-corp.com` to IP | **A** | Maps a hostname to an IPv4 address |
| 2 | Route email to `mail.example-corp.com` | **MX** | MX records direct email delivery and include a priority field |
| 3 | Alias `blog.example-corp.com` to provider host | **CNAME** | CNAME creates an alias pointing to another hostname |
| 4 | Prove domain ownership to Google | **TXT** | TXT records store arbitrary text data, commonly used for verification |
| 5 | Map IP back to hostname | **PTR** | PTR records perform reverse DNS lookup (IP to name) |

### Why This Matters

Each record type serves a distinct purpose. Using the wrong type leads to
broken resolution. For example, you cannot use a CNAME to point to an IP
address -- CNAME targets must be another hostname. Similarly, MX records
are the only standard way to advertise mail servers because MTAs (Mail
Transfer Agents) specifically look up MX records during email delivery.

## Part B: Write Zone File Records

```dns
; Zone: example-corp.com

; Main website (A record maps name to IPv4)
@           300  IN  A      203.0.113.10

; Mail server with priority 10 (MX record with priority)
@           300  IN  MX     10 mail.example-corp.com.

; Blog alias (CNAME points to another hostname)
blog        300  IN  CNAME  blog-hosting.provider.net.

; Google verification (TXT stores arbitrary text)
@           300  IN  TXT    "google-site-verification=abc123xyz789"

; IPv6 for main website (AAAA maps name to IPv6)
@           300  IN  AAAA   2001:db8::1
```

### Key Syntax Notes

- `@` represents the zone apex (example-corp.com itself)
- TTL of 300 seconds (5 minutes) is used here for illustration; real values
  depend on change frequency
- MX records include a priority number (lower = higher priority)
- TXT values must be quoted
- CNAME targets must be FQDNs ending with a dot

## Part C: Record Type Matching

| Scenario | Record Type | Explanation |
|----------|-------------|-------------|
| Redirect `shop.example-corp.com` to third-party host | **CNAME** | Aliases one name to another |
| Specify the email server | **MX** | Defines mail exchangers with priority |
| Support IPv6 connectivity | **AAAA** | Maps name to 128-bit IPv6 address |
| Reverse lookup from IP to hostname | **PTR** | Reverse DNS, stored in `in-addr.arpa` zone |
| Define a service endpoint with port number | **SRV** | Includes protocol, port, weight, and priority |
| Store arbitrary text data for verification | **TXT** | Holds strings for verification, SPF, DMARC, etc. |

### SRV Record Format

SRV records are unique among DNS record types because they include four
fields beyond the standard data:

```
_service._protocol.name  TTL  IN  SRV  priority  weight  port  target
```

Example:
```
_sip._tcp.example-corp.com.  300  IN  SRV  10  60  5060  sip1.example-corp.com.
```

This allows clients to discover services dynamically rather than hardcoding
ports.

### Common Mistakes to Avoid

- **Using CNAME at the zone apex.** RFC 1034 prohibits CNAME records at
  the zone apex (`@`). Use ALIAS/ANAME records or A records instead.
- **Forgetting the trailing dot.** In zone files, hostnames without a
  trailing dot are relative to the zone origin. `mail.example-corp.com`
  becomes `mail.example-corp.com.example-corp.com.` -- always use FQDNs.
- **Confusing A and AAAA.** A records are IPv4 (32-bit), AAAA records are
  IPv6 (128-bit). Most domains need both for dual-stack support.
- **Using CNAME for MX targets.** MX records should point to A records,
  not CNAMEs. While some resolvers tolerate this, it violates RFC 2181
  and can cause intermittent email delivery failures.

## Key Takeaway

DNS record types are not interchangeable. Each serves a specific protocol
function: A/AAAA for address mapping, CNAME for aliasing, MX for email
routing, TXT for metadata, SRV for service discovery, and PTR for reverse
lookup. Understanding when to use each type -- and when not to -- is
fundamental to correct DNS configuration.
