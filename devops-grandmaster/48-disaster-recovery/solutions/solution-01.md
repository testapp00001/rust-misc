# Solution 01: RPO and RTO Analysis

## Part A: Cost of Downtime Calculation

### Downtime Cost Table

| System | Direct Loss/hr | Indirect Loss/hr | Regulatory Risk | Total Cost/hr |
|--------|---------------|-----------------|-----------------|---------------|
| PAYMENTS | $180,000 | $0 | Low (PCI audit) | $180,000 |
| SESSION | $0 | $120,000 | None | $120,000 |
| CUSTOMER | $0 | $40,000 | Low | $40,000 |
| LEDGER | $0 | $0 | $500,000 (SOX) | $500,000* |
| ANALYTICS | $0 | $5,000 | None | $5,000 |
| ARCHIVE | $0 | $0 | Low (retention) | ~$0 |

*LEDGER is special: the SOX fine is a threshold event, not linear. Any data loss triggers the full fine regardless of duration. This makes LEDGER the most expensive system to lose data from, even though it has zero direct revenue impact.

### Cost Ranking (Highest to Lowest)

1. **LEDGER** -- $500,000 SOX violation (threshold-based, any data loss)
2. **PAYMENTS** -- $180,000/hr direct revenue loss
3. **SESSION** -- $120,000/hr indirect (drives PAYMENTS flow)
4. **CUSTOMER** -- $40,000/hr indirect
5. **ANALYTICS** -- $5,000/hr indirect
6. **ARCHIVE** -- ~$0/hr (regulatory risk only if retention violated)

Key insight: SESSION has no direct revenue but ranks third because it is the dependency for PAYMENTS. If SESSION goes down, users cannot authenticate, and PAYMENTS stops generating revenue too. The true cost of SESSION failure is $180,000 + $120,000 = $300,000/hr.

### Common Mistakes to Avoid

- Treating SOX fines as hourly costs. A $500,000 fine is not $500,000/hr -- it is a one-time penalty triggered by any data loss. This means LEDGER's RPO must be 0 regardless of how long the system is down.
- Ignoring dependency chains. SESSION failure causes PAYMENTS failure. ANALYTICS failure does not cause anything else to fail.
- Confusing revenue impact with business impact. ARCHIVE has $0 revenue impact but regulatory penalties for data loss could be severe.

---

## Part B: RPO and RTO Determination

### RPO/RTO Table

| System | RPO | RTO | Tier | Justification |
|--------|-----|-----|------|---------------|
| LEDGER | 0 (zero) | 30 sec | Tier 1 | SOX compliance mandates zero data loss. Any loss triggers $500K fine. RTO is less critical than RPO. |
| PAYMENTS | 0 (zero) | 30 sec | Tier 1 | $180K/hr direct loss. 30-second RTO limits loss to ~$1,500. Synchronous replication required. |
| SESSION | 5 sec | 30 sec | Tier 1 | Session data is ephemeral. 5 seconds of lost sessions means users re-authenticate. RTO must match PAYMENTS because SESSION is a dependency. |
| CUSTOMER | 5 min | 5 min | Tier 2 | $40K/hr indirect loss. 5-minute RTO limits loss to ~$3,300. Profiles are infrequently updated. |
| ANALYTICS | 4 hr | 4 hr | Tier 3 | Batch ETL every 4 hours. Losing one batch is acceptable -- just re-run the ETL. RPO/RTO aligned with ETL schedule. |
| ARCHIVE | 24 hr | 48 hr | Tier 3 | Write-once, read-rarely. Can restore from backup. Regulatory requirement is retention, not availability. |

### Tier Definitions

- **Tier 1 (seconds):** RPO 0-5 sec, RTO 30 sec. Systems where downtime or data loss causes immediate, measurable financial or compliance damage.
- **Tier 2 (minutes):** RPO 1-5 min, RTO 5-15 min. Systems where downtime causes indirect business impact but is tolerable for short periods.
- **Tier 3 (hours):** RPO hours, RTO hours. Systems where downtime has minimal immediate impact and can be restored from backups.

