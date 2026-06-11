# Solution 04: Multi-Environment Deployment Pipeline

## Part A: Define Environment-Specific Configuration

```yaml
name: Multi-Environment Deploy

on:
  push:
    branches: [main]

env:
  IMAGE_NAME: ghcr.io/${{ github.repository }}

jobs:
  build:
    runs-on: ubuntu-latest
    outputs:
      image-tag: ${{ steps.meta.outputs.version }}
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Log in to GitHub Container Registry
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Generate image metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.IMAGE_NAME }}
          tags: type=sha,prefix=

      - name: Build and push image
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          cache-from: type=gha
          cache-to: type=gha,mode=max

  deploy-dev:
    needs: build
    runs-on: ubuntu-latest
    environment:
      name: dev
      url: https://dev.example.com
    env:
      NAMESPACE: dev
      REPLICAS: 1
      CPU_LIMIT: 250m
      MEMORY_LIMIT: 256Mi
      DEBUG: "true"
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Configure kubectl
        uses: azure/setup-kubectl@v3

      - name: Set kubeconfig
        run: echo "${{ secrets.KUBE_CONFIG_DEV }}" | base64 -d > $HOME/.kube/config

      - name: Deploy to dev
        run: |
          kubectl set image deployment/myapp \
            myapp=${{ env.IMAGE_NAME }}:${{ needs.build.outputs.image-tag }} \
            -n ${{ env.NAMESPACE }}
          kubectl rollout status deployment/myapp -n ${{ env.NAMESPACE }} --timeout=120s

      - name: Verify dev deployment
        run: |
          for i in $(seq 1 10); do
            STATUS=$(curl -s -o /dev/null -w "%{http_code}" https://dev.example.com/health || echo "000")
            if [ "$STATUS" = "200" ]; then
              echo "Dev health check passed"
              exit 0
            fi
            echo "Attempt $i: status $STATUS, retrying..."
            sleep 10
          done
          echo "Dev health check failed"
          exit 1
```

### Why This Works

GitHub Actions environments (`environment: name: dev`) provide environment-specific secrets, variables, and deployment protection rules. Each environment job has its own `env:` block with environment-specific settings. The `outputs:` mechanism passes the image tag from the build job to the deploy jobs, ensuring the exact same image is deployed to each environment. The `url:` in the environment definition creates a clickable link in the GitHub deployment UI.

### Common Mistakes

- **Hardcoding environment values in the workflow.** Use GitHub Actions environments and their variables/secrets instead. This allows changing configuration without modifying the workflow file.
- **Not using `outputs:` for the image tag.** If each deploy job independently generates the tag, there is a risk of tag mismatch between environments.

## Part B: Build the Three-Stage Deployment Pipeline

