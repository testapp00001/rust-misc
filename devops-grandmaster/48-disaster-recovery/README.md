# 48 - Disaster Recovery

**Previous:** [47 - Backup & Restore](../47-backup-restore/README.md) | **Next:** [49 - High Availability](../49-high-availability/README.md)

---

## Problem

Your primary datacenter suffers a catastrophic failure. A fire, flood, ransomware attack, or cloud region outage takes your entire production environment offline. Customers cannot access your service. Revenue bleeds by the minute. Without a tested disaster recovery plan, your team scrambles in panic, restores from week-old backups, and loses days of data.

Disaster recovery answers two critical questions:
- **RPO (Recovery Point Objective):** How much data can we afford to lose? 1 hour? 5 minutes? Zero?
- **RTO (Recovery Time Objective):** How long can we be down? 4 hours? 15 minutes? Zero seconds?

These two numbers drive every architectural decision. A startup accepting 4-hour RTO and 1-hour RPO has a radically different setup than a bank requiring zero data loss and sub-minute failover.

---

## Naive Way

```bash
# "We have backups somewhere... I think?"
scp -r /var/lib/mysql user@backup-server:/backups/manual/

# Hope nobody notices if the server dies
# When it dies, restore from the last manual backup
# Realize the backup is 3 weeks old
# Panic
```

**Why this fails:**
- Backups are manual, inconsistent, and untested
- No defined RPO or RTO
- No documented procedure for failover
- Backup integrity is never verified
- The "backup server" is in the same building as production

---

## Right Way

### Define RPO and RTO First

Before writing any code or provisioning any infrastructure, answer these questions with stakeholders:

| System Tier | RPO | RTO | Strategy |
|-------------|-----|-----|----------|
| Critical (payments, auth) | 0 (zero data loss) | < 5 min | Active-active, synchronous replication |
| Important (user data, orders) | < 15 min | < 1 hour | Cross-region replica, automated failover |
| Non-critical (analytics, logs) | < 24 hours | < 24 hours | Daily backups to separate region |

### Backup Strategy with RPO Alignment

```bash
#!/bin/bash
# backup-strategy.sh - Tiered backup approach

DB_HOST="primary-db.internal"
BACKUP_DIR="/backups/postgres"
S3_BUCKET="s3://company-dr-backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Continuous WAL archiving for point-in-time recovery (RPO ~0)
archive_wal() {
    pg_basebackup -h "$DB_HOST" -D "$BACKUP_DIR/base" --wal-method=stream
    aws s3 sync "$BACKUP_DIR/wal" "$S3_BUCKET/wal/" --storage-class STANDARD_IA
}

# Hourly incremental snapshots (RPO ~1 hour)
hourly_snapshot() {
    pg_dump -h "$DB_HOST" -Fc dbname > "$BACKUP_DIR/hourly_${TIMESTAMP}.dump"
    aws s3 cp "$BACKUP_DIR/hourly_${TIMESTAMP}.dump" "$S3_BUCKET/hourly/"
}

# Daily full backup (RPO ~24 hours, long-term retention)
daily_full() {
    pg_basebackup -h "$DB_HOST" -D "$BACKUP_DIR/daily_${TIMESTAMP}" --format=tar --gzip
    aws s3 cp "$BACKUP_DIR/daily_${TIMESTAMP}.tar.gz" "$S3_BUCKET/daily/"
    # Retain daily backups for 30 days
    aws s3 ls "$S3_BUCKET/daily/" | while read -r line; do
        createDate=$(echo "$line" | awk '{print $1" "$2}')
        createDate=$(date -d "$createDate" +%s)
        olderThan=$(date -d "30 days ago" +%s)
        if [[ $createDate -lt $olderThan ]]; then
            fileName=$(echo "$line" | awk '{print $4}')
            aws s3 rm "$S3_BUCKET/daily/$fileName"
        fi
    done
}
```

### Failover Runbook

