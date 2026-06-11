# Solution 03: VM vs Container -- Real Scenario Comparison

## Part A: Categorize Each Service

| Service | Choice | Primary Reason |
|---------|--------|----------------|
| Web Frontend | **Container** | Stateless, frequent deploys, small footprint |
| API Server | **Container** | Stateless, frequent deploys, well-suited for containers |
| Order Service | **Container** | Java 17 requirement is easily pinned in a Dockerfile |
| PostgreSQL | **Bare metal or VM** | Stateful, critical data, needs persistent storage |
| Redis Cache | **Bare metal or VM** | Large memory requirement, stateful, rarely changes |
| Legacy Billing | **VM** | Requires CentOS 6 (different kernel/libs), cannot modify |

### Detailed Reasoning

**Web Frontend (Container):** React + nginx is a textbook container use case.
Stateless, small image size (~20 MB), deploys multiple times per week.
Containers give fast deployments and easy rollbacks.

**API Server (Container):** Python/FastAPI is stateless and lightweight.
A container image pins the Python version and all dependencies.
Deploys 2-3 times per week -- containers make this trivial.

**Order Service (Container):** Java 17 is a specific version requirement
that a Dockerfile handles perfectly: `FROM eclipse-temurin:17-jre`.
The container bundles the exact JRE needed.

**PostgreSQL (Bare metal/VM):** Databases are the classic exception to
containerization. They are stateful, require persistent storage, and
their performance depends on direct access to disk I/O. Running PostgreSQL
in a container adds complexity (volume mounts, backup strategies) with
no real benefit. Use a managed database service or bare metal.

**Redis Cache (Bare metal/VM):** Redis needs 16 GB of dedicated memory
and benefits from direct memory access. It rarely changes (low deploy
frequency), so the fast-deployment advantage of containers is irrelevant.
Running on bare metal avoids the overhead of container memory management.

**Legacy Billing (VM):** This requires CentOS 6 with Apache 2.2, which
is a completely different OS from the Ubuntu 22.04 hosts. Containers share
the host kernel, so a CentOS 6 container on an Ubuntu 22.04 host would
have kernel compatibility issues. A VM provides the necessary isolation
with its own kernel.

## Part B: Resource Planning

### Container Allocation

```
Server 1: Web Frontend + API Server
====================================
  Web Frontend:    4 instances x  50 MB  =  200 MB RAM, 2 CPU
  API Server:      3 instances x 200 MB  =  600 MB RAM, 3 CPU
  Docker overhead:                           ~1 GB RAM, 1 CPU
  ────────────────────────────────────────────────────────────
  Total:                                    1.8 GB RAM, 6 CPU
  Remaining:                              126.2 GB RAM, 26 CPU

Server 2: Order Service + Redis
====================================
  Order Service:   2 instances x 512 MB  = 1024 MB RAM, 4 CPU
  Redis Cache:     1 instance  x  16 GB  =   16 GB RAM, 2 CPU
  Docker overhead:                           ~1 GB RAM, 1 CPU
  ────────────────────────────────────────────────────────────
  Total:                                   18.0 GB RAM, 7 CPU
  Remaining:                              110.0 GB RAM, 25 CPU

Server 3: PostgreSQL + Legacy Billing (VM)
====================================
  PostgreSQL:      1 instance  x   8 GB  =    8 GB RAM, 4 CPU
  Legacy VM:       1 VM        x   4 GB  =    4 GB RAM, 2 CPU
                   (CentOS 6 + Apache + PHP + 1 GB guest OS overhead)
  VM overhead:                               ~1 GB RAM, 1 CPU
  ────────────────────────────────────────────────────────────
  Total:                                   13.0 GB RAM, 7 CPU
  Remaining:                              115.0 GB RAM, 25 CPU
```

### Why This Allocation

- **Server 1** handles the highest-traffic, most-frequently-deployed services.
  Plenty of headroom for scaling up instances during peak traffic.
- **Server 2** groups the stateful services that need dedicated resources.
  Redis's 16 GB memory requirement dominates this server.
- **Server 3** runs the database on bare metal for performance, with the
  legacy VM sharing the server since both have low traffic.

### What If We Used VMs for Everything?

```
VM Approach:
  Each VM: 1 GB RAM + 1 CPU overhead (guest OS)
  Web Frontend VM:     1 GB + 50 MB  = 1.05 GB
  API Server VM:       1 GB + 200 MB = 1.2 GB
  Order Service VM:    1 GB + 512 MB = 1.5 GB
  PostgreSQL VM:       1 GB + 8 GB   = 9.0 GB
  Redis VM:            1 GB + 16 GB  = 17.0 GB
  Legacy VM:           1 GB + 3 GB   = 4.0 GB
  ─────────────────────────────────────────────
  Total: 6 VMs = 33.75 GB RAM + 12 CPU overhead

  vs containers: ~32.8 GB RAM + 20 CPU (including all services)
```

The VM approach wastes 6 GB on guest OS overhead alone, and you cannot
easily scale individual services -- each VM is a monolith.

