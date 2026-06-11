# Solution 04: Disaster Recovery and Chaos Testing

## Part A: RTO/RPO and Backup Strategy

| Tier | Component | RTO | RPO | Backup Strategy |
|------|-----------|-----|-----|-----------------|
| 1 | PostgreSQL database | 60 seconds | 0 seconds (sync replication) | Streaming replication + WAL archiving to cross-region S3 |
| 2 | Application state (pods) | 30 seconds | 0 (stateless) | K8s Deployment with replicas; no backup needed, just redeploy |
| 3 | Configuration (K8s manifests) | 5 minutes | 0 (Git) | GitOps (ArgoCD) -- all manifests in Git, auto-synced to clusters |
| 4 | TLS certificates | 5 minutes | 0 | cert-manager auto-provisions; backed up in Secrets in Git (encrypted) |
| 5 | Observability data | 24 hours | 1 hour | Prometheus: Thanos for long-term storage. Logs: S3 lifecycle policy |

**Database backup CronJob:**

```yaml
# db-backup-cronjob.yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: postgres-backup
  namespace: database
spec:
  schedule: "0 */6 * * *"
  concurrencyPolicy: Forbid
  successfulJobsHistoryLimit: 3
  failedJobsHistoryLimit: 3
  jobTemplate:
    spec:
      backoffLimit: 2
      activeDeadlineSeconds: 3600
      template:
        spec:
          serviceAccountName: backup-sa
          restartPolicy: OnFailure
          containers:
            - name: backup
              image: postgres:16-alpine
              env:
                - name: PGHOST
                  value: "postgres-primary.database.svc.cluster.local"
                - name: PGUSER
                  valueFrom:
                    secretKeyRef:
                      name: backup-credentials
                      key: username
                - name: PGPASSWORD
                  valueFrom:
                    secretKeyRef:
                      name: backup-credentials
                      key: password
                - name: S3_BUCKET
                  value: "s3://payments-backup-eu-west-1"
                - name: BACKUP_DATE
                  value: "$(date +%Y%m%d-%H%M%S)"
              command:
                - /bin/sh
                - -c
                - |
                  set -euo pipefail

                  echo "Starting full backup: ${BACKUP_DATE}"

                  # Take full backup
                  pg_basebackup \
                    -h ${PGHOST} \
                    -U ${PGUSER} \
                    -D /tmp/backup \
                    --format=tar \
                    --gzip \
                    --wal-method=stream \
                    --checkpoint=fast \
                    --label="${BACKUP_DATE}"

                  # Verify backup integrity
                  echo "Verifying backup integrity..."
                  pg_verifybackup /tmp/backup || {
                    echo "BACKUP VERIFICATION FAILED"
                    exit 1
                  }

                  # Upload to S3 (cross-region)
                  aws s3 sync /tmp/backup/ \
                    "${S3_BUCKET}/full/${BACKUP_DATE}/" \
                    --storage-class GLACIER_IR

                  # Record backup metadata
                  echo "${BACKUP_DATE}" | aws s3 cp - \
                    "${S3_BUCKET}/latest-backup.txt"

                  echo "Backup complete: ${BACKUP_DATE}"

                  # Cleanup
                  rm -rf /tmp/backup
              volumeMounts:
                - name: tmp
                  mountPath: /tmp
          volumes:
            - name: tmp
              emptyDir:
                sizeLimit: 50Gi
```

**WAL archiving configuration (in PostgreSQL):**

```yaml
# postgres-wal-archiver.yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: wal-archiver
  namespace: database
spec:
  schedule: "*/5 * * * *"
  concurrencyPolicy: Forbid
  jobTemplate:
    spec:
      template:
        spec:
          containers:
            - name: wal-archive
              image: postgres:16-alpine
              command:
                - /bin/sh
                - -c
                - |
                  set -euo pipefail

                  # Archive any pending WAL files to S3
                  pg_receivewal \
                    -h postgres-primary.database.svc.cluster.local \
                    -U replicator \
                    -D /tmp/wal \
                    --no-loop \
                    --synchronous

                  aws s3 sync /tmp/wal/ \
                    s3://payments-backup-eu-west-1/wal/ \
                    --storage-class GLACIER_IR
```

