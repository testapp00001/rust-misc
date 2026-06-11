# Exercise 05: Helm-Based Deployment Pipeline (Integration)

## Objective

Design and implement a CI/CD pipeline that uses Helm to deploy an
application across dev, staging, and production environments. This
exercise integrates everything from Exercises 01-04 into a real
deployment workflow.

## Scenario

Your team maintains a web application with three environments. The
pipeline must:

1. Lint and validate the Helm chart on every pull request.
2. Deploy to `dev` automatically on merge to `main`.
3. Deploy to `staging` after manual approval.
4. Deploy to `production` after staging smoke tests pass.
5. Support automatic rollback on failed deployments.
6. Use Helm's release management (upgrades, history, rollback).

## Instructions

### Step 1 -- Prepare the chart

Use the `webapp-chart` from Exercise 02-03 (or create a fresh one). Your
chart directory should contain:

```
webapp-chart/
  Chart.yaml
  values.yaml
  values/
    dev.yaml
    staging.yaml
    production.yaml
  templates/
    _helpers.tpl
    deployment.yaml
    service.yaml
    configmap.yaml
    ingress.yaml
    hpa.yaml
    NOTES.txt
```

### Step 2 -- Write the pipeline script

Create a shell script `deploy.sh` that accepts these arguments:

```bash
./deploy.sh <environment> <action> [version]
```

Where:
- `<environment>` is one of `dev`, `staging`, `production`.
- `<action>` is one of `install`, `upgrade`, `rollback`, `status`, `uninstall`.
- `[version]` is the chart version (optional for `upgrade`; required for
  `install`).

The script must:

1. Validate inputs.
2. Select the correct values file (`values/<environment>.yaml`).
3. Set the release name to `webapp-<environment>`.
4. Set the namespace to `<environment>`.
5. Execute the appropriate Helm command.

**Commands the script should implement:**

```bash
# Install
helm install webapp-<env> ./webapp-chart \
  -f webapp-chart/values/<env>.yaml \
  --namespace <env> --create-namespace \
  --version <version> --wait --timeout 5m

# Upgrade
helm upgrade webapp-<env> ./webapp-chart \
  -f webapp-chart/values/<env>.yaml \
  --namespace <env> \
  --wait --timeout 5m

# Rollback
helm rollback webapp-<env> [revision] --namespace <env> --wait

# Status
helm status webapp-<env> --namespace <env>
helm history webapp-<env> --namespace <env>

# Uninstall
helm uninstall webapp-<env> --namespace <env>
```

### Step 3 -- Add pre-deployment checks

Before any install or upgrade, the script should:

1. Run `helm lint ./webapp-chart`.
2. Run `helm template webapp-<env> ./webapp-chart -f values/<env>.yaml`
   and pipe through `kubectl apply --dry-run=client -f -` to validate
   against the cluster schema (if a cluster is available).
3. Check that the namespace exists (create it if it does not).

### Step 4 -- Add post-deployment verification

After install or upgrade, the script should:

1. Wait for the Deployment rollout to complete:

   ```bash
   kubectl rollout status deployment/webapp-<env>-webapp \
     -n <env> --timeout=300s
   ```

2. Run a simple smoke test -- curl the service endpoint (or port-forward
   and curl localhost):

   ```bash
   kubectl port-forward svc/webapp-<env>-webapp 8080:80 -n <env> &
   sleep 2
   curl -sf http://localhost:8080/ || { echo "Smoke test failed"; exit 1; }
   kill %1
   ```

3. If the smoke test fails, automatically roll back:

   ```bash
   helm rollback webapp-<env> 0 --namespace <env>
   echo "Rolled back due to failed smoke test"
   exit 1
   ```

### Step 5 -- Add a GitHub Actions workflow (optional)

Create `.github/workflows/helm-deploy.yml` with the following jobs:

**Job 1: lint**
```yaml
- uses: actions/checkout@v4
- uses: azure/setup-helm@v3
- run: helm lint ./webapp-chart
- run: helm template test ./webapp-chart -f webapp-chart/values/dev.yaml
```

**Job 2: deploy-dev**
- Runs after `lint`.
- Executes `./deploy.sh dev upgrade`.
- Triggers only on push to `main`.

