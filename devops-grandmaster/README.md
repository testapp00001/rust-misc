# DevOps Grandmaster — From Zero to Legendary

> The ultimate step-by-step DevOps mastery curriculum. Every lesson builds on the last.
> You start simple, hit limitations, and the next topic shows you how to solve them.

## Philosophy

```
Level 0: "It works on my machine"
Level 1: "It works in a container"
Level 2: "It works in production"
Level 3: "It works at scale"
Level 4: "It works when things break"
Level 5: "It works no matter what"
Level 6: "I can make anything work"
```

Each module follows this pattern:
1. **The Problem** — What are we trying to solve?
2. **The Naive Way** — The simplest approach, and why it fails
3. **The Right Way** — The proper solution
4. **The Production Way** — How real systems handle it
5. **Hands-On Lab** — Working code you can run right now
6. **Limitation** — What's the next problem this creates?
7. **Next Topic** — Where to go from here

## Prerequisites

- A computer (Linux, Mac, or Windows with WSL2)
- Basic terminal/command-line skills
- Willingness to break things and fix them

## Curriculum Map

### Phase 1: Containerization Foundations (Modules 01–10)
*From "it works on my machine" to "it works in a container"*

| # | Module | What You Learn |
|---|--------|----------------|
| 01 | Why Containers | The problem containers solve |
| 02 | Your First Container | Docker basics — run, stop, inspect |
| 03 | Writing Dockerfiles | Building images from scratch |
| 04 | Multi-Stage Builds | Shrinking images from 1GB to 10MB |
| 05 | Image Caching | Why rebuilds take forever and how to fix it |
| 06 | Image Registries | Sharing and storing images |
| 07 | Docker Compose | Multi-container applications |
| 08 | Container Networking | How containers talk to each other |
| 09 | Volumes & Data | Persisting data beyond container lifecycle |
| 10 | Container Security | Running containers safely |

### Phase 2: Production Containers (Modules 11–17)
*From "it works on my laptop" to "it works in production"*

| # | Module | What You Learn |
|---|--------|----------------|
| 11 | Logging Strategy | stdout, stderr, log drivers, structured logging |
| 12 | Health Checks | Knowing if your app is actually alive |
| 13 | Environment Variables | Config without rebuilding images |
| 14 | Secrets Management | Handling passwords, API keys, certificates |
| 15 | Container Monitoring | CPU, memory, network, disk metrics |
| 16 | CI/CD Pipelines | Automated build, test, deploy |
| 17 | Image Versioning | Tag strategies, rollback, pinning |

### Phase 3: Scaling Basics (Modules 18–23)
*From "one server" to "many servers"*

| # | Module | What You Learn |
|---|--------|----------------|
| 18 | Horizontal vs Vertical Scaling | When to scale up vs scale out |
| 19 | Load Balancing | Distributing traffic across instances |
| 20 | Reverse Proxy | Nginx, Traefik, and the gateway pattern |
| 21 | Session Management | Sticky sessions, shared state, stateless design |
| 22 | Uptime Commitment | SLA, SLO, SLI — what 99.9% actually means |
| 23 | Graceful Shutdown | Handling in-flight requests during restart |

### Phase 4: Deep Kubernetes (Modules 24–38)
*From "what is k8s" to "I am the cluster"*

| # | Module | What You Learn |
|---|--------|----------------|
| 24 | When to Use Kubernetes | Decision framework — do you actually need it? |
| 25 | Kubernetes Architecture | Control plane, nodes, etcd, API server |
| 26 | Pods & Containers | The smallest deployable unit |
| 27 | Deployments & ReplicaSets | Declarative state management |
| 28 | Services & Networking | ClusterIP, NodePort, LoadBalancer |
| 29 | Ingress Controllers | HTTP routing, TLS termination |
| 30 | ConfigMaps & Secrets | Configuration in Kubernetes |
| 31 | Namespaces & RBAC | Multi-tenancy and access control |
| 32 | Persistent Volumes | Stateful workloads in k8s |
| 33 | StatefulSets | Databases and stateful services |
| 34 | DaemonSets & Jobs | Background tasks and node-level agents |
| 35 | Helm Charts | Package management for Kubernetes |
| 36 | Kustomize | Configuration management without templating |
| 37 | Kubernetes Networking Deep Dive | CNI, NetworkPolicies, DNS |
| 38 | Kubernetes Security | PodSecurityStandards, OPA, image scanning |

### Phase 5: Observability (Modules 39–44)
*From "I think it's working" to "I know exactly what's happening"*

| # | Module | What You Learn |
|---|--------|----------------|
| 39 | Metrics Collection | Prometheus, Grafana, time-series data |
| 40 | Distributed Tracing | Jaeger, OpenTelemetry, trace context propagation |
| 41 | Centralized Logging | ELK stack, Loki, log aggregation |
| 42 | Alerting Systems | PagerDuty, AlertManager, on-call rotations |
| 43 | SLO Monitoring | Error budgets, burn rate, alerting on SLOs |
| 44 | Dashboard Design | What to monitor, how to visualize it |

### Phase 6: Database Operations (Modules 45–50)
*From "SELECT *" to "I run databases that never go down"*

| # | Module | What You Learn |
|---|--------|----------------|
| 45 | Database Scaling | Read replicas, connection pooling, query optimization |
| 46 | Replication | Master-slave, multi-master, conflict resolution |
| 47 | Backup & Restore | Point-in-time recovery, automated backups |
| 48 | Disaster Recovery | RPO, RTO, failover procedures |
| 49 | High Availability | Active-passive, active-active, consensus |
| 50 | Database Migration | Schema changes without downtime |

