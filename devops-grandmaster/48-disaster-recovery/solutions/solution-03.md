# Solution 03: Automated Failover Implementation

## Part A: Patroni Configuration

### Complete Patroni YAML Configuration (node-1)

```yaml
# /etc/patroni/config.yml -- node-1 (us-east-1a)

scope: dataforge-cluster
name: node-1

restapi:
  listen: 0.0.0.0:8008
  connect_address: 10.0.1.10:8008
  authentication:
    username: patroni
    password: "secure-restapi-password"

etcd3:
  hosts: 10.0.1.20:2379,10.0.1.21:2379,10.0.1.22:2379
  protocol: https
  cacert: /etc/etcd/ca.crt
  cert: /etc/patroni/etcd-client.crt
  key: /etc/patroni/etcd-client.key

bootstrap:
  dcs:
    ttl: 30
    loop_wait: 10
    retry_timeout: 10
    maximum_lag_on_failover: 1048576  # 1 MB
    synchronous_mode: true
    synchronous_mode_strict: false
    postgresql:
      use_pg_rewind: true
      use_slots: true
      parameters:
        wal_level: replica
        hot_standby: "on"
        max_wal_senders: 5
        max_replication_slots: 5
        wal_log_hints: "on"
        log_connections: "on"
        log_disconnections: "on"
        log_statement: "ddl"
        shared_preload_libraries: "pg_stat_statements"
        synchronous_commit: "on"
        synchronous_standby_names: "*"

  initdb:
    - encoding: UTF8
    - data-checksums

  pg_hba:
    - host replication replicator 10.0.1.0/24 md5
    - host all all 0.0.0.0/0 md5

  users:
    admin:
      password: "admin-password"
      options:
        - createrole
        - createdb
    replicator:
      password: "replicator-password"
      options:
        - replication

postgresql:
  listen: 0.0.0.0:5432
  connect_address: 10.0.1.10:5432
  data_dir: /var/lib/postgresql/15/main
  bin_dir: /usr/lib/postgresql/15/bin
  authentication:
    superuser:
      username: postgres
      password: "postgres-password"
    replication:
      username: replicator
      password: "replicator-password"
  parameters:
    shared_buffers: "4GB"
    effective_cache_size: "12GB"
    maintenance_work_mem: "1GB"
    work_mem: "64MB"
    max_connections: 200
    max_prepared_transactions: 100
    wal_buffers: "64MB"
    checkpoint_completion_target: 0.9
    max_wal_size: "4GB"
    min_wal_size: "1GB"
    random_page_cost: 1.1
    effective_io_concurrency: 200
    huge_pages: "try"

watchdog:
  mode: required
  device: /dev/watchdog
  safety_margin: 5

tags:
  nofailover: false
  noloadbalance: false
  clonefrom: false
  nosync: false
```

### Configuration for node-2 and node-3

The configuration for node-2 and node-3 is identical except for:

```yaml
# node-2 (us-east-1b)
name: node-2
restapi:
  connect_address: 10.0.1.11:8008
postgresql:
  connect_address: 10.0.1.11:5432

# node-3 (us-east-1c)
name: node-3
restapi:
  connect_address: 10.0.1.12:8008
postgresql:
  connect_address: 10.0.1.12:5432
```

### Key Configuration Decisions

- `ttl: 30` -- Leader key expires after 30 seconds. If the leader cannot renew, another node takes over.
- `loop_wait: 10` -- Patroni checks cluster state every 10 seconds.
- `retry_timeout: 10` -- Retry failed operations for up to 10 seconds before giving up.
- `maximum_lag_on_failover: 1048576` -- A replica must have less than 1 MB of replication lag to be eligible for promotion. Prevents promoting a replica with significant lag.
- `synchronous_mode: true` -- At least one replica must acknowledge each write before it is committed. Guarantees RPO=0.
- `watchdog: mode: required` -- Uses the Linux watchdog device to reboot the node if Patroni cannot renew the leader key. This is the last-resort fencing mechanism.