**Why this works:**

- Full backups every 6 hours provide a baseline for restore. WAL
  archiving every 5 minutes provides point-in-time recovery between
  full backups. Combined, RPO is effectively zero for synchronous
  replication, and at most 5 minutes for the async WAL archive.
- Cross-region S3 storage ensures the backup survives a regional outage.
  `GLACIER_IR` (Instant Retrieval) storage class is cheaper than
  standard but still allows fast retrieval.
- `pg_verifybackup` catches corruption before the backup is uploaded.
  A corrupt backup is worse than no backup -- it gives false confidence.

## Part B: Failover Runbook

### Failover: us-east-1 -> eu-west-1

**Prerequisites:** Ensure you have `kubectl` access to both clusters
and AWS CLI access with Route53 permissions.

**Step 1: Confirm us-east-1 is truly down**

```bash
# Check from multiple vantage points
curl -s -o /dev/null -w "%{http_code}" --max-time 10 \
  https://api.payments.example.com/healthz

# Check AWS region health
aws ec2 describe-instance-status --region us-east-1 \
  --filters Name=instance-state-name,Values=running

# Check K8s cluster status
kubectl --context=us-east-1 get nodes

# Expected: All checks fail or timeout
# Timeout: 30 seconds. If any check succeeds, STOP -- us-east-1 is not down.
```

**Step 2: Notify the team**

```bash
# Post to #incident channel
curl -X POST -H 'Content-type: application/json' \
  --data '{"text":"FAILOVER INITIATED: us-east-1 -> eu-west-1. Reason: [fill in]"}' \
  "$SLACK_WEBHOOK_URL"
```

**Step 3: Promote eu-west-1 database to primary**

```bash
# Verify the replica is caught up
kubectl --context=eu-west-1 exec -n database postgres-secondary-0 -- \
  psql -c "SELECT pg_is_in_recovery();"
# Expected: true (it is still a replica)

# Check replication lag
kubectl --context=eu-west-1 exec -n database postgres-secondary-0 -- \
  psql -c "SELECT CASE WHEN pg_is_in_recovery() 
    THEN EXTRACT(EPOCH FROM (now() - pg_last_xact_replay_timestamp()))::int 
    ELSE 0 END AS lag_seconds;"
# Expected: lag_seconds < 5
# Timeout: If lag > 30 seconds, wait and recheck

# Promote to primary
kubectl --context=eu-west-1 exec -n database postgres-secondary-0 -- \
  psql -c "SELECT pg_promote();"

# Verify promotion
kubectl --context=eu-west-1 exec -n database postgres-secondary-0 -- \
  psql -c "SELECT pg_is_in_recovery();"
# Expected: false (now primary)
# Timeout: 10 seconds
```

**Step 4: Update DNS to point to eu-west-1**

```bash
# Get eu-west-1 ingress IP
EU_WEST_IP=$(kubectl --context=eu-west-1 get svc -n ingress-nginx \
  ingress-nginx-controller -o jsonpath='{.status.loadBalancer.ingress[0].hostname}')

# Update Route53 health check to primary eu-west-1
aws route53 change-resource-record-sets --hosted-zone-id Z1234567890 \
  --change-batch '{
    "Changes": [{
      "Action": "UPSERT",
      "ResourceRecordSet": {
        "Name": "api.payments.example.com",
        "Type": "A",
        "AliasTarget": {
          "HostedZoneId": "Z0987654321",
          "DNSName": "'"$EU_WEST_IP"'",
          "EvaluateTargetHealth": true
        }
      }
    }]
  }'

# Verify DNS propagation (from external vantage point)
dig api.payments.example.com +short
# Expected: eu-west-1 IP
# Timeout: DNS TTL is 60 seconds, so wait up to 120 seconds
```

