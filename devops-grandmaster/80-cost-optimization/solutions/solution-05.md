# Solution 05: FinOps Implementation

## Problem Statement

Implement a comprehensive FinOps framework for a Kubernetes-based
infrastructure. This includes enforcing resource tagging via admission
controllers, building a cost allocation system, scheduling off-hours scaling,
cleaning up unused resources, and assessing FinOps maturity.

## Complete Solution

### Part 1: Gatekeeper ConstraintTemplate for Tagging Enforcement

```yaml
apiVersion: templates.gatekeeper.sh/v1
kind: ConstraintTemplate
metadata:
  name: k8srequiredtags
  annotations:
    description: >-
      Requires that all Deployments, StatefulSets, and Jobs
      carry mandatory cost-allocation tags as labels.
spec:
  crd:
    spec:
      names:
        kind: K8sRequiredTags
      validation:
        openAPIV3Schema:
          type: object
          properties:
            tags:
              type: array
              items:
                type: string
              description: "Required label keys for cost allocation"
            exemptNamespaces:
              type: array
              items:
                type: string
              description: "Namespaces excluded from tagging policy"
  targets:
    - target: admission.k8s.gatekeeper.sh
      rego: |
        package k8srequiredtags

        violation[{"msg": msg}] {
          # Check if the resource is in an exempt namespace
          not is_exempt_namespace

          # Get the required tags from parameters
          required_tag := input.parameters.tags[_]

          # Check if the required tag exists in labels
          not has_tag(required_tag)

          msg := sprintf(
            "Resource %v/%v in namespace %v is missing required tag '%v'. All resources must have: %v",
            [
              input.review.object.kind,
              input.review.object.metadata.name,
              input.review.object.metadata.namespace,
              required_tag,
              concat(", ", input.parameters.tags)
            ]
          )
        }

        has_tag(tag) {
          input.review.object.metadata.labels[tag]
        }

        is_exempt_namespace {
          ns := input.parameters.exemptNamespaces[_]
          input.review.object.metadata.namespace == ns
        }

        # Also check pod template labels for Deployments/StatefulSets/Jobs
        violation[{"msg": msg}] {
          not is_exempt_namespace
          required_tag := input.parameters.tags[_]
          has_pod_template
          not has_pod_template_tag(required_tag)

          msg := sprintf(
            "Resource %v/%v in namespace %v is missing required tag '%v' in pod template labels",
            [
              input.review.object.kind,
              input.review.object.metadata.name,
              input.review.object.metadata.namespace,
              required_tag
            ]
          )
        }

        has_pod_template {
          input.review.object.spec.template.metadata.labels
        }

        has_pod_template_tag(tag) {
          input.review.object.spec.template.metadata.labels[tag]
        }
---
# Apply the constraint to require specific cost tags
apiVersion: constraints.gatekeeper.sh/v1beta1
kind: K8sRequiredTags
metadata:
  name: require-cost-tags
spec:
  enforcementAction: deny
  match:
    kinds:
      - apiGroups: ["apps"]
        kinds: ["Deployment", "StatefulSet", "DaemonSet"]
      - apiGroups: ["batch"]
        kinds: ["Job", "CronJob"]
    excludedNamespaces:
      - kube-system
      - kube-public
      - gatekeeper-system
  parameters:
    tags:
      - "cost-center"
      - "team"
      - "environment"
      - "app"
    exemptNamespaces:
      - kube-system
      - gatekeeper-system
```

**Test the policy:**

```bash
# This deployment will be REJECTED (missing cost-center label)
kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: test-app
  namespace: production
  labels:
    app: test-app
    team: backend
    environment: production
    # Missing: cost-center
spec:
  replicas: 1
  selector:
    matchLabels:
      app: test-app
  template:
    metadata:
      labels:
        app: test-app
    spec:
      containers:
        - name: app
          image: nginx:latest
EOF
# Error: admission webhook "validation.gatekeeper.sh" denied the request:
# Resource Deployment/test-app in namespace production is missing required tag 'cost-center'

# This deployment will be ACCEPTED
kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: test-app
  namespace: production
  labels:
    app: test-app
    team: backend
    environment: production
    cost-center: CC-1234
spec:
  replicas: 1
  selector:
    matchLabels:
      app: test-app
  template:
    metadata:
      labels:
        app: test-app
        team: backend
        environment: production
        cost-center: CC-1234
    spec:
      containers:
        - name: app
          image: nginx:latest
EOF
# deployment.apps/test-app created
```

