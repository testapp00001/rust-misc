# Solution 02: Writing an HPA with Custom Metrics and Behavior Policies

## Part A: HPA Manifest

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: payment-api-hpa
  namespace: production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: payment-api
  minReplicas: 3
  maxReplicas: 30
  metrics:
    # Primary metric: CPU utilization
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 65
    # Secondary metric: p99 latency (custom pod metric)
    - type: Pods
      pods:
        metric:
          name: http_request_duration_p99_seconds
        target:
          type: AverageValue
          averageValue: "500m"    # 500ms = 0.5s, expressed as 500m in Kubernetes quantity
    # Tertiary metric: database connections (custom pod metric)
    - type: Pods
      pods:
        metric:
          name: active_db_connections
        target:
          type: AverageValue
          averageValue: "50"
```

### Why This Works

- `type: Resource` with `name: cpu` uses the built-in metrics-server, which requires no additional setup.
- `type: Pods` with custom metric names must match exactly what the Prometheus Adapter exposes. The adapter translates PromQL queries into the Kubernetes custom metrics API (`custom.metrics.k8s.io`).
- `averageValue: "500m"` uses Kubernetes quantity notation. `500m` means 0.5 (500 millicores/500 milliseconds). For latency metrics, this is the per-pod average target.
- HPA calculates desired replicas for each metric independently and selects the **highest** replica count. This ensures no SLO is violated -- if CPU is fine but latency is high, the latency metric drives scaling.

---

## Part B: Behavior Policies

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: payment-api-hpa
  namespace: production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: payment-api
  minReplicas: 3
  maxReplicas: 30
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 65
    - type: Pods
      pods:
        metric:
          name: http_request_duration_p99_seconds
        target:
          type: AverageValue
          averageValue: "500m"
    - type: Pods
      pods:
        metric:
          name: active_db_connections
        target:
          type: AverageValue
          averageValue: "50"
  behavior:
    # Scale up: fast and aggressive
    scaleUp:
      stabilizationWindowSeconds: 0     # React immediately
      policies:
        - type: Percent
          value: 100                    # Can double replicas
          periodSeconds: 15
        - type: Pods
          value: 6                      # Or add 6 pods
          periodSeconds: 15
      selectPolicy: Max                 # Use whichever policy allows more scaling
    # Scale down: slow and cautious
    scaleDown:
      stabilizationWindowSeconds: 600   # Wait 10 minutes before scaling down
      policies:
        - type: Percent
          value: 10                     # Remove at most 10% of pods
          periodSeconds: 60             # Every 60 seconds
```

### Why This Works

**Scale-up:**
- `stabilizationWindowSeconds: 0` means the HPA uses the most recent metric value without any smoothing. This provides the fastest possible reaction to traffic spikes.
- Two policies (100% and 6 pods) with `selectPolicy: Max` means the HPA picks whichever allows more scaling. With 3 replicas: 100% = 3 new pods, but Pods policy = 6 new pods. With 20 replicas: 100% = 20 new pods, Pods policy = 6 new pods. The percentage policy is more aggressive at high replica counts, the absolute policy is more aggressive at low replica counts.

**Scale-down:**
- `stabilizationWindowSeconds: 600` (10 minutes) means the HPA looks at the last 10 minutes of metrics and uses the highest value for the scale-down decision. This prevents premature scale-down from brief dips in traffic.
- `10% every 60 seconds` is conservative. With 30 replicas, that is 3 pods per minute. With 3 replicas, it rounds to 0 -- the HPA will not scale below 3. This gradual approach prevents flapping.

---

## Part C: Verify and Test Commands