**Step 5: Verify the failover**

```bash
# Health check
curl -s -o /dev/null -w "%{http_code}" --max-time 10 \
  https://api.payments.example.com/healthz
# Expected: 200

# Test a payment
curl -X POST https://api.payments.example.com/api/v1/payments \
  -H "Authorization: Bearer $TEST_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"amount": 1.00, "currency": "USD", "recipient": "test"}'
# Expected: 200 with payment ID

# Check observability
curl -s "http://prometheus.eu-west-1:9090/api/v1/query?query=up{app='payment-api'}" | jq '.data.result | length'
# Expected: 3 (three pods running)
```

**Step 6: Rollback when us-east-1 recovers**

```bash
# 1. Rebuild us-east-1 database as replica of eu-west-1
kubectl --context=us-east-1 exec -n database postgres-primary-0 -- \
  psql -c "SELECT pg_promote();" # only if it was still primary

# Take basebackup from eu-west-1
kubectl --context=us-east-1 exec -n database postgres-primary-0 -- \
  bash -c "rm -rf /var/lib/postgresql/data/* && 
    pg_basebackup -h eu-west-1-lb -U replicator -D /var/lib/postgresql/data -R"

# 2. Restart us-east-1 pods
kubectl --context=us-east-1 rollout restart deployment/payment-api -n payment-system

# 3. Wait for replication to catch up (lag < 1 second)
kubectl --context=eu-west-1 exec -n database postgres-secondary-0 -- \
  psql -c "SELECT pg_last_wal_replay_lsn();"

# 4. Switch DNS back to us-east-1 (same command as Step 4, different IP)

# 5. Promote us-east-1 back to primary (same as Step 3)

# 6. Rebuild eu-west-1 as replica
```

## Part C: Chaos Experiments

### Experiment 1: Pod Failure

```yaml
# chaos-pod-kill.yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: PodChaos
metadata:
  name: payment-api-pod-kill
  namespace: payment-system
spec:
  action: pod-kill
  mode: one
  selector:
    namespaces:
      - payment-system
    labelSelectors:
      app: payment-api
  scheduler:
    cron: '@every 30m'
```

**Hypothesis:** When a random payment-api pod is killed, the remaining
two pods absorb traffic with no increase in error rate. The killed pod
is replaced within 30 seconds.

**Blast radius:** One pod in the payment-system namespace.

**Method:** Chaos Mesh PodChaos kills one pod every 30 minutes.

**Rollback:**
```bash
kubectl delete podchaos payment-api-pod-kill -n payment-system
```

**Success criteria:**
- Error rate remains at 0% during pod replacement
- New pod becomes ready within 30 seconds
- p99 latency increases by less than 50ms during replacement

### Experiment 2: Network Latency Injection

```yaml
# chaos-network-latency.yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: NetworkChaos
metadata:
  name: payment-api-latency
  namespace: payment-system
spec:
  action: delay
  mode: one
  selector:
    namespaces:
      - payment-system
    labelSelectors:
      app: payment-api
  delay:
    latency: "500ms"
    jitter: "100ms"
    correlation: "50"
  direction: to
  target:
    selector:
      namespaces:
          - database
      labelSelectors:
        app: postgres
    mode: all
  duration: "5m"
```

**Hypothesis:** When 500ms latency is injected between payment-api and
PostgreSQL, p99 latency increases to ~700ms (500ms network + 200ms
normal) but error rate stays below 1% (timeout threshold is 5s).

**Blast radius:** Traffic from one payment-api pod to the database.

**Method:** Chaos Mesh NetworkChaos adds delay to outgoing traffic.

**Rollback:**
```bash
kubectl delete networkchaos payment-api-latency -n payment-system
```

