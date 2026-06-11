# Exercise 01: DaemonSet vs Deployment vs Job - When to Use Each

## Objective

Understand the fundamental differences between DaemonSets, Deployments, and Jobs, and be able to choose the correct workload type for different scenarios.

## Background

Kubernetes offers several workload types, each designed for specific use cases:

- **Deployment**: Manages stateless applications with a specified number of replicas
- **DaemonSet**: Ensures one pod runs on every node (or selected nodes)
- **Job**: Runs pods to completion for batch or one-time tasks
- **CronJob**: Schedules Jobs to run periodically

Choosing the wrong workload type can lead to resource waste, incomplete tasks, or application failures.

## Instructions

### Part 1: Conceptual Understanding

Answer the following questions:

1. What is the primary difference between a DaemonSet and a Deployment with the same number of replicas as nodes?
2. When would you use a Job instead of a Deployment?
3. Can a DaemonSet be used to run a batch processing task? Why or why not?
4. What happens to a DaemonSet pod if a new node is added to the cluster?
5. How does a CronJob relate to a Job?

### Part 2: Scenario Matching

For each scenario below, identify whether you should use a DaemonSet, Deployment, Job, or CronJob. Explain your reasoning.

**Scenario A**: You need to collect logs from every node in the cluster and send them to a central logging system.

**Scenario B**: Your application needs to process 10,000 image files, converting each to a thumbnail.

**Scenario C**: You want to run a web application with 3 replicas that can handle incoming HTTP requests.

**Scenario D**: You need to generate a daily report at midnight and email it to stakeholders.

**Scenario E**: You want to monitor network traffic on every node for security analysis.

**Scenario F**: You need to run a database migration script once before deploying your application.

**Scenario G**: You want to run a cache server that should be available on every node for low-latency access.

### Part 3: Resource Behavior

Explain what happens in each scenario:

1. You create a Deployment with 3 replicas, but your cluster has 5 nodes. How many pods run? Where do they run?

2. You create a DaemonSet, but your cluster has 5 nodes. How many pods run? Where do they run?

3. You create a Job with `completions: 5` and `parallelism: 2`. How does Kubernetes execute this?

4. You create a CronJob with `schedule: "0 0 * * *"`. When does it run? What happens if the previous run hasn't finished?

## Success Criteria

- [ ] You can explain the purpose of each workload type in one sentence
- [ ] You can correctly identify the appropriate workload type for 7/7 scenarios
- [ ] You understand how each workload type handles scaling and node changes
- [ ] You can describe the execution model of Jobs (completions, parallelism)

## Hints

<details>
<summary>Hint 1: DaemonSet vs Deployment</summary>

The key difference is **scheduling strategy**:
- Deployment: "I want N copies" (Kubernetes decides where)
- DaemonSet: "I want one copy on every node" (Kubernetes ensures coverage)

Think about whether your application needs to be on EVERY node or just needs enough replicas to handle load.
</details>

<details>
<summary>Hint 2: When to use Jobs</summary>

Jobs are for **finite tasks** that should run to completion. Ask yourself:
- Does this task have a clear end state?
- Should it run continuously or finish?
- Do I need to ensure it completes successfully?

If the task should run forever, it's not a Job.
</details>

<details>
<summary>Hint 3: CronJob timing</summary>

CronJobs use standard cron syntax. If a run is still active when the next scheduled time arrives, the behavior depends on the `concurrencyPolicy`:
- `Allow`: Run concurrently (default)
- `Forbid`: Skip the new run
- `Replace`: Cancel the old run, start new one
</details>

<details>
<summary>Hint 4: Job completion modes</summary>

- `completions`: How many times the Job needs to complete successfully
- `parallelism`: How many pods can run at the same time
- `backoffLimit`: How many times to retry on failure before giving up
</details>

## Concepts to Review

After completing this exercise, you should understand:

- The scheduling guarantees of each workload type
- When to choose one workload type over another
- How Jobs handle completion and parallelism
- The relationship between CronJobs and Jobs
