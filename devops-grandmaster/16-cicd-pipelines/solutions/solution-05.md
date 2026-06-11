# Solution 05: Complete CI/CD Strategy for Microservices

## Part A: Per-Service Pipeline Template

### Reusable Workflow Template

`.github/workflows/service-pipeline-template.yml`:

```yaml
name: Reusable Service Pipeline

on:
  workflow_call:
    inputs:
      service-name:
        required: true
        type: string
        description: Name of the service
      language:
        required: true
        type: string
        description: Programming language (node, python, go, rust)
      build-command:
        required: true
        type: string
        description: Command to build the service
      test-command:
        required: true
        type: string
        description: Command to run tests
      dockerfile-path:
        required: false
        type: string
        default: Dockerfile
        description: Path to the Dockerfile
      namespace:
        required: true
        type: string
        description: Kubernetes namespace for deployment
      port:
        required: true
        type: number
        description: Service port number
    secrets:
      KUBE_CONFIG:
        required: true
      REGISTRY_TOKEN:
        required: false

jobs:
  build-test-deploy:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write
      id-token: write
      security-events: write
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Set up language runtime
        uses: actions/setup-node@v4
        if: inputs.language == 'node'
        with:
          node-version: '20'
          cache: 'npm'

      - name: Set up Python
        uses: actions/setup-python@v5
        if: inputs.language == 'python'
        with:
          python-version: '3.12'
          cache: 'pip'

      - name: Set up Go
        uses: actions/setup-go@v5
        if: inputs.language == 'go'
        with:
          go-version: '1.22'
          cache: true

      - name: Set up Rust
        uses: dtolnay/rust-toolchain@stable
        if: inputs.language == 'rust'

      - name: Build service
        run: ${{ inputs.build-command }}

      - name: Run tests
        run: ${{ inputs.test-command }}

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Log in to GitHub Container Registry
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Build and push Docker image
        id: build-push
        uses: docker/build-push-action@v5
        with:
          context: .
          file: ${{ inputs.dockerfile-path }}
          push: true
          tags: |
            ghcr.io/${{ github.repository }}/${{ inputs.service-name }}:${{ github.sha }}
            ghcr.io/${{ github.repository }}/${{ inputs.service-name }}:latest
          cache-from: type=gha
          cache-to: type=gha,mode=max

      - name: Run Trivy vulnerability scan
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: ghcr.io/${{ github.repository }}/${{ inputs.service-name }}:${{ github.sha }}
          format: table
          exit-code: 1
          severity: CRITICAL,HIGH
          ignore-unfixed: true

      - name: Upload SARIF results
        uses: github/codeql-action/upload-sarif@v3
        if: always()
        with:
          sarif_file: trivy-results.sarif

      - name: Deploy to Kubernetes
        if: github.ref == 'refs/heads/main'
        run: |
          echo "${{ secrets.KUBE_CONFIG }}" | base64 -d > $HOME/.kube/config
          kubectl set image deployment/${{ inputs.service-name }} \
            ${{ inputs.service-name }}=ghcr.io/${{ github.repository }}/${{ inputs.service-name }}:${{ github.sha }} \
            -n ${{ inputs.namespace }}
          kubectl rollout status deployment/${{ inputs.service-name }} \
            -n ${{ inputs.namespace }} \
            --timeout=180s
```

### How a Service Calls the Template

`.github/workflows/ci.yml` in the `order-service` repository:

```yaml
name: Order Service CI/CD

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  pipeline:
    uses: ./.github/workflows/service-pipeline-template.yml
    with:
      service-name: order-service
      language: go
      build-command: go build -o bin/order-service ./...
      test-command: go test -v -race ./...
      dockerfile-path: Dockerfile
      namespace: orders
      port: 8002
    secrets:
      KUBE_CONFIG: ${{ secrets.KUBE_CONFIG }}
```

### Why This Works

The `workflow_call` trigger makes the workflow reusable. Each service repository has a thin workflow that calls the shared template with service-specific parameters. This eliminates duplication while allowing each service to customize build commands, test commands, and deployment targets. The template handles the entire lifecycle: build, test, Docker image, security scan, and deploy.

### Common Mistakes

