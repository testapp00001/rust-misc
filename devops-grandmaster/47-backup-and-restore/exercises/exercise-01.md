# Exercise 01: Backup Strategy Selection

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective
Select the appropriate backup strategy for different database workloads based on RPO, RTO, size, and change rate.

## Scenario
You are the DBA responsible for six PostgreSQL databases with different characteristics:

| Database | Size | Daily Change Rate | RPO | RTO | Query Pattern |
|----------|------|-------------------|-----|-----|---------------|
| Payment transactions | 200GB | 5% | 0 (zero data loss) | < 1 min | Write-heavy |
| Product catalog | 50GB | 1% | < 1 hour | < 15 min | Read-heavy |
| User sessions | 100GB | 20% | < 5 min | < 5 min | Read-write |
| Audit logs | 2TB | 10% | < 24 hours | < 4 hours | Append-only |
| Analytics warehouse | 5TB | 2% | < 1 hour | < 1 hour | Read-only |
| Configuration | 1GB | 0.1% | 0 (zero data loss) | < 1 min | Rare writes |

## Tasks

### Part A: Classify Backup Needs
For each database, determine:
1. Primary backup method: pg_dump, continuous archiving (WAL), or streaming replication
2. Backup frequency: continuous, hourly, daily, or weekly
3. Whether you need a physical or logical backup

<details>
<summary>Hint</summary>
Zero RPO requires synchronous replication or continuous WAL archiving. Large databases benefit from physical backups (pg_basebackup) over logical (pg_dump). Append-only databases can use incremental strategies.
</details>

### Part B: Calculate Storage Requirements
For each database, calculate:
1. Full backup size (assume 60% compression ratio)
2. Daily incremental backup size
3. Total storage needed for 30-day retention

<details>
<summary>Hint</summary>
Full backup size = database size * 0.6 (compression). Incremental size = daily change rate * database size * 0.6. Total = full backup + (30 * incremental). For WAL archiving, estimate daily WAL volume.
</details>

### Part C: Map RPO/RTO to Technology
Create a table mapping each database's RPO and RTO to the specific technology that achieves it:

| Database | RPO | RTO | Technology |
|----------|-----|-----|------------|
| Payment transactions | 0 | < 1 min | ??? |

<details>
<summary>Hint</summary>
RPO = 0 requires synchronous replication. RTO < 1 min requires a hot standby that can be promoted instantly. RPO < 24 hours can be achieved with daily pg_dump. RTO < 15 min can use pg_basebackup + WAL replay.
</details>

## Success Criteria

- [ ] Each database is assigned a primary backup method with justification
- [ ] Storage calculations are shown with work
- [ ] RPO/RTO mapping uses specific, achievable technologies
- [ ] No database has a backup strategy that cannot meet its RPO or RTO

## What You Should Understand After This Exercise

Backup strategy is not one-size-fits-all. A payment database needs zero data loss (synchronous replication), while audit logs can tolerate 24-hour RPO (daily dump). The choice between pg_dump (logical) and pg_basebackup (physical) depends on database size and restore speed requirements. WAL archiving bridges the gap between full backups by providing point-in-time recovery.
