# Solution 04: Disaster Recovery Drill

## Part A: DR Drill Plan

### DR Drill Plan Document

```markdown
# DR Drill Plan: NimbusCloud PostgreSQL Failover
# Date: [Scheduled Date]
# Time: [Maintenance Window, e.g., Saturday 02:00-04:00 UTC]
# Environment: Staging

## 1. Objectives

1. Validate that automatic failover completes within 60 seconds (RTO target).
2. Validate that zero data loss occurs during failover (RPO target = 0).
3. Validate that applications reconnect to the new primary without manual intervention.
4. Identify any gaps in monitoring, alerting, or runbook procedures.

## 2. Success Criteria (Binary Pass/Fail)

| Metric | Target | Tolerance | Pass Condition |
|--------|--------|-----------|----------------|
| RTO | 60 sec | +15 sec buffer | Actual RTO <= 75 seconds |
| RPO | 0 sec | N/A | Zero committed transactions lost |
| Application recovery | Automatic | N/A | All 12 pods healthy within 90 seconds |
| Alerting | < 60 sec | N/A | PagerDuty alert fires within 60 seconds of failure |
| Data integrity | 100% | N/A | All test records present and correct after failover |

## 3. Scope

### Included
- PostgreSQL 3-node Patroni cluster (staging)
- etcd 3-node consensus cluster (staging)
- 12 application pods (staging)
- Prometheus/Alertmanager alerting pipeline
- PgBouncer connection pooling

### Excluded
- Production environment
- Redis cache layer (separate DR exercise)
- S3 file storage (already cross-region replicated)
- DNS failover (single-region exercise)

## 4. Timeline

| Time (UTC) | Activity | Owner |
|------------|----------|-------|
| 01:30 | Pre-drill health checks | SRE Lead |
| 01:45 | Start continuous write load | SRE Engineer |
| 01:50 | Verify monitoring baseline | SRE Engineer |
| 02:00 | DRILL START -- Inject failure | SRE Lead |
| 02:01 | Observe failover (no intervention) | All |
| 02:02 | Verify new primary accepts writes | SRE Engineer |
| 02:05 | Verify application reconnection | SRE Engineer |
| 02:10 | Calculate RPO and RTO | SRE Engineer |
| 02:15 | DRILL END -- Begin cleanup | SRE Lead |
| 02:30 | Restore original topology | SRE Lead |
| 03:00 | Post-drill debrief | All |
| 03:30 | Write post-drill report | SRE Lead |

## 5. Roles

| Role | Person | Responsibility |
|------|--------|----------------|
| Drill Commander | SRE Lead | Executes failure injection, calls abort if needed |
| Observer | SRE Engineer | Watches monitoring, records timestamps |
| App Validator | Platform Engineer | Verifies application health and reconnection |
| Communications | Engineering Manager | Notifies stakeholders of drill status |
| Safety Officer | DBA Lead | Monitors for unintended production impact |

## 6. Abort Criteria

The drill is immediately aborted if:
1. Production environment is affected (any production alerts fire).
2. The staging environment enters an unrecoverable state (data corruption detected).
3. The failure spreads beyond the intended scope (e.g., etcd cluster fully fails).
4. The drill exceeds the maintenance window (04:00 UTC).

Abort procedure:
1. Drill Commander announces "DRILL ABORT" on Slack.
2. Restore the failed node immediately.
3. Verify staging environment stability.
4. Document the reason for abort and reschedule.
```

### Common Mistakes to Avoid

- No defined abort criteria. Without them, the team may continue a drill that is damaging the environment.
- Vague success criteria. "RTO should be fast" is not measurable. "RTO <= 75 seconds" is.
- Not having a safety officer. Someone must watch for unintended production impact.
- Scheduling during business hours. Even staging drills can have unexpected consequences.

---

## Part B: Failure Simulation Script

### Script: Kill Patroni Process

