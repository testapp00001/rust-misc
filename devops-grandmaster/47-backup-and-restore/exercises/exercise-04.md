# Exercise 04: Backup Verification and Integrity Testing

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective
Design and implement an automated backup verification pipeline that ensures every backup is restorable and consistent.

## Scenario
Your team discovered that a backup from last week was corrupted and could not be restored. The corruption was not detected because there was no verification process. You need to build an automated system that verifies every backup within 1 hour of completion.

## Tasks

### Part A: Design a Verification Pipeline
Design an automated pipeline that, for every backup:
1. Restores the backup to an isolated test instance
2. Runs consistency checks on critical tables
3. Verifies row counts match the source database
4. Reports success or failure

Draw an ASCII flowchart of the pipeline.

<details>
<summary>Hint</summary>
The pipeline should: create a temporary Docker container with PostgreSQL, restore the backup, run verification queries, compare results with the source, report via Slack/email, and destroy the container. Use `pg_restore --no-owner` for logical backups.
</details>

### Part B: Write Verification Scripts
Write a bash script that:
1. Restores a pg_dump backup to a test database
2. Runs `ANALYZE` on all tables
3. Compares row counts with the source database
4. Checks for any corruption using `amcheck`

```bash
#!/bin/bash
# verify_backup.sh -- complete this script
```

<details>
<summary>Hint</summary>
Use `pg_restore -d test_db backup.dump` for restoration. Compare row counts with `SELECT count(*)` queries. Use `pg_amcheck` (PostgreSQL 14+) for index consistency checks. Run `ANALYZE` to update statistics and detect corruption.
</details>

### Part C: Implement Checksum Verification
Add checksum verification for backup files:
1. Generate SHA-256 checksums after each backup
2. Verify checksums before restoration
3. Detect bit rot or partial file corruption

<details>
<summary>Hint</summary>
Use `sha256sum backup.dump > backup.dump.sha256` after backup. Verify with `sha256sum -c backup.dump.sha256` before restore. Store checksums separately from backup files (in a different S3 bucket or directory).
</details>

### Part D: Create Alerting for Backup Failures
Design an alerting system that notifies the team when:
1. A backup fails to complete
2. A backup verification fails
3. A backup file's checksum does not match
4. No backup has completed in the last 25 hours

<details>
<summary>Hint</summary>
Use Prometheus metrics with `postgres_exporter` for backup status. Create alerts for `pg_backup_status == failed` and `time() - pg_backup_last_success_timestamp > 90000`. Send notifications via Alertmanager to Slack.
</details>

## Success Criteria

- [ ] Verification pipeline restores backup and runs checks automatically
- [ ] Row count comparison catches data loss
- [ ] Checksum verification detects file corruption
- [ ] Alerting covers all failure modes (backup, verification, checksum, staleness)

## What You Should Understand After This Exercise

A backup that has never been tested is not a backup -- it is a hope. Automated verification ensures every backup is restorable by actually restoring it. Checksum verification catches file-level corruption. The combination of restoration testing and checksum verification provides confidence that you can recover when needed.
