# Exercise 03: Spot Instance Strategy

**Type:** Independent | **Time:** 30 min | **Difficulty:** Medium

## Objective

Design a spot instance strategy for a mixed workload environment running on Kubernetes,
including tolerations, affinity rules, graceful shutdown handling, and interruption
management.

## Background

Your organization runs three types of workloads on Kubernetes (AWS EKS):

1. **Web Application** (stateless, high-availability required)
   - 6 replicas of a Go web server
   - Must handle 99.9% uptime SLA
   - Scales between 6-20 replicas based on traffic

2. **Batch Processing** (interruptible, checkpointing enabled)
   - Nightly ETL jobs that process 500 GB of data
   - Jobs run between 11 PM and 5 AM
   - Checkpointing every 10 minutes; can resume from last checkpoint

3. **CI/CD Pipeline** (short-lived, interruptible)
   - Build jobs take 5-15 minutes each
   - ~100 builds per day during business hours
   - Failed builds can be retried automatically

Current setup: all workloads run on On-Demand m5.xlarge instances. Monthly compute cost
is $12,000. The goal is to reduce this by 50% using spot instances where appropriate.

## Tasks

### Task 1: Workload Classification

Classify each workload by spot suitability. For each, explain your reasoning.

| Workload | Spot Suitable? | Risk Tolerance | Justification |
|----------|---------------|----------------|---------------|
| Web Application | ??? | ??? | ??? |
| Batch Processing | ??? | ??? | ??? |
| CI/CD Pipeline | ??? | ??? | ??? |

Then decide what percentage of each workload should run on spot vs on-demand:

| Workload | On-Demand % | Spot % | Reasoning |
|----------|-------------|--------|-----------|
| Web Application | ??? | ??? | ??? |
| Batch Processing | ??? | ??? | ??? |
| CI/CD Pipeline | ??? | ??? | ??? |

### Task 2: Spot Node Pool Configuration

Write the Terraform or AWS CLI configuration for a spot node group. The node group
should:
- Use multiple instance types (at least 4) for availability
- Use `capacity_rebalance` to replace unhealthy nodes
- Set appropriate labels and taints

```hcl
# Write your Terraform resource for a spot node group
resource "aws_eks_node_group" "spot" {
  # Fill in the configuration
  # Hint: use instance_types list, capacity_type = "SPOT"
  # Add labels and taints for spot workloads
}
```

### Task 3: Kubernetes Scheduling Configuration

Write the pod specs for each workload type. You need to configure:

**For Batch Processing (should run on spot):**
- Toleration for spot taint
- Node affinity for spot nodes
- Pod disruption budget
- Graceful shutdown hooks

```yaml
# Write the complete pod spec for a batch processing job
apiVersion: batch/v1
kind: Job
metadata:
  name: etl-job
  namespace: batch
spec:
  template:
    spec:
      # YOUR CONFIGURATION HERE
      # Include: tolerations, affinity, terminationGracePeriodSeconds,
      # lifecycle preStop hook
      containers:
        - name: etl-worker
          image: acme/etl-worker:v1.5
          # YOUR RESOURCE CONFIG HERE
```

**For Web Application (mix of on-demand and spot):**
- Use topology spread constraints to distribute across on-demand and spot
- Ensure at least 2 replicas always run on on-demand nodes
- Configure Pod Disruption Budget

```yaml
# Write the deployment spec for the web application
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-app
  namespace: production
spec:
  replicas: 6
  template:
    spec:
      # YOUR CONFIGURATION HERE
      # Include: topology spread constraints, PDB, graceful shutdown
```

**For CI/CD (should prefer spot, retry on failure):**
- Run on spot nodes
- Configure automatic retry on spot interruption
- Set active deadline

```yaml
# Write the Job spec for CI/CD build jobs
apiVersion: batch/v1
kind: Job
metadata:
  name: ci-build
  namespace: cicd
spec:
  backoffLimit: 3
  template:
    spec:
      # YOUR CONFIGURATION HERE
```

### Task 4: Interruption Handling

Write a Kubernetes manifest for a Spot Interruption Handler DaemonSet. This DaemonSet
should:
- Run on every spot node
- Watch for the EC2 instance metadata interruption warning (2-minute notice)
- Cordon and drain the node before termination
- Send a notification (webhook or event)

```yaml
# Write the DaemonSet for spot interruption handling
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: spot-interruption-handler
  namespace: kube-system
spec:
  # YOUR CONFIGURATION HERE
```

Also, explain how the following tools can help with spot interruption handling:
- **AWS Node Termination Handler** (what does it do?)
- **Karpenter** (how does it differ from Cluster Autoscaler for spot?)
- **Spot.io / Ocean** (what value does it add?)

<details>
<summary>Hint 1: Spot Taints and Tolerations</spot>

AWS EKS spot node groups typically have this taint:
```
taint:
  - key: kubernetes.azure.com/scalesetpriority  # AKS
  - key: eks.amazonaws.com/capacityType          # EKS label (not a taint)
```

For EKS, use node affinity with `eks.amazonaws.com/capacityType=SPOT` label rather than
taints, or add a custom taint when creating the node group.

</details>

<details>
<summary>Hint 2: Topology Spread Constraints</summary>

To distribute pods across on-demand and spot nodes:
```yaml
topologySpreadConstraints:
  - maxSkew: 1
    topologyKey: eks.amazonaws.com/capacityType
    whenUnsatisfiable: DoNotSchedule
    labelSelector:
      matchLabels:
        app: web-app
```

</details>

<details>
<summary>Hint 3: Graceful Shutdown</summary>

Key configurations for graceful shutdown:
```yaml
terminationGracePeriodSeconds: 60
containers:
  - lifecycle:
      preStop:
        exec:
          command: ["/bin/sh", "-c", "sleep 15"]
    # In your app: handle SIGTERM, stop accepting new requests,
    # finish in-flight requests, flush buffers
```
The `preStop` sleep gives the pod time to deregister from the load balancer before
shutting down.

</details>

## Verification

After completing this exercise, you should have:
- A workload classification table with spot suitability analysis
- Terraform or AWS CLI config for spot node groups
- Kubernetes YAML for batch, web, and CI/CD workloads with spot scheduling
- A DaemonSet or description for spot interruption handling
- An explanation of how each workload handles spot interruptions
- Estimated cost savings (target: 50% reduction from $12,000/month)

## Reflection Questions

1. What happens if spot capacity is unavailable for all your chosen instance types?
2. How does the "2-minute warning" from AWS affect your application architecture?
3. When should you NOT use spot instances, even if the workload is technically interruptible?
