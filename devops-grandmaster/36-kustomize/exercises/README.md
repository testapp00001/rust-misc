# Module 36: Kustomize - Exercises

## Overview

Kustomize is Kubernetes-native configuration management that lets you customize
YAML manifests without templating. It uses a declarative overlay model where
you define a **base** configuration and then layer **patches**, **generators**,
and **transformers** on top for different environments or use cases.

Unlike Helm, Kustomize does not use Go templates or require packaging charts.
It is built into `kubectl` (v1.14+) via `kubectl apply -k` and produces
deterministic, plain YAML output.

## Prerequisites

- Kubernetes cluster (minikube, kind, or k3d)
- kubectl v1.14+ with built-in Kustomize support
- Kustomize standalone binary (v4.0+) - `kustomize version`
- Basic understanding of Kubernetes resources (Deployments, Services, ConfigMaps)

## Exercise Index

| # | Title | Type | Focus |
|---|-------|------|-------|
| 01 | Kustomize vs Helm | Conceptual | Decision framework, trade-offs |
| 02 | Base Configuration and Overlays | Guided | Directory structure, bases, overlays |
| 03 | Patches per Environment | Independent | Strategic and JSON patches |
| 04 | Multi-Cluster Strategy | Challenge | Generators, transformers, composition |
| 05 | Kustomize with ArgoCD | Integration | GitOps workflows, Application CRDs |

## How to Use These Exercises

1. Read the exercise file completely before starting.
2. Work through the steps in order -- each builds on the previous.
3. Use `kustomize build` or `kubectl apply -k` to validate your work.
4. Check your output against the success criteria before looking at solutions.
5. Solutions are in the `../solutions/` directory.

## Directory Convention

Each exercise expects you to create files under a working directory structure
that mirrors standard Kustomize layouts:

```
project/
  base/
    kustomization.yaml
    deployment.yaml
    service.yaml
  overlays/
    dev/
      kustomization.yaml
      patch.yaml
    staging/
      kustomization.yaml
    production/
      kustomization.yaml
```
