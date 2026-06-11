# Solution 05: Multi-Region DR Architecture

## Part A: Multi-Region Topology Design

### Architecture Diagram

```
                    +------------------------------------------+
                    |           Route 53 (Global DNS)          |
                    |   Latency-based routing + failover       |
                    +----+----------------+----------------+---+
                         |                |                |
                         v                v                v
               +------------------+  +------------------+  +------------------+
               |   us-east-1      |  |   eu-west-1      |  | ap-southeast-1   |
               |   (PRIMARY)      |  |   (WARM STANDBY) |  |  (WARM STANDBY)  |
               |                  |  |                  |  |                  |
               | +--------------+ |  | +--------------+ |  | +--------------+ |
               | | EKS Cluster  | |  | | EKS Cluster  | |  | | EKS Cluster  | |
               | | (50 svc,     | |  | | (50 svc,     | |  | | (50 svc,     | |
               | |  scaled=0)   | |  | |  scaled=0)   | |  | |  scaled=0)   | |
               | +------+-------+ |  | +------+-------+ |  | +------+-------+ |
               |        |         |  |        |         |  |        |         |
               | +------+-------+ |  | +------+-------+ |  | +------+-------+ |
               | | PgBouncer    | |  | | PgBouncer    | |  | | PgBouncer    | |
               | +------+-------+ |  | +------+-------+ |  | +------+-------+ |
               |        |         |  |        |         |  |        |         |
               | +------+-------+ |  | +------+-------+ |  | +------+-------+ |
               | | Aurora       | |  | | Aurora       | |  | | Aurora       | |
               | | PostgreSQL   | |  | | PostgreSQL   | |  | | PostgreSQL   | |
               | | (WRITER)     | |  | | (READER)     | |  | | (READER)     | |
               | +------+-------+ |  | +------+-------+ |  | +------+-------+ |
               |        |         |  |        ^         |  |        ^         |
               +--------|---------+  +--------|---------+  +--------|---------+
                        |                     |                     |
                        |  Aurora Global DB Replication (<1s lag)   |
                        +---------------------+---------------------+
                                              |
                        +---------------------+---------------------+
                        |                     |                     |
               +--------|---------+  +--------|---------+  +--------|---------+
               | +------+-------+ |  | +------+-------+ |  | +------+-------+ |
               | | Redis 7      | |  | | Redis 7      | |  | | Redis 7      | |
               | | (PRIMARY)    | |  | | (REPLICA)    | |  | | (REPLICA)    | |
               | +--------------+ |  | +--------------+ |  | +--------------+ |
               |                  |  |                  |  |                  |
               | +--------------+ |  | +--------------+ |  | +--------------+ |
               | | S3 Bucket    | |  | | S3 Bucket    | |  | | S3 Bucket    | |
               | | (us-east-1)  | |  | | (eu-west-1)  | |  | | (ap-se-1)    | |
               | +--------------+ |  | +--------------+ |  | +--------------+ |
               |  CRR enabled -->|--+  |                  |  |                  |
               +------------------+  +------------------+  +------------------+
```

### PostgreSQL Replication Strategy

Aurora Global Database is the correct choice here, not self-managed streaming replication.

| Aspect | Self-Managed Streaming Replication | Aurora Global Database |
|--------|-----------------------------------|----------------------|
| Cross-region lag | 1-5 seconds (depends on WAN) | <1 second (dedicated infrastructure) |
| Failover automation | Manual or Patroni (complex) | Built-in (1-2 minutes) |
| Operational overhead | High (etcd, Patroni, pg_basebackup) | Low (managed service) |
| Cost | Lower (EC2 only) | Higher (Aurora pricing) |
| GDPR compliance | Requires manual filtering | Requires logical replication layer |

Aurora Global Database configuration:

```yaml
# CloudFormation / Terraform snippet for Aurora Global Database

Resources:
  GlobalDBCluster:
    Type: AWS::RDS::GlobalCluster
    Properties:
      GlobalClusterIdentifier: helios-global-cluster
      Engine: aurora-postgresql
      EngineVersion: "15.4"
      DeletionProtection: true
      StorageEncrypted: true

  PrimaryCluster:
    Type: AWS::RDS::DBCluster
    Properties:
      DBClusterIdentifier: helios-us-east-1
      Engine: aurora-postgresql
      EngineVersion: "15.4"
      GlobalClusterIdentifier: !Ref GlobalDBCluster
      MasterUsername: !Ref DBMasterUsername
      MasterUserPassword: !Ref DBMasterPassword
      StorageEncrypted: true
      EnableCloudwatchLogsExports:
        - postgresql
      BackupRetentionPeriod: 35
      DeletionProtection: true

  PrimaryWriter:
    Type: AWS::RDS::DBInstance
    Properties:
      DBClusterIdentifier: !Ref PrimaryCluster
      DBInstanceClass: db.r6g.2xlarge
      Engine: aurora-postgresql
      PubliclyAccessible: false

  PrimaryReader:
    Type: AWS::RDS::DBInstance
    Properties:
      DBClusterIdentifier: !Ref PrimaryCluster
      DBInstanceClass: db.r6g.2xlarge
      Engine: aurora-postgresql
      PubliclyAccessible: false

  EuWestCluster:
    Type: AWS::RDS::DBCluster
    Properties:
      DBClusterIdentifier: helios-eu-west-1
      Engine: aurora-postgresql
      EngineVersion: "15.4"
      GlobalClusterIdentifier: !Ref GlobalDBCluster
      StorageEncrypted: true
      EnableCloudwatchLogsExports:
        - postgresql

  EuWestReader:
    Type: AWS::RDS::DBInstance
    Properties:
      DBClusterIdentifier: !Ref EuWestCluster
      DBInstanceClass: db.r6g.xlarge
      Engine: aurora-postgresql
      PubliclyAccessible: false

  ApSeCluster:
    Type: AWS::RDS::DBCluster
    Properties:
      DBClusterIdentifier: helios-ap-southeast-1
      Engine: aurora-postgresql
      EngineVersion: "15.4"
      GlobalClusterIdentifier: !Ref GlobalDBCluster
      StorageEncrypted: true
      EnableCloudwatchLogsExports:
        - postgresql

  ApSeReader:
    Type: AWS::RDS::DBInstance
    Properties:
      DBClusterIdentifier: !Ref ApSeCluster
      DBInstanceClass: db.r6g.xlarge
      Engine: aurora-postgresql
      PubliclyAccessible: false
```

