# Exercise 02: Pick the Right Orchestrator for Each App

**Type:** Guided
**Time:** 35 minutes
**Difficulty:** Easy-Medium

## Objective

Apply the decision framework to real application profiles. You will evaluate specific technical characteristics, team constraints, and business requirements to recommend the right orchestration approach for each application.

## Scenario / Starting Point

You are a platform engineer at a company that has acquired 6 smaller companies. Each acquired company has its own application running on different infrastructure. The CTO wants to consolidate everything onto a single platform strategy. Your job is to evaluate each application and recommend the best deployment approach.

## The Applications

### App 1: InvoicePro

```yaml
Services:
  - Web frontend (React SPA)
  - API server (Node.js)
  - Background job processor (Node.js)
  - PostgreSQL database
  - Redis cache

Team: 3 developers
Deployment frequency: 2-3 times per week
Traffic: 10,000 requests/day, predictable
Current infrastructure: Single DigitalOcean droplet with Docker Compose
Downtime tolerance: 1 hour/month
Monthly revenue: $5,000
```

### App 2: DataStream

```yaml
Services:
  - 12 microservices (Go, Python, Java mix)
  - Apache Kafka (3 brokers)
  - Elasticsearch cluster (5 nodes)
  - PostgreSQL (primary + 2 replicas)
  - Redis cluster

Team: 15 developers across 4 squads
Deployment frequency: 20+ times/day across services
Traffic: 500,000 requests/day with 10x spikes during business hours
Current infrastructure: AWS EC2 instances, manual deployment scripts
Downtime tolerance: Zero during business hours
Monthly revenue: $500,000
```

### App 3: QuickAPI

```yaml
Services:
  - Single REST API (Python/FastAPI)
  - PostgreSQL database

Team: 1 solo developer
Deployment frequency: Once a month
Traffic: 1,000 requests/day, very steady
Current infrastructure: Heroku (migrating due to cost)
Downtime tolerance: 4 hours/month
Monthly revenue: $500 (side project)
```

### App 4: GameSync

```yaml
Services:
  - WebSocket server (Go) - handles 100k concurrent connections
  - Match-making service (Go)
  - Leaderboard service (Go)
  - Player data service (Go)
  - Redis cluster (session state)
  - MongoDB (player profiles)

Team: 8 developers
Deployment frequency: Daily
Traffic: Extreme spikes during game launches (10x-50x normal)
Current infrastructure: Bare metal servers in 2 data centers
Downtime tolerance: Zero during events
Monthly revenue: $2M
```

### App 5: InternalCRM

```yaml
Services:
  - Web application (Django)
  - PostgreSQL database
  - Celery workers (2)
  - File storage (S3-compatible)

Team: 2 developers
Deployment frequency: Weekly
Traffic: 50 internal users, predictable 9-5 pattern
Current infrastructure: On-premises VM
Downtime tolerance: 8 hours/month
Budget: Very limited ($200/month max)
```

### App 6: PayStream

```yaml
Services:
  - Payment processing API
  - Webhook handler
  - Transaction reconciliation service
  - Fraud detection service (ML model)
  - PostgreSQL (encrypted at rest)
  - Audit log service

Team: 10 developers across 3 teams
Deployment frequency: 5-10 times/day
Traffic: 100,000 transactions/day
Compliance: PCI DSS Level 1, SOC 2 Type II
Current infrastructure: AWS with manual orchestration
Downtime tolerance: Zero
Monthly revenue: $5M
```

## Tasks

### Part A: Recommendation Matrix

For each application, fill in the following recommendation:

| App | Recommended Approach | Key Deciding Factor | Estimated Monthly Cost | Migration Priority |
|-----|---------------------|---------------------|----------------------|-------------------|
| InvoicePro | ? | ? | ? | ? |
| DataStream | ? | ? | ? | ? |
| QuickAPI | ? | ? | ? | ? |
| GameSync | ? | ? | ? | ? |
| InternalCRM | ? | ? | ? | ? |
| PayStream | ? | ? | ? | ? |

**Approach options:**
- Docker Compose + VPS
- PaaS (Railway, Render, Fly.io)
- Docker Swarm
- Managed Kubernetes (EKS, GKE, AKS)
- AWS ECS/Fargate

**Migration priority:** Low, Medium, High, Critical

<details>
<summary>Hint 1</summary>

Consider these factors in order of importance:
1. Compliance requirements (if any) -- this is often non-negotiable
2. Team size and deployment frequency
3. Traffic patterns and scaling needs
4. Budget constraints
5. Current infrastructure and migration effort

</details>

### Part B: Trade-Off Analysis

For DataStream and QuickAPI, write a brief analysis (3-5 sentences each) comparing your top 2 recommended approaches. Include:

- What you gain with the more powerful option
- What you lose (simplicity, cost, learning curve)
- Why you made your final choice

<details>
<summary>Hint 2</summary>

For DataStream, the key tension is between Kubernetes (which handles 12 microservices and independent deployments well) and a simpler alternative. Ask: does the complexity of Kubernetes pay for itself given the team size and traffic patterns?

For QuickAPI, the key tension is between cost and simplicity. Heroku is expensive for what it provides, but Kubernetes is massive overkill for a single API.

</details>

### Part C: Counter-Arguments

Your CTO has suggested putting ALL 6 applications on a single Kubernetes cluster to "standardize the platform." Write a brief response (5-8 sentences) explaining why this might not be the best approach. Consider:

- Which applications would be harmed by Kubernetes?
- What is the operational cost of managing Kubernetes for simple workloads?
- Are there better ways to standardize?

<details>
<summary>Hint 3</summary>

Standardization has value, but forcing every workload onto Kubernetes creates a different kind of complexity. A hybrid approach (some on K8s, some on PaaS, some on Compose) with shared CI/CD and monitoring can be more practical than a single platform.

</details>

## Success Criteria

- [ ] All 6 applications have a recommendation with a clear key deciding factor.
- [ ] The estimated monthly costs are realistic (research cloud provider pricing if needed).
- [ ] Part B trade-off analyses compare two viable alternatives with honest pros/cons.
- [ ] Part C identifies at least 3 specific applications that would be harmed by forced Kubernetes adoption.
- [ ] Your recommendations are driven by technical requirements, not personal preferences.
- [ ] You can explain why "standardize on Kubernetes" is not always the best strategy.

## What You Should Understand After This Exercise

Each application has unique constraints that point to a specific deployment approach. Compliance requirements (like PCI DSS) often force the decision toward managed Kubernetes with RBAC and audit trails. Team size and deployment frequency determine whether the operational overhead of Kubernetes pays off. Budget constraints and simplicity should not be dismissed -- the cheapest solution that meets requirements is often the best. A portfolio of applications rarely fits a single platform.
