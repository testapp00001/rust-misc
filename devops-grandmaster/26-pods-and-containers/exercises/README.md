# Module 26: Pods and Containers -- Exercises

## Overview

These exercises build your understanding of Kubernetes Pods from the ground up. You will start by explaining Pod lifecycle phases conceptually, then progress through creating Pods with init containers and resource limits, implementing sidecar patterns, debugging failing Pods, and finally designing complete multi-container Pod specifications for a real application.

Each exercise increases in difficulty and independence. The first verifies your conceptual understanding, the second walks you through creation step by step, the third asks you to solve from scratch, the fourth simulates a real production debugging scenario, and the fifth integrates Pods with broader application architecture.

## Exercise List

| # | Name | Type | Time | Difficulty |
|---|------|------|------|------------|
| 01 | Pod Lifecycle Phases | Conceptual | 20 min | Easy |
| 02 | Init Containers and Resource Limits | Guided | 30 min | Easy-Medium |
| 03 | Sidecar Logging Pattern | Independent | 30 min | Medium |
| 04 | Debugging CrashLoopBackOff | Challenge | 45 min | Medium-Hard |
| 05 | Multi-Container Application Design | Integration | 45 min | Hard |

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
- Familiarity with container concepts from earlier modules

## Solutions

Solutions are available in the [solutions directory](../solutions/). Use them to verify your work, not to avoid the learning process.
