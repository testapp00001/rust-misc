# Module 69: Capacity Planning — Forecasting, Load Testing, Bottleneck Analysis

> **Previous Module:** [68 - Traffic Surge Handling](../68-traffic-surge-handling/README.md)
> **Next Module:** [70 - Caching Strategies](../70-caching-strategies/README.md)

## The Problem

Your company announces a product launch next month. Marketing expects 10x normal traffic. The CEO asks: "Will our infrastructure hold?" Nobody knows. The team guesses, provisions double the current capacity, and hopes for the best. Launch day arrives. Traffic hits 15x. The system crashes. Post-mortem reveals the database was the bottleneck all along — adding more application servers did nothing.

The fundamental issue: **guessing is not planning**. Without load testing, forecasting, and bottleneck analysis, capacity decisions are gambling.

## The Naive Way

```bash
# "Just double everything"
kubectl scale deployment web --replicas=40
kubectl scale deployment api --replicas=30
kubectl scale deployment worker --replicas=20

# Or: "Throw money at it — use the biggest instances"
aws ec2 run-instances --instance-type m5.24xlarge --count 20
```

**Why this fails:**
- You do not know which tier is the bottleneck (probably the database, not the app servers)
- Doubling app servers when the database is saturated makes things worse (more connection contention)
- Cost explodes without solving the problem
- No data to justify the spend to finance
- No way to validate until the actual event (too late)

## The Right Way

### Step 1: Establish Baselines

Before you can plan, you must know your current capacity.

```python
# baseline_collector.py — Collect baseline metrics for capacity planning
import time
import json
import requests
from datetime import datetime
from dataclasses import dataclass
from typing import List

@dataclass
class BaselineMetrics:
    timestamp: str
    requests_per_second: float
    p50_latency_ms: float
    p95_latency_ms: float
    p99_latency_ms: float
    error_rate: float
    cpu_utilization: float
    memory_utilization: float
    db_connections: int
    db_query_time_ms: float
    queue_depth: int

class BaselineCollector:
    """Collect and analyze baseline metrics over time."""

    def __init__(self, prometheus_url: str):
        self.prometheus_url = prometheus_url
        self.baselines: List[BaselineMetrics] = []

    def query_prometheus(self, query: str):
        """Query Prometheus for metrics."""
        response = requests.get(f"{self.prometheus_url}/api/v1/query", params={
            "query": query,
            "time": datetime.now().isoformat()
        })
        data = response.json()
        if data["status"] == "success" and data["data"]["result"]:
            return float(data["data"]["result"][0]["value"][1])
        return 0.0

    def collect_baseline(self) -> BaselineMetrics:
        """Collect current baseline metrics."""
        metrics = BaselineMetrics(
            timestamp=datetime.now().isoformat(),
            requests_per_second=self.query_prometheus(
                'sum(rate(http_requests_total[5m]))'
            ),
            p50_latency_ms=self.query_prometheus(
                'histogram_quantile(0.50, rate(http_request_duration_seconds_bucket[5m])) * 1000'
            ),
            p95_latency_ms=self.query_prometheus(
                'histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) * 1000'
            ),
            p99_latency_ms=self.query_prometheus(
                'histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m])) * 1000'
            ),
            error_rate=self.query_prometheus(
                'sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m]))'
            ),
            cpu_utilization=self.query_prometheus(
                'avg(rate(container_cpu_usage_seconds_total[5m])) * 100'
            ),
            memory_utilization=self.query_prometheus(
                'avg(container_memory_working_set_bytes / container_spec_memory_limit_bytes) * 100'
            ),
            db_connections=self.query_prometheus('pg_stat_activity_count'),
            db_query_time_ms=self.query_prometheus('avg(pg_stat_activity_query_duration_ms)'),
            queue_depth=self.query_prometheus('rabbitmq_queue_messages'),
        )
        self.baselines.append(metrics)
        return metrics

    def collect_over_period(self, hours: int = 168, interval_seconds: int = 300):
        """Collect baselines over a period (default 1 week, every 5 min)."""
        total_samples = (hours * 3600) // interval_seconds
        print(f"Collecting {total_samples} samples over {hours} hours...")
        for i in range(total_samples):
            metrics = self.collect_baseline()
            print(f"Sample {i+1}/{total_samples}: {metrics.requests_per_second:.0f} RPS, "
                  f"p95={metrics.p95_latency_ms:.1f}ms, CPU={metrics.cpu_utilization:.1f}%")
            time.sleep(interval_seconds)
        return self.analyze_baselines()

    def analyze_baselines(self) -> dict:
        """Analyze collected baselines to determine capacity limits."""
        if not self.baselines:
            return {}
        rps_values = [m.requests_per_second for m in self.baselines]
        cpu_values = [m.cpu_utilization for m in self.baselines]
        p95_values = [m.p95_latency_ms for m in self.baselines]
        avg_rps = sum(rps_values) / len(rps_values)
        peak_cpu = max(cpu_values)
        return {
            "avg_rps": avg_rps,
            "peak_rps": max(rps_values),
            "p95_rps": sorted(rps_values)[int(len(rps_values) * 0.95)],
            "avg_cpu": sum(cpu_values) / len(cpu_values),
            "peak_cpu": peak_cpu,
            "avg_p95_latency": sum(p95_values) / len(p95_values),
            "samples_collected": len(self.baselines),
            "recommendation": self._generate_recommendation(rps_values, cpu_values),
        }

    def _generate_recommendation(self, rps, cpu):
        peak_cpu = max(cpu)
        avg_rps = sum(rps) / len(rps)
        if peak_cpu > 80:
            headroom = (100 - peak_cpu) / peak_cpu
            return {
                "status": "WARNING",
                "message": f"Peak CPU {peak_cpu:.1f}% — limited headroom",
                "estimated_max_rps": round(avg_rps * (1 + headroom)),
                "action": "Scale up before next traffic event"
            }
        elif peak_cpu > 60:
            return {
                "status": "OK",
                "message": f"Peak CPU {peak_cpu:.1f}% — moderate headroom",
                "estimated_max_rps": round(avg_rps * 2),
                "action": "Monitor and plan scaling for 2x traffic"
            }
        else:
            return {
                "status": "GOOD",
                "message": f"Peak CPU {peak_cpu:.1f}% — ample headroom",
                "estimated_max_rps": round(avg_rps * 3),
                "action": "Current capacity supports 3x traffic"
            }
```

