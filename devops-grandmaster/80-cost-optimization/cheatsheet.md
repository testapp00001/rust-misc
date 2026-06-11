# Cheatsheet: Cost Optimization

## Cost Monitoring
```bash
# AWS Cost Explorer
aws ce get-cost-and-usage --time-period Start=2024-01-01,End=2024-01-31

# GCP Billing
gcloud billing budgets list
```

## Right-Sizing
```
1. Monitor actual usage (CPU, memory)
2. Compare to provisioned resources
3. Downsize over-provisioned instances
4. Upsize under-provisioned instances
```

## Spot/Preemptible Instances
```yaml
# Kubernetes node selector for spot instances
spec:
  nodeSelector:
    node.kubernetes.io/lifecycle: spot
  tolerations:
    - key: "node.kubernetes.io/lifecycle"
      operator: "Equal"
      value: "spot"
      effect: "NoSchedule"
```

## Reserved Instances
```
1-year commitment: ~30% savings
3-year commitment: ~60% savings
Best for: Stable, predictable workloads
```

## Auto-Scaling for Cost
```yaml
# Scale down during off-hours
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
spec:
  minReplicas: 1  # Scale to minimum at night
  maxReplicas: 20
```

## Resource Tagging
```yaml
tags:
  Environment: production
  Team: backend
  Project: my-app
  CostCenter: engineering
```
