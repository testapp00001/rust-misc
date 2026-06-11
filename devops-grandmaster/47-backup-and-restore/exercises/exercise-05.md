# Exercise 05: Complete Backup and DR Plan

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective
Design a complete backup and disaster recovery plan for a production PostgreSQL database that covers all failure scenarios and meets defined SLAs.

## Scenario
You are the DBA for an e-commerce platform with the following database profile:

| Metric | Value |
|--------|-------|
| Database size | 500GB |
| Daily transactions | 10 million |
| Peak concurrent connections | 500 |
| Business hours | 24/7 (global) |
| Annual revenue | $50 million |
| Cost of downtime | $5,700/minute |

Data classification:
- Customer PII: Highly sensitive (encryption required)
- Order data: Critical (zero data loss)
- Product catalog: Important (1-hour RPO acceptable)
- Session data: Transient (5-minute RPO acceptable)
- Analytics: Low priority (24-hour RPO acceptable)

## Tasks

### Part A: Define RPO and RTO Requirements
For each data classification, determine:
1. RPO (Recovery Point Objective)
2. RTO (Recovery Time Objective)
3. The cost of not meeting each objective
4. The technology that achieves each objective

<details>
<summary>Hint</summary>
Map data classifications to backup technologies. Order data needs RPO = 0 (synchronous replication). Customer PPI needs encryption at rest. Session data can use async replication. Analytics can use daily dumps.
</details>

### Part B: Design the Backup Architecture
Create a comprehensive backup architecture that includes:
1. Full physical backups (frequency, retention, storage)
2. WAL archiving (configuration, storage, retention)
3. Logical backups (pg_dump for specific tables/schemas)
4. Offsite replication (S3, cross-region)
5. Encryption for all backups

Draw an ASCII diagram of the backup architecture.

<details>
<summary>Hint</summary>
Layer your backups: physical base backup (daily) + WAL archiving (continuous) for PITR. Logical backups for selective restore. S3 with lifecycle policies for retention. GPG or AWS KMS for encryption.
</details>

### Part C: Write the Complete Automation
Provide the scripts and configuration for:
1. Automated daily backup with WAL archiving
2. Backup upload to S3 with encryption
3. Backup verification pipeline
4. Retention cleanup (local: 7 days, S3: 90 days)

<details>
<summary>Hint</summary>
Use pgBackRest or a custom script with pg_basebackup. For S3, use `aws s3 cp --sse aws:kms`. For verification, restore to a Docker container and run checks. For cleanup, use S3 lifecycle policies and local cron jobs.
</details>

### Part D: Create Recovery Runbooks
Write runbooks for:
1. Single table recovery (developer accidentally drops a table)
2. Full database recovery (primary disk failure)
3. Point-in-time recovery (data corruption at a known time)
4. Cross-region recovery (entire region is unavailable)

<details>
<summary>Hint</summary>
Each runbook should include: trigger condition, prerequisites, step-by-step commands, verification steps, expected duration, and rollback plan. Make them copy-pasteable.
</details>

## Success Criteria

- [ ] RPO/RTO requirements are defined for each data classification
- [ ] Backup architecture diagram shows all components and data flow
- [ ] Automation scripts are complete and idempotent
- [ ] Each recovery runbook is specific and includes verification steps
- [ ] Encryption is configured for all backup storage locations

## What You Should Understand After This Exercise

A complete backup and DR plan is a layered defense. Physical backups with WAL archiving provide point-in-time recovery. Logical backups enable selective restoration. Offsite storage protects against site failures. Encryption protects against data breaches. Verification ensures backups are restorable. Runbooks ensure the team can execute recovery under pressure.
