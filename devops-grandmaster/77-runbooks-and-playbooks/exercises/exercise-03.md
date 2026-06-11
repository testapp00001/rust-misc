# Exercise 03: Write a Database Failover Runbook

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective

Create a complete runbook for database failover from scratch, without a provided template. This
exercise tests your ability to structure a runbook independently, handle both automatic and manual
failover procedures, and account for the complexity of stateful systems.

## Scenario

Your company runs a PostgreSQL database cluster with streaming replication:

```
                    +------------------+
                    |   Load Balancer  |
                    |  (HAProxy/PGbdr) |
                    +--------+---------+
                             |
                    +--------+---------+
                    |                  |
             +------v------+   +------v------+
             |   Primary   |   |  Replica 1  |
             |  (pg-primary|   | (pg-replica-|
             |   -0)       |   |  1)         |
             |  Read/Write |   |  Read-Only  |
             +------+------+   +-------------+
                    |
                    | streaming replication
                    |
             +------v------+
             |  Replica 2  |
             | (pg-replica-|
             |  2)         |
             |  Read-Only  |
             +-------------+
```

At 14:22 UTC, the following alert fires:

```
[CRITICAL] PostgreSQL Primary Unreachable
  Host: pg-primary-0.db.internal:5432
  Last successful health check: 14:19 UTC
  Replication lag on replica-1: 0 bytes (caught up)
  Replication lag on replica-2: 45 MB (behind)
  Automatic failover: ENABLED (Patroni/pg_auto_failover)
```

The automatic failover system is in place, but you need a runbook that covers:
1. Verifying whether automatic failover succeeded.
2. Manual failover if automatic failover fails.
3. Post-failover verification and cleanup.
4. Failback procedures.

## Tasks

### Part A: Design the Runbook Structure

Before writing content, design the section structure for this runbook. Database failover is more
complex than a pod restart because:
- Data consistency is at risk.
- Replication state must be verified.
- Application connection strings may need updating.
- Failback (returning to the original primary) is a separate procedure.

List the sections your runbook will contain with a one-sentence description of what each covers.

<details><summary>Hint</summary>
Think about the lifecycle: Detect -> Verify Failure -> Check Replication State -> Failover
(Auto or Manual) -> Verify New Primary -> Update Routing -> Monitor -> Plan Failback.
</details>

### Part B: Write the Diagnosis Section

Write the diagnosis steps that determine:
1. Is the primary truly down, or is it a network partition?
2. Which replica is the best failover candidate?
3. What is the replication state of each replica?

Include commands for:
- Checking connectivity to the primary.
- Checking replication lag on each replica.
- Checking Patroni/pg_auto_failover cluster status.
- Verifying the primary is not just slow but actually unreachable.

```bash
# Example command structure (fill in the details)
patronictl -c /etc/patroni/patroni.yaml list
psql -h pg-replica-1 -U monitor -c "SELECT pg_is_in_recovery(), 
  pg_last_wal_receive_lsn(), pg_last_wal_replay_lsn();"
```

<details><summary>Hint</summary>
Use `patronictl list` or `pg_autoctl show state` to see cluster health.
Check replication lag with `SELECT now() - pg_last_xact_replay_timestamp() AS replication_lag;`
on each replica. A network partition can be detected if the primary is reachable from some
hosts but not others.
</details>

### Part C: Write the Automatic Failover Verification Section

The system has automatic failover enabled. Write steps to:
1. Check if automatic failover has already occurred.
2. Verify the new primary is accepting writes.
3. Verify the old primary (if it comes back) is now a replica and not causing split-brain.
4. Confirm application connectivity to the new primary.

<details><summary>Hint</summary>
After automatic failover, check `patronictl list` again. The new primary should show as "Leader."
Test writes with `psql -h <new-primary> -c "CREATE TABLE failover_test (id int); DROP TABLE failover_test;"`.
The old primary, if it recovers, should show as "Replica" -- if it still thinks it is primary,
you have a split-brain situation.
</details>

### Part D: Write the Manual Failover Section

Write the manual failover procedure for when automatic failover fails. Include:

1. **Pre-checks**: Verify replication state, confirm no split-brain.
2. **Promote the best replica**: Exact commands to promote a replica to primary.
3. **Update routing**: How to update the load balancer or service discovery to point to the new primary.
4. **Demote the old primary**: How to ensure the old primary does not accept writes (fencing).
5. **Reconfigure remaining replicas**: Point them at the new primary.

```bash
# Provide the exact sequence of commands
# Step 1: Promote replica
# Step 2: Update DNS/load balancer
# Step 3: Fence old primary
# Step 4: Reconfigure replicas
```

<details><summary>Hint</summary>
For Patroni: `patronictl failover --candidate pg-replica-1 --master pg-primary-0 --force`.
For manual PostgreSQL: `pg_ctl promote -D /var/lib/postgresql/data`.
Update HAProxy health checks or Consul/etcd service registration.
Fence the old primary by revoking its ability to accept connections (pg_hba.conf, firewall rule,
or shutting it down).
</details>

### Part E: Write the Post-Failover and Failback Sections

**Post-Failover**:
- Monitoring to confirm the new primary is healthy.
- Application-level verification (can the app read and write?).
- Alerting adjustments (the old primary-down alert should be resolved; new alerts for the new primary).

**Failback** (returning to the original topology):
- When is it safe to fail back? (e.g., during a maintenance window.)
- How to rebuild the old primary as a replica.
- How to perform a controlled switchover back to the original primary.

<details><summary>Hint</summary>
Failback is not urgent. Wait until the root cause of the original failure is understood and fixed.
Rebuild the old primary as a replica using `pg_basebackup` or Patroni's `patronictl remove` +
re-add workflow. Schedule the switchover during low-traffic hours.
</details>

## Success Criteria

- [ ] Runbook structure is well-organized with clear sections.
- [ ] Diagnosis distinguishes between true failure and network partition.
- [ ] Replication lag checking is included with specific commands.
- [ ] Automatic failover verification includes split-brain detection.
- [ ] Manual failover procedure has exact commands for promotion, routing, fencing, and reconfiguration.
- [ ] Post-failover verification covers both database and application health.
- [ ] Failback procedure is safe (maintenance window, root cause addressed).
- [ ] The runbook accounts for the scenario where Replica 2 (45 MB behind) is the only option.
- [ ] An on-call engineer with PostgreSQL knowledge could follow this runbook under pressure.

## What You Should Understand After This Exercise

Database failover runbooks are among the highest-stakes operational documents. Unlike stateless
services where you can just restart a pod, database failover risks data loss, split-brain, and
cascading failures. A good database failover runbook must be precise about replication state,
explicit about fencing (preventing the old primary from accepting writes), and cautious about
failback. The difference between a 2-minute failover and a 2-hour outage is often the quality
of the runbook.