### Part 2: Cost Allocation Python Class

```python
#!/usr/bin/env python3
"""
CostAllocator -- Attribute Kubernetes cluster costs to teams using labels
and Prometheus metrics.
"""

from dataclasses import dataclass, field
from typing import Optional
from datetime import datetime, timedelta
import json


@dataclass
class ResourceUsage:
    """Resource usage for a single workload."""
    namespace: str
    pod_name: str
    labels: dict[str, str]
    cpu_request_cores: float
    cpu_usage_cores: float
    memory_request_bytes: int
    memory_usage_bytes: int
    gpu_request: int
    storage_bytes: int
    hours_running: float


@dataclass
class TeamAllocation:
    """Aggregated cost allocation for a team."""
    team: str
    cost_center: str
    environment: str
    namespaces: list[str] = field(default_factory=list)
    total_cpu_cores: float = 0.0
    total_memory_gb: float = 0.0
    total_gpu: int = 0
    total_storage_gb: float = 0.0
    compute_cost: float = 0.0
    storage_cost: float = 0.0
    network_cost: float = 0.0
    total_cost: float = 0.0
    pct_of_cluster: float = 0.0

    def to_dict(self) -> dict:
        return {
            "team": self.team,
            "cost_center": self.cost_center,
            "environment": self.environment,
            "namespaces": self.namespaces,
            "total_cpu_cores": round(self.total_cpu_cores, 2),
            "total_memory_gb": round(self.total_memory_gb, 2),
            "total_gpu": self.total_gpu,
            "total_storage_gb": round(self.total_storage_gb, 2),
            "compute_cost": round(self.compute_cost, 2),
            "storage_cost": round(self.storage_cost, 2),
            "network_cost": round(self.network_cost, 2),
            "total_cost": round(self.total_cost, 2),
            "pct_of_cluster": round(self.pct_of_cluster, 1),
        }


# Cluster hourly costs (would come from cloud billing API in production)
CLUSTER_COSTS = {
    "cpu_per_core_hour": 0.0425,      # $/core/hour
    "memory_per_gb_hour": 0.0058,     # $/GB/hour
    "gpu_per_unit_hour": 1.20,        # $/GPU/hour
    "storage_per_gb_month": 0.10,     # $/GB/month
    "network_per_gb": 0.09,           # $/GB transferred
    "cluster_monthly_fixed": 2400.00, # Control plane, LBs, etc.
}


class CostAllocator:
    """
    Attribute cluster costs to teams based on resource usage and labels.

    Usage:
        allocator = CostAllocator()
        allocator.load_usage_from_prometheus(prometheus_url)
        # OR
        allocator.load_usage_from_csv("usage-export.csv")
        allocations = allocator.allocate()
        allocator.print_report(allocations)
    """

    def __init__(self, costs: Optional[dict] = None):
        self.costs = costs or CLUSTER_COSTS
        self.usages: list[ResourceUsage] = []
        self.allocations: dict[str, TeamAllocation] = {}

    def load_usage_from_csv(self, path: str) -> None:
        """Load resource usage data from a CSV export."""
        import csv
        with open(path, "r") as f:
            reader = csv.DictReader(f)
            for row in reader:
                usage = ResourceUsage(
                    namespace=row["namespace"],
                    pod_name=row["pod_name"],
                    labels=json.loads(row.get("labels", "{}")),
                    cpu_request_cores=float(row["cpu_request_cores"]),
                    cpu_usage_cores=float(row["cpu_usage_cores"]),
                    memory_request_bytes=int(row["memory_request_bytes"]),
                    memory_usage_bytes=int(row["memory_usage_bytes"]),
                    gpu_request=int(row.get("gpu_request", 0)),
                    storage_bytes=int(row.get("storage_bytes", 0)),
                    hours_running=float(row.get("hours_running", 730)),
                )
                self.usages.append(usage)

    def load_usage_from_prometheus(self, prometheus_url: str) -> None:
        """
        Query Prometheus for actual resource usage data.
        This is a template -- adjust queries for your environment.
        """
        import requests

        # Query: CPU requests by namespace and pod
        query = """
        sum by (namespace, pod, label_team, label_cost_center, label_environment, label_app) (
          kube_pod_container_resource_requests{resource="cpu"}
        )
        """
        # In production, you would execute multiple queries and join the results.
        # This is a simplified example showing the pattern.
        response = requests.get(
            f"{prometheus_url}/api/v1/query",
            params={"query": query.strip()},
        )
        data = response.json()

        for result in data.get("data", {}).get("result", []):
            metric = result["metric"]
            labels = {
                "team": metric.get("label_team", "unknown"),
                "cost-center": metric.get("label_cost_center", "unknown"),
                "environment": metric.get("label_environment", "unknown"),
                "app": metric.get("label_app", "unknown"),
            }
            usage = ResourceUsage(
                namespace=metric.get("namespace", "unknown"),
                pod_name=metric.get("pod", "unknown"),
                labels=labels,
                cpu_request_cores=float(result["value"][1]),
                cpu_usage_cores=0.0,   # Filled by separate query
                memory_request_bytes=0,
                memory_usage_bytes=0,
                gpu_request=0,
                storage_bytes=0,
                hours_running=730,
            )
            self.usages.append(usage)

    def allocate(self) -> dict[str, TeamAllocation]:
        """Compute cost allocation grouped by team."""
        self.allocations = {}

        # Group by team
        team_groups: dict[str, list[ResourceUsage]] = {}
        for usage in self.usages:
            team = usage.labels.get("team", "untagged")
            team_groups.setdefault(team, []).append(usage)

        # Aggregate per team
        for team, usages in team_groups.items():
            first = usages[0]
            alloc = TeamAllocation(
                team=team,
                cost_center=first.labels.get("cost-center", "unknown"),
                environment=first.labels.get("environment", "unknown"),
                namespaces=list(set(u.namespace for u in usages)),
            )

            for u in usages:
                alloc.total_cpu_cores += u.cpu_request_cores
                alloc.total_memory_gb += u.memory_request_bytes / (1024 ** 3)
                alloc.total_gpu += u.gpu_request
                alloc.total_storage_gb += u.storage_bytes / (1024 ** 3)

                hours = u.hours_running
                alloc.compute_cost += (
                    u.cpu_request_cores
                    * self.costs["cpu_per_core_hour"]
                    * hours
                )
                alloc.compute_cost += (
                    (u.memory_request_bytes / (1024 ** 3))
                    * self.costs["memory_per_gb_hour"]
                    * hours
                )
                alloc.compute_cost += (
                    u.gpu_request
                    * self.costs["gpu_per_unit_hour"]
                    * hours
                )
                alloc.storage_cost += (
                    (u.storage_bytes / (1024 ** 3))
                    * self.costs["storage_per_gb_month"]
                )

            alloc.total_cost = alloc.compute_cost + alloc.storage_cost + alloc.network_cost
            self.allocations[team] = alloc

        # Calculate percentages
        grand_total = sum(a.total_cost for a in self.allocations.values())
        if grand_total > 0:
            for alloc in self.allocations.values():
                alloc.pct_of_cluster = (alloc.total_cost / grand_total) * 100

        return self.allocations

    def print_report(self, allocations: Optional[dict] = None) -> str:
        """Generate a formatted cost allocation report."""
        allocs = allocations or self.allocations
        if not allocs:
            return "No allocations computed. Call allocate() first."

        lines = []
        lines.append("=" * 95)
        lines.append("  COST ALLOCATION REPORT BY TEAM")
        lines.append(f"  Period: {datetime.now().strftime('%B %Y')}")
        lines.append("=" * 95)
        lines.append("")
        lines.append(
            f"{'Team':<18} {'CC':<10} {'Env':<10} {'CPU':>8} {'Mem(GB)':>8} "
            f"{'GPU':>5} {'Compute':>12} {'Storage':>10} {'Total':>12} {'%':>6}"
        )
        lines.append("-" * 95)

        sorted_allocs = sorted(
            allocs.values(), key=lambda a: a.total_cost, reverse=True
        )

        grand_total = sum(a.total_cost for a in sorted_allocs)

        for a in sorted_allocs:
            lines.append(
                f"{a.team:<18} {a.cost_center:<10} {a.environment:<10} "
                f"{a.total_cpu_cores:>7.1f} {a.total_memory_gb:>7.1f} "
                f"{a.total_gpu:>5} ${a.compute_cost:>10,.2f} "
                f"${a.storage_cost:>8,.2f} ${a.total_cost:>10,.2f} "
                f"{a.pct_of_cluster:>5.1f}%"
            )

        lines.append("-" * 95)
        lines.append(
            f"{'TOTAL':<18} {'':<10} {'':<10} "
            f"{sum(a.total_cpu_cores for a in sorted_allocs):>7.1f} "
            f"{sum(a.total_memory_gb for a in sorted_allocs):>7.1f} "
            f"{sum(a.total_gpu for a in sorted_allocs):>5} "
            f"${sum(a.compute_cost for a in sorted_allocs):>10,.2f} "
            f"${sum(a.storage_cost for a in sorted_allocs):>8,.2f} "
            f"${grand_total:>10,.2f} "
            f"{'100.0':>5}%"
        )

        lines.append("")
        lines.append("  Untagged Resources: ${:,.2f}".format(
            allocs.get("untagged", TeamAllocation(
                team="untagged", cost_center="", environment=""
            )).total_cost
        ))
        lines.append("=" * 95)

        return "\n".join(lines)

    def export_csv(self, path: str) -> None:
        """Export allocations to CSV."""
        import csv
        with open(path, "w", newline="") as f:
            writer = csv.DictWriter(f, fieldnames=[
                "team", "cost_center", "environment", "namespaces",
                "total_cpu_cores", "total_memory_gb", "total_gpu",
                "total_storage_gb", "compute_cost", "storage_cost",
                "network_cost", "total_cost", "pct_of_cluster",
            ])
            writer.writeheader()
            for alloc in self.allocations.values():
                d = alloc.to_dict()
                d["namespaces"] = ";".join(d["namespaces"])
                writer.writerow(d)
```

