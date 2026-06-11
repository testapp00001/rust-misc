# Solution 02: Progressive Load Testing

## Part A: Load Test Stages

```javascript
export const options = {
  stages: [
    { duration: '1m', target: 100 },    // Warmup: 100 VUs for 1 min
    { duration: '3m', target: 100 },    // 1x traffic: 100 VUs for 3 min
    { duration: '3m', target: 200 },    // 2x traffic: 200 VUs for 3 min
    { duration: '3m', target: 500 },    // 5x traffic: 500 VUs for 3 min
    { duration: '3m', target: 1000 },   // 10x traffic: 1000 VUs for 3 min
    { duration: '1m', target: 0 },      // Ramp down: 0 VUs over 1 min
  ],
};
```

**Why this works**: Each stage holds a constant number of VUs for 3 minutes. This gives the system time to stabilize and produces reliable metrics. The warmup phase ensures the system is fully initialized (JIT compilation, connection pool warmup, etc.) before measurement begins.

---

## Part B: Thresholds

```javascript
export const options = {
  // ... stages from Part A ...
  thresholds: {
    'http_req_failed': ['rate<0.05'],           // Error rate < 5%
    'http_req_duration{p(95)}': ['p(95)<2000'], // P95 < 2,000ms
    'http_req_duration{p(99)}': ['p(99)<5000'], // P99 < 5,000ms
  },
};
```

**Why this works**: Thresholds are evaluated at the end of the test. If any threshold is violated, k6 exits with a non-zero code. This makes it easy to integrate into CI/CD pipelines -- a failed load test fails the build.

---

## Part C: Realistic Traffic Mix

```javascript
const BASE_URL = __ENV.TARGET_URL || 'http://localhost:8080';

export default function () {
  const rand = Math.random();
  let res;

  if (rand < 0.60) {
    // 60%: Browse products
    res = http.get(`${BASE_URL}/api/products`, {
      headers: { 'Content-Type': 'application/json' },
    });
  } else if (rand < 0.80) {
    // 20%: View product detail
    const productId = Math.floor(Math.random() * 1000) + 1;
    res = http.get(`${BASE_URL}/api/products/${productId}`, {
      headers: { 'Content-Type': 'application/json' },
    });
  } else if (rand < 0.90) {
    // 10%: Add to cart
    const payload = JSON.stringify({
      product_id: Math.floor(Math.random() * 1000) + 1,
      quantity: Math.floor(Math.random() * 3) + 1,
    });
    res = http.post(`${BASE_URL}/api/cart`, payload, {
      headers: { 'Content-Type': 'application/json' },
    });
  } else {
    // 10%: Checkout
    const payload = JSON.stringify({
      items: [{ product_id: Math.floor(Math.random() * 1000) + 1, quantity: 1 }],
      payment_method: 'credit_card',
    });
    res = http.post(`${BASE_URL}/api/orders`, payload, {
      headers: { 'Content-Type': 'application/json' },
    });
  }

  check(res, {
    'status is 2xx': (r) => r.status >= 200 && r.status < 300,
    'latency < 2s': (r) => r.timings.duration < 2000,
  });

  sleep(Math.random() * 2 + 0.5);  // Random think time: 0.5-2.5s
}
```

**Why this works**: The traffic mix simulates realistic user behavior. Most users browse (60%), some view details (20%), fewer add to cart (10%), and even fewer checkout (10%). Random think time (0.5-2.5 seconds) between requests simulates real user pauses.

---

## Part D: Results Analysis Script

```bash
#!/bin/bash
# analyze-load-test.sh — Run progressive load test and analyze results
set -euo pipefail

TARGET_URL="${TARGET_URL:-http://localhost:8080}"
RESULTS_DIR="./load-test-results/$(date +%Y%m%d-%H%M%S)"
mkdir -p "$RESULTS_DIR"

echo "=== Progressive Load Test Suite ==="
echo "Target: $TARGET_URL"
echo "Results: $RESULTS_DIR"

# Run the load test with JSON export
k6 run \
  --summary-export="$RESULTS_DIR/summary.json" \
  -e TARGET_URL="$TARGET_URL" \
  load-test-scenario.js 2>&1 | tee "$RESULTS_DIR/output.txt"

# Parse results
echo ""
echo "=== Results Analysis ==="
echo ""

if [ ! -f "$RESULTS_DIR/summary.json" ]; then
  echo "ERROR: Summary file not found"
  exit 1
fi

RPS=$(jq -r '.metrics.http_reqs.values.rate' "$RESULTS_DIR/summary.json")
P95=$(jq -r '.metrics.http_req_duration.values["p(95)"]' "$RESULTS_DIR/summary.json")
P99=$(jq -r '.metrics.http_req_duration.values["p(99)"]' "$RESULTS_DIR/summary.json")
ERROR_RATE=$(jq -r '.metrics.http_req_failed.values.rate' "$RESULTS_DIR/summary.json")

printf "%-20s %s\n" "Requests/sec:" "$RPS"
printf "%-20s %s ms\n" "P95 Latency:" "$P95"
printf "%-20s %s ms\n" "P99 Latency:" "$P99"
printf "%-20s %s%%\n" "Error Rate:" "$(echo "$ERROR_RATE * 100" | bc)"

# Determine sustainable capacity
echo ""
echo "=== Capacity Assessment ==="

# Check if thresholds were met
if (( $(echo "$ERROR_RATE < 0.05" | bc -l) )) && (( $(echo "$P95 < 2000" | bc -l) )); then
  echo "VERDICT: System PASSED at 10x traffic ($RPS RPS)"
  echo "Sustainable capacity: $RPS RPS"
else
  echo "VERDICT: System FAILED at 10x traffic"
  echo "Error rate: $ERROR_RATE (threshold: 0.05)"
  echo "P95 latency: $P95 ms (threshold: 2000 ms)"
  echo ""
  echo "ACTION: Re-run at lower traffic levels to find breaking point"
fi

echo ""
echo "Results saved to: $RESULTS_DIR"
```

**Why this works**: The script runs k6 with `--summary-export` to produce a machine-readable JSON file. It then uses `jq` to extract key metrics and compares them against thresholds. The `bc` command handles floating-point comparison in bash.

---

## Common Mistakes
1. **Testing for too short a duration**: A 30-second test does not give the system time to stabilize. Use at least 3 minutes per traffic level.
2. **Not warming up**: The first requests are slow (cold caches, JIT compilation). A warmup phase prevents these from skewing results.
3. **Using a constant think time**: Real users pause for varying amounts of time. Use `Math.random()` to vary the sleep duration.
4. **Not exporting results**: Without JSON export, you cannot automate analysis. Always use `--summary-export`.
5. **Testing only GET requests**: Production traffic includes POST, PUT, DELETE. Include write operations in your traffic mix.

## Relevant README Sections
- [Load Testing to Find Breaking Points](../README.md#step-2-load-testing-to-find-breaking-points)
- [Hands-On Lab](../README.md#hands-on-lab-capacity-planning-for-a-product-launch)
