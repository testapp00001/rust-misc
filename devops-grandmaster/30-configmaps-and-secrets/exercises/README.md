# Module 30: ConfigMaps & Secrets - Exercises

## Overview

ConfigMaps and Secrets are Kubernetes primitives for injecting configuration
into containers without rebuilding images. ConfigMaps hold non-sensitive
configuration (feature flags, database hostnames, log levels). Secrets hold
sensitive data (passwords, API keys, TLS certificates). Both can be consumed
as environment variables or mounted as files, but they differ in encoding,
encryption at rest, and update behavior.

These exercises progress from understanding the conceptual differences between
ConfigMaps and Secrets, through hands-on creation and consumption, to
production-grade patterns like secret rotation and external secret management.

## Prerequisites

- Kubernetes cluster (minikube, kind, or k3d)
- kubectl configured and connected to the cluster
- Basic understanding of Pods and Deployments (Modules 26-27)
- Familiarity with environment variables and volume mounts

## Exercise Index

| # | Title | Type | Focus |
|---|-------|------|-------|
| 01 | ConfigMap vs Secret | Conceptual | When to use each, encoding vs encryption, update behavior |
| 02 | ConfigMaps as Files and Env Vars | Guided | Creating, consuming, and verifying ConfigMaps |
| 03 | Secret Rotation Without Restart | Independent | Volume-mounted secrets, auto-update, atomicity |
| 04 | Configuration Management Across Environments | Challenge | Multi-env overlays, immutable configs, promotion |
| 05 | External Secrets Operator | Integration | Vault, ESO CRDs, sync policies, drift detection |

## How to Use These Exercises

1. Read the exercise file completely before starting.
2. Work through the tasks in order -- each builds on the previous.
3. Try to answer without using hints first.
4. Verify your work against the success criteria before looking at solutions.
5. Solutions are in the `../solutions/` directory.