Why Aurora Global Database over self-managed replication:

- Aurora replicates at the storage layer, not the WAL layer, achieving sub-second cross-region lag without tuning.
- Failover is a single API call (`failover-global-cluster`) rather than a multi-step Patroni promotion.
- Storage-level replication means no `max_wal_senders`, no replication slots, no slot bloat management.
- The managed service handles the hardest part: ensuring the secondary cluster is crash-consistent after promotion.

### Redis Cross-Region Replication

Redis Global Datastore provides cross-region replication for ElastiCache Redis.

```yaml
# Terraform for Redis Global Datastore

resource "aws_elasticache_global_replication_group" "helios" {
  global_replication_group_id_suffix = "helios-global"
  primary_replication_group_id       = aws_elasticache_replication_group.us_east.id
  description                        = "Helios global Redis replication group"

  global_node_group_count = 1
}

resource "aws_elasticache_replication_group" "us_east" {
  replication_group_id = "helios-redis-us-east-1"
  description          = "Primary Redis cluster in us-east-1"
  node_type            = "cache.r6g.xlarge"
  num_cache_clusters   = 3
  engine               = "redis"
  engine_version       = "7.0"
  port                 = 6379

  automatic_failover_enabled = true
  multi_az_enabled           = true
  at_rest_encryption_enabled = true
  transit_encryption_enabled = true

  snapshot_retention_limit = 7
  snapshot_window          = "03:00-05:00"
  maintenance_window       = "sun:05:00-sun:07:00"
}

resource "aws_elasticache_replication_group" "eu_west" {
  replication_group_id          = "helios-redis-eu-west-1"
  description                   = "Secondary Redis cluster in eu-west-1"
  node_type                     = "cache.r6g.xlarge"
  num_cache_clusters            = 2
  engine                        = "redis"
  engine_version                = "7.0"
  port                          = 6379
  global_replication_group_id   = aws_elasticache_global_replication_group.helios.id

  automatic_failover_enabled = true
  at_rest_encryption_enabled = true
  transit_encryption_enabled = true
}

resource "aws_elasticache_replication_group" "ap_se" {
  replication_group_id          = "helios-redis-ap-southeast-1"
  description                   = "Secondary Redis cluster in ap-southeast-1"
  node_type                     = "cache.r6g.xlarge"
  num_cache_clusters            = 2
  engine                        = "redis"
  engine_version                = "7.0"
  port                          = 6379
  global_replication_group_id   = aws_elasticache_global_replication_group.helios.id

  automatic_failover_enabled = true
  at_rest_encryption_enabled = true
  transit_encryption_enabled = true
}
```

Session handling during failover:

- Redis Global Datastore replicates asynchronously with sub-second lag.
- If the primary region fails, sessions in transit (replicated but not yet applied) are lost.
- Application must handle session loss gracefully: redirect to login with a "session expired" message.
- Session TTL should be set to 24 hours, and session tokens should be JWTs with the region encoded, so after failover the application can detect and re-issue sessions.

### GDPR Data Residency Strategy

GDPR requires that EU user data remains in eu-west-1 unless the user explicitly consents to cross-region transfer. This constrains the replication design.

```sql
-- Table partitioning strategy for GDPR compliance

-- All tables that contain PII get a region_code column
ALTER TABLE users ADD COLUMN region_code VARCHAR(10) NOT NULL DEFAULT 'us';
ALTER TABLE documents ADD COLUMN region_code VARCHAR(10) NOT NULL DEFAULT 'us';
ALTER TABLE messages ADD COLUMN region_code VARCHAR(10) NOT NULL DEFAULT 'us';

-- Create a view that filters by region for each regional cluster
-- On eu-west-1:
CREATE OR REPLACE VIEW regional_users AS
SELECT * FROM users
WHERE region_code = 'eu'
   OR (region_code = 'us' AND user_consent_global = true);

-- Row-level security policy (enforced at database level)
ALTER TABLE users ENABLE ROW LEVEL SECURITY;

CREATE POLICY eu_residency ON users
    FOR ALL
    USING (
        region_code != 'eu'
        OR current_setting('app.region', true) = 'eu'
        OR user_consent_global = true
    );
```

The replication strategy for GDPR:

| Data Category | us-east-1 (primary) | eu-west-1 | ap-southeast-1 |
|--------------|---------------------|-----------|----------------|
| US user data | Full (read/write) | Full (read) | Full (read) |
| EU user data | Encrypted blob only (no PII) | Full (read/write) | Encrypted blob only |
| APAC user data | Full (read/write) | Full (read) | Full (read/write) |
| Non-PII metadata | Full | Full | Full |

For EU user data, Aurora Global Database replicates the full dataset to all regions, but the application layer and row-level security policies prevent PII access outside eu-west-1. The encrypted blob in us-east-1 allows disaster recovery of EU data if eu-west-1 fails, but requires re-encryption with region-specific keys before the data is readable.

### Kubernetes Workload Strategy

Each region runs an EKS cluster in warm-standby mode (pilot light), not active-active.

| Region | EKS State | Services | Scaling | Purpose |
|--------|-----------|----------|---------|---------|
| us-east-1 | Active | 50/50 running | Normal (auto-scaling) | Serves all traffic |
| eu-west-1 | Warm standby | 50/50 running, scaled to 0 | Scale to 100% on failover | DR for EU + global |
| ap-southeast-1 | Warm standby | 50/50 running, scaled to 0 | Scale to 100% on failover | DR for APAC |

Why warm standby over active-active:

- Active-active requires conflict resolution for database writes. With PostgreSQL, this means either multi-master (CockroachDB, Citus) or application-level sharding -- both are major architectural changes.
- Warm standby with Aurora Global Database is a proven pattern that achieves the 5-minute RTO without the complexity of multi-master writes.
- The warm standby EKS cluster has all deployments running but scaled to 0 replicas. On failover, a single `kubectl scale` command brings all services online.

```yaml
# Helm values for warm standby region
# All deployments are present but replicas=0

global:
  region: eu-west-1
  standby: true

api-gateway:
  replicaCount: 0

payment-service:
  replicaCount: 0

user-service:
  replicaCount: 0

# ... all 50 services with replicaCount: 0
```

### Common Mistakes to Avoid

