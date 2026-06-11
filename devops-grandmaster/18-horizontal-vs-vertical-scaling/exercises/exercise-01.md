# Exercise 01: Scaling Trade-Offs

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

Analyze real-world scenarios and determine whether horizontal or vertical scaling is the appropriate choice, articulating the trade-offs behind each decision.

## Background

Your team runs several different services. Each service has different characteristics: some are stateless API servers, some are databases, some do heavy computation. Leadership wants a scaling strategy document that explains which approach to use for each service and why.

You need to understand not just *what* to choose, but *why* -- because the wrong choice wastes money or creates an operational ceiling you cannot break through.

## Tasks

### Part A: Classify Each Scenario

For each scenario below, state whether you would choose **horizontal scaling**, **vertical scaling**, or **both** (and in what order). Explain your reasoning in 2-3 sentences.

1. A PostgreSQL database that is running out of memory and slow on complex joins.
2. A stateless REST API handling 500 requests/second, with CPU usage at 80%.
3. A machine learning model that must load a 50GB dataset into RAM for each inference.
4. A WebSocket server managing 10,000 persistent connections with per-connection state in memory.
5. A file-processing service that converts uploaded videos to different formats.

<details>
<summary>Hint 1</summary>

Think about whether the bottleneck is something that can be parallelized across machines or something that requires shared resources on a single machine.

</details>

<details>
<summary>Hint 2</summary>

Consider the statefulness of each component. Where does state live? Can it be externalized?

</details>

### Part B: Draw the Cost Curve

The following table shows the cost and capacity of different server sizes on a cloud provider:

| Instance | vCPUs | RAM | Cost/hour | Requests/second |
|----------|-------|-----|-----------|-----------------|
| small | 2 | 8 GB | $0.08 | 200 |
| medium | 4 | 16 GB | $0.19 | 380 |
| large | 8 | 32 GB | $0.42 | 700 |
| xlarge | 16 | 64 GB | $0.92 | 1,200 |
| 2xlarge | 32 | 128 GB | $2.01 | 1,800 |
| 4xlarge | 64 | 256 GB | $4.30 | 2,400 |

You need to serve **3,000 requests/second**. Calculate the cheapest way to do this using:
1. Vertical scaling only (single largest instance -- is it enough?)
2. Horizontal scaling only (multiple smaller instances)

Show your math. Which approach is cheaper? Which gives you more headroom?

<details>
<summary>Hint</summary>

For horizontal scaling, divide the target throughput by the capacity of each instance type to find how many instances you need. Then multiply by cost per hour.

</details>

### Part C: Identify the Ceiling

For each of these scaling approaches, identify the hard ceiling -- the point beyond which you cannot scale further with that approach alone:

1. Vertical scaling of a single EC2 instance.
2. Horizontal scaling of a stateful application with in-memory sessions.
3. Horizontal scaling of a database (without sharding).

<details>
<summary>Hint</summary>

Think about what fundamentally limits each approach. Hardware limits? Coordination overhead? Data consistency?

</details>

## Success Criteria

- [ ] You correctly classified all 5 scenarios with justified reasoning.
- [ ] You calculated the cost for both vertical and horizontal approaches to serve 3,000 req/s.
- [ ] You identified the hard ceiling for each scaling approach.
- [ ] You can explain why vertical scaling has diminishing returns.
- [ ] You can articulate when horizontal scaling is not the right first move.

## What You Should Understand After This Exercise

Scaling is not a binary choice -- it is a trade-off analysis. Vertical scaling is simpler but hits hard ceilings and has diminishing cost returns. Horizontal scaling is more flexible but requires architectural support (statelessness, load balancing). The right choice depends on the bottleneck type, the statefulness of the application, the budget, and the reliability requirements. In practice, most production systems use a combination: vertically scale databases (hard to distribute) and horizontally scale application servers (easy to distribute).
