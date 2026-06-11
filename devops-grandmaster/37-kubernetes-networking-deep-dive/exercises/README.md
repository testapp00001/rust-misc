# Module 37: Kubernetes Networking Deep Dive - Exercises

## Overview

Kubernetes networking is fundamentally different from traditional infrastructure.
Every Pod gets its own IP address, every Pod can communicate with every other Pod
without NAT, and Services provide stable endpoints for unstable backends. CNI
plugins implement this networking model, NetworkPolicies enforce segmentation,
and CoreDNS handles service discovery.

These exercises build from conceptual understanding through hands-on
troubleshooting to designing and implementing production-grade network security
for multi-tier applications.

## Prerequisites

- Kubernetes cluster with a CNI plugin that supports NetworkPolicy (Calico or
  Cilium recommended -- Flannel does NOT enforce NetworkPolicies)
- kubectl configured and connected to the cluster
- Basic understanding of Kubernetes resources (Deployments, Services, Namespaces)
- `nicolaka/netshoot` or `busybox` image available for debug Pods

## Exercise Index

| # | Title | Type | Focus |
|---|-------|------|-------|
| 01 | CNI, NetworkPolicy, and DNS | Conceptual | How Kubernetes networking works |
| 02 | Default-Deny with Selective Allow | Guided | NetworkPolicy zero-trust pattern |
| 03 | Debug Cross-Namespace Connectivity | Independent | Troubleshooting network issues |
| 04 | Multi-Tier Network Segmentation | Challenge | Layered policy design |
| 05 | Complete Network Security Policy | Integration | Production-ready policy set |

## How to Use These Exercises

1. Read the exercise file completely before starting.
2. Work through the steps in order -- each builds on the previous.
3. Apply YAML manifests to your cluster and verify behavior with kubectl.
4. Check your output against the success criteria before looking at solutions.
5. Solutions are in the `../solutions/` directory.

## Directory Convention

Each exercise expects you to create YAML manifests in a working directory:

```
exercise-0X/
  namespace.yaml
  network-policy.yaml
  deployment.yaml
  service.yaml
```

## Verifying Your CNI Supports NetworkPolicy

Before starting, confirm your cluster enforces NetworkPolicies:

```bash
# Check which CNI is installed
kubectl get pods -n kube-system | grep -E "calico|cilium|flannel|weave"

# Create a test: deny all ingress, then verify traffic is blocked
# If traffic still flows, your CNI does not support NetworkPolicy
```

**Warning:** If you use Flannel, NetworkPolicy resources will be silently
ignored. Switch to Calico or Cilium for these exercises.