### Step 2: Load Testing to Find Breaking Points

```bash
#!/bin/bash
# load-test-suite.sh — Progressive load testing to find system limits
set -euo pipefail

TARGET_URL="${TARGET_URL:-http://localhost:8080}"
RESULTS_DIR="./load-test-results/$(date +%Y%m%d-%H%M%S)"
mkdir -p "$RESULTS_DIR"

echo "=== Capacity Planning Load Test Suite ==="
echo "Target: $TARGET_URL"
echo "Results: $RESULTS_DIR"

for multiplier in 1 2 5 10; do
  vus=$((50 * multiplier))
  echo ""
  echo "--- Test: ${multiplier}x traffic ($vus VUs) ---"
  k6 run --vus $vus --duration 5m \
    --summary-export="$RESULTS_DIR/${multiplier}x-summary.json" \
    -e TARGET_URL="$TARGET_URL" \
    load-test-scenario.js
done

# Breaking point test: ramp until failure
echo ""
echo "--- Test: Breaking Point (ramp to 2000 VUs) ---"
k6 run --stage 2m:200 --stage 3m:500 --stage 3m:1000 --stage 2m:2000 \
  --summary-export="$RESULTS_DIR/breaking-point-summary.json" \
  -e TARGET_URL="$TARGET_URL" \
  load-test-scenario.js

echo ""
echo "=== Load test suite complete ==="
echo "Analyze results in: $RESULTS_DIR"
```

```javascript
// load-test-scenario.js — k6 load test scenario for capacity planning
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

const errorRate = new Rate('errors');
const latencyTrend = new Trend('request_latency');

export const options = {
  thresholds: {
    'errors': ['rate<0.05'],
    'http_req_duration': ['p(95)<5000'],
  },
};

const BASE_URL = __ENV.TARGET_URL || 'http://localhost:8080';

const ENDPOINTS = [
  { path: '/', weight: 30, method: 'GET' },
  { path: '/api/products', weight: 25, method: 'GET' },
  { path: '/api/products/1', weight: 15, method: 'GET' },
  { path: '/api/cart', weight: 10, method: 'GET' },
  { path: '/api/cart', weight: 8, method: 'POST', body: { product_id: 1, quantity: 1 } },
  { path: '/api/orders', weight: 5, method: 'POST', body: { items: [{ product_id: 1 }] } },
  { path: '/api/search', weight: 7, method: 'GET', params: { q: 'test' } },
];

function selectEndpoint() {
  const rand = Math.random() * 100;
  let cumulative = 0;
  for (const ep of ENDPOINTS) {
    cumulative += ep.weight;
    if (rand <= cumulative) return ep;
  }
  return ENDPOINTS[0];
}

export default function () {
  const endpoint = selectEndpoint();
  const url = endpoint.params
    ? `${BASE_URL}${endpoint.path}?${new URLSearchParams(endpoint.params)}`
    : `${BASE_URL}${endpoint.path}`;

  let res;
  const params = { headers: { 'Content-Type': 'application/json' } };
  if (endpoint.method === 'POST') {
    res = http.post(url, JSON.stringify(endpoint.body || {}), params);
  } else {
    res = http.get(url, params);
  }

  const success = check(res, {
    'status is 2xx': (r) => r.status >= 200 && r.status < 300,
    'latency < 2s': (r) => r.timings.duration < 2000,
  });

  errorRate.add(!success);
  latencyTrend.add(res.timings.duration);
  sleep(Math.random() * 2 + 0.5);
}
```

