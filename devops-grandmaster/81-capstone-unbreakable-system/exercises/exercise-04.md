# Exercise 04: Disaster Recovery and Chaos Testing

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Implement disaster recovery procedures and validate them with chaos
engineering. You will design backup strategies, write failover runbooks,
and create chaos experiments that prove the system survives real failures.
The goal is not to hope the system is resilient -- it is to prove it
breaks in controlled ways and recovers automatically.

## Scenario

The payment-api is running across two regions (us-east-1 primary,
eu-west-1 secondary) with the observability stack from Exercise 03.
The database uses PostgreSQL with streaming replication. Leadership
asks: "If us-east-1 goes down right now, how long until we are back
online, and do we lose any data?"

You cannot answer this question without testing it. You must:

1. Define RTO (Recovery Time Objective) and RPO (Recovery Point
   Objective) targets
2. Implement automated backup and restore procedures
3. Write and test a failover runbook
4. Design chaos experiments that validate each failure domain
5. Build automated recovery procedures that reduce human dependency

## Tasks

### Part A: Define RTO/RPO and Backup Strategy

Define RTO and RPO targets for each tier of the system:

| Tier | Component | RTO | RPO | Backup Strategy |
|------|-----------|-----|-----|-----------------|
| 1 | PostgreSQL database | ? | ? | ? |
| 2 | Application state (pods) | ? | ? | ? |
| 3 | Configuration (K8s manifests) | ? | ? | ? |
| 4 | TLS certificates | ? | ? | ? |
| 5 | Observability data (metrics, logs) | ? | ? | ? |

For the database, write a CronJob manifest that:

1. Takes a full backup every 6 hours
2. Takes a WAL (Write-Ahead Log) archive every 5 minutes
3. Stores backups in an S3 bucket in a different region
4. Retains backups for 30 days
5. Verifies backup integrity automatically

<details>
<summary>Hint</summary>

RTO is the maximum acceptable time to restore service. RPO is the
maximum acceptable data loss measured in time. For a payment system,
Tier 1 should have RPO near zero (synchronous replication) and RTO
under 60 seconds (automated failover). Use `pg_basebackup` for full
backups and `pg_receivewal` or `archive_command` for WAL archiving.

</details>

### Part B: Write the Failover Runbook

Write a step-by-step runbook for failing over from us-east-1 to
eu-west-1. The runbook must cover:

1. Detection -- how do you know us-east-1 is down?
2. Decision -- who decides to fail over? What criteria?
3. Execution -- what commands run, in what order?
4. Verification -- how do you confirm the failover succeeded?
5. Rollback -- how do you switch back when us-east-1 recovers?

Each step must include the exact command or API call, the expected
output, and the timeout before moving to the next step.

<details>
<summary>Hint</summary>

The runbook should be executable by any engineer, not just the person
who wrote it. Every command should be copy-pasteable. Include checks
at each step that verify the previous step succeeded. The rollback
procedure is as important as the failover -- do not skip it.

</details>

### Part C: Design Chaos Experiments

Design five chaos experiments that validate the system's resilience.
For each experiment, write:

1. **Name** -- descriptive name
2. **Hypothesis** -- what should happen when this failure is injected
3. **Blast radius** -- what is affected
4. **Method** -- how to inject the failure (command or tool)
5. **Rollback** -- how to stop the experiment if things go wrong
6. **Success criteria** -- how to verify the hypothesis

The five experiments must cover:

1. Pod failure (kill random pods)
2. Network latency injection (add 500ms latency between services)
3. Database failover (promote replica to primary)
4. Node failure (cordon and drain a node)
5. Region failure (simulate entire region outage)

<details>
<summary>Hint</summary>

Use Litmus Chaos or Chaos Mesh for Kubernetes-native chaos experiments.
For region failure, you can simulate it by updating the DNS health
check to mark us-east-1 as unhealthy, which triggers automatic DNS
failover. Always have a rollback command ready before starting any
experiment.

</details>

### Part D: Automated Recovery Procedures

Write a Kubernetes CronJob that runs a daily "recovery rehearsal." The
job should:

1. Create a test database in the secondary region
2. Restore the latest backup to it
3. Run a set of validation queries (row counts, checksums, recent
   transactions)
4. Compare results with the primary database
5. Report discrepancies to Slack
6. Clean up the test database

This proves that backups are not just taken -- they are recoverable.

<details>
<summary>Hint</summary>

A backup that has never been restored is not a backup -- it is a prayer.
The validation queries should check that: (a) the restore completes
within RTO, (b) data is consistent (no corruption), and (c) the
most recent transaction is within RPO. Use `pg_restore --dry-run`
first to verify the backup is valid before actually restoring.

</details>

### Part E: Build a Chaos Dashboard

Design a Grafana dashboard that shows the results of chaos experiments.
It must display:

1. A timeline of all chaos experiments run in the last 30 days
2. For each experiment: hypothesis, actual outcome, pass/fail
3. The system's recovery time for each experiment vs. the RTO target
4. A "resilience score" -- the percentage of experiments that passed

Describe the data model (how experiment results are stored) and the
Prometheus/Grafana queries needed.

<details>
<summary>Hint</summary>

Store experiment results as Prometheus metrics with labels for
experiment name, status (pass/fail), and recovery time. Use a custom
exporter or a simple script that pushes metrics via the Pushgateway
after each experiment. The resilience score is
`passed_experiments / total_experiments * 100`.

</details>

## Success Criteria

- [ ] RTO/RPO targets are defined for all 5 tiers with realistic values
- [ ] The backup CronJob is valid YAML and includes integrity verification
- [ ] The failover runbook has exact commands, expected outputs, and timeouts for every step
- [ ] All 5 chaos experiments have hypotheses, methods, and rollback procedures
- [ ] The recovery rehearsal CronJob validates backup integrity and reports discrepancies
- [ ] The chaos dashboard shows experiment history, outcomes, and resilience score

## What You Should Understand After This Exercise

Disaster recovery is not a document you write once and file away. It
is a practice you rehearse regularly. Chaos engineering is the tool
that proves your DR actually works. If you have never tested your
failover procedure, you do not have a failover procedure -- you have
a theory. The most dangerous state for a system is one where everyone
believes it is resilient but no one has verified it.
