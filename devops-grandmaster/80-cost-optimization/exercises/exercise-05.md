# Exercise 05: FinOps Implementation

**Type:** Integration | **Time:** 45 min | **Difficulty:** Hard

## Objective

Design a complete FinOps implementation for your organization, including resource tagging
policies enforced via OPA Gatekeeper, cost allocation dashboards, auto-scaling schedules,
and automated cleanup of unused resources.

## Background

Your organization, **TechCorp**, has grown from 2 teams to 8 teams over the past year.
Cloud spending has increased from $50,000/month to $180,000/month with no clear
accountability. The CTO has mandated a FinOps program to:

1. Make every team accountable for their cloud spend
2. Eliminate waste from unused and idle resources
3. Implement guardrails to prevent cost overruns
4. Provide visibility into cost trends and anomalies

**Teams:**
- Platform (infrastructure, shared services)
- Backend (API servers, microservices)
- Frontend (web app, CDN)
- Data Engineering (ETL, data warehouse)
- ML/AI (training, inference)
- Mobile (mobile API, push notifications)
- QA (testing environments)
- Security (monitoring, scanning)

## Tasks

### Task 1: Resource Tagging Policy

Design a mandatory tagging schema. Every resource must have these tags to be deployed.

| Tag Key | Description | Example Values | Required |
|---------|-------------|----------------|----------|
| `cost-center` | Budget center code | `CC-1001`, `CC-1002` | Yes |
| `team` | Owning team | `platform`, `backend`, `ml-ai` | Yes |
| `environment` | Deployment environment | `prod`, `staging`, `dev` | Yes |
| `application` | Application name | `user-api`, `payment-svc` | Yes |
| `managed-by` | Provisioning tool | `terraform`, `kubectl`, `manual` | Yes |
| `expiry-date` | Auto-delete date (optional) | `2026-09-01` | No |
| `cost-cap` | Monthly budget limit (optional) | `500` (USD) | No |

Write an OPA Gatekeeper `ConstraintTemplate` and `Constraint` that enforces this tagging
policy on all Kubernetes namespaces. The policy should:
- Require all mandatory tags as labels on namespaces
- Deny namespace creation if any required label is missing
- Validate that `environment` is one of: `prod`, `staging`, `dev`, `sandbox`
- Validate that `team` matches a known team list

```yaml
# Write the ConstraintTemplate
apiVersion: templates.gatekeeper.sh/v1
kind: ConstraintTemplate
metadata:
  name: k8srequiredcostlabels
spec:
  crd:
    spec:
      names:
        kind: K8sRequiredCostLabels
      validation:
        # Define parameters for required labels and allowed values
        openAPIV3Schema:
          type: object
          properties:
            # YOUR SCHEMA HERE
  targets:
    - target: admission.k8s.gatekeeper.sh
      rego: |
        package k8srequiredcostlabels
        # YOUR REGO POLICY HERE
```

```yaml
# Write the Constraint resource
apiVersion: constraints.gatekeeper.sh/v1beta1
kind: K8sRequiredCostLabels
metadata:
  name: require-cost-labels
spec:
  match:
    kinds:
      - apiGroups: [""]
        kinds: ["Namespace"]
  parameters:
    # YOUR PARAMETERS HERE
```

### Task 2: Cost Allocation Dashboard

Design a Grafana dashboard specification for cost allocation. The dashboard should
display:

1. **Daily Cost by Team** (stacked area chart, last 30 days)
2. **Cost by Environment** (pie chart, current month)
3. **Cost Trend** (line chart, last 6 months, with anomaly highlighting)
4. **Budget vs Actual by Team** (bar chart with threshold lines)
5. **Top 10 Most Expensive Resources** (table)
6. **Idle Resource Detection** (table of resources with <10% utilization)

Write the Grafana dashboard JSON structure (you can use pseudocode for the panels, but
include the query structure):

```json
{
  "dashboard": {
    "title": "FinOps - Cost Allocation Dashboard",
    "tags": ["finops", "cost"],
    "timezone": "browser",
    "panels": [
      {
        "id": 1,
        "title": "Daily Cost by Team",
        "type": "timeseries",
        "gridPos": { "h": 8, "w": 24, "x": 0, "y": 0 },
        "targets": [
          {
            "datasource": "Prometheus",
            "expr": "YOUR_PROMQL_QUERY_HERE",
            "legendFormat": "{{team}}"
          }
        ]
      },
      // ADD REMAINING PANELS
    ],
    "templating": {
      "list": [
        // Add variables for team, environment, time range
      ]
    }
  }
}
```

