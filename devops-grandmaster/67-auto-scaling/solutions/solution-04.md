# Solution 04: Debugging a Flapping HPA

## Part A: Root Causes Identified

There are **4 distinct problems** contributing to the flapping behavior:

### Problem 1: Symmetric Scale-Up and Scale-Down Policies

Both `scaleUp` and `scaleDown` have `stabilizationWindowSeconds: 0` and `value: 100` every 15 seconds. This means the HPA can double replicas going up and halve them going down, with no smoothing. The result is a sawtooth pattern: scale up to 12, immediately scale down to 5, immediately scale up to 12.

**Why this causes flapping:** Without a stabilization window, the HPA uses only the most recent metric value. After scaling up, CPU per pod drops (same load, more pods), and the HPA immediately interprets this as "all metrics below target" and scales back down.

### Problem 2: CPU Target Too Low (50%)

A 50% CPU target is aggressive for most workloads. Normal traffic variance easily crosses 50% in both directions. With 5 replicas at 52% CPU, the HPA scales up. With 12 replicas, CPU drops to approximately 22%, well below 50%, triggering scale-down.

**Why this causes flapping:** The 50% target creates a narrow band where the HPA is satisfied. Any variance above triggers scale-up; the resulting load distribution drops CPU below the target, triggering scale-down. A higher target (65-70%) gives more headroom for normal variance.

### Problem 3: Aggressive Scale-Down (100% Every 15 Seconds)

The scale-down policy allows removing 100% of excess replicas every 15 seconds. When the HPA decides to scale from 12 to 5, it can do so in a single step. This rapid removal causes a sudden CPU spike on the remaining pods, which triggers scale-up again.

**Why this causes flapping:** Rapid scale-down increases load on remaining pods, which immediately triggers scale-up. The system oscillates between two states: too many pods (CPU low) and too few pods (CPU high).

### Problem 4: Noisy Memory Metric

Memory utilization is at 35%, well below the 70% target. However, the HPA evaluates all metrics when making scaling decisions. When scaling down, the HPA checks: "Are all metrics below target?" CPU is below target (after scale-up), and memory is below target. Both conditions are met, so scale-down proceeds.

**Why this contributes to flapping:** The memory metric reinforces the scale-down decision even though memory is not a meaningful scaling signal for this workload. It is a passive accomplice -- it does not drive scale-up, but it enables scale-down by always being "below target."

---

## Part B: Oscillation Cycle

```
Step 1: 5 replicas, CPU at 52% (above 50% target)
        HPA calculates: ceil(5 * 52/50) = ceil(5.2) = 6
        Scale-up policy allows 100% = 5 new pods -> 10 replicas
        (selectPolicy: Max, so the 100% policy wins)

Step 2: 10 replicas, CPU drops to ~26% (52% * 5/10)
        CPU is below 50% target. Memory at 35% is below 70% target.
        "All metrics below target" -> scale-down triggers
        Scale-down policy allows 100% removal -> remove 5 pods -> 5 replicas

Step 3: 5 replicas, CPU climbs back to 52%
        Above 50% target -> scale-up triggers again
        Back to 10-12 replicas

Step 4: 10-12 replicas, CPU drops below target
        Scale-down triggers -> back to 5 replicas

(cycle repeats every 5-10 minutes)
```

The key insight: the 50% CPU target creates a feedback loop. Scale-up distributes load, dropping CPU below target. Scale-down concentrates load, pushing CPU above target. With symmetric policies, the HPA has no reason to settle at any intermediate replica count.

---

## Part C: Fixed HPA Manifest

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: web-app-hpa
  namespace: production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: web-app
  minReplicas: 3
  maxReplicas: 50
  metrics:
    # Single metric: CPU utilization at 65%
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 65
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 30
      policies:
        - type: Percent
          value: 100
          periodSeconds: 30
        - type: Pods
          value: 4
          periodSeconds: 30
      selectPolicy: Max
    scaleDown:
      stabilizationWindowSeconds: 600
      policies:
        - type: Percent
          value: 10
          periodSeconds: 60
```

### What Changed and Why

1. **CPU target raised from 50% to 65%.** This gives 15% more headroom for normal traffic variance before triggering scale-up. The HPA will only scale when the system is genuinely under pressure, not during routine fluctuations.

2. **Memory metric removed.** Memory at 35% is not a useful scaling signal. It was contributing to scale-down decisions without driving scale-up. Removing it eliminates a source of noise. If memory becomes a concern later, add it back with a VPA (not HPA).

3. **Scale-down stabilization window: 0 -> 600 seconds.** This is the single most important fix. The HPA now looks at the last 10 minutes of metrics before deciding to scale down. Brief dips in CPU (e.g., after a scale-up distributes load) are smoothed out. The HPA will only scale down if CPU has been consistently below target for 10 minutes.

4. **Scale-down rate: 100% every 15s -> 10% every 60s.** Even when the HPA decides to scale down, it does so gradually. With 50 replicas, that is 5 pods per minute. With 3 replicas, it rounds to 0. This prevents the sudden load concentration that triggers scale-up.

5. **Scale-up stabilization window: 0 -> 30 seconds.** A small smoothing window prevents the HPA from reacting to sub-second CPU spikes. 30 seconds is short enough to remain responsive to genuine traffic increases.

### Why This Works

The fix implements the core anti-flap principle: **asymmetric policies.** Scale-up is still fast (0-30 second window, 100% or 4 pods), so users do not experience latency during traffic spikes. Scale-down is slow (600 second window, 10% per minute), so the system does not oscillate. The higher CPU target (65%) reduces sensitivity to normal variance.

---

## Part D: Prometheus Alert for Flapping Detection

```yaml
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: hpa-flapping-alert
  namespace: monitoring
