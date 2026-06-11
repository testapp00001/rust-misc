# Exercise 03: Design a Multi-Environment GitOps Repo

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective

Design a Git repository structure that supports multiple environments
(dev, staging, production) using Kustomize overlays and ArgoCD
Applications. This exercise trains you to think about how GitOps handles
environment promotion.

## Scenario

Your organization has three services (`api`, `web`, `worker`) that deploy
to three environments (`dev`, `staging`, `production`). Each environment
has different:
- Replica counts (dev: 1, staging: 2, production: 5)
- Resource limits (dev: small, staging: medium, production: large)
- Ingress hostnames (dev.example.com, staging.example.com, api.example.com)

## Tasks

### Part A: Design the Repository Structure

Draw the complete directory tree for the GitOps repository. Use Kustomize
base/overlay pattern. Show how shared base manifests are customized per
environment.

<details>
<summary>Hint</summary>

Use this structure:
```
gitops-repo/
  base/
    api/
      deployment.yaml
      service.yaml
      kustomization.yaml
    web/
      ...
    worker/
      ...
  overlays/
    dev/
      api/
        kustomization.yaml
        patch-replicas.yaml
      web/
        ...
      worker/
        ...
    staging/
      ...
    production/
      ...
```

</details>

### Part B: Write the Kustomize Overlays

Write the Kustomize overlay files for the `api` service in the
`production` environment. The overlay should:
- Set replicas to 5
- Set resource requests to 500m CPU, 512Mi memory
- Set resource limits to 2000m CPU, 2Gi memory
- Set the namespace to `production`
- Set the ingress hostname to `api.example.com`

<details>
<summary>Hint</summary>

The overlay `kustomization.yaml` references the base and applies patches:

```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - ../../base/api
namespace: production
patches:
  - path: patch-replicas.yaml
  - path: patch-resources.yaml
  - path: patch-ingress.yaml
```

</details>

### Part C: Write the ArgoCD Applications

Create ArgoCD Application resources for each environment. Design the
sync policies differently for each environment:
- **dev:** Auto-sync, auto-prune, self-heal (deploy on every commit)
- **staging:** Auto-sync, auto-prune, self-heal (deploy on every commit)
- **production:** Manual sync (require human approval to deploy)

<details>
<summary>Hint</summary>

For production, do not use `automated` sync policy. Instead, use manual
sync via the ArgoCD UI or CLI:

```yaml
spec:
  syncPolicy:
    syncOptions:
      - CreateNamespace=true
    # No "automated" section = manual sync
```

</details>

### Part D: Design the Promotion Workflow

Describe how code moves from dev to staging to production. Include:
1. What triggers promotion to staging?
2. What triggers promotion to production?
3. How does the Git workflow support this?

<details>
<summary>Hint</summary>

Consider two approaches:
1. **Branch-based:** `develop` branch -> dev, `main` branch -> staging/production
2. **Directory-based:** Same branch, different overlay directories. Promotion
   is a PR that updates the image tag in the target overlay.

</details>

## Success Criteria

- [ ] Repository structure uses Kustomize base/overlay pattern
- [ ] Overlays customize replicas, resources, and ingress per environment
- [ ] ArgoCD Applications are defined for each environment
- [ ] Dev and staging use auto-sync; production uses manual sync
- [ ] Promotion workflow is clearly defined with Git operations

## What You Should Understand After This Exercise

Multi-environment GitOps uses Kustomize overlays to customize shared base
manifests for each environment. ArgoCD watches different paths (or branches)
for each environment. Promotion is a Git operation (PR merge) that moves
code from one environment's configuration to the next.
