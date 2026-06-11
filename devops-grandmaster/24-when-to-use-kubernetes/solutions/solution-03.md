# Solution 03: Build Your Organization's Decision Framework

## Part A: Define the Criteria

### Criterion 1: Service Count

**Question:** How many independent services (microservices) does the application have?
**Measurement:** Numeric count
**Mapping:**
- 1-3 services: Points toward Docker Compose / PaaS
- 4-10 services: Points toward Docker Swarm / ECS
- 11+ services: Points toward Kubernetes

### Criterion 2: Team Size

**Question:** How many developers work on this application?
**Measurement:** Numeric count
**Mapping:**
- 1-4 developers: Points toward Docker Compose / PaaS (team cannot absorb K8s overhead)
- 5-15 developers: Points toward ECS / Kubernetes
- 16+ developers: Points toward Kubernetes (multiple teams need isolation)

### Criterion 3: Deployment Frequency

**Question:** How often does the team deploy to production?
**Measurement:** Deploys per week
**Mapping:**
- 1-2 deploys/week: Docker Compose manual deployment is fine
- 3-10 deploys/week: Need automated CI/CD, consider ECS or K8s
- 10+ deploys/day: Need sophisticated orchestration (Kubernetes with GitOps)

### Criterion 4: Team Independence

**Question:** How many teams deploy independently (not coordinating releases)?
**Measurement:** Number of independent deploy teams
**Mapping:**
- 1 team: Docker Compose / PaaS is sufficient
- 2-3 teams: ECS or Kubernetes with namespaces
- 4+ teams: Kubernetes with namespace isolation and RBAC

### Criterion 5: Traffic Variability

**Question:** What is the ratio of peak traffic to average traffic?
**Measurement:** Peak-to-average ratio
**Mapping:**
- 1-2x (steady): Fixed capacity works (Docker Compose)
- 3-5x (moderate spikes): Auto-scaling helpful (ECS or K8s)
- 10x+ (extreme spikes): Auto-scaling essential (Kubernetes HPA + Cluster Autoscaler)

### Criterion 6: Compliance Requirements

**Question:** Does the application need to meet regulatory compliance standards?
**Measurement:** Yes/No, and which standards
**Mapping:**
- None: Any platform works
- SOC 2: Need audit logging, access control (Kubernetes or managed platform with compliance features)
- PCI DSS, HIPAA, GDPR: Need RBAC, network policies, secrets management, audit trails (Kubernetes strongly preferred)

### Criterion 7: Monthly Infrastructure Budget

**Question:** What is the monthly budget for infrastructure?
**Measurement:** Dollar amount
**Mapping:**
- Under $200/month: Docker Compose + VPS (K8s control plane alone costs $73+)
- $200-$2,000/month: ECS, Docker Swarm, or PaaS
- $2,000+/month: Kubernetes is financially viable

### Criterion 8: Operational Maturity

**Question:** Does the team have experience with container orchestration, CI/CD pipelines, and monitoring?
**Measurement:** Self-assessment: None / Basic / Intermediate / Advanced
**Mapping:**
- None: Start with Docker Compose or PaaS, learn fundamentals first
- Basic: Docker Swarm or ECS (simpler than K8s)
- Intermediate/Advanced: Kubernetes is viable

---

## Part B: Build the Scoring System

### Scoring Table

Each criterion is scored 0-3 points based on the answer:

| Criterion | 0 Points | 1 Point | 2 Points | 3 Points |
|-----------|----------|---------|----------|----------|
| Services | 1-3 | 4-7 | 8-15 | 16+ |
| Team Size | 1-4 | 5-10 | 11-25 | 26+ |
| Deploy Freq | 1-2/week | 3-7/week | 1-3/day | 4+/day |
| Team Independence | 1 team | 2 teams | 3-4 teams | 5+ teams |
| Traffic Variability | 1-2x | 3-5x | 6-10x | 11x+ |
| Compliance | None | SOC 2 only | PCI/HIPAA | Multiple |
| Budget | <$200 | $200-$1k | $1k-$5k | $5k+ |
| Op Maturity | None | Basic | Intermediate | Advanced |

### Score Ranges

