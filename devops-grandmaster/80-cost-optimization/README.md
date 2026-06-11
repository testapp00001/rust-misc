# Module 80: Cost Optimization — Right-Sizing, Spot Instances, Reserved Capacity

> **Previous Module:** [79 - Change Management](../79-change-management/README.md)
> **Next Module:** [81 - Capstone: The Unbreakable System](../81-capstone-unbreakable-system/README.md)

## The Problem

Your cloud bill is $50,000 per month. The CFO asks what you are getting for it. You look at the bill and see:
- 20 instances running at 15% CPU utilization
- 3 TB of unattached EBS volumes from old experiments
- Reserved instances that expired and converted to on-demand pricing
- A dev cluster running 24/7 that nobody uses after 6 PM

30-40% of cloud spend is waste. But waste is invisible until you look for it.

## The Naive Way

```bash
# "Just keep adding instances, performance is more important than cost"
# Or: "We'll optimize later" (you never do)
# Or: "Let's just set a budget alert at $60K" (not optimization, just a ceiling)
```

**Why this fails:**
- Cost grows linearly with usage (no economies of scale)
- No visibility into what is driving costs
- No accountability (nobody owns the cloud bill)
- Over-provisioning becomes the default ("just in case")
- Budget surprises at the end of each month

## The Right Way

### Cloud Cost Monitoring

```python
# cost_monitor.py — Cloud cost monitoring and alerting
import boto3
from datetime import datetime, timedelta
from dataclasses import dataclass

@dataclass
class CostBreakdown:
    service: str
    current_month: float
    previous_month: float
    change_percent: float

class AWSCostMonitor:
    def __init__(self):
        self.client = boto3.client('ce', region_name='us-east-1')

    def get_monthly_costs(self) -> list:
        """Get current month costs by service."""
        today = datetime.now()
        start = today.replace(day=1).strftime('%Y-%m-%d')
        end = today.strftime('%Y-%m-%d')

        response = self.client.get_cost_and_usage(
            TimePeriod={'Start': start, 'End': end},
            Granularity='MONTHLY',
            Metrics=['UnblendedCost'],
            GroupBy=[{'Type': 'DIMENSION', 'Key': 'SERVICE'}]
        )

        costs = []
        for result in response['ResultsByTime'][0]['Groups']:
            costs.append({
                'service': result['Keys'][0],
                'cost': float(result['Metrics']['UnblendedCost']['Amount'])
            })

        return sorted(costs, key=lambda x: -x['cost'])

    def get_cost_trend(self, days: int = 30) -> list:
        """Get daily cost trend."""
        end = datetime.now()
        start = end - timedelta(days=days)

        response = self.client.get_cost_and_usage(
            TimePeriod={
                'Start': start.strftime('%Y-%m-%d'),
                'End': end.strftime('%Y-%m-%d')
            },
            Granularity='DAILY',
            Metrics=['UnblendedCost']
        )

        return [
            {
                'date': result['TimePeriod']['Start'],
                'cost': float(result['Total']['UnblendedCost']['Amount'])
            }
            for result in response['ResultsByTime']
        ]

    def get_rightsizing_recommendations(self) -> list:
        """Get AWS right-sizing recommendations."""
        response = self.client.get_rightsizing_recommendation(
            Service='AmazonEC2',
            Configuration={
                'RecommendationTarget': 'SAME_INSTANCE_FAMILY',
                'BenefitsConsidered': True
            }
        )

        recommendations = []
        for rec in response.get('RightsizingRecommendations', []):
            recommendations.append({
                'instance': rec['CurrentInstance']['InstanceName'],
                'current_type': rec['CurrentInstance']['InstanceType'],
                'recommended_type': rec.get('RightsizingType', 'Terminate'),
                'estimated_savings': float(
                    rec.get('EstimatedMonthlySavings', {}).get('Value', 0)
                )
            })

        return recommendations
```

### Right-Sizing Instances

```yaml
# right-sizing-report.yaml — Generate right-sizing recommendations
apiVersion: batch/v1
kind: CronJob
metadata:
  name: right-sizing-report
  namespace: monitoring
spec:
  schedule: "0 9 * * 1"  # Every Monday at 9 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
            - name: analyzer
              image: cost-analyzer:latest
              command: ["python", "right_sizing.py"]
              env:
                - name: PROMETHEUS_URL
                  value: "http://prometheus:9090"
                - name: SLACK_WEBHOOK
                  valueFrom:
                    secretKeyRef:
                      name: slack-webhook
                      key: url
```

