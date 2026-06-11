# Exercise 05: Production-Grade Auto-Scaling System

**Type:** Integration
**Time:** 45-60 minutes
**Difficulty:** Hard

## Objective

Combine HPA, KEDA, Cluster Autoscaler, and Prometheus monitoring into a complete, production-grade auto-scaling system for a real-world workload: an order processing pipeline with a web frontend, an API, and asynchronous workers.

## Scenario / Starting Point

You are building the auto-scaling infrastructure for a food delivery platform. The system has three components:

**Component 1: Web Frontend (`delivery-web`)**
- Serves the customer-facing web application
- Stateless, deployed as a Deployment
- Must maintain at least 3 replicas across 3 availability zones for HA
- Traffic peaks at lunch (11 AM - 2 PM) and dinner (5 PM - 9 PM)

**Component 2: Order API (`order-api`)**
- REST API that receives orders and publishes them to a Kafka topic
- Each pod opens database connections (bottleneck: connection pool, max 200 per pod)
- Must maintain at least 2 replicas for HA

**Component 3: Order Processor (`order-processor`)**
- Kafka consumer that processes orders from the `orders` topic
- Each pod processes approximately 100 orders/minute
- Can scale to zero when no orders are pending (late night)
- Needs a pre-warming cron trigger for meal rush hours

The cluster runs on AWS EKS with:
- A general-purpose node group: `m5.xlarge` (4 vCPU, 16 GB), min 2, max 10 nodes
- A compute-optimized node group: `c5.2xlarge` (8 vCPU, 16 GB), min 0, max 5 nodes (for burst capacity)

## Tasks

### Part A: Web Frontend HPA with Custom Metrics

Write a complete HPA manifest for `delivery-web` that:
- Scales between 3 and 40 replicas
- Uses CPU at 65% as primary metric
- Uses `http_requests_per_second` at 800 RPS per pod as secondary metric
- Implements fast scale-up (100% or 10 pods every 15s, stabilization 0s)
- Implements slow scale-down (10% every 60s, stabilization 300s)
- Includes an annotation linking to the team's runbook

```yaml
# delivery-web HPA
```

<details>
<summary>Hint 1</summary>

Use `autoscaling/v2` API version. Place annotations in `metadata.annotations`. The `behavior.scaleUp.selectPolicy: Max` ensures the more aggressive of the two policies (Percent or Pods) is used.

</details>

### Part B: Order API HPA with Connection-Based Scaling

Write a complete HPA manifest for `order-api` that:
- Scales between 2 and 20 replicas
- Uses `active_db_connections` at 150 per pod as primary metric (leaving 50 connections headroom from the 200 hard limit)
- Uses CPU at 70% as secondary metric
- Implements conservative scale-down (stabilization 600s, max 5% every 120s)
- Includes a custom metric for p99 latency: `http_request_duration_p99_seconds` with a target of 0.3 (300ms)

```yaml
# order-api HPA
```

<details>
<summary>Hint 2</summary>

Three metrics means the HPA calculates desired replicas for each independently and picks the highest. This ensures the API never exceeds connection limits, never exceeds latency SLOs, and never starves for CPU -- whichever constraint is hit first triggers scaling.

</details>

### Part C: Order Processor KEDA ScaledObject

Write a KEDA `ScaledObject` for `order-processor` that:
- Targets the `order-processor` Deployment
- Scales from 0 to 30 replicas
- Primary trigger: Kafka topic `orders` with `lagThreshold: 100` (one pod per 100 messages of lag)
- Secondary trigger: Cron-based pre-warming for lunch rush (scale to 10 replicas between 10:30 AM and 2:30 PM EST)
- Secondary trigger: Cron-based pre-warming for dinner rush (scale to 15 replicas between 4:30 PM and 9:30 PM EST)
- Polling interval: 15 seconds
- Cooldown period: 300 seconds
- Also write the `TriggerAuthentication` resource needed for Kafka SASL credentials (stored in a Secret named `kafka-credentials`)

```yaml
# KEDA ScaledObject and TriggerAuthentication
```

<details>
<summary>Hint 3</summary>

KEDA supports multiple triggers -- the one requesting the highest replica count wins. For Kafka, use `type: kafka` with `bootstrapServers`, `consumerGroup`, `topic`, and `lagThreshold`. The `TriggerAuthentication` resource references a Secret and maps its keys to the trigger's authentication parameters.

</details>

