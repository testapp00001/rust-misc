# Solution 01: Backup Strategy Selection

## Part A: Backup Classification

| Database | Primary Method | Frequency | Backup Type | Rationale |
|----------|---------------|-----------|-------------|-----------|
| Payment transactions | Streaming replication + WAL archiving | Continuous | Physical | RPO = 0 requires synchronous replication. Physical backup supports fast restore. |
| Product catalog | pg_basebackup + WAL archiving | Daily full + continuous WAL | Physical | 50GB is small enough for daily physical backup. WAL archiving provides PITR. |
| User sessions | pg_basebackup + WAL archiving | Daily full + continuous WAL | Physical | High change rate (20%) makes pg_dump slow. Physical backup + WAL handles it. |
| Audit logs | pg_dump (partitioned) | Daily incremental | Logical | Append-only data allows incremental dumps by date partition. 2TB is too large for daily full. |
| Analytics warehouse | pg_basebackup | Weekly full | Physical | 5TB is too large for daily full. Weekly physical backup + daily incremental. |
| Configuration | Streaming replication + pg_dump | Continuous replication + daily dump | Both | RPO = 0 needs replication. Daily dump provides a portable logical backup for DR. |

### Key Decision Factors

- **Physical vs. Logical**: Physical backups (pg_basebackup) are faster for large databases. Logical backups (pg_dump) are portable and allow selective restore.
- **Continuous vs. Periodic**: RPO = 0 requires continuous replication or WAL archiving. RPO > 1 hour can use periodic pg_dump.
- **Size matters**: A 5TB database cannot be dumped daily -- physical backup with WAL archiving is the only viable approach.

## Part B: Storage Calculations

| Database | Full Size (60% compression) | Daily Incremental | 30-Day Total |
|----------|----------------------------|-------------------|--------------|
| Payment transactions | 120GB | 6GB (5% of 200GB * 0.6) | 120GB + (30 * 6GB) = 300GB |
| Product catalog | 30GB | 0.3GB (1% of 50GB * 0.6) | 30GB + (30 * 0.3GB) = 39GB |
| User sessions | 60GB | 12GB (20% of 100GB * 0.6) | 60GB + (30 * 12GB) = 420GB |
| Audit logs | 1,200GB | 120GB (10% of 2TB * 0.6) | 1,200GB + (30 * 120GB) = 4,800GB |
| Analytics warehouse | 3,000GB | 60GB (2% of 5TB * 0.6) | 3,000GB + (30 * 60GB) = 4,800GB |
| Configuration | 0.6GB | 0.0006GB (0.1% of 1GB * 0.6) | 0.6GB + (30 * 0.0006GB) = 0.62GB |

**Total storage for 30-day retention: ~10,360 GB (~10.1 TB)**

### WAL Archiving Storage Estimate

For databases using continuous WAL archiving:

| Database | Daily WAL Volume (estimated) | 30-Day WAL Storage |
|----------|------------------------------|-------------------|
| Payment transactions | ~10GB (5% change rate, ~200GB * 5%) | 300GB |
| Product catalog | ~0.5GB | 15GB |
| User sessions | ~20GB | 600GB |

## Part C: RPO/RTO Technology Mapping

| Database | RPO | RTO | Technology | How It Achieves the Goal |
|----------|-----|-----|------------|--------------------------|
| Payment transactions | 0 | < 1 min | Synchronous streaming replication + hot standby | Sync replication ensures zero data loss. Hot standby can be promoted in seconds. |
| Product catalog | < 1 hour | < 15 min | WAL archiving + pg_basebackup | WAL replay can restore to any point within the last hour. 15 min is enough to replay 1 hour of WAL. |
| User sessions | < 5 min | < 5 min | Async streaming replication + hot standby | Async replication with < 5 min lag. Promotion takes seconds. |
| Audit logs | < 24 hours | < 4 hours | Daily pg_dump + S3 storage | Daily dump captures all data up to the dump time. Restore from S3 takes 1-2 hours for 2TB. |
| Analytics warehouse | < 1 hour | < 1 hour | pg_basebackup + WAL archiving | Weekly full backup + continuous WAL. Restore base backup (30 min) + replay 1 hour of WAL. |
| Configuration | 0 | < 1 min | Synchronous replication + daily pg_dump | Sync replication for zero RPO. pg_dump for portable DR backup. |

### Why Each Technology Matches the Requirement

**Synchronous replication** (Payment, Config): The primary waits for the replica to confirm each write before acknowledging to the client. If the primary fails, the replica has every committed transaction. RPO = 0.

**Async streaming replication** (User sessions): The primary sends WAL to the replica but does not wait for confirmation. The replica may be seconds behind. RPO = replication lag (typically < 5 seconds).

**WAL archiving** (Product catalog, Analytics): WAL segments are copied to archive storage. Combined with a base backup, you can replay WAL to any point in time. RPO = time of last archived WAL segment.

**Daily pg_dump** (Audit logs): A logical dump of the database at a point in time. RPO = 24 hours (time between dumps). RTO depends on dump size and restore speed.

## Common Mistakes to Avoid

- Using pg_dump for a 5TB database -- the dump takes hours and the restore takes even longer
- Setting RPO = 0 without synchronous replication -- async replication always has some lag
- Forgetting that RTO includes detection time, not just restore time -- a 1-minute restore is useless if it takes 30 minutes to detect the failure
- Not considering WAL volume in storage calculations -- WAL can generate terabytes per month for high-write databases

## Key Takeaway

Backup strategy selection is a trade-off between RPO, RTO, storage cost, and complexity. Synchronous replication achieves RPO = 0 but adds write latency. WAL archiving provides point-in-time recovery but requires continuous management. pg_dump is simple but only works for small databases. The right strategy matches the technology to the requirement -- no more, no less.
