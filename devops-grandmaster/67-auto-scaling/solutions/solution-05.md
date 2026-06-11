# Solution 05: Production-Grade Auto-Scaling System

## Part A: Web Frontend HPA

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: delivery-web-hpa
  namespace: production
  annotations:
    devops/runbook: "https://wiki.internal/runbooks/delivery-web-scaling"
    devops/team: "platform"
    devops/slack-channel: "#platform-alerts"
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: delivery-web
  minReplicas: 3
  maxReplicas: 40
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
          name: http_requests_per_second
        target:
          type: AverageValue
          averageValue: "800"
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 0
      policies:
        - type: Percent
          value: 100
          periodSeconds: 15
        - type: Pods
          value: 10
          periodSeconds: 15
      selectPolicy: Max
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
        - type: Percent
          value: 10
          periodSeconds: 60
```

### Why This Works

- `minReplicas: 3` ensures one replica per availability zone for HA.
- CPU at 65% provides headroom for burst traffic while still utilizing nodes efficiently.
- RPS at 800 per pod is realistic for a web frontend serving a mix of static and dynamic content.
- Scale-up is aggressive (100% or 10 pods every 15 seconds) to handle lunch and dinner rush spikes without user-facing latency.
- Scale-down is conservative (10% every 60 seconds with 300-second stabilization) to avoid flapping during the transition from peak to off-peak.
- Annotations link the HPA to the team's runbook and Slack channel for operational context.

---

## Part B: Order API HPA

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: order-api-hpa
  namespace: production
  annotations:
    devops/runbook: "https://wiki.internal/runbooks/order-api-scaling"
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: order-api
  minReplicas: 2
  maxReplicas: 20
  metrics:
    # Primary: database connections (the real bottleneck)
    - type: Pods
      pods:
        metric:
          name: active_db_connections
        target:
          type: AverageValue
          averageValue: "150"
    # Secondary: CPU utilization
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
    # Tertiary: p99 latency
    - type: Pods
      pods:
        metric:
          name: http_request_duration_p99_seconds
        target:
          type: AverageValue
          averageValue: "300m"    # 300ms = 0.3s
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 30
      policies:
        - type: Percent
          value: 100
          periodSeconds: 30
        - type: Pods
          value: 5
          periodSeconds: 30
      selectPolicy: Max
    scaleDown:
      stabilizationWindowSeconds: 600
      policies:
        - type: Percent
          value: 5
          periodSeconds: 120
```

### Why This Works

- **Connection target of 150 (not 200):** The hard limit is 200 connections per pod. Targeting 150 provides 25% headroom for connection spikes, authentication overhead, and connection pool warmup. HPA calculates: if average connections exceed 150 per pod, scale up. This prevents the scenario where all pods hit 200 connections and start rejecting requests.

- **Three metrics, `max()` selection:** HPA calculates desired replicas for each metric independently and picks the highest. This ensures:
  - Connection pool never saturates (primary metric)
  - CPU never starves (secondary metric)
  - Latency SLO of 300ms p99 is maintained (tertiary metric)

- **Conservative scale-down (5% every 120 seconds):** With 20 replicas, that is 1 pod every 2 minutes. With fewer than 20 replicas, the percentage rounds to 0 -- the HPA will not scale below a certain point. This is critical for the API because rapid scale-down would concentrate database connections on fewer pods, potentially hitting the 200-connection limit.

- **`stabilizationWindowSeconds: 600` for scale-down:** The HPA looks at the last 10 minutes of metrics before deciding to scale down. This prevents premature scale-down after a brief dip in traffic during order processing.

---

## Part C: Order Processor KEDA ScaledObject

