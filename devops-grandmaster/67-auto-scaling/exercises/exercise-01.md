# Exercise 01: HPA vs VPA vs Cluster Autoscaler Decision Framework

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

Build a mental model for choosing the right autoscaling mechanism (HPA, VPA, or Cluster Autoscaler) for different workload characteristics, and understand how they compose together in the scaling flow.

## Scenario / Starting Point

Your team runs 8 different workloads on a shared Kubernetes cluster. The platform team has asked you to classify each workload by its ideal autoscaling strategy. You need to understand the trade-offs to make informed recommendations.

Here are the 8 workloads:

| # | Workload | Characteristics |
|---|----------|-----------------|
| A | Stateless REST API | Traffic varies 10x between peak and off-peak. Stateless. 3 replicas minimum for HA. |
| B | PostgreSQL database | Stateful. Vertical scaling only (single-writer). Memory-heavy workload. |
| C | Image processing workers | Consumes 4 CPU per pod. Pulls jobs from a queue. Can scale to zero when idle. |
| D | Redis cache | Stateful. Holds hot data in memory. Cannot split across pods without application changes. |
| E | Batch ETL jobs | Runs nightly. Needs 50 CPU cores for 2 hours, then zero. Not latency-sensitive. |
| F | WebSocket gateway | Long-lived connections. Cannot kill pods without draining connections. Connection count is the bottleneck, not CPU. |
| G | ML model inference | GPU-bound. Each pod needs 1 GPU. Cluster has a fixed GPU node pool. |
| H | Internal monitoring stack | Prometheus + Grafana. Steady resource usage. Over-provisioned by 3x currently. |

## Tasks

### Part A: Scaling Mechanism Classification

For each workload (A through H), choose the **primary** scaling mechanism and explain why in 1-2 sentences. Your choices are:

- **HPA** (Horizontal Pod Autoscaler)
- **VPA** (Vertical Pod Autoscaler)
- **Cluster Autoscaler**
- **KEDA** (Event-Driven Autoscaler)
- **Manual / No autoscaling**

Fill in the table:

| Workload | Primary Mechanism | Reasoning |
|----------|-------------------|-----------|
| A | ? | ? |
| B | ? | ? |
| C | ? | ? |
| D | ? | ? |
| E | ? | ? |
| F | ? | ? |
| G | ? | ? |
| H | ? | ? |

<details>
<summary>Hint 1</summary>

Think about what the bottleneck is for each workload. CPU? Memory? Queue depth? Connections? GPU? The bottleneck metric determines which autoscaler is appropriate.

</details>

<details>
<summary>Hint 2</summary>

HPA scales replica count (horizontal). VPA adjusts resource requests per pod (vertical). KEDA scales based on events/queues. Cluster Autoscaler adds/removes nodes. Some workloads benefit from combining mechanisms.

</details>

### Part B: Scaling Flow Tracing

For workload A (the stateless REST API), trace the complete scaling flow from a sudden traffic spike to the system reaching equilibrium. Fill in the missing steps:

1. Traffic increases 5x at 2:00 PM
2. CPU utilization per pod rises from 30% to ______
3. HPA detects that ______ exceeds the target of 70%
4. HPA creates ______ new pods (current: 3, target: ?)
5. New pods enter ______ state because there are no nodes with available capacity
6. Cluster Autoscaler detects ______ and provisions a new node
7. Node joins the cluster (takes approximately ______ minutes on AWS EKS)
8. Pods are scheduled and begin ______
9. CPU utilization per pod ______
10. After traffic decreases, HPA waits ______ (stabilization window) before scaling down
11. Cluster Autoscaler waits ______ before removing underutilized nodes

<details>
<summary>Hint 3</summary>

The stabilization window for scale-down is typically 300-600 seconds (5-10 minutes). Cluster Autoscaler has its own delay: `--scale-down-unneeded-time` defaults to 10 minutes. Node provisioning on cloud providers takes 2-5 minutes.

</details>

### Part C: Combining Mechanisms

Workload B (PostgreSQL) needs vertical scaling, but your team also wants the Cluster Autoscaler to handle node capacity. Explain:

1. Can you use HPA and VPA on the same workload targeting the same resource (e.g., CPU)? Why or why not?
2. Can you use VPA and Cluster Autoscaler together? What does each handle?
3. For PostgreSQL specifically, why might VPA in "Off" mode (recommendation-only) be safer than "Auto" mode?

<details>
<summary>Hint 4</summary>

VPA in "Auto" mode restarts pods to apply new resource requests. For stateful workloads like databases, a restart means downtime and potential data loss if not handled correctly. "Off" mode gives you recommendations without automatic action.

</details>

## Success Criteria

- [ ] All 8 workloads are classified with a primary scaling mechanism and a valid technical reason.
- [ ] The scaling flow trace (Part B) includes correct values for CPU thresholds, stabilization windows, and node provisioning times.
- [ ] Part C correctly identifies the HPA/VPA conflict on CPU and explains why VPA "Off" mode is safer for databases.
- [ ] You can explain the scaling flow: Traffic -> HPA -> Pending Pods -> Cluster Autoscaler -> New Nodes -> Pods Scheduled.
- [ ] You understand why KEDA is the right choice for queue-driven workloads (C and E).

## What You Should Understand After This Exercise

HPA, VPA, Cluster Autoscaler, and KEDA solve different layers of the same problem. HPA adjusts pod count, VPA adjusts pod size, Cluster Autoscaler adjusts node count, and KEDA reacts to external events. The right choice depends on the workload's bottleneck metric, its statefulness, and whether it can scale horizontally at all. In production, these mechanisms compose together -- HPA triggers pods, Cluster Autoscaler provides the nodes, and monitoring ensures you catch problems before users do.