```yaml
  deploy-staging:
    needs: [build, deploy-dev]
    runs-on: ubuntu-latest
    environment:
      name: staging
      url: https://staging.example.com
    env:
      NAMESPACE: staging
      REPLICAS: 2
      CPU_LIMIT: 500m
      MEMORY_LIMIT: 512Mi
      DEBUG: "false"
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Configure kubectl
        uses: azure/setup-kubectl@v3

      - name: Set kubeconfig
        run: echo "${{ secrets.KUBE_CONFIG_STAGING }}" | base64 -d > $HOME/.kube/config

      - name: Deploy to staging
        run: |
          kubectl set image deployment/myapp \
            myapp=${{ env.IMAGE_NAME }}:${{ needs.build.outputs.image-tag }} \
            -n ${{ env.NAMESPACE }}
          kubectl scale deployment/myapp --replicas=${{ env.REPLICAS }} -n ${{ env.NAMESPACE }}
          kubectl rollout status deployment/myapp -n ${{ env.NAMESPACE }} --timeout=180s

      - name: Run integration tests
        run: |
          # Wait for readiness
          for i in $(seq 1 15); do
            STATUS=$(curl -s -o /dev/null -w "%{http_code}" https://staging.example.com/health || echo "000")
            if [ "$STATUS" = "200" ]; then
              echo "Staging is ready"
              break
            fi
            echo "Waiting for staging... attempt $i"
            sleep 10
          done

          # Smoke tests
          echo "Running smoke tests..."
          curl -f https://staging.example.com/health
          curl -f https://staging.example.com/api/v1/status
          curl -f https://staging.example.com/api/v1/version

  deploy-production:
    needs: [build, deploy-staging]
    runs-on: ubuntu-latest
    environment:
      name: production
      url: https://production.example.com
    env:
      NAMESPACE: production
      REPLICAS: 3
      CPU_LIMIT: 1000m
      MEMORY_LIMIT: 1Gi
      DEBUG: "false"
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Configure kubectl
        uses: azure/setup-kubectl@v3

      - name: Set kubeconfig
        run: echo "${{ secrets.KUBE_CONFIG_PROD }}" | base64 -d > $HOME/.kube/config

      - name: Deploy to production
        id: deploy
        run: |
          kubectl set image deployment/myapp \
            myapp=${{ env.IMAGE_NAME }}:${{ needs.build.outputs.image-tag }} \
            -n ${{ env.NAMESPACE }}
          kubectl scale deployment/myapp --replicas=${{ env.REPLICAS }} -n ${{ env.NAMESPACE }}
          kubectl rollout status deployment/myapp -n ${{ env.NAMESPACE }} --timeout=300s

      - name: Production health check
        id: health
        run: |
          for i in $(seq 1 12); do
            STATUS=$(curl -s -o /dev/null -w "%{http_code}" https://production.example.com/health || echo "000")
            if [ "$STATUS" = "200" ]; then
              echo "Production health check passed"
              exit 0
            fi
            echo "Attempt $i: status $STATUS, retrying..."
            sleep 5
          done
          echo "Production health check failed"
          exit 1

      - name: Rollback on failure
        if: failure() && steps.deploy.outcome == 'success'
        run: |
          echo "Rolling back production deployment..."
          kubectl rollout undo deployment/myapp -n ${{ env.NAMESPACE }}
          kubectl rollout status deployment/myapp -n ${{ env.NAMESPACE }} --timeout=180s

      - name: Notify on failure
        if: failure()
        run: |
          curl -X POST "${{ secrets.SLACK_WEBHOOK }}" \
            -H "Content-Type: application/json" \
            -d '{
              "text": "Production deployment FAILED and was rolled back. Commit: ${{ github.sha }}. Workflow: ${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}"
            }'
```

### Why This Works

The `needs:` chain enforces deployment order: staging deploys only after dev succeeds, production deploys only after staging succeeds. The `environment: name: production` with "Required reviewers" configured in GitHub settings creates the manual approval gate -- the workflow pauses until an authorized person approves. The rollback step uses `if: failure() && steps.deploy.outcome == 'success'` to ensure rollback only happens when the deploy succeeded but the health check failed (not when the deploy itself failed).

### Common Mistakes

- **Not configuring required reviewers in GitHub.** The `environment: name: production` block alone does not create an approval gate. You must configure "Required reviewers" in the repository's environment settings.
- **Rolling back when the deploy step fails.** If `kubectl set image` fails, there is nothing to roll back. The `steps.deploy.outcome == 'success'` condition prevents unnecessary rollback attempts.
- **Not setting a rollout timeout.** Without `--timeout`, `kubectl rollout status` waits indefinitely. Always set a timeout to prevent hung pipelines.

## Part C: Add Integration Tests in Staging

```yaml
      - name: Wait for staging deployment
        run: |
          kubectl rollout status deployment/myapp \
            -n staging \
            --timeout=180s

      - name: Wait for pods to be ready
        run: |
          kubectl wait --for=condition=ready pod \
            -l app=myapp \
            -n staging \
            --timeout=120s

      - name: Run health check
        run: |
          for i in $(seq 1 20); do
            RESPONSE=$(curl -s -w "\n%{http_code}" https://staging.example.com/health)
            STATUS=$(echo "$RESPONSE" | tail -1)
            BODY=$(echo "$RESPONSE" | head -1)
            if [ "$STATUS" = "200" ]; then
              echo "Health check passed: $BODY"
              break
            fi
            echo "Attempt $i: status $STATUS"
            sleep 5
          done
          [ "$STATUS" = "200" ] || exit 1

      - name: Run smoke tests
        id: smoke
        run: |
          PASS=0
          FAIL=0

          # Test 1: API root responds
          if curl -sf https://staging.example.com/api/v1/status > /dev/null; then
            echo "PASS: API status endpoint"
            PASS=$((PASS + 1))
          else
            echo "FAIL: API status endpoint"
            FAIL=$((FAIL + 1))
          fi

          # Test 2: Version endpoint returns expected format
          VERSION=$(curl -sf https://staging.example.com/api/v1/version | jq -r '.version')
          if [ -n "$VERSION" ] && [ "$VERSION" != "null" ]; then
            echo "PASS: Version endpoint returned $VERSION"
            PASS=$((PASS + 1))
          else
            echo "FAIL: Version endpoint returned unexpected response"
            FAIL=$((FAIL + 1))
          fi

          # Test 3: Database connectivity
          if curl -sf https://staging.example.com/api/v1/health/db > /dev/null; then
            echo "PASS: Database health check"
            PASS=$((PASS + 1))
          else
            echo "FAIL: Database health check"
            FAIL=$((FAIL + 1))
          fi

          echo ""
          echo "Results: $PASS passed, $FAIL failed"
          if [ "$FAIL" -gt 0 ]; then
            exit 1
          fi
```

