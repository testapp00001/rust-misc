# Solution 03: TTL Configuration

## Part A: Analyze Current TTL Problems

With all records set to 86400 seconds (24 hours), the following problems arise:

### Problem 1: Slow Failover for Critical Service

`status.saas-platform.com` has a 24-hour TTL. If the status page server
goes down, resolvers worldwide will continue caching the old IP for up to
24 hours. Users will see connection timeouts for the status page -- the
one page that should always be available to communicate incident status.

### Problem 2: Slow Deployment Propagation

`staging.saas-platform.com` changes with every deployment. A 24-hour TTL
means after changing the IP, some resolvers will keep serving the old IP
for up to 24 hours. Developers will see stale staging environments and
may report false bugs or miss real ones.

### Problem 3: Unnecessary DNS Load for Stable Records

`_dmarc.saas-platform.com` and `api.saas-platform.com` rarely change.
A 24-hour TTL is reasonable for them, but the fact that ALL records share
the same TTL suggests a lack of per-record tuning. If you lower the TTL
for staging, you should not accidentally lower it for stable records.

### Problem 4: DNS Migration Complexity

When you need to change any IP, you must wait 24 hours after lowering the
TTL before the old TTL expires from all caches worldwide. This means any
planned migration takes at least 2 days (lower TTL + wait + change IP).

## Part B: Recommend TTL Values

| Record | Current TTL | Recommended TTL | Justification |
|--------|-------------|-----------------|---------------|
| `api.saas-platform.com` | 86400 | **3600** (1 hour) | Stable infrastructure, but needs reasonable update window for planned changes |
| `www.saas-platform.com` | 86400 | **300** (5 min) | Occasional deployments need faster propagation; moderate traffic absorbs query cost |
| `staging.saas-platform.com` | 86400 | **60** (1 min) | Changes with every deploy; developers need immediate resolution updates |
| `status.saas-platform.com` | 86400 | **60** (1 min) | Critical service needs fastest possible failover; low TTL enables this |
| `_dmarc.saas-platform.com` | 86400 | **86400** (24 hours) | Policy records change rarely; aggressive caching reduces DNS load |

### Why These Values

The general principle: **TTL = acceptable delay for changes to propagate.**

- **60 seconds:** For services that need near-real-time changes (failover,
  frequent deploys). Accepts higher DNS query volume as a trade-off.
- **300 seconds:** Good balance for services that change occasionally.
  5 minutes is a reasonable propagation delay for most deployments.
- **3600 seconds:** For stable infrastructure that rarely changes.
  1 hour gives good caching efficiency while still allowing same-day changes.
- **86400 seconds:** For records that almost never change (policy records,
  DKIM keys). Maximum caching efficiency.

## Part C: TTL Migration Strategy

### Two-Phase TTL Migration

```
Phase 1: Lower the TTL
═══════════════════════
  Day 1, 09:00:
    Change:  www.saas-platform.com.  86400  →  300
    Keep IP: 203.0.113.20 (unchanged)

  Wait: 24 hours + buffer
  Reason: Resolvers that cached the record at TTL=86400 will hold it
          for up to 24 hours. You must wait for ALL caches to expire
          the old TTL before proceeding.

Phase 2: Change the IP
═══════════════════════
  Day 2, 10:00 (25 hours later):
    Change:  www.saas-platform.com.  300  IN  A  203.0.113.200
    All resolvers now have TTL=300, so they will pick up the change
    within 5 minutes.

Phase 3: Raise the TTL (optional)
══════════════════════════════════
  Day 2, 12:00 (after verifying the new IP works):
    Change:  www.saas-platform.com.  300  →  86400
    This restores caching efficiency now that the migration is stable.
```

### Why Two Phases Are Necessary

If you change both the TTL and IP simultaneously, resolvers that cached
the record at the old TTL (86400) will continue serving the old IP for
up to 24 hours, regardless of the new TTL. The TTL in the cached record
counts down independently -- the resolver does not re-query until the
cached TTL expires.

## Part D: Calculate Cache Duration

### Resolver A (queries at T=0)

```
TTL received: 300 seconds
Cache expiry: T=0 + 300 = T=300
IP served:    OLD IP (the original IP before T=120 change)
Note:         At T=60, the TTL was changed to 3600, but Resolver A
              does not know this. It still has 240 seconds left on
              its cached TTL=300 record.
```

### Resolver B (queries at T=60)

```
TTL received: 3600 seconds (the TTL was changed at T=60)
Cache expiry: T=60 + 3600 = T=3660
IP served:    OLD IP (the IP was not changed until T=120)
Note:         Resolver B received the new TTL but the old IP.
              It will cache this for 1 hour.
```

### Resolver C (queries at T=120)

```
TTL received: 3600 seconds (TTL was changed at T=60, still active)
Cache expiry: T=120 + 3600 = T=3720
IP served:    NEW IP (the IP was changed at T=120)
Note:         Resolver C receives both the new TTL and the new IP.
```

### Summary Table

| Resolver | Query Time | TTL Received | Cache Expiry | IP Served |
|----------|------------|--------------|--------------|-----------|
| A | T=0 | 300 | T=300 | Old |
| B | T=60 | 3600 | T=3660 | Old |
| C | T=120 | 3600 | T=3720 | New |

### The Critical Insight

Resolver B received the **new TTL** (3600) but the **old IP**. It will
cache the old IP for a full hour. This is why the two-phase migration
is necessary: you must ensure all resolvers are querying with the new
(lower) TTL before you change the IP.

### Common Mistakes to Avoid

- **Changing TTL and IP simultaneously.** Cached records with old TTLs
  will persist, causing inconsistent resolution for up to 24 hours.
- **Setting TTL too low for high-traffic domains.** A 1-second TTL on a
  domain receiving 1M queries/day means 1M queries/second to your
  authoritative servers. Balance propagation speed with DNS load.
- **Forgetting about intermediate resolvers.** ISPs, corporate networks,
  and OS-level caches may add their own caching layers. The actual
  propagation time can be longer than the TTL suggests.
- **Not monitoring TTL compliance.** After setting a TTL, verify with
  `dig` from multiple locations that the TTL is being honored.

## Key Takeaway

TTL is the dial that controls the trade-off between caching efficiency and
change propagation speed. Lower TTL = faster changes but more DNS queries.
The two-phase migration pattern (lower TTL, wait, change, optionally raise
TTL) is essential for zero-downtime DNS changes. Always calculate the
worst-case propagation time as: old TTL + migration buffer.