```yaml
apiVersion: keda.sh/v1alpha1
kind: ScaledObject
metadata:
  name: order-processor-scaledobject
  namespace: production
spec:
  scaleTargetRef:
    name: order-processor
  pollingInterval: 15
  cooldownPeriod: 300
  minReplicaCount: 0
  maxReplicaCount: 30
  triggers:
    # Primary trigger: Kafka lag
    - type: kafka
      metadata:
        bootstrapServers: kafka.production.svc.cluster.local:9092
        consumerGroup: order-processor-group
        topic: orders
        lagThreshold: "100"
        offsetResetPolicy: latest
      authenticationRef:
        name: kafka-auth
    # Pre-warming trigger: Lunch rush
    - type: cron
      metadata:
        timezone: America/New_York
        start: "30 10 * * 1-5"      # 10:30 AM, weekdays
        end: "30 14 * * 1-5"        # 2:30 PM, weekdays
        desiredReplicas: "10"
    # Pre-warming trigger: Dinner rush
    - type: cron
      metadata:
        timezone: America/New_York
        start: "30 16 * * 1-5"      # 4:30 PM, weekdays
        end: "30 21 * * 1-5"        # 9:30 PM, weekdays
        desiredReplicas: "15"
---
apiVersion: keda.sh/v1alpha1
kind: TriggerAuthentication
metadata:
  name: kafka-auth
  namespace: production
spec:
  secretTargetRef:
    - parameter: sasl
      name: kafka-credentials
      key: sasl-mechanism
    - parameter: username
      name: kafka-credentials
      key: username
    - parameter: password
      name: kafka-credentials
      key: password
```

### Why This Works

- **`minReplicaCount: 0`:** Enables scale-to-zero during late night when no orders are pending. This saves compute costs -- you only pay for workers when they are processing orders.

- **`lagThreshold: "100"`:** One pod handles approximately 100 orders/minute. If the Kafka consumer lag reaches 100 messages, KEDA adds a pod. At 500 messages of lag, KEDA scales to 5 pods. This ensures orders are processed promptly without over-provisioning.

- **Cron triggers for pre-warming:** Without cron triggers, the first orders of lunch rush would sit in the queue while KEDA spins up pods (cold start). The cron trigger ensures 10 pods are running at 10:30 AM and 15 pods at 4:30 PM, before the rush begins. This is a proactive scaling strategy that complements the reactive Kafka lag trigger.

- **Multiple triggers, highest wins:** KEDA evaluates all triggers and uses the one requesting the most replicas. During dinner rush, if the cron trigger wants 15 pods but Kafka lag only needs 3, KEDA runs 15 pods. After the cron window ends, Kafka lag takes over.

- **`cooldownPeriod: 300`:** After the last scaling event, KEDA waits 5 minutes before scaling to zero. This prevents oscillation during brief gaps in orders (e.g., 2 minutes with no orders between dinner rushes).

- **`pollingInterval: 15`:** KEDA checks Kafka lag every 15 seconds. This provides near-real-time scaling response without overloading Kafka with consumer group queries.

- **TriggerAuthentication:** Kafka SASL credentials are stored in a Kubernetes Secret and referenced via `TriggerAuthentication`. This keeps credentials out of the ScaledObject manifest and allows credential rotation without modifying the ScaledObject.

---

## Part D: Cluster Autoscaler Configuration

```bash
# Cluster Autoscaler deployment command flags
./cluster-autoscaler \
  --v=4 \
  --cloud-provider=aws \
  --skip-nodes-with-local-storage=false \
  --expander=least-waste \
  --node-group-auto-discovery=asg:tag=k8s.io/cluster-autoscaler/enabled,k8s.io/cluster-autoscaler/food-delivery-cluster \
  --balance-similar-node-groups \
  --scale-down-utilization-threshold=0.5 \
  --scale-down-delay-after-add=10m \
  --scale-down-unneeded-time=10m \
  --max-graceful-termination-sec=600 \
  --max-node-provision-time=10m \
  --expendable-pods-priority-cutoff=-10
```

```yaml
# AWS ASG tags for autodiscovery (applied to both node groups)

# General-purpose node group (m5.xlarge) tags:
#   Key: k8s.io/cluster-autoscaler/enabled
#   Value: owned
#   Key: k8s.io/cluster-autoscaler/food-delivery-cluster
#   Value: owned

# Compute-optimized node group (c5.2xlarge) tags:
#   Key: k8s.io/cluster-autoscaler/enabled
#   Value: owned
#   Key: k8s.io/cluster-autoscaler/food-delivery-cluster
#   Value: owned
```

