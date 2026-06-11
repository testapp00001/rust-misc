# Solution 04: Automated Rollback System

## config.env

```bash
#!/usr/bin/env bash
# config.env -- Default configuration for the automated rollback system
# All values can be overridden via command-line arguments

# Health check settings
HEALTH_CHECK_INTERVAL=10        # seconds between health checks
HEALTH_CHECK_COUNT=6            # number of checks before declaring stable
ERROR_THRESHOLD=3               # consecutive failures before triggering rollback

# Rollback settings
ROLLBACK_TIMEOUT=120            # seconds to wait for rollback to complete
HEALTH_CHECK_TIMEOUT=30         # seconds for individual kubectl commands

# Notification settings
NOTIFY_ON_ROLLBACK=true
NOTIFICATION_WEBHOOK=""         # Slack/webhook URL for notifications

# Logging
LOG_DIR="/var/log/change-management"
LOG_LEVEL="INFO"                # DEBUG, INFO, WARN, ERROR

# Deployment defaults
DEFAULT_NAMESPACE="default"
```

## health-check.sh

```bash
#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/config.env"

# --- Argument Parsing ---
NAMESPACE="${DEFAULT_NAMESPACE}"
DEPLOYMENT=""
CHECK_INTERVAL="${HEALTH_CHECK_INTERVAL}"
CHECK_COUNT="${HEALTH_CHECK_COUNT}"
ERR_THRESHOLD="${ERROR_THRESHOLD}"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --namespace)       NAMESPACE="$2";       shift 2 ;;
        --deployment)      DEPLOYMENT="$2";      shift 2 ;;
        --check-interval)  CHECK_INTERVAL="$2";  shift 2 ;;
        --check-count)     CHECK_COUNT="$2";     shift 2 ;;
        --error-threshold) ERR_THRESHOLD="$2";   shift 2 ;;
        *) echo "Unknown option: $1"; exit 2 ;;
    esac
done

if [[ -z "$DEPLOYMENT" ]]; then
    echo "ERROR: --deployment is required"
    exit 2
fi

# --- Logging ---
mkdir -p "${LOG_DIR}"
LOG_FILE="${LOG_DIR}/health-check-${DEPLOYMENT}-$(date -u '+%Y%m%dT%H%M%SZ').log"

log() {
    local level="$1"
    shift
    local msg="[$(date -u '+%Y-%m-%dT%H:%M:%SZ')] [$level] [${DEPLOYMENT}] $*"
    echo "$msg" | tee -a "$LOG_FILE"
}

# --- Health Checks ---

check_pods_running() {
    local not_running
    not_running=$(kubectl get pods -n "$NAMESPACE" -l "app=${DEPLOYMENT}" \
        --field-selector='status.phase!=Running' \
        -o name 2>/dev/null | wc -l)
    if [[ "$not_running" -gt 0 ]]; then
        log "WARN" "Pod readiness check: $not_running pod(s) not in Running state"
        return 1
    fi
    return 0
}

check_pods_ready() {
    local total ready
    total=$(kubectl get pods -n "$NAMESPACE" -l "app=${DEPLOYMENT}" \
        -o jsonpath='{range .items[*]}{.status.conditions[?(@.type=="Ready")].status}{"\n"}{end}' 2>/dev/null \
        | wc -l)
    ready=$(kubectl get pods -n "$NAMESPACE" -l "app=${DEPLOYMENT}" \
        -o jsonpath='{range .items[*]}{.status.conditions[?(@.type=="Ready")].status}{"\n"}{end}' 2>/dev/null \
        | grep -c "True" || true)

    if [[ "$total" -eq 0 ]]; then
        log "WARN" "Readiness check: no pods found for deployment ${DEPLOYMENT}"
        return 1
    fi
    if [[ "$ready" -lt "$total" ]]; then
        log "WARN" "Readiness check: $ready/$total pods ready"
        return 1
    fi
    log "DEBUG" "Readiness check: $ready/$total pods ready"
    return 0
}

check_available_replicas() {
    local desired available
    desired=$(kubectl get deployment "$DEPLOYMENT" -n "$NAMESPACE" \
        -o jsonpath='{.spec.replicas}' 2>/dev/null)
    available=$(kubectl get deployment "$DEPLOYMENT" -n "$NAMESPACE" \
        -o jsonpath='{.status.availableReplicas}' 2>/dev/null)

    if [[ -z "$desired" || -z "$available" ]]; then
        log "WARN" "Replica check: could not read deployment status"
        return 1
    fi
    if [[ "$available" -lt "$desired" ]]; then
        log "WARN" "Replica check: $available/$desired replicas available"
        return 1
    fi
    log "DEBUG" "Replica check: $available/$desired replicas available"
    return 0
}

check_rollout_progress() {
    local status_output
    if ! status_output=$(kubectl rollout status deployment/"$DEPLOYMENT" \
        -n "$NAMESPACE" --timeout="${HEALTH_CHECK_TIMEOUT}s" 2>&1); then
        log "WARN" "Rollout check: rollout not progressing -- $status_output"
        return 1
    fi
    log "DEBUG" "Rollout check: $status_output"
    return 0
}

# --- Main Loop ---
log "INFO" "Starting health check for deployment=${DEPLOYMENT} namespace=${NAMESPACE}"
log "INFO" "Settings: interval=${CHECK_INTERVAL}s count=${CHECK_COUNT} threshold=${ERR_THRESHOLD}"

consecutive_failures=0
consecutive_successes=0

while true; do
    check_failed=false

    # Run all checks
    for check_fn in check_pods_running check_pods_ready check_available_replicas check_rollout_progress; do
        if ! $check_fn; then
            check_failed=true
            break
        fi
    done

    if $check_failed; then
        consecutive_failures=$((consecutive_failures + 1))
        consecutive_successes=0
        log "ERROR" "Health check FAILED (consecutive failures: ${consecutive_failures}/${ERR_THRESHOLD})"

        if [[ "$consecutive_failures" -ge "$ERR_THRESHOLD" ]]; then
            log "ERROR" "ERROR THRESHOLD REACHED (${consecutive_failures} >= ${ERR_THRESHOLD}). Deployment is unhealthy."
            exit 1
        fi
    else
        consecutive_successes=$((consecutive_successes + 1))
        consecutive_failures=0
        log "INFO" "Health check PASSED (${consecutive_successes}/${CHECK_COUNT})"

        if [[ "$consecutive_successes" -ge "$CHECK_COUNT" ]]; then
            log "INFO" "Deployment is STABLE after ${CHECK_COUNT} consecutive successful checks."
            exit 0
        fi
    fi

    sleep "$CHECK_INTERVAL"
done
```

