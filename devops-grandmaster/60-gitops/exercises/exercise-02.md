# Exercise 02: Set Up an ArgoCD Application

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Configure an ArgoCD Application resource that watches a Git repository
and automatically deploys a Kubernetes application. This exercise walks
you through the ArgoCD configuration that connects Git to the cluster.

## Scenario

You have a Git repository at `https://github.com/myorg/k8s-configs` with
this structure:

```
k8s-configs/
  apps/
    web-app/
      deployment.yaml
      service.yaml
      ingress.yaml
```

You need to configure ArgoCD to watch this path and deploy to the
`production` namespace.

## Tasks

### Part A: Write the ArgoCD Application

Create an ArgoCD `Application` resource that:
- Watches the `main` branch of the Git repo
- Deploys manifests from `apps/web-app/`
- Targets the `production` namespace
- Enables automatic sync with pruning and self-healing

<details>
<summary>Hint</summary>

The Application resource has three main sections: `source` (where to find
manifests), `destination` (where to deploy), and `syncPolicy` (how to
sync).

```yaml
spec:
  source:
    repoURL: https://github.com/myorg/k8s-configs.git
    targetRevision: main
    path: apps/web-app
  destination:
    server: https://kubernetes.default.svc
    namespace: production
```

</details>

### Part B: Configure Sync Policy

Explain each sync policy option and when to use it:
- `automated.prune`
- `automated.selfHeal`
- `syncOptions.CreateNamespace`
- `retry.limit` and `retry.backoff`

<details>
<summary>Hint</summary>

- `prune`: Deletes resources removed from Git. Without it, deleting a
  manifest from Git does not remove the resource from the cluster.
- `selfHeal`: Reverts manual changes to the cluster. Without it, someone
  can `kubectl edit` and the change persists.
- `CreateNamespace`: Creates the target namespace if it does not exist.
- `retry`: Retries failed syncs with exponential backoff.

</details>

### Part C: Write the Kubernetes Manifests

Create the `deployment.yaml`, `service.yaml`, and `ingress.yaml` files
that ArgoCD will deploy. The deployment should have:
- 3 replicas
- Health checks
- Resource limits
- A label that ArgoCD uses to track the resource

<details>
<summary>Hint</summary>

ArgoCD tracks resources by label. Use `app.kubernetes.io/instance`
to identify which ArgoCD application owns the resource. ArgoCD
automatically adds this label, but you can also set it explicitly.

</details>

### Part D: Design the Sync Workflow

Describe the complete workflow from a developer making a change to the
change being live in production. Include:
1. The developer's actions
2. The Git operations
3. What ArgoCD detects
4. What ArgoCD does

<details>
<summary>Hint</summary>

The workflow is:
1. Developer changes a YAML file in a branch
2. Developer creates a PR, gets it reviewed
3. PR is merged to main
4. ArgoCD detects the change (polls every 3 minutes by default)
5. ArgoCD diffs the desired state (Git) against the actual state (cluster)
6. ArgoCD applies the diff (creates, updates, or deletes resources)
7. ArgoCD reports sync status in the UI

</details>

## Success Criteria

- [ ] Application resource correctly points to the Git repo and path
- [ ] Sync policy includes prune and self-heal
- [ ] Kubernetes manifests are valid and include health checks
- [ ] You can describe the complete developer-to-production workflow
- [ ] You understand each sync policy option and its trade-offs

## What You Should Understand After This Exercise

An ArgoCD Application connects a Git repository path to a Kubernetes
namespace. The sync policy determines how ArgoCD handles differences
between Git and the cluster. With automated sync, prune, and self-heal,
the cluster continuously converges to the desired state in Git.