```bash
#!/bin/bash
# failover.sh - Automated failover procedure

set -euo pipefail

PRIMARY_REGION="us-east-1"
DR_REGION="us-west-2"
DB_PRIMARY="primary-db.us-east-1.rds.amazonaws.com"
DB_REPLICA="replica-db.us-west-2.rds.amazonaws.com"

echo "=== DISASTER RECOVERY FAILOVER ==="
echo "Primary: $PRIMARY_REGION -> DR: $DR_REGION"
echo "Time: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

# Step 1: Verify primary is actually down (prevent split-brain)
echo "[1/6] Verifying primary is unreachable..."
if pg_isready -h "$DB_PRIMARY" -t 5 2>/dev/null; then
    echo "ERROR: Primary is still responding! Aborting failover."
    echo "If you are sure, run with --force flag."
    exit 1
fi

# Step 2: Promote replica to primary
echo "[2/6] Promoting replica to primary..."
aws rds promote-read-replica --db-instance-identifier replica-db-us-west-2

# Step 3: Wait for promotion to complete
echo "[3/6] Waiting for promotion..."
aws rds wait db-instance-available --db-instance-identifier replica-db-us-west-2

# Step 4: Update DNS to point to DR region
echo "[4/6] Updating DNS records..."
aws route53 change-resource-record-sets \
    --hosted-zone-id Z1234567890 \
    --change-batch '{
        "Changes": [{
            "Action": "UPSERT",
            "ResourceRecordSet": {
                "Name": "api.company.com",
                "Type": "CNAME",
                "TTL": 60,
                "ResourceRecords": [{"Value": "dr-lb.us-west-2.elb.amazonaws.com"}]
            }
        }]
    }'

# Step 5: Scale up DR application servers
echo "[5/6] Scaling DR application tier..."
aws ecs update-service --cluster dr-cluster --service api-service --desired-count 3 \
    --region "$DR_REGION"

# Step 6: Notify team
echo "[6/6] Sending notifications..."
curl -X POST "$SLACK_WEBHOOK" -H 'Content-type: application/json' \
    -d '{"text":"DISASTER RECOVERY: Failover to us-west-2 complete."}'

echo ""
echo "Failover complete. Monitor DR region closely for the next 24 hours."
```

### Failback Procedure

```bash
#!/bin/bash
# failback.sh - Returning to primary after recovery

echo "=== FAILOVER RECOVERY - RETURN TO PRIMARY ==="

# Step 1: Set DR database as read-only
echo "[1/5] Setting DR database to read-only..."
psql -h "$DB_REPLICA" -c "ALTER SYSTEM SET default_transaction_read_only = on;"
psql -h "$DB_REPLICA" -c "SELECT pg_reload_conf();"

# Step 2: Wait for in-flight transactions
echo "[2/5] Waiting 30s for in-flight transactions..."
sleep 30

# Step 3: Replicate DR data back to primary
echo "[3/5] Replicating data back to primary region..."
pg_dump -h "$DB_REPLICA" dbname | psql -h "$DB_PRIMARY" dbname

# Step 4: Switch DNS back to primary
echo "[4/5] Switching DNS back to primary..."
# (same Route53 command pointing to primary)

# Step 5: Re-enable writes on primary
echo "[5/5] Enabling writes on primary..."
psql -h "$DB_PRIMARY" -c "ALTER SYSTEM SET default_transaction_read_only = off;"
psql -h "$DB_PRIMARY" -c "SELECT pg_reload_conf();"

echo "Failback complete. Monitor primary region closely."
```

---

## Production Way

### Multi-Region Active-Active with Automatic Failover

```
                    +------------------+
                    |   Global Load    |
                    |    Balancer      |
                    |  (Route53/GSLB)  |
                    +--------+---------+
                             |
                   Health checks every 10s
                             |
              +--------------+--------------+
              |                             |
    +---------v---------+       +-----------v-------+
    |   Region A         |       |   Region B         |
    |   (Primary)        |       |   (Secondary)      |
    |                    |       |                    |
    |  +-----------+     |       |  +-----------+     |
    |  | App Servers|    |       |  | App Servers|    |
    |  +-----+-----+   |       |  +-----+-----+   |
    |        |          |       |        |          |
    |  +-----v-----+   |sync   |  +-----v-----+   |
    |  | Database   +---+------>+  | Database   |   |
    |  | Primary    |   | repl  |  | Replica    |   |
    |  +------------+   |       |  +------------+   |
    +--------------------+       +-------------------+
```

### Terraform DR Configuration