## rollback-executor.sh

```bash
#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/config.env"

# --- Argument Parsing ---
NAMESPACE="${DEFAULT_NAMESPACE}"
DEPLOYMENT=""
MAX_REVISION=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --namespace)              NAMESPACE="$2";       shift 2 ;;
        --deployment)             DEPLOYMENT="$2";      shift 2 ;;
        --max-rollback-revision)  MAX_REVISION="$2";    shift 2 ;;
        *) echo "Unknown option: $1"; exit 2 ;;
    esac
done

if [[ -z "$DEPLOYMENT" ]]; then
    echo "ERROR: --deployment is required"
    exit 2
fi

# --- Logging ---
mkdir -p "${LOG_DIR}"
TIMESTAMP=$(date -u '+%Y%m%dT%H%M%SZ')
LOG_FILE="${LOG_DIR}/rollback-${DEPLOYMENT}-${TIMESTAMP}.log"

log() {
    local level="$1"
    shift
    local msg="[$(date -u '+%Y-%m-%dT%H:%M:%SZ')] [$level] [rollback:${DEPLOYMENT}] $*"
    echo "$msg" | tee -a "$LOG_FILE"
}

# --- Capture Current State ---
log "INFO" "Starting rollback for deployment=${DEPLOYMENT} namespace=${NAMESPACE}"

CURRENT_IMAGE=$(kubectl get deployment "$DEPLOYMENT" -n "$NAMESPACE" \
    -o jsonpath='{.spec.template.spec.containers[0].image}' 2>/dev/null || echo "unknown")
log "INFO" "Current image before rollback: ${CURRENT_IMAGE}"

CURRENT_REVISION=$(kubectl rollout history deployment/"$DEPLOYMENT" -n "$NAMESPACE" \
    2>/dev/null | tail -2 | head -1 | awk '{print $1}' || echo "unknown")
log "INFO" "Current revision: ${CURRENT_REVISION}"

# --- Check max-rollback-revision ---
if [[ -n "$MAX_REVISION" && "$CURRENT_REVISION" != "unknown" ]]; then
    if [[ "$CURRENT_REVISION" -le "$MAX_REVISION" ]]; then
        log "ERROR" "Cannot roll back: current revision (${CURRENT_REVISION}) <= max revision (${MAX_REVISION})"
        exit 1
    fi
fi

# --- Execute Rollback ---
log "INFO" "Executing: kubectl rollout undo deployment/${DEPLOYMENT} -n ${NAMESPACE}"
if ! kubectl rollout undo deployment/"$DEPLOYMENT" -n "$NAMESPACE" 2>&1 | tee -a "$LOG_FILE"; then
    log "ERROR" "kubectl rollout undo failed"
    exit 1
fi

# --- Wait for Rollback to Complete ---
log "INFO" "Waiting for rollback to complete (timeout: ${ROLLBACK_TIMEOUT}s)..."
if kubectl rollout status deployment/"$DEPLOYMENT" -n "$NAMESPACE" \
    --timeout="${ROLLBACK_TIMEOUT}s" 2>&1 | tee -a "$LOG_FILE"; then
    log "INFO" "Rollback rollout completed"
else
    log "ERROR" "Rollback did not complete within ${ROLLBACK_TIMEOUT}s"
    exit 1
fi

# --- Verify Rollback ---
log "INFO" "Verifying rollback health..."

# Check new image
NEW_IMAGE=$(kubectl get deployment "$DEPLOYMENT" -n "$NAMESPACE" \
    -o jsonpath='{.spec.template.spec.containers[0].image}' 2>/dev/null || echo "unknown")
log "INFO" "Image after rollback: ${NEW_IMAGE}"

if [[ "$NEW_IMAGE" == "$CURRENT_IMAGE" ]]; then
    log "WARN" "Image did not change after rollback -- may already be at the target revision"
fi

# Check pod readiness
READY_PODS=$(kubectl get pods -n "$NAMESPACE" -l "app=${DEPLOYMENT}" \
    -o jsonpath='{range .items[*]}{.status.conditions[?(@.type=="Ready")].status}{"\n"}{end}' 2>/dev/null \
    | grep -c "True" || echo "0")
TOTAL_PODS=$(kubectl get pods -n "$NAMESPACE" -l "app=${DEPLOYMENT}" \
    -o name 2>/dev/null | wc -l)

log "INFO" "Pod readiness after rollback: ${READY_PODS}/${TOTAL_PODS} ready"

if [[ "$READY_PODS" -lt "$TOTAL_PODS" ]]; then
    log "ERROR" "Not all pods are ready after rollback"
    exit 1
fi

# --- Done ---
log "INFO" "Rollback completed successfully"
log "INFO" "  Previous image: ${CURRENT_IMAGE}"
log "INFO" "  Current image:  ${NEW_IMAGE}"
log "INFO" "  Ready pods:     ${READY_PODS}/${TOTAL_PODS}"
log "INFO" "  Log file:       ${LOG_FILE}"

exit 0
```

