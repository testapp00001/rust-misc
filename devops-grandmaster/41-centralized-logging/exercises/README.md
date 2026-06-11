# Module 41: Centralized Logging -- Exercises

## Overview

These exercises build your understanding of centralized logging from
foundational architecture concepts through production-grade log aggregation
with Loki, Fluentd, and Grafana. Complete them in order -- each builds on the
previous one.

## Exercise List

| # | Name | Type | Time | Difficulty |
|---|------|------|------|------------|
| 01 | Centralized Logging Architecture and Design Decisions | Conceptual | 25 min | Easy |
| 02 | Set Up Loki with Promtail for Kubernetes Log Collection | Guided | 45 min | Easy-Medium |
| 03 | Write LogQL Queries and Build Grafana Dashboards | Independent | 45 min | Medium |
| 04 | Build Log-Based Alerting for Error Patterns | Challenge | 60 min | Medium-Hard |
| 05 | Design a Logging Strategy Correlating Logs, Metrics, and Traces | Integration | 60 min | Hard |

## How to Use These Exercises

1. Work through them in order -- each builds on the previous one.
2. Read the objective and instructions carefully before starting.
3. Use the hints only after you have attempted the exercise yourself.
4. Check your work against the success criteria listed at the end of each exercise.
5. Compare your solution with the corresponding file in `../solutions/`.

## Prerequisites

- Docker Engine 24+ installed and running
- Docker Compose v2 installed
- A Kubernetes cluster (minikube, kind, or k3d) for Exercises 02 and 04
- Helm 3 installed
- Basic familiarity with Kubernetes resources (Deployments, DaemonSets, Pods)
- Understanding of Prometheus and Grafana from earlier observability modules
- A terminal and a text editor

## Getting Help

If you get stuck:

- Re-read the relevant section of the [module notes](../README.md).
- Check the [cheatsheet](../cheatsheet.md) for LogQL syntax and configuration
  snippets.
- Look at the hint sections (hidden behind `<details>` tags).
- Review the Grafana Loki documentation at https://grafana.com/docs/loki/.
- As a last resort, review the solution file -- but try to understand *why*
  the solution works, not just copy it.

## Solutions

See the [solutions directory](../solutions/) for detailed solutions with
explanations.
