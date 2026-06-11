# Module 79 -- Change Management: Exercises

## Overview

These exercises build practical skills in managing infrastructure and application
changes safely, predictably, and with minimal disruption. You will progress from
foundational concepts through hands-on Kubernetes rolling updates, maintenance
window design, automated rollback systems, and finally a full change management
pipeline with approval gates.

## Prerequisites

- Basic Kubernetes knowledge (Deployments, Services, Pods)
- Familiarity with YAML and command-line tooling
- Understanding of CI/CD concepts
- A running Kubernetes cluster (minikube, kind, or cloud-managed) for exercises 2-5
- `kubectl` configured and working

## Exercise List

| # | Title | Type | Focus |
|---|-------|------|-------|
| 1 | Change Management Principles | Conceptual | ITIL-aligned change management theory, risk classification, RFC lifecycle |
| 2 | Rolling Updates for Kubernetes | Guided | `maxUnavailable`, `maxSurge`, readiness probes, rollout status |
| 3 | Maintenance Window Procedure | Independent | Scheduling, communication plans, pre/post checks, stakeholder coordination |
| 4 | Automated Rollback System | Challenge | Health monitoring, automated detection of failures, rollback triggers and execution |
| 5 | Change Management Pipeline with Approval Gates | Integration | CI/CD pipeline stages, manual and automated approvals, audit trails |

## How to Use These Exercises

1. **Read the objective first.** Each exercise states exactly what you must
   produce or demonstrate.
2. **Work through the instructions step by step.** Hints are hidden behind
   `<details>` tags -- try without hints first.
3. **Check success criteria** before considering an exercise complete.
4. **Compare with the solution** only after attempting the exercise yourself.

## Estimated Time

| Exercise | Estimated Duration |
|----------|--------------------|
| 1 | 30-45 minutes |
| 2 | 45-60 minutes |
| 3 | 60-90 minutes |
| 4 | 90-120 minutes |
| 5 | 120-180 minutes |

## Directory Structure

```
79-change-management/
  exercises/
    README.md          <-- you are here
    exercise-01.md     -- Conceptual
    exercise-02.md     -- Guided
    exercise-03.md     -- Independent
    exercise-04.md     -- Challenge
    exercise-05.md     -- Integration
  solutions/
    README.md
    solution-01.md
    solution-02.md
    solution-03.md
    solution-04.md
    solution-05.md
```