**Success criteria:**
- p99 latency increases but stays below 1 second
- Error rate stays below 1%
- No pods enter CrashLoopBackOff

### Experiment 3: Database Failover

```yaml
# chaos-db-failover.yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: PodChaos
metadata:
  name: postgres-primary-kill
  namespace: database
spec:
  action: pod-kill
  mode: one
  selector:
    namespaces:
      - database
    labelSelectors:
      statefulset.kubernetes.io/pod-name: postgres-primary-0
  duration: "0s"
```

**Hypothesis:** When the PostgreSQL primary is killed, Patroni promotes
the replica within 30 seconds. The payment-api reconnects automatically
via the connection pool's retry logic. Total disruption: < 60 seconds.

**Blast radius:** All writes to the database are blocked during promotion.

**Method:** Chaos Mesh kills the primary pod. Patroni detects the failure
and promotes the replica.

**Rollback:**
```bash
kubectl delete podchaos postgres-primary-kill -n database
# Patroni will handle recovery automatically
```

**Success criteria:**
- Replica promotes within 30 seconds
- Payment-api reconnects within 60 seconds
- No data loss (all acknowledged writes are preserved)

### Experiment 4: Node Failure

```bash
# chaos-node-failure.sh
#!/bin/bash

NODE=$(kubectl get nodes -l topology.kubernetes.io/zone=us-east-1a \
  -o jsonpath='{.items[0].metadata.name}')

echo "Draining node: $NODE"

kubectl drain "$NODE" \
  --ignore-daemonsets \
  --delete-emptydir-data \
  --force \
  --grace-period=30

echo "Node drained. Waiting 2 minutes to observe recovery..."
sleep 120

echo "Verifying all pods are running on other nodes..."
NOT_RUNNING=$(kubectl get pods -n payment-system \
  --field-selector=status.phase!=Running -o name | wc -l)

if [ "$NOT_RUNNING" -gt 0 ]; then
  echo "FAILURE: $NOT_RUNNING pods are not running"
  exit 1
fi

echo "Recovery successful. Uncordoning node..."
kubectl uncordon "$NODE"
```

**Hypothesis:** When a node is drained, all pods are rescheduled to
other nodes within 2 minutes. The PDB ensures at most 1 pod is
disrupted at a time.

**Blast radius:** All pods on one node.

**Method:** `kubectl drain` gracefully evicts pods.

**Rollback:**
```bash
kubectl uncordon $NODE
```

**Success criteria:**
- All pods rescheduled within 2 minutes
- PDB respected (never fewer than 2 pods running)
- Zero errors during the drain

### Experiment 5: Region Failure

```bash
# chaos-region-failure.sh
#!/bin/bash

echo "Simulating us-east-1 failure by updating Route53 health check..."

# Mark us-east-1 as unhealthy
aws route53 change-resource-record-sets --hosted-zone-id Z1234567890 \
  --change-batch '{
    "Changes": [{
      "Action": "UPSERT",
      "ResourceRecordSet": {
        "Name": "api.payments.example.com",
        "Type": "A",
        "AliasTarget": {
          "HostedZoneId": "Z0987654321",
          "DNSName": "eu-west-1-ingress.example.com",
          "EvaluateTargetHealth": true
        }
      }
    }]
  }'

echo "DNS updated. Waiting for propagation (120 seconds)..."
sleep 120

# Verify failover
STATUS=$(curl -s -o /dev/null -w "%{http_code}" --max-time 10 \
  https://api.payments.example.com/healthz)

if [ "$STATUS" = "200" ]; then
  echo "SUCCESS: Region failover completed"
else
  echo "FAILURE: Health check returned $STATUS"
fi

echo "Restoring us-east-1 as primary..."
# Run the rollback procedure from the failover runbook
```

**Hypothesis:** When us-east-1 is marked unhealthy, Route53 fails over
to eu-west-1 within 120 seconds. The payment-api serves requests from
eu-west-1 with no data loss.

**Blast radius:** All traffic to us-east-1.

