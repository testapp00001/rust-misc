# Solution 01: HPA vs VPA vs Cluster Autoscaler Decision Framework

## Part A: Scaling Mechanism Classification

| Workload | Primary Mechanism | Reasoning |
|----------|-------------------|-----------|
| A | **HPA** | Stateless REST API with variable traffic is the textbook HPA use case. Scale replicas horizontally based on CPU or request-rate metrics. |
| B | **VPA** | PostgreSQL is a single-writer stateful workload. You cannot add replicas without application-level changes (read replicas require config). VPA adjusts CPU/memory requests to match actual usage. |
| C | **KEDA** | Queue-driven workers that can scale to zero. KEDA monitors the queue depth natively and supports `minReplicaCount: 0`. HPA cannot scale to zero. |
| D | **VPA** | Redis as a single-instance cache is vertically scalable only. Adding replicas requires application-level sharding or sentinel configuration. VPA right-sizes the single pod. |
| E | **KEDA** | Batch ETL with a predictable schedule and zero-to-burst scaling. KEDA's cron trigger can pre-provision nodes, and the burst CPU demand maps to a resource-based or queue-based trigger. Alternatively, KEDA with a cron schedule combined with Cluster Autoscaler for the node burst. |
| F | **HPA (custom metric)** | WebSocket connections are the bottleneck, not CPU. HPA with a custom metric scaling on `active_websocket_connections` per pod. Standard CPU-based HPA would miss the real constraint. |
| G | **Manual / No autoscaling** | GPU nodes are fixed and expensive. Autoscaling GPU workloads requires GPU-aware scheduling, and the node pool is typically fixed-size. VPA could help right-size, but the real constraint is GPU count per node, which is not dynamically allocatable. |
| H | **VPA** | Monitoring stacks have steady, predictable resource usage and are currently over-provisioned. VPA in "Off" (recommendation) mode can right-size the resource requests without disruption, then you manually apply the recommendations. |

### Why This Works

The key insight is that the bottleneck determines the mechanism:
- **Horizontal bottleneck** (request count, connections, CPU across replicas) -> HPA
- **Vertical bottleneck** (memory per pod, CPU per pod, single-instance constraint) -> VPA
- **Event-driven bottleneck** (queue depth, stream lag, cron schedules) -> KEDA
- **Node-level bottleneck** (pods pending due to no capacity) -> Cluster Autoscaler

Many workloads benefit from combining mechanisms. For example, workload A uses HPA for pods and Cluster Autoscaler for nodes. Workload C uses KEDA for pods and Cluster Autoscaler for nodes.

---

## Part B: Scaling Flow Tracing

1. Traffic increases 5x at 2:00 PM
2. CPU utilization per pod rises from 30% to **75%** (assuming linear relationship)
3. HPA detects that **average CPU utilization** exceeds the target of 70%
4. HPA creates **new pods** (current: 3, desired: ceil(3 * 75/70) = **4**, or more aggressively depending on behavior policy -- could jump to 6-8 with a 100% scale-up policy)
5. New pods enter **Pending** state because there are no nodes with available capacity
6. Cluster Autoscaler detects **unschedulable pods** and provisions a new node
7. Node joins the cluster (takes approximately **2-5 minutes** on AWS EKS)
8. Pods are scheduled and begin **receiving traffic**
9. CPU utilization per pod **decreases** (load is now spread across more pods)
10. After traffic decreases, HPA waits **300-600 seconds** (stabilization window) before scaling down
11. Cluster Autoscaler waits **10 minutes** (`--scale-down-unneeded-time`) before removing underutilized nodes

### Why This Works

The scaling flow is a chain of dependencies:
- HPA reacts in seconds (metrics check interval: 15s default)
- Cluster Autoscaler reacts in minutes (node provisioning: 2-5 min)
- Total time from traffic spike to full capacity: 3-7 minutes

This is why [Module 68: Traffic Surge Handling](../68-traffic-surge-handling/) covers proactive strategies -- reactive scaling alone has inherent latency.

---

## Part C: Combining Mechanisms

### 1. Can you use HPA and VPA on the same workload targeting the same resource?

**No, not safely for the same resource.** HPA adjusts replica count based on CPU utilization. VPA adjusts CPU requests per pod. If both target CPU, they create a feedback loop: VPA increases CPU requests -> utilization percentage drops -> HPA scales down replicas -> load per pod increases -> CPU utilization rises -> HPA scales up -> VPA increases requests again.

**Exception:** You can use HPA on CPU and VPA on memory simultaneously, because they target different resources and do not interfere with each other's calculations.

### 2. Can you use VPA and Cluster Autoscaler together?

**Yes, and they complement each other well.** VPA adjusts pod resource requests (vertical scaling), which may cause pods to need more resources than the node can provide. Cluster Autoscaler detects that the updated resource requests exceed node capacity and provisions larger or additional nodes. The flow is:

VPA increases resource requests -> Pod needs rescheduling -> Node capacity insufficient -> Cluster Autoscaler adds nodes

### 3. Why is VPA "Off" mode safer for PostgreSQL?

VPA in "Auto" mode **restarts pods** to apply new resource requests. For a database like PostgreSQL:
- A restart means downtime (even if brief)
- If PostgreSQL is a single writer, there is no failover during restart
- Uncommitted transactions may be lost
- Connection pools need to reconnect
- PVC data is preserved, but the restart itself is disruptive

VPA in "Off" mode provides **recommendations only** -- you can review them and apply them during a maintenance window with proper drain procedures.

---

## Common Mistakes to Avoid

- **Using HPA on memory.** Memory utilization is typically stable and grows slowly. Scaling on memory causes HPA to react to slow trends, not traffic spikes. Use VPA for memory right-sizing instead.

- **Assuming Cluster Autoscaler replaces HPA.** Cluster Autoscaler adds nodes, not pods. You still need HPA or KEDA to create the pods that consume the node capacity.

- **Setting VPA to "Auto" on stateful workloads.** Any workload with persistent data or long-lived connections should use VPA in "Off" mode with manual application of recommendations.

- **Forgetting that HPA cannot scale to zero.** HPA has a minimum of 1 replica (default). If you need scale-to-zero, use KEDA which supports `minReplicaCount: 0`.

- **Ignoring the scaling latency chain.** HPA reacts in seconds, but Cluster Autoscaler needs minutes. If your traffic spikes are faster than node provisioning, you need over-provisioning or queue-based buffering.

## Key Takeaway

HPA, VPA, Cluster Autoscaler, and KEDA are complementary tools that operate at different layers of the scaling stack. The right choice depends on the workload's bottleneck metric, its statefulness, and whether it needs to scale to zero. In production, these compose together: HPA/KEDA handles pods, Cluster Autoscaler handles nodes, VPA right-sizes resource requests, and Prometheus monitoring ensures you catch problems before they cascade.
