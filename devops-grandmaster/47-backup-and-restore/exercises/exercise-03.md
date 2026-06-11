# Exercise 03: Point-in-Time Recovery Execution

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective
Execute a point-in-time recovery (PITR) to restore a PostgreSQL database to a specific moment before a data corruption event.

## Scenario
At 14:35 UTC on 2024-03-15, a developer accidentally ran `DELETE FROM customers;` on the production database (instead of running it on the staging database). The deletion was committed at 14:35:22 UTC. You need to restore the database to 14:34:00 UTC -- the last known good state before the deletion.

Your backup infrastructure:
- Full physical backup taken at 2:00 AM UTC daily
- WAL archives stored in `/var/lib/postgresql/wal_archive/`
- The backup manifest shows the backup ended at 2:15 AM UTC

## Tasks

### Part A: Identify the Recovery Target
Determine:
1. Which full backup to use as the base
2. The exact WAL segment needed for recovery
3. The recovery target time in the correct format

<details>
<summary>Hint</summary>
Use the most recent full backup (2:00 AM on 2024-03-15). PITR replays WAL from the backup end time to the target time. The recovery target is `2024-03-15 14:34:00+00`.
</details>

### Part B: Write the Recovery Configuration
Write the PostgreSQL configuration for PITR:
1. Restore the base backup to a new directory
2. Configure `postgresql.auto.conf` for recovery
3. Create the `recovery.signal` file

```bash
# Write the recovery commands
```

<details>
<summary>Hint</summary>
PostgreSQL 12+ uses `restore_command`, `recovery_target_time`, and `recovery_target_action` in `postgresql.conf` or `postgresql.auto.conf`. Create an empty `recovery.signal` file to trigger recovery mode.
</details>

### Part C: Execute Recovery and Verify
Write the step-by-step procedure to:
1. Stop the corrupted database
2. Restore the backup
3. Start PostgreSQL in recovery mode
4. Verify the data is correct (customers table is intact)
5. Promote to primary if the recovered instance will replace the original

<details>
<summary>Hint</summary>
Stop PostgreSQL before restoring. Use `pg_basebackup` output or `tar` for the base backup. Start PostgreSQL -- it will automatically enter recovery mode due to `recovery.signal`. Verify with `SELECT count(*) FROM customers;`. Promote with `SELECT pg_promote();`.
</details>

### Part D: Write a Recovery Runbook
Document the recovery procedure as a runbook that any team member can follow. Include:
1. Prerequisites and assumptions
2. Step-by-step commands
3. Verification steps
4. Rollback plan if recovery fails

<details>
<summary>Hint</summary>
A runbook should be copy-pasteable. Include exact commands with placeholders for environment-specific values (host, port, paths). Add expected output for each step so the operator knows if something is wrong.
</details>

## Success Criteria

- [ ] Recovery target is correctly identified (14:34:00 UTC)
- [ ] Recovery configuration uses `recovery_target_time` and `restore_command`
- [ ] Verification confirms customers table is intact
- [ ] Runbook is complete and followable by a non-expert

## What You Should Understand After This Exercise

Point-in-time recovery works by restoring a base backup and replaying WAL segments up to the target time. The key configuration is `restore_command` (where to find archived WAL) and `recovery_target_time` (when to stop). PITR is the foundation of database disaster recovery -- it allows you to recover from any point in time, not just the last backup.