## change-orchestrator.sh

```bash
#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/config.env"

# --- Argument Parsing ---
NAMESPACE="${DEFAULT_NAMESPACE}"
DEPLOYMENT=""
NEW_IMAGE=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --namespace)  NAMESPACE="$2";  shift 2 ;;
        --deployment) DEPLOYMENT="$2"; shift 2 ;;
        --image)      NEW_IMAGE="$2";  shift 2 ;;
        *) echo "Unknown option: $1"; exit 2 ;;
    esac
done

if [[ -z "$DEPLOYMENT" ]]; then
    echo "ERROR: --deployment is required"
    exit 2
fi
if [[ -z "$NEW_IMAGE" ]]; then
    echo "ERROR: --image is required"
    exit 2
fi

# --- Logging ---
mkdir -p "${LOG_DIR}"
TIMESTAMP=$(date -u '+%Y%m%dT%H%M%SZ')
LOG_FILE="${LOG_DIR}/orchestrator-${DEPLOYMENT}-${TIMESTAMP}.log"

log() {
    local level="$1"
    shift
    local msg="[$(date -u '+%Y-%m-%dT%H:%M:%SZ')] [$level] [orchestrator] $*"
    echo "$msg" | tee -a "$LOG_FILE"
}

# --- Pre-flight Checks ---
log "INFO" "=== Change Orchestrator Started ==="
log "INFO" "Deployment: ${DEPLOYMENT}"
log "INFO" "Namespace:  ${NAMESPACE}"
log "INFO" "New Image:  ${NEW_IMAGE}"

# Verify deployment exists
if ! kubectl get deployment "$DEPLOYMENT" -n "$NAMESPACE" &>/dev/null; then
    log "ERROR" "Deployment '${DEPLOYMENT}' not found in namespace '${NAMESPACE}'"
    exit 1
fi

# Capture current state
PREV_IMAGE=$(kubectl get deployment "$DEPLOYMENT" -n "$NAMESPACE" \
    -o jsonpath='{.spec.template.spec.containers[0].image}')
PREV_REVISION=$(kubectl rollout history deployment/"$DEPLOYMENT" -n "$NAMESPACE" \
    | tail -2 | head -1 | awk '{print $1}')

log "INFO" "Previous image:    ${PREV_IMAGE}"
log "INFO" "Previous revision: ${PREV_REVISION}"

# Check if the image is already deployed (idempotency)
if [[ "$PREV_IMAGE" == "$NEW_IMAGE" ]]; then
    log "INFO" "Image '${NEW_IMAGE}' is already deployed. No change needed."
    exit 0
fi

# --- Deploy ---
log "INFO" "Step 1: Applying new image..."
if ! kubectl set image deployment/"$DEPLOYMENT" \
    "${DEPLOYMENT}=${NEW_IMAGE}" -n "$NAMESPACE" 2>&1 | tee -a "$LOG_FILE"; then
    log "ERROR" "Failed to set image"
    exit 1
fi
log "INFO" "Image update command sent"

# --- Health Check ---
log "INFO" "Step 2: Starting health check monitor..."
if "${SCRIPT_DIR}/health-check.sh" \
    --namespace "$NAMESPACE" \
    --deployment "$DEPLOYMENT" \
    --check-interval "$HEALTH_CHECK_INTERVAL" \
    --check-count "$HEALTH_CHECK_COUNT" \
    --error-threshold "$ERROR_THRESHOLD" 2>&1 | tee -a "$LOG_FILE"; then

    log "INFO" "Step 3: Health check PASSED. Change deployed successfully."
    FINAL_IMAGE=$(kubectl get deployment "$DEPLOYMENT" -n "$NAMESPACE" \
        -o jsonpath='{.spec.template.spec.containers[0].image}')
    log "INFO" "=== Change Complete ==="
    log "INFO" "  Previous: ${PREV_IMAGE}"
    log "INFO" "  Current:  ${FINAL_IMAGE}"
    log "INFO" "  Status:   SUCCESS"
    exit 0
else
    log "ERROR" "Step 3: Health check FAILED. Initiating rollback..."

    # --- Rollback ---
    log "INFO" "Step 4: Executing rollback..."
    if "${SCRIPT_DIR}/rollback-executor.sh" \
        --namespace "$NAMESPACE" \
        --deployment "$DEPLOYMENT" 2>&1 | tee -a "$LOG_FILE"; then

        log "INFO" "Rollback executed. Verifying post-rollback health..."

        # --- Post-rollback Health Check ---
        if "${SCRIPT_DIR}/health-check.sh" \
            --namespace "$NAMESPACE" \
            --deployment "$DEPLOYMENT" \
            --check-interval "$HEALTH_CHECK_INTERVAL" \
            --check-count 3 \
            --error-threshold 2 2>&1 | tee -a "$LOG_FILE"; then

            log "INFO" "Post-rollback health check PASSED."
            log "INFO" "=== Change Complete (ROLLED BACK) ==="
            log "INFO" "  Attempted:  ${NEW_IMAGE}"
            log "INFO" "  Restored:   ${PREV_IMAGE}"
            log "INFO" "  Status:     ROLLED BACK"
            exit 1
        else
            log "ERROR" "CRITICAL: Post-rollback health check FAILED."
            log "ERROR" "MANUAL INTERVENTION REQUIRED."
            log "INFO" "=== Change Complete (ROLLBACK UNHEALTHY) ==="
            log "INFO" "  Attempted:  ${NEW_IMAGE}"
            log "INFO" "  Status:     ROLLBACK FAILED -- ESCALATE"
            exit 2
        fi
    else
        log "ERROR" "CRITICAL: Rollback execution FAILED."
        log "ERROR" "MANUAL INTERVENTION REQUIRED."
        log "INFO" "=== Change Complete (ROLLBACK FAILED) ==="
        log "INFO" "  Attempted:  ${NEW_IMAGE}"
        log "INFO" "  Status:     ROLLBACK FAILED -- ESCALATE"
        exit 2
    fi
fi
```

