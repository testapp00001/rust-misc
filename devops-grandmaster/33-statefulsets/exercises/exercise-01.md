# Exercise 01: StatefulSet vs Deployment for Stateful Workloads

## Objective

Understand the fundamental differences between StatefulSets and Deployments, and be able to choose the correct workload type for stateful applications like databases, message queues, and distributed systems.

## Background

Kubernetes provides several workload types. Deployments are the default choice for most applications, but they break down when applied to stateful workloads. StatefulSets exist specifically to solve the problems that arise when running databases and similar services in Kubernetes.

This exercise tests your conceptual understanding of why StatefulSets exist and when to use them.

## Instructions

### Part 1: Core Concept Questions

Answer the following questions in your own words:

1. What are the three guarantees that a StatefulSet provides that a Deployment does not?

2. Why does a Deployment's random pod naming break database replication?

3. What is a headless Service, and why is it required for StatefulSets?

4. How does `volumeClaimTemplates` differ from a regular `volumes` section with a PVC reference in a Deployment?

5. When a StatefulSet pod is deleted and recreated, what three things remain the same?

### Part 2: Scenario Analysis

For each scenario below, explain whether you should use a StatefulSet or a Deployment, and why.

**Scenario A**: A PostgreSQL database with one primary and two read replicas. Replicas connect to the primary by hostname to stream WAL logs.

**Scenario B**: A Node.js API server that connects to an external managed database (RDS). It stores no local state.

**Scenario C**: A Kafka cluster with 3 brokers. Each broker needs its own storage and a stable identity so producers and consumers can find it.

**Scenario D**: A Redis single-instance cache used by a web application. Data is ephemeral and can be lost on restart.

**Scenario E**: An etcd cluster used as Kubernetes' backing store. It requires consensus between members and stable peer addresses.

**Scenario F**: A static file server (nginx) serving pre-built assets from a ConfigMap.

### Part 3: Failure Behavior

Explain what happens in each scenario:

1. You deploy a PostgreSQL database as a Deployment with `replicas: 3`. A node crashes. What happens to the data on that node's pod?

2. You deploy the same PostgreSQL database as a StatefulSet with `replicas: 3`. A node crashes. What happens to the data?

3. You scale a StatefulSet from 3 to 1 replica. Which pod remains? What happens to the PVCs of the removed pods?

4. You scale a Deployment from 3 to 1 replica. Which pod remains? What happens to the data?

## Success Criteria

- [ ] You can name the three StatefulSet guarantees (stable identity, ordered operations, per-pod storage)
- [ ] You can correctly identify the appropriate workload type for all 6 scenarios
- [ ] You understand why Deployments fail for databases (random names, random scaling, shared storage)
- [ ] You can explain what happens to PVCs when StatefulSet pods are scaled down
- [ ] You understand the role of the headless Service in providing DNS

## Hints

<details>
<summary>Hint 1: The Three Guarantees</summary>

Think about what a database needs that a web server does not:
- A name that clients can find and remember (identity)
- Things starting in a specific order (ordering)
- Data that belongs to one specific instance (storage)
</details>

<details>
<summary>Hint 2: Headless Service DNS</summary>

A normal ClusterIP Service gives you one IP that load-balances across pods. A headless Service (`clusterIP: None`) gives you individual DNS entries for each pod:

```
Regular Service:    myservice.default.svc.cluster.local → one VIP
Headless Service:   pod-0.myservice.default.svc.cluster.local → pod-0's IP
                    pod-1.myservice.default.svc.cluster.local → pod-1's IP
```

This is how clients find a specific pod by its stable identity.
</details>

<details>
<summary>Hint 3: PVC Lifecycle in StatefulSets</summary>

When you scale a StatefulSet down:
- Pods are deleted in reverse order (highest ordinal first)
- PVCs are NOT deleted (data is preserved)
- When you scale back up, pods reattach to their existing PVCs

This is different from Deployments where PVCs can be shared or orphaned.
</details>

<details>
<summary>Hint 4: Scenario D - Redis Single Instance</summary>

A single Redis instance used as a cache is stateless from the application's perspective - the app can survive Redis losing all data. You do not need stable identity or per-pod storage. A Deployment with a PVC (or even no PVC if data loss is acceptable) works fine.

StatefulSets are for when each instance has a DISTINCT role. A single-instance cache has no peers to coordinate with.
</details>

## Concepts to Review

After completing this exercise, you should understand:

- Why Deployments are insufficient for stateful workloads
- The three StatefulSet guarantees and what problems they solve
- When a Deployment is the right choice even for stateful-looking workloads
- How headless Services enable stable network identity
- How volumeClaimTemplates provide per-pod persistent storage