```python
# right_sizing.py — Analyze and recommend right-sizing
import requests
from datetime import datetime, timedelta

class RightSizingAnalyzer:
    def __init__(self, prometheus_url: str):
        self.prometheus = prometheus_url

    def analyze_pods(self, namespace: str = "production") -> list:
        """Analyze pod resource utilization vs requests."""
        # Get CPU utilization
        cpu_query = f"""
        sum(rate(container_cpu_usage_seconds_total{{namespace="{namespace}"}}[7d])) by (pod, container)
        /
        sum(kube_pod_container_resource_requests{{namespace="{namespace}", resource="cpu"}}) by (pod, container)
        """

        # Get memory utilization
        mem_query = f"""
        sum(container_memory_working_set_bytes{{namespace="{namespace}"}}) by (pod, container)
        /
        sum(kube_pod_container_resource_requests{{namespace="{namespace}", resource="memory"}}) by (pod, container)
        """

        cpu_data = self._query(cpu_query)
        mem_data = self._query(mem_query)

        recommendations = []

        for pod_data in cpu_data:
            pod = pod_data['metric'].get('pod', 'unknown')
            container = pod_data['metric'].get('container', 'unknown')
            cpu_util = float(pod_data['value'][1])

            # Find matching memory data
            mem_util = 0
            for m in mem_data:
                if m['metric'].get('pod') == pod and m['metric'].get('container') == container:
                    mem_util = float(m['value'][1])
                    break

            # Generate recommendation
            if cpu_util < 0.3 and mem_util < 0.3:
                recommendations.append({
                    'pod': pod,
                    'container': container,
                    'cpu_utilization': f"{cpu_util:.1%}",
                    'memory_utilization': f"{mem_util:.1%}",
                    'recommendation': 'Reduce resource requests by 50%',
                    'estimated_savings': '30-50%'
                })
            elif cpu_util > 0.8 or mem_util > 0.8:
                recommendations.append({
                    'pod': pod,
                    'container': container,
                    'cpu_utilization': f"{cpu_util:.1%}",
                    'memory_utilization': f"{mem_util:.1%}",
                    'recommendation': 'Increase resource requests',
                    'risk': 'Potential OOMKill or throttling'
                })

        return recommendations

    def _query(self, query: str) -> list:
        response = requests.get(
            f"{self.prometheus}/api/v1/query",
            params={'query': query}
        )
        return response.json().get('data', {}).get('result', [])
```

### Spot/Preemptible Instances

```yaml
# spot-instances.yaml — Use spot instances for non-critical workloads
apiVersion: karpenter.sh/v1beta1
kind: NodePool
metadata:
  name: spot-pool
spec:
  template:
    spec:
      # Use spot instances
      requirements:
        - key: karpenter.sh/capacity-type
          operator: In
          values: ["spot"]
        - key: node.kubernetes.io/instance-type
          operator: In
          values:
            - m5.large
            - m5.xlarge
            - m5a.large
            - m5a.xlarge
            - m6i.large
            - m6i.xlarge

      # Taint spot nodes (only tolerate workloads that handle interruption)
      taints:
        - key: spot
          value: "true"
          effect: NoSchedule

  # Scale limits
  limits:
    cpu: "100"
    memory: 400Gi

  # Disruption policy
  disruption:
    consolidationPolicy: WhenUnderutilized
    expireAfter: 720h  # 30 days

---
# Deployment that tolerates spot instances
apiVersion: apps/v1
kind: Deployment
metadata:
  name: batch-processor
spec:
  replicas: 5
  selector:
    matchLabels:
      app: batch-processor
  template:
    metadata:
      labels:
        app: batch-processor
    spec:
      # Tolerate spot taint
      tolerations:
        - key: spot
          value: "true"
          effect: NoSchedule

      # Prefer spot but allow on-demand
      affinity:
        nodeAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
            - weight: 100
              preference:
                matchExpressions:
                  - key: karpenter.sh/capacity-type
                    operator: In
                    values: ["spot"]

      containers:
        - name: processor
          image: batch-processor:latest
          # Design for interruption:
          # - Checkpoint progress
          # - Graceful shutdown
          # - Idempotent processing
          lifecycle:
            preStop:
              exec:
                command: ["/bin/sh", "-c", "sleep 15"]
          env:
            - name: GRACEFUL_SHUTDOWN_TIMEOUT
              value: "30"
```

