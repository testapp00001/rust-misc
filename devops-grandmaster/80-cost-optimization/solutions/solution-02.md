# Solution 02: Right-Sizing Kubernetes Workloads

## Problem Statement

A Kubernetes cluster has workloads that are over-provisioned and
under-provisioned. Using Prometheus metrics, build a right-sizing pipeline
that identifies misconfigured resource requests and limits, generates
recommendations, and applies corrections.

## Complete Solution

### Part 1: Prometheus Queries for CPU Utilization

Query actual CPU usage against requested CPU over a 7-day window:

```promql
-- Actual CPU usage (cores) per container, 7-day average
avg_over_time(
  rate(
    container_cpu_usage_seconds_total{
      container!="POD",
      container!="",
      namespace=~"production|staging"
    }[5m]
  )[7d:5m]
)
```

```promql
-- CPU requested per container
kube_pod_container_resource_requests{
  resource="cpu",
  container!="POD",
  container!="",
  namespace=~"production|staging"
}
```

```promql
-- CPU utilization ratio (usage / requested) -- target is 60-80%
avg_over_time(
  rate(
    container_cpu_usage_seconds_total{
      container!="POD",
      container!="",
      namespace=~"production|staging"
    }[5m]
  )[7d:5m]
)
/
kube_pod_container_resource_requests{
  resource="cpu",
  container!="POD",
  container!="",
  namespace=~"production|staging"
}
```

```promql
-- P99 CPU usage for setting limits
quantile_over_time(0.99,
  rate(
    container_cpu_usage_seconds_total{
      container!="POD",
      container!="",
      namespace=~"production|staging"
    }[5m]
  )[7d:5m]
)
```

### Part 2: Prometheus Queries for Memory Utilization

```promql
-- Actual memory usage per container, 7-day average
avg_over_time(
  container_memory_working_set_bytes{
    container!="POD",
    container!="",
    namespace=~"production|staging"
  }[7d]
)
```

```promql
-- Memory requested per container
kube_pod_container_resource_requests{
  resource="memory",
  container!="POD",
  container!="",
  namespace=~"production|staging"
}
```

```promql
-- Memory utilization ratio
avg_over_time(
  container_memory_working_set_bytes{
    container!="POD",
    container!="",
    namespace=~"production|staging"
  }[7d]
)
/
kube_pod_container_resource_requests{
  resource="memory",
  container!="POD",
  container!="",
  namespace=~"production|staging"
}
```

```promql
-- P99 memory usage for setting limits
quantile_over_time(0.99,
  container_memory_working_set_bytes{
    container!="POD",
    container!="",
    namespace=~"production|staging"
  }[7d]
)
```

### Part 3: Right-Sizing Recommendation Logic

The recommendation algorithm applies safety margins on top of observed usage:

```
Right-Sizing Algorithm
=======================

For each container:

  CPU Request Recommendation:
    cpu_request_new = p95_cpu_usage * 1.20   (20% headroom)
    cpu_limit_new   = p99_cpu_usage * 1.50   (50% above p99)

  Memory Request Recommendation:
    mem_request_new = p99_memory * 1.15       (15% headroom)
    mem_limit_new   = max(p99_memory * 1.50, current_limit)

  Classification Rules:
    IF current_request > cpu_request_new * 2.0:
      label = "OVER_PROVISIONED"
      action = "REDUCE"
      savings = (current - recommended) * price_per_unit * hours

    ELIF current_request < cpu_request_new * 0.8:
      label = "UNDER_PROVISIONED"
      action = "INCREASE"
      risk = "potential throttling or OOM"

    ELSE:
      label = "RIGHT_SIZED"
      action = "NO_CHANGE"

  Priority:
    UNDER_PROVISIONED items first (reliability risk)
    OVER_PROVISIONED items second (cost savings)
```

### Part 4: Example Output

```
Right-Sizing Report -- Generated 2026-06-11
==============================================

OVER-PROVISIONED (22 containers)
---------------------------------

Namespace     Pod/Container          CPU Req  CPU Avg  CPU Rec  Mem Req  Mem Avg  Mem Rec  Savings/mo
                                     (cores)  (cores)  (cores)  (MiB)    (MiB)    (MiB)    ($)
-----------------------------------------------------------------------------------------------------+
prod          api-server/app         2.000    0.340    0.500    4096     1200     1536     $142.56
prod          api-server/envoy       1.000    0.080    0.150    512      180      256      $ 72.18
prod          worker-pool/processor  4.000    0.920    1.250    8192     3200     4096     $198.24
prod          web-frontend/nginx     1.000    0.120    0.200    2048     400      512      $ 68.40
staging       test-runner/runner     2.000    0.150    0.250    4096     600      768      $125.80
...           ...                    ...      ...      ...      ...      ...      ...      ...
-----------------------------------------------------------------------------------------------------+
                                                                              TOTAL:     $2,847/mo

UNDER-PROVISIONED (8 containers)
----------------------------------

Namespace     Pod/Container          CPU Req  CPU P99  CPU Rec  Mem Req  Mem P99  Mem Rec  Throttle%
                                     (cores)  (cores)  (cores)  (MiB)    (MiB)    (MiB)
-----------------------------------------------------------------------------------------------------+
prod          search-service/app     0.250    0.480    0.600    256      480      576      12.3%
prod          payment-gw/app         0.500    0.920    1.150    512      920      1100     8.7%
prod          notification/app       0.100    0.190    0.250    128      240      288      5.2%
-----------------------------------------------------------------------------------------------------+

RIGHT-SIZED (45 containers) -- No changes needed.
```