```bash
#!/bin/bash
# /usr/local/bin/simulate-patroni-kill.sh
# Kills the Patroni process on the primary node to simulate failure

set -euo pipefail

DRY_RUN="${1:-false}"
PRIMARY_HOST="${2:-}"
LOG_FILE="/var/log/dr-drill.log"

log() {
    echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) $*" | tee -a "${LOG_FILE}"
}

confirm() {
    local message="$1"
    read -rp "${message} (yes/no): " answer
    if [ "${answer}" != "yes" ]; then
        log "Aborted by user"
        exit 0
    fi
}

find_primary() {
    if [ -n "${PRIMARY_HOST}" ]; then
        echo "${PRIMARY_HOST}"
        return
    fi

    local primary
    primary=$(patronictl list --format json 2>/dev/null | \
        python3 -c "
import sys, json
data = json.load(sys.stdin)
for node in data:
    if node.get('Role') in ('Leader', 'Master'):
        print(node.get('Host', ''))
        break
" 2>/dev/null)

    if [ -z "${primary}" ]; then
        log "ERROR: Cannot determine primary host"
        exit 1
    fi

    echo "${primary}"
}

simulate_kill() {
    local target_host="$1"

    log "Target primary: ${target_host}"

    if [ "${DRY_RUN}" = "--dry-run" ]; then
        log "DRY RUN: Would kill Patroni on ${target_host}"
        log "Command: ssh ${target_host} 'sudo systemctl kill patroni'"
        return 0
    fi

    confirm "This will kill Patroni on ${target_host}. Proceed?"

    log "Killing Patroni on ${target_host}"
    ssh "${target_host}" "sudo systemctl kill patroni"

    log "Patroni killed on ${target_host} at $(date -u +%Y-%m-%dT%H:%M:%SZ)"
}

cleanup() {
    local target_host="$1"

    log "Cleanup: Restarting Patroni on ${target_host}"
    ssh "${target_host}" "sudo systemctl restart patroni"

    log "Waiting for Patroni to rejoin cluster..."
    sleep 10

    local status
    status=$(patronictl list --format json 2>/dev/null | python3 -c "
import sys, json
data = json.load(sys.stdin)
for node in data:
    if node.get('Host') == '${target_host}':
        print(node.get('State', 'unknown'))
        break
" 2>/dev/null) || status="unknown"

    log "Node ${target_host} state: ${status}"
}

main() {
    log "=== DR Drill: Patroni Kill Simulation ==="

    local primary
    primary=$(find_primary)
    log "Detected primary: ${primary}"

    simulate_kill "${primary}"

    if [ "${DRY_RUN}" = "--dry-run" ]; then
        log "DRY RUN complete. No changes made."
        exit 0
    fi

    log "Failure injected. Waiting for drill commander to call cleanup..."
    log "To restore: $0 --cleanup ${primary}"
}

if [ "${1:-}" = "--cleanup" ]; then
    cleanup "${2:?Host required for cleanup}"
else
    main
fi
```

### Script: Network Partition Simulation

```bash
#!/bin/bash
# /usr/local/bin/simulate-network-partition.sh
# Simulates a network partition isolating the primary from replicas

set -euo pipefail

DRY_RUN="${1:-false}"
PRIMARY_HOST="${2:-}"
REPLICA_HOSTS="${3:-}"
LOG_FILE="/var/log/dr-drill.log"

log() {
    echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) $*" | tee -a "${LOG_FILE}"
}

confirm() {
    local message="$1"
    read -rp "${message} (yes/no): " answer
    if [ "${answer}" != "yes" ]; then
        log "Aborted by user"
        exit 0
    fi
}

apply_partition() {
    local target="$1"
    local blocked_hosts="$2"

    log "Applying network partition on ${target}"

    for host in ${blocked_hosts}; do
        local cmd="sudo iptables -A INPUT -s ${host} -j DROP && sudo iptables -A OUTPUT -d ${host} -j DROP"

        if [ "${DRY_RUN}" = "--dry-run" ]; then
            log "DRY RUN: Would run on ${target}: ${cmd}"
        else
            ssh "${target}" "${cmd}"
            log "Blocked ${host} on ${target}"
        fi
    done
}

remove_partition() {
    local target="$1"

    log "Removing network partition on ${target}"

    if [ "${DRY_RUN}" = "--dry-run" ]; then
        log "DRY RUN: Would remove iptables rules on ${target}"
    else
        ssh "${target}" "sudo iptables -F INPUT && sudo iptables -F OUTPUT"
        log "Removed iptables rules on ${target}"
    fi
}

main() {
    local action="${1:-inject}"
    local primary="${PRIMARY_HOST:?Primary host required}"
    local replicas="${REPLICA_HOSTS:?Replica hosts required (space-separated)}"

    case "${action}" in
        inject)
            log "=== DR Drill: Network Partition Simulation ==="
            confirm "This will partition ${primary} from replicas. Proceed?"
            apply_partition "${primary}" "${replicas}"
            log "Network partition applied. Primary ${primary} is isolated from replicas."
            log "To restore: $0 --cleanup ${primary} ${replicas}"
            ;;
        cleanup)
            log "=== DR Drill: Removing Network Partition ==="
            remove_partition "${primary}"
            log "Network partition removed."
            ;;
        *)
            echo "Usage: $0 {inject|--cleanup} PRIMARY_HOST REPLICA_HOSTS..."
            exit 1
            ;;
    esac
}

main "$@"
```

