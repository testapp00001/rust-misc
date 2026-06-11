# Exercise 04: Debugging a Flapping HPA

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Diagnose why an HPA is oscillating between scale-up and scale-down (flapping), identify the root causes in the configuration, and fix the behavior policies to achieve stable scaling.

## Scenario / Starting Point

Your team has deployed an HPA for a web application. The on-call engineer reports that every 5-10 minutes, the HPA scales from 5 replicas to 12, then back down to 5, then back up to 12. This causes:

- Intermittent latency spikes during pod startup
- Wasted compute costs from constant node provisioning
- Alert fatigue from repeated scaling notifications

Here is the current HPA manifest:

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
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 50
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 70
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 0
      policies:
        - type: Percent
          value: 100
          periodSeconds: 15
      selectPolicy: Max
    scaleDown:
      stabilizationWindowSeconds: 0
      policies:
        - type: Percent
          value: 100
          periodSeconds: 15
      selectPolicy: Max
```

Here is the output of `kubectl describe hpa web-app-hpa`:

```
Name:                                                  web-app-hpa
Namespace:                                             production
Metrics:                                               ( current / target )
  resource cpu on pods  (as a percentage of request):  52% (208m) / 50%
  resource memory on pods  (as a percentage of request): 35% (179Mi) / 70%
Conditions:
  Type            Status  Reason              Message
  AbleToScale     True    ReadyForNewScale    recommended size matches current size
  ScalingActive   True    ValidMetricFound    the HPA was able to successfully calculate a replica count from cpu resource utilization (percentage of request)
  ScalingLimited  False   DesiredWithinRange  the desired count is within the acceptable range
Events:
  Type    Reason              Age   From                       Message
  ----    ------              ----  ----                       -------
  Normal  SuccessfulRescale   2m    horizontal-pod-autoscaler  New size: 12; reason: cpu resource utilization (percentage of request) above target
  Normal  SuccessfulRescale   90s   horizontal-pod-autoscaler  New size: 5; reason: All metrics below target
  Normal  SuccessfulRescale   45s   horizontal-pod-autoscaler  New size: 11; reason: cpu resource utilization (percentage of request) above target
  Normal  SuccessfulRescale   10s   horizontal-pod-autoscaler  New size: 5; reason: All metrics below target
```

## Tasks

### Part A: Identify All Root Causes

There are at least **4 distinct problems** in this HPA configuration that contribute to the flapping behavior. Identify each one and explain why it causes oscillation.

List your findings:

1. Problem 1: ...
2. Problem 2: ...
3. Problem 3: ...
4. Problem 4: ...

<details>
<summary>Hint 1</summary>

Look at the scale-down behavior first. The `stabilizationWindowSeconds: 0` for scale-down means the HPA will immediately consider scaling down as soon as metrics dip below the target. What happens when 12 pods share the load? The CPU per pod drops, triggering scale-down. Then with fewer pods, CPU per pod rises again.

</details>

<details>
<summary>Hint 2</summary>

The CPU target is 50%. That is very aggressive. Each time new pods start, the CPU per pod drops because the load is spread across more pods. With a 50% target, even a small fluctuation crosses the threshold in both directions.

</details>

<details>
<summary>Hint 3</summary>

Look at the scale-up policy: 100% every 15 seconds with stabilization window 0. This means the HPA can double the replica count every 15 seconds. Combined with an equally aggressive scale-down, you get a sawtooth pattern.

</details>

<details>
<summary>Hint 4</summary>

Memory utilization at 35% is well below the 70% target. Yet the HPA considers "all metrics below target" when scaling down. The memory metric is contributing to the scale-down decision even though memory is not the bottleneck. Think about how multi-metric HPA evaluates metrics for scale-up vs. scale-down.

</details>

### Part B: Explain the Oscillation Cycle

Draw out (in text) one complete oscillation cycle, explaining what happens at each step:

```
Step 1: 5 replicas, CPU at 52% -> ...
Step 2: ...
Step 3: ...
Step 4: ...
(cycle repeats)
```

<details>
<summary>Hint 5</summary>

The cycle is:
1. 5 replicas, CPU at 52% (above 50% target) -> scale up
2. 12 replicas, CPU drops to ~22% (52% * 5/12) -> below 50% target
3. Scale-down triggers immediately (stabilization = 0) -> scale down
4. Back to ~5 replicas, CPU climbs above 50% -> scale up again

The key insight: scale-up adds too many pods too fast, then scale-down removes them just as fast.

</details>

### Part C: Fix the HPA Manifest

Write a corrected HPA manifest that prevents flapping while remaining responsive to genuine traffic increases. Your fix should address all 4 problems identified in Part A.

Requirements for the fixed version:
- CPU target: 65% (not 50%)
- Remove the memory metric (memory at 35% is noise, not signal)
- Scale-up: stabilization window of 30 seconds, allow doubling or adding 4 pods every 30 seconds
- Scale-down: stabilization window of 600 seconds, remove at most 10% of pods every 60 seconds
- Keep `minReplicas: 3` and `maxReplicas: 50`

```yaml
# Your fixed HPA manifest here
```

<details>
<summary>Hint 6</summary>

The key changes are:
1. Raise CPU target from 50% to 65% -- gives more headroom before triggering scale-up
2. Remove memory metric -- it is not a useful scaling signal at 35% utilization
3. Add stabilization window to scale-down (600s) -- prevents reactive scale-down
4. Slow down scale-down rate (10% per 60s instead of 100% per 15s)

</details>

### Part D: Write a Prometheus Alert for Flapping Detection

Write a Prometheus alert rule that fires when an HPA has scaled more than 4 times in the last 30 minutes, which indicates flapping behavior.

```yaml
# Your PrometheusRule here
```

<details>
<summary>Hint 7</summary>

Use `changes()` over a time window:

```promql
changes(kube_horizontalpodautoscaler_status_current_replicas[30m]) > 4
```

This counts the number of times the metric value changed in the 30-minute window.

</details>

### Part E: Validate the Fix

After applying your corrected HPA, how would you verify that flapping is resolved? Write the commands and describe what you would look for in the output.

<details>
<summary>Hint 8</summary>

You would:
1. `kubectl get hpa web-app-hpa -w` -- watch for stable replica counts
2. `kubectl describe hpa web-app-hpa` -- check events show gradual scaling, not rapid oscillation
3. Generate steady load and observe: replicas should climb once and stabilize, not oscillate
4. Remove load and observe: replicas should decrease slowly over 10+ minutes, not drop suddenly

</details>

## Success Criteria

- [ ] You identified at least 4 root causes: aggressive scale-down (stabilization=0), low CPU target (50%), symmetric policies, and noisy memory metric.
- [ ] The oscillation cycle explanation correctly traces the replica count up and down with CPU percentage calculations.
- [ ] The fixed HPA has asymmetric policies: fast scale-up, slow scale-down with a 600-second stabilization window.
- [ ] The Prometheus alert correctly uses `changes()` to detect rapid scaling events.
- [ ] You can explain why asymmetric policies (fast up, slow down) prevent flapping.

## What You Should Understand After This Exercise

HPA flapping is the most common autoscaling problem in production. It happens when scale-up and scale-down policies are equally aggressive, causing the system to oscillate between two replica counts. The fix is always asymmetric: scale up fast (users are waiting), scale down slow (you already paid for the resources). A stabilization window on scale-down is the single most effective anti-flap mechanism. Raising the CPU target from 50% to 65% also reduces sensitivity to normal traffic variance.
