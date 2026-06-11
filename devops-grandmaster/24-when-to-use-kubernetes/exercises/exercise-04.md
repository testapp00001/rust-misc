# Exercise 04: Kubernetes vs Alternatives -- The Real Cost

**Type:** Challenge
**Time:** 50 minutes
**Difficulty:** Medium-Hard

## Objective

Calculate the true total cost of ownership (TCO) for deploying the same application on Kubernetes vs. three alternative platforms. You will account for infrastructure costs, operational overhead, developer time, and hidden costs that are often overlooked.

## Scenario / Starting Point

Your team runs a SaaS application with the following profile:

```yaml
Application: TaskFlow (project management SaaS)
Services:
  - Web frontend (React)
  - API server (Node.js) - 3 replicas
  - Background worker (Node.js) - 2 replicas
  - PostgreSQL database
  - Redis cache
  - Scheduled job runner (1 instance)

Traffic:
  - Average: 50,000 requests/day
  - Peak: 200,000 requests/day (weekday mornings)
  - Growth: 30% per year

Team:
  - 6 developers (2 senior, 4 mid-level)
  - 1 dedicated DevOps engineer
  - No existing Kubernetes experience

Current state: Docker Compose on a single AWS EC2 instance
Monthly revenue: $50,000
Budget for infrastructure: Up to $3,000/month
```

## Tasks

### Part A: Cost Calculation

Calculate the monthly cost for each of the following deployment options. Be thorough -- include infrastructure, operational, and hidden costs.

**Option 1: Docker Compose + VPS**

Calculate costs for:
- Infrastructure (servers, load balancer, database hosting)
- SSL certificates
- Monitoring (basic)
- Backup solution
- Developer time for deployment (hours/month)
- Developer time for troubleshooting (hours/month)

<details>
<summary>Hint 1</summary>

For Docker Compose, you need at least 2 servers for redundancy behind a load balancer. A managed database (like RDS) is more reliable than self-hosted PostgreSQL on the same server. Factor in the time your DevOps engineer spends on manual deployments, log management, and server maintenance.

</details>

**Option 2: AWS ECS with Fargate**

Calculate costs for:
- Fargate tasks (vCPU and memory pricing)
- Application Load Balancer
- RDS PostgreSQL
- ElastiCache Redis
- CloudWatch monitoring
- Developer time for deployment
- Developer time for troubleshooting

<details>
<summary>Hint 2</summary>

Fargate pricing is per vCPU-hour and GB-hour. Calculate based on your service requirements. ECS with Fargate eliminates server management but costs more per compute unit than EC2. Factor in the learning curve for ECS task definitions, service configurations, and CloudWatch.

</details>

**Option 3: Managed Kubernetes (EKS)**

Calculate costs for:
- EKS control plane ($0.10/hour = $73/month)
- Worker nodes (EC2 instances or Fargate)
- Application Load Balancer (Ingress controller)
- RDS PostgreSQL
- ElastiCache Redis
- Monitoring (Prometheus + Grafana or CloudWatch)
- Helm charts / ArgoCD
- SSL certificate management (cert-manager)
- Developer time for Kubernetes learning curve (first 6 months)
- Developer time for ongoing operations

<details>
<summary>Hint 3</summary>

Kubernetes costs extend far beyond the control plane. You need worker nodes (typically 3 for HA), an Ingress controller, monitoring stack, and CI/CD pipeline changes. The biggest hidden cost is developer time: learning Kubernetes, writing manifests, debugging scheduling issues, and maintaining the cluster. Estimate 2-3 months of reduced productivity for the team to become comfortable.

</details>

**Option 4: PaaS (Railway or Render)**

Calculate costs for:
- Service hosting (per service pricing)
- Database hosting
- Redis hosting
- SSL (included)
- Monitoring (included)
- Developer time for deployment
- Developer time for troubleshooting

<details>
<summary>Hint 4</summary>

PaaS platforms include many things that are separate costs on other platforms: SSL, deployment pipelines, monitoring, logging, and automatic scaling. The per-unit cost is higher, but the total cost including operational overhead may be lower. Check current pricing for Railway or Render.

</details>

### Part B: TCO Comparison Table

Create a comprehensive comparison table:

| Cost Category | Docker Compose | ECS Fargate | Kubernetes (EKS) | PaaS |
|---------------|---------------|-------------|-------------------|------|
| Infrastructure | $ | $ | $ | $ |
| Database | $ | $ | $ | $ |
| Monitoring | $ | $ | $ | $ |
| SSL/Security | $ | $ | $ | $ |
| DevOps Engineer Time | hrs $ | hrs $ | hrs $ | hrs $ |
| Developer Time (deploy) | hrs $ | hrs $ | hrs $ | hrs $ |
| Learning Curve (amortized) | $ | $ | $ | $ |
| **Total Monthly** | **$** | **$** | **$** | **$** |
| **Total Annual** | **$** | **$** | **$** | **$** |

Use $100/hour as the loaded cost of developer time.

<details>
<summary>Hint 5</summary>

Do not forget to amortize one-time costs (like Kubernetes learning curve) over 12 months. A 3-month learning curve for 6 developers at reduced productivity is a significant cost that many analyses ignore.

</details>

### Part C: Scaling Scenario

Recalculate costs when the application scales to 5x current traffic (250,000 requests/day average, 1,000,000 at peak). Which option becomes more cost-effective at scale?

For each option, determine:
1. What changes are needed to handle 5x traffic?
2. What is the new monthly cost?
3. How much developer time is required for the scaling change?

<details>
<summary>Hint 6</summary>

Docker Compose scaling is manual and limited. ECS and Kubernetes scale more gracefully but have different cost curves. PaaS platforms may have steep cost increases at higher usage tiers. At 5x traffic, the simplicity advantage of Docker Compose may become a liability.

</details>

### Part D: Hidden Cost Analysis

Identify and quantify at least 3 hidden costs for each platform that are often overlooked. Examples include:

- Incident response time differences
- On-call burden
- Security patching overhead
- Compliance audit preparation
- Vendor lock-in migration costs
- Opportunity cost (what the team could be building instead)

<details>
<summary>Hint 7</summary>

The most expensive hidden cost is usually developer time spent on infrastructure instead of product features. A team that spends 30% of its time on Kubernetes operations is losing 30% of its product development capacity. Quantify this in terms of feature delivery speed.

</details>

## Success Criteria

- [ ] All 4 options have detailed cost breakdowns with realistic pricing (use current cloud provider pricing).
- [ ] The TCO comparison table includes at least 8 cost categories.
- [ ] Developer time is quantified in hours and dollars for each option.
- [ ] The scaling scenario (Part C) shows how costs change at 5x traffic.
- [ ] Part D identifies at least 3 hidden costs per platform.
- [ ] The analysis includes a clear recommendation with justification based on the numbers.
- [ ] You can articulate why the cheapest infrastructure option is not always the cheapest total cost.

## What You Should Understand After This Exercise

Infrastructure cost is only a fraction of the total cost of ownership. Developer time, learning curves, operational overhead, and hidden costs often dominate the equation. Kubernetes may appear expensive for small workloads (control plane cost + worker nodes + operational overhead), but it can become cost-effective at scale when amortized across many services and teams. Conversely, PaaS platforms appear expensive per unit but may be cheaper overall when you factor in the operational simplicity. The right choice depends on your specific team, scale, and growth trajectory.