1. **Using self-managed PostgreSQL streaming replication across regions.** WAN latency and operational complexity make this fragile. Aurora Global Database is purpose-built for this.
2. **Replicating all data to all regions without considering GDPR.** EU PII cannot be accessible in us-east-1 or ap-southeast-1 without explicit user consent.
3. **Running active-active PostgreSQL without a clear conflict resolution strategy.** Multi-master databases sound appealing but introduce CAP theorem tradeoffs that are hard to reason about.
4. **Forgetting that warm standby EKS clusters need regular validation.** A cluster scaled to 0 that has not been tested may have stale container images, expired IAM roles, or misconfigured networking.
5. **Assuming Redis session loss is acceptable everywhere.** For payment flows, losing a session mid-transaction can cause double-charging. Session loss handling must be designed per use case.

---

## Part B: Automated Failover Triggers

### Region Health Checks

A region being "down" is different from a node being "down." Region health must be measured from outside the region to avoid false negatives -- if us-east-1 monitoring reports itself healthy, that means nothing if the network is partitioned.

```bash
#!/bin/bash
# /usr/local/bin/region-health-check.sh
# Runs from an external monitoring region (e.g., eu-west-1 checks us-east-1)
# Requires: curl, jq, aws cli

set -euo pipefail

REGION="$1"
ENDPOINT="https://api.${REGION}.helios.io/health"
METRICS_ENDPOINT="https://prometheus.${REGION}.helios.io/api/v1/query"

check_region_health() {
    local region="$1"
    local score=0
    local max_score=5

    # Check 1: API endpoint reachable (HTTP 200)
    local http_code
    http_code=$(curl -s -o /dev/null -w "%{http_code}" \
        --connect-timeout 10 --max-time 15 "${ENDPOINT}" 2>/dev/null) || http_code="000"
    if [ "${http_code}" = "200" ]; then
        score=$((score + 1))
    fi

    # Check 2: API latency < 2x normal (normal = 50ms, threshold = 100ms)
    local latency_ms
    latency_ms=$(curl -s -o /dev/null -w "%{time_total}" \
        --connect-timeout 10 --max-time 15 "${ENDPOINT}" 2>/dev/null | \
        awk '{printf "%.0f", $1 * 1000}') || latency_ms="9999"
    if [ "${latency_ms}" -lt 100 ]; then
        score=$((score + 1))
    fi

    # Check 3: Database writer available
    local db_healthy
    db_healthy=$(curl -s --connect-timeout 10 \
        "${METRICS_ENDPOINT}?query=aurora_replica_lag" 2>/dev/null | \
        jq -r '.data.result[0].value[1] // "0"' 2>/dev/null) || db_healthy="0"
    if [ "${db_healthy}" != "0" ]; then
        score=$((score + 1))
    fi

    # Check 4: At least 50% of pods healthy
    local pod_health_pct
    pod_health_pct=$(curl -s --connect-timeout 10 \
        "${METRICS_ENDPOINT}?query=up{job=\"kubernetes-pods\"}" 2>/dev/null | \
        jq '[.data.result[].value[1] | tonumber] | add / length * 100 | floor' \
        2>/dev/null) || pod_health_pct="0"
    if [ "${pod_health_pct}" -ge 50 ]; then
        score=$((score + 1))
    fi

    # Check 5: Redis reachable
    local redis_ok
    redis_ok=$(curl -s -o /dev/null -w "%{http_code}" \
        --connect-timeout 5 \
        "https://redis-health.${region}.helios.io/ping" 2>/dev/null) || redis_ok="000"
    if [ "${redis_ok}" = "200" ]; then
        score=$((score + 1))
    fi

    echo "${region} ${score}/${max_score}"

    if [ "${score}" -lt 3 ]; then
        return 1  # Region unhealthy
    fi
    return 0
}

check_region_health "${REGION}"
```

### Threshold Logic

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Health check interval | 30 seconds | Frequent enough to detect failure within 2 minutes |
| Consecutive failures required | 3 | Prevents false positives from transient network issues |
| Independent monitoring sources | 2 | Two regions must agree that the target is down |
| Failure window | 90 seconds | 3 failures x 30 seconds = 90 seconds before failover triggers |
| Cooldown period | 15 minutes | Prevents failover flapping |

The decision requires agreement from 2 independent sources:

```
us-east-1 health (from eu-west-1): FAIL x3
us-east-1 health (from ap-southeast-1): FAIL x3
  --> Both agree: trigger failover

us-east-1 health (from eu-west-1): FAIL x3
us-east-1 health (from ap-southeast-1): PASS x3
  --> Disagree: do NOT trigger failover (partial network issue)
```

### Failover Decision Pseudocode

```
function evaluateFailover(region):
    // Collect health check results from both monitoring regions
    checks_eu = getHealthChecks(source="eu-west-1", target=region)
    checks_ap = getHealthChecks(source="ap-southeast-1", target=region)

    // Count consecutive failures from each source
    eu_failures = countConsecutiveFailures(checks_eu, window=3)
    ap_failures = countConsecutiveFailures(checks_ap, window=3)

    // Both monitoring regions must agree
    if eu_failures >= 3 AND ap_failures >= 3:
        // Check cooldown
        lastFailover = getLastFailoverTime(region)
        if now() - lastFailover < COOLDOWN_PERIOD:
            log("Failover suppressed: cooldown period active")
            return NO_FAILOVER

        // Determine target region
        candidateRegions = getHealthyRegions(exclude=[region])
        if candidateRegions is empty:
            log("CRITICAL: No healthy region available for failover")
            alert("CRITICAL", "No failover target available")
            return NO_FAILOVER

        targetRegion = selectBestTarget(candidateRegions, criteria=[
            "data_replication_lag",   // Prefer lowest lag
            "geographic_proximity",   // Prefer closest to users
            "capacity_available"      // Prefer region with most headroom
        ])

        // Verify target can absorb traffic
        if NOT canAbsorbTraffic(targetRegion, multiplier=3.0):
            log("Target region cannot absorb full traffic load")
            alert("WARNING", "Failover target has insufficient capacity")
            // Proceed anyway -- degraded service is better than no service

        // Execute failover
        log("FAILOVER TRIGGERED: {region} -> {targetRegion}")
        executeFailover(region, targetRegion)
        return FAILOVER_EXECUTED

    else:
        // Partial failure -- investigate but do not fail over
        if eu_failures >= 3 OR ap_failures >= 3:
            alert("WARNING", "Partial health check failure for {region}")
        return NO_FAILOVER

function executeFailover(sourceRegion, targetRegion):
    // Step 1: Promote Aurora secondary in target region
    aws rds failover-global-cluster \
        --global-cluster-identifier helios-global-cluster \
        --target-db-cluster-identifier helios-{targetRegion}

    // Step 2: Update DNS
    aws route53 change-resource-record-sets \
        --hosted-zone-id Z1234 \
        --change-batch file://failover-dns-{targetRegion}.json

    // Step 3: Scale up EKS in target region
    for service in $(kubectl --context={targetRegion} get deploy -o name):
        kubectl --context={targetRegion} scale $service --replicas=3

    // Step 4: Update API gateway routing
    updateAPIGatewayRouting(primary=targetRegion)

    // Step 5: Record failover event
    recordFailoverEvent(sourceRegion, targetRegion, timestamp=now())
```