### Common Mistakes to Avoid

- Setting `ttl` too low (e.g., 5 seconds). Network blips cause unnecessary failovers.
- Setting `maximum_lag_on_failover` too high. A replica with 100 MB of lag will lose data on promotion.
- Not enabling `wal_log_hints`. Required for pg_rewind to work correctly.
- Setting `synchronous_mode_strict: true` in production. If all sync replicas are down, the primary stops accepting writes. `false` allows fallback to async mode.

---

## Part B: Distributed Consensus Layer Setup

### etcd Configuration (3-node cluster)

```yaml
# /etc/etcd/etcd.conf.yml -- etcd-node-1 (10.0.1.20)

name: etcd-node-1
data-dir: /var/lib/etcd
listen-peer-urls: https://10.0.1.20:2380
listen-client-urls: https://10.0.1.20:2379,https://127.0.0.1:2379
advertise-client-urls: https://10.0.1.20:2379
initial-advertise-peer-urls: https://10.0.1.20:2380
initial-cluster: etcd-node-1=https://10.0.1.20:2380,etcd-node-2=https://10.0.1.21:2380,etcd-node-3=https://10.0.1.22:2380
initial-cluster-state: new
initial-cluster-token: dataforge-etcd-cluster

client-transport-security:
  cert-file: /etc/etcd/server.crt
  key-file: /etc/etcd/server.key
  client-cert-auth: true
  trusted-ca-file: /etc/etcd/ca.crt

peer-transport-security:
  cert-file: /etc/etcd/peer.crt
  key-file: /etc/etcd/peer.key
  client-cert-auth: true
  trusted-ca-file: /etc/etcd/ca.crt

auto-compaction-mode: periodic
auto-compaction-retention: "1"
quota-backend-bytes: 8589934592  # 8 GB
```

### etcd Health Check Script

```bash
#!/bin/bash
# /usr/local/bin/check-etcd-health.sh

set -euo pipefail

ETCD_ENDPOINTS="https://10.0.1.20:2379,https://10.0.1.21:2379,https://10.0.1.22:2379"
CERT_DIR="/etc/etcd"
LOG_FILE="/var/log/etcd-health-check.log"

check_etcd_health() {
    local timestamp
    timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)

    # Check 1: etcd cluster health
    local health_output
    health_output=$(etcdctl \
        --endpoints="${ETCD_ENDPOINTS}" \
        --cacert="${CERT_DIR}/ca.crt" \
        --cert="${CERT_DIR}/client.crt" \
        --key="${CERT_DIR}/client.key" \
        endpoint health 2>&1) || true

    local healthy_count
    healthy_count=$(echo "${health_output}" | grep -c "is healthy" || true)

    echo "${timestamp} etcd health: ${healthy_count}/3 nodes healthy" >> "${LOG_FILE}"

    if [ "${healthy_count}" -lt 2 ]; then
        echo "CRITICAL: Only ${healthy_count}/3 etcd nodes healthy"
        echo "${timestamp} CRITICAL: etcd cluster degraded" >> "${LOG_FILE}"
        return 1
    fi

    # Check 2: etcd cluster status
    local status_output
    status_output=$(etcdctl \
        --endpoints="${ETCD_ENDPOINTS}" \
        --cacert="${CERT_DIR}/ca.crt" \
        --cert="${CERT_DIR}/client.crt" \
        --key="${CERT_DIR}/client.key" \
        endpoint status --write-out=table 2>&1)

    echo "${timestamp} etcd status:" >> "${LOG_FILE}"
    echo "${status_output}" >> "${LOG_FILE}"

    # Check 3: Patroni key exists in etcd
    local patroni_key
    patroni_key=$(etcdctl \
        --endpoints="${ETCD_ENDPOINTS}" \
        --cacert="${CERT_DIR}/ca.crt" \
        --cert="${CERT_DIR}/client.crt" \
        --key="${CERT_DIR}/client.key" \
        get /service/dataforge-cluster/leader --print-value-only 2>&1) || true

    if [ -z "${patroni_key}" ]; then
        echo "WARNING: No Patroni leader key found in etcd"
        echo "${timestamp} WARNING: No Patroni leader key" >> "${LOG_FILE}"
        return 2
    fi

    echo "${timestamp} Patroni leader: ${patroni_key}" >> "${LOG_FILE}"
    echo "OK: etcd cluster healthy, Patroni leader: ${patroni_key}"
    return 0
}

check_etcd_health
```