```hcl
# dr-infrastructure.tf

provider "aws" {
  alias  = "primary"
  region = "us-east-1"
}

provider "aws" {
  alias  = "dr"
  region = "us-west-2"
}

# Primary RDS with automated backups
resource "aws_db_instance" "primary" {
  provider               = aws.primary
  identifier             = "app-primary"
  engine                 = "postgres"
  engine_version         = "15.4"
  instance_class         = "db.r6g.xlarge"
  allocated_storage      = 100
  db_name                = "appdb"
  username               = "admin"
  password               = var.db_password
  multi_az               = true
  backup_retention_period = 7
  backup_window          = "03:00-04:00"
  storage_encrypted      = true
  deletion_protection    = true

  tags = {
    Environment = "production"
    DR-Role     = "primary"
  }
}

# Cross-region read replica
resource "aws_db_instance" "replica" {
  provider               = aws.dr
  identifier             = "app-replica"
  replicate_source_db    = aws_db_instance.primary.arn
  instance_class         = "db.r6g.xlarge"
  storage_encrypted      = true
  deletion_protection    = true

  tags = {
    Environment = "production"
    DR-Role     = "replica"
  }
}

# Route53 health check for primary
resource "aws_route53_health_check" "primary" {
  fqdn              = "api-primary.company.com"
  port               = 443
  type               = "HTTPS"
  resource_path      = "/health"
  failure_threshold  = 3
  request_interval   = 10
}

# DNS failover record
resource "aws_route53_record" "api" {
  zone_id = var.dns_zone_id
  name    = "api.company.com"
  type    = "CNAME"
  ttl     = 60

  failover_routing_policy {
    type = "PRIMARY"
  }

  set_identifier  = "primary"
  records         = ["api-primary.company.com"]
  health_check_id = aws_route53_health_check.primary.id
}

resource "aws_route53_record" "api_dr" {
  zone_id = var.dns_zone_id
  name    = "api.company.com"
  type    = "CNAME"
  ttl     = 60

  failover_routing_policy {
    type = "SECONDARY"
  }

  set_identifier = "dr"
  records        = ["api-dr.company.com"]
}
```

### DR Testing Automation

```python
# dr_test.py - Automated DR drill runner
import subprocess
import time
import json
import sys
from datetime import datetime

class DRDrill:
    def __init__(self, config_path="dr-config.json"):
        with open(config_path) as f:
            self.config = json.load(f)
        self.results = []

    def log(self, step, status, detail=""):
        entry = {
            "timestamp": datetime.utcnow().isoformat(),
            "step": step,
            "status": status,
            "detail": detail
        }
        self.results.append(entry)
        symbol = "PASS" if status == "pass" else "FAIL"
        print(f"[{symbol}] {step}: {detail}")

    def test_backup_restore(self):
        """Verify we can restore from backup within RTO."""
        start = time.time()
        result = subprocess.run(
            ["./scripts/restore-to-test.sh"],
            capture_output=True, text=True, timeout=3600
        )
        elapsed = time.time() - start
        rto_seconds = self.config["rto_critical_seconds"]

        if result.returncode == 0 and elapsed < rto_seconds:
            self.log("Backup Restore", "pass",
                     f"Restored in {elapsed:.0f}s (RTO: {rto_seconds}s)")
        else:
            self.log("Backup Restore", "fail",
                     f"Took {elapsed:.0f}s, exceeds RTO of {rto_seconds}s")

    def test_replica_promotion(self):
        """Verify replica can be promoted."""
        result = subprocess.run(
            ["aws", "rds", "describe-db-instances",
             "--db-instance-identifier", self.config["replica_id"],
             "--query", "DBInstances[0].Status"],
            capture_output=True, text=True
        )
        status = json.loads(result.stdout)
        if status == "available":
            self.log("Replica Health", "pass", "Replica is available")
        else:
            self.log("Replica Health", "fail", f"Replica status: {status}")

    def test_dns_failover(self):
        """Verify DNS failover would work."""
        result = subprocess.run(
            ["aws", "route53", "get-health-check-status",
             "--health-check-id", self.config["health_check_id"]],
            capture_output=True, text=True
        )
        self.log("DNS Failover", "pass", "Health check configured correctly")

    def test_data_integrity(self):
        """Verify restored data matches expected state."""
        result = subprocess.run(
            ["psql", "-h", self.config["test_db_host"],
             "-c", "SELECT COUNT(*) FROM critical_table;"],
            capture_output=True, text=True
        )
        count = int(result.stdout.strip().split('\n')[-1].strip())
        expected = self.config.get("expected_min_rows", 1000)

        if count >= expected:
            self.log("Data Integrity", "pass",
                     f"Found {count} rows (expected >= {expected})")
        else:
            self.log("Data Integrity", "fail",
                     f"Found {count} rows (expected >= {expected})")

    def run_full_drill(self):
        print("=" * 60)
        print("DISASTER RECOVERY DRILL")
        print(f"Started: {datetime.utcnow().isoformat()}")
        print("=" * 60)

        self.test_backup_restore()
        self.test_replica_promotion()
        self.test_dns_failover()
        self.test_data_integrity()

        passed = sum(1 for r in self.results if r["status"] == "pass")
        total = len(self.results)

        print("=" * 60)
        print(f"RESULTS: {passed}/{total} passed")
        if passed < total:
            print("DR DRILL FAILED - Review failures before next drill")
            sys.exit(1)
        else:
            print("DR DRILL PASSED - System meets RPO/RTO requirements")

if __name__ == "__main__":
    drill = DRDrill()
    drill.run_full_drill()
```