### Script: Slow Disk Simulation

```bash
#!/bin/bash
# /usr/local/bin/simulate-slow-disk.sh
# Simulates degraded I/O using dm-delay

set -euo pipefail

DRY_RUN="${1:-false}"
TARGET_HOST="${2:-}"
DELAY_MS="${3:-500}"  # 500ms delay per I/O

log() {
    echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) $*" | tee -a "/var/log/dr-drill.log"
}

confirm() {
    read -rp "$1 (yes/no): " answer
    [ "${answer}" = "yes" ] || { log "Aborted"; exit 0; }
}

apply_slow_disk() {
    local host="$1"
    local delay="$2"

    local cmd=$(cat <<'SCRIPT'
# Find the PostgreSQL data disk
PG_DISK=$(df /var/lib/postgresql/15/main | tail -1 | awk '{print $1}')
DM_NAME="pg_delay"

# Create dm-delay device
sudo dmsetup create ${DM_NAME} --table \
  "0 $(sudo blockdev --getsz ${PG_DISK}) delay ${PG_DISK} 0 ${DELAY}"

# Remount with delayed device
sudo mount /dev/mapper/${DM_NAME} /var/lib/postgresql/15/main
SCRIPT
)

    cmd=$(echo "${cmd}" | sed "s/\${DELAY}/${delay}/")

    if [ "${DRY_RUN}" = "--dry-run" ]; then
        log "DRY RUN: Would apply ${delay}ms I/O delay on ${host}"
    else
        confirm "This will add ${delay}ms I/O delay on ${host}. Proceed?"
        ssh "${host}" "${cmd}"
        log "Applied ${delay}ms I/O delay on ${host}"
    fi
}

remove_slow_disk() {
    local host="$1"

    if [ "${DRY_RUN}" = "--dry-run" ]; then
        log "DRY RUN: Would remove I/O delay on ${host}"
    else
        ssh "${host}" "sudo umount /var/lib/postgresql/15/main && sudo dmsetup remove pg_delay && sudo mount ${PG_DISK} /var/lib/postgresql/15/main"
        log "Removed I/O delay on ${host}"
    fi
}

case "${1:-inject}" in
    inject) apply_slow_disk "${TARGET_HOST}" "${DELAY_MS}" ;;
    --cleanup) remove_slow_disk "${TARGET_HOST}" ;;
    *) echo "Usage: $0 {inject|--cleanup} HOST [DELAY_MS]"; exit 1 ;;
esac
```

### Common Mistakes to Avoid

- No dry-run mode. Every failure injection script must support `--dry-run` to verify the command before executing.
- No cleanup function. If the drill is aborted, you need a way to undo the failure injection.
- Destructive commands without confirmation prompts. A typo in the target host should not destroy production.
- Not testing the simulation script in isolation first. Run it against a single test node before using it in a drill.

---

## Part C: Drill Execution and Measurement

### Continuous Write Load Script

