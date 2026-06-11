# Module 14: Secrets Management -- Exercises

## Overview

These exercises reinforce the core concepts from [Module 14: Secrets Management](../README.md).
Complete them in order -- each builds on the previous one.

## Exercise List

| # | Name | Type | Time | Difficulty |
|---|------|------|------|------------|
| 01 | Why Environment Variables Fail for Secrets | Conceptual | 15 min | Easy |
| 02 | Docker Secrets for a Swarm Service | Guided | 30 min | Easy-Medium |
| 03 | Kubernetes Secrets with Encryption at Rest | Independent | 45 min | Medium |
| 04 | Secrets Rotation with Zero Downtime | Challenge | 45 min | Medium-Hard |
| 05 | Secrets Management Strategy with Vault | Integration | 60 min | Hard |

## How to Use These Exercises

1. Read the exercise file completely before starting.
2. Try to solve it without looking at hints first.
3. Use the `<details>` hint sections only if you are stuck.
4. After completing an exercise, check your answer against the solution.
5. If your answer differs from the solution, that is fine -- understand *why* both approaches work.

## Prerequisites

- You have read the [Module 14 README](../README.md).
- Docker Engine and Docker Compose installed.
- (Exercise 02) Docker Swarm initialized (`docker swarm init`).
- (Exercises 03-05) A local Kubernetes cluster (minikube, kind, or Docker Desktop) with `kubectl` configured.
- (Exercise 05) HashiCorp Vault CLI installed or Docker available to run Vault.
- A text editor for writing YAML, shell scripts, and configuration files.

## Getting Help

If you get stuck:
- Re-read the relevant section of the module notes.
- Check `docker secret --help` and `kubectl create secret --help` documentation.
- Look at the hint sections (hidden behind `<details>` tags).
- As a last resort, review the solution file -- but try to understand *why*
  the solution works, not just copy it.

## Solutions

See the [solutions directory](../solutions/) for detailed solutions with explanations.