### Authorization Model

| Failover Type | Authorization | Latency |
|--------------|---------------|---------|
| Single-region automated | Fully automated, no human approval | <2 minutes |
| Multi-region (primary down) | Fully automated, notify on-call after | <2 minutes |
| Planned cutover | Requires 2-person approval (SRE lead + DBA lead) | 5-15 minutes |
| Failback to original region | Requires 4-person approval + 24-hour soak time | 24+ hours |

Automated failover is the correct default for regional outages. Requiring human approval adds 5-15 minutes to RTO, which exceeds the 5-minute target. The failover script notifies on-call engineers immediately after executing, so humans are informed but not in the critical path.

### Cooldown Period

```python
#!/usr/bin/env python3
# /usr/local/bin/failover-cooldown.py

import time
import json
import redis

COOLDOWN_SECONDS = 900  # 15 minutes
REDIS_KEY_PREFIX = "helios:failover:cooldown"


class FailoverCooldown:
    def __init__(self, redis_client):
        self.redis = redis_client

    def is_in_cooldown(self, region: str) -> bool:
        key = f"{REDIS_KEY_PREFIX}:{region}"
        last_failover = self.redis.get(key)
        if last_failover is None:
            return False

        elapsed = time.time() - float(last_failover)
        return elapsed < COOLDOWN_SECONDS

    def get_remaining_cooldown(self, region: str) -> int:
        key = f"{REDIS_KEY_PREFIX}:{region}"
        last_failover = self.redis.get(key)
        if last_failover is None:
            return 0

        elapsed = time.time() - float(last_failover)
        remaining = COOLDOWN_SECONDS - elapsed
        return max(0, int(remaining))

    def record_failover(self, region: str):
        key = f"{REDIS_KEY_PREFIX}:{region}"
        self.redis.setex(key, COOLDOWN_SECONDS * 2, str(time.time()))

    def check_and_record(self, region: str) -> dict:
        if self.is_in_cooldown(region):
            remaining = self.get_remaining_cooldown(region)
            return {
                "allowed": False,
                "reason": f"Cooldown active: {remaining}s remaining",
                "remaining_seconds": remaining
            }

        self.record_failover(region)
        return {"allowed": True, "reason": "No cooldown active"}
```

Why 15 minutes:

- A failover that completes in 2 minutes followed by a failback 3 minutes later indicates a transient issue, not a real outage.
- Failover flapping (failover, failback, failover) is worse than staying on the secondary region for 15 minutes.
- 15 minutes gives enough time for the source region to stabilize if the issue was transient (e.g., AWS AZ outage that self-resolves).

### Common Mistakes to Avoid

1. **Requiring human approval for automated failover.** This adds 5-15 minutes of delay, which exceeds the 5-minute RTO. Humans should be notified, not consulted.
2. **Monitoring a region from within that region.** If the region is down, its monitoring is also down. Always monitor from at least 2 external regions.
3. **Setting the cooldown period too short (e.g., 1 minute).** This allows failover flapping. Set it long enough to absorb transient issues.
4. **Not testing the failover triggers.** Run a DR drill quarterly to verify the health checks, threshold logic, and failover execution work end-to-end.
5. **Using a single monitoring source.** One monitoring region losing connectivity to the target triggers a false failover. Always require quorum from 2 independent sources.

---

## Part C: Split-Brain Prevention

### Split-Brain Scenario

The most dangerous split-brain scenario in this architecture:

```
Time 0:00 -- us-east-1 (primary) and eu-west-1 lose network connectivity.
           ap-southeast-1 can reach both.
           us-east-1 is still serving 60% of global traffic (North America).
           eu-west-1 is still serving 25% of global traffic (Europe).

Time 0:30 -- eu-west-1 health checks for us-east-1 begin failing.
           us-east-1 health checks for eu-west-1 begin failing.
           ap-southeast-1 sees both regions as healthy (it can reach both).

Time 1:30 -- eu-west-1 has 3 consecutive failures for us-east-1.
           ap-southeast-1 has 0 failures for us-east-1 (it can still reach it).
           No failover triggers (quorum not met -- only 1 of 2 monitoring
           sources reports failure).

Time 2:00 -- If the partition persists, the system stays in this state.
           us-east-1 continues as primary (it can write to Aurora).
           eu-west-1 cannot reach the Aurora writer, so its reads fail.
           No split-brain occurs because Aurora has a single writer endpoint.

Key insight: Aurora Global Database prevents split-brain at the database level
because there is only ONE writer at any time. The writer endpoint is global --
it always points to the current primary cluster, regardless of which region
the client connects from.
```

However, if the partition causes an automated failover:

```
Time 3:00 -- ap-southeast-1's health check for us-east-1 fails
           (ap-southeast-1 also loses connectivity to us-east-1).
           Now both eu-west-1 and ap-southeast-1 agree: us-east-1 is down.

Time 3:30 -- Failover triggers. Aurora Global Database promotes eu-west-1
           to writer. eu-west-1 is now the primary.

Time 3:31 -- us-east-1 is still running, still serving North American traffic.
           us-east-1's Aurora cluster is now a reader (promoted eu-west-1
           is writer). us-east-1's writes fail because the writer endpoint
           now points to eu-west-1. us-east-1's reads succeed against the
           (now stale) local reader.

No split-brain: Aurora's single-writer architecture prevents dual writes.
```

### Quorum-Based Decision Mechanism

With 3 regions, a quorum of 2 prevents split-brain:

