# Solution 03: Multi-Tier Auto-Scaling Strategy Design

## Part A: Scaling Mechanism for Each Tier

| Tier | Mechanism(s) | Justification |
|------|--------------|---------------|
| Web Frontend | **HPA** | Stateless, horizontally scalable, traffic-driven. CPU and request rate are good scaling signals. Classic HPA use case. |
| API Backend | **HPA (custom metrics)** | Stateless and horizontally scalable, but the bottleneck is database connections, not CPU. HPA with `active_db_connections` as a custom metric targets the actual constraint. |
| Order Workers | **KEDA** | Queue-driven workload that can scale to zero. KEDA natively monitors RabbitMQ queue depth and supports `minReplicaCount: 0`, which HPA does not. The cron trigger enables pre-warming. |

### Why This Works

Each tier's scaling mechanism matches its bottleneck:
- Web frontend: request volume (horizontal) -> HPA on CPU/RPS
- API backend: connection pool (horizontal, but custom metric) -> HPA on connections
- Order workers: queue depth (event-driven, can scale to zero) -> KEDA

The wrong choice for any tier leads to either over-provisioning or under-provisioning. For example, using CPU-based HPA on the API backend would miss connection saturation until it is too late.

---

## Part B: Web Frontend HPA

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: web-frontend-hpa
  namespace: production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: web-frontend
  minReplicas: 4
  maxReplicas: 20
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 60
    - type: Pods
      pods:
        metric:
          name: http_requests_per_second
        target:
          type: AverageValue
          averageValue: "500"
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 0
      policies:
        - type: Percent
          value: 100
          periodSeconds: 30
        - type: Pods
          value: 4
          periodSeconds: 30
      selectPolicy: Max
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
        - type: Percent
          value: 10
          periodSeconds: 60
```

### Why This Works

- CPU at 60% leaves headroom for burst traffic. At 50%, normal variance causes flapping.
- RPS at 500 per pod is a realistic target for a web frontend serving a mix of cached and dynamic content.
- Scale-up with `stabilizationWindowSeconds: 0` reacts immediately to traffic spikes. The `Max` select policy ensures the more aggressive of 100% or 4 pods is used.
- Scale-down with 300-second stabilization prevents premature removal after brief traffic dips.

---

## Part C: API Backend HPA

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: api-backend-hpa
  namespace: production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api-backend
  minReplicas: 3
  maxReplicas: 15
  metrics:
    # Primary: database connections (the real bottleneck)
    - type: Pods
      pods:
        metric:
          name: active_db_connections
        target:
          type: AverageValue
          averageValue: "70"
    # Secondary: CPU utilization
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 30
      policies:
        - type: Percent
          value: 100
          periodSeconds: 30
        - type: Pods
          value: 3
          periodSeconds: 30
      selectPolicy: Max
    scaleDown:
      stabilizationWindowSeconds: 600
      policies:
        - type: Percent
          value: 5
          periodSeconds: 120
```

### Why This Works

- **Connection target of 70, not 100:** The hard limit is 100 connections per pod. Setting the target at 70 provides 30% headroom for:
  - Connection establishment overhead (new connections take time to authenticate)
  - Burst traffic that temporarily exceeds steady-state
  - Connection pool exhaustion protection (once you hit 100, new requests fail)
  - This is a general rule: never target the absolute maximum of a finite resource.

- **CPU at 70% as secondary:** CPU is a backup signal. If connections are fine but CPU spikes (e.g., a computationally expensive request), the CPU metric triggers scaling.

- **Conservative scale-down:** 5% every 120 seconds with 600-second stabilization. With 15 replicas, that is less than 1 pod per 2 minutes. This prevents the API from thrashing between replica counts, which would cause connection pool rebalancing overhead.

---

## Part D: KEDA ScaledObject for Order Workers

```yaml
apiVersion: keda.sh/v1alpha1
kind: ScaledObject
metadata:
  name: order-worker-scaledobject
  namespace: production
spec:
  scaleTargetRef:
    name: order-worker
  pollingInterval: 15
  cooldownPeriod: 300
  minReplicaCount: 0
  maxReplicaCount: 20
  triggers:
    # Primary trigger: RabbitMQ queue depth
    - type: rabbitmq
      metadata:
        host: amqp://rabbitmq.production.svc:5672
        queueName: orders
        queueLength: "50"               # One pod per 50 messages
      authenticationRef:
        name: rabbitmq-credentials
    # Secondary trigger: Pre-warming for business hours
    - type: cron
      metadata:
        timezone: America/New_York
        start: "0 8 * * 1-5"           # 8 AM, weekdays
        end: "0 20 * * 1-5"            # 8 PM, weekdays
        desiredReplicas: "5"
---
apiVersion: keda.sh/v1alpha1
kind: TriggerAuthentication
metadata:
  name: rabbitmq-credentials
  namespace: production
spec:
  secretTargetRef:
    - parameter: username
      name: rabbitmq-credentials
      key: username
    - parameter: password
      name: rabbitmq-credentials
      key: password
```

### Why This Works

- **`minReplicaCount: 0`:** Enables scale-to-zero. When the queue is empty and the cron trigger is not active, all pods are removed. This saves compute costs during off-hours.

- **`queueLength: "50"`:** One pod handles approximately 50 orders/minute. If the queue has 200 messages, KEDA scales to 4 pods. If the queue has 0 messages and the cron trigger is inactive, KEDA scales to 0.