- **Duplicating the pipeline in each repository.** When you need to change the scan threshold or add a new stage, you must update every repository. A shared template changes once.
- **Not parameterizing the Dockerfile path.** Some services use `Dockerfile`, others use `docker/Dockerfile.service`. The template must accept this as input.
- **Hardcoding the registry URL.** Use `${{ github.repository }}` to construct the registry path dynamically.

## Part B: Matrix Builds for Multiple Services

`.github/workflows/integration-matrix.yml`:

```yaml
name: Integration Matrix Build

on:
  push:
    branches: [main]
    paths:
      - 'contracts/**'
      - 'proto/**'
      - 'openapi/**'
  workflow_dispatch:

jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        service:
          - name: api-gateway
            language: node
            build-command: npm ci
            test-command: npm test
            port: 8080
          - name: user-service
            language: python
            build-command: pip install -r requirements.txt
            test-command: pytest
            port: 8001
          - name: order-service
            language: go
            build-command: go build ./...
            test-command: go test ./...
            port: 8002
          - name: inventory-service
            language: rust
            build-command: cargo build
            test-command: cargo test
            port: 8003
          - name: notification-service
            language: node
            build-command: npm ci
            test-command: npm test
            port: 8004
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Build and test ${{ matrix.service.name }}
        run: |
          cd services/${{ matrix.service.name }}
          ${{ matrix.service.build-command }}
          ${{ matrix.service.test-command }}

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: test-results-${{ matrix.service.name }}
          path: services/${{ matrix.service.name }}/test-results/
          retention-days: 7

  aggregate-results:
    needs: build
    if: always()
    runs-on: ubuntu-latest
    steps:
      - name: Download all test results
        uses: actions/download-artifact@v4
        with:
          path: all-results/

      - name: Generate summary report
        run: |
          echo "## Integration Matrix Test Results" >> $GITHUB_STEP_SUMMARY
          echo "" >> $GITHUB_STEP_SUMMARY
          echo "| Service | Status |" >> $GITHUB_STEP_SUMMARY
          echo "|---------|--------|" >> $GITHUB_STEP_SUMMARY

          FAILED=0
          for dir in all-results/test-results-*/; do
            SERVICE=$(basename "$dir" | sed 's/test-results-//')
            if ls "$dir"/*.xml > /dev/null 2>&1; then
              echo "| $SERVICE | PASS |" >> $GITHUB_STEP_SUMMARY
            else
              echo "| $SERVICE | FAIL |" >> $GITHUB_STEP_SUMMARY
              FAILED=1
            fi
          done

          if [ "$FAILED" -eq 1 ]; then
            echo ""
            echo "One or more services failed. See individual job logs."
            exit 1
          fi

      - name: Check build job status
        run: |
          if [ "${{ needs.build.result }}" = "failure" ]; then
            echo "One or more matrix jobs failed"
            exit 1
          fi
```

### Why This Works

The `strategy: matrix:` with `fail-fast: false` runs all five service builds in parallel and does not cancel the others when one fails. This is important for a matrix build: you want to see which services fail, not just the first one. The `aggregate-results` job downloads all artifacts and generates a unified report. The `if: always()` on the aggregate job ensures it runs even when some matrix jobs fail. The `paths:` trigger limits matrix builds to changes in shared contracts, preventing unnecessary full rebuilds.

### Common Mistakes

- **Using `fail-fast: true` (the default).** This cancels all other matrix jobs when one fails. For integration testing, you want to see all failures.
- **Not using `if: always()` on the aggregate job.** Without this, the aggregate job is skipped when any matrix job fails, and you lose the summary report.
- **Triggering the matrix on every push.** The matrix should only run when shared contracts change. Use `paths:` filters to limit triggers.

## Part C: Integration Testing Pipeline

