# Exercise 01: Architect the Unbreakable System

**Type:** Conceptual
**Time:** 30 minutes
**Difficulty:** Medium

## Objective

Design the complete architecture for a system that achieves five nines
(99.999%) availability -- no more than 5.26 minutes of downtime per year.
This exercise forces you to think holistically about every layer of the
stack, from DNS to database, and to reason about failure modes before
they happen.

## Scenario

You are the principal engineer at a fintech startup that processes
real-time payments. The system must:

- Handle 50,000 requests per second at peak
- Process payments with sub-200ms p99 latency
- Survive the loss of any single data center
- Recover from catastrophic failure within 60 seconds
- Detect and mitigate DDoS attacks automatically
- Maintain zero downtime during deployments

The current system runs on a single Kubernetes cluster in one AWS region.
It has a PostgreSQL database with no replicas, no observability stack,
and deployments require a maintenance window.

```
Current State:

    Users
      |
      v
  [Single ALB]
      |
      v
  [K8s Cluster - us-east-1]
      |
      v
  [PostgreSQL - Single Instance]
```

## Tasks

### Part A: Design the Target Architecture

Draw an architecture diagram that achieves all six requirements listed
above. Your diagram must show:

1. How traffic enters the system (DNS, CDN, load balancers)
2. How the application is deployed (multi-region, multi-cluster)
3. How the database achieves high availability
4. How deployments happen without downtime
5. How DDoS mitigation works at each layer

Use ASCII art or describe each component and its connections clearly.

<details>
<summary>Hint</summary>

Think in layers: edge (DNS/CDN/WAF), application (multi-region K8s),
data (database replication), and cross-cutting (observability, CI/CD).
Each layer must be redundant. No single component should have a single
point of failure.

</details>

### Part B: Identify Every Single Point of Failure

List every component in the current architecture that is a single point
of failure. For each one, explain:

1. What happens when it fails
2. How long the system is down
3. What data is lost (if any)

<details>
<summary>Hint</summary>

Look beyond the obvious. The database is a SPOF, but so is the single
ALB, the single region, the single K8s control plane, and even the
single deployment pipeline. Think about what happens when each one
disappears.

</details>

### Part C: Define the Failure Domain Hierarchy

Organize your redundancy into failure domains. Draw a hierarchy showing:

- Process failures (handled by what?)
- Machine failures (handled by what?)
- Rack failures (handled by what?)
- Availability Zone failures (handled by what?)
- Region failures (handled by what?)
- Cloud provider failures (handled by what?)

For each level, specify the detection time, failover time, and mechanism.

<details>
<summary>Hint</summary>

Kubernetes handles process and machine failures with ReplicaSets and
node auto-replacement. AZ failures need multi-AZ deployments. Region
failures need global load balancing (Route53, Cloudflare). Cloud
provider failures are the hardest -- they require multi-cloud or a
hot standby in another provider.

</details>

### Part D: The CAP Theorem Trade-off

A payment system must be consistent -- you cannot double-charge a
customer because of a network partition. Explain:

1. How does your architecture handle the CAP theorem trade-off?
2. What happens to availability during a network partition between regions?
3. What is your consistency model for payment data vs. for analytics data?
4. How do you handle split-brain in the database layer?

<details>
<summary>Hint</summary>

For payments, choose CP (consistency over availability) during partitions.
Use synchronous replication within a region (strong consistency) and
asynchronous replication across regions (eventual consistency with
conflict resolution). Analytics data can tolerate eventual consistency.
Split-brain requires a quorum-based leader election or an external
arbiter.

</details>

### Part E: Estimate the Cost of "Unbreakable"

Five nines is expensive. Estimate (order of magnitude is fine) the
monthly cost difference between:

1. The current single-instance setup
2. A multi-AZ setup within one region
3. A multi-region active-active setup
4. A multi-cloud active-active setup

At what point does the cost of availability exceed the cost of downtime?
Write the formula.

<details>
<summary>Hint</summary>

Cost of downtime = (revenue per hour) * (downtime hours per year).
Compare this against the infrastructure cost increase. Most companies
target three nines (99.9%) because the jump from four nines to five
nines costs 10-100x more but only buys 47 fewer minutes of downtime
per year.

</details>

## Success Criteria

- [ ] Architecture diagram shows at least 3 regions with active-active or active-passive failover
- [ ] Every component in the diagram has at least 2 replicas
- [ ] The failure domain hierarchy covers all 6 levels with specific mechanisms
- [ ] The CAP theorem analysis correctly identifies the consistency model for payment vs. analytics data
- [ ] The cost analysis includes a formula and acknowledges the diminishing returns of higher availability

## What You Should Understand After This Exercise

An unbreakable system is not built by making any single component
unbreakable -- it is built by assuming every component will break and
designing the system to survive it. The architecture must be layered,
with each layer providing redundancy for the layer below it. The
hardest part is not the technology; it is deciding how much availability
you can afford and where to draw the line between resilience and cost.
