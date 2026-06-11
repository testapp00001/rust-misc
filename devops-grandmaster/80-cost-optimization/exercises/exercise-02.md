# Exercise 02: Right-Sizing Kubernetes Workloads

**Type:** Guided | **Time:** 30 min | **Difficulty:** Easy-Medium

## Objective

Analyze Kubernetes pod resource requests versus actual usage using Prometheus queries,
identify over-provisioned workloads, and generate right-sizing recommendations.

## Background

Your Kubernetes cluster has been running for 6 months. The initial resource requests and
limits were set based on developer estimates, and nobody has reviewed them since. The
cluster is running on 3x m5.2xlarge nodes (8 vCPU, 32 GB RAM each), and you are seeing
high costs but only moderate CPU/memory utilization from the cloud provider metrics.

## Current Workload Inventory

| Deployment | Namespace | Replicas | CPU Request | CPU Limit | Mem Request | Mem Limit |
|-----------|-----------|----------|-------------|-----------|-------------|-----------|
| web-frontend | production | 4 | 500m | 1000m | 512Mi | 1Gi |
| api-server | production | 4 | 1000m | 2000m | 1Gi | 2Gi |
| payment-service | production | 3 | 1000m | 2000m | 1Gi | 2Gi |
| auth-service | production | 3 | 500m | 1000m | 512Mi | 1Gi |
| order-service | production | 4 | 1000m | 2000m | 1Gi | 2Gi |
| notification-svc | production | 2 | 500m | 1000m | 512Mi | 1Gi |
| cron-worker | batch | 2 | 2000m | 4000m | 2Gi | 4Gi |
| data-pipeline | batch | 3 | 2000m | 4000m | 4Gi | 8Gi |
| redis | databases | 3 | 500m | 1000m | 1Gi | 2Gi |
| postgres | databases | 2 | 2000m | 4000m | 4Gi | 8Gi |

**Total Requested:** 36.5 vCPU, 52.5 GiB Memory
**Node Capacity:** 24 vCPU, 96 GiB Memory (3 nodes)
**Status:** Cluster is over-committed on CPU but actual usage is low.

## Tasks

### Task 1: Prometheus Queries for Actual Usage

Write Prometheus queries to measure actual CPU and memory usage for each workload. You
need two sets of queries: one for average usage and one for 95th percentile (P95) usage.

Fill in the PromQL queries below:

```
# Average CPU usage per deployment (last 7 days)
avg by (deployment) (
  rate(container_cpu_usage_seconds_total{namespace=~"production|batch|databases"}[7d])
  * 1000
)

# P95 CPU usage per deployment (last 7 days)
# YOUR QUERY HERE

# Average memory usage per deployment (last 7 days)
# YOUR QUERY HERE

# P95 memory usage per deployment (last 7 days)
# YOUR QUERY HERE
```

### Task 2: Analyze the Results

After running your queries, you collected these results:

| Deployment | CPU Avg | CPU P95 | Mem Avg | Mem P95 |
|-----------|---------|---------|---------|---------|
| web-frontend | 80m | 200m | 180Mi | 350Mi |
| api-server | 150m | 450m | 400Mi | 750Mi |
| payment-service | 120m | 380m | 350Mi | 650Mi |
| auth-service | 60m | 150m | 150Mi | 300Mi |
| order-service | 200m | 600m | 500Mi | 900Mi |
| notification-svc | 40m | 100m | 120Mi | 250Mi |
| cron-worker | 1800m | 3500m | 2.5Gi | 3.8Gi |
| data-pipeline | 1600m | 3200m | 3Gi | 5.5Gi |
| redis | 100m | 250m | 600Mi | 900Mi |
| postgres | 800m | 2000m | 2Gi | 3.5Gi |

For each deployment, calculate:
1. **CPU Request Utilization %** = (CPU P95 / CPU Request) x 100
2. **Memory Request Utilization %** = (Mem P95 / Mem Request) x 100
3. **Right-sizing recommendation** (Keep, Reduce, or Increase)

Present your analysis as a table:

| Deployment | CPU Util % | Mem Util % | CPU Action | Mem Action | Notes |
|-----------|------------|------------|------------|------------|-------|
| web-frontend | ? | ? | ? | ? | ? |
| api-server | ? | ? | ? | ? | ? |
| ... | | | | | |

### Task 3: Generate Right-Sizing Recommendations

For each deployment, write new resource requests and limits. Use this policy:
- **CPU Request:** Set to P95 usage x 1.25 (25% headroom)
- **CPU Limit:** Set to P95 usage x 2.0 (allow bursting)
- **Memory Request:** Set to P95 usage x 1.20 (20% headroom)
- **Memory Limit:** Set to P95 usage x 1.50 (50% headroom)

Round to the nearest standard unit (e.g., 100m, 250m, 500m, 1000m for CPU; 64Mi, 128Mi,
256Mi, 512Mi, 1Gi for memory).

Write the updated resource blocks in YAML format for at least 3 deployments:

```yaml
# Example for web-frontend (fill in the values)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-frontend
  namespace: production
spec:
  template:
    spec:
      containers:
        - name: web-frontend
          resources:
            requests:
              cpu: ???
              memory: ???
            limits:
              cpu: ???
              memory: ???
```

### Task 4: Calculate Savings

After right-sizing, calculate the new total resource requests and compare with the
original. How many nodes can you potentially remove?

```
Original Total CPU Request:    36.5 vCPU
Original Total Memory Request: 52.5 GiB

New Total CPU Request:         ??? vCPU
New Total Memory Request:      ??? GiB

CPU Reduction:                 ??? %
Memory Reduction:              ??? %

Nodes Needed Before: 3x m5.2xlarge ($???/month)
Nodes Needed After:  ???x m5.2xlarge ($???/month)
Monthly Savings:     $???
```

Assume an m5.2xlarge costs ~$0.384/hr On-Demand (~$276.48/month).

<details>
<summary>Hint 1: PromQL for P95</summary>

Use the `quantile` function:
```
quantile(0.95, rate(container_cpu_usage_seconds_total{...}[7d])) * 1000
```
For memory, use `container_memory_working_set_bytes`.

</details>

<details>
<summary>Hint 2: Identifying Over-Provisioned Workloads</summary>

A workload is over-provisioned if:
- CPU utilization < 50% of request (headroom > 2x actual usage)
- Memory utilization < 60% of request

These thresholds indicate the request can be safely reduced.

</details>

<details>
<summary>Hint 3: Node Calculation</summary>

Each m5.2xlarge provides 8 vCPU and 32 GiB RAM. Calculate how many pods can fit:
- Sum all CPU requests, divide by 8 = nodes needed by CPU
- Sum all memory requests, divide by 32 = nodes needed by memory
- Take the higher of the two

</details>

## Verification

After completing this exercise, you should have:
- PromQL queries for CPU and memory usage at average and P95
- A utilization analysis table for all 10 deployments
- Updated YAML resource blocks for at least 3 deployments
- A savings calculation showing potential node reduction
- Target: at least 40% reduction in total resource requests

## Reflection Questions

1. Why do we use P95 instead of average for right-sizing?
2. What is the risk of setting CPU limits too low?
3. How would you implement automated right-sizing recommendations (e.g., using the
   Vertical Pod Autoscaler)?
