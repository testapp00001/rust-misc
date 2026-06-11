# Exercise 04: Create an Automated Rollback System

**Type:** Challenge
**Duration:** 90-120 minutes

## Objective

Build an automated rollback system that monitors a Kubernetes deployment after
a change, detects failures through health metrics, and automatically rolls back
to the previous version if the change is deemed unhealthy. The system must be
configurable, log all decisions, and avoid false-positive rollbacks.

## Context

Your team deploys to production multiple times per day. Manual rollback
decisions take too long -- by the time an engineer is paged, evaluates the
situation, and executes a rollback, 15-30 minutes of degraded service have
elapsed. You need a system that detects deployment failures within 5 minutes
and rolls back automatically.

## Instructions

### Part 1 -- Health Check Script

Create a health check script (`health-check.sh`) that:

1. Accepts parameters:
   - `--namespace` (default: `default`)
   - `--deployment` (required)
   - `--check-interval` (default: `10` seconds)
   - `--check-count` (default: `6` -- number of checks before deciding)
   - `--error-threshold` (default: `3` -- number of failures to trigger rollback)
2. Performs the following checks each interval:
   - Are all pods in `Running` state?
   - Are all readiness probes passing? (Check `kubectl get pods` for
     `Ready` column)
   - Is the deployment's `availableReplicas` equal to `spec.replicas`?
   - Is the rollout status reporting progress? (Detect stuck rollouts)
3. Tracks consecutive failures and prints a clear log line for each check
4. Exits with code 0 if health is stable after `check-count` successful checks
5. Exits with code 1 if `error-threshold` consecutive failures are detected

### Part 2 -- Rollback Executor

Create a rollback script (`rollback-executor.sh`) that:

1. Accepts parameters:
   - `--namespace` (default: `default`)
   - `--deployment` (required)
   - `--max-rollback-revision` (optional -- don't roll back beyond this)
2. Captures the current rollout revision before rolling back
3. Executes `kubectl rollout undo`
4. Waits for the rollback to complete (`kubectl rollout status`)
5. Verifies the rollback succeeded by checking pod health
6. Logs every action with timestamps to a file (`rollback-<deployment>-<timestamp>.log`)
7. Returns appropriate exit codes (0 for success, 1 for failure)

### Part 3 -- Orchestrator

Create an orchestrator script (`change-orchestrator.sh`) that ties everything
together:

1. Accepts parameters:
   - `--namespace` (default: `default`)
   - `--deployment` (required)
   - `--image` (new image to deploy)
2. Records the current image and revision before the change
3. Applies the new image via `kubectl set image`
4. Immediately starts the health check monitor
5. If health check passes: logs success and exits 0
6. If health check fails: invokes the rollback executor
7. After rollback: re-runs health check to confirm the rollback is healthy
8. Logs the entire sequence of events to a structured log file

### Part 4 -- Configuration

Create a `config.env` file that externalizes all configurable parameters:

```bash
# Health check settings
HEALTH_CHECK_INTERVAL=10
HEALTH_CHECK_COUNT=6
ERROR_THRESHOLD=3

# Notification settings (optional)
NOTIFY_ON_ROLLBACK=true
NOTIFICATION_WEBHOOK=""

# Logging
LOG_DIR="/var/log/change-management"
LOG_LEVEL="INFO"  # DEBUG, INFO, WARN, ERROR
```

The scripts should source this file and use its values as defaults, while
still accepting command-line overrides.

### Part 5 -- Testing

Create a test procedure (`test-rollback.md`) that demonstrates:

1. **Happy path** -- Deploy a valid image, health check passes, no rollback
2. **Bad image** -- Deploy a non-existent image tag, verify automatic rollback
   triggers
3. **Slow rollout** -- Deploy an image with a slow startup, verify the stuck
   rollout detection works (does not false-positive)
4. **Idempotency** -- Run the orchestrator twice; the second run should detect
   no change is needed or handle gracefully

For each test, specify:
- The command to run
- The expected behavior
- What to look for in the logs

## Success Criteria

- [ ] `health-check.sh` runs all four check types and reports pass/fail
- [ ] `health-check.sh` correctly detects consecutive failures and exits with code 1
- [ ] `rollback-executor.sh` rolls back, waits, verifies, and logs with timestamps
- [ ] `change-orchestrator.sh` deploys, monitors, and rolls back on failure
- [ ] `config.env` is sourced by all scripts as default configuration
- [ ] All scripts accept command-line arguments that override config file values
- [ ] Test procedure covers happy path, bad image, and slow rollout scenarios
- [ ] Log files are human-readable and include timestamps
- [ ] Scripts are idempotent and handle edge cases (deployment doesn't exist, etc.)

## Hints

<details>
<summary>Hint 1 -- Checking Pod Readiness</summary>

Parse the `READY` column from `kubectl get pods`:

```bash
kubectl get pods -n "$NAMESPACE" -l "app=$DEPLOYMENT" \
  -o jsonpath='{range .items[*]}{.status.conditions[?(@.type=="Ready")].status}{"\n"}{end}'
```

Count how many are `True` vs total.

</details>

<details>
<summary>Hint 2 -- Detecting Stuck Rollouts</summary>

Use `kubectl rollout status` with a timeout:

```bash
if ! kubectl rollout status deployment/"$DEPLOYMENT" -n "$NAMESPACE" --timeout=30s 2>&1; then
    echo "Rollout appears stuck"
fi
```

If `rollout status` times out, the rollout is not progressing.

</details>

<details>
<summary>Hint 3 -- Capturing Current Revision</summary>

Get the current revision from the rollout history:

```bash
CURRENT_REVISION=$(kubectl rollout history deployment/"$DEPLOYMENT" -n "$NAMESPACE" \
  | tail -2 | head -1 | awk '{print $1}')
```

</details>

<details>
<summary>Hint 4 -- Structured Logging</summary>

Create a logging function that all scripts share:

```bash
log() {
    local level="$1"
    shift
    echo "[$(date -u '+%Y-%m-%dT%H:%M:%SZ')] [$level] $*" | tee -a "$LOG_FILE"
}
```

</details>

<details>
<summary>Hint 5 -- Waiting for Rollout</summary>

After `kubectl set image`, the command returns immediately. You must explicitly
wait:

```bash
kubectl set image deployment/"$DEPLOYMENT" "$DEPLOYMENT=$NEW_IMAGE" -n "$NAMESPACE"
if kubectl rollout status deployment/"$DEPLOYMENT" -n "$NAMESPACE" --timeout=120s; then
    log "INFO" "Rollout completed successfully"
else
    log "ERROR" "Rollout failed or timed out"
fi
```

</details>

## Deliverables

1. `health-check.sh` -- executable health check script
2. `rollback-executor.sh` -- executable rollback script
3. `change-orchestrator.sh` -- executable orchestrator script
4. `config.env` -- configuration file
5. `test-rollback.md` -- test procedure document
6. Sample log output from at least one test run