```
Region health vote:
  us-east-1 votes on eu-west-1:     [cannot vote -- it is the target]
  eu-west-1 votes on us-east-1:     FAIL (3/3 consecutive failures)
  ap-southeast-1 votes on us-east-1: FAIL (3/3 consecutive failures)

Quorum: 2/2 non-target regions agree --> failover authorized

If only 1 region reports failure:
  eu-west-1 votes on us-east-1:     FAIL
  ap-southeast-1 votes on us-east-1: PASS
  Quorum NOT met --> no failover
```

The quorum rule:

| Scenario | eu-west-1 vote | ap-southeast-1 vote | Decision |
|----------|---------------|---------------------|----------|
| us-east-1 truly down | FAIL | FAIL | Failover to best available |
| eu-west-1 cannot reach us-east-1 (partial partition) | FAIL | PASS | No failover -- investigate |
| Both monitoring regions down | N/A | N/A | No failover -- cannot form quorum |

### Fencing Mechanism

Fencing ensures the old primary stops accepting writes after failover. Three layers provide defense in depth.

**Layer 1: DNS Removal**

```bash
#!/bin/bash
# /usr/local/bin/fence-dns.sh
# Removes the old primary from DNS immediately after failover

set -euo pipefail

OLD_REGION="$1"
HOSTED_ZONE_ID="Z1234567890"

# Remove old primary from latency-based routing
aws route53 change-resource-record-sets \
    --hosted-zone-id "${HOSTED_ZONE_ID}" \
    --change-batch "{
        \"Changes\": [{
            \"Action\": \"DELETE\",
            \"ResourceRecordSet\": {
                \"Name\": \"api.helios.io\",
                \"Type\": \"A\",
                \"SetIdentifier\": \"${OLD_REGION}\",
                \"Region\": \"${OLD_REGION}\",
                \"TTL\": 60,
                \"ResourceRecords\": [{
                    \"Value\": \"$(get_region_ip ${OLD_REGION})\"
                }]
            }
        }]
    }"

echo "DNS record for ${OLD_REGION} removed"
```

**Layer 2: API Gateway Rejection**

```yaml
# API Gateway configuration to reject requests in fenced region
# Deploy as Lambda@Edge or API Gateway custom authorizer

Resources:
  FencedRegionAuthorizer:
    Type: AWS::Serverless::Function
    Properties:
      FunctionName: helios-fence-check
      Runtime: python3.11
      Handler: index.handler
      Environment:
        Variables:
          FENCED_REGIONS: "us-east-1"
      InlineCode: |
        import json
        import os

        def handler(event, context):
            fenced_regions = os.environ.get('FENCED_REGIONS', '').split(',')
            current_region = os.environ.get('AWS_REGION', '')

            if current_region in fenced_regions:
                return {
                    'statusCode': 503,
                    'headers': {
                        'Retry-After': '60',
                        'X-Fenced-Region': current_region
                    },
                    'body': json.dumps({
                        'error': 'Region temporarily unavailable',
                        'retry_after': 60,
                        'region': current_region
                    })
                }

            return {'statusCode': 200}
```

**Layer 3: Database Read-Only Mode**

```sql
-- On the old primary's Aurora cluster, force read-only mode
-- This is a safety net -- Aurora should already be a reader after promotion

-- Set the cluster to read-only at the database level
ALTER SYSTEM SET default_transaction_read_only = on;
SELECT pg_reload_conf();

-- Verify
SHOW default_transaction_read_only;
-- Expected: on

-- Any write attempt will now fail:
-- ERROR: cannot execute INSERT in a read-only transaction
```

Why three layers:

- DNS removal stops new connections from reaching the old primary. But existing connections (keep-alive, connection pools) may persist for up to 60 seconds.
- API gateway rejection catches requests that bypass DNS (direct IP, cached DNS, internal service mesh). Returns 503 with Retry-After header.
- Database read-only mode is the last resort. Even if a request somehow reaches the old primary, the database will reject the write. This is the most reliable fencing mechanism.

### Data Reconciliation After Partition

Aurora Global Database handles most reconciliation automatically because it is single-writer. However, if split-brain did occur (e.g., due to a bug in the failover logic), reconciliation is required.

```python
#!/usr/bin/env python3
# /usr/local/bin/data-reconciliation.py
# Compares data between old primary and new primary after partition resolution

import psycopg2
import json
import hashlib
from datetime import datetime, timezone

OLD_PRIMARY_DSN = "host=old-primary.helios.io dbname=helios user=readonly"
NEW_PRIMARY_DSN = "host=new-primary.helios.io dbname=helios user=readonly"


def get_row_hash(conn, table, primary_key_col, pk_value):
    """Compute a hash of an entire row for comparison."""
    with conn.cursor() as cur:
        cur.execute(
            f"SELECT * FROM {table} WHERE {primary_key_col} = %s",
            (pk_value,)
        )
        row = cur.fetchone()
        if row is None:
            return None
        colnames = [desc[0] for desc in cur.description]
        row_dict = dict(zip(colnames, row))
        # Convert non-serializable types
        for k, v in row_dict.items():
            if isinstance(v, datetime):
                row_dict[k] = v.isoformat()
            elif isinstance(v, (bytes, memoryview)):
                row_dict[k] = str(v)
        return hashlib.sha256(
            json.dumps(row_dict, sort_keys=True, default=str).encode()
        ).hexdigest()


def reconcile_table(conn_old, conn_new, table, pk_col,
                     conflict_strategy="last_write_wins"):
    """Compare and reconcile a single table between two databases."""
    discrepancies = []
    resolved = 0

    with conn_old.cursor() as cur_old, conn_new.cursor() as cur_new:
        # Get all PKs from both databases
        cur_old.execute(f"SELECT {pk_col} FROM {table}")
        old_pks = {row[0] for row in cur_old.fetchall()}

        cur_new.execute(f"SELECT {pk_col} FROM {table}")
        new_pks = {row[0] for row in cur_new.fetchall()}

        # Records only in old primary (lost during failover)
        only_in_old = old_pks - new_pks
        for pk in only_in_old:
            cur_old.execute(
                f"SELECT * FROM {table} WHERE {pk_col} = %s", (pk,)
            )
            row = cur_old.fetchone()
            discrepancies.append({
                "table": table,
                "pk": pk,
                "issue": "missing_from_new",
                "action": "insert_to_new",
                "row": str(row)
            })

        # Records only in new primary (written after failover)
        only_in_new = new_pks - old_pks
        for pk in only_in_new:
            discrepancies.append({
                "table": table,
                "pk": pk,
                "issue": "missing_from_old",
                "action": "keep_in_new",
            })

        # Records in both -- compare hashes
        common_pks = old_pks & new_pks
        for pk in common_pks:
            old_hash = get_row_hash(conn_old, table, pk_col, pk)
            new_hash = get_row_hash(conn_new, table, pk_col, pk)

            if old_hash != new_hash:
                if conflict_strategy == "last_write_wins":
                    # Compare updated_at timestamps
                    with conn_old.cursor() as c:
                        c.execute(
                            f"SELECT updated_at FROM {table} "
                            f"WHERE {pk_col} = %s", (pk,)
                        )
                        old_ts = c.fetchone()[0]
                    with conn_new.cursor() as c:
                        c.execute(
                            f"SELECT updated_at FROM {table} "
                            f"WHERE {pk_col} = %s", (pk,)
                        )
                        new_ts = c.fetchone()[0]

                    winner = "new" if new_ts >= old_ts else "old"
                    discrepancies.append({
                        "table": table,
                        "pk": pk,
                        "issue": "data_divergence",
                        "strategy": "last_write_wins",
                        "winner": winner,
                        "old_timestamp": str(old_ts),
                        "new_timestamp": str(new_ts),
                    })
                    resolved += 1

    return {"discrepancies": discrepancies, "auto_resolved": resolved}


def main():
    conn_old = psycopg2.connect(OLD_PRIMARY_DSN)
    conn_new = psycopg2.connect(NEW_PRIMARY_DSN)

    tables = [
        ("users", "user_id"),
        ("documents", "doc_id"),
        ("messages", "msg_id"),
        ("projects", "project_id"),
    ]

    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "tables": {}
    }

    for table, pk_col in tables:
        result = reconcile_table(conn_old, conn_new, table, pk_col)
        report["tables"][table] = result
        print(
            f"{table}: {len(result['discrepancies'])} discrepancies, "
            f"{result['auto_resolved']} auto-resolved"
        )

    # Write report
    with open("/tmp/reconciliation-report.json", "w") as f:
        json.dump(report, f, indent=2)

    print("\nReconciliation report: /tmp/reconciliation-report.json")

    conn_old.close()
    conn_new.close()


if __name__ == "__main__":
    main()
```