### Why This Works

The integration tests use a layered approach: first verify the deployment is rolled out, then verify pods are ready, then verify the application responds, then run functional smoke tests. Each layer catches a different class of problem: rollout catches scheduling issues, readiness catches startup failures, health checks catch runtime errors, and smoke tests catch functional regressions. The smoke test script tracks pass/fail counts and exits non-zero if any test fails, which blocks the production deployment.

### Common Mistakes

- **Not waiting for rollout before health checks.** The pods might be starting up. `kubectl rollout status` blocks until all replicas are ready.
- **Only checking HTTP 200.** A health check should verify the response body, not just the status code. An application might return 200 with `{"status": "degraded"}`.
- **Not testing inter-service connectivity.** A service can be healthy in isolation but unable to reach its dependencies. The database health check catches this.

## Part D: Implement Automatic Rollback in Production

The rollback and notification logic is included in Part B's `deploy-production` job above. Here is the key section explained:

```yaml
      - name: Deploy to production
        id: deploy
        run: |
          kubectl set image deployment/myapp \
            myapp=${{ env.IMAGE_NAME }}:${{ needs.build.outputs.image-tag }} \
            -n ${{ env.NAMESPACE }}
          kubectl rollout status deployment/myapp -n ${{ env.NAMESPACE }} --timeout=300s

      - name: Production health check
        id: health
        run: |
          for i in $(seq 1 12); do
            STATUS=$(curl -s -o /dev/null -w "%{http_code}" https://production.example.com/health || echo "000")
            if [ "$STATUS" = "200" ]; then
              echo "Production health check passed"
              exit 0
            fi
            echo "Attempt $i: status $STATUS, retrying..."
            sleep 5
          done
          echo "Production health check failed"
          exit 1

      - name: Rollback on failure
        if: failure() && steps.deploy.outcome == 'success'
        run: |
          echo "Rolling back production deployment..."
          kubectl rollout undo deployment/myapp -n ${{ env.NAMESPACE }}
          kubectl rollout status deployment/myapp -n ${{ env.NAMESPACE }} --timeout=180s

      - name: Notify on failure
        if: failure()
        run: |
          curl -X POST "${{ secrets.SLACK_WEBHOOK }}" \
            -H "Content-Type: application/json" \
            -d '{
              "text": "Production deployment FAILED. Commit: ${{ github.sha }}. Rolled back automatically."
            }'
```

### Why This Works

The `id: deploy` on the deploy step allows the rollback step to check whether the deploy succeeded. The `if: failure() && steps.deploy.outcome == 'success'` condition ensures rollback only happens when the deploy succeeded but a later step (health check) failed. If the deploy itself failed, there is nothing to roll back to. The `kubectl rollout undo` reverts to the previous ReplicaSet. The Slack notification includes the commit SHA and a link to the workflow run for debugging.

### Common Mistakes

- **Not using `steps.deploy.outcome`.** Without checking the deploy step's outcome, the rollback step might try to undo a deployment that never happened.
- **Rolling back without verifying.** After `kubectl rollout undo`, always run `kubectl rollout status` to confirm the rollback succeeded.
- **Not notifying the team.** Silent rollbacks mean the team does not know a deployment failed. Always send a notification with enough context to investigate.

## Key Takeaway

Multi-environment pipelines enforce promotion discipline through `needs:` chains and GitHub Actions environments. Each environment has its own configuration, secrets, and protection rules. Manual approval gates are configured in GitHub's environment settings, not in the workflow YAML. Health checks with retry loops catch deployment failures. Automatic rollback with `kubectl rollout undo` recovers from bad deployments. Notifications ensure the team is aware of failures. The pipeline enforces the rule that code cannot skip environments.