```bash
# 1. Apply the HPA manifest
kubectl apply -f payment-api-hpa.yaml

# 2. Watch HPA status in real time (refresh every 3 seconds)
kubectl get hpa payment-api-hpa -n production -w --all-namespaces

# 3. Describe the HPA to see events and conditions
kubectl describe hpa payment-api-hpa -n production

# 4. Check custom metrics from Prometheus Adapter
kubectl get --raw "/apis/custom.metrics.k8s.io/v1beta1/namespaces/production/pods/*/http_request_duration_p99_seconds?selector=app%3Dpayment-api"
kubectl get --raw "/apis/custom.metrics.k8s.io/v1beta1/namespaces/production/pods/*/active_db_connections?selector=app%3Dpayment-api"

# 5. Generate load to trigger scale-up
# Option A: Using hey (install: go install github.com/rakyll/hey@latest)
hey -z 5m -c 100 -q 50 http://payment-api.production.svc.cluster.local:8080/

# Option B: Using kubectl run with a simple loop
kubectl run load-generator --image=busybox --restart=Never -- \
  /bin/sh -c "while true; do wget -q -O- http://payment-api.production.svc.cluster.local:8080/; done"

# Option C: Using k6 (install: brew install k6)
cat > load-test.js << 'EOF'
import http from 'k6/http';
import { sleep } from 'k6';

export let options = {
  stages: [
    { duration: '2m', target: 100 },
    { duration: '5m', target: 100 },
    { duration: '2m', target: 0 },
  ],
};

export default function () {
  http.get('http://payment-api.production.svc.cluster.local:8080/');
  sleep(0.02);
}
EOF
k6 run load-test.js
```

### Why This Works

- `kubectl get hpa -w` watches for changes in real time, showing replica count changes as they happen.
- `kubectl describe hpa` shows the Events section, which logs every scaling decision with the reason (which metric triggered it).
- The custom metrics API query uses URL-encoded label selectors (`app%3Dpayment-api` is `app=payment-api`). If this returns an error, the Prometheus Adapter is not configured correctly for this metric.

---

## Part D: Metric Priority Analysis

Given:

| Metric | Current Value | Target | Current Replicas |
|--------|---------------|--------|------------------|
| CPU | 82% average | 65% | 10 |
| p99 Latency | 0.8s average | 0.5s | 10 |
| DB Connections | 45 average | 50 | 10 |

**Calculations:**

```
CPU:          desiredReplicas = ceil(10 * (82 / 65))  = ceil(10 * 1.2615) = ceil(12.615) = 13
p99 Latency:  desiredReplicas = ceil(10 * (0.8 / 0.5)) = ceil(10 * 1.6)   = ceil(16.0)   = 16
DB Connections: desiredReplicas = ceil(10 * (45 / 50))  = ceil(10 * 0.9)   = ceil(9.0)    = 9
```

**Result:** HPA picks the **maximum** across all metrics: **16 replicas** (driven by p99 latency).

The DB connections metric (9 desired) is below the current count, so it does not drive scaling. The CPU metric wants 13 replicas. The latency metric wants 16 replicas. HPA chooses 16 to ensure the latency SLO is met.

### Why This Works

HPA uses `max()` across all metrics to guarantee that **no** SLO is violated. If it used `min()`, the low DB connections value (9) would prevent scaling, and latency would stay high. If it used `average()`, the result (12.67) would undershoot the latency requirement. `max()` is the correct strategy because it treats each metric as an independent constraint -- all constraints must be satisfied simultaneously.

---

## Common Mistakes to Avoid

- **Using `Utilization` instead of `AverageValue` for custom metrics.** `Utilization` is only valid for `type: Resource` (CPU/memory). Custom pod metrics use `AverageValue`, which is an absolute value, not a percentage of a request.

- **Forgetting the Prometheus Adapter.** Custom metrics only work if Prometheus Adapter is configured to expose them. Test with `kubectl get --raw` before relying on them in HPA.

- **Setting `stabilizationWindowSeconds: 0` for scale-down.** This causes flapping. Scale-down should always have a stabilization window of 300-600 seconds.

- **Using a single metric when multiple bottlenecks exist.** CPU alone misses database saturation. Latency alone misses cold-start issues. Use multiple metrics to cover all bottleneck vectors.

- **Confusing Kubernetes quantity notation.** `500m` means 0.5, not 500. For latency in seconds, `500m` = 0.5s. For CPU, `500m` = 0.5 cores. The `m` suffix always means "milli" (1/1000).

## Key Takeaway

HPA with multiple metrics and proper behavior policies is the foundation of production autoscaling. Each metric covers a different bottleneck vector, and the `max()` selection strategy ensures all SLOs are met. The behavior section is not optional -- without asymmetric policies (fast scale-up, slow scale-down), HPA will oscillate and waste resources.
