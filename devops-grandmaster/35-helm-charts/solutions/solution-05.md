# Solution 05: Helm-Based Deployment Pipeline

## deploy.sh

```bash
#!/usr/bin/env bash
set -euo pipefail

# ─── Input validation ──────────────────────────────────────────────

ENV="${1:?Usage: deploy.sh <environment> <action> [version]}"
ACTION="${2:?Usage: deploy.sh <environment> <action> [version]}"
VERSION="${3:-}"

RELEASE="webapp-${ENV}"
NAMESPACE="${ENV}"
CHART="./webapp-chart"
VALUES="${CHART}/values/${ENV}.yaml"

if [[ ! "$ENV" =~ ^(dev|staging|production)$ ]]; then
  echo "ERROR: environment must be dev, staging, or production"
  exit 1
fi

if [[ ! "$ACTION" =~ ^(install|upgrade|rollback|status|uninstall)$ ]]; then
  echo "ERROR: action must be install, upgrade, rollback, status, or uninstall"
  exit 1
fi

if [[ ! -f "$VALUES" ]]; then
  echo "ERROR: values file not found: $VALUES"
  exit 1
fi

# ─── Helper functions ──────────────────────────────────────────────

log() {
  echo "==> $*"
}

lint() {
  log "Linting chart..."
  helm lint "$CHART" --strict
}

validate_templates() {
  log "Validating rendered templates..."
  helm template "$RELEASE" "$CHART" -f "$VALUES" > /dev/null
  log "Template rendering succeeded."
}

ensure_namespace() {
  if ! kubectl get namespace "$NAMESPACE" >/dev/null 2>&1; then
    log "Creating namespace $NAMESPACE..."
    kubectl create namespace "$NAMESPACE"
  fi
}

pre_deploy() {
  lint
  validate_templates
  ensure_namespace
}

smoke_test() {
  log "Running smoke test..."
  local port=8080
  local svc_name
  svc_name=$(helm template "$RELEASE" "$CHART" -f "$VALUES" | \
    grep -m1 'name:' | awk '{print $2}' | tr -d '"')

  # Attempt port-forward in background
  kubectl port-forward "svc/${svc_name}" "${port}:80" \
    -n "$NAMESPACE" >/dev/null 2>&1 &
  local pf_pid=$!
  sleep 3

  local result=0
  if curl -sf "http://localhost:${port}/" >/dev/null 2>&1; then
    log "Smoke test passed."
  else
    log "Smoke test FAILED."
    result=1
  fi

  kill "$pf_pid" 2>/dev/null || true
  wait "$pf_pid" 2>/dev/null || true
  return $result
}

post_deploy() {
  log "Waiting for rollout..."
  kubectl rollout status deployment/"${RELEASE}-webapp" \
    -n "$NAMESPACE" --timeout=300s

  smoke_test
}

auto_rollback() {
  log "Rolling back $RELEASE due to failure..."
  helm rollback "$RELEASE" 0 --namespace "$NAMESPACE" --wait
  log "Rollback complete."
}

# ─── Trap for automatic rollback on deploy failure ─────────────────

trap 'if [[ "$ACTION" =~ ^(install|upgrade)$ ]]; then auto_rollback; fi' ERR

# ─── Main ──────────────────────────────────────────────────────────

case "$ACTION" in
  install)
    if [[ -z "$VERSION" ]]; then
      echo "ERROR: version is required for install"
      exit 1
    fi
    pre_deploy
    log "Installing $RELEASE (version $VERSION) to $ENV..."
    helm install "$RELEASE" "$CHART" \
      -f "$VALUES" \
      --namespace "$NAMESPACE" --create-namespace \
      --version "$VERSION" \
      --wait --timeout 5m
    post_deploy
    log "Install complete. Release: $RELEASE, Namespace: $NAMESPACE"
    ;;

  upgrade)
    pre_deploy
    log "Upgrading $RELEASE in $ENV..."
    helm upgrade "$RELEASE" "$CHART" \
      -f "$VALUES" \
      --namespace "$NAMESPACE" \
      --wait --timeout 5m \
      ${VERSION:+--version "$VERSION"}
    post_deploy
    log "Upgrade complete."
    ;;

  rollback)
    REVISION="${VERSION:-}"
    log "Rolling back $RELEASE${REVISION:+ to revision $REVISION}..."
    helm rollback "$RELEASE" "$REVISION" \
      --namespace "$NAMESPACE" --wait
    log "Rollback complete."
    ;;

  status)
    log "Status of $RELEASE:"
    helm status "$RELEASE" --namespace "$NAMESPACE"
    echo ""
    log "History of $RELEASE:"
    helm history "$RELEASE" --namespace "$NAMESPACE"
    ;;

  uninstall)
    log "Uninstalling $RELEASE from $NAMESPACE..."
    helm uninstall "$RELEASE" --namespace "$NAMESPACE"
    log "Uninstall complete."
    ;;
esac
```

