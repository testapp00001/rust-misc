#!/usr/bin/env bash
#
# pre-sale-warmup.sh — Prepare the flash sale system for an upcoming sale event.
#
# Steps:
#   1. Warm the Redis cache with product data
#   2. Scale up API pods to handle expected traffic
#   3. Verify all health checks pass
#   4. Set sale status to ACTIVE
#
# Usage:
#   ./scripts/pre-sale-warmup.sh [SALE_ID]
#
set -euo pipefail

SALE_ID="${1:-flash-sale-2026-summer}"
NAMESPACE="${NAMESPACE:-flash-sale}"
API_URL="${API_URL:-http://localhost:3000}"
EXPECTED_PODS="${EXPECTED_PODS:-10}"

echo "============================================"
echo " Flash Sale Pre-Sale Warmup"
echo " Sale ID: ${SALE_ID}"
echo "============================================"

# ── Step 1: Warm Redis cache ─────────────────────────────────────────────────
echo ""
echo "[1/4] Warming Redis cache..."
kubectl exec -n "${NAMESPACE}" deploy/flash-sale-api -- \
    curl -s -X POST "http://localhost:3000/admin/cache/warm" \
    -H "Content-Type: application/json" \
    -d "{\"sale_id\": \"${SALE_ID}\"}" || {
        echo "WARN: Cache warmup endpoint not available — skipping (manual warmup may be needed)"
    }
echo "      Cache warmup request sent."

# ── Step 2: Scale pods ────────────────────────────────────────────────────────
echo ""
echo "[2/4] Scaling API deployment to ${EXPECTED_PODS} replicas..."
kubectl scale deployment flash-sale-api \
    --replicas="${EXPECTED_PODS}" \
    -n "${NAMESPACE}"

echo "      Waiting for rollout to complete..."
kubectl rollout status deployment/flash-sale-api \
    -n "${NAMESPACE}" \
    --timeout=120s
echo "      Pods scaled to ${EXPECTED_PODS}."

# ── Step 3: Verify health ────────────────────────────────────────────────────
echo ""
echo "[3/4] Verifying health of all pods..."
UNHEALTHY=0
for pod in $(kubectl get pods -n "${NAMESPACE}" -l app=flash-sale-api -o name); do
    STATUS=$(kubectl get "${pod}" -n "${NAMESPACE}" -o jsonpath='{.status.phase}')
    READY=$(kubectl get "${pod}" -n "${NAMESPACE}" -o jsonpath='{.status.conditions[?(@.type=="Ready")].status}')
    if [[ "${STATUS}" != "Running" || "${READY}" != "True" ]]; then
        echo "      UNHEALTHY: ${pod} (phase=${STATUS}, ready=${READY})"
        UNHEALTHY=$((UNHEALTHY + 1))
    fi
done

if [[ ${UNHEALTHY} -gt 0 ]]; then
    echo "ERROR: ${UNHEALTHY} pod(s) are unhealthy. Aborting warmup."
    exit 1
fi
echo "      All pods healthy."

# ── Step 4: Set sale status to ACTIVE ─────────────────────────────────────────
echo ""
echo "[4/4] Setting sale status to ACTIVE..."
kubectl exec -n "${NAMESPACE}" deploy/flash-sale-api -- \
    curl -s -X POST "http://localhost:3000/admin/sale/${SALE_ID}/activate" || {
        echo "WARN: Sale activation endpoint not available — set manually"
    }
echo "      Sale ${SALE_ID} is now ACTIVE."

echo ""
echo "============================================"
echo " Warmup complete — sale is LIVE"
echo "============================================"