**Method:** DNS health check manipulation.

**Rollback:** Restore DNS to point to us-east-1 (same as runbook
rollback).

**Success criteria:**
- Failover completes within 120 seconds
- Payment-api in eu-west-1 serves requests successfully
- No data loss (eu-west-1 database has all transactions)

## Part D: Automated Recovery Rehearsal

```yaml
# recovery-rehearsal-cronjob.yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: recovery-rehearsal
  namespace: database
spec:
  schedule: "0 4 * * *"
  concurrencyPolicy: Forbid
  jobTemplate:
    spec:
      backoffLimit: 0
      template:
        spec:
          serviceAccountName: backup-sa
          restartPolicy: Never
          containers:
            - name: rehearsal
              image: postgres:16-alpine
              env:
                - name: SLACK_WEBHOOK
                  valueFrom:
                    secretKeyRef:
                      name: slack-webhook
                      key: url
              command:
                - /bin/sh
                - -c
                - |
                  set -euo pipefail

                  REPORT=""
                  PASS=true

                  # Step 1: Get latest backup
                  LATEST=$(aws s3 cp s3://payments-backup-eu-west-1/latest-backup.txt -)
                  echo "Restoring backup: ${LATEST}"

                  # Step 2: Create temporary database
                  TEMP_DB="recovery_test_$(date +%s)"
                  createdb -h postgres-secondary.database.svc.cluster.local "${TEMP_DB}"

                  # Step 3: Restore backup
                  START_TIME=$(date +%s)
                  pg_restore -h postgres-secondary.database.svc.cluster.local \
                    -d "${TEMP_DB}" \
                    --no-owner \
                    --no-privileges \
                    "s3://payments-backup-eu-west-1/full/${LATEST}/"
                  END_TIME=$(date +%s)
                  RESTORE_DURATION=$((END_TIME - START_TIME))

                  if [ "$RESTORE_DURATION" -gt 60 ]; then
                    REPORT="${REPORT}FAIL: Restore took ${RESTORE_DURATION}s (RTO: 60s)\n"
                    PASS=false
                  else
                    REPORT="${REPORT}PASS: Restore took ${RESTORE_DURATION}s\n"
                  fi

                  # Step 4: Validate data integrity
                  # Check row counts
                  EXPECTED=$(psql -h postgres-primary.database.svc.cluster.local \
                    -t -c "SELECT count(*) FROM payments;")
                  ACTUAL=$(psql -h postgres-secondary.database.svc.cluster.local \
                    -d "${TEMP_DB}" -t -c "SELECT count(*) FROM payments;")

                  if [ "$EXPECTED" != "$ACTUAL" ]; then
                    REPORT="${REPORT}FAIL: Row count mismatch (expected: ${EXPECTED}, got: ${ACTUAL})\n"
                    PASS=false
                  else
                    REPORT="${REPORT}PASS: Row count matches (${EXPECTED})\n"
                  fi

                  # Check latest transaction is within RPO (5 minutes)
                  LATEST_TX=$(psql -h postgres-secondary.database.svc.cluster.local \
                    -d "${TEMP_DB}" -t -c \
                    "SELECT EXTRACT(EPOCH FROM (now() - max(created_at)))::int FROM payments;")
                  if [ "$LATEST_TX" -gt 300 ]; then
                    REPORT="${REPORT}FAIL: Latest transaction is ${LATEST_TX}s old (RPO: 300s)\n"
                    PASS=false
                  else
                    REPORT="${REPORT}PASS: Latest transaction is ${LATEST_TX}s old\n"
                  fi

                  # Check checksums
                  EXPECTED_CHECKSUM=$(psql -h postgres-primary.database.svc.cluster.local \
                    -t -c "SELECT md5(string_agg(id::text, '' ORDER BY id)) FROM payments;")
                  ACTUAL_CHECKSUM=$(psql -h postgres-secondary.database.svc.cluster.local \
                    -d "${TEMP_DB}" -t -c "SELECT md5(string_agg(id::text, '' ORDER BY id)) FROM payments;")

                  if [ "$EXPECTED_CHECKSUM" != "$ACTUAL_CHECKSUM" ]; then
                    REPORT="${REPORT}FAIL: Data checksum mismatch\n"
                    PASS=false
                  else
                    REPORT="${REPORT}PASS: Data checksums match\n"
                  fi

                  # Step 5: Cleanup
                  dropdb -h postgres-secondary.database.svc.cluster.local "${TEMP_DB}"

                  # Step 6: Report
                  STATUS="SUCCESS"
                  if [ "$PASS" = false ]; then
                    STATUS="FAILURE"
                  fi

                  curl -X POST -H 'Content-type: application/json' \
                    --data "{\"text\":\"Recovery Rehearsal ${STATUS}\n${REPORT}\"}" \
                    "$SLACK_WEBHOOK"

                  if [ "$PASS" = false ]; then
                    exit 1
                  fi
```