### Split-Brain Detection Script

```python
#!/usr/bin/env python3
# /usr/local/bin/split-brain-detector.py
# Monitors for split-brain conditions across all regions

import boto3
import time
import json
import redis
import requests
from datetime import datetime, timezone

REGIONS = ["us-east-1", "eu-west-1", "ap-southeast-1"]
GLOBAL_CLUSTER_ID = "helios-global-cluster"
ALERT_WEBHOOK = "https://hooks.slack.com/services/T00/B00/xxx"


def get_aurora_writer_region():
    """Query Aurora Global Database to find the current writer region."""
    rds = boto3.client("rds", region_name="us-east-1")
    response = rds.describe_global_clusters(
        GlobalClusterIdentifier=GLOBAL_CLUSTER_ID
    )

    for cluster in response["GlobalClusters"][0]["GlobalClusterMembers"]:
        if cluster["IsWriter"]:
            # Extract region from the DB cluster ARN
            arn = cluster["DBClusterArn"]
            # arn:aws:rds:us-east-1:123456789:cluster:helios-us-east-1
            region = arn.split(":")[3]
            return region

    return None


def get_api_primary_region():
    """Query the API gateway to find which region is accepting writes."""
    for region in REGIONS:
        try:
            response = requests.get(
                f"https://api.{region}.helios.io/health/primary",
                timeout=10
            )
            if response.status_code == 200:
                data = response.json()
                return data.get("primary_region")
        except requests.RequestException:
            continue

    return None


def check_redis_primary_region():
    """Query Redis Global Datastore to find the primary region."""
    for region in REGIONS:
        try:
            r = redis.Redis(
                host=f"redis.{region}.helios.io",
                port=6379,
                ssl=True,
                socket_timeout=5
            )
            info = r.info("replication")
            if info.get("role") == "master":
                return region
        except (redis.ConnectionError, redis.TimeoutError):
            continue

    return None


def detect_split_brain():
    """Compare primary declarations across all systems."""
    aurora_writer = get_aurora_writer_region()
    api_primary = get_api_primary_region()
    redis_primary = check_redis_primary_region()

    timestamp = datetime.now(timezone.utc).isoformat()
    result = {
        "timestamp": timestamp,
        "aurora_writer": aurora_writer,
        "api_primary": api_primary,
        "redis_primary": redis_primary,
    }

    # Check for disagreement
    primaries = set()
    for source, region in [
        ("aurora", aurora_writer),
        ("api", api_primary),
        ("redis", redis_primary),
    ]:
        if region is not None:
            primaries.add(region)

    if len(primaries) > 1:
        result["split_brain_detected"] = True
        result["conflicting_primaries"] = list(primaries)
        alert(result)
    elif len(primaries) == 0:
        result["split_brain_detected"] = False
        result["warning"] = "No primary detected in any system"
        alert_warning(result)
    else:
        result["split_brain_detected"] = False
        result["primary_region"] = list(primaries)[0]

    return result


def alert(result):
    """Send critical alert for split-brain detection."""
    message = (
        f"SPLIT-BRAIN DETECTED\n"
        f"Aurora writer: {result['aurora_writer']}\n"
        f"API primary: {result['api_primary']}\n"
        f"Redis primary: {result['redis_primary']}\n"
        f"Conflicting regions: {result['conflicting_primaries']}"
    )

    requests.post(ALERT_WEBHOOK, json={
        "text": f"CRITICAL: {message}"
    })

    # Also write to local audit log
    with open("/var/log/split-brain-events.log", "a") as f:
        f.write(json.dumps(result) + "\n")


def alert_warning(result):
    """Send warning alert."""
    requests.post(ALERT_WEBHOOK, json={
        "text": f"WARNING: {result['warning']}"
    })


def main():
    while True:
        try:
            result = detect_split_brain()
            print(json.dumps(result))
        except Exception as e:
            print(f"Error: {e}")
        time.sleep(30)


if __name__ == "__main__":
    main()
```

### Common Mistakes to Avoid