```python
#!/usr/bin/env python3
# /usr/local/bin/dr-drill-writer.py
# Continuously inserts test records with timestamps for RPO measurement

import psycopg2
import time
import sys
import json
from datetime import datetime, timezone

DB_CONFIG = {
    "host": "pgbouncer.staging.internal",
    "port": 6432,
    "dbname": "drill_test",
    "user": "drill_user",
    "password": "drill_password",
}

def create_test_table(conn):
    with conn.cursor() as cur:
        cur.execute("""
            CREATE TABLE IF NOT EXISTS drill_writes (
                id BIGSERIAL PRIMARY KEY,
                written_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                writer_host TEXT NOT NULL,
                sequence_num BIGINT NOT NULL
            )
        """)
        conn.commit()

def write_loop():
    conn = psycopg2.connect(**DB_CONFIG)
    create_test_table(conn)

    sequence = 0
    writer_host = f"writer-{datetime.now(timezone.utc).strftime('%Y%m%d%H%M%S')}"
    results_file = f"/tmp/dr-drill-writes-{writer_host}.jsonl"

    print(f"Starting write loop. Results: {results_file}")

    with open(results_file, "w") as f:
        while True:
            sequence += 1
            try:
                with conn.cursor() as cur:
                    start = time.monotonic()
                    cur.execute(
                        "INSERT INTO drill_writes (writer_host, sequence_num) VALUES (%s, %s) RETURNING id, written_at",
                        (writer_host, sequence)
                    )
                    row = cur.fetchone()
                    conn.commit()
                    elapsed = time.monotonic() - start

                    result = {
                        "sequence": sequence,
                        "id": row[0],
                        "written_at": row[1].isoformat(),
                        "elapsed_ms": round(elapsed * 1000, 2),
                        "status": "success",
                        "timestamp": datetime.now(timezone.utc).isoformat(),
                    }
                    f.write(json.dumps(result) + "\n")
                    f.flush()

                    if sequence % 100 == 0:
                        print(f"Write #{sequence}: {elapsed*1000:.1f}ms")

            except Exception as e:
                elapsed = time.monotonic() - start
                result = {
                    "sequence": sequence,
                    "status": "error",
                    "error": str(e),
                    "elapsed_ms": round(elapsed * 1000, 2),
                    "timestamp": datetime.now(timezone.utc).isoformat(),
                }
                f.write(json.dumps(result) + "\n")
                f.flush()
                print(f"Write #{sequence} FAILED: {e}")

                # Reconnect
                try:
                    conn.close()
                except Exception:
                    pass
                time.sleep(1)
                try:
                    conn = psycopg2.connect(**DB_CONFIG)
                except Exception as conn_err:
                    print(f"Reconnect failed: {conn_err}")

            time.sleep(0.1)  # 10 writes per second

if __name__ == "__main__":
    write_loop()
```

### RPO/RTO Measurement Script

