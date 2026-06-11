# Module 12: Health Checks -- Exercises

## Overview

These exercises build your understanding of container health checks from
foundational concepts through production-grade health check design. Complete
them in order -- each builds on the previous one.

## Exercise List

| # | Name | Type | Time | Difficulty |
|---|------|------|------|------------|
| 01 | Liveness vs Readiness vs Startup Probes | Conceptual | 20 min | Easy |
| 02 | Add Health Check Endpoints to an Application | Guided | 30 min | Easy-Medium |
| 03 | Configure Kubernetes Probes for a Stateful App | Independent | 45 min | Medium |
| 04 | Design Health Checks for Partial Failures | Challenge | 60 min | Medium-Hard |
| 05 | Health Checks Integrated with Monitoring | Integration | 45 min | Hard |

## How to Use These Exercises

1. Work through them in order -- each builds on the previous one.
2. Read the objective and instructions carefully before starting.
3. Use the hints only after you have attempted the exercise yourself.
4. Check your work against the success criteria listed at the end of each exercise.
5. Compare your solution with the corresponding file in `../solutions/`.

## Prerequisites

- Docker Engine 24+ installed and running
- Docker Compose v2 installed
- Python 3.10+ installed
- Basic familiarity with `docker run`, `docker ps`, and `docker inspect`
- (Exercises 03-05) A running Kubernetes cluster (minikube, kind, or k3s)
- A terminal and a text editor

## Getting Help

If you get stuck:

- Re-read the relevant section of the [module notes](../README.md).
- Check `docker inspect --format='{{json .State.Health}}'` for health details.
- Look at the hint sections (hidden behind `<details>` tags).
- Review the [cheatsheet](../cheatsheet.md) for quick command reference.
- As a last resort, review the solution file -- but try to understand *why*
  the solution works, not just copy it.

## Solutions

See the [solutions directory](../solutions/) for detailed solutions with
explanations.
