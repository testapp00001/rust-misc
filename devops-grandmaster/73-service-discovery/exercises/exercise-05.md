# Exercise 05: Multi-Datacenter Discovery Architecture

**Type:** Integration
**Time:** 60 minutes
**Difficulty:** Hard

## Objective

You will design a complete multi-datacenter service discovery architecture using Consul, covering WAN federation, cross-datacenter queries, failure scenarios, and operational procedures.

## Scenario

Your company has two datacenters:

- **dc-us-east** (Virginia) -- Primary datacenter, runs the main application stack.
- **dc-eu-west** (Ireland) -- European datacenter, runs a read replica of the database and a subset of services for GDPR compliance.

You need:

1. Services in each datacenter to discover local instances first.
2. Services to fail over to the other datacenter if local instances are unavailable.
3. A unified view of all services across both datacenters.
4. GDPR compliance: European user data must not leave `dc-eu-west`.

## Tasks

### Part A: Consul Multi-Datacenter Architecture

Design the Consul cluster architecture. Specify:

1. Number of server nodes per datacenter and why.
2. How the two datacenters are connected (WAN gossip).
3. Configuration for server nodes in each datacenter.
4. How services register with the correct datacenter.

Write the Consul server configuration for both datacenters.

<details>
<summary>Hint</summary>
Each datacenter needs 3 or 5 Consul servers for Raft quorum. WAN federation uses `retry_join_wan` to connect server pools across datacenters. Services register with their local datacenter's agents.
</details>

### Part B: Cross-Datacenter Service Queries

Write the commands and/or code to:

1. Query all healthy instances of `api` in the local datacenter only.
2. Query all healthy instances of `api` across all datacenters.
3. Query only `dc-eu-west` instances of `api` from `dc-us-east`.
4. Implement a failover strategy: try `dc-eu-west` first, fall back to `dc-us-east`.

<details>
<summary>Hint</summary>
Local queries use the local agent's HTTP API. Cross-datacenter queries use `?dc=dc-eu-west` parameter or the `/<service>.service.<datacenter>.consul` DNS format. Failover requires application-level logic.
</details>

### Part C: Failure Scenarios

For each scenario, describe what happens and how the system recovers:

1. One Consul server in `dc-us-east` crashes.
2. All Consul servers in `dc-us-east` crash.
3. The WAN link between datacenters is severed.
4. A service in `dc-eu-west` registers but its health check fails.
5. A GDPR-sensitive service in `dc-eu-west` is queried from `dc-us-east`.

<details>
<summary>Hint</summary>
For scenario 2, think about quorum. For scenario 3, think about whether local discovery still works. For scenario 5, think about data access vs. service discovery metadata.
</details>

## Success Criteria

- [ ] Architecture specifies 3 or 5 servers per datacenter with quorum explanation.
- [ ] WAN federation configuration is complete with `retry_join_wan`.
- [ ] Cross-datacenter queries use correct DNS format and HTTP API parameters.
- [ ] Failover logic is implemented with local-first preference.
- [ ] All 5 failure scenarios are analyzed with recovery procedures.
- [ ] GDPR compliance is addressed in the architecture (data isolation, not just service isolation).

## What You Should Understand After This Exercise

Multi-datacenter service discovery requires WAN federation to connect independent Consul clusters. Each datacenter operates autonomously (local Raft consensus, local gossip), but WAN gossip propagates server membership for cross-datacenter queries. Local discovery continues to work even if the WAN link fails. GDPR compliance is not just about where services run -- it is about where data is stored and processed. Service discovery metadata (service name, address) is not user data, but routing decisions must ensure user data stays in the correct jurisdiction.