1. **Relying on application-level fencing alone.** DNS removal and API gateway rejection are necessary but not sufficient. The database must also be in read-only mode to prevent writes from direct connections.
2. **Using a 2-region quorum.** With 2 regions, a network partition means neither region can form a quorum, and both stop serving traffic. Always use an odd number of voting members (3 regions, or 3 external health check endpoints).
3. **Not testing data reconciliation.** The reconciliation script must be tested with known divergent data to verify it resolves conflicts correctly.
4. **Assuming Aurora prevents all split-brain scenarios.** Aurora prevents database-level split-brain (dual writes), but application-level split-brain (two regions both processing business logic, updating caches, sending notifications) can still occur. Application-layer fencing (API gateway rejection) prevents this.
5. **Forgetting to fence Redis.** If two regions both think they are primary, their Redis caches will diverge. After failover, the new primary must flush and rebuild its Redis cache from the database.

---

## Part D: DNS-Based Traffic Routing

### Route 53 Routing Policy

The routing strategy uses latency-based routing as the primary policy, with failover routing as the secondary mechanism.

```
Primary policy: Latency-based routing
  - Route users to the region with lowest latency
  - us-east-1 serves North America (60% of traffic)
  - eu-west-1 serves Europe (25% of traffic)
  - ap-southeast-1 serves Asia-Pacific (15% of traffic)

Failover policy: Health check-based failover
  - If a region's health check fails, Route 53 stops routing to it
  - Traffic redistributes to healthy regions automatically
  - This is not a separate policy -- it is built into latency-based routing
    when health checks are associated with each record
```

### Route 53 Health Check Configuration

```bash
# Create health checks for each region

# us-east-1 health check
aws route53 create-health-check \
    --caller-reference "helios-us-east-1-$(date +%s)" \
    --health-check-config '{
        "FullyQualifiedDomainName": "api.us-east-1.helios.io",
        "Port": 443,
        "Type": "HTTPS",
        "ResourcePath": "/health/region",
        "RequestInterval": 10,
        "FailureThreshold": 3,
        "EnableSNI": true,
        "MeasureLatency": true,
        "Regions": ["eu-west-1", "ap-southeast-1", "us-west-2"]
    }'

# eu-west-1 health check
aws route53 create-health-check \
    --caller-reference "helios-eu-west-1-$(date +%s)" \
    --health-check-config '{
        "FullyQualifiedDomainName": "api.eu-west-1.helios.io",
        "Port": 443,
        "Type": "HTTPS",
        "ResourcePath": "/health/region",
        "RequestInterval": 10,
        "FailureThreshold": 3,
        "EnableSNI": true,
        "MeasureLatency": true,
        "Regions": ["us-east-1", "ap-southeast-1", "eu-central-1"]
    }'

# ap-southeast-1 health check
aws route53 create-health-check \
    --caller-reference "helios-ap-se-1-$(date +%s)" \
    --health-check-config '{
        "FullyQualifiedDomainName": "api.ap-southeast-1.helios.io",
        "Port": 443,
        "Type": "HTTPS",
        "ResourcePath": "/health/region",
        "RequestInterval": 10,
        "FailureThreshold": 3,
        "EnableSNI": true,
        "MeasureLatency": true,
        "Regions": ["us-east-1", "eu-west-1", "ap-northeast-1"]
    }'
```

Health check endpoint implementation:

```python
# /health/region endpoint for each region
# Returns 200 only if the region can serve traffic

from fastapi import FastAPI
from fastapi.responses import JSONResponse
import boto3
import redis
import psycopg2

app = FastAPI()


@app.get("/health/region")
async def region_health():
    checks = {}

    # Check Aurora writer availability
    try:
        conn = psycopg2.connect(
            host="helios-cluster.cluster-xxx.us-east-1.rds.amazonaws.com",
            dbname="helios",
            user="health_check",
            connect_timeout=5
        )
        with conn.cursor() as cur:
            cur.execute("SELECT 1")
        conn.close()
        checks["database"] = "healthy"
    except Exception as e:
        checks["database"] = f"unhealthy: {e}"

    # Check Redis
    try:
        r = redis.Redis(
            host="helios-redis.xxx.cache.amazonaws.com",
            port=6379,
            ssl=True,
            socket_timeout=5
        )
        r.ping()
        checks["redis"] = "healthy"
    except Exception as e:
        checks["redis"] = f"unhealthy: {e}"

    # Check EKS pod count
    try:
        # Simplified -- in reality check Kubernetes API
        checks["pods"] = "healthy"
    except Exception as e:
        checks["pods"] = f"unhealthy: {e}"

    all_healthy = all(v == "healthy" for v in checks.values())

    status_code = 200 if all_healthy else 503
    return JSONResponse(
        content={
            "status": "healthy" if all_healthy else "unhealthy",
            "checks": checks
        },
        status_code=status_code
    )
```

### TTL Strategy

| DNS Record | TTL | Rationale |
|-----------|-----|-----------|
| api.helios.io (latency-based) | 60 seconds | Balances failover speed with DNS query cost |
| health.helios.io (health checks) | N/A | Route 53 health checks are internal, not resolved by clients |
| static.helios.io (CloudFront) | 300 seconds | Static content behind CDN, CDN handles failover internally |
| db.helios.io (Aurora endpoint) | N/A | Aurora endpoints are managed by AWS, not Route 53 |

Why 60 seconds:

- Lower TTL (e.g., 10 seconds) means faster failover propagation but higher DNS query costs and more load on Route 53.
- Higher TTL (e.g., 300 seconds) means clients cache the old IP for up to 5 minutes after failover, which violates the 5-minute RTO.
- 60 seconds is the standard for production failover. Combined with health check interval of 10 seconds and failure threshold of 3, the total failover detection + propagation time is approximately:

```
Detection: 3 failures x 10 seconds = 30 seconds
DNS propagation: up to 60 seconds (TTL)
Total: 90 seconds worst case for DNS-based cutover
```

This is well within the 5-minute RTO. The Aurora promotion adds another 30-60 seconds, bringing the total to approximately 2-3 minutes.

### Traffic Cutover Procedure

For a planned cutover (e.g., maintenance) or an unplanned failover, the procedure is the same.

