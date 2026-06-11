# Solution 05: Kustomize with ArgoCD for GitOps

---

## Task 1: ArgoCD Application for Dev

### exercise-05/argocd/app-dev.yaml

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: webapp-dev
  namespace: argocd
  labels:
    environment: dev
    app.kubernetes.io/part-of: webapp
spec:
  project: webapp
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
      - PrunePropagationPolicy=foreground
      - PruneLast=true
    retry:
      limit: 5
      backoff:
        duration: 5s
        factor: 2
        maxDuration: 3m
```

**Explanation:**

- `selfHeal: true` means ArgoCD will automatically revert any manual changes
  to the cluster that drift from the Git state. If someone runs
  `kubectl edit deployment webapp` in the dev namespace, ArgoCD will revert it
  on the next sync cycle.
- `prune: true` means resources removed from Git will be deleted from the
  cluster.
- `CreateNamespace=true` ensures the `dev` namespace is created if it does not
  exist.
- `PrunePropagationPolicy=foreground` ensures dependent resources are deleted
  before the parent (prevents orphaned resources).
- `PruneLast=true` delays pruning until after all other resources are synced,
  reducing downtime during deployments.
- The retry policy handles transient failures (e.g., API server timeouts).

---

## Task 2: ArgoCD Application for Staging

### exercise-05/argocd/app-staging.yaml

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: webapp-staging
  namespace: argocd
  labels:
    environment: staging
    app.kubernetes.io/part-of: webapp
  annotations:
    notifications.argoproj.io/subscribe.on-sync-succeeded.slack: staging-deploys
    notifications.argoproj.io/subscribe.on-sync-failed.slack: staging-deploys
spec:
  project: webapp
  source:
    repoURL: https://github.com/myorg/webapp-config.git
    targetRevision: main
    path: overlays/staging
  destination:
    server: https://kubernetes.default.svc
    namespace: staging
  syncPolicy:
    automated:
      selfHeal: true
      prune: false
    syncOptions:
      - CreateNamespace=true
      - PrunePropagationPolicy=foreground
    retry:
      limit: 5
      backoff:
        duration: 5s
        factor: 2
        maxDuration: 3m
```

**Explanation:**

- `prune: false` means resources removed from Git are NOT automatically deleted.
  An operator must manually approve pruning. This prevents accidental deletion
  of resources during a bad merge or accidental removal from Git.
- `selfHeal: true` still reverts manual cluster changes, ensuring the cluster
  state matches Git.
- The Slack notification annotation sends a message to the `staging-deploys`
  channel on sync success and failure.
- The source uses `main` branch, not `dev`. Staging promotes from dev via
  merge to main.

---

## Task 3: ArgoCD Application for Production

### exercise-05/argocd/app-production.yaml

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: webapp-production
  namespace: argocd
  labels:
    environment: production
    app.kubernetes.io/part-of: webapp
  annotations:
    notifications.argoproj.io/subscribe.on-sync-succeeded.slack: production-deploys
    notifications.argoproj.io/subscribe.on-sync-failed.slack: production-alerts
    notifications.argoproj.io/subscribe.on-health-degraded.slack: production-alerts
spec:
  project: webapp
  source:
    repoURL: https://github.com/myorg/webapp-config.git
    targetRevision: main
    path: overlays/production
  destination:
    server: https://kubernetes.default.svc
    namespace: production
  ignoreDifferences:
    - group: autoscaling
      kind: HorizontalPodAutoscaler
      jsonPointers:
        - /status/currentReplicas
        - /status/currentMetrics
        - /status/lastScaleTime
    - group: apps
      kind: Deployment
      jsonPointers:
        - /spec/replicas
  syncPolicy:
    syncOptions:
      - CreateNamespace=true
      - PrunePropagationPolicy=foreground
      - RespectIgnoreDifferences=true
    retry:
      limit: 3
      backoff:
        duration: 10s
        factor: 2
        maxDuration: 5m
```

**Explanation:**

- **No `automated` field** under `syncPolicy`. This means all syncs are manual.
  An operator must explicitly trigger sync via the ArgoCD UI, CLI, or API.
- `ignoreDifferences` tells ArgoCD to ignore changes to HPA status fields.
  When the HPA scales the Deployment, `currentReplicas` changes. Without this
  ignore rule, ArgoCD would show the application as "OutOfSync" whenever the
  HPA scales.
- The Deployment's `spec/replicas` is also ignored because the HPA controls
  the replica count. If ArgoCD tried to sync replicas to the Git state, it
  would fight the HPA.
- `RespectIgnoreDifferences=true` ensures the ignored fields are also excluded
  during sync, not just during status comparison.

---

## Task 4: ArgoCD AppProject

### exercise-05/argocd/project-webapp.yaml

```yaml
apiVersion: argoproj.io/v1alpha1
kind: AppProject
metadata:
  name: webapp
  namespace: argocd
