# Exercise 05: GitOps with Canary Deployment

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Combine GitOps (Module 60) with canary deployment (Module 59) to create
a system where canary rollouts are managed declaratively through Git. This
exercise integrates two powerful patterns into a single workflow.

## Scenario

Your team uses ArgoCD for GitOps and Argo Rollouts for canary deployments.
You need to design a workflow where:

1. A developer updates the image tag in a Git repo
2. ArgoCD syncs the change to the cluster
3. Argo Rollouts manages the canary progression automatically
4. The canary analysis determines whether to promote or rollback
5. The final state (promoted or rolled back) is reflected in Git

## Tasks

### Part A: Design the Repository Structure

Design the Git repo structure that supports both ArgoCD Applications and
Argo Rollouts resources. The repo should have:
- Base manifests for the application
- ArgoCD Application definitions
- Rollout and AnalysisTemplate resources
- Environment-specific overlays

<details>
<summary>Hint</summary>

```
gitops-canary/
  base/
    app/
      rollout.yaml          # Argo Rollouts Rollout resource
      analysis-template.yaml # Analysis definitions
      service.yaml
      ingress.yaml
      kustomization.yaml
  overlays/
    production/
      kustomization.yaml
      patch-image.yaml       # Image tag override
  argocd/
    application.yaml         # ArgoCD Application definition
```

</details>

### Part B: Write the Rollout Resource

Write the Argo Rollouts `Rollout` resource that defines the canary
strategy. The image tag should be parameterized so it can be overridden
by the Kustomize overlay.

<details>
<summary>Hint</summary>

In the base `rollout.yaml`, use a placeholder image tag:

```yaml
containers:
  - name: app
    image: myapp:PLACEHOLDER
```

In the production overlay, use a Kustomize image override:

```yaml
# kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - ../../base/app
images:
  - name: myapp
    newTag: v1.2.3
```

</details>

### Part C: Design the Image Update Workflow

Design the process for updating the image tag. This should be:
1. A CI pipeline builds and pushes a new image
2. The CI pipeline updates the image tag in the GitOps repo
3. ArgoCD syncs the change
4. Argo Rollouts starts the canary

Write the CI pipeline step that updates the image tag in Git.

<details>
<summary>Hint</summary>

The CI pipeline can use a tool like `kustomize edit` to update the image
tag, then commit and push:

```yaml
- name: Update image tag
  run: |
    cd gitops-canary/overlays/production
    kustomize edit set image myapp=ghcr.io/myorg/myapp:${{ github.sha }}
    git add .
    git commit -m "chore: update myapp to ${{ github.sha }}"
    git push
```

Or use a specialized tool like ArgoCD Image Updater that automatically
updates the image tag in Git when a new image is pushed.

</details>

### Part D: Handle the Feedback Loop

Design how the canary analysis result (promote or rollback) is reflected
back in Git. This is the hardest part: Argo Rollouts manages the canary
in the cluster, but Git should reflect the final state.

<details>
<summary>Hint</summary>

There are two approaches:

1. **Git as desired state only:** Git specifies the image tag and canary
   strategy. Argo Rollouts manages the actual traffic weights. If the
   canary fails, Argo Rollouts rolls back in the cluster. The Git repo
   still has the new image tag, but the cluster runs the old version.
   This creates drift.

2. **Git as source of truth:** A webhook or controller detects when Argo
   Rollouts aborts and reverts the image tag in Git. This keeps Git
   synchronized with the cluster.

Approach 2 is more aligned with GitOps principles but requires additional
automation.

</details>

## Success Criteria

- [ ] Repository structure supports ArgoCD and Argo Rollouts together
- [ ] Rollout resource defines a canary strategy with analysis
- [ ] Image tag is parameterized for Kustomize overlay
- [ ] CI pipeline updates the image tag in Git and triggers ArgoCD sync
- [ ] The feedback loop from canary analysis back to Git is designed

## What You Should Understand After This Exercise

Combining GitOps with canary deployment creates a declarative progressive
delivery system. Git defines the desired state (image tag, canary strategy).
ArgoCD syncs it to the cluster. Argo Rollouts manages the canary. The
challenge is keeping Git synchronized with the cluster when the canary
outcome (promote or rollback) is determined at runtime.