## Part C: Deployment Speed Comparison

### Per-Service Deployment Times

| Service | VM Deploy | Container Deploy | Savings |
|---------|-----------|------------------|---------|
| Web Frontend | 8 min 15 sec | 1 min 37 sec | 6 min 38 sec |
| API Server | 8 min 15 sec | 1 min 37 sec | 6 min 38 sec |
| Order Service | 8 min 15 sec | 1 min 37 sec | 6 min 38 sec |
| PostgreSQL | 8 min 15 sec | N/A (bare metal) | -- |
| Redis Cache | 8 min 15 sec | N/A (bare metal) | -- |
| Legacy Billing | 8 min 15 sec | N/A (VM required) | -- |

### Calculation

```
VM deployment:
  Build image:    5 min 00 sec
  Transfer:       2 min 00 sec
  Boot:           0 min 45 sec
  Health check:   0 min 30 sec
  ───────────────────────────
  Total:          8 min 15 sec

Container deployment:
  Build image:    1 min 00 sec
  Transfer:       0 min 30 sec
  Start:          0 min 02 sec
  Health check:   0 min 05 sec
  ───────────────────────────
  Total:          1 min 37 sec
```

**Weekly impact** (assuming Web + API deploy 3x/week, Order 1x/week):

```
VM:     (3 + 3 + 1) x 8.25 min = 57.75 min/week of deployment time
Container: (3 + 3 + 1) x 1.62 min = 11.33 min/week of deployment time
Savings: 46.42 minutes per week
```

Over a year, containers save approximately **40 hours** of deployment time
for just these three services.

## Part D: Recommendation Document

### Architecture Diagram

```
QuickShip Production Architecture
==================================

Server 1 (Container Host)
┌─────────────────────────────────────────────────┐
│  Docker Engine                                  │
│  ┌──────────────┐  ┌──────────────┐            │
│  │ Web Frontend  │  │  API Server  │            │
│  │ (nginx+React) │  │  (FastAPI)   │            │
│  │ x4 instances  │  │  x3 instances│            │
│  └──────────────┘  └──────────────┘            │
│  Ubuntu 22.04 / Docker                          │
└─────────────────────────────────────────────────┘
         │                    │
         ▼                    ▼
Server 2 (Container Host)
┌─────────────────────────────────────────────────┐
│  Docker Engine                                  │
│  ┌──────────────┐  ┌──────────────┐            │
│  │ Order Service │  │  Redis Cache │            │
│  │ (Java 17)    │  │  (16 GB)     │            │
│  │ x2 instances  │  │  x1 instance │            │
│  └──────────────┘  └──────────────┘            │
│  Ubuntu 22.04 / Docker                          │
└─────────────────────────────────────────────────┘

Server 3 (Mixed)
┌─────────────────────────────────────────────────┐
│  ┌──────────────┐  ┌──────────────┐            │
│  │ PostgreSQL    │  │ Legacy VM    │            │
│  │ (bare metal)  │  │ (CentOS 6)   │            │
│  │ 500 GB data   │  │ Apache 2.2   │            │
│  │               │  │ PHP 5.6      │            │
│  └──────────────┘  └──────────────┘            │
│  Ubuntu 22.04 / KVM                             │
└─────────────────────────────────────────────────┘
```

### Trade-offs Considered

1. **PostgreSQL on bare metal vs container:** Bare metal gives better I/O
   performance and simpler backup/restore. The trade-off is less portability,
   but databases rarely move between environments.

2. **Legacy Billing in VM vs container:** A VM is necessary because CentOS 6
   uses kernel 2.6.32, which is incompatible with the Ubuntu 22.04 host kernel
   (5.15). Containers share the host kernel, so this would not work.

3. **Redis on bare metal vs container:** Redis benefits from direct memory
   access and does not need the deployment agility of containers. Bare metal
   avoids container memory management overhead.

4. **Multiple container instances per service:** Running multiple instances
   of Web Frontend and API Server provides redundancy. If one container
   crashes, others continue serving traffic.

### Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Docker daemon failure on Server 1 | All web/API traffic down | Health checks + automatic restart, failover to Server 2 |
| PostgreSQL disk failure | Data loss | Daily backups to external storage, replication (future) |
| Legacy VM security vulnerability | Compliance risk | Network isolation, plan for migration away from CentOS 6 |
| Container image vulnerability | Security breach | Image scanning in CI, regular base image updates |
| Server hardware failure | Service outage | Each server runs different services, no single point of failure |

### Common Mistakes to Avoid

- **Containerizing everything.** Databases and legacy systems often do not
  belong in containers. Use the right tool for each job.
- **Ignoring the legacy system.** It will not go away on its own. Plan a
  migration path (rewrite in modern stack, or at minimum upgrade the OS).
- **Over-provisioning containers.** Do not run 50 instances of a service
  that gets 10 requests per minute. Scale based on actual traffic.
- **Forgetting about persistent storage.** Stateful services need volumes
  or external storage. A container restart should not lose data.
