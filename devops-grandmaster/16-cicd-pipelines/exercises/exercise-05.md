# Exercise 05: Complete CI/CD Strategy for Microservices

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Design a complete CI/CD strategy for a microservices architecture with five services, each with its own repository, its own pipeline, and its own deployment lifecycle, plus a shared integration pipeline that validates the entire system.

## Scenario

Your company operates a platform with five microservices:

| Service | Language | Port | Database | Description |
|---------|----------|------|----------|-------------|
| `api-gateway` | Node.js | 8080 | Redis | Routes requests to backend services |
| `user-service` | Python | 8001 | PostgreSQL | Authentication and user management |
| `order-service` | Go | 8002 | PostgreSQL | Order processing and history |
| `inventory-service` | Rust | 8003 | PostgreSQL | Stock management |
| `notification-service` | Node.js | 8004 | RabbitMQ | Email and push notifications |

Each service has its own Git repository. The platform is deployed to Kubernetes. Services communicate via HTTP and message queues.

You must design the CI/CD strategy that covers all five services and their integration.

## Tasks

### Part A: Per-Service Pipeline Template

Design a reusable GitHub Actions workflow template that each service can adopt. The template must include:

1. Build stage (language-specific).
2. Unit test stage.
3. Linting and formatting check.
4. Docker image build and push to ghcr.io.
5. Vulnerability scan with Trivy.
6. Deploy to the service's own namespace in Kubernetes.

The template must be parameterized so that each service provides its own values for: language, build command, test command, Dockerfile path, and namespace.

Write the template as a reusable workflow (`.github/workflows/service-pipeline-template.yml`) and show how one service would call it.

<details>
<summary>Hint</summary>

Use GitHub Actions' `workflow_call` trigger with `inputs:` for parameterization. Each service repository has its own workflow that calls the shared template with its specific values. Use `docker/build-push-action` with parameterized Dockerfile path.

</details>

### Part B: Matrix Builds for Multiple Services

When a change affects the shared API contract (protobuf definitions, OpenAPI spec), all services may need to be rebuilt and tested together. Design a matrix-based workflow that:

1. Defines a build matrix for all five services.
2. Builds and tests each service in parallel.
3. Collects test results from all services into a single report.
4. Fails the overall pipeline if any single service fails.

Write the matrix configuration and the result aggregation step.

<details>
<summary>Hint</summary>

Use `strategy: matrix:` with service names as the matrix dimension. Each matrix job builds and tests one service. Use `actions/upload-artifact` to collect results. A final `needs: [build]` job downloads and aggregates all artifacts.

</details>

### Part C: Integration Testing Pipeline

Design a pipeline stage that spins up all five services together and runs end-to-end tests:

1. Deploy all five services to a temporary namespace (or use Docker Compose).
2. Wait for all services to be healthy.
3. Run a sequence of integration test scenarios:
   - Create a user account.
   - Place an order.
   - Verify inventory is decremented.
   - Verify notification is sent.
4. Tear down the temporary environment.
5. Report integration test results.

Write the workflow steps for this integration test stage.

<details>
<summary>Hint</summary>

Use `kubectl create namespace` for a temporary namespace, deploy all services with `kubectl apply`, wait with `kubectl rollout status`, run tests with `curl` or a test framework, then delete the namespace with `kubectl delete namespace`. Alternatively, use `docker compose` for local integration testing.

</details>

### Part D: Coordinated Deployment Strategy

When multiple services need to be deployed together (a coordinated release), the deployment order matters. Design the deployment strategy:

1. Define the deployment order: `user-service` -> `inventory-service` -> `order-service` -> `notification-service` -> `api-gateway`.
2. Each service deploys only after the previous one is healthy.
3. If any service fails to deploy, halt the pipeline and roll back the already-deployed services.
4. The `api-gateway` deploys last because it routes traffic to the others.

Write the workflow that orchestrates this coordinated deployment. Include the rollback logic for partially deployed releases.

<details>
<summary>Hint</summary>

Use `needs:` chains to enforce deployment order. Each deployment job checks `if: success()` on its predecessor. For rollback, use a `failure()` condition that triggers `kubectl rollout undo` for each already-deployed service.

</details>

### Part E: Monitoring and Observability Integration

After deployment, the pipeline must verify the health of the entire system:

1. Check that all five services are responding on their health endpoints.
2. Verify that inter-service communication works (api-gateway can reach each backend).
3. Check that the message queue connection (RabbitMQ) is healthy.
4. Check that all database connections are healthy.
5. If any check fails, trigger the rollback from Part D.

Write a post-deployment verification script that the pipeline runs after all services are deployed.

<details>
<summary>Hint</summary>

Write a bash script that curls each service's `/health` endpoint and checks the response. For inter-service communication, make a request through the api-gateway that exercises each backend. Parse health check JSON responses for database and queue status.

</details>

## Success Criteria

- [ ] The reusable workflow template can be called by any service with service-specific parameters.
- [ ] The matrix build runs all five services in parallel and reports aggregated results.
- [ ] The integration test stage spins up all services, runs end-to-end scenarios, and tears down.
- [ ] The coordinated deployment enforces the correct service order.
- [ ] Failed deployments halt the pipeline and roll back partially deployed services.
- [ ] Post-deployment verification checks health endpoints, inter-service communication, and infrastructure connections.
- [ ] The complete design is implementable as GitHub Actions workflows with valid YAML.

## What You Should Understand After This Exercise

CI/CD for microservices is fundamentally different from CI/CD for a monolith. Each service has its own pipeline, but services also need integration testing and coordinated deployment. Reusable workflow templates reduce duplication. Matrix builds provide parallelism. Deployment ordering prevents traffic from reaching services whose dependencies are not yet ready. Post-deployment verification catches integration issues that unit tests miss. The entire strategy must account for partial failures and include rollback mechanisms for coordinated releases.
