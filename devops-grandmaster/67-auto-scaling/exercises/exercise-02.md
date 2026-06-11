# Exercise 02: Writing an HPA with Custom Metrics and Behavior Policies

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Write a complete HorizontalPodAutoscaler manifest that uses custom metrics alongside CPU-based scaling, with behavior policies that prevent scaling oscillation while remaining responsive to real traffic changes.

## Scenario / Starting Point

You run a payment processing API in the `production` namespace. The API is deployed as a Deployment named `payment-api`. During Black Friday, request latency becomes the primary concern -- CPU may look fine while the database connection pool is saturated, causing p99 latency to spike.

Your team has already installed:
- `metrics-server` (for CPU/memory metrics)
- Prometheus Adapter (for custom metrics from your app's `/metrics` endpoint)

The application exposes these Prometheus metrics:
- `http_requests_per_second` -- current RPS per pod
- `http_request_duration_p99_seconds` -- p99 latency per pod
- `active_db_connections` -- active database connections per pod

Current deployment spec:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: payment-api
  namespace: production
spec:
  replicas: 3
  selector:
    matchLabels:
      app: payment-api
  template:
    metadata:
      labels:
        app: payment-api
    spec:
      containers:
        - name: api
          image: payment-api:v2.3.1
          resources:
            requests:
              cpu: 250m
              memory: 512Mi
            limits:
              cpu: "1"
              memory: 1Gi
```

## Tasks

### Part A: Write the HPA Manifest

Create a complete `HorizontalPodAutoscaler` manifest (apiVersion `autoscaling/v2`) that:

1. Targets the `payment-api` Deployment
2. Sets `minReplicas: 3` (HA requirement) and `maxReplicas: 30`
3. Includes three metrics:
   - CPU utilization with a target of 65%
   - `http_request_duration_p99_seconds` with a target of 0.5 (500ms) per pod
   - `active_db_connections` with a target of 50 per pod
4. Uses `AverageValue` for the custom metrics (not `Utilization`)

Write the complete YAML below:

```yaml
# Your HPA manifest here
```

<details>
<summary>Hint 1</summary>

For custom pod metrics, use `type: Pods` with `target.type: AverageValue`. The metric name must match exactly what Prometheus Adapter exposes. The structure is:

```yaml
- type: Pods
  pods:
    metric:
      name: <metric_name>
    target:
      type: AverageValue
      averageValue: "<value>"
```

</details>

### Part B: Add Behavior Policies

Extend your HPA manifest with a `behavior` section that implements these rules:

**Scale-up policy:**
- React immediately (stabilization window of 0 seconds)
- Allow doubling replicas (100% increase) every 15 seconds
- Alternatively, add up to 6 pods every 15 seconds
- Use whichever policy allows more scaling (`selectPolicy: Max`)

**Scale-down policy:**
- Wait 10 minutes before considering scale-down (stabilization window of 600 seconds)
- Remove at most 10% of pods every 60 seconds
- This ensures connections drain properly and avoids flapping

Add the `behavior` block to your YAML from Part A.

<details>
<summary>Hint 2</summary>

The `behavior` field goes under `spec`, at the same level as `metrics` and `scaleTargetRef`. Scale-up and scale-down are configured independently with their own `stabilizationWindowSeconds` and `policies` arrays.

</details>

<details>
<summary>Hint 3</summary>

A `Percent` policy of `10` with `periodSeconds: 60` means "remove at most 10% of current replicas every 60 seconds." With 30 replicas, that is 3 pods per minute. With 3 replicas, that rounds down to 0 -- the HPA will not scale below 3 if the math rounds to zero removal.

</details>

### Part C: Verify and Test Commands

Write the `kubectl` commands to:

1. Apply your HPA manifest
2. Watch the HPA status in real time (refresh every 3 seconds)
3. Describe the HPA to see events and conditions
4. Check what custom metrics the Prometheus Adapter is exposing for your pods
5. Manually generate load to trigger scale-up (assume the API is at `http://payment-api.production.svc.cluster.local:8080/`)

<details>
<summary>Hint 4</summary>

To query custom metrics from the Kubernetes API:

```bash
kubectl get --raw "/apis/custom.metrics.k8s.io/v1beta1/namespaces/production/pods/*/http_request_duration_p99_seconds?selector=app%3Dpayment-api"
```

For load generation, you can use `kubectl run` with a busybox image and a loop, or use `hey`/`k6`/`wrk`.

</details>

### Part D: Metric Priority Analysis

When HPA has multiple metrics, it calculates the desired replica count for each metric independently and picks the **highest** value. Given these current readings, calculate the desired replica count for each metric and determine the final replica count:

| Metric | Current Value | Target | Current Replicas |
|--------|---------------|--------|------------------|
| CPU | 82% average | 65% | 10 |
| p99 Latency | 0.8s average | 0.5s | 10 |
| DB Connections | 45 average | 50 | 10 |

Show your calculation for each metric using the formula:

```
desiredReplicas = ceil[currentReplicas * (currentMetricValue / targetMetricValue)]
```

<details>
<summary>Hint 5</summary>

For utilization-based metrics (CPU), the formula is straightforward. For AverageValue custom metrics, the same ratio applies. The HPA picks the maximum across all metrics to ensure no SLO is violated.

</details>

## Success Criteria

- [ ] HPA manifest uses `autoscaling/v2` API version and targets the correct Deployment.
- [ ] All three metrics are defined with correct types (`Resource` for CPU, `Pods` for custom metrics).
- [ ] Custom metrics use `AverageValue` targets with realistic values (500ms for latency, 50 for connections).
- [ ] Behavior policies implement fast scale-up and slow scale-down with correct stabilization windows.
- [ ] Metric priority calculation is correct: HPA picks the metric that demands the most replicas.

## What You Should Understand After This Exercise

HPA with multiple metrics gives you resilience against single-metric blindness. CPU alone misses database saturation; latency alone misses cold-start issues. The behavior section is not optional in production -- without it, HPA will oscillate aggressively, especially during traffic fluctuations. Scale-up should be fast (users are waiting), scale-down should be slow (you paid for those resources already, and premature removal causes re-provisioning churn).