### Scheduled DR Drills in CI/CD

```yaml
# .github/workflows/dr-drill.yml
name: Monthly DR Drill
on:
  schedule:
    - cron: '0 6 1 * *'  # 1st of every month at 6am UTC
  workflow_dispatch:

jobs:
  dr-drill:
    runs-on: ubuntu-latest
    environment: dr-drill
    steps:
      - uses: actions/checkout@v4

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          role-to-assume: ${{ secrets.DR_DRILL_ROLE_ARN }}
          aws-region: us-west-2

      - name: Run DR Drill
        run: python scripts/dr_test.py
        env:
          DR_CONFIG: ${{ secrets.DR_CONFIG }}

      - name: Notify results
        if: always()
        run: |
          STATUS=${{ job.status }}
          curl -X POST "$SLACK_WEBHOOK" \
            -H 'Content-type: application/json' \
            -d "{\"text\":\"Monthly DR Drill: $STATUS\"}"
```

---

## Hands-On Lab

### Lab: Implement and Test a Complete DR Plan

**Duration:** 90 minutes

**Prerequisites:**
- Two AWS regions configured (or two Docker Compose environments simulating regions)
- PostgreSQL running in "primary" region
- Basic familiarity with database replication

**Objectives:**
1. Set up cross-region database replication
2. Create an automated failover script
3. Define and verify RPO/RTO for your application
4. Execute a DR drill and measure actual recovery metrics

**Step 1: Set Up Primary Database with WAL Archiving**

```bash
# Enable WAL archiving in postgresql.conf
cat >> postgresql.conf <<EOF
archive_mode = on
archive_command = 'cp %p /archive/%f'
wal_level = replica
max_wal_senders = 3
EOF

pg_ctl restart

# Create replication user
psql -c "CREATE USER replicator WITH REPLICATION ENCRYPTED 'password';"
```

**Step 2: Create a Replica in a Separate Environment**

```bash
# Take a base backup from primary
pg_basebackup -h primary-db -D /var/lib/postgresql/replica \
    -U replicator -Fp -Xs -P -R

# Start the replica
pg_ctl -D /var/lib/postgresql/replica start

# Verify replication is working
psql -h primary-db -c "SELECT * FROM pg_stat_replication;"
```

**Step 3: Simulate Data and Measure RPO**

```bash
# On primary, insert test data continuously
while true; do
    psql -h primary-db -c \
        "INSERT INTO test_data (value, created_at) VALUES (random(), now());"
    sleep 1
done

# On replica, check replication lag
psql -h replica-db -c "SELECT now() - pg_last_xact_replay_timestamp() AS lag;"
```

**Step 4: Execute Failover**

```bash
# Stop primary to simulate disaster
docker stop primary-db

# Promote replica
psql -h replica-db -c "SELECT pg_promote();"

# Verify replica is now writable
psql -h replica-db -c "INSERT INTO test_data (value) VALUES ('after-failover');"

# Measure RPO: how much data was lost?
psql -h replica-db -c "SELECT MAX(created_at) FROM test_data;"
```

**Step 5: Verify and Document**

Record your findings:
- Actual RPO achieved: _____ seconds
- Actual RTO achieved: _____ seconds
- Issues encountered: _____
- Changes needed to meet business requirements: _____

---

## Limitation

Disaster recovery handles **restoring service after a catastrophe**, but it does not prevent downtime in the first place. If your primary database goes down and you need 5 minutes to fail over, those 5 minutes of downtime are still lost revenue.

DR also has inherent data loss risk tied to your RPO. If your RPO is 15 minutes and a disaster strikes, you lose up to 15 minutes of data. For many applications this is acceptable, but for financial transactions or healthcare records, even seconds of data loss is unacceptable.

**The fundamental limitation:** DR is reactive. It responds to failures after they happen. For zero-downtime and zero-data-loss guarantees, you need **high availability** -- systems that continue operating even when components fail, with no manual intervention required.

---

## Next Topic

[49 - High Availability](../49-high-availability/README.md) -- Learn how to build systems with zero downtime using active-passive and active-active architectures, consensus protocols like Raft, and automated failover that happens in seconds, not minutes.
