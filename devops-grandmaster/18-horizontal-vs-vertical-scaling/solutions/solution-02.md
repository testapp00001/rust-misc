# Solution 02: Component Scaling Classification

## Part A: Classify Each Component

| Component | Current Scaling | Can Scale Horizontally? | Blocker (if any) | Recommended Strategy |
|-----------|----------------|------------------------|-------------------|---------------------|
| App Server | Single instance, vertical | Yes | Local file uploads and in-memory LRU cache | Horizontal (after externalizing state) |
| PostgreSQL | Single instance, vertical | Partially | Write conflicts, data consistency | Vertical first, read replicas for reads |
| Redis | Single instance, vertical | Partially | Distributed state coordination | Vertical first (single-node is fast enough) |
| Local Disk (uploads) | Single instance, local filesystem | No | Files stored on local disk, not accessible by other instances | Move to object storage (S3) |
| In-memory LRU cache | In-process, single instance | No | Cache is inside the app process, not shared | Move to Redis or Memcached |

### Why each decision was made:

**App Server:** The app server itself is stateless in its request processing logic. The blockers are the local file storage and the in-memory cache. Once those are externalized, you can run as many app server instances as you need. This is the primary horizontal scaling candidate.

**PostgreSQL:** Databases are fundamentally hard to scale horizontally because writes must be coordinated. The first step is always vertical (more RAM for buffer cache, faster SSD for I/O). For the read-heavy workload (80% read), adding read replicas is a valid horizontal strategy -- but the primary remains a single vertically-scaled instance.

**Redis:** Redis is single-threaded and extremely fast. A single Redis instance can handle hundreds of thousands of operations per second. Vertical scaling (more RAM) is the right first step. Redis Cluster exists for horizontal scaling, but it adds operational complexity that is rarely justified until you hit very high scale.

**Local Disk:** This is the critical blocker. Files stored on local disk are only accessible by the instance that wrote them. If you run 3 app server instances, a file uploaded to instance 1 cannot be served by instances 2 or 3. This must be moved to object storage before horizontal scaling.

**In-memory LRU cache:** The cache is inside the app process. Each instance would have its own separate cache, leading to inconsistency (instance 1 caches a product, instance 2 does not). Moving to a shared Redis cache solves this.

---

## Part B: Externalize the State

Three pieces of state must be externalized:

### 1. File uploads (Local Disk -> Object Storage)

**Current:** Files saved to the local filesystem of the app server.
**Solution:** Move to S3 (or MinIO for local development). The app server streams uploads directly to S3 and returns an S3 URL. No files touch the local disk.
**Why:** Object storage is accessible from any instance, is inherently distributed, and handles replication and durability automatically.

### 2. Product catalog cache (In-memory LRU -> Redis)

**Current:** Each app server has an in-memory LRU cache for product lookups.
**Solution:** Use the existing Redis instance (or a separate Redis instance) as a shared cache. Replace the in-memory LRU with Redis GET/SET calls.
**Why:** A shared cache means all instances see the same cached data. Cache invalidation is centralized -- update Redis once, all instances see the change.

### 3. Sessions (Already externalized to Redis)

**Current:** Sessions are already stored in Redis.
**Action:** No change needed. This was the right architectural decision.

After externalization, the app server becomes fully stateless:
- No files on local disk
- No caches in process memory
- No sessions in process memory
- Every request can be served by any instance

---

## Part C: Design the Target Architecture

```
                    ┌──────────────┐
                    │   Browser    │
                    └──────┬───────┘
                           │
                    ┌──────┴───────┐
                    │    Nginx     │
                    │ (load        │
                    │  balancer)   │
                    └──────┬───────┘
                           │
            ┌──────────────┼──────────────┐
            │              │              │
       ┌────┴────┐   ┌────┴────┐   ┌────┴────┐
       │ App #1  │   │ App #2  │   │ App #3  │
       │ :3000   │   │ :3000   │   │ :3000   │
       └────┬────┘   └────┬────┘   └────┬────┘
            │              │              │
            └──────────────┼──────────────┘
                           │
          ┌────────────────┼────────────────┐
          │                │                │
    ┌─────┴─────┐   ┌─────┴─────┐   ┌─────┴──────┐
    │ PostgreSQL│   │   Redis   │   │     S3     │
    │ (primary) │   │ (sessions,│   │  (uploads) │
    │  Vertical │   │  cache)   │   │            │
    │  scaling  │   │  Vertical │   │  Managed   │
    └───────────┘   └───────────┘   └────────────┘

    Horizontal:     Vertical:        Managed:
    App servers     PostgreSQL       S3 (auto)
    (3 replicas)    Redis
```

**Scaling summary:**
- **Horizontal:** App servers (3 instances, can add more behind the load balancer)
- **Vertical:** PostgreSQL (upgrade instance size), Redis (upgrade instance size)
- **Managed/auto:** S3 (scales automatically, pay per use)

---

## Part D: Write the Docker Compose

```yaml
version: "3.8"

services:
  app:
    build: .
    deploy:
      replicas: 3
      resources:
        limits:
          cpus: "0.50"
          memory: 256M
    environment:
      - DATABASE_URL=postgresql://user:pass@db:5432/app
      - REDIS_URL=redis://redis:6379
      - S3_ENDPOINT=http://minio:9000
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 10s
      timeout: 5s
      retries: 3
    depends_on:
      db:
        condition: service_healthy
      redis:
        condition: service_started

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      - app
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost/nginx-health"]
      interval: 10s
      timeout: 5s
      retries: 3

  db:
    image: postgres:16-alpine
    environment:
      - POSTGRES_USER=user
      - POSTGRES_PASSWORD=pass
      - POSTGRES_DB=app
    volumes:
      - pgdata:/var/lib/postgresql/data
    deploy:
      resources:
        limits:
          cpus: "1.0"
          memory: 512M
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U user -d app"]
      interval: 10s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    deploy:
      resources:
        limits:
          cpus: "0.25"
          memory: 128M

  minio:
    image: minio/minio
    command: server /data --console-address ":9001"
    environment:
      - MINIO_ROOT_USER=minioadmin
      - MINIO_ROOT_PASSWORD=minioadmin
    volumes:
      - minio-data:/data
    deploy:
      resources:
        limits:
          cpus: "0.25"
          memory: 256M

volumes:
  pgdata:
  minio-data:
```

---

## Common Mistakes

1. **Scaling the app server before externalizing state.** If you add replicas while files are still on local disk, users will get 404 errors when they try to access files uploaded to a different instance. Externalize state first, then scale.

2. **Scaling the database horizontally as the first step.** Read replicas help with read traffic, but they add complexity (replication lag, connection routing). The first step should always be vertical -- a bigger instance with more RAM and faster storage is simpler and often sufficient.

3. **Using a shared filesystem (NFS) instead of object storage for uploads.** NFS works but introduces a single point of failure, performance bottlenecks, and file-locking issues. Object storage (S3) is designed for this exact use case and scales automatically.

4. **Forgetting to externalize the cache.** It is easy to miss the in-memory cache because it seems harmless -- it is "just a cache." But with multiple instances, each has a different view of the data, leading to inconsistent behavior. A shared cache (Redis) ensures all instances see the same data.

5. **Not setting resource limits on containers.** Without limits, one container can consume all host resources, starving others. Resource limits ensure fair distribution and make capacity planning predictable.

---

## Relevant README Sections

- [Stateful vs Stateless Applications](../README.md#stateful-vs-stateless-applications)
- [Making Applications Stateless](../README.md#making-applications-stateless)
- [Real-World Architecture](../README.md#real-world-architecture)