spec:
  groups:
    - name: hpa.flapping.rules
      rules:
        - alert: HPAFlapping
          expr: |
            changes(kube_horizontalpodautoscaler_status_current_replicas[30m]) > 4
          for: 5m
          labels:
            severity: warning
          annotations:
            summary: "HPA {{ $labels.horizontalpodautoscaler }} in {{ $labels.namespace }} is flapping"
            description: >-
              HPA has changed replica count more than 4 times in the last 30 minutes.
              This indicates oscillating scaling behavior. Check scaling policies
              and metric stability.
```

### Why This Works

- `changes()` counts the number of times a time series value changed within the given time window. A healthy HPA might scale 1-2 times in 30 minutes. More than 4 changes indicates flapping.
- `for: 5m` prevents alert flapping itself -- the condition must persist for 5 minutes before the alert fires.
- The alert uses `$labels.horizontalpodautoscaler` and `$labels.namespace` to identify which HPA is flapping, making it actionable.

---

## Part E: Validate the Fix

```bash
# 1. Apply the corrected HPA
kubectl apply -f web-app-hpa-fixed.yaml

# 2. Watch HPA status in real time
kubectl get hpa web-app-hpa -n production -w

# What to look for:
# - Replica count should change at most once every few minutes
# - CPU should stabilize around 60-70% per pod
# - No rapid oscillation between two replica counts

# 3. Describe HPA to check events
kubectl describe hpa web-app-hpa -n production

# What to look for:
# - Events should show gradual scaling (e.g., 3 -> 5 -> 7, not 3 -> 12 -> 3)
# - Scale-down events should show "All metrics below target" with a delay
# - No "All metrics below target" events immediately after scale-up

# 4. Generate steady load and observe
kubectl run load-test --image=busybox --restart=Never -- \
  /bin/sh -c "while true; do wget -q -O- http://web-app.production.svc.cluster.local/; done"

# What to look for:
# - Replicas should climb gradually and stabilize (e.g., 3 -> 5 -> 8 -> 8 -> 8)
# - No drop-and-climb pattern

# 5. Remove load and observe scale-down
kubectl delete pod load-test

# What to look for:
# - No immediate scale-down (stabilization window should hold for 10 minutes)
# - Gradual scale-down over 10-20 minutes (10% per minute)
# - Replicas should settle at minReplicas (3), not oscillate

# 6. Verify with Prometheus (if available)
# Check for the HPAFlapping alert
curl -s http://prometheus:9090/api/v1/alerts | jq '.data.alerts[] | select(.labels.alertname=="HPAFlapping")'
# Should return empty (no flapping detected)
```

### What Indicates Success

- Replica count remains stable during steady traffic (no oscillation)
- Scale-down happens gradually over 10+ minutes after traffic drops
- The `kubectl describe hpa` events show no more than 2-3 scaling events per 30 minutes
- The Prometheus `HPAFlapping` alert does not fire

---

## Common Mistakes to Avoid

- **Raising the CPU target too high (e.g., 80%).** While this reduces flapping, it also reduces headroom. If traffic spikes suddenly, the HPA has less time to react before pods become overloaded. 65% is a good balance.

- **Removing the memory metric without understanding why it was there.** Memory metrics are useful when memory is the actual bottleneck (e.g., in-memory caches). For this workload, memory is not the constraint, so removing it is correct. But do not remove metrics blindly.

- **Setting scale-down stabilization too long (e.g., 3600 seconds / 1 hour).** While this prevents flapping, it also means you pay for unused capacity for an hour after traffic drops. 600 seconds (10 minutes) is a reasonable default.

- **Only fixing one problem.** This HPA had 4 interacting problems. Fixing only the stabilization window but keeping the 50% target would reduce but not eliminate flapping. All 4 fixes work together.

- **Not testing under realistic load.** Synthetic tests with constant load do not reveal flapping. Real traffic has variance, which is what triggers the oscillation. Test with variable load patterns.

## Key Takeaway

HPA flapping is caused by symmetric scaling policies that react to each other's effects. The fix is always asymmetric: fast scale-up (stabilization 0-30s) and slow scale-down (stabilization 300-600s). A higher CPU target (65% instead of 50%) reduces sensitivity to normal variance. Removing noisy metrics that do not represent actual bottlenecks eliminates passive contributors to oscillation.
