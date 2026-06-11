# Cheatsheet: Kustomize

## Structure
```
base/
  kustomization.yaml
  deployment.yaml
  service.yaml

overlays/
  dev/
    kustomization.yaml
    patch-replicas.yaml
  staging/
    kustomization.yaml
  prod/
    kustomization.yaml
    patch-replicas.yaml
```

## Base kustomization.yaml
```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - deployment.yaml
  - service.yaml
commonLabels:
  app: my-app
```

## Overlay kustomization.yaml
```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - ../../base
patches:
  - path: patch-replicas.yaml
configMapGenerator:
  - name: app-config
    literals:
      - APP_ENV=production
```

## Common Commands
```bash
kubectl apply -k overlays/prod/       # Apply
kubectl kustomize overlays/prod/      # Preview
kubectl diff -k overlays/prod/        # Diff
```

## Kustomize vs Helm

| | Kustomize | Helm |
|---|---|---|
| Templating | No (patches) | Yes (Go templates) |
| Complexity | Simple | Complex |
| Learning curve | Low | Medium |
| Ecosystem | Smaller | Larger |
| Built into kubectl | Yes | No (plugin) |
