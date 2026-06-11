# Exercise 01: Service Discovery Fundamentals

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

You will understand why hardcoded service addresses fail in dynamic environments and compare the three main service discovery mechanisms: DNS-based, client-side, and server-side discovery.

## Scenario

Your team manages a microservices architecture with 12 services. Currently, every service has a configuration file with hardcoded IP addresses of its dependencies:

```yaml
# payment-service/config.yaml
database:
  host: 10.0.1.50
  port: 5432
inventory_service:
  host: 10.0.1.60
  port: 8080
notification_service:
  host: 10.0.1.70
  port: 9090
```

Last week, the inventory service was redeployed with a new IP address. Three services went down because their configs still pointed to the old address. The CTO wants this fixed.

## Tasks

### Part A: Why Hardcoded Addresses Fail

List at least 5 specific failure scenarios that occur when services use hardcoded IP addresses. For each scenario, explain the impact and how a service discovery system prevents it.

<details>
<summary>Hint</summary>
Think about: deployments, scaling, failover, maintenance, and auto-scaling events.
</details>

### Part B: Discovery Mechanism Comparison

Fill in the comparison table for the three discovery mechanisms:

| Aspect | DNS-Based | Client-Side | Server-Side |
|--------|-----------|-------------|-------------|
| Who resolves the address? | | | |
| Extra infrastructure needed? | | | |
| Load balancing approach | | | |
| Health checking | | | |
| Client complexity | | | |

<details>
<summary>Hint</summary>
DNS-based: the DNS server resolves. Client-side: the calling service resolves. Server-side: a proxy/load balancer resolves.
</details>

### Part C: Choosing the Right Approach

For each scenario, recommend a discovery mechanism and explain why:

1. A simple Docker Swarm deployment with 5 services.
2. A polyglot microservices architecture (Python, Go, Java, Node.js) with 50 services.
3. A high-frequency trading system where every millisecond of latency matters.

<details>
<summary>Hint</summary>
Consider the trade-off between simplicity (DNS), flexibility (client-side), and centralization (server-side).
</details>

## Success Criteria

- [ ] You identify at least 5 distinct failure scenarios for hardcoded addresses.
- [ ] Your comparison table is accurate and complete.
- [ ] Each scenario recommendation is justified with a concrete reason.
- [ ] You can explain the trade-off between client-side and server-side discovery.

## What You Should Understand After This Exercise

Service discovery replaces static configuration with dynamic resolution. DNS-based discovery is simplest but limited. Client-side discovery gives the caller control over load balancing and failover but couples discovery logic to every service. Server-side discovery centralizes routing in a proxy but adds a hop. The right choice depends on your architecture, language diversity, and latency requirements.