### Reserved Instances and Savings Plans

```python
# savings_calculator.py — Calculate savings from reserved instances
from dataclasses import dataclass

@dataclass
class InstancePricing:
    on_demand_hourly: float
    reserved_1yr_hourly: float
    reserved_3yr_hourly: float
    spot_hourly: float

class SavingsCalculator:
    """Compare pricing models."""

    # AWS pricing examples (us-east-1, Linux)
    PRICING = {
        "m5.xlarge": InstancePricing(0.192, 0.121, 0.078, 0.058),
        "m5.2xlarge": InstancePricing(0.384, 0.242, 0.156, 0.115),
        "c5.xlarge": InstancePricing(0.17, 0.107, 0.069, 0.051),
        "r5.xlarge": InstancePricing(0.252, 0.159, 0.102, 0.076),
    }

    def compare(self, instance_type: str, count: int, hours_per_month: int = 730):
        pricing = self.PRICING[instance_type]

        on_demand = pricing.on_demand_hourly * count * hours_per_month
        reserved_1yr = pricing.reserved_1yr_hourly * count * hours_per_month
        reserved_3yr = pricing.reserved_3yr_hourly * count * hours_per_month
        spot = pricing.spot_hourly * count * hours_per_month

        report = f"\n=== COST COMPARISON: {count}x {instance_type} ===\n\n"
        report += f"{'Model':<20} {'Monthly':>12} {'Annual':>14} {'vs On-Demand':>14}\n"
        report += "-" * 62 + "\n"
        report += f"{'On-Demand':<20} ${on_demand:>10,.2f} ${on_demand*12:>12,.2f} {'baseline':>14}\n"
        report += f"{'Reserved (1yr)':<20} ${reserved_1yr:>10,.2f} ${reserved_1yr*12:>12,.2f} {((on_demand-reserved_1yr)/on_demand*100):>+12.1f}%\n"
        report += f"{'Reserved (3yr)':<20} ${reserved_3yr:>10,.2f} ${reserved_3yr*12:>12,.2f} {((on_demand-reserved_3yr)/on_demand*100):>+12.1f}%\n"
        report += f"{'Spot':<20} ${spot:>10,.2f} ${spot*12:>12,.2f} {((on_demand-spot)/on_demand*100):>+12.1f}%\n"
        report += f"\nAnnual savings (Reserved 3yr vs On-Demand): ${(on_demand-reserved_3yr)*12:,.2f}\n"

        return report


calculator = SavingsCalculator()
print(calculator.compare("m5.xlarge", count=10))
# Annual savings: ~$10,000 per 10 instances
```

### Resource Tagging

```yaml
# tagging-policy.yaml — Enforce resource tagging for cost allocation
apiVersion: templates.gatekeeper.sh/v1beta1
kind: ConstraintTemplate
metadata:
  name: requiredtags
spec:
  crd:
    spec:
      names:
        kind: RequiredTags
      validation:
        openAPIV3Schema:
          type: object
          properties:
            tags:
              type: array
              items:
                type: string
  targets:
    - target: admission.k8s.gatekeeper.sh
      rego: |
        package requiredtags

        violation[{"msg": msg}] {
          required := input.parameters.tags
          provided := input.review.object.metadata.labels
          missing := required[_]
          not provided[missing]
          msg := sprintf("Missing required label: %v", [missing])
        }

---
apiVersion: constraints.gatekeeper.sh/v1beta1
kind: RequiredTags
metadata:
  name: require-cost-tags
spec:
  match:
    kinds:
      - apiGroups: ["apps"]
        kinds: ["Deployment"]
    namespaces: ["production"]
  parameters:
    tags:
      - "cost-center"
      - "team"
      - "environment"
```

### FinOps Practices

```
FINOPS PRACTICES:

1. INFORM
   - Tag all resources with cost-center, team, environment
   - Showback: Report costs per team (not chargeback)
   - Daily cost dashboards visible to all engineers

2. OPTIMIZE
   - Right-size instances based on actual usage
   - Use spot instances for fault-tolerant workloads
   - Purchase reserved instances for steady-state workloads
   - Delete unused resources (unattached volumes, idle load balancers)
   - Implement auto-scaling to match demand

3. OPERATE
   - Set budget alerts (50%, 75%, 90%, 100%)
   - Review costs weekly in engineering standup
   - Monthly cost optimization review
   - Quarterly reserved instance planning
```