### Patroni etcd Integration

Patroni connects to etcd using the configuration in Part A. The key points:

```yaml
# Patroni uses etcd for two purposes:
# 1. Leader election: The primary holds a key with a TTL. If the key expires, another node takes over.
# 2. Cluster state: Patroni stores cluster topology, replication state, and configuration in etcd.

etcd3:
  hosts: 10.0.1.20:2379,10.0.1.21:2379,10.0.1.22:2379
  protocol: https
  cacert: /etc/etcd/ca.crt
  cert: /etc/patroni/etcd-client.crt
  key: /etc/patroni/etcd-client.key
```

### What Happens When etcd Becomes Unavailable

If all 3 etcd nodes become unavailable:

1. Patroni on the primary cannot renew its leader key.
2. After the TTL expires (30 seconds), the primary demotes itself to read-only.
3. No replica can promote because there is no quorum for leader election.
4. The cluster enters a "safe" state: no writes, no split-brain.
5. When etcd recovers, Patroni automatically re-elects a leader and resumes normal operation.

This is the correct behavior: it is safer to stop writes than to risk split-brain.

### Common Mistakes to Avoid

- Running etcd on the same nodes as PostgreSQL. If the node fails, you lose both the database and the consensus layer.
- Not using TLS for etcd communication. etcd stores the leader key and cluster state -- if compromised, an attacker can force a failover.
- Setting the etcd quota too small. Patroni stores WAL position and replication state in etcd. A full etcd cluster stops accepting writes.
- Not monitoring etcd independently. If etcd fails silently, Patroni will demote the primary without an obvious reason.

---

## Part C: Health Check Scripts

### Patroni Health Check Script