### Step 3: Bottleneck Analysis

```python
# bottleneck_analyzer.py — Identify the system bottleneck from load test results
from dataclasses import dataclass

@dataclass
class BottleneckResult:
    component: str
    utilization: float
    severity: str  # LOW, MEDIUM, HIGH, CRITICAL
    recommendation: str

class BottleneckAnalyzer:
    """Analyze load test results to identify the system bottleneck."""

    def __init__(self, prometheus_url: str):
        self.prometheus_url = prometheus_url

    def analyze(self) -> list:
        """Run full bottleneck analysis during a load test."""
        results = [
            self._check_cpu(),
            self._check_memory(),
            self._check_database(),
            self._check_network(),
            self._check_disk_io(),
            self._check_connections(),
        ]
        severity_order = {"CRITICAL": 0, "HIGH": 1, "MEDIUM": 2, "LOW": 3}
        results.sort(key=lambda r: severity_order.get(r.severity, 4))
        return results

    def _check_cpu(self) -> BottleneckResult:
        utilization = 78.5  # Query Prometheus in production
        if utilization > 90:
            return BottleneckResult("CPU", utilization, "CRITICAL",
                "CPU is the bottleneck. Scale horizontally or vertically.")
        elif utilization > 75:
            return BottleneckResult("CPU", utilization, "HIGH",
                "CPU approaching limit. Plan to scale before next traffic increase.")
        elif utilization > 50:
            return BottleneckResult("CPU", utilization, "MEDIUM",
                "CPU moderate. Monitor during traffic events.")
        return BottleneckResult("CPU", utilization, "LOW", "CPU has ample headroom.")

    def _check_memory(self) -> BottleneckResult:
        utilization = 65.2
        if utilization > 90:
            return BottleneckResult("Memory", utilization, "CRITICAL",
                "Memory critical. Risk of OOM kills. Increase limits or optimize allocation.")
        elif utilization > 75:
            return BottleneckResult("Memory", utilization, "HIGH",
                "Memory high. Check for leaks and plan to increase limits.")
        return BottleneckResult("Memory", utilization, "LOW", "Memory has sufficient headroom.")

    def _check_database(self) -> BottleneckResult:
        connection_pct = 82.0
        if connection_pct > 90:
            return BottleneckResult("Database Connections", connection_pct, "CRITICAL",
                "Database connections nearly exhausted. Add connection pooling or read replicas.")
        elif connection_pct > 70:
            return BottleneckResult("Database Connections", connection_pct, "HIGH",
                "Database connections high. Implement connection pooling if not already.")
        return BottleneckResult("Database Connections", connection_pct, "LOW",
            "Database connections healthy.")

    def _check_network(self) -> BottleneckResult:
        bandwidth_pct = 45.0
        if bandwidth_pct > 80:
            return BottleneckResult("Network", bandwidth_pct, "HIGH",
                "Network bandwidth saturated. Consider larger instances or compression.")
        return BottleneckResult("Network", bandwidth_pct, "LOW", "Network bandwidth sufficient.")

    def _check_disk_io(self) -> BottleneckResult:
        io_util_pct = 55.0
        if io_util_pct > 80:
            return BottleneckResult("Disk I/O", io_util_pct, "HIGH",
                "Disk I/O saturated. Move to SSD, add read replicas, or implement caching.")
        return BottleneckResult("Disk I/O", io_util_pct, "LOW", "Disk I/O within normal range.")

    def _check_connections(self) -> BottleneckResult:
        connection_pct = 40.0
        if connection_pct > 80:
            return BottleneckResult("Connection Pools", connection_pct, "HIGH",
                "Connection pool near limit. Increase pool size or add multiplexing.")
        return BottleneckResult("Connection Pools", connection_pct, "LOW",
            "Connection pools healthy.")

    def generate_report(self, results: list) -> str:
        bottleneck = results[0]
        lines = [
            "=" * 60, "BOTTLENECK ANALYSIS REPORT", "=" * 60,
            f"\nPRIMARY BOTTLENECK: {bottleneck.component}",
            f"  Utilization: {bottleneck.utilization:.1f}%",
            f"  Severity: {bottleneck.severity}",
            f"  Recommendation: {bottleneck.recommendation}",
            "\nALL COMPONENTS:", "-" * 60,
        ]
        for r in results:
            lines.append(f"  {r.component:<25} {r.utilization:>6.1f}%  {r.severity:<10}")
        lines.append("\nCAPACITY HEADROOM:" + "\n" + "-" * 60)
        for r in results:
            headroom = max(0, 100 - r.utilization)
            multiplier = (100 / r.utilization) if r.utilization > 0 else float('inf')
            lines.append(f"  {r.component:<25} {headroom:>6.1f}% free  (~{multiplier:.1f}x current)")
        return "\n".join(lines)
```

