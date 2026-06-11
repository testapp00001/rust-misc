# Exercise 01: How Deployments Manage ReplicaSets

**Type:** Conceptual
**Time:** 20 minutes
**Difficulty:** Easy

## Objective

Demonstrate your understanding of the relationship between Deployments, ReplicaSets, and Pods. You will explain how a Deployment creates and manages ReplicaSets, describe what happens during updates and rollbacks at the ReplicaSet level, and predict the state of these objects after various operations. This exercise verifies that you understand the controller hierarchy, not just how to write YAML.

## Background

A Deployment is a higher-level controller that manages ReplicaSets, which in turn manage Pods. You rarely interact with ReplicaSets directly -- the Deployment controller handles their creation, scaling, and lifecycle. Understanding this hierarchy is essential for debugging deployment issues, interpreting `kubectl rollout history`, and knowing why old ReplicaSets linger in your cluster.

## Tasks

### Part A: The Controller Hierarchy

Fill in the table below. For each resource, describe its purpose, what it owns, and what happens when you delete it.

| Resource | Purpose | It Owns | What Happens When You Delete It |
|----------|---------|---------|--------------------------------|
| Deployment | | | |
| ReplicaSet | | | |
| Pod | | | |

<details>
<summary>Hint -- Ownership and Cascading Deletion</summary>
Kubernetes uses owner references to track which controller owns which resource. When you delete a parent resource, Kubernetes cascades the deletion to its children. Think about what creates what, and what happens to children when the parent disappears.
</details>

### Part B: ReplicaSet Creation During Updates

A Deployment named `web-app` starts with image `nginx:1.25` and 3 replicas. You update it to `nginx:1.26`. Answer the following questions:

1. Before the update, how many ReplicaSets exist for this Deployment? Why?
2. During the update (rolling update in progress), how many ReplicaSets exist? Why does Kubernetes keep the old one?
3. After the update completes, how many ReplicaSets exist? What determines whether the old ReplicaSet is deleted?
4. If you run `kubectl rollout undo`, what happens at the ReplicaSet level?

<details>
<summary>Hint -- ReplicaSet Naming</summary>
ReplicaSet names include a hash of the pod template. When the pod template changes (e.g., different image), a new ReplicaSet is created with a different hash. The old ReplicaSet is scaled down but kept for rollback purposes.
</details>

### Part C: Revision History

Explain the following observations:

**Observation 1:** You run `kubectl rollout history deployment web-app` and see revisions 1, 3, and 4. Revision 2 is missing. What happened?

**Observation 2:** You set `revisionHistoryLimit: 2` on a Deployment. After performing 5 updates, how many ReplicaSets exist? What happened to the earlier ones?

**Observation 3:** You run `kubectl get replicasets` and see three ReplicaSets for your Deployment. One has 3 replicas, one has 0, and one has 0. Which one is active? Why do the other two still exist?

<details>
<summary>Hint -- Revision vs ReplicaSet</summary>
A revision number is not the same as the number of ReplicaSets. Revisions are recorded in the Deployment's rollout history. If a rollback creates the same pod template as a previous revision, Kubernetes reuses the existing ReplicaSet rather than creating a new one. The revisionHistoryLimit controls how many old ReplicaSets are kept, not how many revisions are recorded.
</details>

### Part D: Predicting Behavior

For each scenario, describe what the Deployment controller does. Be specific about ReplicaSet scaling actions.

**Scenario 1:** A Deployment has `replicas: 5`. A node crashes, killing 2 pods. What does the ReplicaSet do?

**Scenario 2:** You change `replicas` from 3 to 7 in the Deployment YAML and run `kubectl apply`. What happens?

**Scenario 3:** You change the image from `nginx:1.25` to `nginx:1.26` and the replicas from 3 to 5 simultaneously. What happens? Does the Deployment handle both changes at once or sequentially?

**Scenario 4:** A rolling update is in progress (old ReplicaSet has 2 pods, new ReplicaSet has 1 pod). You run `kubectl rollout undo`. What happens?

<details>
<summary>Hint -- Desired State vs Current State</summary>
The Deployment controller always works toward the desired state declared in the spec. It compares the current state (pods running) to the desired state (replicas count, pod template) and takes action to close the gap. When you change both the image and the replica count, the controller reconciles both changes.
</details>

## Success Criteria

- [ ] Your controller hierarchy table accurately describes ownership and cascading deletion
- [ ] You can explain how many ReplicaSets exist before, during, and after an update
- [ ] You understand why old ReplicaSets are kept and what revisionHistoryLimit controls
- [ ] You can predict Deployment controller behavior for scaling, updating, and rollback scenarios
- [ ] You can distinguish between revisions and ReplicaSets

## What You Should Understand After This Exercise

After completing this exercise, you should be able to look at a Deployment and its ReplicaSets in your cluster, understand why each ReplicaSet exists, predict what will happen when you change the Deployment spec, and explain the relationship between rollout history and ReplicaSet lifecycle. This understanding is the foundation for debugging stuck rollbacks, identifying resource waste from old ReplicaSets, and designing safe update strategies.
