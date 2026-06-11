# Exercise 01: DNS Record Types

**Type:** Conceptual
**Time:** 20 min
**Difficulty:** Easy

## Objective

Understand the purpose and structure of common DNS record types and identify when to use each one.

## Scenario

You are setting up DNS for a new company domain `example-corp.com`. The company has the following infrastructure:

```
example-corp.com
├── Main website at 203.0.113.10
├── Mail handled by mail.example-corp.com (203.0.113.20)
├── Blog hosted on a separate server at blog.example-corp.com (203.0.113.30)
├── API at api.example-corp.com, which points to the same IP as the main site
├── A legacy system reachable at 203.0.113.40 port 8080
└── TXT record needed for domain verification with Google
```

## Tasks

### Part A: Identify Record Types

For each requirement below, identify the correct DNS record type (A, AAAA, CNAME, MX, TXT, SRV, or PTR) and explain why.

1. Point `www.example-corp.com` to the main website IP
2. Route email to `mail.example-corp.com`
3. Make `blog.example-corp.com` an alias for `blog-hosting.provider.net`
4. Prove domain ownership to Google via a verification string
5. Map the IP `203.0.113.10` back to `www.example-corp.com`

<details>
<summary>Hint</summary>
Think about the direction of the lookup: A/AAAA records go from name to IP, PTR records go from IP to name. CNAME records point to another name, not an IP. MX records have a priority field.
</details>

### Part B: Write Zone File Records

Write the DNS zone file entries for the following requirements. Include the full record format with TTL, class, type, and value.

```
; Zone: example-corp.com
; Fill in the missing records below

; Main website
______ IN A ______

; Mail server with priority 10
______ IN MX ______

; Blog alias
______ IN CNAME ______

; Google verification
______ IN TXT ______

; IPv6 for main website
______ IN AAAA ______
```

<details>
<summary>Hint</summary>
Zone file format: `name TTL class type value`. The `@` symbol represents the zone apex (root domain). MX records include a priority number before the mail server hostname.
</details>

### Part C: Record Type Matching

Match each scenario to the correct record type. A record type may be used more than once.

| Scenario | Record Type |
|----------|-------------|
| Redirect `shop.example-corp.com` to a third-party host | |
| Specify the email server for the domain | |
| Support IPv6 connectivity | |
| Reverse lookup from IP to hostname | |
| Define a service endpoint with port number | |
| Store arbitrary text data for verification | |

<details>
<summary>Hint</summary>
SRV records are unique in that they include protocol, port, and weight in addition to the target. They look like `_service._proto.name TTL class SRV priority weight port target`.
</details>

## Success Criteria

- [ ] You can explain the difference between A, AAAA, CNAME, MX, TXT, SRV, and PTR records
- [ ] You can write syntactically correct zone file entries
- [ ] You can identify the correct record type for any given scenario
- [ ] You understand when CNAME records cannot be used (at the zone apex)

## What You Should Understand After This Exercise

DNS records are the building blocks of domain name resolution. Each record type serves a specific purpose: A and AAAA records map names to IPs, CNAME records create aliases, MX records direct email, TXT records store metadata, SRV records define services with ports, and PTR records enable reverse lookups. Choosing the wrong record type leads to broken resolution or unexpected behavior.