```python
#!/usr/bin/env python3
# /usr/local/bin/dr-drill-measure.py
# Analyzes write logs to calculate actual RPO and RTO

import json
import sys
from datetime import datetime, timezone

def parse_write_log(log_file):
    """Parse the write log and identify the failure window."""
    successes = []
    errors = []

    with open(log_file) as f:
        for line in f:
            record = json.loads(line.strip())
            if record["status"] == "success":
                successes.append(record)
            else:
                errors.append(record)

    return successes, errors

def calculate_rto(successes, errors):
    """Calculate RTO: time between last successful write and first successful write after failure."""
    if not successes or not errors:
        return None

    # Find the failure window: first error after a series of successes
    failure_start = None
    for i, record in enumerate(successes):
        if i + 1 < len(successes):
            gap = (
                datetime.fromisoformat(successes[i + 1]["timestamp"]) -
                datetime.fromisoformat(record["timestamp"])
            )
            if gap.total_seconds() > 5:  # Gap > 5 seconds indicates failure
                failure_start = record["timestamp"]
                break

    if not failure_start:
        # Use first error timestamp
        failure_start = errors[0]["timestamp"]

    # Find first successful write after failure
    failure_start_dt = datetime.fromisoformat(failure_start)
    recovery_point = None
    for record in successes:
        if datetime.fromisoformat(record["timestamp"]) > failure_start_dt:
            recovery_point = record["timestamp"]
            break

    if not recovery_point:
        return None

    rto = (
        datetime.fromisoformat(recovery_point) -
        datetime.fromisoformat(failure_start)
    ).total_seconds()

    return {
        "failure_start": failure_start,
        "recovery_point": recovery_point,
        "rto_seconds": rto,
    }

def calculate_rpo(successes):
    """Calculate RPO: gap between last write on old primary and first readable write on new primary."""
    if len(successes) < 2:
        return None

    # Find the largest sequential gap in write IDs
    max_gap = 0
    gap_position = None

    for i in range(len(successes) - 1):
        current_id = successes[i]["id"]
        next_id = successes[i + 1]["id"]
        gap = next_id - current_id - 1  # Number of missing IDs

        if gap > max_gap:
            max_gap = gap
            gap_position = {
                "last_write_before_gap": successes[i],
                "first_write_after_gap": successes[i + 1],
                "missing_records": gap,
            }

    # RPO is the time corresponding to the missing records
    if gap_position:
        last_ts = datetime.fromisoformat(gap_position["last_write_before_gap"]["timestamp"])
        first_ts = datetime.fromisoformat(gap_position["first_write_after_gap"]["timestamp"])
        rpo_seconds = (first_ts - last_ts).total_seconds()
    else:
        rpo_seconds = 0

    return {
        "rpo_seconds": rpo_seconds,
        "missing_records": max_gap,
        "gap_details": gap_position,
    }

def main():
    log_file = sys.argv[1] if len(sys.argv) > 1 else "/tmp/dr-drill-writes-writer-*.jsonl"

    successes, errors = parse_write_log(log_file)

    print("=" * 60)
    print("DR DRILL MEASUREMENT RESULTS")
    print("=" * 60)

    print(f"\nTotal writes attempted: {len(successes) + len(errors)}")
    print(f"Successful writes: {len(successes)}")
    print(f"Failed writes: {len(errors)}")

    # RTO
    rto = calculate_rto(successes, errors)
    if rto:
        print(f"\n--- RTO ---")
        print(f"Failure detected at: {rto['failure_start']}")
        print(f"Recovery at: {rto['recovery_point']}")
        print(f"Actual RTO: {rto['rto_seconds']:.1f} seconds")
        target_rto = 75  # 60 sec target + 15 sec buffer
        print(f"Target RTO: {target_rto} seconds")
        print(f"RTO {'PASS' if rto['rto_seconds'] <= target_rto else 'FAIL'}")

    # RPO
    rpo = calculate_rpo(successes)
    if rpo:
        print(f"\n--- RPO ---")
        print(f"Actual RPO: {rpo['rpo_seconds']:.1f} seconds")
        print(f"Missing records: {rpo['missing_records']}")
        print(f"RPO {'PASS' if rpo['missing_records'] == 0 else 'FAIL'}")

    # Data integrity
    print(f"\n--- Data Integrity ---")
    ids = [s["id"] for s in successes]
    if ids:
        expected_count = max(ids) - min(ids) + 1
        actual_count = len(ids)
        gaps = expected_count - actual_count
        print(f"ID range: {min(ids)} to {max(ids)}")
        print(f"Expected records: {expected_count}")
        print(f"Actual records: {actual_count}")
        print(f"Missing records: {gaps}")
        print(f"Integrity {'PASS' if gaps == 0 else 'FAIL'}")

if __name__ == "__main__":
    main()
```

### Common Mistakes to Avoid

- Measuring RPO at the database level only. Application-level writes may be buffered in PgBouncer. Measure end-to-end.
- Using wall clock time for RTO. Use monotonic time or synchronized NTP clocks to avoid clock skew.
- Not running the write load long enough before injecting failure. You need a baseline of successful writes to measure the gap.
- Forgetting to verify data integrity after failover. RPO=0 means nothing if the data is corrupted.

---

## Part D: Post-Drill Report Template

