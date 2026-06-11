# Cheatsheet: Horizontal vs Vertical Scaling

## Comparison

| | Vertical (Scale Up) | Horizontal (Scale Out) |
|---|---|---|
| Method | Bigger machine | More machines |
| Limit | Hardware max | Practically unlimited |
| Complexity | Simple | Complex |
| Cost | Expensive at scale | More cost-effective |
| Downtime | Usually required | Zero downtime |
| State | Easy (single machine) | Hard (distributed) |

## When to Use Vertical
- Databases (hard to distribute)
- Legacy applications
- Simple applications
- Development/testing

## When to Use Horizontal
- Stateless web servers
- API servers
- Microservices
- High-traffic applications

## Kubernetes Horizontal Scaling
```yaml
# HPA — Horizontal Pod Autoscaler
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: my-app
  minReplicas: 2
  maxReplicas: 20
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
```

## Scaling Bottlenecks
```
Stateless App → Easy to scale horizontally
Stateful App → Need shared state (Redis, database)
Database → Read replicas, then sharding
File Storage → Object storage (S3, GCS)
```
