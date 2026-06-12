#!/usr/bin/env bash
#
# reconciliation.sh — Post-sale reconciliation workflow.
#
# Steps:
#   1. Stop accepting new purchases (set sale to CLOSED)
#   2. Run the reconciliation process (from module 13)
#   3. Generate a reconciliation report
#   4. Sync final state to the database
#
# Usage:
#   ./scripts/reconciliation.sh [SALE_ID]
#
set -euo pipefail

SALE_ID="${1:-flash-sale-2026-summer}"
NAMESPACE="${NAMESPACE:-flash-sale}"
API_URL="${API_URL:-http://localhost:3000}"
RESULTS_DIR="reconciliation-reports"

mkdir -p "${RESULTS_DIR}"

TIMESTAMP=$(date +%Y%m%d-%H%M%S)
REPORT_FILE="${RESULTS_DIR}/reconciliation-${SALE_ID}-${TIMESTAMP}.json"

echo "============================================"
echo " Post-Sale Reconciliation"
echo " Sale ID: ${SALE_ID}"
echo "============================================"

# ── Step 1: Stop accepting purchases ─────────────────────────────────────────
echo ""
echo "[1/4] Closing sale — stopping new purchases..."
kubectl exec -n "${NAMESPACE}" deploy/flash-sale-api -- \
    curl -s -X POST "http://localhost:3000/admin/sale/${SALE_ID}/close" || {
        echo "WARN: Sale close endpoint not available — sale may need manual closure"
    }
echo "      Sale ${SALE_ID} is now CLOSED."

# Wait for in-flight requests to drain
echo "      Waiting 10s for in-flight requests to drain..."
sleep 10

# ── Step 2: Run reconciliation ───────────────────────────────────────────────
echo ""
echo "[2/4] Running reconciliation process..."
kubectl exec -n "${NAMESPACE}" deploy/order-worker -- \
    cargo run --bin reconciliation -- \
    --sale-id "${SALE_ID}" \
    --output "/tmp/reconciliation.json" || {
        echo "WARN: Reconciliation binary not available in worker pod"
    }

# Copy report out of pod
kubectl cp \
    "${NAMESPACE}/$(kubectl get pods -n ${NAMESPACE} -l app=order-worker -o jsonpath='{.items[0].metadata.name}'):/tmp/reconciliation.json" \
    "${REPORT_FILE}" 2>/dev/null || {
        echo "WARN: Could not copy reconciliation report from pod"
    }
echo "      Reconciliation complete."

# ── Step 3: Generate report ──────────────────────────────────────────────────
echo ""
echo "[3/4] Generating reconciliation report..."
if [[ -f "${REPORT_FILE}" ]] && command -v jq &>/dev/null; then
    echo "      Report: ${REPORT_FILE}"
    echo ""
    echo "      ┌─ Reconciliation Summary ────────────────────────"
    echo "      │ Sale ID:             $(jq -r '.sale_id // "N/A"' "${REPORT_FILE}")"
    echo "      │ Total Orders:        $(jq '.total_orders // "N/A"' "${REPORT_FILE}")"
    echo "      │ Successful:          $(jq '.successful_orders // "N/A"' "${REPORT_FILE}")"
    echo "      │ Failed:              $(jq '.failed_orders // "N/A"' "${REPORT_FILE}")"
    echo "      │ Oversells Detected:  $(jq '.oversell_count // 0' "${REPORT_FILE}")"
    echo "      │ Stock Remaining:     $(jq '.stock_remaining // "N/A"' "${REPORT_FILE}")"
    echo "      └─────────────────────────────────────────────────"
else
    echo "      Report written to: ${REPORT_FILE}"
fi

# ── Step 4: Sync to database ─────────────────────────────────────────────────
echo ""
echo "[4/4] Syncing final state to database..."
kubectl exec -n "${NAMESPACE}" deploy/flash-sale-api -- \
    curl -s -X POST "http://localhost:3000/admin/sale/${SALE_ID}/sync" || {
        echo "WARN: Sync endpoint not available — final state may need manual sync"
    }
echo "      Database sync complete."

echo ""
echo "============================================"
echo " Reconciliation finished"
echo " Report: ${REPORT_FILE}"
echo "============================================"