```yaml
name: Integration Tests

on:
  workflow_run:
    workflows: ["Integration Matrix Build"]
    types: [completed]
    branches: [main]

jobs:
  integration-test:
    if: ${{ github.event.workflow_run.conclusion == 'success' }}
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Create temporary namespace
        run: |
          kubectl create namespace integration-test-${{ github.run_id }} || true

      - name: Deploy all services
        run: |
          NAMESPACE="integration-test-${{ github.run_id }}"
          SERVICES="api-gateway user-service order-service inventory-service notification-service"

          for SERVICE in $SERVICES; do
            kubectl apply -f k8s/$SERVICE/ -n $NAMESPACE
          done

      - name: Wait for all services to be ready
        run: |
          NAMESPACE="integration-test-${{ github.run_id }}"
          SERVICES="api-gateway user-service order-service inventory-service notification-service"

          for SERVICE in $SERVICES; do
            echo "Waiting for $SERVICE..."
            kubectl rollout status deployment/$SERVICE \
              -n $NAMESPACE \
              --timeout=300s
          done

      - name: Run end-to-end tests
        id: e2e
        run: |
          API_URL="http://$(kubectl get svc api-gateway -n integration-test-${{ github.run_id }} -o jsonpath='{.status.loadBalancer.ingress[0].ip}'):8080"

          echo "Running end-to-end tests against $API_URL"

          # Test 1: Create user account
          echo "--- Test 1: Create user ---"
          USER_RESPONSE=$(curl -sf -X POST "$API_URL/api/v1/users" \
            -H "Content-Type: application/json" \
            -d '{"email": "test@example.com", "name": "Test User", "password": "securepass123"}')
          USER_ID=$(echo "$USER_RESPONSE" | jq -r '.id')
          echo "Created user: $USER_ID"

          # Test 2: Place order
          echo "--- Test 2: Place order ---"
          ORDER_RESPONSE=$(curl -sf -X POST "$API_URL/api/v1/orders" \
            -H "Content-Type: application/json" \
            -d "{\"user_id\": \"$USER_ID\", \"items\": [{\"product_id\": \"prod-001\", \"quantity\": 2}]}")
          ORDER_ID=$(echo "$ORDER_RESPONSE" | jq -r '.id')
          echo "Created order: $ORDER_ID"

          # Test 3: Verify inventory decremented
          echo "--- Test 3: Verify inventory ---"
          INVENTORY=$(curl -sf "$API_URL/api/v1/inventory/prod-001")
          STOCK=$(echo "$INVENTORY" | jq -r '.quantity')
          if [ "$STOCK" -lt 100 ]; then
            echo "Inventory decremented: $STOCK remaining"
          else
            echo "FAIL: Inventory not decremented"
            exit 1
          fi

          # Test 4: Verify notification sent
          echo "--- Test 4: Verify notification ---"
          sleep 5
          NOTIFICATIONS=$(curl -sf "$API_URL/api/v1/notifications?user_id=$USER_ID")
          NOTIF_COUNT=$(echo "$NOTIFICATIONS" | jq '. | length')
          if [ "$NOTIF_COUNT" -gt 0 ]; then
            echo "Notification sent: $NOTIF_COUNT notifications"
          else
            echo "FAIL: No notifications found"
            exit 1
          fi

          echo "All end-to-end tests passed"

      - name: Tear down test environment
        if: always()
        run: |
          kubectl delete namespace integration-test-${{ github.run_id }} --grace-period=0 --force || true

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: integration-test-results
          path: test-results/
```

### Why This Works

A temporary Kubernetes namespace isolates the integration test environment from other environments. The `workflow_run` trigger ensures integration tests only run after the matrix build succeeds. The end-to-end tests exercise the full request flow: user creation through the api-gateway reaches the user-service, order creation reaches the order-service and inventory-service, and the notification-service sends a confirmation. The `if: always()` on the teardown step ensures the namespace is cleaned up even when tests fail.

### Common Mistakes

- **Not using a temporary namespace.** Shared namespaces cause test pollution and race conditions when multiple runs execute concurrently.
- **Not tearing down on failure.** Without `if: always()`, failed tests leave the namespace running, consuming cluster resources.
- **Hardcoding service URLs.** Use `kubectl get svc` to discover service IPs dynamically, since they change with each deployment.

## Part D: Coordinated Deployment Strategy

