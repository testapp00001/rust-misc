# Exercise 05: Kustomize with ArgoCD for GitOps

**Type:** Integration
**Time:** 45 minutes
**Objective:** Integrate a Kustomize-based repository with ArgoCD to implement
a GitOps deployment workflow with automated sync and environment promotion.

---

## Background

Your team has adopted ArgoCD for GitOps. You need to configure ArgoCD to deploy
the multi-environment Kustomize structure from the previous exercises. The
requirements are:

1. Each environment (dev, staging, production) should be a separate ArgoCD
   Application.
2. Dev should auto-sync on every commit to the `dev` branch.
3. Staging should auto-sync but require manual approval for prune operations.
4. Production should require manual sync (no auto-sync) and use a
   `SyncWindow` to restrict deployments to business hours.
5. All environments should use the `git` repository type pointing to a
   specific repo URL.
6. Health checks should verify Deployments are at desired replica count before
   marking as synced.

---

## Tasks

### Task 1: ArgoCD Application for Dev

Create an ArgoCD `Application` manifest for the dev environment.

File: `exercise-05/argocd/app-dev.yaml`

Requirements:
- Application name: `webapp-dev`
- Namespace: `argocd`
- Source: Git repo `https://github.com/myorg/webapp-config.git`, revision
  `dev`, path `overlays/dev`
- Destination: namespace `dev` on the same cluster (`https://kubernetes.default.svc`)
- Sync policy: automated with `selfHeal: true` and `prune: true`
- Sync options: `CreateNamespace=true`
- Add the label `environment: dev`

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: webapp-dev
  namespace: argocd
spec:
  project: default
  source:
    repoURL: https://github.com/myorg/webapp-config.git
    targetRevision: dev
    path: overlays/dev
  destination:
    server: https://kubernetes.default.svc
    namespace: dev
  syncPolicy:
    automated:
      selfHeal: true
      prune: true
    syncOptions:
      - CreateNamespace=true
```

### Task 2: ArgoCD Application for Staging

Create an ArgoCD `Application` manifest for staging.

File: `exercise-05/argocd/app-staging.yaml`

Requirements:
- Application name: `webapp-staging`
- Source: Git repo, revision `main`, path `overlays/staging`
- Destination: namespace `staging`
- Sync policy: automated but with `prune: false` (manual pruning required)
- Add a `SyncPolicy` retry with limit 5 and backoff
- Add annotation `notifications.argoproj.io/subscribe.on-sync-succeeded.slack: staging-deploys`

### Task 3: ArgoCD Application for Production

Create an ArgoCD `Application` manifest for production.

File: `exercise-05/argocd/app-production.yaml`

Requirements:
- Application name: `webapp-production`
- Source: Git repo, revision `main`, path `overlays/production`
- Destination: namespace `production`
- **No** `syncPolicy.automated` -- all syncs are manual
- Add `ignoreDifferences` for the HPA's `currentReplicas` field (ArgoCD
  should not show drift when the HPA scales)
- Add a `SyncWindow` that only allows syncs during business hours (Monday-
  Friday, 9AM-5PM UTC) and denies syncs outside that window

### Task 4: ArgoCD AppProject

Create an ArgoCD `AppProject` manifest that scopes the webapp applications.

File: `exercise-05/argocd/project-webapp.yaml`

Requirements:
- Project name: `webapp`
- Source repositories: `https://github.com/myorg/webapp-config.git`
- Destination namespaces: `dev`, `staging`, `production`
- Destination clusters: `https://kubernetes.default.svc` (and optionally
  remote clusters)
- Deny all cluster-scoped resources except `Namespace` and
  `ResourceQuota`
- Add a role `deployer` with sync permissions and a binding to a group

### Task 5: Kustomize for ArgoCD Itself

Create a kustomization that deploys ArgoCD's own configuration using
Kustomize. This is meta: using Kustomize to manage ArgoCD Applications.

File: `exercise-05/argocd/kustomization.yaml`

This should:
1. List all three Application manifests as resources
2. Add the `project-webapp.yaml` as a resource
3. Add common labels: `app.kubernetes.io/part-of: webapp`
4. Optionally, reference the ArgoCD installation as a remote base:
   ```yaml
   resources:
     - https://github.com/argoproj/argo-cd//manifests/cluster-install?ref=v2.9.0
   ```

### Task 6: Deployment Workflow Documentation

Create a markdown file `exercise-05/workflow.md` that documents:

1. The branching strategy: which branches trigger which environments
2. The promotion process: how a change moves from dev to staging to production
3. The rollback process: how to revert a bad deployment in each environment
4. The emergency process: how to bypass sync windows for production hotfixes

---

## Success Criteria

- [ ] `app-dev.yaml` has automated sync with selfHeal and prune enabled.
- [ ] `app-staging.yaml` has automated sync but prune disabled.
- [ ] `app-production.yaml` has no automated sync policy.
- [ ] `app-production.yaml` has `ignoreDifferences` for HPA.
- [ ] `project-webapp.yaml` restricts destination namespaces.
- [ ] `project-webapp.yaml` has RBAC role and binding.
- [ ] `kustomization.yaml` can build all ArgoCD resources together.
- [ ] `workflow.md` documents the full lifecycle.

---

## Hints

<details>
<summary>Hint 1: ArgoCD Application Spec</summary>

An ArgoCD Application is a CRD. The key fields are:

```yaml
spec:
  project: default          # or your AppProject name
  source:
    repoURL: <git-url>
    targetRevision: <branch-or-tag>
    path: <kustomize-overlay-path>
  destination:
    server: <cluster-url>
    namespace: <target-namespace>
  syncPolicy:
    automated:
      selfHeal: true   # re-sync if someone manually changes resources
      prune: true       # delete resources removed from Git
```

</details>

<details>
<summary>Hint 2: SyncWindows</summary>

SyncWindows control when syncs can occur. Add them to the AppProject:

```yaml
spec:
  syncWindows:
    - kind: allow
      schedule: "0 9 * * 1-5"    # Mon-Fri at 9AM
      duration: 8h                 # for 8 hours
      applications:
        - webapp-production
      manualSync: true
    - kind: deny
      schedule: "0 0 * * *"
      duration: 24h
      applications:
        - webapp-production
```

The `manualSync: true` flag in the allow window means even manual syncs are
only allowed during the window.

</details>

<details>
<summary>Hint 3: ignoreDifferences</summary>

For HPA resources, ArgoCD may show drift when `currentReplicas` changes. Use
`ignoreDifferences` to suppress this:

```yaml
spec:
  ignoreDifferences:
    - group: autoscaling
      kind: HorizontalPodAutoscaler
      jsonPointers:
        - /status/currentReplicas
        - /status/currentMetrics
```

This tells ArgoCD to ignore changes in those fields when computing sync status.

</details>

<details>
<summary>Hint 4: AppProject RBAC</summary>

```yaml
spec:
  roles:
    - name: deployer
      policies:
        - p, proj:webapp:deployer, applications, sync, webapp/*, allow
        - p, proj:webapp:deployer, applications, get, webapp/*, allow
      groups:
        - my-team
```

This creates a role `deployer` in the `webapp` project that allows syncing
and viewing applications. The group `my-team` is bound to this role.

</details>

<details>
<summary>Hint 5: Kustomize Build for ArgoCD</summary>

You can manage ArgoCD Applications themselves with Kustomize. The
`kustomization.yaml` just needs to list the Application YAML files as
resources. When you run `kustomize build`, it produces the Application
manifests which you then `kubectl apply` to the `argocd` namespace.

This is the "App of Apps" pattern: one Kustomize build manages all ArgoCD
Applications.

</details>