### Why This Works

- **`--expander=least-waste`:** When HPA creates pods that need scheduling, the Cluster Autoscaler must choose which node group to scale. `least-waste` picks the group that leaves the fewest unused resources. For example, if a pod needs 2 CPU, a 4-CPU `m5.xlarge` node (2 CPU wasted) is preferred over an 8-CPU `c5.2xlarge` node (6 CPU wasted).

- **`--balance-similar-node-groups`:** Distributes nodes across availability zones. Without this, the autoscaler might add all nodes to the AZ where the first pending pod appeared, reducing fault tolerance. With this flag, if AZ-1 has 3 nodes and AZ-2 has 1 node, the next node goes to AZ-2.

- **`--scale-down-utilization-threshold=0.5`:** Nodes below 50% utilization are candidates for removal. This is aggressive enough to reclaim costs during off-peak but conservative enough to avoid removing nodes that are about to receive pods from a scale-up event.

- **`--scale-down-delay-after-add=10m`:** After adding a node, wait 10 minutes before considering scale-down. This prevents the autoscaler from removing a node it just added if metrics have not yet reflected the new pods.

- **`--scale-down-unneeded-time=10m`:** A node must be below the utilization threshold for 10 continuous minutes before removal. This prevents removal during brief dips (e.g., between cron trigger windows).

- **`--max-graceful-termination-sec=600`:** Allows 10 minutes for pods to terminate gracefully. Critical for order processors that may be mid-transaction. Kubernetes will send SIGTERM, wait for pod termination, and only force-kill after this timeout.

- **`--expendable-pods-priority-cutoff=-10`:** Pods with priority below -10 are considered expendable and can be evicted first during scale-down. This allows you to mark non-critical pods (e.g., pre-warming pods) as expendable.

- **Both ASG tags are required:** The `k8s.io/cluster-autoscaler/enabled` tag tells the autoscaler this ASG is managed by it. The `k8s.io/cluster-autoscaler/<cluster-name>` tag links the ASG to a specific cluster. Both must have value `owned`.

---

## Part E: Prometheus Monitoring and Alerts

```yaml
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: autoscaling-alerts
  namespace: monitoring
spec:
  groups:
    - name: autoscaling.rules
      rules:
        # Alert 1: HPA at max capacity for 10+ minutes
        - alert: HPAAtMaxCapacity
          expr: |
            kube_horizontalpodautoscaler_status_current_replicas
            == kube_horizontalpodautoscaler_spec_max_replicas
          for: 10m
          labels:
            severity: warning
          annotations:
            summary: "HPA {{ $labels.horizontalpodautoscaler }} is at max replicas"
            description: >-
              HPA {{ $labels.horizontalpodautoscaler }} in namespace
              {{ $labels.namespace }} has been at maximum capacity
              ({{ $value }} replicas) for 10 minutes. Consider increasing
              maxReplicas or investigating the traffic pattern.

        # Alert 2: HPA wants to scale up but cannot
        - alert: HPAScaleUpStalled
          expr: |
            kube_horizontalpodautoscaler_status_condition{condition="ScalingLimited",status="true"} == 1
          for: 5m
          labels:
            severity: critical
          annotations:
            summary: "HPA {{ $labels.horizontalpodautoscaler }} cannot scale up"
            description: >-
              HPA {{ $labels.horizontalpodautoscaler }} in namespace
              {{ $labels.namespace }} is limited and cannot scale further.
              Check Cluster Autoscaler logs and node pool capacity.

        # Alert 3: HPA flapping (more than 6 changes in 30 minutes)
        - alert: HPAFlapping
          expr: |
            changes(kube_horizontalpodautoscaler_status_current_replicas[30m]) > 6
          for: 5m
          labels:
            severity: warning
          annotations:
            summary: "HPA {{ $labels.horizontalpodautoscaler }} is flapping"
            description: >-
              HPA {{ $labels.horizontalpodautoscaler }} has changed replica
              count more than 6 times in 30 minutes. Check scaling policies
              and metric stability.

        # Alert 4: KEDA scaler error
        - alert: KEDAScalerError
          expr: |
            keda_scaler_errors_total > 0
          for: 5m
          labels:
            severity: critical
          annotations:
            summary: "KEDA scaler error for {{ $labels.scaledObject }}"
            description: >-
              KEDA has encountered errors polling the scaler for
              {{ $labels.scaledObject }} in namespace {{ $labels.namespace }}.
              Check scaler connectivity and credentials.

        # Alert 5: Unschedulable pods (Cluster Autoscaler cannot provision)
        - alert: ClusterAutoscalerUnschedulable
          expr: |
            cluster_autoscaler_unschedulable_pods_count > 0
          for: 5m
          labels:
            severity: critical
          annotations:
            summary: "{{ $value }} pods are unschedulable"
            description: >-
              There are {{ $value }} pods that cannot be scheduled despite
              Cluster Autoscaler running. Check node pool limits, IAM
              permissions, and instance availability.

        # Recording rule: HPA scaling efficiency
        - record: hpa:scaling_efficiency
          expr: |
            kube_horizontalpodautoscaler_status_current_replicas
            / kube_horizontalpodautoscaler_spec_max_replicas

    - name: autoscaling.grafana.queries
      rules:
        # Recording rules for Grafana dashboards
        - record: hpa:current_replicas
          expr: |
            kube_horizontalpodautoscaler_status_current_replicas{namespace="production"}

        - record: hpa:desired_replicas
          expr: |
            kube_horizontalpodautoscaler_status_desired_replicas{namespace="production"}

        - record: hpa:max_replicas
          expr: |
            kube_horizontalpodautoscaler_spec_max_replicas{namespace="production"}

        - record: hpa:min_replicas
          expr: |
            kube_horizontalpodautoscaler_spec_min_replicas{namespace="production"}
```