```yaml
name: Coordinated Deployment

on:
  workflow_dispatch:
    inputs:
      image-tag:
        required: true
        description: The image tag to deploy across all services

jobs:
  deploy-user-service:
    runs-on: ubuntu-latest
    environment:
      name: production-user-service
    steps:
      - name: Deploy user-service
        id: deploy
        run: |
          echo "${{ secrets.KUBE_CONFIG }}" | base64 -d > $HOME/.kube/config
          kubectl set image deployment/user-service \
            user-service=ghcr.io/org/user-service:${{ inputs.image-tag }} \
            -n production
          kubectl rollout status deployment/user-service -n production --timeout=300s

      - name: Health check
        id: health
        run: |
          for i in $(seq 1 20); do
            STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://user-service.production.svc.cluster.local:8001/health || echo "000")
            if [ "$STATUS" = "200" ]; then
              exit 0
            fi
            sleep 5
          done
          exit 1

      - name: Rollback on failure
        if: failure() && steps.deploy.outcome == 'success'
        run: |
          kubectl rollout undo deployment/user-service -n production
          kubectl rollout status deployment/user-service -n production --timeout=180s

  deploy-inventory-service:
    needs: [deploy-user-service]
    if: ${{ needs.deploy-user-service.result == 'success' }}
    runs-on: ubuntu-latest
    environment:
      name: production-inventory-service
    steps:
      - name: Deploy inventory-service
        id: deploy
        run: |
          echo "${{ secrets.KUBE_CONFIG }}" | base64 -d > $HOME/.kube/config
          kubectl set image deployment/inventory-service \
            inventory-service=ghcr.io/org/inventory-service:${{ inputs.image-tag }} \
            -n production
          kubectl rollout status deployment/inventory-service -n production --timeout=300s

      - name: Health check
        id: health
        run: |
          for i in $(seq 1 20); do
            STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://inventory-service.production.svc.cluster.local:8003/health || echo "000")
            if [ "$STATUS" = "200" ]; then
              exit 0
            fi
            sleep 5
          done
          exit 1

      - name: Rollback on failure
        if: failure() && steps.deploy.outcome == 'success'
        run: |
          kubectl rollout undo deployment/inventory-service -n production
          kubectl rollout status deployment/inventory-service -n production --timeout=180s

  rollback-user-on-inventory-failure:
    needs: [deploy-user-service, deploy-inventory-service]
    if: ${{ needs.deploy-inventory-service.result == 'failure' && needs.deploy-user-service.result == 'success' }}
    runs-on: ubuntu-latest
    steps:
      - name: Rollback user-service
        run: |
          echo "${{ secrets.KUBE_CONFIG }}" | base64 -d > $HOME/.kube/config
          kubectl rollout undo deployment/user-service -n production
          kubectl rollout status deployment/user-service -n production --timeout=180s

  deploy-order-service:
    needs: [deploy-inventory-service]
    if: ${{ needs.deploy-inventory-service.result == 'success' }}
    runs-on: ubuntu-latest
    environment:
      name: production-order-service
    steps:
      - name: Deploy order-service
        id: deploy
        run: |
          echo "${{ secrets.KUBE_CONFIG }}" | base64 -d > $HOME/.kube/config
          kubectl set image deployment/order-service \
            order-service=ghcr.io/org/order-service:${{ inputs.image-tag }} \
            -n production
          kubectl rollout status deployment/order-service -n production --timeout=300s

      - name: Health check
        id: health
        run: |
          for i in $(seq 1 20); do
            STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://order-service.production.svc.cluster.local:8002/health || echo "000")
            if [ "$STATUS" = "200" ]; then
              exit 0
            fi
            sleep 5
          done
          exit 1

      - name: Rollback on failure
        if: failure() && steps.deploy.outcome == 'success'
        run: |
          kubectl rollout undo deployment/order-service -n production
          kubectl rollout status deployment/order-service -n production --timeout=180s

  rollback-inventory-on-order-failure:
    needs: [deploy-inventory-service, deploy-order-service]
    if: ${{ needs.deploy-order-service.result == 'failure' && needs.deploy-inventory-service.result == 'success' }}
    runs-on: ubuntu-latest
    steps:
      - name: Rollback inventory-service
        run: |
          echo "${{ secrets.KUBE_CONFIG }}" | base64 -d > $HOME/.kube/config
          kubectl rollout undo deployment/inventory-service -n production
          kubectl rollout status deployment/inventory-service -n production --timeout=180s

  deploy-notification-service:
    needs: [deploy-order-service]
    if: ${{ needs.deploy-order-service.result == 'success' }}
    runs-on: ubuntu-latest
    environment:
      name: production-notification-service
    steps:
      - name: Deploy notification-service
        id: deploy
        run: |
          echo "${{ secrets.KUBE_CONFIG }}" | base64 -d > $HOME/.kube/config
          kubectl set image deployment/notification-service \
            notification-service=ghcr.io/org/notification-service:${{ inputs.image-tag }} \
            -n production
          kubectl rollout status deployment/notification-service -n production --timeout=300s

      - name: Health check
        id: health
        run: |
          for i in $(seq 1 20); do
            STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://notification-service.production.svc.cluster.local:8004/health || echo "000")
            if [ "$STATUS" = "200" ]; then
              exit 0
            fi
            sleep 5
          done
          exit 1

      - name: Rollback on failure
        if: failure() && steps.deploy.outcome == 'success'
        run: |
          kubectl rollout undo deployment/notification-service -n production
          kubectl rollout status deployment/notification-service -n production --timeout=180s

  deploy-api-gateway:
    needs: [deploy-notification-service]
    if: ${{ needs.deploy-notification-service.result == 'success' }}
    runs-on: ubuntu-latest
    environment:
      name: production-api-gateway
    steps:
      - name: Deploy api-gateway
        id: deploy
        run: |
          echo "${{ secrets.KUBE_CONFIG }}" | base64 -d > $HOME/.kube/config
          kubectl set image deployment/api-gateway \
            api-gateway=ghcr.io/org/api-gateway:${{ inputs.image-tag }} \
            -n production
          kubectl rollout status deployment/api-gateway -n production --timeout=300s

      - name: Health check
        id: health
        run: |
          for i in $(seq 1 20); do
            STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://api-gateway.production.svc.cluster.local:8080/health || echo "000")
            if [ "$STATUS" = "200" ]; then
              exit 0
            fi
            sleep 5
          done
          exit 1

      - name: Rollback on failure
        if: failure() && steps.deploy.outcome == 'success'
        run: |
          kubectl rollout undo deployment/api-gateway -n production
          kubectl rollout status deployment/api-gateway -n production --timeout=180s

      - name: Rollback all on gateway failure
        if: failure()
        run: |
          echo "${{ secrets.KUBE_CONFIG }}" | base64 -d > $HOME/.kube/config
          for SVC in notification-service order-service inventory-service user-service; do
            kubectl rollout undo deployment/$SVC -n production || true
          done

      - name: Notify deployment status
        if: always()
        run: |
          STATUS="${{ job.status }}"
          curl -X POST "${{ secrets.SLACK_WEBHOOK }}" \
            -H "Content-Type: application/json" \
            -d "{
              \"text\": \"Coordinated deployment $STATUS. Tag: ${{ inputs.image-tag }}. Workflow: ${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}\"
            }"
```