- **0-6 points: Docker Compose + VPS** -- Simple application, small team, limited budget. Kubernetes adds complexity without benefit.
- **7-12 points: Docker Swarm or ECS** -- Growing application with moderate complexity. Managed container services provide orchestration without full Kubernetes overhead.
- **13-18 points: Managed Kubernetes (EKS/GKE/AKS)** -- Complex application with multiple teams, significant traffic, or compliance requirements. Kubernetes operational overhead is justified.
- **19-24 points: Managed Kubernetes with Advanced Tooling** -- Enterprise-grade application requiring full Kubernetes ecosystem (Helm, ArgoCD, service mesh, advanced monitoring).

### Hard Stops (Override the Score)

These conditions force a specific recommendation regardless of the total score:

1. **PCI DSS or HIPAA compliance required** --> Forces Managed Kubernetes (score 13+)
   - Reason: Compliance requires RBAC, network policies, audit logging, and secrets management that Kubernetes provides natively.

2. **Budget under $150/month** --> Forces Docker Compose + VPS (score 0-6)
   - Reason: Kubernetes control plane ($73/month) plus minimum 2 worker nodes exceeds this budget.

3. **Single developer, no DevOps experience** --> Forces Docker Compose or PaaS (score 0-6)
   - Reason: One developer cannot absorb Kubernetes learning curve while maintaining the application.

4. **100+ microservices** --> Forces Managed Kubernetes (score 13+)
   - Reason: At this scale, you need service discovery, load balancing, and deployment orchestration that only Kubernetes provides.

5. **Scale-to-zero requirement** --> Forces Serverless or Kubernetes with KEDA
   - Reason: Standard Kubernetes HPA minimum is 1 replica. Scale-to-zero requires KEDA or serverless platforms.

---

## Part C: Validate with Test Cases

### Test Case 1: Startup MVP

| Criterion | Answer | Score |
|-----------|--------|-------|
| Services | 2 (API + DB) | 0 |
| Team Size | 2 | 0 |
| Deploy Freq | ~4/month | 0 |
| Team Independence | 1 team | 0 |
| Traffic Variability | Steady (100 users) | 0 |
| Compliance | None | 0 |
| Budget | $50/month | 0 |
| Op Maturity | First deployment | 0 |
| **Total** | | **0** |

**Recommendation: Docker Compose + VPS** (or PaaS)

**Reasoning:** Zero points across all criteria. A 2-person team with 2 services and $50 budget has no need for orchestration. Docker Compose on a $10-20 VPS handles 100 users easily. PaaS is also viable if the team wants zero server management.

---

### Test Case 2: Growing SaaS

| Criterion | Answer | Score |
|-----------|--------|-------|
| Services | 7 (5 microservices + 2 DBs) | 1 |
| Team Size | 8 | 1 |
| Deploy Freq | 5/week | 1 |
| Team Independence | 2 teams | 1 |
| Traffic Variability | Doubling quarterly | 1 |
| Compliance | SOC 2 (in 6 months) | 1 |
| Budget | $2,000/month | 2 |
| Op Maturity | Basic | 1 |
| **Total** | | **9** |

**Recommendation: Docker Swarm or ECS** (borderline Managed Kubernetes)

**Reasoning:** Score of 9 falls in the Docker Swarm/ECS range. However, the SOC 2 requirement (coming in 6 months) and quarterly traffic growth suggest planning for Kubernetes now rather than migrating twice. This is a borderline case where a human decision-maker should consider the growth trajectory.

---

### Test Case 3: Internal Tool

| Criterion | Answer | Score |
|-----------|--------|-------|
| Services | 2 (web app + DB) | 0 |
| Team Size | 1 | 0 |
| Deploy Freq | 1/month | 0 |
| Team Independence | 1 person | 0 |
| Traffic Variability | Steady (30 users) | 0 |
| Compliance | None | 0 |
| Budget | $100/month | 0 |
| Op Maturity | Basic | 1 |
| **Total** | | **1** |

**Recommendation: Docker Compose + VPS** (or PaaS)

**Reasoning:** Score of 1 is firmly in Docker Compose territory. A single developer with 2 services and 30 users needs the simplest possible setup. The $100 budget is tight even for Docker Compose -- a $20-50 VPS leaves room for a managed database.

---

### Test Case 4: High-Traffic Platform