```bash
#!/bin/bash
# /usr/local/bin/check-patroni-health.sh

set -euo pipefail

PATRONI_API="http://localhost:8008"
LOG_FILE="/var/log/patroni-health-check.log"
ALERT_WEBHOOK="https://hooks.slack.com/services/T00/B00/xxx"

log() {
    echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) $*" >> "${LOG_FILE}"
}

alert() {
    local message="$1"
    local severity="$2"
    log "ALERT [${severity}]: ${message}"
    curl -s -X POST "${ALERT_WEBHOOK}" \
        -H 'Content-type: application/json' \
        -d "{\"text\":\"[Patroni Health] [${severity}] ${message}\"}" || true
}

check_patroni_liveness() {
    local response
    response=$(curl -s --connect-timeout 5 --max-time 10 "${PATRONI_API}/health" 2>&1) || {
        alert "Patroni REST API unreachable" "CRITICAL"
        return 1
    }

    local status_code
    status_code=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 5 "${PATRONI_API}/health" 2>&1) || true

    if [ "${status_code}" != "200" ]; then
        alert "Patroni health check returned HTTP ${status_code}" "WARNING"
        return 1
    fi

    log "Patroni liveness: OK"
    return 0
}

check_postgresql_running() {
    local response
    response=$(curl -s --connect-timeout 5 "${PATRONI_API}/patroni" 2>&1) || {
        alert "Cannot reach Patroni API for PostgreSQL check" "CRITICAL"
        return 1
    }

    local pg_running
    pg_running=$(echo "${response}" | python3 -c "import sys, json; print(json.load(sys.stdin).get('postmaster_start_time', ''))" 2>/dev/null) || true

    if [ -z "${pg_running}" ]; then
        alert "PostgreSQL is not running according to Patroni" "CRITICAL"
        return 1
    fi

    log "PostgreSQL running since: ${pg_running}"
    return 0
}

check_replication_lag() {
    local response
    response=$(curl -s --connect-timeout 5 "${PATRONI_API}/replica" 2>&1) || return 0  # Not a replica, skip

    local lag
    lag=$(echo "${response}" | python3 -c "
import sys, json
data = json.load(sys.stdin)
print(data.get('replication_state', {}).get('lag', 'unknown'))
" 2>/dev/null) || lag="unknown"

    if [ "${lag}" != "unknown" ] && [ "${lag}" -gt 1048576 ]; then  # > 1 MB
        alert "Replication lag exceeds 1 MB: ${lag} bytes" "WARNING"
        return 1
    fi

    log "Replication lag: ${lag}"
    return 0
}

check_split_brain() {
    # Check if this node thinks it's primary AND another node also thinks it's primary
    local this_role
    this_role=$(curl -s --connect-timeout 5 "${PATRONI_API}/patroni" | \
        python3 -c "import sys, json; print(json.load(sys.stdin).get('role', 'unknown'))" 2>/dev/null) || {
        alert "Cannot determine this node's role" "CRITICAL"
        return 1
    }

    if [ "${this_role}" != "master" ] && [ "${this_role}" != "primary" ]; then
        log "This node is ${this_role}, no split-brain check needed"
        return 0
    fi

    # This node thinks it's primary. Check etcd for the leader key.
    local etcd_leader
    etcd_leader=$(etcdctl \
        --endpoints="https://10.0.1.20:2379,https://10.0.1.21:2379,https://10.0.1.22:2379" \
        --cacert=/etc/etcd/ca.crt \
        --cert=/etc/etcd/client.crt \
        --key=/etc/etcd/client.key \
        get /service/dataforge-cluster/leader --print-value-only 2>&1) || {
        alert "Cannot reach etcd to verify leader" "CRITICAL"
        return 1
    }

    local hostname
    hostname=$(hostname)

    if [ "${etcd_leader}" != "${hostname}" ]; then
        alert "SPLIT-BRAIN DETECTED: This node (${hostname}) is primary but etcd leader is ${etcd_leader}" "CRITICAL"
        # Demote self immediately
        curl -s -X POST "${PATRONI_API}/demote" || true
        return 1
    fi

    log "Split-brain check: OK (leader is ${hostname})"
    return 0
}

# Run all checks
main() {
    log "Starting health checks"
    local failures=0

    check_patroni_liveness || ((failures++))
    check_postgresql_running || ((failures++))
    check_replication_lag || ((failures++))
    check_split_brain || ((failures++))

    if [ "${failures}" -gt 0 ]; then
        log "Health check completed with ${failures} failure(s)"
        exit 1
    fi

    log "Health check completed: all OK"
    exit 0
}

main
```

### Cron Configuration

```cron
# /etc/cron.d/patroni-health-checks

# Patroni health check every 30 seconds
* * * * * root /usr/local/bin/check-patroni-health.sh >> /var/log/patroni-health-check.log 2>&1
* * * * * root sleep 30 && /usr/local/bin/check-patroni-health.sh >> /var/log/patroni-health-check.log 2>&1

# etcd health check every 60 seconds
* * * * * root /usr/local/bin/check-etcd-health.sh >> /var/log/etcd-health-check.log 2>&1
```

### Common Mistakes to Avoid

- Health checks that are too aggressive (every 1 second). This adds load and produces false positives from transient network issues.
- Health checks that only check "is the process running." A running PostgreSQL that cannot accept connections is not healthy.
- Not checking for split-brain. The most dangerous failure mode is two nodes both accepting writes.
- Health checks that fail silently. Every failure should be logged and alerted.

