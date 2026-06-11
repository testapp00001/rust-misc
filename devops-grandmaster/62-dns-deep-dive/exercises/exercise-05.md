# Exercise 05: Production DNS Architecture

**Type:** Integration
**Time:** 60 min
**Difficulty:** Hard

## Objective

Design a complete, production-grade DNS architecture for a multi-service platform that handles millions of queries per day with high availability, security, and performance requirements.

## Scenario

You are the DNS architect for a growing SaaS company with these requirements:

```
Services:
├── www.company.com          → Web application (high traffic)
├── api.company.com          → REST API (high traffic, low latency)
├── admin.company.com        → Internal admin panel (restricted access)
├── mail.company.com         → Email server
├── *.app.company.com        → Customer subdomains (wildcard, dynamic)
├── _dmarc.company.com       → Email authentication
└── _acme-challenge.company.com → Certificate automation

Constraints:
- 50M+ DNS queries per day
- Global user base (US, EU, APAC)
- 99.99% DNS availability required
- Must support DNSSEC
- Must prevent DNS amplification attacks
- Budget: moderate (not enterprise unlimited)
```

## Tasks

### Part A: Architecture Design

Design the DNS architecture. Include:
1. Provider selection strategy (single vs. multi-provider)
2. Nameserver placement and count
3. Zone structure (single zone vs. split)
4. Anycast vs. unicast considerations

Draw an ASCII diagram of your architecture showing the relationship between components.

<details>
<summary>Hint</summary>
Consider using multiple DNS providers for redundancy (primary + secondary). Anycast provides geographic distribution automatically. Split-horizon DNS can separate internal and external resolution.
</details>

### Part B: Record Design

Design the complete zone file for the domain. Include:
1. All required record types with appropriate TTLs
2. Health checks and failover configuration
3. Geographic routing where appropriate
4. Security records (SPF, DKIM, DMARC, CAA)

<details>
<summary>Hint</summary>
Use different TTL values for different record types based on change frequency. Wildcard records handle dynamic subdomains. CAA records restrict which CAs can issue certificates.
</details>

### Part C: Security Hardening

Implement DNS security measures:
1. Write the DNSSEC signing configuration
2. Write the DMARC, SPF, and DKIM records for email security
3. Configure rate limiting rules to prevent amplification attacks
4. Describe how to implement DNS-over-HTTPS (DoH) for the resolver

<details>
<summary>Hint</summary>
DNSSEC uses RRSIG, DNSKEY, and DS records. SPF defines allowed senders via TXT records. CAA records limit certificate issuance. Rate limiting can be done at the nameserver level or with a DNS proxy.
</details>

### Part D: Monitoring and Alerting

Design a monitoring strategy for the DNS infrastructure:
1. What metrics to monitor
2. What thresholds trigger alerts
3. How to detect DNS poisoning or hijacking
4. How to measure and report on the 99.99% SLA

<details>
<summary>Hint</summary>
Monitor query rates, response times, error rates, and health check status. Compare answers from multiple resolvers to detect poisoning. Use synthetic monitoring from multiple global locations.
</details>

## Success Criteria

- [ ] You can design a multi-provider, globally distributed DNS architecture
- [ ] You can write a complete, secure zone file with all record types
- [ ] You can implement DNSSEC and email authentication records
- [ ] You can design monitoring and alerting for DNS infrastructure
- [ ] You understand the trade-offs between cost, performance, and redundancy

## What You Should Understand After This Exercise

Production DNS is far more than just pointing names to IPs. It requires careful consideration of availability (multi-provider, anycast), performance (TTL tuning, geographic routing), security (DNSSEC, rate limiting, email authentication), and observability (monitoring, alerting, SLA tracking). A well-designed DNS architecture is the foundation of a reliable platform.
