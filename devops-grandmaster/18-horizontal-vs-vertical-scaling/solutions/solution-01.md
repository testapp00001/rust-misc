# Solution 01: Scaling Trade-Offs

## Part A: Classify Each Scenario

### 1. PostgreSQL database -- out of memory, slow on complex joins

**Vertical scaling.**

PostgreSQL is a stateful, monolithic system. Complex joins require data to be in memory on a single node for efficient execution. You cannot split a single query across multiple machines. Upgrading to a machine with more RAM allows more data to fit in the shared buffer cache, speeding up joins directly. Horizontal scaling (read replicas) helps with read traffic but does not help a single slow join -- that query still runs on one node.

### 2. Stateless REST API -- 500 req/s, CPU at 80%

**Horizontal scaling.**

The API is stateless (by definition in the scenario), so running multiple copies is straightforward. At 80% CPU, you are approaching the vertical ceiling on a single instance. Adding a second instance behind a load balancer drops each instance to roughly 40% CPU, giving headroom for growth. This is the textbook horizontal scaling use case: stateless, CPU-bound, near capacity.

### 3. Machine learning model -- 50GB dataset in RAM

**Vertical scaling (primarily), then horizontal if parallelizable.**

Each inference requires 50GB of RAM. You need a machine with at least 64GB RAM just to run one instance. If the model cannot be sharded, vertical scaling is the only option. If you can run independent inferences in parallel (batch processing), you could run multiple large machines -- but each one still needs to be vertically large enough to hold the dataset.

### 4. WebSocket server -- 10,000 connections with per-connection state

**Vertical scaling first, then horizontal with sticky sessions.**

Each WebSocket connection holds state in memory on the server that established it. You cannot move a WebSocket connection to a different server mid-stream. Vertical scaling (more RAM, more CPU) lets you hold more connections on one server. If you must go horizontal, you need sticky sessions (route each client to the same server) or externalize the connection state -- both add significant complexity.

### 5. File-processing service -- video transcoding

**Horizontal scaling.**

Video transcoding is CPU-intensive but embarrassingly parallel. Each video is an independent job. You can run N workers, each processing one video, and distribute jobs via a queue. This is the ideal horizontal scaling workload: stateless (each job is independent), CPU-bound (parallelizable), and no shared state between workers.

---

## Part B: Draw the Cost Curve

### Vertical scaling only (single instance)

The largest instance (4xlarge) handles 2,400 req/s. The target is 3,000 req/s. **A single instance cannot handle the load.** Even the largest available instance falls 600 req/s short. Vertical scaling alone fails here.

### Horizontal scaling only (multiple smaller instances)

| Instance | Capacity | Instances needed | Cost/hour total | Cost/month (730h) |
|----------|----------|------------------|-----------------|-------------------|
| small (200 req/s) | 200 | 15 | 15 x $0.08 = $1.20 | $876 |
| medium (380 req/s) | 380 | 8 | 8 x $0.19 = $1.52 | $1,110 |
| large (700 req/s) | 700 | 5 | 5 x $0.42 = $2.10 | $1,533 |
| xlarge (1,200 req/s) | 1,200 | 3 | 3 x $0.92 = $2.76 | $2,015 |
| 2xlarge (1,800 req/s) | 1,800 | 2 | 2 x $2.01 = $4.02 | $2,935 |

**Cheapest option:** 15 small instances at $876/month.

**Best value option:** 5 large instances at $1,533/month -- fewer instances to manage, more headroom (5 x 700 = 3,500 req/s, 17% headroom), and each instance is a reasonable size.

**Why this works:** Horizontal scaling with smaller instances is dramatically cheaper because cost scales linearly with capacity at the small end. The 4xlarge instance costs 54x more than the small instance but only delivers 12x the capacity. The cost curve is exponential at the top end.

### Headroom comparison

- Vertical (if it worked): 2,400 req/s capacity for 3,000 req/s demand -- **negative headroom, cannot serve the load.**
- Horizontal (5 large): 3,500 req/s capacity for 3,000 req/s demand -- **17% headroom.**
- Horizontal (15 small): 3,000 req/s capacity for 3,000 req/s demand -- **0% headroom, risky.**

The 5 large instances provide the best balance of cost, headroom, and operational simplicity.

---

## Part C: Identify the Ceiling

### 1. Vertical scaling of a single EC2 instance

**Hard ceiling: the largest available instance type.** AWS's largest instance (u-24tb1.metal) has 448 vCPUs and 24TB RAM. When your workload exceeds that, vertical scaling is done. In practice, the effective ceiling is much lower because most workloads do not scale linearly with hardware -- NUMA effects, memory bus contention, and lock contention cause diminishing returns well before the hardware limit.

### 2. Horizontal scaling of a stateful application with in-memory sessions

**Hard ceiling: session affinity and failover.** Each client must be routed to the server holding its session. As you add more servers, the probability of a server failure increases. When a server dies, all sessions on it are lost. You can mitigate with session replication (copy sessions to other servers), but replication overhead grows quadratically with the number of servers. Beyond 10-20 servers, replication traffic consumes more bandwidth than actual user traffic.

### 3. Horizontal scaling of a database (without sharding)

**Hard ceiling: write throughput and cross-node coordination.** A single primary handles all writes. Read replicas help with reads, but writes still go to one node. Cross-node transactions require distributed consensus (2PC, Paxos), which adds latency proportional to the number of nodes. Without sharding, the primary is the bottleneck. With sharding, cross-shard queries become expensive, and resharding is operationally complex.

---

## Common Mistakes

1. **Assuming horizontal scaling is always cheaper.** At small scales, a single larger instance can be cheaper than multiple smaller ones due to fixed costs (load balancer, networking, operational overhead). The cost advantage of horizontal scaling appears at scale.

2. **Ignoring operational complexity.** Running 15 instances requires monitoring, deployment, and debugging infrastructure that a single instance does not. The cheapest option on paper may not be the cheapest when you factor in engineering time.

3. **Forgetting about the load balancer.** Horizontal scaling requires a load balancer, which is an additional cost and a single point of failure (unless you run multiple load balancers, which adds more complexity).

4. **Confusing "can scale horizontally" with "should scale horizontally."** A database *can* be horizontally scaled with read replicas, but the first step should always be vertical scaling (more RAM, faster disk) because it is simpler and does not require application changes.

5. **Not considering diminishing returns in vertical scaling.** Doubling the CPU count does not double throughput. Due to contention on shared resources (memory bus, locks, I/O), the actual speedup is often 1.3-1.7x, not 2x.

---

## Key Takeaway

The choice between horizontal and vertical scaling is a trade-off analysis, not a rule. Vertical scaling is simpler but has hard ceilings and diminishing cost returns. Horizontal scaling is more flexible but requires stateless architecture and operational infrastructure. In practice, production systems use both: vertically scale stateful components (databases) and horizontally scale stateless components (API servers). The cost-optimal choice depends on where you are on the cost curve -- small instances are cheap per unit of capacity, while large instances are expensive but simpler to manage.