---

## Part D: Alerting Configuration

### Prometheus Alert Rules

```yaml
# /etc/prometheus/rules/patroni-alerts.yml

groups:
  - name: patroni_failover
    rules:
      # Failover detected: primary role changed
      - alert: PatroniFailoverDetected
        expr: changes(patroni_master[5m]) > 0
        for: 0m
        labels:
          severity: critical
          team: database
        annotations:
          summary: "Patroni failover detected on {{ $labels.instance }}"
          description: "The Patroni primary role has changed in the last 5 minutes. This indicates a failover event."
          runbook_url: "https://wiki.internal/runbooks/patroni-failover"

      # Replication lag exceeded threshold
      - alert: PatroniReplicationLagHigh
        expr: patroni_replication_lag > 1048576
        for: 2m
        labels:
          severity: warning
          team: database
        annotations:
          summary: "Replication lag exceeds 1 MB on {{ $labels.instance }}"
          description: "Replication lag is {{ $value }} bytes. This may impact RPO if a failover occurs."

      # Replication lag critical
      - alert: PatroniReplicationLagCritical
        expr: patroni_replication_lag > 10485760
        for: 1m
        labels:
          severity: critical
          team: database
        annotations:
          summary: "Replication lag critical on {{ $labels.instance }}"
          description: "Replication lag is {{ $value }} bytes (10 MB). Immediate investigation required."

      # Patroni node down
      - alert: PatroniNodeDown
        expr: up{job="patroni"} == 0
        for: 1m
        labels:
          severity: critical
          team: database
        annotations:
          summary: "Patroni node {{ $labels.instance }} is down"
          description: "Patroni exporter is not responding on {{ $labels.instance }}."

      # PostgreSQL not running
      - alert: PatroniPostgresDown
        expr: patroni_postgres_running == 0
        for: 0m
        labels:
          severity: critical
          team: database
        annotations:
          summary: "PostgreSQL is not running on {{ $labels.instance }}"
          description: "Patroni reports PostgreSQL is not running. Failover may be in progress."

      # etcd cluster unhealthy
      - alert: EtcdClusterUnhealthy
        expr: etcd_server_has_leader == 0
        for: 0m
        labels:
          severity: critical
          team: infrastructure
        annotations:
          summary: "etcd cluster has no leader"
          description: "etcd cluster has no elected leader. Patroni cannot perform failover."

      # etcd node down
      - alert: EtcdNodeDown
        expr: up{job="etcd"} == 0
        for: 1m
        labels:
          severity: warning
          team: infrastructure
        annotations:
          summary: "etcd node {{ $labels.instance }} is down"
          description: "One etcd node is unreachable. Cluster can tolerate this if 2/3 nodes are healthy."

      # No Patroni leader
      - alert: PatroniNoLeader
        expr: count(patroni_master) == 0
        for: 0m
        labels:
          severity: critical
          team: database
        annotations:
          summary: "No Patroni leader elected"
          description: "No node in the Patroni cluster is reporting as master. Writes are failing."
```

### Alertmanager Routing Configuration

