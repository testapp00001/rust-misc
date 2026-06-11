# Exercise 04: Zero-Downtime Deployment Strategies

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Design and implement zero-downtime deployment strategies for different application scenarios. You will choose the correct deployment strategy for each scenario, write the Deployment manifests, and explain why each strategy is appropriate. This exercise requires you to reason about application characteristics, not just follow instructions.

## Background

Kubernetes supports two built-in deployment strategies: `RollingUpdate` and `Recreate`. But within `RollingUpdate`, the `maxSurge` and `maxUnavailable` parameters give you significant control over the update behavior. Additionally, you can implement canary and blue-green patterns using multiple Deployments. Choosing the right strategy depends on your application's tolerance for running two versions simultaneously, its startup time, and its resource constraints.

## Tasks

### Part A: Strategy Selection

For each application scenario below, select the appropriate deployment strategy and explain why. Be specific about `maxSurge` and `maxUnavailable` values if you choose `RollingUpdate`.

| Scenario | Strategy | maxSurge | maxUnavailable | Why |
|----------|----------|----------|----------------|-----|
| 1. Stateless REST API serving 1000 req/s with no backward-compatible database migrations | | | | |
| 2. Batch processor that reads from a queue and must not run two instances simultaneously (risk of duplicate processing) | | | | |
| 3. Critical payment service that must never drop below 5 available replicas | | | | |
| 4. Internal admin dashboard with 2 users, running on a resource-constrained cluster | | | | |
| 5. Application that performs a database schema migration on startup that breaks the old version | | | | |

<details>
<summary>Hint -- When to Use Recreate</summary>
Use the `Recreate` strategy when your application cannot tolerate two versions running simultaneously. This includes cases where the new version makes breaking changes to shared resources (database schema, shared file system) or when duplicate processing would cause data corruption.
</details>

<details>
<summary>Hint -- maxSurge and maxUnavailable Trade-offs</summary>
Higher `maxSurge` means faster rollouts but more resource usage (extra Pods during the update). Lower `maxUnavailable` means safer rollouts but slower (fewer Pods replaced at once). For critical services, set `maxUnavailable: 0` to maintain full capacity. For resource-constrained environments, you might accept some unavailability.
</details>

### Part B: Implement a Canary Deployment

You have an application `myapp` currently running version 1.0.0 with 9 replicas. You want to test version 2.0.0 with only 10% of traffic before rolling it out to everyone.

Create two Deployment manifests:

1. `canary-stable.yaml` -- The stable version (v1.0.0) receiving 90% of traffic
2. `canary-new.yaml` -- The canary version (v2.0.0) receiving 10% of traffic

Requirements:
- Both Deployments must have the label `app: myapp` (so a single Service can select both)
- Each Deployment must have a unique additional label to distinguish them
- The stable Deployment should have 9 replicas
- The canary Deployment should have 1 replica
- Both should have resource requests and limits
- Both should have readiness probes

<details>
<summary>Hint -- Service Selection</summary>
A Kubernetes Service distributes traffic to all Pods matching its selector. If you create a Service with `selector: { app: myapp }`, it will send traffic to Pods from both Deployments. The traffic split is proportional to the number of ready Pods: 9/10 to stable, 1/10 to canary. This is the simplest form of canary deployment without a service mesh.
</details>

Apply both manifests:

```bash
kubectl apply -f canary-stable.yaml
kubectl apply -f canary-new.yaml
```

Verify:

```bash
kubectl get pods -l app=myapp --show-labels
kubectl get replicasets -l app=myapp
```

### Part C: Promote or Rollback the Canary

After testing the canary version, you decide it is safe to promote. Describe the steps you would take to:

1. Promote the canary to full production (replace stable with the new version)
2. Rollback the canary if something went wrong (remove canary and keep stable)

For each case, write the kubectl commands and explain what happens to the Pods and ReplicaSets.

<details>
<summary>Hint -- Promotion</summary>
To promote, update the stable Deployment's image to the canary version and scale it back to the desired count. Then delete the canary Deployment. Alternatively, if you are confident, you can update the stable Deployment and delete the canary in one step.
</details>

### Part D: Implement a Blue-Green Deployment

You need a blue-green deployment for an application where you cannot run two versions simultaneously, but you also cannot afford downtime. Create the manifests for:

1. `blue.yaml` -- The current version (v1), labeled `version: blue`
2. `green.yaml` -- The new version (v2), labeled `version: green`

Both Deployments should:
- Be named `app-blue` and `app-green`
- Have 3 replicas
- Use the label `app: myapp` for Service selection
- Include readiness probes

Write the Service manifest that initially points to the blue deployment. Then describe the exact command to switch traffic to green.

<details>
<summary>Hint -- Service Selector</summary>
The Service selector determines which Pods receive traffic. To switch from blue to green, update the Service selector from `version: blue` to `version: green`. This is an atomic operation -- all traffic switches at once. The blue Deployment can be kept running for quick rollback.
</details>

### Part E: Compare Strategies

Fill in the comparison table:

| Aspect | RollingUpdate | Canary | Blue-Green |
|--------|---------------|--------|------------|
| Downtime | | | |
| Resource overhead during deployment | | | |
| Rollback speed | | | |
| Traffic control granularity | | | |
| Complexity | | | |
| Use case | | | |

## Success Criteria

- [ ] You can select the correct deployment strategy for each scenario with clear reasoning
- [ ] You created valid canary Deployment manifests with correct labels and replica counts
- [ ] You can describe promotion and rollback steps for a canary deployment
- [ ] You created valid blue-green Deployment manifests with a Service selector switch
- [ ] You can compare the trade-offs of each strategy

## What You Should Understand After This Exercise

After completing this exercise, you should be able to choose and implement the right deployment strategy for any application scenario. You should understand that "zero downtime" does not mean "zero risk" -- each strategy has trade-offs in resource usage, rollback speed, and complexity. The ability to reason about these trade-offs is what separates a Kubernetes user from a Kubernetes operator.
