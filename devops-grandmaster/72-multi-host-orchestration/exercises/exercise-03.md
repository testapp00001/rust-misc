# Exercise 03: Nomad Job Specification

**Type:** Independent
**Time:** 40 minutes
**Difficulty:** Medium

## Objective

You will write a complete Nomad job specification in HCL for a multi-group service deployment. This exercise practices Nomad-specific concepts: job types, task groups, spread strategies, rescheduling, service registration with health checks, and resource allocation.

## Scenario

You are deploying a data processing pipeline called "pipeline" on Nomad. The pipeline has two task groups:

1. **ingest** -- Receives data from an external source. Must run 3 instances spread across datacenters. Uses the Docker driver. Needs 500 MHz CPU and 256 MB memory per instance. Registers a health check at `/health` on port 3000.
2. **processor** -- Processes ingested data. Must run 5 instances. Uses the Docker driver. Needs 2000 MHz CPU and 1024 MB memory per instance. Has a template that injects the ingest service address from Consul.

The job should auto-revert on failure and reschedule tasks with exponential backoff.

## Tasks

### Part A: Write the Job Specification

Write a complete Nomad job file (`pipeline.nomad`) in HCL that defines:

- Job type: `service`
- Datacenter: `["dc1", "dc2"]`
- Two task groups: `ingest` (count=3) and `processor` (count=5)
- Spread strategy for the `ingest` group across `${node.datacenter}`
- Update block with `auto_revert = true` and canary deployment
- Reschedule block with exponential backoff
- Service registration with HTTP health checks for both groups
- Resource allocation matching the scenario
- A template stanza in the `processor` group that discovers the `ingest` service via Consul

<details>
<summary>Hint</summary>
The Nomad template stanza uses Consul Template syntax. To discover a service: `{{ range service "ingest" }}{{ .Address }}:{{ .Port }}{{ end }}`. The spread block uses `attribute` and `weight` fields.
</details>

### Part B: Deployment and Operations Commands

Write the commands to:

1. Validate the job file without submitting it.
2. Submit (run) the job.
3. Check the job status.
4. Scale the `processor` group to 8 instances.
5. View logs for a specific allocation.
6. Stop the job.

<details>
<summary>Hint</summary>
Use `nomad job validate` to check syntax. `nomad job run` submits the job. `nomad job scale` takes the job name, group name, and count.
</details>

### Part C: Comparison with Docker Swarm

Explain how the Nomad job spec you wrote maps to a Docker Swarm stack definition. Specifically address:

1. How does `group "ingest" { count = 3 }` compare to `deploy.replicas: 3`?
2. How does Nomad's `service` block compare to Swarm's built-in DNS discovery?
3. How does Nomad's `update` block compare to Swarm's `deploy.update_config`?

<details>
<summary>Hint</summary>
Nomad is more explicit -- you define networking, service registration, health checks, and resources at the task group level. Swarm infers much of this from the Docker Compose format.
</details>

## Success Criteria

- [ ] Your Nomad job file is valid HCL with both task groups defined.
- [ ] The `ingest` group uses a spread strategy across datacenters.
- [ ] Both groups register services with health checks.
- [ ] The `processor` group includes a template that discovers the `ingest` service.
- [ ] Update and reschedule blocks are configured for safe deployments.
- [ ] Your comparison identifies at least 3 concrete differences between Nomad and Swarm.

## What You Should Understand After This Exercise

Nomad uses HCL job files as the declarative definition of your workload. Unlike Docker Swarm's Compose-based approach, Nomad requires you to explicitly define networking, service registration, and health checks. This verbosity gives you more control but requires deeper understanding. Nomad's real strength is its ability to schedule any workload type, not just containers.