For each panel, write the PromQL or data source query that would power it. Assume you
have these metrics available:
- `cloud_cost_daily{team, environment, service, instance_type}` - daily cost per resource
- `cloud_budget_monthly{team}` - monthly budget per team
- `resource_utilization{resource_id, metric}` - CPU/memory utilization
- `cloud_cost_monthly_total{team, environment}` - monthly totals

### Task 3: Auto-Scaling Schedules

Design auto-scaling schedules for dev/staging environments that scale down during
off-hours to save costs. Write CronJob manifests or HPA configurations.

**Requirements:**
- Dev environments: scale to 0 replicas from 8 PM to 8 AM weekdays, and all weekend
- Staging environments: scale to minimum from 10 PM to 6 AM weekdays, and all weekend
- Production: maintain baseline, scale up during known peak hours (9 AM - 6 PM)
- QA environments: scale to 0 when not in use, wake on-demand via API call

Write the Kubernetes CronJob manifests for at least two schedules:

```yaml
# CronJob: Scale down dev environments at 8 PM
apiVersion: batch/v1
kind: CronJob
metadata:
  name: scale-down-dev
  namespace: finops-automation
spec:
  schedule: "0 20 * * 1-5"  # 8 PM Mon-Fri
  jobTemplate:
    spec:
      template:
        spec:
          serviceAccountName: finops-scaler
          containers:
            - name: scaler
              image: bitnami/kubectl:latest
              command:
                - /bin/sh
                - -c
                - |
                  # YOUR SCALING COMMANDS HERE
                  # Scale all deployments in dev namespaces to 0
          restartPolicy: OnFailure
```

```yaml
# CronJob: Scale up dev environments at 8 AM
# YOUR YAML HERE
```

Also write an HPA configuration for production that scales based on cost-efficiency
(requests per dollar):

```yaml
# HPA that scales based on custom metric: requests/dollar
# YOUR YAML HERE
```

### Task 4: Automated Cleanup of Unused Resources

Write a Kubernetes CronJob that identifies and cleans up unused resources. The job should:

1. **Find unused PVCs:** PVCs not mounted by any pod for >7 days
2. **Find idle namespaces:** Namespaces with no running pods for >14 days
3. **Find old ConfigMaps/Secrets:** Not referenced by any pod for >30 days
4. **Find expired resources:** Resources where `expiry-date` label has passed

Write the cleanup script and CronJob:

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: resource-cleanup
  namespace: finops-automation
spec:
  schedule: "0 3 * * *"  # Daily at 3 AM
  jobTemplate:
    spec:
      template:
        spec:
          serviceAccountName: finops-cleaner
          containers:
            - name: cleaner
              image: bitnami/kubectl:latest
              command:
                - /bin/sh
                - -c
                - |
                  echo "=== Resource Cleanup Report ==="
                  echo "Date: $(date)"

                  # Task 1: Find unused PVCs
                  # YOUR SCRIPT HERE
                  # Hint: kubectl get pvc -A -o json | jq ...

                  # Task 2: Find idle namespaces
                  # YOUR SCRIPT HERE

                  # Task 3: Find expired resources
                  # YOUR SCRIPT HERE
                  # Hint: kubectl get ns -l expiry-date -o json |
                  #   jq '.items[] | select(.metadata.labels["expiry-date"] < now)'

                  echo "=== Cleanup Complete ==="
          restartPolicy: OnFailure
```

Also write the RBAC configuration for the cleanup service account:

```yaml
# ServiceAccount, Role, and RoleBinding for finops-cleaner
# It needs permissions to:
# - List/get/delete PVCs, ConfigMaps, Secrets across namespaces
# - List/get/delete namespaces (with confirmation)
# - List pods to check usage
# YOUR YAML HERE
```

### Task 5: Cost Anomaly Detection

Design a simple cost anomaly detection system using Prometheus alerting rules. The system
should alert when:

1. A team's daily cost exceeds 150% of their 7-day moving average
2. A single resource's cost spikes by more than 200% day-over-day
3. Total monthly spend is projected to exceed the annual budget (based on current run rate)
4. Spot instance spending exceeds the configured safety margin

Write the PrometheusRule manifest:

```yaml
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: cost-anomaly-alerts
  namespace: monitoring