### Part 3: CronJobs for Off-Hours Scaling

Scale down non-production workloads during nights and weekends to save costs:

```yaml
# Scale DOWN at 8 PM UTC on weekdays
apiVersion: batch/v1
kind: CronJob
metadata:
  name: scale-down-evening
  namespace: kube-system
  labels:
    app: cost-scheduler
    schedule: evening
spec:
  schedule: "0 20 * * 1-5"  # Mon-Fri at 8 PM UTC
  timeZone: "America/New_York"
  concurrencyPolicy: Forbid
  successfulJobsHistoryLimit: 3
  failedJobsHistoryLimit: 3
  jobTemplate:
    spec:
      backoffLimit: 2
      activeDeadlineSeconds: 300
      template:
        spec:
          serviceAccountName: cost-scheduler
          restartPolicy: OnFailure
          containers:
            - name: scaler
              image: bitnami/kubectl:1.29
              command:
                - /bin/bash
                - -c
                - |
                  set -euo echo
                  echo "=== Evening Scale-Down $(date -u) ==="

                  # Scale all deployments in non-prod namespaces
                  NAMESPACES="staging development qa preview"

                  for ns in $NAMESPACES; do
                    echo "--- Namespace: $ns ---"
                    for deploy in $(kubectl get deploy -n "$ns" -o name 2>/dev/null); do
                      CURRENT=$(kubectl get "$deploy" -n "$ns" -o jsonpath='{.spec.replicas}')
                      ANNOTATION=$(kubectl get "$deploy" -n "$ns" \
                        -o jsonpath='{.metadata.annotations.cost-scheduler/original-replicas}')

                      # Save original replica count if not already saved
                      if [ -z "$ANNOTATION" ]; then
                        kubectl annotate "$deploy" -n "$ns" \
                          "cost-scheduler/original-replicas=$CURRENT" \
                          --overwrite
                      fi

                      # Scale to zero
                      kubectl scale "$deploy" -n "$ns" --replicas=0
                      echo "  $deploy: $CURRENT -> 0"
                    done
                  done

                  # Scale StatefulSets too
                  for ns in $NAMESPACES; do
                    for sts in $(kubectl get statefulset -n "$ns" -o name 2>/dev/null); do
                      CURRENT=$(kubectl get "$sts" -n "$ns" -o jsonpath='{.spec.replicas}')
                      kubectl annotate "$sts" -n "$ns" \
                        "cost-scheduler/original-replicas=$CURRENT" \
                        --overwrite 2>/dev/null || true
                      kubectl scale "$sts" -n "$ns" --replicas=0
                      echo "  $sts: $CURRENT -> 0"
                    done
                  done

                  echo "=== Scale-down complete ==="
---
# Scale UP at 7 AM UTC on weekdays
apiVersion: batch/v1
kind: CronJob
metadata:
  name: scale-up-morning
  namespace: kube-system
  labels:
    app: cost-scheduler
    schedule: morning
spec:
  schedule: "0 7 * * 1-5"  # Mon-Fri at 7 AM UTC
  timeZone: "America/New_York"
  concurrencyPolicy: Forbid
  successfulJobsHistoryLimit: 3
  failedJobsHistoryLimit: 3
  jobTemplate:
    spec:
      backoffLimit: 2
      activeDeadlineSeconds: 300
      template:
        spec:
          serviceAccountName: cost-scheduler
          restartPolicy: OnFailure
          containers:
            - name: scaler
              image: bitnami/kubectl:1.29
              command:
                - /bin/bash
                - -c
                - |
                  set -euo echo
                  echo "=== Morning Scale-Up $(date -u) ==="

                  NAMESPACES="staging development qa preview"

                  for ns in $NAMESPACES; do
                    echo "--- Namespace: $ns ---"
                    for deploy in $(kubectl get deploy -n "$ns" -o name 2>/dev/null); do
                      ORIGINAL=$(kubectl get "$deploy" -n "$ns" \
                        -o jsonpath='{.metadata.annotations.cost-scheduler/original-replicas}')

                      if [ -n "$ORIGINAL" ] && [ "$ORIGINAL" -gt 0 ]; then
                        kubectl scale "$deploy" -n "$ns" --replicas="$ORIGINAL"
                        kubectl annotate "$deploy" -n "$ns" \
                          "cost-scheduler/original-replicas-" 2>/dev/null || true
                        echo "  $deploy: 0 -> $ORIGINAL"
                      fi
                    done
                  done

                  for ns in $NAMESPACES; do
                    for sts in $(kubectl get statefulset -n "$ns" -o name 2>/dev/null); do
                      ORIGINAL=$(kubectl get "$sts" -n "$ns" \
                        -o jsonpath='{.metadata.annotations.cost-scheduler/original-replicas}')

                      if [ -n "$ORIGINAL" ] && [ "$ORIGINAL" -gt 0 ]; then
                        kubectl scale "$sts" -n "$ns" --replicas="$ORIGINAL"
                        kubectl annotate "$sts" -n "$ns" \
                          "cost-scheduler/original-replicas-" 2>/dev/null || true
                        echo "  $sts: 0 -> $ORIGINAL"
                      fi
                    done
                  done

                  echo "=== Scale-up complete ==="
---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: cost-scheduler
  namespace: kube-system
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: cost-scheduler
rules:
  - apiGroups: ["apps"]
    resources: ["deployments", "statefulsets", "deployments/scale", "statefulsets/scale"]
    verbs: ["get", "list", "patch", "update"]
  - apiGroups: [""]
    resources: ["pods"]
    verbs: ["get", "list"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: cost-scheduler
subjects:
  - kind: ServiceAccount
    name: cost-scheduler
    namespace: kube-system
roleRef:
  kind: ClusterRole
  name: cost-scheduler
  apiGroup: rbac.authorization.k8s.io
```

