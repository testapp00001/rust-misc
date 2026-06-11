# Exercise 04: Design a Containerization Strategy

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Design a containerization strategy for a monolithic application that is being
broken into microservices. This exercise requires you to think about what
*should* be containerized, what *should not*, and how to handle the transition.

## Scenario

**MedRecord Systems** runs a hospital management platform. The current system
is a single monolithic Java application running on a dedicated server.
The CTO wants to move to containers, but you need to convince the
security and compliance team.

```
Current Architecture (Monolith)
===============================

┌─────────────────────────────────────────┐
│         Hospital Management App          │
│                                          │
│  ┌──────────┐ ┌──────────┐ ┌─────────┐ │
│  │ Patient  │ │ Billing  │ │ Lab     │ │
│  │ Records  │ │ Module   │ │ Results │ │
│  └──────────┘ └──────────┘ └─────────┘ │
│  ┌──────────┐ ┌──────────┐ ┌─────────┐ │
│  │ Sched-   │ │ Pharmacy │ │ Report- │ │
│  │ uling    │ │ Module   │ │ ing     │ │
│  └──────────┘ └──────────┘ └─────────┘ │
│                                          │
│  ┌──────────────────────────────────┐   │
│  │      PostgreSQL (shared DB)      │   │
│  └──────────────────────────────────┘   │
└─────────────────────────────────────────┘

Single server: Ubuntu 18.04, Java 11, 64 GB RAM
```

Proposed target architecture (microservices):

```
Target Architecture
===================

[Patient Records Service]  [Billing Service]  [Lab Results Service]
         │                        │                    │
         └────────┬───────────────┘────────────────────┘
                  │
           [API Gateway]
                  │
         ┌────────┴────────┐
         │                 │
  [Scheduling]     [Pharmacy]    [Reporting]
         │                 │           │
         └────────┬────────┘───────────┘
                  │
          [PostgreSQL]
```

## Tasks

### Part A: Service Decomposition Analysis

For each module in the monolith, determine:

1. Should it become its own container? Why or why not?
2. What are the dependencies between modules?
3. Which modules share data and how?

Create a dependency matrix:

| Module | Patient | Billing | Lab | Scheduling | Pharmacy | Reporting |
|--------|---------|---------|-----|------------|----------|-----------|
| Patient | - | | | | | |
| Billing | | - | | | | |
| Lab | | | - | | | |
| Scheduling | | | | - | | |
| Pharmacy | | | | | - | |
| Reporting | | | | | | - |

<details>
<summary>Hint</summary>

Fill in the matrix with the type of dependency:
- "R" = reads data from
- "W" = writes data to
- "S" = shares a database table with
- "A" = calls API of

If two modules are tightly coupled (many "S" entries), they probably
should not be separated into different containers initially.

</details>

### Part B: Data Strategy

The monolith uses a single PostgreSQL database with shared tables.
Design a data strategy for the containerized version:

1. Should each service get its own database? Why or why not?
2. How do you handle data that is currently shared between modules?
3. What about the Patient Records data -- can it be containerized with
   the service, or does it need external persistent storage?

<details>
<summary>Hint</summary>

In containerized architectures, databases are typically NOT containerized
for production. They run on dedicated servers or managed services (RDS,
Cloud SQL). The database contains the most critical data and needs
persistent, reliable storage that outlives any individual container.

</details>

### Part C: Compliance and Security

The hospital has strict requirements:

```
Compliance Requirements
=======================

1. Patient data must be encrypted at rest
2. All access to patient records must be logged
3. No patient data in container images
4. Containers must run as non-root users
5. Network traffic between services must be encrypted
6. Container images must be scanned for vulnerabilities
7. Secrets (DB passwords, API keys) must not be in images or environment variables
```

For each requirement, explain how you would implement it in a containerized
architecture. If a requirement is *harder* to implement with containers
than with the monolith, say so.

<details>
<summary>Hint</summary>

Consider:
- Volume mounts for encrypted storage
- Sidecar containers for logging
- Multi-stage builds to exclude data from images
- `USER` directive in Dockerfiles
- mTLS or service mesh for encryption
- Image scanning tools (Trivy, Snyk)
- Secrets management (Vault, K8s Secrets, AWS Secrets Manager)

</details>

### Part D: Migration Plan

You cannot switch from monolith to microservices overnight.
Design a phased migration plan with these constraints:

- The hospital operates 24/7 (no downtime allowed)
- Each phase must be independently deployable and rollback-able
- Phase 1 must deliver value within 2 weeks
- Budget allows 6 months for the full migration

Write your plan as a timeline with phases, milestones, and risks.

<details>
<summary>Hint</summary>

A common pattern is the **Strangler Fig** pattern:
1. Wrap the monolith in a container (lift-and-shift)
2. Add an API gateway in front
3. Extract one service at a time, starting with the least coupled
4. Route traffic through the gateway to new or old service
5. Gradually move traffic to new services
6. Decommission the monolith when all services are extracted

</details>

## Success Criteria

- [ ] Dependency matrix is filled in with specific dependency types
- [ ] Data strategy addresses shared data, persistence, and backups
- [ ] All 7 compliance requirements have concrete implementation approaches
- [ ] Migration plan has clear phases with timelines and milestones
- [ ] Risks are identified for each phase
- [ ] The plan is realistic (no "rewrite everything in 2 weeks")

## What You Should Understand After This Exercise

Containerization is not just "put everything in a Docker container."
It requires thinking about data, security, compliance, dependencies,
and migration strategy. The technical part is often the easy part --
the hard part is the organizational and operational changes.
