# Module 18: Horizontal vs Vertical Scaling

> **Previous module (17):** Image versioning — tagging, pinning, and rolling back container images.
> **Limitation:** A single container instance, no matter how well-versioned, cannot handle increasing traffic.
> **This module:** Learn when to scale up (bigger machine) vs scale out (more machines).

---

## 1. The Problem

Your application is running. Users love it. Traffic is growing. One server handled 100 requests/second fine, but now you need 1,000 requests/second. Your CPU is at 95%. Memory is maxed out. What do you do?

This is the fundamental scaling question: do you make your one machine more powerful, or do you add more machines?

The answer depends on your application architecture, your budget, and your reliability requirements. Choose wrong and you either waste money or hit a hard ceiling you cannot break through.

---

## 2. The Naive Way — Just Buy a Bigger Server

The simplest approach: upgrade the machine. More CPU cores, more RAM, faster disk. This is **vertical scaling** (scaling up).

```
Before:  4 CPU, 16GB RAM  → handles 500 req/s
After:   16 CPU, 64GB RAM → handles 2,000 req/s
```

**Why it seems good:**
- No code changes needed
- No architecture changes
- Your application runs exactly the same, just faster
- Simple to reason about

**Why it fails:**

1. **Hardware limits exist.** The largest cloud instance tops out. AWS `u-24tb1.metal` has 448 vCPUs and 24TB RAM. That is the ceiling. When you hit it, vertical scaling is done.

2. **Cost grows exponentially.** Doubling CPU does not double performance due to shared memory buses, NUMA effects, and kernel contention. A 96-core machine is not 24x faster than a 4-core machine for most workloads.

3. **Single point of failure.** One machine means one failure domain. A hardware fault, kernel panic, or bad deployment takes down 100% of your service.

4. **Downtime for upgrades.** Resizing a server often requires a restart. With one server, that means downtime.

5. **Diminishing returns.** More cores means more contention on locks, caches, and I/O. Beyond a certain point, adding cores makes some workloads slower.

```
Performance
    ^
    |            .----------- flatline (hardware limit)
    |          /
    |        /
    |      /
    |    /
    |  /
    | /
    +------------------------->
         Vertical Scaling (machine size)
```

---

## 3. The Right Way — Scale Horizontally (Scale Out)

**Horizontal scaling** means adding more machines (or containers) instead of making one machine bigger.

```
Before:  1 server  × 4 CPU, 16GB RAM → 500 req/s
After:   4 servers × 4 CPU, 16GB RAM → 2,000 req/s
```

**Why horizontal scaling is usually the right answer:**

1. **No hard ceiling.** Need more capacity? Add another instance. There is no maximum (in practice, there are limits, but they are far higher).

2. **Linear cost scaling.** Each additional instance costs roughly the same. Predictable budgets.

3. **Built-in redundancy.** If one instance dies, the others keep serving traffic. High availability comes for free.

4. **Zero-downtime deployments.** Deploy to one instance at a time while others serve traffic (rolling updates).

5. **Geographic distribution.** Instances can run in different regions, closer to users.

**The catch:** Your application must be designed for it.

### Stateful vs Stateless Applications

The key question: does your application store state in memory between requests?

**Stateless** (easy to scale horizontally):
```
Request comes in → Process it → Send response → Forget everything

- No in-memory session data
- No local file writes
- No in-process caches
- Each request is independent
```

**Stateful** (hard to scale horizontally):
```
Request comes in → Look up session in memory → Process → Update session → Send response

- Sessions stored in process memory
- Local file uploads
- In-memory caches (HashMap, LRU cache)
- WebSocket connections with state
- Long-running processes
```

### Making Applications Stateless

The pattern: externalize all state.

```
┌─────────────────────────────────────────────────┐
│              STATELESS APP INSTANCE              │
│                                                  │
│  - No sessions in memory                         │
│  - No files on local disk                        │
│  - No caches in process                          │
│  - All state lives OUTSIDE the app               │
└──────────────┬──────────────────┬────────────────┘
               │                  │
               v                  v
      ┌─────────────┐   ┌──────────────┐
      │   Redis     │   │   Database   │
      │ (sessions,  │   │  (persistent │
      │  cache)     │   │   data)      │
      └─────────────┘   └──────────────┘
```

State goes to external services:
- Sessions -> Redis or database
- File uploads -> S3 or object storage
- Cache -> Redis or Memcached
- Search -> Elasticsearch
- Message queues -> RabbitMQ or Kafka

---

## 4. The Production Way — How Real Systems Handle It

### Scaling Decision Framework