### Step 4: Growth Forecasting

```python
# capacity_forecaster.py — Forecast when capacity will be exhausted
import math
from dataclasses import dataclass
from typing import List
from datetime import datetime, timedelta

@dataclass
class GrowthDataPoint:
    date: datetime
    metric_value: float

@dataclass
class ForecastResult:
    current_value: float
    current_capacity: float
    utilization_today: float
    projected_exhaustion_date: datetime
    days_until_exhaustion: int
    recommended_action_date: datetime
    recommended_capacity: float

class CapacityForecaster:
    """Forecast when capacity will be exhausted based on growth trends."""

    def __init__(self, current_capacity: float):
        self.current_capacity = current_capacity

    def forecast_linear(self, historical_data: List[GrowthDataPoint]) -> ForecastResult:
        """Linear growth forecast — simple but conservative."""
        if len(historical_data) < 2:
            raise ValueError("Need at least 2 data points for forecasting")
        first, last = historical_data[0], historical_data[-1]
        days = max((last.date - first.date).days, 1)
        daily_growth = (last.metric_value - first.metric_value) / days

        headroom = self.current_capacity - last.metric_value
        days_to_exhaustion = headroom / daily_growth if daily_growth > 0 else 9999
        exhaustion_date = last.date + timedelta(days=days_to_exhaustion)
        action_date = exhaustion_date - timedelta(days=30)
        recommended = last.metric_value + (daily_growth * 180)

        return ForecastResult(
            current_value=last.metric_value,
            current_capacity=self.current_capacity,
            utilization_today=(last.metric_value / self.current_capacity) * 100,
            projected_exhaustion_date=exhaustion_date,
            days_until_exhaustion=int(days_to_exhaustion),
            recommended_action_date=max(action_date, datetime.now()),
            recommended_capacity=recommended * 1.2,
        )

    def forecast_exponential(self, historical_data: List[GrowthDataPoint]) -> ForecastResult:
        """Exponential growth forecast — for rapidly growing systems."""
        if len(historical_data) < 3:
            return self.forecast_linear(historical_data)

        n = len(historical_data)
        first_third = historical_data[:n//3]
        last_third = historical_data[2*n//3:]

        avg_first = sum(d.metric_value for d in first_third) / len(first_third)
        avg_last = sum(d.metric_value for d in last_third) / len(last_third)
        days_span = (last_third[-1].date - first_third[0].date).days

        if days_span == 0 or avg_first <= 0:
            return self.forecast_linear(historical_data)

        daily_growth_rate = math.log(avg_last / avg_first) / days_span
        current = historical_data[-1].metric_value
        current_date = historical_data[-1].date

        if daily_growth_rate <= 0:
            days_to_exhaustion = 9999
        else:
            days_to_exhaustion = math.log(self.current_capacity / current) / daily_growth_rate

        exhaustion_date = current_date + timedelta(days=days_to_exhaustion)
        action_date = exhaustion_date - timedelta(days=60)
        recommended = current * math.exp(daily_growth_rate * 180)

        return ForecastResult(
            current_value=current,
            current_capacity=self.current_capacity,
            utilization_today=(current / self.current_capacity) * 100,
            projected_exhaustion_date=exhaustion_date,
            days_until_exhaustion=int(days_to_exhaustion),
            recommended_action_date=max(action_date, datetime.now()),
            recommended_capacity=recommended * 1.3,
        )

    def generate_capacity_plan(self, forecast: ForecastResult) -> str:
        lines = [
            "=" * 60, "CAPACITY PLAN", "=" * 60,
            f"Current Usage:     {forecast.current_value:,.0f}",
            f"Current Capacity:  {forecast.current_capacity:,.0f}",
            f"Utilization Today: {forecast.utilization_today:.1f}%",
            "", "PROJECTION:",
            f"  Exhaustion Date:   {forecast.projected_exhaustion_date.strftime('%Y-%m-%d')}",
            f"  Days Remaining:    {forecast.days_until_exhaustion}",
            "", "RECOMMENDATION:",
            f"  Action By:         {forecast.recommended_action_date.strftime('%Y-%m-%d')}",
            f"  Target Capacity:   {forecast.recommended_capacity:,.0f}", "",
        ]
        if forecast.days_until_exhaustion < 30:
            lines.append("  *** URGENT: Capacity exhaustion within 30 days ***")
        elif forecast.days_until_exhaustion < 90:
            lines.append("  ** WARNING: Capacity exhaustion within 90 days **")
        else:
            lines.append("  Status: Capacity plan on track")
        return "\n".join(lines)
```

