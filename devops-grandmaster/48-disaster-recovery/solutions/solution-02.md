# Solution 02: Failover Procedure Design

## Part A: Manual Failover Runbook

### Trigger Conditions

Before executing failover, confirm the primary is truly down with 3 independent checks:

```bash
# Check 1: PostgreSQL readiness (from monitoring host)
pg_isready -h primary.us-east-1a.internal -p 5432 -t 5
# Expected if down: "no response" or timeout

# Check 2: Network connectivity (from monitoring host)
ping -c 3 -W 2 primary.us-east-1a.internal
# Expected if down: 100% packet loss

# Check 3: Patroni REST API (from sync replica)
curl -s --connect-timeout 5 http://primary.us-east-1a.internal:8008/health
# Expected if down: connection refused or timeout
```

If all 3 checks fail, proceed with failover. If any check succeeds, investigate before proceeding.

### Step-by-Step Failover Runbook

**Step 1: Confirm primary is unreachable (0:00)**

```bash
# Run all 3 checks from Part A trigger conditions
# Log the results with timestamps
echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) - Starting failover procedure" >> /var/log/failover.log
```

**Step 2: Fence the old primary (0:30)**

Prevent the old primary from accepting writes if it comes back:

```bash
# Option A: If SSH is available, put PostgreSQL in recovery mode
ssh primary.us-east-1a.internal "sudo -u postgres psql -c 'SELECT pg_promote();' 2>/dev/null || true"
ssh primary.us-east-1a.internal "sudo systemctl stop postgresql"

# Option B: If SSH is unavailable, use network fencing
# Remove the old primary from the security group or ACL
aws ec2 modify-network-interface-attribute \
  --network-interface-id eni-oldprimary \
  --groups sg-no-access

# Option C: Update PgBouncer to exclude the old primary immediately
```

**Step 3: Promote the sync replica (1:00)**

```bash
# On the sync replica (us-east-1b)
sudo -u postgres psql -c "SELECT pg_promote();"

# Verify promotion
sudo -u postgres psql -c "SELECT pg_is_in_recovery();"
# Expected: f (false = primary)
```

**Step 4: Verify new primary accepts writes (1:30)**

```bash
# Write a test record
sudo -u postgres psql -c "CREATE TABLE IF NOT EXISTS failover_test (id serial, ts timestamp);"
sudo -u postgres psql -c "INSERT INTO failover_test (ts) VALUES (now());"

# Read it back
sudo -u postgres psql -c "SELECT * FROM failover_test ORDER BY id DESC LIMIT 1;"

# Check for replication state
sudo -u postgres psql -c "SELECT * FROM pg_stat_replication;"
```

**Step 5: Reconfigure PgBouncer (2:00)**

```bash
# Update PgBouncer configuration
sudo sed -i 's/host=primary.us-east-1a.internal/host=replica.us-east-1b.internal/' /etc/pgbouncer/pgbouncer.ini

# Reload PgBouncer (no restart needed)
psql -h localhost -p 6432 -U admin pgbouncer -c "RELOAD;"

# Verify connections are routing to new primary
psql -h localhost -p 6432 -U admin pgbouncer -c "SHOW POOLS;"
```

**Step 6: Verify application connectivity (2:30)**

```bash
# Check application health endpoints
for svc in api gateway payments users; do
  curl -s http://${svc}.internal:8080/health | jq .
done

# Check Prometheus for error rate
curl -s 'http://prometheus:9090/api/v1/query?query=rate(http_requests_total{status=~"5.."}[1m])' | jq .
```

**Step 7: Rebuild replication (5:00)**

```bash
# On the old primary (after recovery), rebuild as replica
sudo -u postgres pg_basebackup -h replica.us-east-1b.internal -D /var/lib/postgresql/15/main -P -R

# Create standby.signal
sudo -u postgres touch /var/lib/postgresql/15/main/standby.signal

# Start PostgreSQL as replica
sudo systemctl start postgresql

# Verify replication
sudo -u postgres psql -h replica.us-east-1b.internal -c "SELECT * FROM pg_stat_replication;"
```

### Common Mistakes to Avoid

- Skipping fencing. If the old primary comes back and accepts writes, you have split-brain. Two databases accepting writes = data corruption.
- Promoting without confirming synchronous replication state. If the sync replica has lag, promoting it causes data loss.
- Not verifying writes after promotion. A promoted replica may be read-only due to configuration issues.
- Forgetting the async replica. After failover, the async replica in us-west-2 needs to be reconfigured to replicate from the new primary.

---

## Part B: Decision Criteria -- Failover vs. Repair-in-Place

### Decision Matrix