spec:
  description: Webapp multi-environment project

  sourceRepos:
    - "https://github.com/myorg/webapp-config.git"

  destinations:
    - server: https://kubernetes.default.svc
      namespace: dev
    - server: https://kubernetes.default.svc
      namespace: staging
    - server: https://kubernetes.default.svc
      namespace: production

  clusterResourceWhitelist:
    - group: ""
      kind: Namespace

  namespaceResourceBlacklist:
    - group: ""
      kind: ResourceQuota

  roles:
    - name: deployer
      description: Can sync webapp applications
      policies:
        - p, proj:webapp:deployer, applications, get, webapp/*, allow
        - p, proj:webapp:deployer, applications, sync, webapp/*, allow
        - p, proj:webapp:deployer, applications, action/*, webapp/*, allow
      groups:
        - my-team
        - platform-eng
      syncWindows:
        - kind: allow
          schedule: "0 9 * * 1-5"
          duration: 8h
          applications:
            - webapp-production
          manualSync: true
        - kind: deny
          schedule: "0 0 * * *"
          duration: 24h
          applications:
            - webapp-production
```

**Explanation:**

- `sourceRepos` restricts which Git repositories the project can reference.
  This prevents a compromised Application from pulling from an attacker-
  controlled repo.
- `destinations` restricts which namespaces and clusters the project can
  deploy to. An Application in this project cannot deploy to `kube-system` or
  a remote cluster unless explicitly listed.
- `clusterResourceWhitelist` allows only `Namespace` as a cluster-scoped
  resource. All other cluster-scoped resources (ClusterRole, ClusterRoleBinding)
  are denied.
- `namespaceResourceBlacklist` denies `ResourceQuota` within the allowed
  namespaces. This prevents Applications from accidentally modifying resource
  quotas.
- The `deployer` role grants get, sync, and action permissions on all
  applications in the `webapp` project. The `action/*` permission allows
  running custom actions (e.g., restart Deployment).
- **SyncWindows** in the project define when syncs are allowed:
  - The `allow` window permits syncs Monday-Friday, 9AM-5PM UTC. The
    `manualSync: true` flag means even manual syncs must occur within this
    window.
  - The `deny` window blocks all syncs outside the allow window.

---

## Task 5: Kustomize for ArgoCD Itself

### exercise-05/argocd/kustomization.yaml

```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

resources:
  - project-webapp.yaml
  - app-dev.yaml
  - app-staging.yaml
  - app-production.yaml

commonLabels:
  app.kubernetes.io/part-of: webapp
  app.kubernetes.io/managed-by: kustomize

commonAnnotations:
  app.kubernetes.io/version: "1.0.0"
```

**Build and apply:**

```bash
# Preview the output
kustomize build exercise-05/argocd/

# Apply to the cluster
kustomize build exercise-05/argocd/ | kubectl apply -f -

# Or using kubectl directly
kubectl apply -k exercise-05/argocd/
```

**App of Apps pattern:** This `kustomization.yaml` manages all ArgoCD
Application resources. When you add a new environment (e.g., staging-canary),
you create the Application YAML, add it to `resources`, and commit. ArgoCD
picks up the change and creates the new Application, which in turn manages
the actual Kubernetes resources.

---

## Task 6: Workflow Documentation

### exercise-05/workflow.md

```markdown
# Webapp GitOps Deployment Workflow

## Branching Strategy

| Branch | Environment | Sync Mode | Promotion |
|--------|-------------|-----------|-----------|
| `dev`  | dev | Auto-sync (selfHeal, prune) | Push to dev branch |
| `main` | staging | Auto-sync (selfHeal, no prune) | Merge dev into main |
| `main` | production | Manual sync | Tag release, sync via UI/CLI |

## Promotion Process

### Dev to Staging
1. Feature branch is merged into `dev` via PR.
2. ArgoCD auto-syncs the dev environment.
3. QA tests in dev.
4. When ready, create a PR from `dev` to `main`.
5. After merge, ArgoCD auto-syncs staging.
6. QA tests in staging.

### Staging to Production
1. Staging is verified by QA and product.
2. Create a Git tag: `git tag v1.2.0 && git push origin v1.2.0`.
3. Update `targetRevision` in `app-production.yaml` to `v1.2.0` (or use
   a `main` branch reference and sync to a specific commit SHA).
4. An operator triggers sync via ArgoCD UI or CLI:
   ```bash
   argocd app sync webapp-production
   ```
5. Monitor the sync. If health checks fail, do not promote.

### Production Rollback
1. Revert the last commit on `main` or move the tag to the previous version.
2. Trigger sync: `argocd app sync webapp-production`.
3. ArgoCD applies the previous state. HPA and Deployment roll back.
4. Alternatively, use ArgoCD rollback:
   ```bash
   argocd app history webapp-production
   argocd app rollback webapp-production <revision>
   ```

## Emergency Process (Bypass Sync Windows)

Sync windows restrict production syncs to business hours. For emergencies:

1. **Option A: Suspend the sync window.** In the ArgoCD UI, navigate to the
   `webapp` project, find the sync window, and click "Suspend." This
   temporarily disables the window, allowing immediate sync. Re-enable after
   the emergency.

2. **Option B: Update the AppProject.** Edit `project-webapp.yaml` to remove
   or widen the sync window. Commit and apply:
   ```bash
   kubectl apply -f exercise-05/argocd/project-webapp.yaml
   argocd app sync webapp-production
   ```
   Revert the window change after the emergency.

3. **Option C: Use a dedicated emergency override.** Add a sync window with
   `manualSync: false` that covers emergencies:
   ```yaml
   - kind: allow
     schedule: "0 0 * * *"
     duration: 1h
     applications:
       - webapp-production
     manualSync: false
   ```
   This allows automated syncs at midnight, which is not ideal. The better
   approach is Option A (suspend the window).

## Monitoring and Alerting

- ArgoCD notifications send Slack messages on sync success/failure.
- Health status changes (Progressing, Degraded, Healthy) trigger alerts.
- The ArgoCD dashboard shows real-time sync and health status for all
  environments.
```

---

## Why It Works

**Automated vs. Manual sync** is the core GitOps control mechanism. Dev uses
full automation for fast feedback. Staging uses automation without pruning for
safety. Production uses manual sync for human oversight.

**`ignoreDifferences`** prevents ArgoCD from fighting the HPA. Without it,
ArgoCD sees the current replica count differs from the Git-specified count and
marks the app as OutOfSync. Adding `/spec/replicas` to the ignore list for
Deployments ensures the HPA owns replica scaling.

**AppProject** is ArgoCD's RBAC boundary. It scopes what repositories can be
used, what destinations are allowed, and who can do what. Without a project,
any Application can deploy anything anywhere.

**SyncWindows** enforce deployment discipline. They prevent accidental
off-hours deployments and provide a policy layer on top of the technical sync
configuration.

**Kustomize managing ArgoCD resources** enables the App of Apps pattern. One
`kustomize build` produces all Application manifests. Adding a new environment
is a Git commit, not a manual `kubectl apply`.

---

## Common Mistakes

1. **Setting `prune: true` in production.** If someone accidentally removes a
   resource YAML from Git, ArgoCD deletes it from the production cluster. Use
   `prune: false` or require manual pruning approval.

2. **Not configuring `ignoreDifferences` for HPA.** Without it, the
   application constantly shows as OutOfSync, leading to alert fatigue and
   confusion about whether the deployment is actually healthy.

3. **Using `automated` sync in production without `selfHeal`.** Without
   `selfHeal`, manual changes to the cluster persist until the next Git commit
   triggers a sync. With `selfHeal`, ArgoCD reverts drift automatically.

4. **Not restricting `sourceRepos` in the AppProject.** A compromised or
   misconfigured Application could point to an attacker-controlled repository.
   Always restrict source repos to your organization's repos.

5. **Forgetting that SyncWindows apply to manual syncs too.** The
   `manualSync: true` flag on an allow window means even explicit sync
   commands are blocked outside the window. If you need emergency access, you
   must suspend the window first.

6. **Using `targetRevision: main` for production.** A branch reference means
   every commit to main triggers a potential sync. For production, pin to a
   tag or commit SHA for immutability:
   ```yaml
   targetRevision: v1.2.0  # or a full commit SHA
   ```
