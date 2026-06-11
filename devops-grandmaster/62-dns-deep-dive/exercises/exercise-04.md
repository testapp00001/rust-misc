# Exercise 04: DNS Failover Setup

**Type:** Challenge
**Time:** 45 min
**Difficulty:** Medium-Hard

## Objective

Design and configure DNS-based failover to ensure high availability when primary servers become unreachable.

## Scenario

Your company runs an e-commerce platform with the following setup:

```
Primary datacenter:   US-East  → 203.0.113.10 (primary server)
Secondary datacenter: US-West  → 198.51.100.10 (standby server)
Health check URL:     /health  returns HTTP 200 when healthy

Requirements:
- If primary goes down, traffic should shift to secondary within 5 minutes
- Normal operation should send all traffic to primary
- When primary recovers, traffic should shift back to primary
- DNS provider supports health-check-based failover
```

## Tasks

### Part A: Design the Failover Configuration

Design the DNS records needed for failover. Include:
1. The primary A record with health check
2. The secondary/failover A record
3. Appropriate TTL values for failover behavior
4. The health check configuration

Write the configuration as you would set it up in a DNS provider's console (use a YAML or JSON format of your choice).

<details>
<summary>Hint</summary>
Most DNS providers with failover support use a primary/secondary model. The primary record is active when healthy; the secondary is active only when the primary health check fails. TTL should be low enough to allow fast failover.
</details>

### Part B: Multi-Region Failover

Extend the design to support three regions with automatic failover:

```
Region 1 (Primary):  US-East  → 203.0.113.10
Region 2 (Secondary): EU-West  → 198.51.100.10
Region 3 (Tertiary):  AP-South → 192.0.2.10

Failover order: US-East → EU-West → AP-South
```

Design the configuration that:
1. Routes traffic to the nearest healthy region
2. Falls back through the priority chain if a region fails
3. Returns to the highest priority region when it recovers

<details>
<summary>Hint</summary>
You can combine geographic routing with failover priorities. Some providers use weighted records with health checks; others use explicit failover chains. Consider whether all three regions can be active or if only one should serve traffic at a time.
</details>

### Part C: Failover Testing Plan

Write a test plan to verify the failover configuration works correctly. Include:

1. Test cases for normal operation
2. Test cases for failover trigger
3. Test cases for failback
4. Test cases for cascading failure (primary and secondary both down)

<details>
<summary>Hint</summary>
Use `dig` with specific resolvers to test. You can simulate failure by blocking traffic to a server with iptables. Measure the time between failure and DNS resolution change.
</details>

### Part D: Failover Limitations

Identify three limitations of DNS-based failover and explain how to mitigate each one.

<details>
<summary>Hint</summary>
Think about TTL-based caching delays, client-side DNS caching, and the split-brain problem. Also consider that DNS failover does not provide session persistence.
</details>

## Success Criteria

- [ ] You can design a primary/secondary DNS failover configuration
- [ ] You can extend failover to multiple regions with priority chains
- [ ] You can write a comprehensive failover testing plan
- [ ] You understand the limitations of DNS-based failover and their mitigations

## What You Should Understand After This Exercise

DNS failover is a powerful tool for high availability, but it has inherent limitations due to caching and propagation delays. The key trade-off is TTL: too high and failover is slow, too low and you flood DNS servers with queries. Health checks must be carefully configured to avoid false positives (failing over when the server is actually fine) and false negatives (not failing over when the server is down).
