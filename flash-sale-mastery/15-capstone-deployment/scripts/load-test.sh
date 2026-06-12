#!/usr/bin/env bash
#
# load-test.sh — Run load test scenarios against the flash sale API.
#
# Scenarios:
#   ramp-up   : Gradually increase load from 0 to target RPS
#   spike      : Sudden burst of traffic (simulates viral moment)
#   sustained  : Constant high load for a duration
#
# Usage:
#   ./scripts/load-test.sh [scenario] [target_rps] [duration]
#
# Examples:
#   ./scripts/load-test.sh ramp-up 5000 60
#   ./scripts/load-test.sh spike 10000 10
#   ./scripts/load-test.sh sustained 3000 120
#
set -euo pipefail

SCENARIO="${1:-ramp-up}"
TARGET_RPS="${2:-5000}"
DURATION="${3:-60}"
API_URL="${API_URL:-http://localhost:3000}"
RESULTS_DIR="load-test-results"

mkdir -p "${RESULTS_DIR}"

TIMESTAMP=$(date +%Y%m%d-%H%M%S)
REPORT_FILE="${RESULTS_DIR}/${SCENARIO}-${TIMESTAMP}.json"

echo "============================================"
echo " Flash Sale Load Test"
echo " Scenario:   ${SCENARIO}"
echo " Target RPS: ${TARGET_RPS}"
echo " Duration:   ${DURATION}s"
echo " API URL:    ${API_URL}"
echo " Report:     ${REPORT_FILE}"
echo "============================================"

case "${SCENARIO}" in
    ramp-up)
        echo ""
        echo "Ramping up from 0 to ${TARGET_RPS} RPS over ${DURATION}s..."
        cargo run --package load-testing -- \
            --url "${API_URL}/api/v1/purchase" \
            --scenario ramp-up \
            --target-rps "${TARGET_RPS}" \
            --ramp-duration "${DURATION}" \
            --total-duration "$((DURATION + 30))" \
            --output "${REPORT_FILE}"
        ;;

    spike)
        echo ""
        echo "Launching spike of ${TARGET_RPS} RPS for ${DURATION}s..."
        cargo run --package load-testing -- \
            --url "${API_URL}/api/v1/purchase" \
            --scenario spike \
            --target-rps "${TARGET_RPS}" \
            --duration "${DURATION}" \
            --output "${REPORT_FILE}"
        ;;

    sustained)
        echo ""
        echo "Running sustained ${TARGET_RPS} RPS for ${DURATION}s..."
        cargo run --package load-testing -- \
            --url "${API_URL}/api/v1/purchase" \
            --scenario sustained \
            --target-rps "${TARGET_RPS}" \
            --duration "${DURATION}" \
            --output "${REPORT_FILE}"
        ;;

    *)
        echo "ERROR: Unknown scenario '${SCENARIO}'"
        echo "Available scenarios: ramp-up, spike, sustained"
        exit 1
        ;;
esac

echo ""
echo "============================================"
echo " Load test complete"
echo " Results: ${REPORT_FILE}"
echo "============================================"

# Print summary if jq is available
if command -v jq &>/dev/null && [[ -f "${REPORT_FILE}" ]]; then
    echo ""
    echo "Summary:"
    echo "  Total requests:   $(jq '.total_requests // "N/A"' "${REPORT_FILE}")"
    echo "  Success rate:     $(jq '.success_rate // "N/A"' "${REPORT_FILE}")%"
    echo "  p50 latency:      $(jq '.latency_p50_ms // "N/A"' "${REPORT_FILE}")ms"
    echo "  p95 latency:      $(jq '.latency_p95_ms // "N/A"' "${REPORT_FILE}")ms"
    echo "  p99 latency:      $(jq '.latency_p99_ms // "N/A"' "${REPORT_FILE}")ms"
fi