**Savings estimate from off-hours scaling:**

```
Off-Hours Savings Calculation
================================

Non-production namespaces:
  staging:      35 deployments, avg 3 replicas = 105 pods
  development:  20 deployments, avg 2 replicas = 40 pods
  qa:           15 deployments, avg 2 replicas = 30 pods
  preview:      10 deployments, avg 1 replica  = 10 pods
                Total: 185 pods

Average resources per pod: 0.5 CPU, 512 MiB memory
Hours saved per week: (12 hrs/night x 5 nights) + (48 hrs/weekend) = 108 hrs/week

Weekly savings:
  CPU:    185 x 0.5 cores x 108 hrs x $0.0425/core/hr = $ 421.65
  Memory: 185 x 0.5 GB   x 108 hrs x $0.0058/GB/hr   = $  57.67
  Weekly:  $479.32

Annual savings: $479.32 x 52 weeks = $24,924.64/year
```

### Part 4: Cleanup Script for Unused Resources

```bash
#!/usr/bin/env bash
# cleanup-unused-resources.sh -- Find and remove unused cloud resources
# Run in dry-run mode first: ./cleanup-unused-resources.sh --dry-run
set -euo pipefail

DRY_RUN="${1:-}"
DRY_RUN_FLAG=""
if [[ "$DRY_RUN" == "--dry-run" ]]; then
  echo "=== DRY RUN MODE -- no changes will be made ==="
  DRY_RUN_FLAG="--dry-run=client"
fi

REPORT_FILE="/tmp/cleanup-report-$(date +%Y%m%d).txt"
TOTAL_SAVINGS=0

log() {
  echo "$@" | tee -a "$REPORT_FILE"
}

log "=========================================="
log "  Unused Resource Cleanup Report"
log "  Date: $(date -u)"
log "=========================================="
log ""

# --- 1. Unattached EBS Volumes ---
log "=== Unattached EBS Volumes ==="
VOLUMES=$(aws ec2 describe-volumes \
  --filters "Name=status,Values=available" \
  --query 'Volumes[*].[VolumeId,Size,CreateTime,Tags[?Key==`Name`].Value|[0]]' \
  --output text 2>/dev/null || echo "")

VOL_COUNT=0
VOL_SIZE=0
while IFS=$'\t' read -r vol_id size created name; do
  [[ -z "$vol_id" ]] && continue
  ((VOL_COUNT++))
  ((VOL_SIZE += size))
  log "  $vol_id  ${size}GB  created:$created  name:${name:-none}"

  if [[ -z "$DRY_RUN_FLAG" ]]; then
    aws ec2 delete-volume --volume-id "$vol_id" 2>/dev/null && \
      log "    DELETED" || log "    FAILED"
  fi
done <<< "$VOLUMES"

VOL_SAVINGS=$((VOL_SIZE * 8))  # ~$0.08/GB/month for gp3
TOTAL_SAVINGS=$((TOTAL_SAVINGS + VOL_SAVINGS))
log "  Found: $VOL_COUNT volumes, ${VOL_SIZE}GB, ~\$${VOL_SAVINGS}/month"
log ""

# --- 2. Old EBS Snapshots (>90 days) ---
log "=== Old EBS Snapshots (>90 days) ==="
CUTOFF=$(date -u -d '90 days ago' +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || \
         date -u -v-90d +%Y-%m-%dT%H:%M:%SZ)

SNAPSHOTS=$(aws ec2 describe-snapshots \
  --owner-ids self \
  --filters "Name=start-time,Values=$CUTOFF" \
  --query 'Snapshots[*].[SnapshotId,VolumeSize,StartTime,Description]' \
  --output text 2>/dev/null || echo "")

SNAP_COUNT=0
SNAP_SIZE=0
while IFS=$'\t' read -r snap_id size start desc; do
  [[ -z "$snap_id" ]] && continue
  ((SNAP_COUNT++))
  ((SNAP_SIZE += size))
  log "  $snap_id  ${size}GB  $start"

  if [[ -z "$DRY_RUN_FLAG" ]]; then
    aws ec2 delete-snapshot --snapshot-id "$snap_id" 2>/dev/null && \
      log "    DELETED" || log "    FAILED"
  fi
done <<< "$SNAPSHOTS"

SNAP_SAVINGS=$((SNAP_SIZE * 5))  # ~$0.05/GB/month for snapshots
TOTAL_SAVINGS=$((TOTAL_SAVINGS + SNAP_SAVINGS))
log "  Found: $SNAP_COUNT snapshots, ${SNAP_SIZE}GB, ~\$${SNAP_SAVINGS}/month"
log ""

# --- 3. Unattached Elastic IPs ---
log "=== Unattached Elastic IPs ==="
EIPS=$(aws ec2 describe-addresses \
  --query 'Addresses[?AssociationId==null].[PublicIp,AllocationId]' \
  --output text 2>/dev/null || echo "")

EIP_COUNT=0
while IFS=$'\t' read -r ip alloc_id; do
  [[ -z "$ip" ]] && continue
  ((EIP_COUNT++))
  log "  $ip  ($alloc_id)"

  if [[ -z "$DRY_RUN_FLAG" ]]; then
    aws ec2 release-address --allocation-id "$alloc_id" 2>/dev/null && \
      log "    RELEASED" || log "    FAILED"
  fi
done <<< "$EIPS"

EIP_SAVINGS=$((EIP_COUNT * 4))  # ~$3.65/month per unused EIP
TOTAL_SAVINGS=$((TOTAL_SAVINGS + EIP_SAVINGS))
log "  Found: $EIP_COUNT unattached EIPs, ~\$${EIP_SAVINGS}/month"
log ""

# --- 4. Stale Load Balancers (zero requests in 30 days) ---
log "=== Stale Load Balancers ==="
STALE_LBS=$(aws elbv2 describe-load-balancers \
  --query 'LoadBalancers[*].[LoadBalancerArn,LoadBalancerName]' \
  --output text 2>/dev/null || echo "")

LB_COUNT=0
while IFS=$'\t' read -r lb_arn lb_name; do
  [[ -z "$lb_arn" ]] && continue

  # Check CloudWatch for request count over last 30 days
  REQUESTS=$(aws cloudwatch get-metric-statistics \
    --namespace AWS/ApplicationELB \
    --metric-name RequestCount \
    --dimensions "Name=LoadBalancer,Value=$(echo $lb_arn | cut -d/ -f2-)" \
    --start-time "$(date -u -d '30 days ago' +%Y-%m-%dT%H:%M:%SZ)" \
    --end-time "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --period 2592000 \
    --statistics Sum \
    --query 'Datapoints[0].Sum' \
    --output text 2>/dev/null || echo "0")

  if [[ "$REQUESTS" == "None" ]] || [[ "$REQUESTS" == "0" ]]; then
    ((LB_COUNT++))
    log "  STALE: $lb_name"

    if [[ -z "$DRY_RUN_FLAG" ]]; then
      aws elbv2 delete-load-balancer --load-balancer-arn "$lb_arn" 2>/dev/null && \
        log "    DELETED" || log "    FAILED"
    fi
  fi
done <<< "$STALE_LBS"

LB_SAVINGS=$((LB_COUNT * 22))  # ~$22/month per ALB
TOTAL_SAVINGS=$((TOTAL_SAVINGS + LB_SAVINGS))
log "  Found: $LB_COUNT stale ALBs, ~\$${LB_SAVINGS}/month"
log ""

# --- 5. Unused Kubernetes Resources ---
log "=== Unused Kubernetes Resources ==="

# Pods in CrashLoopBackOff or ImagePullBackOff for >7 days
CRASH_PODS=$(kubectl get pods --all-namespaces \
  --field-selector 'status.phase!=Running,status.phase!=Succeeded' \
  -o json 2>/dev/null | \
  jq -r '.items[] | select(
    .status.conditions[]? | select(
      .type=="Ready" and .status=="False" and
      (.lastTransitionTime | fromdateiso8601) < (now - 604800)
    )
  ) | "\(.metadata.namespace)/\(.metadata.name)"' 2>/dev/null || echo "")

POD_COUNT=0
while IFS= read -r pod; do
  [[ -z "$pod" ]] && continue
  ((POD_COUNT++))
  log "  CrashLooping >7d: $pod"
done <<< "$CRASH_PODS"

# Empty ConfigMaps and Secrets
EMPTY_CM=$(kubectl get configmaps --all-namespaces -o json 2>/dev/null | \
  jq -r '.items[] | select(.data == null or .data == {}) |
    "\(.metadata.namespace)/\(.metadata.name)"' 2>/dev/null | \
  grep -v "kube-" | head -20 || echo "")

EMPTY_COUNT=0
while IFS= read -r cm; do
  [[ -z "$cm" ]] && continue
  ((EMPTY_COUNT++))
  log "  Empty ConfigMap: $cm"
done <<< "$EMPTY_CM"

log "  Found: $POD_COUNT crashing pods, $EMPTY_COUNT empty ConfigMaps"
log ""

# --- Summary ---
log "=========================================="
log "  CLEANUP SUMMARY"
log "=========================================="
log ""
log "  Estimated Monthly Savings:  \$${TOTAL_SAVINGS}"
log "  Estimated Annual Savings:   \$((TOTAL_SAVINGS * 12))"
log ""
if [[ -n "$DRY_RUN_FLAG" ]]; then
  log "  This was a DRY RUN. Run without --dry-run to apply changes."
fi
log "=========================================="

echo ""
echo "Report saved to: $REPORT_FILE"
```

