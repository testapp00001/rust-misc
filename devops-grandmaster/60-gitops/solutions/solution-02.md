# Solution 02: Set Up an ArgoCD Application

## Part A: ArgoCD Application

```yaml
# argocd-application.yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: web-app
  namespace: argocd
  finalizers:
    - resources-finalizer.argocd.argoproj.io
spec:
  project: default
  source:
    repoURL: https://github.com/myorg/k8s-configs.git
    targetRevision: main
    path: apps/web-app
  destination:
    server: https://kubernetes.default.svc
    namespace: production
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
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

### Why This Works

- `source.path`: Points to the specific directory in the Git repo.
  ArgoCD only watches this path, not the entire repo.
- `targetRevision: main`: ArgoCD watches the `main` branch. Changes to
  other branches are ignored.
- `destination.namespace`: All resources in the manifests are deployed
  to the `production` namespace (overrides any namespace in the manifests).
- `finalizer`: When the Application is deleted, the finalizer ensures
  all managed resources are cleaned up first.

## Part B: Sync Policy Explained

### `automated.prune`

When a resource is removed from Git, ArgoCD deletes it from the cluster.
Without `prune`, the resource stays in the cluster indefinitely.

**When to use:** Almost always. Without prune, deleting a manifest from
Git has no effect on the cluster, which defeats the purpose of GitOps.

**Risk:** If someone accidentally deletes a manifest file, the resource
is deleted from the cluster. Use branch protection to prevent this.

### `automated.selfHeal`

When someone makes a manual change to the cluster (e.g., `kubectl edit`),
ArgoCD reverts it to match Git. Without `selfHeal`, manual changes persist.

**When to use:** In production, almost always. In development, you might
want to allow manual changes for debugging.

**Risk:** Emergency manual changes (e.g., scaling up during an incident)
are reverted. Use the escape hatch pattern (see Exercise 04).

### `syncOptions.CreateNamespace`

Creates the target namespace if it does not exist. Without this, ArgoCD
fails if the namespace is missing.

**When to use:** Always. It is a convenience that prevents a common
failure mode.

### `syncOptions.PrunePropagationPolicy`

Controls how dependent resources are deleted. `foreground` means ArgoCD
waits for dependent resources to be deleted before deleting the parent.

**When to use:** When resources have owner references or finalizers that
require ordered deletion.

### `retry`

Automatically retries failed syncs with exponential backoff. Without
retry, a transient failure (e.g., API server temporarily overloaded)
requires manual intervention.

**When to use:** Always. Transient failures are common, and retry handles
them automatically.

## Part C: Kubernetes Manifests

```yaml
# apps/web-app/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-app
  labels:
    app.kubernetes.io/name: web-app
    app.kubernetes.io/instance: web-app
spec:
  replicas: 3
  selector:
    matchLabels:
      app.kubernetes.io/name: web-app
  template:
    metadata:
      labels:
        app.kubernetes.io/name: web-app
    spec:
      containers:
        - name: web-app
          image: myapp:v1
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
```

```yaml
# apps/web-app/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: web-app
  labels:
    app.kubernetes.io/name: web-app
spec:
  selector:
    app.kubernetes.io/name: web-app
  ports:
    - port: 80
      targetPort: 8080
  type: ClusterIP
```

```yaml
# apps/web-app/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: web-app
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
spec:
  rules:
    - host: web-app.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: web-app
                port:
                  number: 80
```

## Part D: Sync Workflow

```
Developer                 Git                    ArgoCD              Cluster
    |                      |                       |                   |
    |  Edit YAML file      |                       |                   |
    |  Create PR           |                       |                   |
    |--------------------->|                       |                   |
    |                      |                       |                   |
    |  Code review         |                       |                   |
    |  PR approved         |                       |                   |
    |  Merge to main       |                       |                   |
    |--------------------->|                       |                   |
    |                      |                       |                   |
    |                      |  Poll (every 3 min)   |                   |
    |                      |<----------------------|                   |
    |                      |                       |                   |
    |                      |  New revision found   |                   |
    |                      |---------------------->|                   |
    |                      |                       |                   |
    |                      |                       |  Diff Git vs      |
    |                      |                       |  Cluster          |
    |                      |                       |                   |
    |                      |                       |  Apply diff       |
    |                      |                       |------------------>|
    |                      |                       |                   |
    |                      |                       |  Report status    |
    |                      |                       |<------------------|
    |                      |                       |                   |
    |  View in ArgoCD UI   |                       |                   |
    |<---------------------------------------------------------------|
```

### Why This Works

The developer never touches the cluster. Their only interaction is with
Git (edit, PR, merge). ArgoCD handles the rest. This separation of
concerns means:
- Developers focus on the desired state (Git)
- ArgoCD handles the implementation (cluster)
- The Git history is the audit trail

## Common Mistakes to Avoid

- **Not setting `prune`.** Without prune, deleting a manifest from Git
  does not remove the resource from the cluster. This leads to orphaned
  resources.
- **Not setting `selfHeal`.** Without self-heal, manual changes persist
  and the cluster drifts from Git. This undermines the entire GitOps model.
- **Watching the entire repo.** If the `path` is set to the repo root,
  ArgoCD watches every file in the repo. This is slow and can cause
  unintended syncs. Always specify a specific path.
- **Not using `CreateNamespace`.** If the namespace does not exist,
  ArgoCD fails with a cryptic error. `CreateNamespace=true` prevents this.

## Key Takeaway

An ArgoCD Application connects Git to the cluster. The sync policy
determines how ArgoCD handles differences. With automated sync, prune,
and self-heal, the cluster continuously converges to the desired state
in Git. The developer workflow is simple: edit YAML, create PR, merge.
ArgoCD handles everything else.
