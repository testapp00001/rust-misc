# Exercise 02: Failover Procedure Design

**Type:** Guided | **Time:** 30 min | **Difficulty:** Easy-Medium

## Objective

Design a complete failover procedure for a PostgreSQL primary database failure, including the runbook, decision criteria, failback plan, and stakeholder communication.

## Scenario

You are the database reliability engineer for StreamVault, a video streaming platform. The database architecture consists of:

- **Primary:** PostgreSQL 15 on EC2 in us-east-1a (writes and reads)
- **Sync Replica:** PostgreSQL 15 on EC2 in us-east-1b (hot standby, synchronous replication)
- **Async Replica:** PostgreSQL 15 on EC2 in us-west-2 (cross-region, asynchronous replication, 2-5 second lag)
- **Connection pool:** PgBouncer in front of primary
- **Application:** 12 microservices connecting via PgBouncer
- **Monitoring:** Prometheus + Grafana with Alertmanager

At 02:47 UTC, Alertmanager fires: `PostgreSQLPrimaryDown` -- the primary in us-east-1a has been unreachable for 60 seconds. You are the on-call engineer.

## Tasks

### Part A: Manual Failover Runbook

Write a step-by-step runbook for promoting the synchronous replica in us-east-1b to primary. Include the exact commands to run at each step.

1. Define the trigger conditions that confirm the primary is truly down (not a false alarm).
2. Write the steps to promote the sync replica, including fencing the old primary.
3. Write the steps to reconfigure PgBouncer and application connections.
4. Write the steps to verify the new primary is accepting writes.
5. Write the steps to rebuild replication after promotion.

<details>
<summary>Hint</summary>
Step 1 should include multiple independent checks (SSH, pg_isready, network ping). Step 2 needs to ensure the old primary does not come back and accept writes (split-brain). Step 3 can use PgBouncer's RELOAD or a config management tool.
</details>

### Part B: Decision Criteria -- Failover vs. Repair-in-Place

Define the decision matrix for choosing between failing over immediately versus attempting to repair the primary in place.

1. List the conditions under which you should fail over immediately (e.g., hardware failure, disk corruption).
2. List the conditions under which you should attempt repair first (e.g., network blip, OOM kill).
3. Define the maximum time to attempt repair before escalating to failover.
4. Write a decision flowchart in pseudocode or text.

<details>
<summary>Hint</summary>
Consider the RTO. If RTO is 5 minutes and repair takes 10 minutes, you must fail over. If RTO is 1 hour and repair takes 5 minutes, repair is safer.
</details>

### Part C: Failback Procedure

After the original primary in us-east-1a is recovered, design the procedure to return to the original topology.

1. Write the steps to bring the old primary back as a replica of the new primary.
2. Describe how to verify data consistency between old primary and new primary.
3. Define the conditions for switching back to the original primary (or deciding to keep the new primary).
4. Write the steps to reverse the roles if switching back.

<details>
<summary>Hint</summary>
pg_rewind is faster than pg_basebackup for bringing an old primary back as a replica. Data consistency can be verified with pg_stat_replication lag and checksums.
</details>

### Part D: Communication Plan

Create a communication plan for stakeholders during the failover event.

1. Define the audience tiers: who gets notified at each stage (detected, investigating, failing over, recovered).
2. Write the initial notification message (within 2 minutes of detection).
3. Write the status update message (during failover).
4. Write the resolution message (after recovery).
5. Define the escalation path if failover does not succeed.

<details>
<summary>Hint</summary>
Use a structured format: What happened, What is the impact, What are we doing, When is the next update. Keep messages concise -- stakeholders need signal, not noise.
</details>

## Success Criteria

- [ ] Runbook has numbered steps with exact commands and expected outputs.
- [ ] Runbook includes fencing the old primary to prevent split-brain.
- [ ] Decision matrix covers at least 4 failure scenarios with clear criteria.
- [ ] Failback procedure includes data consistency verification.
- [ ] Communication plan has 3 audience tiers with message templates.
- [ ] Escalation path is defined for failed failover attempts.

## What You Should Understand

- A failover runbook is not just a list of commands -- it includes verification steps and rollback points.
- The decision to fail over is time-bound: the RTO drives the decision, not the complexity of the fix.
- Failback is often more dangerous than failover because it involves two working systems swapping roles.
- Communication during incidents is as important as the technical resolution.
