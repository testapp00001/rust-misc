# Solution 05: GitOps with Canary Deployment

## Part A: Repository Structure

```
gitops-canary/
  base/
    app/
      rollout.yaml
      analysis-template.yaml
      service-canary.yaml
      service-stable.yaml
      ingress.yaml
      kustomization.yaml
  overlays/
    production/
      kustomization.yaml
      patch-resources.yaml
      patch-replicas.yaml
  argocd/
    application.yaml
```

### Why This Works

The Rollout resource lives in the base alongside the standard Kubernetes
resources. The overlay customizes resource limits and replica counts.
ArgoCD watches the overlay directory and syncs everything to the cluster,
including the Rollout and AnalysisTemplate.

## Part B: Rollout Resource

```yaml
# base/app/rollout.yaml
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: myapp
spec:
  replicas: 10
  revisionHistoryLimit: 5
  selector:
    matchLabels:
      app.kubernetes.io/name: myapp
  template:
    metadata:
      labels:
        app.kubernetes.io/name: myapp
    spec:
      containers:
        - name: myapp
          image: myapp:PLACEHOLDER
          ports:
            - containerPort: 8080
          readinessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 10
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 15
            periodSeconds: 20
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 500m
              memory: 256Mi
  strategy:
    canary:
      steps:
        - setWeight: 5
        - pause: {duration: 2m}
        - analysis:
            templates:
              - templateName: canary-analysis
            args:
              - name: service-name
                value: myapp
        - setWeight: 25
        - pause: {duration: 3m}
        - analysis:
            templates:
              - templateName: canary-analysis
            args:
              - name: service-name
                value: myapp
        - setWeight: 50
        - pause: {duration: 3m}
        - analysis:
            templates:
              - templateName: canary-analysis
            args:
              - name: service-name
                value: myapp
        - setWeight: 100
      canaryService: myapp-canary
      stableService: myapp-stable
      trafficRouting:
        nginx:
          stableIngress: myapp-ingress
```

```yaml
# base/app/analysis-template.yaml
apiVersion: argoproj.io/v1alpha1
kind: AnalysisTemplate
metadata:
  name: canary-analysis
spec:
  args:
    - name: service-name
  metrics:
    - name: error-rate
      interval: 30s
      count: 10
      failureLimit: 1
      successCondition: result[0] <= 0.01
      provider:
        prometheus:
          address: http://prometheus:9090
          query: |
            sum(rate(http_requests_total{
              service="{{args.service-name}}",
              version="canary",
              status=~"5.."
            }[5m])) /
            sum(rate(http_requests_total{
              service="{{args.service-name}}",
              version="canary"
            }[5m]))
    - name: latency-p99
      interval: 1m
      count: 5
      failureLimit: 2
      successCondition: result[0] <= 2.0
      provider:
        prometheus:
          address: http://prometheus:9090
          query: |
            histogram_quantile(0.99,
              sum(rate(http_request_duration_seconds_bucket{
                service="{{args.service-name}}",
                version="canary"
              }[5m])) by (le)
            ) /
            histogram_quantile(0.99,
              sum(rate(http_request_duration_seconds_bucket{
                service="{{args.service-name}}",
                version="stable"
              }[5m])) by (le)
            )
```

```yaml
# overlays/production/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - ../../base/app
namespace: production
patches:
  - path: patch-resources.yaml
  - path: patch-replicas.yaml
images:
  - name: myapp
    newTag: v1.0.0
```

```yaml
# overlays/production/patch-replicas.yaml
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: myapp
spec:
  replicas: 20
```

```yaml
# overlays/production/patch-resources.yaml
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: myapp
spec:
  template:
    spec:
      containers:
        - name: myapp
          resources:
            requests:
              cpu: 500m
              memory: 512Mi
            limits:
              cpu: 2000m
              memory: 2Gi
```

### Why This Works

The `image: myapp:PLACEHOLDER` in the base is overridden by the Kustomize
`images` section in the overlay. When the CI pipeline updates the image
tag in the overlay, ArgoCD syncs the change, and Argo Rollouts starts
the canary.

The Rollout resource is patched by the overlay to customize replicas and
resources for production. This is the same Kustomize pattern used for
standard Deployments, but applied to Argo Rollouts resources.

## Part C: Image Update Workflow

### CI Pipeline Step

```yaml
# .github/workflows/build-and-update.yaml
name: Build and Update GitOps

on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Build and push image
        run: |
          docker build -t ghcr.io/myorg/myapp:${{ github.sha }} .
          docker push ghcr.io/myorg/myapp:${{ github.sha }}

  update-gitops:
    runs-on: ubuntu-latest
    needs: build
    steps:
      - name: Checkout GitOps repo
        uses: actions/checkout@v4
        with:
          repository: myorg/gitops-canary
          token: ${{ secrets.GITOPS_TOKEN }}

      - name: Update image tag
        run: |
          cd overlays/production
          kustomize edit set image myapp=ghcr.io/myorg/myapp:${{ github.sha }}

      - name: Commit and push
        run: |
          git config user.name "CI Bot"
          git config user.email "ci@myorg.com"
          git add .
          git commit -m "chore: update myapp to ${{ github.sha }}"
          git push
```