## test-rollback.md

# Test Procedure for Automated Rollback System

## Prerequisites

- Kubernetes cluster running with `kubectl` configured
- A test deployment already running (e.g., `web-app` in namespace `change-mgmt`)
- Scripts are executable (`chmod +x *.sh`)

## Test 1: Happy Path

**Goal:** Deploy a valid image. Health check passes. No rollback occurs.

**Setup:**
```bash
kubectl create namespace change-mgmt
kubectl create deployment web-app --image=nginx:1.24 -n change-mgmt --replicas=3
kubectl wait --for=condition=available deployment/web-app -n change-mgmt --timeout=60s
```

**Command:**
```bash
./change-orchestrator.sh \
    --namespace change-mgmt \
    --deployment web-app \
    --image nginx:1.25
```

**Expected behavior:**
- Orchestrator captures current image (nginx:1.24)
- Image is updated to nginx:1.25
- Health check runs and passes after `CHECK_COUNT` (6) consecutive successes
- Script exits with code 0

**What to look for in logs:**
```
[INFO] [orchestrator] Previous image: nginx:1.24
[INFO] [orchestrator] Image update command sent
[INFO] [web-app] Health check PASSED (1/6)
...
[INFO] [web-app] Health check PASSED (6/6)
[INFO] [orchestrator] Step 3: Health check PASSED. Change deployed successfully.
[INFO] [orchestrator] Status: SUCCESS
```

