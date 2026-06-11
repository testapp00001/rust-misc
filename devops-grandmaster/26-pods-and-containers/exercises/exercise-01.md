# Exercise 01: Pod Lifecycle Phases

**Type:** Conceptual
**Time:** 20 minutes
**Difficulty:** Easy

## Objective

Demonstrate your understanding of the Kubernetes Pod lifecycle by explaining each phase, describing what triggers transitions between phases, and predicting Pod behavior in real scenarios. This exercise verifies that you understand *why* Pods behave the way they do, not just *what* the phases are named.

## Background

Every Pod in Kubernetes moves through a defined lifecycle. The kubelet manages this lifecycle, and understanding it is essential for debugging, capacity planning, and designing reliable workloads. The four primary phases are Pending, Running, Succeeded, and Failed. A fifth state, Unknown, indicates the kubelet cannot reach the node.

## Tasks

### Part A: Phase Definitions

Fill in the table below. For each Pod phase, describe what it means, list at least two conditions that cause a Pod to enter that phase, and give one real-world example of a workload that would end up in that phase.

| Phase | Definition | Conditions That Trigger It | Example Workload |
|-------|------------|---------------------------|------------------|
| Pending | | | |
| Running | | | |
| Succeeded | | | |
| Failed | | | |

<details>
<summary>Hint -- Pending vs Running</summary>
A Pod is Pending when it has been accepted by the cluster but one or more containers have not yet been created. This includes time spent scheduling and downloading images. A Pod is Running when at least one container is in the process of running or is in the process of starting or restarting.
</details>

### Part B: Transition Scenarios

For each scenario below, state the phase the Pod will be in and explain *why*.

**Scenario 1:** You create a Pod requesting 8 CPU cores, but every node in the cluster only has 4 cores available.

**Scenario 2:** A Pod runs a batch job that processes 1000 records. All containers exit with code 0 after processing completes.

**Scenario 3:** A Pod runs a web server. The container starts, begins accepting connections, and continues running indefinitely.

**Scenario 4:** A Pod runs a script that attempts to connect to an external API. The API is unreachable, and the container exits with code 1.

**Scenario 5:** A Pod is running normally. The node it is on loses network connectivity to the Kubernetes control plane.

<details>
<summary>Hint -- Exit Codes Matter</summary>
When all containers in a Pod terminate, the Pod phase is determined by the exit codes. Exit code 0 from all containers means Succeeded. Any non-zero exit code from any container means Failed. The Unknown phase is about the node, not the containers.
</details>

### Part C: Conditions vs Phases

Kubernetes Pods have both *phases* and *conditions*. Explain the difference between these two concepts by answering the following questions:

1. What are the Pod conditions? List them.
2. Can a Pod in the Running phase have a condition that indicates a problem? Give an example.
3. Why does Kubernetes maintain both phases and conditions instead of just one mechanism?

<details>
<summary>Hint -- Condition Types</summary>
Pod conditions include PodScheduled, Initialized, ContainersReady, and Ready. Each condition has a status of True, False, or Unknown. A Pod can be in the Running phase while one of its containers is failing its readiness probe.
</details>

## Success Criteria

- [ ] Your phase table has accurate definitions and at least two triggering conditions per phase
- [ ] Each scenario answer includes the correct phase and a clear explanation of why
- [ ] You can distinguish between phases and conditions and explain why both exist
- [ ] You can predict Pod behavior without referencing documentation

## What You Should Understand After This Exercise

After completing this exercise, you should be able to look at any Pod in your cluster, understand what phase it is in, reason about what caused it to reach that phase, and predict what will happen next. This understanding is the foundation for every debugging task you will perform with Kubernetes Pods.