| Criterion | Answer | Score |
|-----------|--------|-------|
| Services | 40 | 3 |
| Team Size | 25 | 2 |
| Deploy Freq | 30/day | 3 |
| Team Independence | 6 teams | 3 |
| Traffic Variability | 10x spikes | 2 |
| Compliance | GDPR + SOC 2 + HIPAA | 3 |
| Budget | $15,000/month | 3 |
| Op Maturity | Advanced | 3 |
| **Total** | | **22** |

**Recommendation: Managed Kubernetes with Advanced Tooling**

**Reasoning:** Score of 22 is at the top of the range. 40 microservices across 6 teams deploying 30 times/day is exactly what Kubernetes is designed for. Triple compliance requirements (GDPR, SOC 2, HIPAA) require RBAC, network policies, and audit trails. The $15,000 budget supports a full Kubernetes ecosystem with monitoring, GitOps, and service mesh.

---

### Test Case 5: Batch Processing

| Criterion | Answer | Score |
|-----------|--------|-------|
| Services | 3 | 0 |
| Team Size | 3 | 0 |
| Deploy Freq | 1/week | 0 |
| Team Independence | 1 team | 0 |
| Traffic Variability | 0x to burst (8h/day) | 2 |
| Compliance | None | 0 |
| Budget | $500/month | 1 |
| Op Maturity | Basic | 1 |
| **Total** | | **4** |

**Recommendation: Docker Compose + VPS** (with scheduled scaling)

**Reasoning:** Score of 4 is in Docker Compose territory, but the traffic pattern (8 hours active, 16 hours idle) is unusual. The framework recommends Docker Compose, but the human decision-maker should consider: can you turn off the worker servers during idle hours? If so, Docker Compose with a cron job to start/stop services saves 66% on compute costs. If the burst requirements exceed a single server, consider ECS with scheduled scaling.

---

## Part D: Edge Cases and Exceptions

### Edge Case 1: Application About to Scale

**What the framework recommends:** Docker Compose (based on current state)
**Why it might be wrong:** The application is about to receive a major customer (10x traffic) or the team is about to double in size. Current metrics understate future needs.
**Additional context:** Ask about the 6-month and 12-month roadmap. If significant growth is planned, it may be better to adopt Kubernetes now rather than migrate twice. The framework should include a "growth trajectory" adjustment.

### Edge Case 2: Team with Unique Expertise

**What the framework recommends:** Docker Compose (based on small team size)
**Why it might be wrong:** The small team happens to include 2 Kubernetes experts from a previous company. The learning curve cost that the framework assumes does not apply.
**Additional context:** The framework assumes average operational maturity. A team of 3 Kubernetes experts can operate a cluster more efficiently than a team of 10 with no experience. Consider adjusting the "Operational Maturity" criterion based on actual team skills, not team size.

### Edge Case 3: Application with Unusual Technical Requirements

**What the framework recommends:** Managed Kubernetes (based on high score)
**Why it might be wrong:** The application requires real-time processing with sub-millisecond latency, GPU workloads, or bare-metal access. Kubernetes networking overhead and container scheduling may add unacceptable latency.
**Additional context:** Some workloads (high-frequency trading, real-time gaming, ML training) have requirements that conflict with Kubernetes abstractions. The framework does not account for latency sensitivity or hardware requirements. Add a "Special Hardware/Latency" hard stop.

---

## Common Mistakes to Avoid

- **Making criteria too vague.** "Is the application complex?" is subjective. "How many independent services?" is measurable.
- **Ignoring hard stops.** A high score does not override compliance requirements. Hard stops exist for non-negotiable constraints.
- **Not validating with real cases.** A framework that gives wrong answers for obvious cases (startup MVP, enterprise platform) needs revision.
- **Forgetting the human element.** Frameworks provide structure, but human judgment is still needed for edge cases and growth trajectory.
- **Overweighting current state.** Evaluate where the application is going, not just where it is today.

## Key Takeaway

A good decision framework transforms subjective opinions into objective, repeatable decisions. It should make the right answer obvious for clear cases and provide structure for borderline cases. The scoring system quantifies the decision factors, and hard stops ensure non-negotiable requirements are always met. Most importantly, the framework should be practical -- completable in under 30 minutes by any team member.
