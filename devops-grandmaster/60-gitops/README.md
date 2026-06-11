# 60 - GitOps

## Problem

You have multiple services, each with Kubernetes manifests, Helm charts, ConfigMaps, Secrets references, Rollout definitions, and Ingress rules. Developers make changes by running `kubectl apply` from their laptops. Someone pushes a hotfix directly to production. Another person edits a ConfigMap in the cluster but forgets to update the repo. Within weeks, nobody knows what is actually running in production versus what is checked into Git. Drift accumulates, rollbacks become guesswork, and audit trails disappear.

GitOps fixes this by making a Git repository the single source of truth for your entire system state. A controller watches the repo and continuously reconciles the cluster to match it. Every change goes through a pull request, gets reviewed, and is automatically applied. If someone makes a manual change in the cluster, the controller reverts it. Your Git history becomes your audit log, and `git revert` becomes your rollback mechanism.

## Naive Way

Write a script that runs `kubectl apply` on a directory of YAML files, triggered by a CI pipeline on every push.

```bash
#!/bin/bash
# deploy.sh - CI-triggered deployment script

set -e

REPO_DIR="/tmp/infra-configs"
CLUSTER_NAME="production"

# Clone the config repo
git clone https://github.com/myorg/infra-configs.git "$REPO_DIR"
cd "$REPO_DIR"

# Apply all manifests in order
kubectl apply -f namespaces/
sleep 5
kubectl apply -f crds/
sleep 10
kubectl apply -f deployments/
kubectl apply -f services/
kubectl apply -f ingress/

echo "Deployed successfully"
```

```yaml
# .github/workflows/deploy.yaml
name: Deploy
on:
  push:
    branches: [main]
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup kubeconfig
        run: |
          mkdir -p ~/.kube
          echo "${{ secrets.KUBECONFIG }}" | base64 -d > ~/.kube/config
      - name: Deploy
        run: ./deploy.sh
```

This is better than manual kubectl, but it is still a push-based model. CI has cluster credentials, which is a security risk. There is no drift detection: if someone changes something in the cluster, nothing corrects it. There is no automatic sync when the repo changes outside of a push event. Rollbacks require a new commit and a new CI run.

## Right Way

Use a GitOps controller like ArgoCD or Flux that runs inside your cluster, watches your Git repo, and continuously reconciles the desired state.

**ArgoCD Application:**

```yaml
# argocd-application.yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: my-service
  namespace: argocd
  finalizers:
    - resources-finalizer.argocd.argoproj.io
spec:
  project: default
  source:
    repoURL: https://github.com/myorg/infra-configs.git
    targetRevision: main
    path: services/my-service
  destination:
    server: https://kubernetes.default.svc
    namespace: production
  syncPolicy:
    automated:
      prune: true          # Delete resources removed from Git
      selfHeal: true        # Revert manual cluster changes
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

```bash
# Install ArgoCD
kubectl create namespace argocd
kubectl apply -n argocd \
  -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml

# Get initial admin password
kubectl -n argocd get secret argocd-initial-admin-secret \
  -o jsonpath="{.data.password}" | base64 -d

# Access the UI
kubectl port-forward svc/argocd-server -n argocd 8080:443

# Apply the application definition
kubectl apply -f argocd-application.yaml

# Check sync status
argocd app get my-service
```

**Flux v2 alternative:**

```bash
# Install Flux CLI
curl -s https://fluxcd.io/install.sh | bash

# Bootstrap Flux (connects to your Git repo)
flux bootstrap github \
  --owner=myorg \
  --repository=infra-configs \
  --branch=main \
  --path=clusters/production \
  --personal
```

```yaml
# flux-source.yaml - tells Flux where to find the repo
apiVersion: source.toolkit.fluxcd.io/v1
kind: GitRepository
metadata:
  name: infra-configs
  namespace: flux-system
spec:
  interval: 1m
  url: https://github.com/myorg/infra-configs.git
  ref:
    branch: main
  secretRef:
    name: git-credentials
---
# flux-kustomization.yaml - tells Flux what to apply
apiVersion: kustomize.toolkit.fluxcd.io/v1
kind: Kustomization
metadata:
  name: my-service
  namespace: flux-system
spec:
  interval: 5m
  path: ./services/my-service
  prune: true
  sourceRef:
    kind: GitRepository
    name: infra-configs
  healthChecks:
    - apiVersion: apps/v1
      kind: Deployment
      name: my-service
      namespace: production
  timeout: 3m
```

## Production Way

A production GitOps setup uses the App of Apps pattern, sealed secrets, diff strategies, notifications, and multi-environment promotion.

**App of Apps pattern (ArgoCD):**

```yaml
# applications/root-app.yaml - single app that manages all other apps
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: root
  namespace: argocd
spec:
  project: default
  source:
    repoURL: https://github.com/myorg/infra-configs.git
    targetRevision: main
    path: applications
  destination:
    server: https://kubernetes.default.svc
    namespace: argocd
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
```

```yaml
# applications/my-service.yaml - individual app definition
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: my-service-production
  namespace: argocd
  annotations:
    notifications.argoproj.io/subscribe.on-sync-failed.slack: deployments
    notifications.argoproj.io/subscribe.on-health-degraded.slack: deployments
