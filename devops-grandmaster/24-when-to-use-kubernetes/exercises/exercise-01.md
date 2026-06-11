# Exercise 01: When Kubernetes Is (and Isn't) the Answer

**Type:** Conceptual
**Time:** 20 minutes
**Difficulty:** Easy

## Objective

Build a clear mental model for recognizing when Kubernetes is the right tool and when it is overkill. You will evaluate real-world scenarios and classify them by their ideal deployment approach.

## Scenario / Starting Point

Your company is growing fast. The CTO has asked everyone to evaluate whether Kubernetes makes sense for each of the company's projects. Before you can advise anyone, you need a solid framework for making that call.

## Tasks

### Part A: Scenario Classification

Read each scenario below. For each one, decide the best deployment approach and explain your reasoning in 1-2 sentences.

Your options are:

- **Docker Compose + VPS** (simplest)
- **Docker Swarm** (multi-host, moderate complexity)
- **Managed Kubernetes (EKS/GKE/AKS)** (full orchestration)
- **Serverless / Cloud Run** (event-driven, scale-to-zero)

| # | Scenario | Best Approach | Why? |
|---|----------|---------------|------|
| 1 | A personal blog with 5,000 visitors/month. Runs WordPress and MySQL on a single $10/month VPS. | ? | ? |
| 2 | A SaaS product with 3 services (API, worker, frontend). Team of 4 developers. Serves 50,000 requests/day. | ? | ? |
| 3 | A fintech platform with 15 microservices, 3 development teams, strict compliance requirements, and 50,000 requests/second at peak. | ? | ? |
| 4 | A data processing pipeline that runs 4 times per day for 30 minutes each time. Zero traffic between runs. | ? | ? |
| 5 | An e-commerce site expecting to grow from 100k to 10M users over the next 18 months. Currently runs on 2 servers. | ? | ? |
| 6 | An internal admin tool used by 20 employees. CRUD operations on a PostgreSQL database. | ? | ? |
| 7 | A gaming backend with 200,000 concurrent WebSocket connections. Spiky traffic patterns during events. | ? | ? |
| 8 | A machine learning training platform. Needs 8 GPUs for 6 hours, then nothing for 18 hours. | ? | ? |

<details>
<summary>Hint 1</summary>

Think about these dimensions for each scenario:
- How many containers/services?
- How many developers/teams?
- Traffic pattern: steady, spiky, or burst?
- Compliance or multi-tenancy needs?
- Budget sensitivity?

</details>

<details>
<summary>Hint 2</summary>

The simplest solution that meets requirements is usually the best. Kubernetes solves real problems, but if you do not have those problems, it just adds complexity.

</details>

### Part B: Decision Criteria Matching

Match each criterion to the deployment approach it most strongly supports. Each approach may be used more than once.

**Criteria:**

1. Team has fewer than 5 developers
2. Needs auto-scaling based on HTTP request volume
3. Must comply with SOC 2 and HIPAA with audit trails
4. Wants to minimize infrastructure cost to under $50/month
5. Has 30+ microservices across 5 teams
6. Needs to scale to zero during idle periods
7. Requires multi-region active-active deployment
8. Simple 3-tier app with predictable traffic

**Approaches:**

- Docker Compose + VPS
- Docker Swarm
- Managed Kubernetes
- Serverless / Cloud Run

| Criterion | Best Approach | Why? |
|-----------|---------------|------|
| 1 | ? | ? |
| 2 | ? | ? |
| 3 | ? | ? |
| 4 | ? | ? |
| 5 | ? | ? |
| 6 | ? | ? |
| 7 | ? | ? |
| 8 | ? | ? |

<details>
<summary>Hint 3</summary>

There is no single "right" answer for every criterion. The goal is to identify which approach *best* fits. For example, you can run a simple 3-tier app on Kubernetes, but Docker Compose is simpler if the traffic is predictable.

</details>

### Part C: Anti-Pattern Identification

For each statement below, identify whether it represents a valid reason to adopt Kubernetes or a common anti-pattern. Explain your answer.

1. "We should use Kubernetes because all the big companies use it."
2. "We have 50 microservices and each team deploys independently 10 times a day."
3. "Our developer wants to learn Kubernetes, so we should migrate our 2-service app."
4. "We need zero-downtime deployments and automatic rollback on failure."
5. "Our current Docker Compose setup works fine, but the CTO saw a Kubernetes demo at a conference."

<details>
<summary>Hint 4</summary>

The best technology choices are driven by specific, measurable requirements -- not hype, resume-building, or conference demos. Ask: "What problem does this solve that we actually have?"

</details>

## Success Criteria

- [ ] All 8 scenarios in Part A are classified with a valid approach and technical reasoning.
- [ ] Part B matches each criterion to the most appropriate approach with a clear explanation.
- [ ] Part C correctly identifies anti-patterns vs. valid reasons, with reasoning that references concrete operational needs.
- [ ] You can articulate the core principle: Kubernetes solves real problems at scale, but introduces real complexity for small teams.
- [ ] You can explain at least 3 alternatives to Kubernetes and when each is the better choice.

## What You Should Understand After This Exercise

Kubernetes is not the default answer. It is a powerful tool for specific problems: multi-team deployments, complex microservice architectures, auto-scaling requirements, and compliance needs. For simpler workloads, Docker Compose, Docker Swarm, Cloud Run, or managed PaaS platforms provide the same outcomes with less operational overhead. The decision should be driven by concrete requirements, not industry hype.