- **Cron trigger for pre-warming:** The cron trigger ensures 5 replicas are running during business hours, even before traffic arrives. Without this, there would be a cold-start delay as KEDA reacts to the first queue messages.

- **`cooldownPeriod: 300`:** After scaling activity stops, KEDA waits 5 minutes before scaling to zero. This prevents oscillation during brief gaps in orders.

- **`pollingInterval: 15`:** KEDA checks the queue every 15 seconds. This provides a good balance between responsiveness and API load on RabbitMQ.

- **TriggerAuthentication:** Credentials are stored in a Kubernetes Secret and referenced via `TriggerAuthentication`. This keeps sensitive values out of the ScaledObject manifest.

---

## Part E: Cluster Autoscaler Configuration

```bash
# Cluster Autoscaler deployment command flags
./cluster-autoscaler \
  --v=4 \
  --cloud-provider=aws \
  --skip-nodes-with-local-storage=false \
  --expander=least-waste \
  --node-group-auto-discovery=asg:tag=k8s.io/cluster-autoscaler/enabled,k8s.io/cluster-autoscaler/my-ecommerce-cluster \
  --balance-similar-node-groups \
  --scale-down-utilization-threshold=0.5 \
  --scale-down-delay-after-add=10m \
  --scale-down-unneeded-time=10m \
  --max-graceful-termination-sec=600 \
  --max-node-provision-time=10m
```

```yaml
# AWS ASG tags for autodiscovery
# Tag 1:
#   Key: k8s.io/cluster-autoscaler/enabled
#   Value: owned
# Tag 2:
#   Key: k8s.io/cluster-autoscaler/my-ecommerce-cluster
#   Value: owned
```

### Why This Works

- `--expander=least-waste`: When multiple node groups can satisfy pending pods, choose the one that wastes the fewest resources. For example, if a pod needs 2 CPU, prefer a 4-CPU node (2 CPU wasted) over an 8-CPU node (6 CPU wasted).

- `--balance-similar-node-groups`: Distributes nodes evenly across availability zones. Without this, the autoscaler might add all nodes to one AZ, reducing fault tolerance.

- `--scale-down-utilization-threshold=0.5`: Nodes below 50% utilization are candidates for removal. This is aggressive enough to save costs but conservative enough to avoid removing nodes that are about to receive new pods.

- `--scale-down-delay-after-add=10m`: After adding a node, wait 10 minutes before considering scale-down. This prevents the autoscaler from immediately removing a node it just added (which can happen if metrics have not updated yet).

- `--max-graceful-termination-sec=600`: Allows 10 minutes for pods to terminate gracefully. Critical for order workers that may be mid-transaction.

- `--max-node-provision-time=10m`: If a node does not become ready within 10 minutes, the autoscaler assumes provisioning failed and tries again.

---

## Part F: Capacity Planning Calculation

**Total resource requests:**

| Component | Replicas | CPU per pod | RAM per pod | Total CPU | Total RAM |
|-----------|----------|-------------|-------------|-----------|-----------|
| Web Frontend | 20 | 200m | 256Mi | 4000m | 5120Mi |
| API Backend | 15 | 500m | 512Mi | 7500m | 7680Mi |
| Order Workers | 20 | 250m | 256Mi | 5000m | 5120Mi |
| System pods | - | - | - | 2000m | 4096Mi |
| **Total** | | | | **18500m** | **22016Mi (21.5 Gi)** |

**Nodes required:**

- By CPU: 18500m / 3800m = 4.87 -> **5 nodes**
- By RAM: 22016Mi / 14336Mi (14 Gi) = 1.54 -> **2 nodes**

The CPU constraint is more demanding: **5 nodes required**.

**Does the max node pool size of 10 accommodate this?**

Yes. 5 nodes needed, 10 available. There is headroom for 10 additional pods to scale further or for other workloads on the cluster.

### Why This Works

CPU is almost always the binding constraint for horizontally scaled web applications. RAM rarely drives node count unless you have memory-intensive workloads (caches, in-memory databases). The calculation assumes all pods are at maximum replicas simultaneously, which is an extreme scenario -- in practice, different tiers peak at different times (web frontend peaks during lunch, order workers peak after), so the actual peak node count may be lower.

---

## Common Mistakes to Avoid

- **Using CPU-based HPA for the API backend.** The bottleneck is database connections, not CPU. CPU-based scaling would allow the API to accept more requests than the connection pool can handle, causing database errors.

- **Setting KEDA `minReplicaCount` to 1 instead of 0.** If you do not need scale-to-zero, you are paying for idle workers. The whole point of KEDA for queue-driven workloads is the ability to scale to zero during off-hours.

- **Forgetting the cron pre-warming trigger.** Without it, the first orders of the day sit in the queue while KEDA spins up pods. The cron trigger ensures workers are ready before traffic arrives.

- **Not leaving headroom on the connection pool.** Targeting 100 connections per pod (the hard limit) means new requests fail during connection establishment. Always target 60-70% of a hard limit.

- **Ignoring the node provisioning delay.** HPA creates pods in seconds, but nodes take 2-5 minutes. If the cluster is at capacity, pods will be Pending during the provisioning window. Plan for this with Cluster Autoscaler's `--max-node-provision-time`.

## Key Takeaway

Each tier in a multi-tier application has different scaling characteristics, and the autoscaling strategy must match each tier's bottleneck. The web frontend scales on request volume, the API scales on connection pool utilization, and the workers scale on queue depth. Cluster Autoscaler is the shared safety net that ensures all tiers have node capacity when they need it.