| Failure Type | Detection | Repair Time | Decision | Rationale |
|-------------|-----------|-------------|----------|-----------|
| Hardware failure (disk, RAM) | SMART errors, kernel panics | Hours (hardware replacement) | Failover immediately | Cannot repair hardware in-place within RTO |
| OOM kill | dmesg, syslog | 2-5 minutes (restart PostgreSQL) | Attempt repair first | Restart is faster than failover; avoid unnecessary failover risk |
| Network blip | Ping loss, packet drops | 1-2 minutes (self-healing) | Wait and monitor | Network issues are often transient |
| Disk full | df, PostgreSQL logs | 5-10 minutes (cleanup) | Attempt repair if <50% full; failover if critical | Cleaning up files is faster than failover |
| Corrupted WAL | PostgreSQL logs, pg_verify | Unknown | Failover immediately | Corruption may spread; replica is clean |
| Configuration error | Recent deploy logs | 2-3 minutes (rollback config) | Attempt repair first | Rollback is deterministic and fast |
| Split-brain detected | Two primaries accepting writes | Immediate | Fence and failover | Split-brain is an emergency; stop writes immediately |

### Decision Flowchart

```
PRIMARY_UNREACHABLE detected
  |
  +--> Run 3 independent health checks (10 seconds)
  |
  +--> All 3 fail?
  |      |
  |      +--> YES: Is SSH to primary available?
  |      |      |
  |      |      +--> YES: Check PostgreSQL process status
  |      |      |      |
  |      |      |      +--> PostgreSQL running but unresponsive?
  |      |      |      |      |
  |      |      |      |      +--> YES: Check disk space and memory
  |      |      |      |      |      |
  |      |      |      |      |      +--> Disk full or OOM?
  |      |      |      |      |      |      +--> YES: Attempt repair (5 min max)
  |      |      |      |      |      |      |      +--> Repair successful? DONE
  |      |      |      |      |      |      |      +--> Repair failed? FAILOVER
  |      |      |      |      |      |
  |      |      |      |      |      +--> Other cause: FAILOVER
  |      |      |      |
  |      |      |      +--> PostgreSQL not running?
  |      |      |             |
  |      |      |             +--> Can restart in <3 min? RESTART
  |      |      |             +--> Cannot restart? FAILOVER
  |      |
  |      +--> NO: SSH unavailable (hardware/network failure)
  |             |
  |             +--> FAILOVER IMMEDIATELY
  |
  +--> Any check succeeds?
         |
         +--> YES: False alarm or partial failure
                +--> Investigate, do NOT failover
```

### Maximum Repair Time Rule

The maximum time to attempt repair before escalating to failover:

```
max_repair_time = RTO - failover_time - verification_time - buffer

Example:
RTO = 5 minutes (300 seconds)
failover_time = 90 seconds (from runbook)
verification_time = 60 seconds
buffer = 30 seconds
max_repair_time = 300 - 90 - 60 - 30 = 120 seconds (2 minutes)
```

If repair cannot be guaranteed within 2 minutes, fail over immediately.

### Common Mistakes to Avoid

- Extending the repair window because "it's almost fixed." The RTO does not care about your feelings.
- Failing over for transient issues. A 5-second network blip does not warrant a failover.
- Not having a decision matrix documented before the incident. Making decisions under pressure without a framework leads to poor outcomes.

---

## Part C: Failback Procedure

### Step-by-Step Failback

**Step 1: Verify old primary is healthy (Day 1)**

```bash
# On the recovered old primary (us-east-1a)
sudo systemctl start postgresql

# Verify PostgreSQL starts in recovery mode (standby.signal exists)
sudo -u postgres psql -c "SELECT pg_is_in_recovery();"
# Expected: t (true = replica)

# If not in recovery mode, do NOT start it as primary
# Create standby.signal and restart
```

**Step 2: Rebuild as replica using pg_rewind (Day 1)**

```bash
# On the recovered old primary (us-east-1a)
# Stop PostgreSQL
sudo systemctl stop postgresql

# Use pg_rewind to synchronize with current primary
sudo -u postgres pg_rewind \
  --target-pgdata=/var/lib/postgresql/15/main \
  --source-server="host=replica.us-east-1b.internal port=5432 dbname=postgres"

# Create standby.signal
sudo -u postgres touch /var/lib/postgresql/15/main/standby.signal

# Start PostgreSQL as replica
sudo systemctl start postgresql
```

**Step 3: Verify data consistency (Day 1-2)**

```bash
# On current primary (us-east-1b)
# Check replication lag
sudo -u postgres psql -c "SELECT client_addr, state, sent_lsn, write_lsn, flush_lsn, replay_lsn,
  (sent_lsn - replay_lsn) AS byte_lag
  FROM pg_stat_replication;"

# On old primary (now replica, us-east-1a)
# Verify specific high-value tables
sudo -u postgres psql -c "SELECT count(*) FROM orders WHERE created_at > now() - interval '24 hours';"
# Compare with same query on current primary

# Verify checksums
sudo -u postgres pg_amcheck --heapallindexed --dbname=streamvault
```

**Step 4: Decision -- Switch back or keep current primary (Day 2-3)**

Switch back if:
- The original primary has better hardware or location (us-east-1a is closer to most users)
- The team has operational familiarity with the original topology
- Compliance requires specific AZ placement