## The Production Way

### Cost Allocation by Team

```python
# cost_allocation.py — Allocate costs to teams based on labels
import boto3
from collections import defaultdict

class CostAllocator:
    def __init__(self):
        self.client = boto3.client('ce')

    def get_costs_by_tag(self, tag_key: str, start: str, end: str) -> dict:
        """Get costs grouped by a specific tag."""
        response = self.client.get_cost_and_usage(
            TimePeriod={'Start': start, 'End': end},
            Granularity='MONTHLY',
            Metrics=['UnblendedCost'],
            GroupBy=[
                {'Type': 'TAG', 'Key': tag_key}
            ]
        )

        costs = defaultdict(float)
        for result in response['ResultsByTime']:
            for group in result['Groups']:
                team = group['Keys'][0].replace(f'{tag_key}$', '') or 'untagged'
                costs[team] += float(group['Metrics']['UnblendedCost']['Amount'])

        return dict(costs)

    def generate_report(self) -> str:
        today = datetime.now()
        start = today.replace(day=1).strftime('%Y-%m-%d')
        end = today.strftime('%Y-%m-%d')

        team_costs = self.get_costs_by_tag('team', start, end)
        total = sum(team_costs.values())

        report = "\n=== COST ALLOCATION BY TEAM ===\n\n"
        report += f"Period: {start} to {end}\n"
        report += f"Total: ${total:,.2f}\n\n"
        report += f"{'Team':<20} {'Cost':>12} {'% of Total':>12}\n"
        report += "-" * 46 + "\n"

        for team, cost in sorted(team_costs.items(), key=lambda x: -x[1]):
            report += f"{team:<20} ${cost:>10,.2f} {cost/total*100:>10.1f}%\n"

        return report
```

### Auto-Scaling for Cost

```yaml
# cost-aware-scaling.yaml — Scale down during off-hours
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web
  namespace: production
spec:
  replicas: 10
  # ...
---
# Scale down at night
apiVersion: batch/v1
kind: CronJob
metadata:
  name: scale-down-evening
  namespace: production
spec:
  schedule: "0 20 * * *"  # 8 PM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
            - name: scaler
              image: bitnami/kubectl
              command:
                - kubectl
                - scale
                - deployment/web
                - --replicas=3
                - -n
                - production
          restartPolicy: OnFailure

---
# Scale up in the morning
apiVersion: batch/v1
kind: CronJob
metadata:
  name: scale-up-morning
  namespace: production
spec:
  schedule: "0 8 * * *"  # 8 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
            - name: scaler
              image: bitnami/kubectl
              command:
                - kubectl
                - scale
                - deployment/web
                - --replicas=10
                - -n
                - production
          restartPolicy: OnFailure
```

### Cleaning Up Unused Resources

```bash
#!/bin/bash
# cleanup-unused-resources.sh — Find and remove unused cloud resources
set -euo pipefail

echo "=== Cloud Resource Cleanup ==="
echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

# 1. Unattached EBS volumes
echo "--- Unattached EBS Volumes ---"
VOLUMES=$(aws ec2 describe-volumes \
  --filters Name=status,Values=available \
  --query 'Volumes[*].[VolumeId,Size,CreateTime]' \
  --output text)
if [ -n "$VOLUMES" ]; then
  echo "$VOLUMES"
  echo "Total unattached volumes: $(echo "$VOLUMES" | wc -l)"
else
  echo "None found."
fi
echo ""

# 2. Unused Elastic IPs
echo "--- Unused Elastic IPs ---"
EIPS=$(aws ec2 describe-addresses \
  --query 'Addresses[?AssociationId==null].[AllocationId,PublicIp]' \
  --output text)
if [ -n "$EIPS" ]; then
  echo "$EIPS"
else
  echo "None found."
fi
echo ""

# 3. Old snapshots (> 90 days)
echo "--- Old EBS Snapshots (> 90 days) ---"
OLD_DATE=$(date -d "90 days ago" +%Y-%m-%dT%H:%M:%SZ)
SNAPSHOTS=$(aws ec2 describe-snapshots \
  --owner-ids self \
  --query "Snapshots[?StartTime<'$OLD_DATE'].[SnapshotId,StartTime,VolumeSize]" \
  --output text)
if [ -n "$SNAPSHOTS" ]; then
  echo "$SNAPSHOTS"
  echo "Total old snapshots: $(echo "$SNAPSHOTS" | wc -l)"
else
  echo "None found."
fi
echo ""

# 4. Idle load balancers
echo "--- Load Balancers with No Targets ---"
ELBS=$(aws elbv2 describe-load-balancers --query 'LoadBalancers[*].LoadBalancerArn' --output text)
for ELB in $ELBS; do
  TARGETS=$(aws elbv2 describe-target-groups --load-balancer-arn $ELB \
    --query 'TargetGroups[*].TargetGroupArn' --output text)
  for TG in $TARGETS; do
    HEALTH=$(aws elbv2 describe-target-health --target-group-arn $TG \
      --query 'TargetHealthDescriptions' --output text)
    if [ -z "$HEALTH" ]; then
      echo "Idle target group: $TG"
    fi
  done
done
echo ""

echo "=== Cleanup Complete ==="
```