**Job 3: deploy-staging**
- Runs after `deploy-dev`.
- Requires manual approval (use GitHub environment protection rules).
- Executes `./deploy.sh staging upgrade`.

**Job 4: deploy-production**
- Runs after `deploy-staging`.
- Requires manual approval.
- Executes `./deploy.sh production upgrade`.

### Step 6 -- Test the workflow manually

```bash
# Install to dev
./deploy.sh dev install 0.1.0

# Upgrade dev
./deploy.sh dev upgrade

# Check status and history
./deploy.sh dev status

# Rollback dev to previous revision
./deploy.sh dev rollback

# Uninstall dev
./deploy.sh dev uninstall
```

## Success Criteria

- [ ] `deploy.sh` handles all five actions (install, upgrade, rollback,
      status, uninstall) correctly.
- [ ] The script selects the right values file for each environment.
- [ ] Pre-deployment linting catches template errors before applying.
- [ ] Post-deployment smoke tests verify the application is running.
- [ ] Failed smoke tests trigger automatic rollback.
- [ ] `helm history` shows the full release lifecycle across upgrades and
      rollbacks.
- [ ] (Optional) GitHub Actions workflow runs lint on PRs and deploys to
      dev/staging/production with appropriate gates.

## Hints

<details>
<summary>Hint: deploy.sh skeleton</summary>

```bash
#!/usr/bin/env bash
set -euo pipefail

ENV="${1:?Usage: deploy.sh <env> <action> [version]}"
ACTION="${2:?Usage: deploy.sh <env> <action> [version]}"
VERSION="${3:-}"
RELEASE="webapp-${ENV}"
NAMESPACE="${ENV}"
CHART="./webapp-chart"
VALUES="${CHART}/values/${ENV}.yaml"

if [[ ! -f "$VALUES" ]]; then
  echo "ERROR: values file not found: $VALUES"
  exit 1
fi

lint() {
  echo "==> Linting chart..."
  helm lint "$CHART"
}

pre_deploy() {
  lint
  kubectl get namespace "$NAMESPACE" >/dev/null 2>&1 || \
    kubectl create namespace "$NAMESPACE"
}

post_deploy() {
  echo "==> Waiting for rollout..."
  kubectl rollout status deployment/"${RELEASE}-webapp" \
    -n "$NAMESPACE" --timeout=300s
  echo "==> Smoke test..."
  # Add your smoke test logic here
}

case "$ACTION" in
  install)
    pre_deploy
    helm install "$RELEASE" "$CHART" \
      -f "$VALUES" \
      --namespace "$NAMESPACE" --create-namespace \
      ${VERSION:+--version "$VERSION"} \
      --wait --timeout 5m
    post_deploy
    ;;
  upgrade)
    pre_deploy
    helm upgrade "$RELEASE" "$CHART" \
      -f "$VALUES" \
      --namespace "$NAMESPACE" \
      --wait --timeout 5m
    post_deploy
    ;;
  rollback)
    helm rollback "$RELEASE" "${VERSION}" --namespace "$NAMESPACE" --wait
    ;;
  status)
    helm status "$RELEASE" --namespace "$NAMESPACE"
    helm history "$RELEASE" --namespace "$NAMESPACE"
    ;;
  uninstall)
    helm uninstall "$RELEASE" --namespace "$NAMESPACE"
    ;;
  *)
    echo "Unknown action: $ACTION"
    exit 1
    ;;
esac
```

</details>

<details>
<summary>Hint: Rollback on failure</summary>

Wrap post-deployment checks in a trap:

```bash
trap 'echo "Deploy failed, rolling back..."; \
      helm rollback "$RELEASE" 0 -n "$NAMESPACE"; \
      exit 1' ERR
```

The `rollback ... 0` syntax rolls back to the previous revision.

</details>

<details>
<summary>Hint: GitHub Actions environment protection</summary>

In the workflow file, reference a GitHub environment with required
reviewers:

```yaml
deploy-staging:
  needs: deploy-dev
  environment:
    name: staging
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: azure/setup-helm@v3
    - run: ./deploy.sh staging upgrade
```

Configure the `staging` environment in GitHub repo Settings > Environments
with "Required reviewers" enabled.

</details>