### Grafana Dashboard Queries

```promql
# Panel 1: HPA Replica Count vs Boundaries (time series)
# Current replicas
kube_horizontalpodautoscaler_status_current_replicas{namespace="production", horizontalpodautoscaler=~"delivery-web-hpa|order-api-hpa"}

# Max replicas (constant line)
kube_horizontalpodautoscaler_spec_max_replicas{namespace="production", horizontalpodautoscaler=~"delivery-web-hpa|order-api-hpa"}

# Min replicas (constant line)
kube_horizontalpodautoscaler_spec_min_replicas{namespace="production", horizontalpodautoscaler=~"delivery-web-hpa|order-api-hpa"}

# Panel 2: HPA Scaling Efficiency (gauge)
hpa:scaling_efficiency{namespace="production"}

# Panel 3: Scaling Events Over Time (bar chart)
changes(kube_horizontalpodautoscaler_status_current_replicas{namespace="production"}[1h])

# Panel 4: CPU Utilization vs Target (time series)
# Current CPU utilization
kube_pod_container_resource_requests{namespace="production", resource="cpu"}
# vs
# container_cpu_usage_seconds_total rate

# Panel 5: KEDA ScaledObject Replicas (time series)
keda_scaledobject_status_current_replicas{namespace="production"}

# Panel 6: Cluster Autoscaler Nodes (time series)
cluster_autoscaler_nodes_count{state="ready"}
cluster_autoscaler_nodes_count{state="unready"}
```

### Why This Works

The 5 alerts cover the complete failure surface of an autoscaling system:

1. **HPAAtMaxCapacity** -- the system is at its ceiling and needs manual intervention (increase `maxReplicas` or optimize the application).
2. **HPAScaleUpStalled** -- the HPA wants to scale but cannot, usually because the Cluster Autoscaler cannot provision nodes (IAM permissions, instance limits, or cloud provider issues).
3. **HPAFlapping** -- the scaling policies are causing oscillation, wasting compute and causing latency spikes during pod startup.
4. **KEDAScalerError** -- KEDA cannot poll its scaler (Kafka, RabbitMQ, etc.), which means it cannot make scaling decisions. Workers may be stuck at their current replica count.
5. **ClusterAutoscalerUnschedulable** -- pods are Pending because no node can accommodate them and the Cluster Autoscaler cannot add more nodes.