---

## Test 2: Bad Image (Automatic Rollback)

**Goal:** Deploy a non-existent image. Health check fails. Automatic rollback triggers.

**Command:**
```bash
./change-orchestrator.sh \
    --namespace change-mgmt \
    --deployment web-app \
    --image nginx:does-not-exist-999
```

**Expected behavior:**
- Orchestrator captures current image (nginx:1.25)
- Image is updated to nginx:does-not-exist-999
- New pods enter `ImagePullBackOff` state
- Health check detects pods not Ready
- After `ERROR_THRESHOLD` (3) consecutive failures, health check exits with code 1
- Rollback executor runs `kubectl rollout undo`
- Image reverts to nginx:1.25
- Post-rollback health check passes
- Script exits with code 1

**What to look for in logs:**
```
[INFO] [orchestrator] Image update command sent
[WARN] [web-app] Readiness check: 0/3 pods ready
[ERROR] [web-app] Health check FAILED (consecutive failures: 1/3)
[ERROR] [web-app] Health check FAILED (consecutive failures: 2/3)
[ERROR] [web-app] Health check FAILED (consecutive failures: 3/3)
[ERROR] [web-app] ERROR THRESHOLD REACHED (3 >= 3). Deployment is unhealthy.
[ERROR] [orchestrator] Step 3: Health check FAILED. Initiating rollback...
[INFO] [rollback:web-app] Executing: kubectl rollout undo deployment/web-app
[INFO] [rollback:web-app] Rollback completed successfully
[INFO] [orchestrator] Status: ROLLED BACK
```