```bash
#!/bin/bash
# /usr/local/bin/traffic-cutover.sh
# Moves 100% of traffic from source region to target region

set -euo pipefail

SOURCE_REGION="$1"
TARGET_REGION="$2"
HOSTED_ZONE_ID="Z1234567890"
DRY_RUN="${3:-}"

log() {
    echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) $*"
}

# Step 1: Verify target region is healthy
log "Verifying target region ${TARGET_REGION} health..."
HEALTH_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
    "https://api.${TARGET_REGION}.helios.io/health/region" \
    --connect-timeout 10)

if [ "${HEALTH_CODE}" != "200" ]; then
    log "ERROR: Target region ${TARGET_REGION} is not healthy (HTTP ${HEALTH_CODE})"
    exit 1
fi
log "Target region healthy"

# Step 2: Update DNS to remove source region
log "Removing ${SOURCE_REGION} from DNS routing..."

CHANGE_BATCH=$(cat <<EOF
{
    "Changes": [
        {
            "Action": "UPSERT",
            "ResourceRecordSet": {
                "Name": "api.helios.io",
                "Type": "A",
                "SetIdentifier": "${TARGET_REGION}",
                "Region": "${TARGET_REGION}",
                "TTL": 60,
                "ResourceRecords": [{"Value": "$(get_region_ip ${TARGET_REGION})"}],
                "HealthCheckId": "$(get_health_check_id ${TARGET_REGION})"
            }
        },
        {
            "Action": "DELETE",
            "ResourceRecordSet": {
                "Name": "api.helios.io",
                "Type": "A",
                "SetIdentifier": "${SOURCE_REGION}",
                "Region": "${SOURCE_REGION}",
                "TTL": 60,
                "ResourceRecords": [{"Value": "$(get_region_ip ${SOURCE_REGION})"}],
                "HealthCheckId": "$(get_health_check_id ${SOURCE_REGION})"
            }
        }
    ]
}
EOF
)

if [ "${DRY_RUN}" = "--dry-run" ]; then
    log "DRY RUN: Would apply the following DNS change:"
    echo "${CHANGE_BATCH}" | python3 -m json.tool
    exit 0
fi

aws route53 change-resource-record-sets \
    --hosted-zone-id "${HOSTED_ZONE_ID}" \
    --change-batch "${CHANGE_BATCH}"

log "DNS updated. Traffic will shift within 60 seconds (TTL)."

# Step 3: Verify DNS propagation
log "Waiting 70 seconds for DNS propagation..."
sleep 70

# Step 4: Verify no traffic on source region
log "Checking source region traffic..."
SOURCE_REQUESTS=$(curl -s \
    "https://prometheus.${SOURCE_REGION}.helios.io/api/v1/query?query=sum(rate(http_requests_total[1m]))" | \
    jq -r '.data.result[0].value[1] // "0"')

log "Source region requests/sec: ${SOURCE_REQUESTS}"

# Step 5: Scale up target region
log "Scaling up ${TARGET_REGION} EKS cluster..."
for deploy in $(kubectl --context="${TARGET_REGION}" get deploy -o name); do
    kubectl --context="${TARGET_REGION}" scale "${deploy}" --replicas=3
done

log "Cutover complete. ${TARGET_REGION} is now serving all traffic."
```

### In-Flight Request Handling

During failover, requests that are in-flight on the old primary must be handled gracefully.

```python
# FastAPI middleware for graceful shutdown
# Returns 503 with Retry-After for new requests during drain

import signal
import asyncio
from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse
from contextlib import asynccontextmanager

SHUTDOWN_TIMEOUT = 30  # seconds to drain in-flight requests
is_draining = False


@asynccontextmanager
async def lifespan(app: FastAPI):
    # Register signal handlers for graceful shutdown
    loop = asyncio.get_event_loop()
    for sig in (signal.SIGTERM, signal.SIGINT):
        loop.add_signal_handler(
            sig, lambda: asyncio.create_task(initiate_drain())
        )
    yield


app = FastAPI(lifespan=lifespan)


async def initiate_drain():
    global is_draining
    is_draining = True
    # Wait for in-flight requests to complete
    await asyncio.sleep(SHUTDOWN_TIMEOUT)
    # Force exit if still running
    import os
    os._exit(0)


@app.middleware("http")
async def drain_middleware(request: Request, call_next):
    if is_draining:
        return JSONResponse(
            status_code=503,
            content={
                "error": "Service temporarily unavailable",
                "reason": "Region failover in progress"
            },
            headers={
                "Retry-After": "60",
                "Connection": "close"
            }
        )
    response = await call_next(request)
    return response
```

The drain sequence during failover:

```
1. Route 53 health check fails (t=0s)
   - New DNS queries return the healthy region's IP
   - Existing clients still have the old IP cached (up to 60s)

2. API gateway starts returning 503 (t=5s)
   - New requests to the old region get 503 + Retry-After: 60
   - Clients with retry logic automatically retry against the healthy region

3. PgBouncer drains connections (t=10s)
   - PgBouncer stops accepting new connections
   - In-flight queries complete (up to 30s timeout)

4. DNS cache expires (t=60s)
   - All clients now resolve to the healthy region
   - Old region receives zero traffic

Total drain time: 60 seconds (limited by DNS TTL)
```

### Common Mistakes to Avoid

1. **Using a TTL lower than 60 seconds.** DNS resolvers and intermediate caches (corporate proxies, ISP resolvers) may not honor very low TTLs. A TTL of 10 seconds may still result in 60 seconds of caching at the resolver level.
2. **Not associating health checks with latency-based routing records.** Without health checks, Route 53 continues routing to a dead region. The health check must return the region-level health endpoint, not just a static page.
3. **Forgetting about in-flight requests.** New requests get 503, but in-flight requests (long-running queries, WebSocket connections) need explicit drain handling. The 30-second drain timeout must be less than the DNS TTL to avoid a gap where clients connect but the region cannot serve.
4. **Not testing DNS failover end-to-end.** Run a quarterly drill that simulates a region failure and verifies that DNS propagation, Aurora promotion, EKS scaling, and application reconnection all work within the 5-minute RTO.
5. **Assuming Route 53 failover is instantaneous.** Even with health checks, there is a 30-second detection window (3 failures x 10s interval) plus up to 60 seconds of DNS caching. Plan for 90 seconds minimum of partial unavailability.

---

## Key Takeaway

Multi-region DR is not just "single-region failover but bigger." It introduces two fundamental challenges that do not exist in single-region architectures: network partitions (which cause split-brain) and data residency constraints (which limit replication topology). Aurora Global Database solves the database split-brain problem at the storage layer by enforcing a single writer. Route 53 latency-based routing with health checks solves the traffic routing problem. But the hardest part is not the technology -- it is the operational discipline: quarterly DR drills, tested reconciliation procedures, and the organizational willingness to let automated systems make failover decisions without human approval. The architecture described here achieves a 2-3 minute RTO with 30-second RPO, which meets the business requirements while respecting GDPR data residency constraints.