### Part 5: FinOps Maturity Model Assessment

```
FinOps Maturity Model Assessment
==================================

Level 1: Crawl (Ad Hoc)
  [ ] Cloud costs are reviewed monthly in a spreadsheet
  [ ] There is a single person responsible for cloud spend
  [ ] Basic cost alerts are set up (>120% of budget)
  [ ] Teams are aware of their cloud costs at a high level
  [ ] No formal tagging policy exists

Level 2: Walk (Informing)
  [x] Mandatory tagging enforced via admission controller (Gatekeeper)
  [x] Cost allocation reports generated monthly by team
  [x] Budget alerts configured per team and per environment
  [x] Reserved Instance coverage tracked and reported
  [ ] Showback/chargeback process implemented
  [ ] FinOps team established with cross-functional representation
  [ ] Cost optimization reviewed in sprint retrospectives

Level 3: Run (Optimizing)
  [x] Right-sizing recommendations generated weekly
  [x] Spot instances used for fault-tolerant workloads
  [x] Off-hours scaling automated for non-production
  [x] Unused resource cleanup automated
  [ ] Savings Plans or RIs actively managed (buy/sell lifecycle)
  [ ] Unit cost metrics defined (cost per transaction, cost per user)
  [ ] Anomaly detection with automated remediation

Level 4: Fly (Operating)
  [ ] Real-time cost dashboards visible to all engineers
  [ ] Cost is a first-class metric in SLOs
  [ ] Automated scaling tied to cost efficiency targets
  [ ] FinOps integrated into CI/CD (cost impact per PR)
  [ ] Multi-cloud cost optimization with arbitrage
  [ ] Predictive forecasting with ML models
  [ ] Continuous optimization with measurable business outcomes

Current Assessment: Level 3 -- Run (partially complete)
Next Steps: Implement unit cost metrics and anomaly detection
```

