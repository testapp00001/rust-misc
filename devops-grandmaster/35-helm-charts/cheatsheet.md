# Cheatsheet: Helm Charts

## Common Commands
```bash
# Repository management
helm repo add bitnami https://charts.bitnami.com/bitnami
helm repo update

# Install
helm install my-release bitnami/nginx
helm install my-release ./my-chart
helm install my-release ./my-chart -f values.yaml
helm install my-release ./my-chart --set key=value

# Upgrade
helm upgrade my-release bitnami/nginx
helm upgrade my-release ./my-chart -f values.yaml

# Rollback
helm rollback my-release 1

# List/Status
helm list
helm status my-release
helm history my-release

# Uninstall
helm uninstall my-release

# Template rendering
helm template my-release ./my-chart
```

## Chart Structure
```
my-chart/
  Chart.yaml          # Chart metadata
  values.yaml         # Default values
  charts/             # Dependencies
  templates/          # Kubernetes manifests
    deployment.yaml
    service.yaml
    ingress.yaml
    _helpers.tpl      # Template helpers
```

## values.yaml Override
```bash
# File override
helm install my-release ./my-chart -f production-values.yaml

# Inline override
helm install my-release ./my-chart --set replicaCount=5

# Multiple overrides
helm install my-release ./my-chart -f base.yaml -f prod.yaml
```