## The Production Way

### Integrated Capacity Planning Pipeline

```yaml
# capacity-planning-pipeline.yaml — Automated weekly capacity planning
apiVersion: tekton.dev/v1beta1
kind: Pipeline
metadata:
  name: capacity-planning
  namespace: ci
spec:
  params:
    - name: target-environment
      type: string
    - name: load-multiplier
      type: string
      default: "5"
  tasks:
    - name: load-test
      taskRef:
        name: k6-load-test
      params:
        - name: target-url
          value: "https://staging.example.com"
        - name: vus
          value: "500"
        - name: duration
          value: "30m"

    - name: collect-metrics
      taskRef:
        name: prometheus-collector
      runAfter: ["load-test"]
      params:
        - name: prometheus-url
          value: "http://prometheus.monitoring:9090"
        - name: duration
          value: "30m"

    - name: analyze
      taskRef:
        name: bottleneck-analyzer
      runAfter: ["collect-metrics"]

    - name: forecast
      taskRef:
        name: capacity-forecaster
      runAfter: ["analyze"]
      params:
        - name: historical-data
          value: "s3://capacity-reports/history/"

    - name: report
      taskRef:
        name: report-generator
      runAfter: ["forecast"]
      params:
        - name: slack-webhook
          valueFrom:
            secretKeyRef:
              name: slack-webhook
              key: url
```

### Automated Scaling Policies Based on Capacity Data

```yaml
# scaling-policies.yaml — Data-driven scaling based on capacity analysis
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: web-hpa
  namespace: production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: web
  minReplicas: 5
  maxReplicas: 50
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 30
      policies:
        - type: Percent
          value: 100
          periodSeconds: 30
      selectPolicy: Max
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
        - type: Percent
          value: 10
          periodSeconds: 60
  metrics:
    - type: Pods
      pods:
        metric:
          name: http_requests_per_second
        target:
          type: AverageValue
          averageValue: "1000"
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
---
# VPA for right-sizing recommendations
apiVersion: autoscaling.k8s.io/v1
kind: VerticalPodAutoscaler
metadata:
  name: web-vpa
  namespace: production
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: web
  updatePolicy:
    updateMode: "Off"  # Recommend only, do not auto-apply
  resourcePolicy:
    containerPolicies:
      - containerName: web
        minAllowed:
          cpu: 100m
          memory: 128Mi
        maxAllowed:
          cpu: 4
          memory: 8Gi
```

## Hands-On Lab: Capacity Planning for a Product Launch

### Scenario

You run a SaaS platform. Normal traffic: 2,000 RPS. A major product launch is in 2 weeks. Expected traffic: 20,000 RPS. Perform capacity planning to determine if your system can handle the load.

### Step 1: Establish Current Baseline

```bash
# Deploy a sample application
kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web
spec:
  replicas: 3
  selector:
    matchLabels:
      app: web
  template:
    metadata:
      labels:
        app: web
    spec:
      containers:
        - name: web
          image: nginx:alpine
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 500m
              memory: 256Mi
---
apiVersion: v1
kind: Service
metadata:
  name: web
spec:
  selector:
    app: web
  ports:
    - port: 80
  type: LoadBalancer
EOF
```