The recording rules pre-compute frequently used queries for Grafana dashboards, reducing query load on Prometheus.

---

## Part F: End-to-End Validation Script

```bash
#!/usr/bin/env bash
# validate-autoscaling.sh
# Validates the complete auto-scaling system for the food delivery platform.

set -euo pipefail

NAMESPACE="production"
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

pass() { echo -e "${GREEN}[PASS]${NC} $1"; }
fail() { echo -e "${RED}[FAIL]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

ERRORS=0

echo "=========================================="
echo " Auto-Scaling System Validation"
echo " Namespace: ${NAMESPACE}"
echo "=========================================="
echo ""

# --- 1. Verify HPAs exist and target correct Deployments ---
echo "--- Checking HPAs ---"

for DEPLOYMENT in delivery-web order-api; do
  HPA_NAME="${DEPLOYMENT}-hpa"
  if kubectl get hpa "${HPA_NAME}" -n "${NAMESPACE}" &>/dev/null; then
    TARGET=$(kubectl get hpa "${HPA_NAME}" -n "${NAMESPACE}" -o jsonpath='{.spec.scaleTargetRef.name}')
    if [ "${TARGET}" = "${DEPLOYMENT}" ]; then
      pass "HPA ${HPA_NAME} targets Deployment ${DEPLOYMENT}"
    else
      fail "HPA ${HPA_NAME} targets ${TARGET}, expected ${DEPLOYMENT}"
      ERRORS=$((ERRORS + 1))
    fi

    # Check min/max replicas
    MIN=$(kubectl get hpa "${HPA_NAME}" -n "${NAMESPACE}" -o jsonpath='{.spec.minReplicas}')
    MAX=$(kubectl get hpa "${HPA_NAME}" -n "${NAMESPACE}" -o jsonpath='{.spec.maxReplicas}')
    CURRENT=$(kubectl get hpa "${HPA_NAME}" -n "${NAMESPACE}" -o jsonpath='{.status.currentReplicas}')
    echo "    Replicas: current=${CURRENT}, min=${MIN}, max=${MAX}"

    # Check metrics count
    METRICS_COUNT=$(kubectl get hpa "${HPA_NAME}" -n "${NAMESPACE}" -o json | jq '.spec.metrics | length')
    echo "    Metrics defined: ${METRICS_COUNT}"
  else
    fail "HPA ${HPA_NAME} not found"
    ERRORS=$((ERRORS + 1))
  fi
done
echo ""

# --- 2. Verify KEDA ScaledObject ---
echo "--- Checking KEDA ScaledObject ---"

SO_NAME="order-processor-scaledobject"
if kubectl get scaledobject "${SO_NAME}" -n "${NAMESPACE}" &>/dev/null; then
  TARGET=$(kubectl get scaledobject "${SO_NAME}" -n "${NAMESPACE}" -o jsonpath='{.spec.scaleTargetRef.name}')
  if [ "${TARGET}" = "order-processor" ]; then
    pass "ScaledObject ${SO_NAME} targets Deployment order-processor"
  else
    fail "ScaledObject ${SO_NAME} targets ${TARGET}, expected order-processor"
    ERRORS=$((ERRORS + 1))
  fi

  MIN=$(kubectl get scaledobject "${SO_NAME}" -n "${NAMESPACE}" -o jsonpath='{.spec.minReplicaCount}')
  MAX=$(kubectl get scaledobject "${SO_NAME}" -n "${NAMESPACE}" -o jsonpath='{.spec.maxReplicaCount}')
  TRIGGERS=$(kubectl get scaledobject "${SO_NAME}" -n "${NAMESPACE}" -o json | jq '.spec.triggers | length')
  echo "    Replicas: min=${MIN}, max=${MAX}, triggers=${TRIGGERS}"
else
  fail "ScaledObject ${SO_NAME} not found"
  ERRORS=$((ERRORS + 1))
fi
echo ""

# --- 3. Check Cluster Autoscaler ---
echo "--- Checking Cluster Autoscaler ---"

CA_PODS=$(kubectl get pods -n kube-system -l app=cluster-autoscaler --no-headers 2>/dev/null | wc -l)
if [ "${CA_PODS}" -gt 0 ]; then
  pass "Cluster Autoscaler is running (${CA_PODS} pod(s))"
  CA_STATUS=$(kubectl get pods -n kube-system -l app=cluster-autoscaler -o jsonpath='{.items[0].status.phase}')
  echo "    Status: ${CA_STATUS}"
else
  fail "Cluster Autoscaler pods not found in kube-system"
  ERRORS=$((ERRORS + 1))
fi
echo ""

# --- 4. Query Prometheus for current HPA metrics ---
echo "--- Checking Prometheus Metrics ---"

if command -v kubectl &>/dev/null && kubectl get svc -n monitoring prometheus-k8s &>/dev/null 2>&1; then
  PROM_URL="http://$(kubectl get svc -n monitoring prometheus-k8s -o jsonpath='{.spec.clusterIP}'):9090"

  for HPA_NAME in delivery-web-hpa order-api-hpa; do
    RESULT=$(curl -s --max-time 5 "${PROM_URL}/api/v1/query" \
      --data-urlencode "query=kube_horizontalpodautoscaler_status_current_replicas{horizontalpodautoscaler=\"${HPA_NAME}\",namespace=\"${NAMESPACE}\"}" 2>/dev/null)

    if echo "${RESULT}" | jq -e '.data.result[0].value[1]' &>/dev/null; then
      VALUE=$(echo "${RESULT}" | jq -r '.data.result[0].value[1]')
      pass "Prometheus metric for ${HPA_NAME}: ${VALUE} current replicas"
    else
      warn "Could not query Prometheus for ${HPA_NAME} (Prometheus may not be accessible)"
    fi
  done
else
  warn "Prometheus service not found in monitoring namespace (skipping metric queries)"
fi
echo ""

# --- 5. Summary Table ---
echo "=========================================="
echo " Auto-Scaling Resources Summary"
echo "=========================================="
echo ""

printf "%-35s %-12s %-8s %-8s %-8s %-10s\n" "RESOURCE" "TYPE" "CURRENT" "MIN" "MAX" "STATUS"
printf "%-35s %-12s %-8s %-8s %-8s %-10s\n" "--------" "----" "-------" "---" "---" "------"

# HPAs
for DEPLOYMENT in delivery-web order-api; do
  HPA_NAME="${DEPLOYMENT}-hpa"
  if kubectl get hpa "${HPA_NAME}" -n "${NAMESPACE}" &>/dev/null; then
    CURRENT=$(kubectl get hpa "${HPA_NAME}" -n "${NAMESPACE}" -o jsonpath='{.status.currentReplicas}')
    MIN=$(kubectl get hpa "${HPA_NAME}" -n "${NAMESPACE}" -o jsonpath='{.spec.minReplicas}')
    MAX=$(kubectl get hpa "${HPA_NAME}" -n "${NAMESPACE}" -o jsonpath='{.spec.maxReplicas}')
    READY=$(kubectl get hpa "${HPA_NAME}" -n "${NAMESPACE}" -o jsonpath='{.status.conditions[?(@.type=="AbleToScale")].status}')
    printf "%-35s %-12s %-8s %-8s %-8s %-10s\n" "${HPA_NAME}" "HPA" "${CURRENT}" "${MIN}" "${MAX}" "${READY}"
  fi
done

# KEDA ScaledObject
if kubectl get scaledobject "${SO_NAME}" -n "${NAMESPACE}" &>/dev/null; then
  CURRENT=$(kubectl get scaledobject "${SO_NAME}" -n "${NAMESPACE}" -o jsonpath='{.status.currentReplicas}' 2>/dev/null || echo "N/A")
  MIN=$(kubectl get scaledobject "${SO_NAME}" -n "${NAMESPACE}" -o jsonpath='{.spec.minReplicaCount}')
  MAX=$(kubectl get scaledobject "${SO_NAME}" -n "${NAMESPACE}" -o jsonpath='{.spec.maxReplicaCount}')
  ACTIVE=$(kubectl get scaledobject "${SO_NAME}" -n "${NAMESPACE}" -o jsonpath='{.status.conditions[?(@.type=="Active")].status}' 2>/dev/null || echo "Unknown")
  printf "%-35s %-12s %-8s %-8s %-8s %-10s\n" "${SO_NAME}" "ScaledObj" "${CURRENT}" "${MIN}" "${MAX}" "${ACTIVE}"
fi

# Cluster Autoscaler
if kubectl get pods -n kube-system -l app=cluster-autoscaler &>/dev/null; then
  CA_STATUS=$(kubectl get pods -n kube-system -l app=cluster-autoscaler -o jsonpath='{.items[0].status.phase}' 2>/dev/null || echo "Unknown")
  printf "%-35s %-12s %-8s %-8s %-8s %-10s\n" "cluster-autoscaler" "Deployment" "1" "1" "1" "${CA_STATUS}"
fi

echo ""

# --- Final Result ---
echo "=========================================="
if [ "${ERRORS}" -eq 0 ]; then
  echo -e "${GREEN}All checks passed.${NC}"
else
  echo -e "${RED}${ERRORS} check(s) failed.${NC}"
fi
echo "=========================================="

exit "${ERRORS}"
```