## Hands-On Lab: Optimize a Cloud Bill

### Step 1: Analyze Current Costs

```bash
# If using AWS CLI
aws ce get-cost-and-usage \
  --time-period Start=$(date -d "30 days ago" +%Y-%m-%d),End=$(date +%Y-%m-%d) \
  --granularity MONTHLY \
  --metrics UnblendedCost \
  --group-by Type=DIMENSION,Key=SERVICE

# If using kubectl (check resource requests vs actual usage)
echo "=== Current Resource Requests ==="
kubectl get pods -n production -o json | jq -r '.items[] |
  .metadata.name as $pod |
  .spec.containers[] |
  "\($pod): CPU=\(.resources.requests.cpu) MEM=\(.resources.requests.memory)"'

echo ""
echo "=== Actual Resource Usage ==="
kubectl top pods -n production
```

### Step 2: Identify Waste

```bash
# Find over-provisioned pods
python3 << 'EOF'
import json
import subprocess

# Get requests
result = subprocess.run(
    ['kubectl', 'get', 'pods', '-n', 'production', '-o', 'json'],
    capture_output=True, text=True
)
pods = json.loads(result.stdout)

# Get actual usage
result = subprocess.run(
    ['kubectl', 'top', 'pods', '-n', 'production', '--no-headers'],
    capture_output=True, text=True
)
usage = {}
for line in result.stdout.strip().split('\n'):
    parts = line.split()
    if len(parts) >= 3:
        usage[parts[0]] = {'cpu': parts[1], 'memory': parts[2]}

print("=== Right-Sizing Recommendations ===\n")
for pod in pods['items']:
    name = pod['metadata']['name']
    for container in pod['spec']['containers']:
        req = container.get('resources', {}).get('requests', {})
        cpu_req = req.get('cpu', 'N/A')
        mem_req = req.get('memory', 'N/A')

        if name in usage:
            cpu_actual = usage[name]['cpu']
            mem_actual = usage[name]['memory']
            print(f"{name}:")
            print(f"  CPU: request={cpu_req}, actual={cpu_actual}")
            print(f"  Memory: request={mem_req}, actual={mem_actual}")
            print()
EOF
```

### Step 3: Implement Optimizations

```bash
# 1. Right-size resource requests
kubectl set resources deployment/web -n production \
  -c web --requests=cpu=200m,memory=256Mi

# 2. Enable cluster autoscaler
kubectl annotate nodegroup --overwrite \
  cluster-autoscaler.k8s.io/scale-down-delay-after-add=5m

# 3. Clean up unused PVCs
kubectl get pvc -n production | grep Released
# Delete released PVCs

# 4. Implement spot instances for batch workloads
# (Apply Karpenter NodePool from above)
```

### Lab Validation Checklist

- [ ] Current costs analyzed and documented
- [ ] Over-provisioned resources identified
- [ ] Right-sizing recommendations generated
- [ ] At least one optimization implemented
- [ ] Cost savings estimated

## Limitation -> Next Topic

You have optimized costs, hardened security, automated deployments, and built monitoring. Now it is time to put it all together. The capstone project: build a production system that demonstrates every concept from this entire curriculum.

**Next: [Module 81 — Capstone: The Unbreakable System](../81-capstone-unbreakable-system/README.md)**