```yaml
# /etc/alertmanager/alertmanager.yml

global:
  resolve_timeout: 5m
  pagerduty_url: 'https://events.pagerduty.com/v2/enqueue'
  slack_api_url: 'https://hooks.slack.com/services/T00/B00/xxx'

route:
  receiver: 'slack-default'
  group_by: ['alertname', 'cluster', 'service']
  group_wait: 10s
  group_interval: 5m
  repeat_interval: 4h

  routes:
    # Critical failover alerts go to PagerDuty
    - match:
        severity: critical
        team: database
      receiver: 'pagerduty-critical'
      group_wait: 0s
      continue: true

    # Database team alerts go to Slack
    - match:
        team: database
      receiver: 'slack-database'

    # Infrastructure alerts go to Slack
    - match:
        team: infrastructure
      receiver: 'slack-infrastructure'

receivers:
  - name: 'slack-default'
    slack_configs:
      - channel: '#alerts-default'
        title: '{{ .GroupLabels.alertname }}'
        text: '{{ range .Alerts }}{{ .Annotations.summary }}\n{{ end }}'

  - name: 'pagerduty-critical'
    pagerduty_configs:
      - service_key: 'REDACTED'
        severity: '{{ .GroupLabels.severity }}'
        description: '{{ .GroupLabels.alertname }}: {{ .CommonAnnotations.summary }}'

  - name: 'slack-database'
    slack_configs:
      - channel: '#db-alerts'
        title: '{{ .GroupLabels.alertname }}'
        text: '{{ range .Alerts }}{{ .Annotations.summary }}\n{{ .Annotations.description }}\n{{ end }}'

  - name: 'slack-infrastructure'
    slack_configs:
      - channel: '#infra-alerts'
        title: '{{ .GroupLabels.alertname }}'
        text: '{{ range .Alerts }}{{ .Annotations.summary }}\n{{ .Annotations.description }}\n{{ end }}'
```

### Audit Webhook Receiver

```bash
#!/bin/bash
# /usr/local/bin/failover-audit-webhook.sh
# Receives webhook from Alertmanager and logs to audit trail

set -euo pipefail

AUDIT_LOG="/var/log/failover-audit.log"
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# Read webhook payload from stdin
PAYLOAD=$(cat)

# Extract alert details
ALERT_NAME=$(echo "${PAYLOAD}" | python3 -c "
import sys, json
data = json.load(sys.stdin)
alerts = data.get('alerts', [])
if alerts:
    print(alerts[0].get('labels', {}).get('alertname', 'unknown'))
else:
    print('unknown')
")

SEVERITY=$(echo "${PAYLOAD}" | python3 -c "
import sys, json
data = json.load(sys.stdin)
alerts = data.get('alerts', [])
if alerts:
    print(alerts[0].get('labels', {}).get('severity', 'unknown'))
else:
    print('unknown')
")

STATUS=$(echo "${PAYLOAD}" | python3 -c "
import sys, json
data = json.load(sys.stdin)
alerts = data.get('alerts', [])
if alerts:
    print(alerts[0].get('status', 'unknown'))
else:
    print('unknown')
")

# Write to audit log
cat >> "${AUDIT_LOG}" << EOF
${TIMESTAMP} | alert=${ALERT_NAME} | severity=${SEVERITY} | status=${STATUS} | payload=${PAYLOAD}
EOF

# If this is a failover alert, also write to failover-specific log
if echo "${ALERT_NAME}" | grep -qi "failover"; then
    cat >> /var/log/failover-events.log << EOF
${TIMESTAMP} | FAILOVER EVENT | ${ALERT_NAME} | ${STATUS}
EOF
fi
```

### Common Mistakes to Avoid

- Alert fatigue. If every replication lag spike triggers PagerDuty, on-call engineers will ignore alerts. Use `for: 2m` to suppress transient spikes.
- Missing the "no leader" alert. This is the most critical alert -- it means writes are failing and no automatic recovery is possible.
- Not having separate channels for failover vs. degraded state. A failover is an event; degraded state is a condition. They need different response urgency.
- Forgetting to test alerts. Alerts that have never fired may have broken webhook URLs or incorrect labels.

---

## Key Takeaway

Automated failover with Patroni reduces RTO from minutes to seconds, but it introduces new risks: false failovers, split-brain, and alert fatigue. The distributed consensus layer (etcd) is the safety net -- when in doubt, Patroni stops writes rather than risk data corruption. Health checks must be comprehensive (liveness, replication lag, split-brain) but not so aggressive that they trigger unnecessary failovers. Alerting must distinguish between "failover happened" (critical, immediate) and "degraded state" (warning, investigation needed).