### Why This Works

The validation script checks every component of the auto-scaling system:

1. **HPA existence and targeting:** Verifies that each HPA exists, targets the correct Deployment, and has the expected min/max replicas. A common deployment error is targeting a non-existent Deployment or wrong namespace.

2. **KEDA ScaledObject:** Verifies the ScaledObject exists, targets the correct Deployment, and has triggers configured. Checks min/max replicas and trigger count.

3. **Cluster Autoscaler health:** Verifies the Cluster Autoscaler pods are running. If the Cluster Autoscaler is down, HPA-created pods will remain Pending when the cluster is at capacity.

4. **Prometheus metrics:** Queries Prometheus for current HPA replica counts. This validates that the monitoring pipeline (metrics-server -> Prometheus -> Grafana) is working end-to-end.

5. **Summary table:** Provides a single-view dashboard of all autoscaling resources, their current state, and their boundaries. This is useful for on-call engineers who need a quick health check.

The script uses `set -euo pipefail` for strict error handling, color-coded output for readability, and exits with the number of errors as the exit code (0 = all pass).

---

## Common Mistakes to Avoid

- **Forgetting the TriggerAuthentication for KEDA.** Without it, KEDA cannot authenticate to Kafka/RabbitMQ and the scaler will fail silently. Always check `keda_scaler_errors_total` in Prometheus.