### Common Mistakes to Avoid

- Setting RPO=0 for everything. Synchronous replication adds write latency. Not every system needs it.
- Setting RTO=0. Zero RTO means the system never goes down, which requires active-active multi-region -- extremely expensive.
- Ignoring dependency chains in RTO. If PAYMENTS depends on SESSION, then SESSION's RTO must be <= PAYMENTS's RTO.
- Aligning RPO with backup schedule. RPO is about data loss tolerance, not backup frequency. If RPO is 5 minutes, you need replication, not 5-minute backups (backups have restore time).

---

## Part C: Technology Mapping

### Technology Mapping Table

| Tier | RPO | RTO | Replication | Failover | Technology |
|------|-----|-----|-------------|----------|------------|
| Tier 1 | 0 sec | 30 sec | Synchronous replication | Automatic with consensus | Patroni + etcd + sync standby |
| Tier 1 | 5 sec | 30 sec | Asynchronous replication (synchronous preferred) | Automatic with consensus | Patroni + etcd + async standby |
| Tier 2 | 5 min | 5 min | Asynchronous replication | Automatic with health checks | Streaming replication + pg_auto_failover |
| Tier 3 | 4 hr | 4 hr | Periodic snapshots | Restore from backup | pg_dump cron + S3 + restore script |
| Tier 3 | 24 hr | 48 hr | Daily backup | Restore from backup | pg_basebackup + S3 lifecycle |

### Detailed Technology Choices

**Tier 1 -- LEDGER and PAYMENTS:**

```yaml
# Patroni configuration for synchronous replication
postgresql:
  parameters:
    synchronous_commit: "on"
    synchronous_standby_names: "*"
  create_replica_methods:
    - basebackup
```

Synchronous replication ensures every write is confirmed on at least one replica before being acknowledged to the client. This guarantees RPO=0. Patroni with etcd provides automatic failover within 30 seconds.

Trade-off: Synchronous replication adds 2-5ms of latency per write. For PAYMENTS processing 2,000 writes/sec, this adds 4-10 seconds of cumulative latency per hour. Acceptable for the RPO guarantee.

**Tier 1 -- SESSION:**

```yaml
# Redis Sentinel for automatic failover
sentinel monitor session-master 10.0.1.10 6379 2
sentinel down-after-milliseconds session-master 5000
sentinel failover-timeout session-master 30000
```

Redis Sentinel provides automatic failover for session storage. Asynchronous replication means up to 5 seconds of sessions may be lost. Users simply re-authenticate -- no data corruption.

**Tier 2 -- CUSTOMER:**

```yaml
# pg_auto_failover configuration
pg_autoctl create postgres \
  --pgdata /var/lib/postgresql/15/main \
  --monitor postgres://autoctl_node@monitor:5432/pg_auto_failover \
  --name node1
```

pg_auto_failover provides simpler automation than Patroni for Tier 2 systems. Asynchronous replication with a 5-minute lag tolerance. Automatic failover triggered by health checks.

**Tier 3 -- ANALYTICS and ARCHIVE:**

```bash
#!/bin/bash
# Backup script for Tier 3 systems
pg_dump -Fc -Z9 analytics | aws s3 cp - s3://backups/analytics/$(date +%Y%m%d_%H%M%S).dump
aws s3api put-object-retention --bucket backups --key "analytics/$(date +%Y%m%d_%H%M%S).dump" --retention '{"Mode":"COMPLIANCE","RetainUntilDate":"2031-01-01T00:00:00Z"}'
```

Periodic backups to S3 with compliance-mode retention for ARCHIVE (7-year retention). ANALYTICS can be rebuilt from source data, so backup is a convenience, not a requirement.

---

## Key Takeaway

DR strategy must be driven by business impact analysis, not technology preferences. The cost of downtime determines the tier, the tier determines the technology, and the technology determines the budget. A one-size-fits-all approach either over-protects low-value systems (wasting money) or under-protects high-value systems (risking revenue and compliance).
