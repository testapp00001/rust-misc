# Module 24: When to Use Kubernetes — The Decision Framework

## The Problem: Everyone Says "Use Kubernetes" But Should You?

Kubernetes (k8s) is powerful. It's also complex. Many teams adopt it too early and drown in operational overhead.

**The question isn't "how to use Kubernetes" — it's "WHEN to use Kubernetes."**

## Decision Framework

### You Do NOT Need Kubernetes If:

```
✅ Single application (monolith or simple microservices)
✅ < 10 containers
✅ Single server can handle your traffic
✅ Small team (< 5 developers)
✅ No complex deployment requirements
✅ Simple scaling needs (1-5 replicas)
```

**Use instead:** Docker Compose + a VPS

### You MIGHT Need Kubernetes If:

```
⚠️  Multiple microservices (10+)
⚠️  Multiple teams deploying independently
⚠️  Need for auto-scaling based on metrics
⚠️  Multi-region deployment
⚠️  Complex networking requirements
⚠️  Mix of stateful and stateless workloads
```

**Consider:** Managed Kubernetes (EKS, GKE, AKS)

### You Definitely Need Kubernetes If:

```
🔥 100+ containers across many services
🔥 Multiple teams with independent deploy cycles
🔥 Need for self-healing, auto-scaling, rolling updates
🔥 Multi-cloud or hybrid-cloud strategy
🔥 Complex service mesh requirements
🔥 Compliance requirements (audit trails, RBAC)
🔥 Significant traffic variability (Black Friday scale)
```

## The Cost of Kubernetes

### Operational Overhead
```
Without Kubernetes:
├── 1 developer manages 5-10 servers
├── Deployment: SSH + docker pull + restart
└── Monitoring: simple scripts

With Kubernetes:
├── 2-3 developers manage the cluster
├── Learning curve: 2-6 months
├── Complexity: networking, storage, security, RBAC
└── Additional tools: Helm, ArgoCD, Prometheus, cert-manager
```

### When Kubernetes Hurts

1. **Small teams** — Spends more time on k8s than on product
2. **Simple apps** — Over-engineering a CRUD app
3. **Stateful workloads** — Databases in k8s are hard (but possible)
4. **Cost sensitivity** — Control plane costs money ($73/month on EKS)
5. **Ramp-up time** — 2-6 months to become productive

## Alternatives to Full Kubernetes

### Docker Compose + VPS
```
Best for: 1-3 servers, < 20 containers
Pros: Simple, cheap, easy to debug
Cons: No auto-scaling, manual deployment
```

### Docker Swarm
```
Best for: Simple multi-host orchestration
Pros: Built into Docker, easy setup
Cons: Limited ecosystem, less flexible than k8s
```

### AWS ECS / Fargate
```
Best for: AWS-native, don't want to manage k8s
Pros: Managed, integrates with AWS services
Cons: Vendor lock-in, less portable
```

### Google Cloud Run
```
Best for: Stateless containers, pay-per-request
Pros: Scale to zero, simple, cheap
Cons: Limited to stateless, vendor lock-in
```

### Railway / Render / Fly.io
```
Best for: Startups, simple deployments
Pros: Heroku-like simplicity, Docker support
Cons: Less control, can get expensive at scale
```

## The Migration Path

```
Stage 1: Single server + Docker Compose
    ↓ (when you need multi-server)
Stage 2: Multiple servers + Docker Compose + Nginx
    ↓ (when you need orchestration)
Stage 3: Docker Swarm or ECS
    ↓ (when you need advanced features)
Stage 4: Kubernetes
    ↓ (when you need multi-cloud)
Stage 5: Multi-cluster Kubernetes
```

**Don't skip stages.** Each stage teaches you what you actually need.

## Real-World Examples

### Startup (5 developers, 100k users)
```
Architecture: Single server, Docker Compose
Cost: $50/month
Deploy: SSH + docker compose up
Why: Simple, cheap, team is small
```

### Growth Stage (20 developers, 1M users)
```
Architecture: 3 servers, Docker Swarm
Cost: $500/month
Deploy: Docker Swarm stack deploy
Why: Multi-host, simple orchestration
```

### Scale (50 developers, 10M users)
```
Architecture: Kubernetes on GKE
Cost: $5,000/month
Deploy: Helm + ArgoCD
Why: Auto-scaling, multi-team, complex services
```

### Enterprise (200 developers, 100M users)
```
Architecture: Multi-cluster Kubernetes
Cost: $50,000/month
Deploy: GitOps with ArgoCD
Why: Multi-region, compliance, complex networking
```

## If You Decide to Use Kubernetes

The rest of this phase (Modules 25-38) will teach you everything:

1. **Architecture** — How k8s works internally
2. **Pods** — The smallest unit
3. **Deployments** — Managing replicas
4. **Services** — Networking
5. **Ingress** — HTTP routing
6. **ConfigMaps/Secrets** — Configuration
7. **Namespaces/RBAC** — Multi-tenancy
8. **Storage** — Persistent data
9. **StatefulSets** — Databases
10. **Helm** — Package management
11. **Networking** — Deep dive
12. **Security** — Hardening

## Hands-On Exercise

### Exercise 1: Evaluate Your Current Project
Answer these questions:
1. How many containers do you run? ___
2. How many servers? ___
3. How many developers? ___
4. What's your monthly traffic? ___
5. Do you need auto-scaling? (yes/no) ___
6. Do you deploy multiple times per day? (yes/no) ___
7. Do you have multiple teams? (yes/no) ___

Scoring:
- 0-2 "yes" to questions 3-7 → Docker Compose
- 3-4 "yes" → Consider Docker Swarm or ECS
- 5+ "yes" → Kubernetes makes sense

### Exercise 2: Try the Alternatives First

```bash
# Docker Compose (simplest)
docker compose up -d

# Docker Swarm (multi-host)
docker swarm init
docker stack deploy -c docker-compose.yml myapp

# Compare the complexity
```

## Next Steps

If you've decided Kubernetes is right for you (or you want to learn it anyway):

→ **Next module:** [25-kubernetes-architecture](../25-kubernetes-architecture/) — How Kubernetes works internally

## Checklist

- [ ] I understand when Kubernetes is appropriate
- [ ] I know the alternatives (Compose, Swarm, ECS, Cloud Run)
- [ ] I understand the operational cost of Kubernetes
- [ ] I've evaluated my current project against the decision framework
- [ ] I know the migration path from simple to Kubernetes