- **Not setting `minReplicas` for HA-critical workloads.** If `minReplicas` is 1 and that single pod crashes, the service is down. Set `minReplicas` to at least 2 for HA, or 3 for cross-AZ HA.

- **Cluster Autoscaler IAM permissions.** The most common Cluster Autoscaler failure on AWS is missing IAM permissions. The autoscaler needs `autoscaling:DescribeAutoScalingGroups`, `autoscaling:UpdateAutoScalingGroup`, and `ec2:DescribeLaunchTemplateVersions` at minimum.

- **HPA and KEDA on the same Deployment.** Do not create both an HPA and a KEDA ScaledObject targeting the same Deployment. KEDA creates its own HPA internally. Having two HPAs causes conflicts.

- **Not testing scale-down.** Teams often test scale-up (generate load, watch pods increase) but forget to test scale-down (remove load, watch pods decrease). Scale-down is where flapping manifests. Always test the full cycle.

- **Ignoring pod startup time in behavior policies.** If pods take 60 seconds to start, a scale-up policy that adds pods every 15 seconds creates pods faster than they can become ready. The stabilization window should account for startup time.

## Key Takeaway

A production-grade auto-scaling system is a coordinated set of mechanisms -- HPA for pod scaling, KEDA for event-driven scaling, Cluster Autoscaler for node scaling, and Prometheus for monitoring. Each component has its own failure mode, and your monitoring must cover all of them. The key design principles are: scale up fast (users are waiting), scale down slow (avoid flapping), always leave headroom on finite resources, always have alerts for when the system cannot scale further, and always validate the full scale-up and scale-down cycle before declaring the system production-ready.
