# Exercise 05: Multi-Region DR Architecture

**Type:** Integration | **Time:** 45 min | **Difficulty:** Hard

## Objective

Design a complete multi-region disaster recovery architecture for a global SaaS platform, including data replication, automated failover triggers, split-brain prevention, and DNS-based traffic routing.

## Scenario

You are the principal architect at Helios Technologies, building the DR architecture for their flagship SaaS product -- a real-time collaboration platform used by 500,000 daily active users across North America, Europe, and Asia-Pacific.

Current state:
- Single-region deployment in us-east-1 (AWS)
- PostgreSQL 15 primary with 2 read replicas
- Redis cluster for caching and sessions
- S3 for file storage (already cross-region replicated)
- 50 microservices on Kubernetes (EKS)
- CloudFront for static content delivery
- Route 53 for DNS

Business requirements:
- Global RTO: 5 minutes (automated, no human in the loop)
- Global RPO: 30 seconds maximum data loss
- No single region failure should cause a global outage
- Must comply with GDPR (EU user data must remain in eu-west-1 unless user consents)
- Must comply with data residency laws in ap-southeast-1

Target architecture: 3 regions (us-east-1, eu-west-1, ap-southeast-1) with one active-primary and two warm-standby regions.

## Tasks

### Part A: Multi-Region Topology Design

Design the complete multi-region topology with data replication strategies.

1. Draw (in text/ASCII) the architecture showing all 3 regions and their components.
2. Define the data replication strategy for PostgreSQL across regions (async streaming replication, logical replication, or a managed service like Aurora Global Database).
3. Define the data replication strategy for Redis across regions.
4. Define the strategy for handling GDPR data residency (EU data stays in eu-west-1).
5. Define how Kubernetes workloads are replicated across regions (active-active, active-warm, or pilot light).

<details>
<summary>Hint</summary>
Aurora Global Database provides <1 second replication lag across regions with automated failover. For Redis, consider Global Datastore for cross-region replication. For GDPR, use separate database schemas or logical replication with row-level filtering.
</details>

### Part B: Automated Failover Triggers

Define the automated failover triggers and the decision logic for regional failover.

1. Define the health checks that determine if a region is unhealthy (not just a single node).
2. Define the threshold for triggering regional failover (e.g., 3 consecutive health check failures from 2 independent monitoring systems).
3. Write the failover decision logic in pseudocode.
4. Define who or what authorizes the failover: automated system, human approval, or hybrid.
5. Define the cooldown period to prevent flapping (failover, failback, failover).

<details>
<summary>Hint</summary>
A region is "down" is different from a node being "down." Define region health as: can the region serve at least 50% of normal traffic with <2x normal latency? Use external monitoring (not in-region) to avoid false negatives.
</details>

### Part C: Split-Brain Prevention

Design the split-brain prevention strategy for a network partition between regions.

1. Describe the split-brain scenario: what happens if us-east-1 and eu-west-1 lose connectivity but both are still serving local users.
2. Define the quorum-based decision mechanism: how many regions must agree before a failover is authorized.
3. Design the fencing mechanism: how do you ensure the old primary stops accepting writes after failover.
4. Define the data reconciliation strategy after a split-brain is resolved.
5. Write the split-brain detection script logic.

<details>
<summary>Hint</summary>
With 3 regions, a quorum of 2 prevents split-brain: if us-east-1 and eu-west-1 partition, ap-southeast-1 breaks the tie. Fencing can use DNS removal + API gateway rejection + database read-only mode. Data reconciliation after split-brain requires last-write-wins or application-level conflict resolution.
</details>

### Part D: DNS-Based Traffic Routing

Design the DNS-based traffic routing strategy using Route 53.

1. Define the Route 53 routing policy for each region (latency-based, failover, or weighted).
2. Write the Route 53 health check configuration for each region.
3. Define the TTL strategy: how quickly can DNS propagate a failover (consider caching).
4. Design the traffic cutover procedure: how do you move 100% of traffic from us-east-1 to eu-west-1.
5. Define how to handle in-flight requests during failover (graceful draining).

<details>
<summary>Hint</summary>
Route 53 failover routing with health checks is the standard approach. TTL of 60 seconds means clients may cache the old IP for up to 60 seconds after failover. Consider using Route 53 Application Recovery Controller for automated failover orchestration. For in-flight requests, the old region should return 503 with Retry-After headers.
</details>

## Success Criteria

- [ ] Architecture diagram shows all 3 regions with data replication paths.
- [ ] PostgreSQL replication strategy achieves <30 second RPO.
- [ ] Redis replication strategy handles session continuity across regions.
- [ ] GDPR data residency is enforced in the replication design.
- [ ] Failover triggers use external monitoring (not in-region).
- [ ] Quorum-based split-brain prevention uses the 3rd region as tiebreaker.
- [ ] Fencing mechanism has 3 layers (DNS, API gateway, database).
- [ ] DNS TTL is 60 seconds or less for fast failover propagation.
- [ ] Traffic cutover procedure includes graceful draining of in-flight requests.

## What You Should Understand

- Multi-region DR is fundamentally harder than single-region failover because of network partitions and data consistency challenges.
- Split-brain is the primary risk in multi-region architectures, and quorum-based consensus is the standard mitigation.
- GDPR and data residency laws constrain your replication topology -- you cannot simply replicate all data to all regions.
- DNS-based failover is simple but has a minimum cutover time equal to the TTL. Sub-minute failover requires application-layer routing (e.g., Anycast, global load balancers).
- The difference between active-active and active-warm standby is not just cost -- it affects your consistency model and conflict resolution strategy.