### Why This Works

The deployment order follows the dependency graph: user-service first (no dependencies), then inventory-service (depends on users), then order-service (depends on users and inventory), then notification-service (depends on orders), and finally api-gateway (depends on all backends). Each deployment job uses `needs:` and `if:` to enforce ordering and halt on failure. The rollback jobs use `needs:` to track which services were successfully deployed and `if:` to trigger rollback only when a downstream service fails. The api-gateway deploys last because it routes traffic to all backends -- deploying it before backends are ready would cause request failures.

### Common Mistakes

- **Deploying the gateway first.** The api-gateway routes traffic to all backends. If it deploys before backends are ready, requests fail. Always deploy backends first.
- **Not rolling back upstream services.** If order-service fails after user-service was already deployed, the user-service should be rolled back to maintain a consistent state across all services.
- **Using `workflow_dispatch` without input validation.** Always require the image tag as input to prevent accidental deployments of undefined versions.

## Part E: Monitoring and Observability Integration

```yaml
  verify-system-health:
    needs: [deploy-api-gateway]
    if: ${{ needs.deploy-api-gateway.result == 'success' }}
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Configure kubectl
        run: echo "${{ secrets.KUBE_CONFIG }}" | base64 -d > $HOME/.kube/config

      - name: Verify all service health endpoints
        run: |
          SERVICES="api-gateway:8080 user-service:8001 order-service:8002 inventory-service:8003 notification-service:8004"
          FAILED=0

          for ENTRY in $SERVICES; do
            SVC=$(echo "$ENTRY" | cut -d: -f1)
            PORT=$(echo "$ENTRY" | cut -d: -f2)
            echo "Checking $SVC on port $PORT..."

            STATUS=$(curl -s -o /dev/null -w "%{http_code}" \
              http://$SVC.production.svc.cluster.local:$PORT/health || echo "000")

            if [ "$STATUS" = "200" ]; then
              echo "  $SVC: HEALTHY"
            else
              echo "  $SVC: UNHEALTHY (status: $STATUS)"
              FAILED=1
            fi
          done

          exit $FAILED

      - name: Verify inter-service communication
        run: |
          API_URL="http://api-gateway.production.svc.cluster.local:8080"

          echo "Testing api-gateway -> user-service..."
          curl -sf "$API_URL/api/v1/users/health" || exit 1

          echo "Testing api-gateway -> order-service..."
          curl -sf "$API_URL/api/v1/orders/health" || exit 1

          echo "Testing api-gateway -> inventory-service..."
          curl -sf "$API_URL/api/v1/inventory/health" || exit 1

          echo "Testing api-gateway -> notification-service..."
          curl -sf "$API_URL/api/v1/notifications/health" || exit 1

          echo "All inter-service communication verified"

      - name: Verify database connectivity
        run: |
          SERVICES="user-service:8001 order-service:8002 inventory-service:8003"

          for ENTRY in $SERVICES; do
            SVC=$(echo "$ENTRY" | cut -d: -f1)
            PORT=$(echo "$ENTRY" | cut -d: -f2)

            DB_STATUS=$(curl -sf "http://$SVC.production.svc.cluster.local:$PORT/health/db" | jq -r '.database')
            if [ "$DB_STATUS" = "connected" ]; then
              echo "  $SVC database: CONNECTED"
            else
              echo "  $SVC database: DISCONNECTED"
              exit 1
            fi
          done

      - name: Verify message queue connectivity
        run: |
          STATUS=$(curl -sf "http://notification-service.production.svc.cluster.local:8004/health/queue" | jq -r '.rabbitmq')
          if [ "$STATUS" = "connected" ]; then
            echo "RabbitMQ: CONNECTED"
          else
            echo "RabbitMQ: DISCONNECTED"
            exit 1
          fi

      - name: Rollback all services on verification failure
        if: failure()
        run: |
          echo "System verification failed. Rolling back all services..."
          for SVC in api-gateway notification-service order-service inventory-service user-service; do
            kubectl rollout undo deployment/$SVC -n production || true
          done
          for SVC in api-gateway notification-service order-service inventory-service user-service; do
            kubectl rollout status deployment/$SVC -n production --timeout=180s || true
          done

      - name: Notify verification status
        if: always()
        run: |
          STATUS="${{ job.status }}"
          curl -X POST "${{ secrets.SLACK_WEBHOOK }}" \
            -H "Content-Type: application/json" \
            -d "{
              \"text\": \"System verification $STATUS. All services deployed with tag ${{ inputs.image-tag }}.\"
            }"
```

### Why This Works

The verification script checks four layers: individual service health, inter-service communication through the gateway, database connectivity, and message queue connectivity. Each layer catches a different class of integration failure. If any check fails, all services are rolled back in reverse deployment order. The rollback order is important: the gateway is rolled back first (to stop routing traffic), then backends in reverse dependency order.

### Common Mistakes

- **Only checking service health endpoints.** A service can return HTTP 200 on `/health` but be unable to reach its database or message queue. Always check dependency connectivity.
- **Not rolling back all services on verification failure.** If the system verification fails after all services are deployed, a partial rollback creates an inconsistent state. Roll back everything.
- **Checking health immediately after deployment.** Services need time to start up and register with service discovery. Use retry loops with delays.

## Key Takeaway

CI/CD for microservices requires three levels of automation: per-service pipelines that build, test, scan, and deploy each service independently; integration pipelines that validate cross-service interactions; and coordinated deployment pipelines that enforce dependency ordering with rollback on failure. Reusable workflow templates eliminate duplication. Matrix builds provide parallelism. Temporary namespaces isolate integration tests. The deployment order follows the dependency graph, and the api-gateway deploys last. Post-deployment verification checks health, communication, databases, and message queues. The entire strategy accounts for partial failures at every stage.