```
FinOps Adoption Journey
=========================

  CRAWL              WALK               RUN                FLY
  (Ad Hoc)           (Informing)        (Optimizing)       (Operating)
  ----+----------------+-----------------+----------------+---->
      |                |                 |                |
  - Manual         - Tagging         - Right-sizing    - Unit cost
    tracking         policy            automation        metrics
  - Single         - Cost            - Spot            - Cost in
    owner            allocation        adoption          SLOs
  - Monthly        - Team            - Off-hours       - PR-level
    reviews          budgets           scheduling        cost
  - Basic          - Showback        - Auto-cleanup    - Predictive
    alerts         - RI tracking     - Anomaly           forecasting
                                      detection       - Multi-cloud
                                                        optimization

  YOUR POSITION: ====================================>
                                              ^
                                              |
                                    Between Walk and Run
```

## Why This Works

1. **Policy-as-code for tagging.** Gatekeeper enforces tagging at admission
   time, preventing untagged resources from entering the cluster. This is
   far more reliable than post-hoc auditing.

2. **Usage-based allocation.** Allocating costs by actual resource requests
   (not just namespace count) gives teams an accurate picture. Teams that
   over-provision pay more, creating a natural incentive to right-size.

3. **Automated off-hours scaling.** The CronJob pattern uses kubectl
   annotations to preserve original replica counts, ensuring accurate
   scale-up without manual tracking. This captures the largest single
   source of waste in most organizations.

4. **Cleanup as a scheduled job.** Running the cleanup script weekly catches
   resource drift before it accumulates into significant cost.

5. **Maturity model.** The assessment gives leadership a clear view of
   current state and a roadmap for improvement. Trying to jump from Level 1
   to Level 4 in one step typically fails; incremental improvement works.

## Common Mistakes to Avoid

- **Enforcing tags without enabling teams.** If engineers cannot easily find
  valid cost-center codes, they will invent fake ones. Provide a self-service
  portal or CLI for cost-center lookup.

- **Chargeback before showback.** Jumping directly to charging teams for
  their usage creates adversarial dynamics. Start with showback (making
  costs visible without charging) for at least one quarter.

- **Not cleaning up after scale-down.** The CronJob annotations for
  original replica counts must be cleaned up on scale-up. If left behind,
  they cause confusion on subsequent cycles.

- **Ignoring network costs in allocation.** Cross-AZ and egress traffic can
  be 10-20% of total cost. If your allocation model only covers compute and
  storage, teams that generate heavy cross-zone traffic get a free ride.

- **Treating FinOps as a one-time project.** Cost optimization is a
  continuous practice. New resources are provisioned every day; without
  ongoing governance, savings erode within months.
