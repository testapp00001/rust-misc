# Module 27: Deployments and ReplicaSets -- Exercises

## Overview

These exercises build your understanding of Kubernetes Deployments and ReplicaSets from the ground up. You will start by explaining how Deployments manage ReplicaSets conceptually, then progress through creating Deployments with rolling updates, performing rollbacks and scaling, designing zero-downtime deployment strategies, and finally building a complete deployment pipeline with health checks.

Each exercise increases in difficulty and independence. The first verifies your conceptual understanding, the second walks you through creation step by step, the third asks you to solve from scratch, the fourth simulates a real production scenario requiring careful strategy selection, and the fifth integrates Deployments with health checks and readiness gates into a full pipeline.

## Exercise List

| # | Name | Type | Time | Difficulty |
|---|------|------|------|------------|
| 01 | How Deployments Manage ReplicaSets | Conceptual | 20 min | Easy |
| 02 | Create a Deployment with Rolling Update | Guided | 30 min | Easy-Medium |
| 03 | Rollback and Scaling Operations | Independent | 30 min | Medium |
| 04 | Zero-Downtime Deployment Strategies | Challenge | 45 min | Medium-Hard |
| 05 | Deployment Pipeline with Health Checks | Integration | 45 min | Hard |

## How to Use These Exercises

1. Read the module README first. These exercises assume you understand the concepts explained there.
2. Attempt each exercise without looking at the solutions. Struggle is part of learning.
3. Use hints only after you have spent genuine effort on the problem.
4. Check your work against the success criteria before moving to the next exercise.
5. Review the solutions even if you completed the exercise -- they often contain additional context.

## Prerequisites

- A running Kubernetes cluster (minikube, kind, or k3s)
- kubectl configured and able to connect to the cluster
- Basic understanding of YAML syntax
- Completion of Module 26 (Pods and Containers)

## Solutions

Solutions are available in the [solutions directory](../solutions/). Use them to verify your work, not to avoid the learning process.