```markdown
# DR Drill Report: NimbusCloud PostgreSQL Failover
# Drill Date: [Date]
# Report Author: [Name]

## Executive Summary

[3 sentences: What was tested, what happened, and whether the drill passed or failed.]

Example: "We tested automatic PostgreSQL failover in the staging environment by killing the
Patroni process on the primary node. Failover completed in 42 seconds with zero data loss.
The drill PASSED all success criteria."

## Drill Timeline

| Time (UTC) | Event | Duration | Notes |
|------------|-------|----------|-------|
| 01:30 | Pre-drill health checks started | 15 min | All systems healthy |
| 01:45 | Write load started | -- | 10 writes/sec sustained |
| 02:00 | Failure injected (Patroni killed) | -- | node-1, us-east-1a |
| 02:00:15 | Patroni detected leader missing | 15 sec | etcd key expired |
| 02:00:30 | node-2 promoted to primary | 15 sec | Automatic promotion |
| 02:00:42 | First successful write on new primary | 12 sec | Application reconnected |
| 02:01:00 | PagerDuty alert fired | 60 sec | Alertmanager -> PagerDuty |
| 02:02:00 | All 12 app pods healthy | 2 min | Kubernetes readiness probes |
| 02:15 | RPO/RTO measurement complete | -- | See metrics below |
| 02:30 | Original topology restored | 15 min | node-1 rebuilt as replica |
| 03:00 | Post-drill debrief | 30 min | Attendees: [list] |

## RPO/RTO Results

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| RTO | <= 75 sec | 42 sec | PASS |
| RPO | 0 sec | 0 sec | PASS |
| Application recovery | <= 90 sec | 120 sec | FAIL |
| Alert latency | <= 60 sec | 60 sec | PASS |
| Data integrity | 100% | 100% | PASS |

## Detailed Metrics

- Detection time: 15 seconds (etcd TTL expiry)
- Promotion time: 15 seconds (pg_promote + confirmation)
- Application reconnection: 12 seconds (PgBouncer + Kubernetes)
- Total RTO: 42 seconds
- Missing records: 0 (synchronous replication confirmed)

## Issues Found

### Issue 1: Application Reconnection Too Slow
- **Severity:** Medium
- **Description:** Application pods took 120 seconds to reconnect, exceeding the 90-second target.
- **Root cause:** Kubernetes readiness probe interval was 10 seconds with 3 failures threshold (30 seconds), plus PgBouncer reconnection timeout was 90 seconds.
- **Recommendation:** Reduce PgBouncer server_connect_timeout from 90s to 15s. Reduce readiness probe period from 10s to 5s.

### Issue 2: [Template for additional issues]
- **Severity:** [Low/Medium/High/Critical]
- **Description:** [What happened]
- **Root cause:** [Why it happened]
- **Recommendation:** [What to fix]

## Gap Analysis

| Area | Expected | Actual | Gap | Action Required |
|------|----------|--------|-----|-----------------|
| RTO | 60 sec | 42 sec | -18 sec | None (better than target) |
| App recovery | 60 sec | 120 sec | +60 sec | Fix PgBouncer timeout |
| Alert latency | 30 sec | 60 sec | +30 sec | Review Alertmanager config |

## Recommendations

1. **Immediate (this sprint):** Reduce PgBouncer server_connect_timeout to 15 seconds.
2. **Short-term (next sprint):** Reduce Kubernetes readiness probe interval to 5 seconds.
3. **Medium-term (this quarter):** Conduct a cross-region DR drill.
4. **Long-term (this year):** Implement application-layer retry with exponential backoff.

## Lessons Learned

| What Went Well | What Needs Improvement |
|----------------|----------------------|
| Automatic failover worked as designed | Application reconnection was slow |
| Zero data loss confirmed | Alert latency needs tuning |
| Team response was calm and methodical | Runbook did not cover the specific failure mode |

## Action Items

| # | Action | Owner | Due Date | Status |
|---|--------|-------|----------|--------|
| 1 | Reduce PgBouncer timeout | Platform Engineer | [Date + 1 week] | Open |
| 2 | Update readiness probe config | SRE Engineer | [Date + 1 week] | Open |
| 3 | Schedule cross-region drill | SRE Lead | [Date + 1 month] | Open |
| 4 | Update runbook with drill findings | DBA Lead | [Date + 2 weeks] | Open |

## Appendix

- Raw write log: [link to /tmp/dr-drill-writes-*.jsonl]
- Prometheus query for RTO: [link to Grafana dashboard]
- Alertmanager log: [link to alert log]
- Drill recording (if available): [link to video/screenshots]
```

### Common Mistakes to Avoid

- Writing the report days after the drill. Details are forgotten. Write the timeline during the drill.
- Not assigning action items with owners and due dates. Unassigned actions do not get done.
- Reporting only successes. The value of the drill is in the failures and gaps.
- Not sharing the report broadly. The engineering team, management, and compliance all benefit from seeing DR drill results.

---

## Key Takeaway

A DR drill is a controlled experiment, not a fire drill. It requires a plan with measurable success criteria, failure injection scripts with safety controls, measurement instrumentation that captures RPO and RTO at the application level, and a report that produces actionable improvements. The most valuable drills are the ones that reveal unexpected failures -- those findings prevent real disasters.
