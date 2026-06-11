# Exercise 01: Blue-Green vs Rolling Update

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

Compare blue-green deployment with Kubernetes rolling updates by analyzing
failure scenarios. This exercise trains you to choose the right deployment
strategy based on risk tolerance, rollback speed, and resource constraints.

## Scenario

Your team runs a payment processing service with 6 pods in Kubernetes. The
service handles 1,000 requests per second. You are deciding between a
rolling update and a blue-green deployment for the next release.

The current deployment uses:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: payment-service
spec:
  replicas: 6
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 2
      maxUnavailable: 2
  template:
    spec:
      containers:
        - name: payment-service
          image: payment:v1
```

## Tasks

### Part A: Analyze the Rolling Update Failure

During a rolling update to `payment:v2`, the new version crashes on startup
with an out-of-memory error. With `maxSurge: 2` and `maxUnavailable: 2`,
describe exactly what happens to the cluster capacity during this failure.
How many healthy pods are serving traffic at each step?

<details>
<summary>Hint</summary>

With `maxSurge: 2`, Kubernetes creates up to 2 extra pods (8 total). With
`maxUnavailable: 2`, Kubernetes can have up to 2 pods unavailable. Trace
the sequence: kill old pod -> start new pod -> new pod crashes -> what
happens next?

</details>

### Part B: Compare with Blue-Green

Describe the same failure scenario (v2 crashes on startup) using a
blue-green deployment. How many healthy pods serve traffic? How quickly
can you roll back?

<details>
<summary>Hint</summary>

In blue-green, v2 is deployed to the idle (green) environment. No traffic
goes to green until you switch. If v2 crashes, you simply do not switch.
Traffic stays on the stable (blue) environment with zero impact.

</details>

### Part C: Resource Cost Analysis

Blue-green requires double the resources during deployment. Calculate the
resource cost for both strategies during a deployment, assuming each pod
requests 500m CPU and 512Mi memory.

<details>
<summary>Hint</summary>

Rolling update with `maxSurge: 2` needs 8 pods temporarily (6 + 2 surge).
Blue-green needs 12 pods temporarily (6 blue + 6 green). Calculate the
total CPU and memory for each case.

</details>

### Part D: Choose the Right Strategy

For each scenario below, choose the better deployment strategy and explain
why:

1. A stateless API with no database, handling 100 req/s, deployed on a
   cost-sensitive small cluster
2. A payment service handling 10,000 req/s where downtime costs $10,000/minute
3. A batch processing job that runs once per hour
4. A real-time trading system where any error during deployment could cause
   financial loss

<details>
<summary>Hint</summary>

Consider the trade-off between resource cost and risk tolerance. Blue-green
is worth the extra resources when the cost of downtime exceeds the cost of
double resources.

</details>

## Success Criteria

- [ ] You can trace a rolling update failure and explain pod capacity at each step
- [ ] You can explain why blue-green has zero user impact during failed deployments
- [ ] You can calculate resource costs for both strategies
- [ ] You can choose the right strategy for different scenarios with justification
- [ ] You understand the trade-off: resource cost vs deployment risk

## What You Should Understand After This Exercise

Blue-green deployment trades resource cost (double the pods) for safety
(zero user impact during deployment). Rolling updates trade safety for
resource efficiency. The right choice depends on the cost of downtime
relative to the cost of infrastructure.
