# Exercise 03: VM vs Container -- Real Scenario Comparison

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective

Given a real-world scenario, evaluate whether virtual machines or containers
are the better fit -- and justify your reasoning with concrete numbers and
trade-offs, not just "containers are newer."

## Scenario

You are the DevOps engineer at **QuickShip**, an e-commerce startup.
The CTO asks you to design the deployment architecture for the following services:

```
QuickShip Production Stack
==========================

1. Web Frontend       -- React app served by nginx
   - Traffic: 10,000 requests/minute at peak
   - Stateless
   - Deploys: 3-5 times per week

2. API Server         -- Python/FastAPI
   - Traffic: 5,000 requests/minute at peak
   - Stateless
   - Deploys: 2-3 times per week

3. Order Service      -- Java/Spring Boot
   - Traffic: 2,000 requests/minute at peak
   - Requires Java 17 specifically
   - Deploys: Once per week

4. PostgreSQL Database -- Stateful, critical data
   - 500 GB of data
   - Requires persistent storage
   - Upgrades: Once per quarter

5. Redis Cache         -- In-memory cache
   - 16 GB memory requirement
   - Stateful (persistence enabled)
   - Upgrades: Rarely

6. Legacy Billing      -- PHP 5.6 application
   - Cannot be modified (vendor lock-in)
   - Requires Apache 2.2 with specific modules
   - Traffic: Low (internal only)
   - Runs on CentOS 6
```

You have **3 physical servers** available:

```
Server Specs (each):
- CPU: 32 cores
- RAM: 128 GB
- Disk: 2 TB NVMe SSD
- OS: Ubuntu 22.04 LTS
```

## Tasks

### Part A: Categorize Each Service

For each of the 6 services, determine whether it is a good candidate for
containers, VMs, or neither (bare metal). Fill in this table:

| Service | Container / VM / Bare Metal | Primary Reason |
|---------|---------------------------|----------------|
| Web Frontend | | |
| API Server | | |
| Order Service | | |
| PostgreSQL | | |
| Redis Cache | | |
| Legacy Billing | | |

<details>
<summary>Hint</summary>

Consider:
- Is the service stateful or stateful?
- Does it have unusual OS/kernel requirements?
- How often does it deploy?
- Is it vendor-locked to a specific OS?
- What are the resource requirements?

</details>

### Part B: Resource Planning

Calculate how many instances of each service you can fit on the 3 servers.

For containers, estimate:
- Web Frontend: ~50 MB RAM, ~0.5 CPU per instance
- API Server: ~200 MB RAM, ~1 CPU per instance
- Order Service: ~512 MB RAM, ~2 CPU per instance
- Redis Cache: 16 GB RAM, ~2 CPU
- PostgreSQL: 8 GB RAM, ~4 CPU

For VMs, estimate overhead:
- Each VM requires a guest OS: ~1 GB RAM, ~1 CPU overhead
- Boot time: ~45 seconds

<details>
<summary>Hint</summary>

Start by allocating the largest/most critical services first.
Containers sharing the host OS kernel have much lower overhead,
so you can fit more on the same hardware.

</details>

### Part C: Deployment Speed Comparison

Estimate how long it takes to deploy an update for each service using
VMs vs containers.

Assume:
- VM: Build image (5 min) + transfer (2 min) + boot (45 sec) + health check (30 sec)
- Container: Build image (1 min) + transfer (30 sec) + start (2 sec) + health check (5 sec)

Calculate total deployment time per service per approach.

### Part D: Write Your Recommendation

Write a 1-2 page recommendation document (`recommendation.md`) that includes:

1. **Architecture diagram** (ASCII art) showing which services run where
2. **Deployment approach** for each service with justification
3. **Resource allocation** across the 3 servers
4. **Trade-offs** you considered and why you made each choice
5. **Risk assessment** -- what could go wrong with your approach

<details>
<summary>Hint</summary>

Structure your recommendation around:
- Which services are containerized and why
- Which services use VMs and why
- How you handle the legacy billing system
- How you distribute load across servers
- What happens when one server fails

</details>

## Success Criteria

- [ ] All 6 services are categorized with clear reasoning
- [ ] Resource calculations are realistic and fit within server capacity
- [ ] Deployment speed comparison shows concrete numbers
- [ ] Recommendation document includes an architecture diagram
- [ ] Trade-offs are explicitly stated, not just advantages
- [ ] The legacy billing system is handled (not ignored)

## What You Should Understand After This Exercise

Containers are not always the answer. The right choice depends on the
service characteristics: statefulness, OS requirements, deployment frequency,
and isolation needs. A good architect uses the right tool for each job.