## Part E: Chaos Dashboard

**Data model -- push experiment results to Prometheus Pushgateway:**

```bash
# After each chaos experiment, push results:
cat <<EOF | curl --data-binary @- http://pushgateway:9091/metrics/job/chaos_experiment
chaos_experiment_result{name="pod-failure",status="pass"} 1
chaos_experiment_recovery_time_seconds{name="pod-failure"} 12
chaos_experiment_result{name="network-latency",status="pass"} 1
chaos_experiment_recovery_time_seconds{name="network-latency"} 0
chaos_experiment_result{name="db-failover",status="pass"} 1
chaos_experiment_recovery_time_seconds{name="db-failover"} 28
chaos_experiment_result{name="node-failure",status="pass"} 1
chaos_experiment_recovery_time_seconds{name="node-failure"} 45
chaos_experiment_result{name="region-failure",status="fail"} 1
chaos_experiment_recovery_time_seconds{name="region-failure"} 180
EOF
```

**Grafana dashboard queries:**

```
Panel 1: Experiment Timeline
  Query: chaos_experiment_result
  Visualization: State timeline
  Legend: {{name}} - {{status}}

Panel 2: Recovery Time vs RTO
  Query: chaos_experiment_recovery_time_seconds
  Visualization: Bar chart
  Override: Add threshold line at RTO target (60s)

Panel 3: Experiment Outcomes Table
  Query: chaos_experiment_result
  Visualization: Table
  Columns: name, status, last run time

Panel 4: Resilience Score
  Query: |
    sum(chaos_experiment_result{status="pass"})
    /
    sum(chaos_experiment_result)
    * 100
  Visualization: Gauge
  Thresholds: green > 80, yellow > 60, red < 60
```

## Common Mistakes

1. **Never testing the backup restore.** A backup that has never been
   restored successfully is not a backup. The automated recovery
   rehearsal catches corruption, storage permission issues, and
   restore time regressions before they matter in a real disaster.

2. **Failing over without checking replication lag.** If the replica
   has 30 seconds of replication lag when you promote it, you lose
   30 seconds of transactions. Always check `pg_last_xact_replay_timestamp()`
   before promoting.

3. **Running chaos experiments in production without rollback procedures.**
   Every chaos experiment must have a one-command rollback. If the
   experiment causes unexpected damage, you need to stop it immediately.
   Write the rollback command before writing the experiment.

4. **Designing the failover runbook for the person who wrote it.** The
   runbook will be executed at 3 AM by someone who has never seen it
   before. Every command must be copy-pasteable. Every check must have
   an explicit expected output. If a step can fail, include the failure
   handling.

5. **Treating chaos engineering as a one-time event.** Resilience decays
   over time as code changes, configurations drift, and team knowledge
   fades. Run chaos experiments on a schedule (weekly for critical
   experiments, monthly for all others) and track the resilience score
   over time.
