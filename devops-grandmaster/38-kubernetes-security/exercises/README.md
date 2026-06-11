# Module 38: Kubernetes Security -- Exercises

## Overview

These exercises cover the core pillars of Kubernetes security: Pod Security
Standards, RBAC, image scanning, seccomp, AppArmor, and runtime security
policies. Each exercise builds on the previous one so complete them in order.

## Prerequisites

- A running Kubernetes cluster (minikube, kind, or k3s)
- `kubectl` configured and working
- `trivy` installed (Exercise 04)
- Basic familiarity with YAML and Kubernetes manifests

## Exercise List

| # | Title | Type | Estimated Time |
|---|-------|------|----------------|
| 01 | Pod Security Standards Explained | Conceptual | 20 min |
| 02 | Harden a Deployment with Security Contexts and PSA | Guided | 30 min |
| 03 | Implement RBAC with Least Privilege | Independent | 45 min |
| 04 | Scan and Fix Vulnerabilities in a Production Image | Challenge | 40 min |
| 05 | Design a Complete Kubernetes Security Policy | Integration | 60 min |

## How to Use

1. Read the exercise markdown file.
2. Follow the instructions and write your manifests or answers.
3. Apply them to your cluster to verify behaviour.
4. Check your work against the success criteria.
5. Compare with the solutions only after attempting the exercise yourself.
