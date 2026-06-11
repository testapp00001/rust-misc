# Exercise 01: Map the Kubernetes Architecture

**Type:** Conceptual
**Time:** 20 minutes
**Difficulty:** Easy

## Objective

Draw and explain the Kubernetes architecture from memory. This exercise
verifies that you understand the major components, where they live, and
how they interact -- the foundation for everything else in this module.

## Scenario

You are onboarding a new team member who has never used Kubernetes.
They ask you: "What actually happens inside a Kubernetes cluster?"
You decide to draw a diagram and explain it.

## Tasks

### Part A: Draw the Architecture

Without looking at the README or any reference, draw an ASCII diagram
of a Kubernetes cluster showing:

1. The control plane and its four components
2. At least two worker nodes
3. The components that run on each worker node
4. Pods running on the worker nodes
5. The connection between control plane and worker nodes

Your diagram should fit in a terminal (roughly 80 characters wide).

<details>
<summary>Hint</summary>

Think about two layers:
- The "brain" layer (control plane) that makes decisions
- The "muscle" layer (worker nodes) that runs containers

Each layer has specific components. The control plane components are
co-located, while worker nodes are distributed across machines.

</details>

### Part B: Label Every Component

Below your diagram, create a table with three columns:

| Component | Runs On | One-Sentence Role |
|-----------|---------|-------------------|

Fill in every component you drew. Be precise -- "stores data" is too
vague. "Stores all cluster state as key-value pairs" is better.

<details>
<summary>Hint</summary>

There are 7 components total:
- 4 on the control plane
- 3 on each worker node

If you have fewer than 7, you are missing something.

</details>

### Part C: Draw the Request Flow

A developer runs `kubectl apply -f pod.yaml`. Draw a sequence diagram
(showing numbered steps) that traces this request from the developer's
terminal to the container running on a worker node.

Your diagram should show which component handles each step and what
it communicates to the next component.

<details>
<summary>Hint</summary>

The request passes through these components in order:
1. kubectl (developer's machine)
2. API server (control plane)
3. etcd (control plane) -- stores the desired state
4. Scheduler (control plane) -- picks a node
5. kubelet (worker node) -- receives the assignment
6. Container runtime (worker node) -- starts the container

There are also status reporting steps that flow back upward.

</details>

### Part D: Explain the "Why"

For each control plane component, explain what would break if that
component were removed. Write one sentence per component:

- "Without the API server, ..."
- "Without etcd, ..."
- "Without the scheduler, ..."
- "Without the controller manager, ..."

<details>
<summary>Hint</summary>

Think about what unique responsibility each component holds.
If two components seem to do the same thing, you have misunderstood
one of them. Each has a distinct, non-overlapping role.

</details>

## Success Criteria

- [ ] Your diagram shows the control plane and at least 2 worker nodes
- [ ] All 7 components are present and correctly placed
- [ ] The component table has 7 rows with accurate one-sentence roles
- [ ] The request flow has at least 6 numbered steps
- [ ] Each "Without X, ..." statement describes a unique failure mode
- [ ] You drew this from memory without consulting the README

## What You Should Understand After This Exercise

Kubernetes has a clear separation between decision-making (control plane)
and execution (worker nodes). The API server is the single gateway for
all communication. etcd is the single source of truth. The scheduler
and controller manager are watchers that react to state changes. Worker
nodes are interchangeable execution environments.
