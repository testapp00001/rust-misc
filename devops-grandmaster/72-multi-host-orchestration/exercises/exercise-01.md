# Exercise 01: Swarm vs Nomad vs Kubernetes

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

You will compare the three major container orchestrators -- Docker Swarm, HashiCorp Nomad, and Kubernetes -- across key dimensions. This exercise tests your understanding of when to use each tool and what trade-offs each makes.

## Scenario

Your startup has three different teams, each proposing a different orchestrator:

- **Team A** wants Docker Swarm because "it is built into Docker and we already know Docker Compose."
- **Team B** wants Nomad because "we already use Consul and Vault, and we run some non-containerized Java services."
- **Team C** wants Kubernetes because "we need the ecosystem, operators, and multi-cloud portability."

The CTO asks you to write a decision matrix so the company can make an informed choice.

## Tasks

### Part A: Feature Comparison Table

Create a comparison table with the following columns: Feature, Docker Swarm, Nomad, Kubernetes. Include rows for:

1. Installation complexity
2. Supported workloads (containers, VMs, binaries, etc.)
3. Service discovery mechanism
4. Secrets management
5. Auto-scaling capabilities
6. Community size and ecosystem

<details>
<summary>Hint</summary>
Think about what each tool was designed for. Swarm is Docker-native, Nomad is a general-purpose scheduler, and Kubernetes is a full platform. Each has a different "center of gravity."
</details>

### Part B: Scenario Matching

For each scenario below, recommend the best orchestrator and explain why in 1-2 sentences:

1. A 5-person team deploying a single Python web app on 3 servers.
2. A company running containers, a legacy Java WAR file, and a compiled Go binary.
3. A large enterprise needing CRDs, operators, and multi-cloud portability with a dedicated platform team.
4. A team already using Consul, Vault, and Terraform who wants minimal operational overhead.

<details>
<summary>Hint</summary>
Match the complexity of the tool to the complexity of the problem. Over-engineering is as costly as under-engineering.
</details>

### Part C: Trade-off Analysis

Explain in your own words why you cannot simply "pick the best one" for all cases. What specific trade-offs does each orchestrator make?

<details>
<summary>Hint</summary>
Consider the axes of simplicity vs. flexibility, built-in features vs. extensibility, and learning curve vs. long-term capability.
</details>

## Success Criteria

- [ ] Your comparison table covers all 6 rows with accurate, specific details (not vague statements like "good" or "bad").
- [ ] Each scenario recommendation is justified with a concrete reason tied to the scenario.
- [ ] Your trade-off analysis identifies at least one strength and one weakness for each orchestrator.
- [ ] You can articulate when each tool is the wrong choice, not just the right one.

## What You Should Understand After This Exercise

There is no universally best orchestrator. Docker Swarm trades advanced features for simplicity. Nomad trades container-native features for workload flexibility. Kubernetes trades simplicity for a massive ecosystem and portability. The right choice depends on your team size, workload mix, existing tooling, and how much operational complexity you can absorb.
