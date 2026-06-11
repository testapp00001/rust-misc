# Exercise 05: Multi-Tier Load Balancing Architecture

**Type:** Integration
**Time:** 60 min
**Difficulty:** Hard

## Objective

Design a complete multi-tier load balancing architecture for a high-availability platform that combines DNS load balancing, L4 load balancing, and L7 load balancing with different algorithms at each tier.

## Scenario

You are the infrastructure architect for a SaaS platform with the following requirements:

```
Traffic profile:
- 100,000 requests/second peak
- Global user base (US, EU, APAC)
- Mix of API calls (short), WebSocket connections (long), and file uploads (variable)

Services:
├── api.company.com       → REST API (stateless, 50ms avg)
├── ws.company.com        → WebSocket (stateful, 2hr avg connection)
├── upload.company.com    → File uploads (100ms to 5min, 1GB max)
└── admin.company.com     → Admin panel (low traffic, IP-restricted)

Constraints:
- 99.99% availability SLA
- SSL termination required
- Geographic routing preferred
- Automatic failover between regions
- Connection draining on deployment
```

## Tasks

### Part A: Architecture Design

Design a multi-tier load balancing architecture with three tiers:

```
Tier 1: DNS/Global Load Balancer
  └── Tier 2: Regional L4 Load Balancer
        └── Tier 3: Application L7 Load Balancer
```

For each tier, specify:
1. The technology to use
2. The load balancing algorithm
3. What it distributes (DNS queries, TCP connections, HTTP requests)
4. Health check mechanism

Draw an ASCII diagram of the complete architecture.

<details>
<summary>Hint</summary>

Tier 1 (DNS): Distributes users to the nearest region using geo-based DNS (Route 53, Cloudflare).
Tier 2 (L4): Distributes TCP connections within a region using least-connections (HAProxy, NLB).
Tier 3 (L7): Distributes HTTP requests to specific backends using different algorithms per path (Nginx, Envoy).

</details>

### Part B: Per-Service Algorithm Selection

For each service, specify the load balancing algorithm at each tier and justify your choice:

| Service | Tier 1 (DNS) | Tier 2 (L4) | Tier 3 (L7) | Justification |
|---------|-------------|-------------|-------------|---------------|
| api.company.com | | | | |
| ws.company.com | | | | |
| upload.company.com | | | | |
| admin.company.com | | | | |

<details>
<summary>Hint</summary>

API: Stateless, so any algorithm works. Optimize for latency (geo DNS) and even distribution (round-robin).
WebSocket: Needs connection affinity. Use IP hash or least-connections at L7.
Upload: Variable duration. Least-connections prevents overloading a server with slow uploads.
Admin: Low traffic, IP-restricted. Simple round-robin is fine.

</details>

### Part C: Failover Design

Design the failover behavior for each tier:

1. What happens when a regional L4 load balancer fails?
2. What happens when an L7 backend server fails during a WebSocket connection?
3. What happens when an entire region goes offline?
4. How do you handle a split-brain scenario where two regions think they are primary?

<details>
<summary>Hint</summary>

DNS failover is slow (TTL-based). L4 failover is fast (TCP RST or timeout). L7 failover depends on the protocol (HTTP can retry, WebSocket must reconnect). For split-brain, use a global coordination layer or accept eventual consistency.

</details>

### Part D: Deployment Strategy

Design a zero-downtime deployment strategy that:

1. Rolls out new backend versions without dropping connections
2. Handles WebSocket connections gracefully (no mid-conversation drops)
3. Supports canary deployments (5% traffic to new version)
4. Enables instant rollback if the new version has errors

Write the deployment procedure as step-by-step instructions with commands.

<details>
<summary>Hint</summary>

Use connection draining: mark the old backend as "draining" (no new connections), wait for existing connections to complete, then stop the old version. For canary, use weighted routing at L7. For WebSocket, send a "reconnect" message to clients before draining.

</details>

## Success Criteria

- [ ] You can design a three-tier load balancing architecture
- [ ] You can select appropriate algorithms for each service and tier
- [ ] You can design failover behavior at each tier
- [ ] You can create a zero-downtime deployment strategy
- [ ] You understand the trade-offs between latency, availability, and consistency

## What You Should Understand After This Exercise

Production load balancing is a multi-tier problem. DNS load balancing distributes users globally but is slow to change. L4 load balancing distributes TCP connections within a region and is fast but protocol-unaware. L7 load balancing distributes HTTP requests and can make intelligent decisions based on content. Each tier has different failover characteristics, and the combination must handle regional outages, backend failures, and zero-downtime deployments.