Keep current primary if:
- The current primary is performing well
- Switching back introduces unnecessary risk
- The original primary's hardware needs replacement

**Step 5: If switching back -- reverse roles (Day 3)**

```bash
# On current primary (us-east-1b)
# Drain connections (set PgBouncer to pause)
psql -h localhost -p 6432 -U admin pgbouncer -c "PAUSE;"

# Wait for in-flight transactions
sleep 30

# Demote current primary
sudo -u postgres psql -c "SELECT pg_demote();"

# On old primary (us-east-1a)
# Promote to primary
sudo -u postgres psql -c "SELECT pg_promote();"

# Update PgBouncer
sudo sed -i 's/host=replica.us-east-1b.internal/host=primary.us-east-1a.internal/' /etc/pgbouncer/pgbouncer.ini
psql -h localhost -p 6432 -U admin pgbouncer -c "RELOAD; UNPAUSE;"

# Verify
sudo -u postgres psql -c "SELECT pg_is_in_recovery();"
# Expected: f (false = primary)
```

### Common Mistakes to Avoid

- Starting the old primary without standby.signal. This causes split-brain immediately.
- Skipping pg_rewind. Without it, the old primary may have divergent data from the period it was down.
- Failing back too quickly. Wait at least 24-48 hours to confirm the current primary is stable.
- Not verifying data consistency. pg_rewind can miss data if the old primary had uncommitted transactions.

---

## Part D: Communication Plan

### Audience Tiers

| Tier | Audience | Notify When | Channel |
|------|----------|-------------|---------|
| Tier 1 | On-call engineer, DBA lead | Immediately (detection) | PagerDuty |
| Tier 2 | Engineering manager, SRE team | During failover (2 min) | Slack #incidents |
| Tier 3 | VP Engineering, Customer Support | After failover complete (5 min) | Email + Slack #exec-incidents |
| Tier 4 | Customers, external stakeholders | If downtime > 5 min | Status page + email |

### Message Templates

**Initial Notification (within 2 minutes of detection):**

```
INCIDENT: Database Primary Unreachable
Severity: P1
Status: Investigating
Impact: Potential service degradation for payment processing

What happened: The primary PostgreSQL database in us-east-1a has been unreachable since 02:47 UTC.
What is the impact: Payment processing may be delayed or unavailable.
What we are doing: Investigating root cause. Failover to standby is being evaluated.
Next update: 02:55 UTC (8 minutes)
Incident commander: [Name]
```

**Status Update (during failover):**

```
INCIDENT UPDATE: Database Failover In Progress
Severity: P1
Status: Failing over
Impact: Payment processing temporarily unavailable (expected 60-90 seconds)

What happened: Primary database confirmed down. Hardware failure in us-east-1a.
What is the impact: Payment processing is currently unavailable. Expected recovery in 60-90 seconds.
What we are doing: Promoting sync replica in us-east-1b to primary.
Next update: 03:00 UTC
Incident commander: [Name]
```

**Resolution Message:**

```
INCIDENT RESOLVED: Database Failover Complete
Severity: P1
Status: Resolved
Duration: 3 minutes 12 seconds (02:47 - 02:50 UTC)
Impact: Payment processing unavailable for 3 minutes 12 seconds.

What happened: Primary database in us-east-1a failed due to hardware failure.
Resolution: Automatic failover to sync replica in us-east-1b completed successfully.
Data loss: Zero (synchronous replication confirmed).
Follow-up: Root cause analysis scheduled for [Date]. Original primary being rebuilt as replica.

Incident commander: [Name]
```

### Escalation Path

```
Failover attempted (0:00)
  |
  +--> Failover successful? (0:90)
  |      |
  |      +--> YES: Send resolution message. Notify Tier 3.
  |
  +--> Failover failed? (0:90)
         |
         +--> Escalate to DBA lead + SRE lead (PagerDuty)
         |
         +--> Attempt manual promotion (2:00)
         |
         +--> Manual promotion successful? (3:00)
         |      |
         |      +--> YES: Send resolution message with "manual intervention required"
         |
         +--> Manual promotion failed? (3:00)
                |
                +--> Escalate to VP Engineering
                +--> Activate cross-region failover to us-west-2
                +--> Notify Tier 4 (customers) of extended outage
```

### Common Mistakes to Avoid

- Sending too many updates. One update every 5-10 minutes is sufficient. More creates noise.
- Using technical jargon in Tier 3/4 messages. "PostgreSQL sync replica promoted" means nothing to a VP.
- Not designating an incident commander. Without one, communication is fragmented and contradictory.
- Forgetting to update the status page. Customers check the status page first -- if it says nothing, they flood support with tickets.

---

## Key Takeaway

A failover procedure is not just a sequence of commands -- it is a decision framework that balances speed against safety. The runbook provides the "how," the decision matrix provides the "when," the failback plan provides the "how to return to normal," and the communication plan provides the "who needs to know." All four are essential for a complete DR capability.