---

## Test 3: Slow Rollout (No False Positive)

**Goal:** Deploy an image that takes time to start. Verify the system does not
false-positive trigger a rollback.

**Setup:** Create a deployment with a long startup time:
```bash
kubectl set image deployment/web-app web-app=nginx:1.25 -n change-mgmt
kubectl wait --for=condition=available deployment/web-app -n change-mgmt
```

**Command:**
```bash
# Deploy an image that starts slowly but eventually becomes healthy
./change-orchestrator.sh \
    --namespace change-mgmt \
    --deployment web-app \
    --image nginx:1.26
```

**Expected behavior:**
- Health check may report some pods not Ready during the rollout
- But the rollout completes before `ERROR_THRESHOLD` consecutive failures
- Health check eventually passes
- No rollback occurs

**What to look for in logs:**
```
[WARN] [web-app] Readiness check: 2/3 pods ready
[WARN] [web-app] Health check FAILED (consecutive failures: 1/3)
[INFO] [web-app] Health check PASSED (1/6)   # rollout completed between checks
...
[INFO] [orchestrator] Status: SUCCESS
```

The key behavior: the system tolerates transient failures during rollout but
only triggers rollback after `ERROR_THRESHOLD` *consecutive* failures.

---

## Test 4: Idempotency

**Goal:** Run the orchestrator when the target image is already deployed.

**Setup:** Ensure nginx:1.26 is already running from Test 3.

**Command:**
```bash
./change-orchestrator.sh \
    --namespace change-mgmt \
    --deployment web-app \
    --image nginx:1.26
```

**Expected behavior:**
- Orchestrator detects that `PREV_IMAGE == NEW_IMAGE`
- Logs that no change is needed
- Exits with code 0 immediately
- No rollout or health check is triggered

**What to look for in logs:**
```
[INFO] [orchestrator] Previous image: nginx:1.26
[INFO] [orchestrator] Image 'nginx:1.26' is already deployed. No change needed.
```

---

## Cleanup

```bash
kubectl delete namespace change-mgmt
rm -rf /var/log/change-management/
```

---

## Design Decisions

### Why consecutive failures?

The health check tracks *consecutive* failures, not total failures. This
prevents false-positive rollbacks during transient issues (e.g., a single pod
restart, a brief network blip). Only sustained failures trigger rollback.

### Why configurable thresholds?

Different deployments have different tolerances:
- A stateless web service can tolerate aggressive thresholds (2-3 failures)
- A stateful service with slow startup needs more lenient thresholds (5-10 failures)
- The config file provides organization-wide defaults; command-line flags
  allow per-deployment overrides

### Why a post-rollback health check?

After rolling back, the system must verify that the rollback itself was
successful. If the rollback also fails (e.g., the previous version also has
an issue), the system must escalate to human intervention rather than
silently leaving the deployment in a broken state.
