# Cheatsheet: Kubernetes Decision Framework

## When to Use What

| Scenario | Solution |
|----------|----------|
| 1 server, < 10 containers | Docker Compose |
| 2-5 servers, simple orchestration | Docker Swarm |
| AWS-native, don't want k8s management | ECS/Fargate |
| Stateless containers, pay-per-request | Cloud Run |
| Complex microservices, multi-team | Kubernetes |
| Multi-cloud, compliance | Multi-cluster K8s |

## Kubernetes Costs

| Component | Cost (AWS EKS) |
|-----------|---------------|
| Control plane | $73/month |
| Worker nodes (t3.medium) | $30/month each |
| Load balancer | $18/month |
| Storage (EBS) | $0.10/GB/month |
| **Minimum (3 nodes)** | **~$180/month** |

## Migration Path

```
Docker Compose → Docker Swarm → ECS/Fargate → Kubernetes → Multi-cluster K8s
```

## Key Decision Questions

1. How many containers? ___
2. How many servers? ___
3. How many developers? ___
4. Need auto-scaling? (y/n) ___
5. Deploy multiple times/day? (y/n) ___
6. Multiple teams? (y/n) ___

**Scoring:** 0-2 yes → Compose, 3-4 yes → Swarm/ECS, 5+ yes → K8s