### Step 2: Run Progressive Load Tests

```bash
# Run load tests at increasing traffic levels
for multiplier in 1 2 5 10; do
  vus=$((50 * multiplier))
  echo ""
  echo "=== Testing at ${multiplier}x traffic ($vus VUs) ==="
  k6 run --vus $vus --duration 5m \
    --summary-export="${multiplier}x-results.json" \
    load-test-scenario.js
  echo "Results:"
  cat "${multiplier}x-results.json" | jq '{
    rps: .metrics.http_reqs.values.rate,
    p95_ms: .metrics.http_req_duration.values["p(95)"],
    error_rate: .metrics.http_req_failed.values.rate
  }'
done
```

### Step 3: Identify the Bottleneck

```bash
# Monitor all components during a 10x load test
# Terminal 1: Application metrics
watch -n 2 'kubectl top pods -l app=web'

# Terminal 2: Node metrics
watch -n 2 'kubectl top nodes'

# Terminal 3: Run the load test
k6 run --vus 500 --duration 10m load-test-scenario.js

# After test, check which component hit limits:
echo "=== Bottleneck Analysis ==="
echo "If CPU > 80%: Application is the bottleneck -> Scale pods"
echo "If Memory > 85%: Memory leak or insufficient limits"
echo "If DB connections > 80%: Database is the bottleneck -> Add pooling/replicas"
echo "If Network saturated: Move to larger instances"
echo "If Disk I/O high: Add caching or faster storage"
```

### Step 4: Generate Capacity Plan

```python
# generate_plan.py — Generate capacity plan from load test results
from datetime import datetime, timedelta

results = {
    "baseline": {"rps": 2000, "p95_ms": 150, "error_rate": 0.001},
    "2x": {"rps": 4000, "p95_ms": 200, "error_rate": 0.005},
    "5x": {"rps": 10000, "p95_ms": 450, "error_rate": 0.02},
    "10x": {"rps": 18000, "p95_ms": 1200, "error_rate": 0.08},
    "breaking_point": {"rps": 22000, "p95_ms": 5000, "error_rate": 0.15},
}

sustainable_rps = 0
for level, data in results.items():
    if data["error_rate"] < 0.05 and data["p95_ms"] < 1000:
        sustainable_rps = max(sustainable_rps, data["rps"])

print("=" * 60)
print("CAPACITY PLAN FOR PRODUCT LAUNCH")
print("=" * 60)
print(f"Launch Date:      {(datetime.now() + timedelta(days=14)).strftime('%Y-%m-%d')}")
print(f"Expected Traffic:  20,000 RPS")
print(f"\nCURRENT CAPACITY:")
print(f"  Sustainable RPS: {sustainable_rps:,}")
print(f"  Max RPS:         {results['breaking_point']['rps']:,}")

if sustainable_rps >= 20000:
    print(f"\nVERDICT: System CAN handle launch traffic")
    print(f"ACTION:  Monitor during launch, no scaling needed")
else:
    scale_factor = 20000 / sustainable_rps
    print(f"\nVERDICT: System CANNOT handle launch traffic")
    print(f"SCALE NEEDED: {scale_factor:.1f}x current capacity")
    print(f"\nRECOMMENDED ACTIONS:")
    print(f"  1. Scale web tier from 3 to {int(3 * scale_factor)} replicas")
    print(f"  2. Add database read replicas (if DB is bottleneck)")
    print(f"  3. Enable caching layer (Redis)")
    print(f"  4. Configure CDN for static assets")
    print(f"  5. Implement rate limiting at edge")
```

### Lab Validation Checklist

- [ ] Baseline metrics collected for current traffic level
- [ ] Load tests run at 2x, 5x, and 10x traffic
- [ ] Breaking point identified with specific RPS number
- [ ] Bottleneck component identified (CPU, memory, DB, network, or disk)
- [ ] Capacity plan generated with specific scaling recommendations
- [ ] Cost estimate calculated for recommended scaling
- [ ] Action items with deadlines created

## Limitation -> Next Topic

You can now forecast demand, find breaking points, and plan capacity. But when traffic grows, the first thing to optimize is not adding more servers — it is reducing the load on existing servers through caching. A well-designed cache can eliminate 90% of database queries and reduce your infrastructure needs by 10x.

Capacity planning tells you how much infrastructure you need. Caching strategies tell you how to need less.

**Next: [Module 70 — Caching Strategies](../70-caching-strategies/README.md)**