### Phase 7: Security (Modules 51–56)
*From "it works" to "it works and nobody can break in"*

| # | Module | What You Learn |
|---|--------|----------------|
| 51 | Network Security | Firewalls, VPNs, network segmentation |
| 52 | DDoS Protection | Rate limiting, traffic analysis, mitigation |
| 53 | WAF Rules | SQL injection, XSS, CSRF protection |
| 54 | Secrets Rotation | Automated credential cycling |
| 55 | Vulnerability Scanning | Container image scanning, dependency audits |
| 56 | Zero Trust Architecture | Never trust, always verify |

### Phase 8: CI/CD Mastery (Modules 57–61)
*From "manual deploy" to "ship 100 times a day"*

| # | Module | What You Learn |
|---|--------|----------------|
| 57 | Pipeline Design | Build, test, stage, deploy patterns |
| 58 | Blue-Green Deployment | Zero-downtime releases |
| 59 | Canary Deployment | Gradual rollout with traffic splitting |
| 60 | GitOps | Declarative infrastructure with ArgoCD/Flux |
| 61 | Feature Flags | Decoupling deploy from release |

### Phase 9: Networking (Modules 62–66)
*From "ping works" to "I design the network"*

| # | Module | What You Learn |
|---|--------|----------------|
| 62 | DNS Deep Dive | Records, resolution, TTL, failover |
| 63 | Load Balancing Algorithms | Round-robin, least-connections, IP hash |
| 64 | CDN & Edge | Caching, geographic distribution |
| 65 | Service Mesh | Istio, Linkerd, mTLS, traffic management |
| 66 | Network Troubleshooting | tcpdump, traceroute, packet analysis |

### Phase 10: Extreme Scaling (Modules 67–71)
*From "it handles normal traffic" to "it handles Black Friday"*

| # | Module | What You Learn |
|---|--------|----------------|
| 67 | Auto-Scaling | HPA, VPA, cluster autoscaler |
| 68 | Traffic Surge Handling | Pre-warming, queue-based architecture |
| 69 | Capacity Planning | Forecasting, load testing, bottleneck analysis |
| 70 | Caching Strategies | Redis, Memcached, CDN, cache invalidation |
| 71 | Performance Tuning | OS-level, kernel, network stack optimization |

### Phase 11: Cluster & Multi-Host (Modules 72–75)
*From "one host" to "many clusters"*

| # | Module | What You Learn |
|---|--------|----------------|
| 72 | Multi-Host Orchestration | Docker Swarm, Nomad, comparison with k8s |
| 73 | Service Discovery | Consul, etcd, DNS-based discovery |
| 74 | Distributed Systems Patterns | CAP theorem, consensus, eventual consistency |
| 75 | Multi-Cloud & Hybrid | Avoiding vendor lock-in, federation |

### Phase 12: Production Mastery (Modules 76–80)
*From "it works" to "I run the whole show"*

| # | Module | What You Learn |
|---|--------|----------------|
| 76 | Incident Response | On-call, escalation, communication |
| 77 | Runbooks & Playbooks | Documented procedures for every scenario |
| 78 | Post-Mortems | Blameless analysis and continuous improvement |
| 79 | Change Management | Rolling updates, maintenance windows |
| 80 | Cost Optimization | Right-sizing, spot instances, reserved capacity |

### Capstone: The Unbreakable System (Module 81)
*Build a production system that handles everything*

Deploy a real application that demonstrates:
- Zero-downtime deployments
- Auto-scaling under load
- Self-healing when containers/servers fail
- Complete observability (metrics, logs, traces)
- Automated backup and disaster recovery
- DDoS mitigation
- Database high availability
- Multi-region deployment

## How to Use This Curriculum

```
1. Read the module README (the lesson)
2. Study the examples/ directory (working code)
3. Do the exercises/ directory (hands-on practice)
4. Check your work against solutions/
5. Read the "Limitation" section at the end
6. Move to the next module
```

## Module Structure

Each module follows this layout:
```
XX-module-name/
  README.md          — The lesson (read this first)
  examples/          — Working code examples
  exercises/         — Practice problems
  solutions/         — Reference solutions
  cheatsheet.md      — Quick reference
```

## Time Estimate

- **Phase 1–2** (Modules 01–17): ~2 weeks — You can containerize anything
- **Phase 3** (Modules 18–23): ~1 week — You understand scaling
- **Phase 4** (Modules 24–38): ~3 weeks — You are a Kubernetes expert
- **Phase 5** (Modules 39–44): ~1 week — You can observe everything
- **Phase 6** (Modules 45–50): ~1 week — You run databases in production
- **Phase 7** (Modules 51–56): ~1 week — You secure everything
- **Phase 8** (Modules 57–61): ~1 week — You ship code fearlessly
- **Phase 9** (Modules 62–66): ~1 week — You design networks
- **Phase 10** (Modules 67–71): ~1 week — You handle any traffic
- **Phase 11** (Modules 72–75): ~1 week — You orchestrate clusters
- **Phase 12** (Modules 76–80): ~1 week — You run production
- **Capstone** (Module 81): ~1 week — You prove it all

**Total: ~14 weeks to Legendary Grandmaster**

## Start Here

→ Go to [01-why-containers](01-why-containers/) to begin your journey.
