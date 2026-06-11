# Exercise 04: Disaster Recovery Drill

**Type:** Challenge | **Time:** 45 min | **Difficulty:** Medium-Hard

## Objective

Design and execute a disaster recovery drill that validates your automated failover system, measures actual RPO and RTO, and produces a post-drill report.

## Scenario

You are the SRE lead at NimbusCloud, a SaaS platform serving 10,000 enterprise customers. The platform runs on Kubernetes with a PostgreSQL database (Patroni-managed) as the primary data store. The team has never conducted a formal DR drill, and leadership wants confidence that the system can recover within the documented RTO of 60 seconds and RPO of 0 seconds (synchronous replication).

The production environment:
- 3-node Patroni PostgreSQL cluster (us-east-1a, us-east-1b, us-east-1c)
- 3-node etcd cluster for consensus
- 12 application pods behind a Kubernetes Service
- Prometheus, Grafana, and Alertmanager for monitoring
- A staging environment that mirrors production topology

The drill will be conducted in the staging environment during a maintenance window.

## Tasks

### Part A: DR Drill Plan

Write a comprehensive DR drill plan document.

1. Define the drill objectives and success criteria (measurable).
2. Define the scope: which systems are included, which are excluded.
3. Define the timeline: pre-drill checks, drill execution, post-drill validation, rollback.
4. Define roles: who executes, who observes, who communicates.
5. Define the abort criteria: conditions that trigger immediate drill cancellation.

<details>
<summary>Hint</summary>
Success criteria should be binary: either you achieved 60-second RTO or you did not. "Approximately 60 seconds" is not a success criterion -- define a tolerance (e.g., RTO <= 75 seconds including 15 seconds of buffer).
</details>

### Part B: Failure Simulation Script

Write a script that simulates a primary database failure in the staging environment.

1. Write a script that kills the Patroni process on the primary node.
2. Write a script that simulates a network partition (primary loses connectivity to replicas).
3. Write a script that simulates a slow disk (degraded I/O, not complete failure).
4. Ensure all scripts are safe for staging use (confirmation prompts, dry-run mode).

<details>
<summary>Hint</summary>
For network partition simulation, use iptables or tc to drop traffic between specific nodes. For slow disk simulation, use dm-delay or cgroups to throttle I/O. Always include a cleanup/undo function.
</details>

### Part C: Drill Execution and Measurement

Write the procedure for executing the drill and measuring actual RPO and RTO.

1. Write a script that continuously inserts test records with timestamps into the database.
2. Write a script that detects when failover completes (new primary accepts writes).
3. Write a script that calculates actual RPO by comparing the last write on the old primary with the first read on the new primary.
4. Write a script that calculates actual RTO by measuring the time between the last successful write on the old primary and the first successful write on the new primary.
5. Write a script that verifies data integrity after failover (no lost or corrupted records).

<details>
<summary>Hint</summary>
RPO measurement: insert records with monotonically increasing IDs and timestamps. After failover, query the new primary for the highest ID. The gap between the last ID written and the first ID readable is the data loss. RTO measurement: record timestamps of write failures and the first successful write after failover.
</details>

### Part D: Post-Drill Report Template

Write a post-drill report template that captures all findings.

1. Write a report template with sections: Executive Summary, Drill Timeline, RPO/RTO Results, Issues Found, Recommendations, Follow-up Actions.
2. Include a metrics table: Actual RPO, Actual RTO, Number of errors during failover, Time to detect failure, Time to promote new primary, Time to reconnect applications.
3. Include a gap analysis section: compare actual results to target RPO/RTO.
4. Include a lessons learned section with a template for action items.

<details>
<summary>Hint</summary>
The executive summary should be 3 sentences: what we tested, what happened, and whether we passed. Drill Timeline should be a minute-by-minute log with timestamps. Gap analysis should identify specific bottlenecks (e.g., "detection took 45 seconds because health check interval was too long").
</details>

## Success Criteria

- [ ] Drill plan has measurable success criteria with specific numbers.
- [ ] Failure simulation scripts include dry-run mode and cleanup functions.
- [ ] RPO measurement script correctly identifies the last committed transaction.
- [ ] RTO measurement script timestamps the failure window accurately.
- [ ] Data integrity verification checks for missing and corrupted records.
- [ ] Post-drill report template is complete and can be filled in during a real drill.

## What You Should Understanding

- A DR drill is not just "kill the primary and see what happens" -- it requires measurement, safety controls, and a structured report.
- Measuring RPO requires application-level instrumentation, not just database-level monitoring.
- RTO is composed of detection time + decision time + failover time + verification time. Optimizing any one component reduces RTO.
- The value of a DR drill is in the findings, not in passing. A drill that reveals a 5-minute RTO when you expected 30 seconds is more valuable than a drill that confirms what you already knew.