### Part 5: Applying Recommendations with kubectl

**Dry-run first to validate changes:**

```bash
# Generate patch commands from recommendations
kubectl patch deployment api-server -n production --type='json' \
  -p='[{"op":"replace","path":"/spec/template/spec/containers/0/resources/requests/cpu","value":"500m"},
       {"op":"replace","path":"/spec/template/spec/containers/0/resources/requests/memory","value":"1536Mi"},
       {"op":"replace","path":"/spec/template/spec/containers/0/resources/limits/cpu","value":"750m"},
       {"op":"replace","path":"/spec/template/spec/containers/0/resources/limits/memory","value":"2304Mi"}]' \
  --dry-run=server
```

**Automated batch script for applying changes:**

```bash
#!/usr/bin/env bash
# apply-right-sizing.sh -- Apply right-sizing recommendations from CSV report
set -euo pipefail

REPORT_FILE="${1:-right-sizing-report.csv}"
DRY_RUN="${2:---dry-run=server}"

if [[ ! -f "$REPORT_FILE" ]]; then
  echo "Error: Report file not found: $REPORT_FILE" >&2
  exit 1
fi

APPLIED=0
FAILED=0

while IFS=, read -r namespace workload container cpu_req cpu_limit mem_req mem_limit action; do
  # Skip header and non-actionable rows
  [[ "$action" == "NO_CHANGE" ]] && continue
  [[ "$namespace" == "namespace" ]] && continue

  echo ">> Applying: $namespace/$workload ($container) -- $action"

  PATCH=$(cat <<EOF
[
  {"op":"replace","path":"/spec/template/spec/containers/0/resources/requests/cpu","value":"${cpu_req}"},
  {"op":"replace","path":"/spec/template/spec/containers/0/resources/limits/cpu","value":"${cpu_limit}"},
  {"op":"replace","path":"/spec/template/spec/containers/0/resources/requests/memory","value":"${mem_req}"},
  {"op":"replace","path":"/spec/template/spec/containers/0/resources/limits/memory","value":"${mem_limit}"}
]
EOF
)

  if kubectl patch deployment "$workload" -n "$namespace" \
    --type='json' -p="$PATCH" "$DRY_RUN"; then
    ((APPLIED++))
  else
    echo "   FAILED: $namespace/$workload" >&2
    ((FAILED++))
  fi

done < "$REPORT_FILE"

echo ""
echo "Summary: $APPLIED applied, $FAILED failed"
```

**Use VPA in recommendation mode to get continuous suggestions:**

```yaml
apiVersion: autoscaling.k8s.io/v1
kind: VerticalPodAutoscaler
metadata:
  name: api-server-vpa
  namespace: production
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api-server
  updatePolicy:
    updateMode: "Off"          # Recommendation only, no auto-apply
  resourcePolicy:
    containerPolicies:
      - containerName: app
        minAllowed:
          cpu: 100m
          memory: 128Mi
        maxAllowed:
          cpu: 4
          memory: 8Gi
        controlledResources: ["cpu", "memory"]
```

Check VPA recommendations:

```bash
kubectl describe vpa api-server-vpa -n production
```

```
Status:
  Recommendation:
    Container Recommendations:
      Container Name:  app
      Lower Bound:
        Cpu:     200m
        Memory:  512Mi
      Target:
        Cpu:     500m
        Memory:  1536Mi
      Upper Bound:
        Cpu:     1
        Memory:  3Gi
      Uncapped Target:
        Cpu:     480m
        Memory:  1400Mi
```

## Why This Works

1. **Data over intuition.** The 7-day observation window captures weekday and
   weekend patterns, giving a complete picture of actual resource needs.

2. **P95/P99 percentiles.** Using percentiles instead of averages prevents
   under-sizing for bursty workloads. The 20% headroom on P95 for requests
   and 50% above P99 for limits provides a safety margin without massive
   over-provisioning.

3. **Two-sided correction.** Right-sizing is not just about cutting costs.
   Under-provisioned containers cause CPU throttling (latency spikes) and
   OOM kills. Both directions must be addressed.

4. **VPA in Off mode.** Running VPA in recommendation-only mode lets the team
   review changes before applying them. Auto-applying VPA to production
   workloads without review is risky because it causes pod restarts.

## Common Mistakes to Avoid

- **Using averages instead of percentiles.** A container that averages 0.1
  CPU but spikes to 2.0 CPU during deploys will OOM or get throttled if
  right-sized to the average.

- **Right-sizing limits to match requests.** Limits should always be higher
  than requests. Setting them equal removes the burst capability that
  allows containers to handle temporary load spikes.

- **Not accounting for startup overhead.** Some applications (JVM, .NET)
  consume significantly more CPU during startup. Exclude the first 60
  seconds of pod lifetime from the analysis.

- **Applying all changes at once.** Roll out right-sizing changes gradually.
  Batch by namespace, validate metrics for 24 hours, then proceed to the
  next batch.

- **Ignoring HPA interactions.** If Horizontal Pod Autoscaler is active,
  changing resource requests affects the HPA scaling threshold. A CPU-based
  HPA at 80% utilization with requests of 2 cores triggers at 1.6 cores
  actual usage. Dropping requests to 1 core changes the trigger to 0.8
  cores -- the HPA may scale up more aggressively.
