# Exercise 03: TTL Configuration

**Type:** Independent
**Time:** 35 min
**Difficulty:** Medium

## Objective

Understand how DNS TTL (Time To Live) values affect caching, propagation speed, and system reliability, and learn to choose appropriate TTL values for different scenarios.

## Scenario

You manage DNS for a SaaS platform with these services:

```
Services and their change frequency:
├── api.saas-platform.com      → Changes rarely (stable infrastructure)
├── www.saas-platform.com      → Changes occasionally (deployments)
├── staging.saas-platform.com  → Changes frequently (every deploy)
├── status.saas-platform.com   → Critical, needs fast failover
└── _dmarc.saas-platform.com   → Policy record, changes during security updates
```

Current zone file:
```bash
api.saas-platform.com.      86400  IN  A  203.0.113.10
www.saas-platform.com.      86400  IN  A  203.0.113.20
staging.saas-platform.com.  86400  IN  A  203.0.113.30
status.saas-platform.com.   86400  IN  A  203.0.113.40
_dmarc.saas-platform.com.   86400  IN  TXT "v=DMARC1; p=reject;"
```

## Tasks

### Part A: Analyze Current TTL Problems

The current TTL for all records is 86400 seconds (24 hours). Identify at least three problems this causes for the services listed above.

<details>
<summary>Hint</summary>
Think about what happens when you need to change an IP address, when a server goes down, or when you want to roll out a deployment with a DNS change.
</details>

### Part B: Recommend TTL Values

Propose appropriate TTL values for each record. Justify each choice. Fill in the table:

| Record | Current TTL | Recommended TTL | Justification |
|--------|-------------|-----------------|---------------|
| api.saas-platform.com | 86400 | | |
| www.saas-platform.com | 86400 | | |
| staging.saas-platform.com | 86400 | | |
| status.saas-platform.com | 86400 | | |
| _dmarc.saas-platform.com | 86400 | | |

<details>
<summary>Hint</summary>
Lower TTL = faster propagation but more DNS queries (higher load/cost). Higher TTL = better caching but slower changes. Consider the trade-off for each service's requirements.
</details>

### Part C: TTL Migration Strategy

You need to change the IP for `www.saas-platform.com` from `203.0.113.20` to `203.0.113.200`. The current TTL is 86400. Describe the step-by-step process to do this with minimal downtime.

<details>
<summary>Hint</summary>
You cannot simply change the IP and TTL at the same time and expect it to work immediately. Resolvers that cached the old record at the old TTL will keep serving it. You need a two-phase approach.
</details>

### Part D: Calculate Cache Duration

Given these DNS responses at different times, calculate how long each resolver will cache the record and when they will query again.

```
Timeline:
  T=0:       TTL set to 300 seconds
  T=60:      TTL changed to 3600 (still serving old IP)
  T=120:     IP changed to new value

Resolver A queries at T=0
Resolver B queries at T=60
Resolver C queries at T=120
```

For each resolver:
- What TTL value do they receive?
- When does their cache expire?
- Which IP address do they resolve to until cache expiry?

<details>
<summary>Hint</summary>
The TTL in the response is a countdown. The resolver stores the record and counts down from the TTL value. It does not see subsequent TTL changes until it queries again.
</details>

## Success Criteria

- [ ] You can identify TTL-related problems in a DNS configuration
- [ ] You can recommend appropriate TTL values based on service requirements
- [ ] You understand the two-phase TTL migration process (lower TTL, wait, change, raise TTL)
- [ ] You can calculate cache expiry times for different resolvers

## What You Should Understand After This Exercise

TTL is the most critical tuning parameter in DNS. It controls the trade-off between caching efficiency (fewer queries, lower latency) and change propagation speed (how fast updates reach clients). The two-phase migration pattern is essential for zero-downtime DNS changes.