```
Is your bottleneck...
│
├─ CPU-bound? (computation)
│   ├─ Can you parallelize? → Horizontal
│   └─ Single-threaded work? → Vertical (then optimize code)
│
├─ Memory-bound?
│   ├─ Can you add more instances? → Horizontal + external cache
│   └─ Need shared memory? → Vertical (then refactor)
│
├─ I/O-bound? (disk/network)
│   ├─ Database queries? → Read replicas (horizontal) + caching
│   └─ File storage? → Object storage (S3)
│
└─ Connection-bound? (WebSockets, long polling)
    └─ Connection draining + horizontal with sticky sessions
```

### Cost Comparison

```
Vertical: 1 instance, 64 vCPU, 256GB RAM
  AWS m5.16xlarge: ~$3.07/hr = $2,210/month
  Single point of failure
  Max capacity: whatever this one machine handles

Horizontal: 16 instances, 4 vCPU, 16GB RAM each
  AWS m5.xlarge: ~$0.19/hr each = $2,197/month (similar cost!)
  16 redundant instances
  Max capacity: scales by adding more instances
```

Similar cost, radically different reliability and scaling ceiling.

### Real-World Architecture

```
                    ┌─────────────┐
                    │Load Balancer│
                    └──────┬──────┘
                           │
            ┌──────────────┼──────────────┐
            │              │              │
       ┌────┴────┐   ┌────┴────┐   ┌────┴────┐
       │ App #1  │   │ App #2  │   │ App #3  │
       └────┬────┘   └────┬────┘   └────┬────┘
            │              │              │
            └──────────────┼──────────────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
        ┌─────┴─────┐ ┌───┴───┐ ┌─────┴─────┐
        │  Database │ │ Redis │ │    S3     │
        │ (primary) │ │(cache)│ │  (files)  │
        └───────────┘ └───────┘ └───────────┘
```

The app instances are stateless. All state is in external services. You can add or remove app instances without affecting anything.

---

## 5. Hands-On Lab

### Lab A: Observe Vertical Scaling Limits

Create a CPU-bound application that hits the ceiling.

**app.py:**
```python
from flask import Flask
import time

app = Flask(__name__)

@app.route('/heavy')
def heavy():
    """CPU-intensive task: calculate primes."""
    start = time.time()
    count = 0
    for num in range(2, 10000):
        if all(num % i != 0 for i in range(2, int(num**0.5) + 1)):
            count += 1
    elapsed = time.time() - start
    return f"Found {count} primes in {elapsed:.2f}s"

@app.route('/health')
def health():
    return "OK"

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

**Dockerfile:**
```dockerfile
FROM python:3.11-slim
WORKDIR /app
RUN pip install flask
COPY app.py .
CMD ["python", "app.py"]
```

**docker-compose.yml (vertical — one container, limited CPU):**
```yaml
version: "3.8"
services:
  app:
    build: .
    ports:
      - "5000:5000"
    deploy:
      resources:
        limits:
          cpus: "0.5"    # Half a CPU core
          memory: 256M
```

Run and test:
```bash
docker compose up -d

# Stress test with limited CPU
docker run --rm --network host alpine/curl \
  sh -c 'for i in $(seq 1 10); do curl -s http://localhost:5000/heavy & done; wait'
# Note how long each request takes
```

Now increase CPU:
```yaml
deploy:
  resources:
    limits:
      cpus: "2.0"    # Two full CPU cores
      memory: 256M
```

```bash
docker compose up -d
# Same test — requests complete faster, but you've hit the limit of one machine
```

### Lab B: Horizontal Scaling with Docker Compose

**docker-compose.scale.yml:**
```yaml
version: "3.8"
services:
  app:
    build: .
    deploy:
      replicas: 4    # 4 identical instances
      resources:
        limits:
          cpus: "0.5"
          memory: 256M

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      - app
```

**nginx.conf (basic round-robin across replicas):**
```nginx
events {
    worker_connections 1024;
}

http {
    upstream app_servers {
        server app:5000;
        # Docker Compose DNS resolves to all replicas
    }

    server {
        listen 80;

        location / {
            proxy_pass http://app_servers;
        }
    }
}
```

```bash
docker compose -f docker-compose.scale.yml up -d

# Test — requests are distributed across 4 instances
for i in $(seq 1 8); do
  curl -s http://localhost/health
  echo
done
```

**Key observation:** Each instance uses only 0.5 CPU, but combined they deliver 2 CPU cores worth of throughput. You achieved the same capacity as vertical scaling, with redundancy.

---

## 6. Limitation

Horizontal scaling solves the compute problem, but creates a new one: **how do you distribute traffic across multiple instances?**

Right now, Nginx is doing basic round-robin. But what if one instance is slow? What if one is unhealthy? What about SSL? What about routing rules?

The next problem is intelligent traffic distribution.

---

## 7. Next Topic

**Module 19: Load Balancing** — Distributing traffic across multiple instances with health checks, algorithms, and failover. [Go to Module 19 →](../19-load-balancing/README.md)