### Why This Works

The CI pipeline builds the image and updates the image tag in the GitOps
repo. ArgoCD detects the change and syncs it to the cluster. Argo Rollouts
sees the new image and starts the canary progression.

The `GITOPS_TOKEN` is a GitHub token with write access to the GitOps
repo. This is the only credential the CI system needs -- it never has
cluster access.

## Part D: Handle the Feedback Loop

### The Problem

Argo Rollouts manages the canary in the cluster. If the canary fails,
Argo Rollouts rolls back in the cluster. But the GitOps repo still has
the new image tag. This creates drift: Git says `v2.0.0` but the cluster
runs `v1.0.0`.

### Solution 1: Git as Desired State Only (Simple)

Accept that Git may temporarily differ from the cluster during canary
rollouts. The drift resolves when:
- Canary succeeds: cluster runs `v2.0.0`, matches Git
- Canary fails: a separate process reverts the Git commit

```yaml
# Webhook or CI job that reverts Git on canary failure
# ArgoCD sends a webhook to CI on sync failure
# CI reverts the image tag in Git
```

**Pros:** Simple, no additional infrastructure.
**Cons:** Git drift during canary rollout. Requires webhook integration.

### Solution 2: Git as Source of Truth (Recommended)

Use a controller or webhook to keep Git synchronized with the cluster:

```yaml
# .github/workflows/canary-rollback-handler.yaml
name: Handle Canary Rollback

on:
  repository_dispatch:
    types: [canary-failed]

jobs:
  revert:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout GitOps repo
        uses: actions/checkout@v4
        with:
          repository: myorg/gitops-canary
          token: ${{ secrets.GITOPS_TOKEN }}

      - name: Get current image
        id: current
        run: |
          cd overlays/production
          CURRENT=$(kustomize edit set image myapp= 2>/dev/null || echo "unknown")
          echo "current=$CURRENT" >> "$GITHUB_OUTPUT"

      - name: Revert to previous image
        run: |
          cd overlays/production
          # Revert the last commit that changed the image tag
          git revert HEAD --no-edit
          git push
```

```yaml
# ArgoCD notification configuration
# Send webhook to GitHub on canary abort
apiVersion: v1
kind: ConfigMap
metadata:
  name: argocd-notifications-cm
  namespace: argocd
data:
  service.github: |
    token: $github-token
  template.canary-failed: |
    message: |
      Canary for {{.app.metadata.name}} failed.
      Revision: {{.app.status.sync.revision}}
    github:
      status:
        state: failure
        description: "Canary deployment failed"
  trigger.on-aborted: |
    - when: app.status.operationState.phase in ['Failed', 'Error']
      send: [canary-failed]
```

### Solution 3: ArgoCD Image Updater (Automated)

Use the ArgoCD Image Updater, which automatically updates the image tag
in Git when a new image is pushed:

```yaml
# In the ArgoCD Application annotations
metadata:
  annotations:
    argocd-image-updater.argoproj.io/image-list: myapp=ghcr.io/myorg/myapp
    argocd-image-updater.argoproj.io/myapp.update-strategy: git
    argocd-image-updater.argoproj.io/myapp.git-repository: https://github.com/myorg/gitops-canary.git
    argocd-image-updater.argoproj.io/myapp.git-branch: main
```

**Pros:** Fully automated, no CI pipeline needed for Git updates.
**Cons:** Additional component to operate.

### Why This Works

The challenge is that canary outcomes are determined at runtime (analysis
passes or fails), but Git is a static record of desired state. The
solutions bridge this gap by:
1. Accepting temporary drift (simple)
2. Reverting Git on failure (recommended)
3. Automating Git updates (advanced)

The recommended approach (Solution 2) keeps Git as the source of truth
while handling the runtime feedback loop from Argo Rollouts.

## Common Mistakes to Avoid

- **Not handling canary rollback in Git.** If the canary fails and the
  cluster rolls back, but Git still has the new image tag, the next
  ArgoCD sync will try to deploy the failed image again.
- **Using `automated` sync with canary.** If ArgoCD auto-syncs on every
  Git change, it might interrupt an in-progress canary. Use manual sync
  or careful timing.
- **Not separating the image update from the canary trigger.** The CI
  pipeline should update Git. ArgoCD should sync. Argo Rollouts should
  manage the canary. If CI directly triggers the canary, you bypass GitOps.
- **Forgetting the AnalysisTemplate in the repo.** The AnalysisTemplate
  must be in the GitOps repo so ArgoCD syncs it to the cluster. If it
  is only in the cluster, it is not tracked by Git.

## Key Takeaway

Combining GitOps with canary deployment creates a declarative progressive
delivery system. Git defines the desired state (image tag, canary strategy).
ArgoCD syncs it to the cluster. Argo Rollouts manages the canary. The
key challenge is the feedback loop: canary outcomes are runtime decisions,
but Git is a static record. Solutions range from accepting temporary drift
to automated Git reverts on canary failure.
