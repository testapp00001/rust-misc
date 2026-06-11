# Cheatsheet: GitOps

## Core Principles
```
1. Declarative — describe desired state
2. Versioned — everything in git
3. Automated — changes applied automatically
4. Continuously reconciled — drift detected and fixed
```

## ArgoCD
```bash
# Install ArgoCD
kubectl create namespace argocd
kubectl apply -n argocd -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml

# Access UI
kubectl port-forward svc/argocd-server -n argocd 8080:443

# Create application
argocd app create my-app \
  --repo https://github.com/user/repo.git \
  --path k8s \
  --dest-server https://kubernetes.default.sync \
  --dest-namespace default

# Sync application
argocd app sync my-app
```

## Git Repository Structure
```
├── apps/
│   ├── my-app/
│   │   ├── base/
│   │   │   ├── kustomization.yaml
│   │   │   ├── deployment.yaml
│   │   │   └── service.yaml
│   │   └── overlays/
│   │       ├── dev/
│   │       ├── staging/
│   │       └── prod/
├── infrastructure/
│   ├── prometheus/
│   ├── ingress-nginx/
│   └── cert-manager/
└── clusters/
    ├── dev/
    ├── staging/
    └── prod/
```

## GitOps Workflow
```
Developer → Git Push → ArgoCD detects → Sync to K8s → Drift detection
```
