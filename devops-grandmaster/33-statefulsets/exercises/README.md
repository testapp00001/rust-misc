# Module 33: StatefulSets - Exercises

## Overview

These exercises will teach you how to deploy and manage stateful applications using StatefulSets. You'll learn stable network identities, ordered deployment, persistent storage patterns, and safe scaling strategies for databases and distributed systems.

## Exercise List

| Exercise | Type | Difficulty | Description |
|----------|------|------------|-------------|
| 01 | Conceptual | Beginner | StatefulSet vs Deployment for stateful workloads |
| 02 | Guided | Intermediate | Deploy a PostgreSQL StatefulSet with persistent storage |
| 03 | Independent | Intermediate | Implement a Redis cluster using StatefulSets |
| 04 | Challenge | Advanced | Handle StatefulSet scaling and rolling updates safely |
| 05 | Integration | Advanced | Design a database HA strategy with StatefulSets |

## Prerequisites

- A running Kubernetes cluster (minikube, kind, or cloud provider)
- `kubectl` configured to access your cluster
- Understanding of Pods, Deployments, Services, and PersistentVolumes (Modules 27, 28, 32)
- Familiarity with YAML syntax

## How to Use These Exercises

1. Start with Exercise 01 to understand the concepts
2. Work through exercises in order - they build on each other
3. Try to complete each exercise before looking at hints
4. Check your work against the success criteria
5. Review the solutions only after attempting the exercise

## Getting Started

```bash
# Verify your cluster is running
kubectl cluster-info

# Check available storage classes
kubectl get storageclass

# Create a namespace for these exercises
kubectl create namespace module33
```

## Key Concepts You'll Practice

- **StatefulSet**: Workload API for stateful applications needing stable identity
- **Headless Service**: Service with `clusterIP: None` that provides stable DNS for StatefulSet pods
- **volumeClaimTemplates**: Creates a separate PVC for each pod in the StatefulSet
- **Ordered Deployment**: Pods are created sequentially (0, 1, 2) and deleted in reverse (2, 1, 0)
- **Stable Network Identity**: DNS names like `{pod}.{service}.{namespace}.svc.cluster.local`
- **Partitioned Updates**: Rolling updates that start from a specific ordinal
