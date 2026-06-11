# Exercise 03: Multi-Tier Auto-Scaling Strategy Design

**Type:** Independent
**Time:** 30-45 minutes
**Difficulty:** Medium

## Objective

Design a complete auto-scaling strategy for a three-tier application (web frontend, API backend, background workers) where each tier has different scaling characteristics, bottlenecks, and constraints.

## Scenario / Starting Point

You are the platform engineer for an e-commerce company. The application has three tiers:

**Tier 1 -- Web Frontend (`web-frontend`)**
- Serves static assets and server-rendered pages
- Stateless, horizontally scalable
- Behind an Ingress controller
- Traffic pattern: 80% of requests are cacheable, 20% hit the API
- Current: 4 replicas, each requesting 200m CPU / 256Mi memory

**Tier 2 -- API Backend (`api-backend`)**
- Handles business logic, reads/writes to PostgreSQL
- Stateless, but each pod opens up to 100 database connections
- Connection pool is the real bottleneck, not CPU
- Current: 3 replicas, each requesting 500m CPU / 512Mi memory

**Tier 3 -- Order Workers (`order-worker`)**
- Processes orders from a RabbitMQ queue
- Each pod handles approximately 50 orders/minute
- Can scale to zero when no orders are pending
- Current: 2 replicas, each requesting 250m CPU / 256Mi memory

The cluster runs on AWS EKS with a node pool of `m5.xlarge` instances (4 vCPU, 16 GB RAM). The node pool has a minimum of 2 nodes and a maximum of 10 nodes.

## Tasks

### Part A: Choose the Scaling Mechanism for Each Tier

For each tier, select the appropriate autoscaling mechanism(s) and justify your choice. If you combine mechanisms, explain what each one handles.

| Tier | Mechanism(s) | Justification |
|------|--------------|---------------|
| Web Frontend | ? | ? |
| API Backend | ? | ? |
| Order Workers | ? | ? |

<details>
<summary>Hint 1</summary>

The web frontend is a textbook HPA candidate -- stateless, CPU-bound, horizontally scalable. The API backend's bottleneck is database connections, which is a custom metric -- HPA with custom metrics. The order workers are queue-driven and can scale to zero -- that is KEDA territory.

</details>

### Part B: Write the HPA for the Web Frontend

Write a complete HPA manifest for `web-frontend` that:
- Targets the `web-frontend` Deployment in the `production` namespace
- Scales between 4 and 20 replicas
- Uses CPU utilization at 60% as the primary metric
- Includes a custom metric `http_requests_per_second` with a target of 500 RPS per pod
- Has behavior policies: scale up fast (100% every 30s, stabilization 0s), scale down slowly (10% every 60s, stabilization 300s)

```yaml
# Your web-frontend HPA here
```

<details>
<summary>Hint 2</summary>

Remember that `autoscaling/v2` requires the `metrics` array with entries of `type: Resource` for CPU and `type: Pods` for custom metrics. The `behavior` section controls the rate of change.

</details>

### Part C: Write the HPA for the API Backend

Write a complete HPA manifest for `api-backend` that:
- Targets the `api-backend` Deployment in the `production` namespace
- Scales between 3 and 15 replicas
- Uses CPU at 70% as a secondary metric
- Uses `active_db_connections` with a target of 70 per pod as the primary metric (leave headroom from the 100-connection hard limit)
- Has conservative scale-down behavior: stabilization window of 600 seconds, max 5% removal per 120 seconds

```yaml
# Your api-backend HPA here
```

<details>
<summary>Hint 3</summary>

Setting the connection target to 70 (not 100) gives 30% headroom for connection spikes and connection establishment overhead. This is a common production pattern -- never target the absolute maximum of a finite resource.

</details>

### Part D: Write the KEDA ScaledObject for Order Workers

Write a KEDA `ScaledObject` manifest for `order-worker` that:
- Targets the `order-worker` Deployment in the `production` namespace
- Scales from 0 to 20 replicas
- Uses RabbitMQ queue depth as the trigger: one pod per 50 messages
- Polls the queue every 15 seconds
- Has a cooldown period of 300 seconds (5 minutes) before scaling to zero
- Adds a secondary cron trigger: scale to 5 replicas between 8 AM and 10 PM EST on weekdays (pre-warming for business hours)

```yaml
# Your KEDA ScaledObject here
```

<details>
<summary>Hint 4</summary>

KEDA ScaledObject structure:
- `scaleTargetRef.name` references the Deployment
- `minReplicaCount: 0` enables scale-to-zero
- `triggers` is an array -- you can have multiple triggers (RabbitMQ + cron)
- For the cron trigger, use `type: cron` with `start`, `end`, `timezone`, and `desiredReplicas` in `metadata`

</details>

### Part E: Cluster Autoscaler Configuration

The Cluster Autoscaler needs to handle the case where all three tiers scale up simultaneously. Write the key flags for a Cluster Autoscaler deployment that:

1. Discovers nodes via AWS ASG tags
2. Uses the `least-waste` expander (picks the node group that wastes the fewest resources)
3. Waits 10 minutes before scaling down an unneeded node
4. Scales down when node utilization drops below 50%
5. Balances across similar node groups (for multi-AZ)

```bash
# Cluster Autoscaler command flags
```

<details>
<summary>Hint 5</summary>

The key flags are:
- `--node-group-auto-discovery=asg:tag=k8s.io/cluster-autoscaler/enabled,k8s.io/cluster-autoscaler/<cluster-name>`
- `--expander=least-waste`
- `--scale-down-unneeded-time=10m`
- `--scale-down-utilization-threshold=0.5`
- `--balance-similar-node-groups`

</details>

### Part F: Capacity Planning Calculation

Given the current load scenario, calculate whether the cluster can handle all tiers scaling to their maximum simultaneously:

- Web Frontend: 20 replicas x (200m CPU, 256Mi RAM)
- API Backend: 15 replicas x (500m CPU, 512Mi RAM)
- Order Workers: 20 replicas x (250m CPU, 256Mi RAM)
- System pods (kube-system, monitoring): approximately 2 CPU, 4GB RAM

Each `m5.xlarge` node has 4 vCPU (4000m) and 16 GB RAM. After system reservations, approximately 3800m CPU and 14 GB is allocatable per node.

Show:
1. Total CPU needed
2. Total RAM needed
3. Number of nodes required (calculate for both CPU and RAM, take the max)
4. Does the max node pool size of 10 accommodate this?

## Success Criteria

- [ ] Each tier has an appropriate scaling mechanism with a valid technical justification.
- [ ] Web frontend HPA uses CPU + RPS metrics with fast scale-up and slow scale-down.
- [ ] API backend HPA uses connection count as primary metric with headroom below the hard limit.
- [ ] KEDA ScaledObject enables scale-to-zero with a queue-based trigger and a cron pre-warming trigger.
- [ ] Capacity calculation shows whether the node pool can handle the maximum scale scenario.

## What You Should Understand After This Exercise

Each tier in a multi-tier application has different scaling characteristics. The frontend scales on request volume, the backend scales on resource contention (connections, not CPU), and workers scale on queue depth. Choosing the wrong metric for any tier leads to either over-provisioning (wasting money) or under-provisioning (dropping requests). Cluster Autoscaler is the safety net that ensures HPA-created pods actually have nodes to run on.