### Part D: Cluster Autoscaler Configuration

Write the Cluster Autoscaler deployment command flags that:
1. Manages both node groups (general-purpose and compute-optimized)
2. Uses `least-waste` expander to minimize resource waste
3. Sets scale-down utilization threshold to 50%
4. Waits 10 minutes after a scale-up before considering scale-down
5. Enables `--balance-similar-node-groups` for multi-AZ balance
6. Sets `--max-graceful-termination-sec=600` to allow 10 minutes for pod eviction

Additionally, write the Kubernetes annotations needed on the ASG (node group) to enable autodiscovery.

```bash
# Cluster Autoscaler flags
```

```yaml
# ASG tags for autodiscovery
```

<details>
<summary>Hint 4</summary>

ASG autodiscovery requires two tags on the Auto Scaling Group:
- `k8s.io/cluster-autoscaler/enabled`: `owned`
- `k8s.io/cluster-autoscaler/<cluster-name>`: `owned`

The `--node-group-auto-discovery` flag uses these tags to find node groups.

</details>

### Part E: Prometheus Monitoring and Alerts

Write Prometheus alert rules for the auto-scaling system. Create a `PrometheusRule` resource with alerts for:

1. **HPAAtMaxCapacity** -- fires when any HPA has been at max replicas for more than 10 minutes
2. **HPAScaleUpStalled** -- fires when HPA wants to scale up but is limited (no node capacity) for more than 5 minutes
3. **HPAFlapping** -- fires when an HPA has scaled more than 6 times in the last 30 minutes
4. **KEDAScalerError** -- fires when KEDA has been unable to poll the scaler for more than 5 minutes
5. **ClusterAutoscalerUnschedulable** -- fires when pods have been unschedulable for more than 5 minutes despite Cluster Autoscaler running

Also write a PromQL query for a Grafana panel that shows HPA replica count vs. max replicas over time.

```yaml
# PrometheusRule
```

```promql
# Grafana queries
```

<details>
<summary>Hint 5</summary>

Key metrics:
- `kube_horizontalpodautoscaler_status_current_replicas`
- `kube_horizontalpodautoscaler_spec_max_replicas`
- `kube_horizontalpodautoscaler_status_condition{condition="ScalingLimited"}`
- `changes(kube_horizontalpodautoscaler_status_current_replicas[30m])`
- `kube_pod_status_unschedulable` or `cluster_autoscaler_unschedulable_pods_count`

</details>

### Part F: End-to-End Validation Script

Write a bash script that validates the entire auto-scaling system is working correctly. The script should:

1. Verify all HPAs exist and are attached to their target Deployments
2. Verify KEDA ScaledObject exists and references the correct Deployment
3. Check that Cluster Autoscaler pods are running
4. Query Prometheus for current HPA metrics
5. Output a summary table of all autoscaling resources and their current state

```bash
#!/usr/bin/env bash
# validate-autoscaling.sh
```

<details>
<summary>Hint 6</summary>

Use `kubectl get hpa -A -o json` and `jq` to extract status. Check `kubectl get scaledobject -A` for KEDA resources. Check `kubectl get pods -n kube-system -l app=cluster-autoscaler` for Cluster Autoscaler health.

</details>

## Success Criteria

- [ ] Web frontend HPA uses two metrics (CPU + RPS) with asymmetric scaling policies.
- [ ] Order API HPA uses three metrics (connections + CPU + latency) with connection headroom below the hard limit.
- [ ] KEDA ScaledObject enables scale-to-zero with Kafka lag trigger and cron pre-warming triggers for both meal rushes.
- [ ] Cluster Autoscaler configuration manages both node groups with appropriate scale-down delays.
- [ ] Prometheus alerts cover all failure modes: max capacity, stalled scaling, flapping, KEDA errors, and unschedulable pods.
- [ ] Validation script checks all components and outputs a summary.

## What You Should Understand After This Exercise

A production auto-scaling system is not a single HPA -- it is a coordinated set of mechanisms that work together. HPA handles pod-level scaling, KEDA handles event-driven scaling (including scale-to-zero), Cluster Autoscaler handles node-level scaling, and Prometheus monitoring catches problems before they become incidents. The key design principles are: scale up fast (users are waiting), scale down slow (avoid flapping), always leave headroom on finite resources (connections, memory), and always have alerts for when the system cannot scale further. Each component has its own failure mode, and your monitoring must cover all of them.
