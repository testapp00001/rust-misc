# Module 34: DaemonSets and Jobs - Exercises

## Overview

These exercises will teach you when and how to use DaemonSets, Jobs, and CronJobs in Kubernetes. You'll learn to deploy node-level agents, run batch tasks, and schedule recurring work.

## Exercise List

| Exercise | Type | Difficulty | Description |
|----------|------|------------|-------------|
| 01 | Conceptual | Beginner | DaemonSet vs Deployment vs Job - when to use each |
| 02 | Guided | Intermediate | Deploy a logging agent as a DaemonSet |
| 03 | Independent | Intermediate | Create a CronJob for database backups |
| 04 | Challenge | Advanced | Implement a job queue with retry and backoff |
| 05 | Integration | Advanced | Design batch processing with Jobs and monitoring with DaemonSets |

## Prerequisites

- A running Kubernetes cluster (minikube, kind, or cloud provider)
- `kubectl` configured to access your cluster
- Basic understanding of Pods, Deployments, and Services
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

# Check node count (important for DaemonSet exercises)
kubectl get nodes

# Create a namespace for these exercises
kubectl create namespace module34
```

## Key Concepts You'll Practice

- **DaemonSet**: Ensures a pod runs on every node (or a subset)
- **Job**: Runs pods to completion for batch tasks
- **CronJob**: Schedules Jobs to run at specific times
- **Backoff Limit**: Controls retry behavior for failed jobs
- **Completions vs Parallelism**: Managing batch job execution