spec:
  groups:
    - name: cost-anomaly-detection
      interval: 6h
      rules:
        # Alert 1: Team daily cost exceeds 150% of 7-day average
        - alert: TeamCostAnomaly
          expr: |
            # YOUR PROMQL HERE
          for: 1d
          labels:
            severity: warning
          annotations:
            summary: "Cost anomaly detected for team {{ $labels.team }}"
            description: "..."

        # Alert 2: Single resource cost spike
        # YOUR ALERT HERE

        # Alert 3: Monthly spend projection exceeds budget
        # YOUR ALERT HERE

        # Alert 4: Spot spending safety margin
        # YOUR ALERT HERE
```

<details>
<summary>Hint 1: Gatekeeper Rego Policy</summary>

```rego
package k8srequiredcostlabels

violation[{"msg": msg}] {
  required := input.parameters.requiredLabels[_]
  not input.review.object.metadata.labels[required]
  msg := sprintf("Missing required cost label: %v", [required])
}

violation[{"msg": msg}] {
  val := input.review.object.metadata.labels["environment"]
  allowed := input.parameters.allowedEnvironments
  not value_in_list(val, allowed)
  msg := sprintf("Environment %v not in allowed list: %v", [val, allowed])
}

value_in_list(val, lst) {
  lst[_] == val
}
```

</details>

<details>
<summary>Hint 2: Cost Queries in PromQL</summary>

For daily cost by team:
```promql
sum by (team) (
  cloud_cost_daily{team!=""}
  and on() (day_of_month() == day_of_month())
)
```

For budget vs actual:
```promql
# Actual (cumulative this month)
sum by (team) (
  increase(cloud_cost_daily[30d])
)
# Budget
cloud_budget_monthly
```

For cost anomaly detection:
```promql
# Current day cost vs 7-day average
(
  sum by (team) (cloud_cost_daily)
  /
  avg_over_time(sum by (team) (cloud_cost_daily)[7d:1d])
) > 1.5
```

</details>

<details>
<summary>Hint 3: Cleanup Script Approach</summary>

For finding unused PVCs:
```bash
# Get all PVCs
pvcs=$(kubectl get pvc -A -o json)

# For each PVC, check if any pod is using it
for pvc in $(echo $pvcs | jq -r '.items[] | .metadata.namespace + "/" + .metadata.name'); do
  ns=$(echo $pvc | cut -d/ -f1)
  name=$(echo $pvc | cut -d/ -f2)
  # Check if any pod mounts this PVC
  users=$(kubectl get pods -n $ns -o json |
    jq -r ".items[].spec.volumes[]?.persistentVolumeClaim.claimName // empty" |
    grep -c "^${name}$" || true)
  if [ "$users" -eq 0 ]; then
    echo "UNUSED PVC: $ns/$name"
  fi
done
```

For expired resources, compare the `expiry-date` label with the current date:
```bash
kubectl get ns -l expiry-date -o json |
  jq -r '.items[] |
    select(.metadata.labels["expiry-date"] < "'$(date +%Y-%m-%d)'") |
    .metadata.name'
```

</details>

## Verification

After completing this exercise, you should have:
- A Gatekeeper ConstraintTemplate and Constraint for cost labeling
- A Grafana dashboard specification with 6 panels and PromQL queries
- At least 2 CronJob manifests for auto-scaling schedules
- A cleanup CronJob with RBAC configuration
- PrometheusRule alerts for cost anomaly detection (at least 3 alerts)
- Everything should be valid YAML that could be applied to a real cluster

## Reflection Questions

1. How do you balance cost optimization with developer productivity?
2. What is the "crawl, walk, run" model of FinOps maturity, and where would you start?
3. How would you handle a team that consistently exceeds their budget?
4. What cultural changes are needed alongside the technical implementation?
5. How do you prevent FinOps from becoming a bottleneck for engineering teams?