Make the script executable:

```bash
chmod +x deploy.sh
```

## GitHub Actions workflow

Create `.github/workflows/helm-deploy.yml`:

```yaml
name: Helm Deploy Pipeline

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  lint:
    name: Lint and Validate
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Helm
        uses: azure/setup-helm@v3
        with:
          version: v3.14.0

      - name: Lint chart
        run: helm lint ./webapp-chart --strict

      - name: Render templates (dev)
        run: helm template test ./webapp-chart -f webapp-chart/values/dev.yaml

      - name: Render templates (staging)
        run: helm template test ./webapp-chart -f webapp-chart/values/staging.yaml

      - name: Render templates (production)
        run: helm template test ./webapp-chart -f webapp-chart/values/production.yaml

  deploy-dev:
    name: Deploy to Dev
    needs: lint
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    environment:
      name: dev
    steps:
      - uses: actions/checkout@v4

      - name: Set up Helm
        uses: azure/setup-helm@v3

      - name: Configure kubeconfig
        uses: azure/k8s-set-context@v3
        with:
          kubeconfig: ${{ secrets.KUBE_CONFIG_DEV }}

      - name: Deploy to dev
        run: ./deploy.sh dev upgrade

  deploy-staging:
    name: Deploy to Staging
    needs: deploy-dev
    runs-on: ubuntu-latest
    environment:
      name: staging     # requires manual approval in GitHub
    steps:
      - uses: actions/checkout@v4

      - name: Set up Helm
        uses: azure/setup-helm@v3

      - name: Configure kubeconfig
        uses: azure/k8s-set-context@v3
        with:
          kubeconfig: ${{ secrets.KUBE_CONFIG_STAGING }}

      - name: Deploy to staging
        run: ./deploy.sh staging upgrade

  deploy-production:
    name: Deploy to Production
    needs: deploy-staging
    runs-on: ubuntu-latest
    environment:
      name: production  # requires manual approval in GitHub
    steps:
      - uses: actions/checkout@v4

      - name: Set up Helm
        uses: azure/setup-helm@v3

      - name: Configure kubeconfig
        uses: azure/k8s-set-context@v3
        with:
          kubeconfig: ${{ secrets.KUBE_CONFIG_PROD }}

      - name: Deploy to production
        run: ./deploy.sh production upgrade
```

## Manual testing walkthrough

```bash
# Install to dev
./deploy.sh dev install 0.1.0

# Check status and history
./deploy.sh dev status

# Make a change and upgrade
# (edit values.yaml or chart templates)
./deploy.sh dev upgrade

# View release history -- shows revision 1 (install) and 2 (upgrade)
helm history webapp-dev -n dev

# Simulate a bad deployment and rollback
./deploy.sh dev rollback 1

# Clean up
./deploy.sh dev uninstall
```

## Key concepts demonstrated

**Release management:** Each `helm install` creates a release with a
revision number. `helm upgrade` bumps the revision. `helm history`
shows the full timeline.

**Atomic operations:** The `--wait` flag makes Helm wait until all
resources are ready before marking the release as successful. If any
resource fails to become ready within `--timeout`, the upgrade is
marked as failed.

**Automatic rollback:** The `trap ... ERR` in `deploy.sh` catches any
non-zero exit code (lint failure, rollout failure, smoke test failure)
and triggers `helm rollback ... 0`. The `0` argument means "roll back
to the previous revision."

**Environment gates:** The GitHub Actions `environment` field with
"Required reviewers" configured in repository settings creates manual
approval gates between staging and production deployments.