spec:
  project: production
  source:
    repoURL: https://github.com/myorg/infra-configs.git
    targetRevision: main
    path: services/my-service/overlays/production
  destination:
    server: https://kubernetes.default.svc
    namespace: production
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
```

**Multi-environment promotion with Kustomize overlays:**

```
infra-configs/
  services/
    my-service/
      base/
        deployment.yaml
        service.yaml
        kustomization.yaml
      overlays/
        staging/
          kustomization.yaml
          patch-replicas.yaml
        production/
          kustomization.yaml
          patch-replicas.yaml
          patch-resources.yaml
```

```yaml
# overlays/staging/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - ../../base
namePrefix: staging-
namespace: staging
patches:
  - path: patch-replicas.yaml
```

```yaml
# overlays/production/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - ../../base
namespace: production
patches:
  - path: patch-replicas.yaml
  - path: patch-resources.yaml
```

**Sealed Secrets for encrypting secrets in Git:**

```bash
# Install sealed-secrets controller
kubectl apply -f https://github.com/bitnami-labs/sealed-secrets/releases/latest/download/controller.yaml

# Create a sealed secret
echo -n mypassword | kubectl create secret generic db-password \
  --dry-run=client --from-file=password=/dev/stdin -o yaml | \
  kubeseal -o yaml > sealed-db-password.yaml

# This file is safe to commit to Git
cat sealed-db-password.yaml
```

```yaml
# sealed-db-password.yaml (safe to commit)
apiVersion: bitnami.com/v1alpha1
kind: SealedSecret
metadata:
  name: db-password
  namespace: production
spec:
  encryptedData:
    password: AgBy3i4OJSWK+PiTySYZZA9rO43cGDEq...
```

**Notification configuration:**

```yaml
# argocd-notifications-cm.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: argocd-notifications-cm
  namespace: argocd
data:
  service.slack: |
    token: $slack-token
  template.app-sync-succeeded: |
    message: |
      Application {{.app.metadata.name}} sync succeeded.
      Revision: {{.app.status.sync.revision}}
  template.app-sync-failed: |
    message: |
      Application {{.app.metadata.name}} sync FAILED.
      Error: {{.app.status.operationState.message}}
```

## Hands-On Lab

**Exercise: Set up GitOps with ArgoCD on a local cluster**

1. Create a cluster and install ArgoCD:

```bash
kind create cluster --name gitops-lab

kubectl create namespace argocd
kubectl apply -n argocd \
  -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml

# Wait for ArgoCD to be ready
kubectl wait --for=condition=available deployment/argocd-server \
  -n argocd --timeout=300s

# Get admin password
PASS=$(kubectl -n argocd get secret argocd-initial-admin-secret \
  -o jsonpath="{.data.password}" | base64 -d)
echo "Admin password: $PASS"

# Port forward
kubectl port-forward svc/argocd-server -n argocd 8080:443 &
```

2. Create a Git repo with application manifests:

```bash
mkdir -p gitops-lab-repo/apps/nginx
cd gitops-lab-repo
git init

cat > apps/nginx/deployment.yaml <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nginx
spec:
  replicas: 2
  selector:
    matchLabels:
      app: nginx
  template:
    metadata:
      labels:
        app: nginx
    spec:
      containers:
        - name: nginx
          image: nginx:1.24
          ports:
            - containerPort: 80
EOF

cat > apps/nginx/service.yaml <<EOF
apiVersion: v1
kind: Service
metadata:
  name: nginx
spec:
  selector:
    app: nginx
  ports:
    - port: 80
      targetPort: 80
EOF

git add . && git commit -m "initial: nginx app"
git remote add origin https://github.com/YOUR_USER/gitops-lab-repo.git
git push -u origin main
```

3. Create an ArgoCD Application:

```bash
argocd app create nginx \
  --repo https://github.com/YOUR_USER/gitops-lab-repo.git \
  --path apps/nginx \
  --dest-server https://kubernetes.default.svc \
  --dest-namespace default \
  --sync-policy automated \
  --auto-prune \
  --self-heal

# Watch it sync
argocd app get nginx
```

4. Test drift detection and self-healing:

```bash
# Manually scale the deployment (introduce drift)
kubectl scale deployment nginx --replicas=5

# Watch ArgoCD detect and revert the drift
argocd app get nginx
# Within seconds, it should show "OutOfSync" then auto-correct back to 2
```

5. Test Git-based rollback:

```bash
# Change the image in Git
cd gitops-lab-repo
sed -i 's/nginx:1.24/nginx:1.25/' apps/nginx/deployment.yaml
git add . && git commit -m "upgrade nginx to 1.25"
git push

# Watch ArgoCD sync the change
argocd app get nginx

# Rollback by reverting the commit
git revert HEAD
git push

# ArgoCD automatically rolls back
argocd app get nginx
```

## Limitation

GitOps gives you declarative, auditable, self-healing infrastructure management. But it treats every deployment the same: when a new commit lands on main, it gets synced. There is no concept of decoupling deployment from release. You cannot ship code to production dark (deployed but not exposed to users) and then flip a switch to enable it for specific users or cohorts. For that, you need feature flags.

## Next Topic

[61 - Feature Flags](../61-feature-flags/README.md) - Decoupling deployment from release with feature flags, enabling trunk-based development and progressive feature rollouts.
