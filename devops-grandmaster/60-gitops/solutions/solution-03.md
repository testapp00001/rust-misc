# Solution 03: Design a Multi-Environment GitOps Repo

## Part A: Repository Structure

```
gitops-repo/
  base/
    api/
      deployment.yaml
      service.yaml
      ingress.yaml
      kustomization.yaml
    web/
      deployment.yaml
      service.yaml
      ingress.yaml
      kustomization.yaml
    worker/
      deployment.yaml
      service.yaml
      kustomization.yaml
  overlays/
    dev/
      kustomization.yaml
      namespace.yaml
      patches/
        api-replicas.yaml
        api-resources.yaml
        web-replicas.yaml
        web-resources.yaml
        worker-replicas.yaml
        worker-resources.yaml
    staging/
      kustomization.yaml
      namespace.yaml
      patches/
        api-replicas.yaml
        api-resources.yaml
        api-ingress.yaml
        web-replicas.yaml
        web-resources.yaml
        web-ingress.yaml
        worker-replicas.yaml
        worker-resources.yaml
    production/
      kustomization.yaml
      namespace.yaml
      patches/
        api-replicas.yaml
        api-resources.yaml
        api-ingress.yaml
        web-replicas.yaml
        web-resources.yaml
        web-ingress.yaml
        worker-replicas.yaml
        worker-resources.yaml
  argocd/
    dev-application.yaml
    staging-application.yaml
    production-application.yaml
```

### Why This Works

The base directory contains the default manifests shared across all
environments. Each overlay customizes these manifests for a specific
environment. The argocd directory defines the ArgoCD Applications that
watch each overlay.

## Part B: Kustomize Overlays

```yaml
# base/api/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - deployment.yaml
  - service.yaml
  - ingress.yaml
commonLabels:
  app.kubernetes.io/name: api
```

```yaml
# base/api/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api
spec:
  replicas: 1
  selector:
    matchLabels:
      app.kubernetes.io/name: api
  template:
    metadata:
      labels:
        app.kubernetes.io/name: api
    spec:
      containers:
        - name: api
          image: myapp-api:latest
          ports:
            - containerPort: 8080
          readinessProbe:
            httpGet:
              path: /health
              port: 8080
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 500m
              memory: 256Mi
```

```yaml
# overlays/production/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - ../../base/api
  - ../../base/web
  - ../../base/worker
namespace: production
patches:
  - path: patches/api-replicas.yaml
  - path: patches/api-resources.yaml
  - path: patches/api-ingress.yaml
  - path: patches/web-replicas.yaml
  - path: patches/web-resources.yaml
  - path: patches/web-ingress.yaml
  - path: patches/worker-replicas.yaml
  - path: patches/worker-resources.yaml
images:
  - name: myapp-api
    newTag: v1.2.3
  - name: myapp-web
    newTag: v1.2.3
  - name: myapp-worker
    newTag: v1.2.3
```

```yaml
# overlays/production/patches/api-replicas.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api
spec:
  replicas: 5
```

```yaml
# overlays/production/patches/api-resources.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api
spec:
  template:
    spec:
      containers:
        - name: api
          resources:
            requests:
              cpu: 500m
              memory: 512Mi
            limits:
              cpu: 2000m
              memory: 2Gi
```

```yaml
# overlays/production/patches/api-ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: api
spec:
  rules:
    - host: api.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: api
                port:
                  number: 80
```

## Part C: ArgoCD Applications

```yaml
# argocd/dev-application.yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: dev
  namespace: argocd
spec:
  project: default
  source:
    repoURL: https://github.com/myorg/gitops-repo.git
    targetRevision: develop
    path: overlays/dev
  destination:
    server: https://kubernetes.default.svc
    namespace: dev
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
      - CreateNamespace=true
```

```yaml
# argocd/staging-application.yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: staging
  namespace: argocd
spec:
  project: default
  source:
    repoURL: https://github.com/myorg/gitops-repo.git
    targetRevision: main
    path: overlays/staging
  destination:
    server: https://kubernetes.default.svc
    namespace: staging
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
      - CreateNamespace=true
```

```yaml
# argocd/production-application.yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: production
  namespace: argocd
spec:
  project: default
  source:
    repoURL: https://github.com/myorg/gitops-repo.git
    targetRevision: main
    path: overlays/production
  destination:
    server: https://kubernetes.default.svc
    namespace: production
  syncPolicy:
    syncOptions:
      - CreateNamespace=true
    # No "automated" section = manual sync
```

### Why This Works

Dev and staging have `automated` sync, so every commit to their branch
triggers a deployment. Production has no `automated` section, so
deployments require manual action (ArgoCD UI or CLI).

The `targetRevision` differs: dev watches `develop`, staging and
production watch `main`. This means:
- Pushing to `develop` deploys to dev
- Merging to `main` deploys to staging automatically
- Production requires manual sync after staging is verified

## Part D: Promotion Workflow

### Directory-Based Promotion (Recommended)

```
1. Developer pushes to `develop` branch
2. ArgoCD auto-syncs to dev environment
3. Developer verifies in dev
4. Developer creates PR: `develop` -> `main`
5. PR is reviewed and merged
6. ArgoCD auto-syncs to staging
7. QA verifies in staging
8. Engineer manually syncs production in ArgoCD UI
```

### Image Tag Update Workflow

```
1. CI builds image: myapp-api:abc123
2. CI updates overlays/dev/kustomization.yaml:
   images:
     - name: myapp-api
       newTag: abc123
3. CI commits and pushes to develop
4. ArgoCD deploys to dev
5. After verification, same change is cherry-picked to main
6. ArgoCD deploys to staging
7. After QA, engineer syncs production
```

### Why This Works

Promotion is a Git operation. Moving code from dev to staging is a PR
merge. Moving from staging to production is a manual sync. This creates
a clear audit trail: who promoted, when, and what changed.

## Common Mistakes to Avoid

- **Using the same branch for all environments.** Without branch
  separation, every commit deploys everywhere. Use `develop` for dev
  and `main` for staging/production.
- **Auto-syncing production.** Production should always require human
  approval. Auto-sync means a bad commit goes to production immediately.
- **Not using Kustomize image overrides.** Hardcoding image tags in base
  manifests means every environment gets the same image. Use Kustomize
  `images` to override tags per environment.
- **Not pinning the base image tag.** Using `latest` in the base means
  the image changes unpredictably. Always use a specific tag or SHA.

## Key Takeaway

Multi-environment GitOps uses Kustomize overlays to customize shared
manifests for each environment. ArgoCD Applications watch different
paths or branches. Promotion is a Git operation (PR merge or manual
sync). This creates a clear, auditable deployment pipeline where Git
is the single source of truth.
