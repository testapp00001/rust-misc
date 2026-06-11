# Exercise 02: Automated Backup Pipeline Setup

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective
Build an automated backup pipeline that combines pg_dump for logical backups, pg_basebackup for physical backups, and WAL archiving for point-in-time recovery.

## Scenario
Your team needs to automate PostgreSQL backups for a production database. The requirements are:
- Daily full backup at 2 AM UTC
- WAL archiving enabled continuously
- Backups stored locally for 7 days and in S3 for 30 days
- Backup completion notifications via Slack
- Automatic cleanup of expired backups

## Tasks

### Part A: Write a pg_dump Backup Script
Create a bash script that:
1. Takes a logical backup using pg_dump with custom format
2. Compresses the backup
3. Includes a timestamp in the filename
4. Logs the backup size and duration

```bash
#!/bin/bash
# backup_logical.sh -- complete this script
```

<details>
<summary>Hint</summary>
Use `pg_dump -Fc` for custom format (compressed, supports selective restore). Include error handling with `set -e`. Use `date +%Y%m%d_%H%M%S` for timestamps. Measure duration with `SECONDS` variable.
</details>

### Part B: Configure WAL Archiving
Write the PostgreSQL configuration for continuous WAL archiving:
1. `archive_command` in `postgresql.conf`
2. `archive_mode` settings
3. A script that archives WAL files to a local directory and S3

<details>
<summary>Hint</summary>
Set `archive_mode = on` (requires restart) and `archive_command` to copy WAL files. The archive command must return 0 on success and non-zero on failure. Use `aws s3 cp` for S3 upload with retry logic.
</details>

### Part C: Create a Scheduling System
Write the cron configuration for:
1. Full backup at 2 AM UTC daily
2. WAL archive cleanup at 3 AM UTC daily
3. Backup verification at 4 AM UTC daily

<details>
<summary>Hint</summary>
Cron syntax: `0 2 * * *` for 2 AM daily. Use `>/dev/null 2>&1` to suppress output, or redirect to a log file. Consider using systemd timers instead of cron for better logging and dependency management.
</details>

### Part D: Add S3 Upload
Extend the backup script to upload backups to S3 with:
1. Server-side encryption
2. Lifecycle policy for automatic expiration
3. Retry logic for failed uploads

<details>
<summary>Hint</summary>
Use `aws s3 cp --sse AES256` for encryption. Set S3 lifecycle policy to delete objects after 30 days. Use a loop with exponential backoff for retries. Consider `aws s3 sync` for incremental uploads.
</details>

## Success Criteria

- [ ] pg_dump script is complete with error handling and logging
- [ ] WAL archiving is configured with both local and S3 storage
- [ ] Cron schedule covers backup, cleanup, and verification
- [ ] S3 uploads include encryption and retry logic

## What You Should Understand After This Exercise

A production backup pipeline is more than a single pg_dump command. It requires WAL archiving for point-in-time recovery, scheduling for consistency, S3 upload for offsite storage, retention policies for cleanup, and verification to ensure backups are actually restorable. The automation must be idempotent -- running it twice should not create duplicate backups.
