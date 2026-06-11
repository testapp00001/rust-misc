# Exercise 05: Blue-Green with CI/CD Integration

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Integrate blue-green deployment into a complete CI/CD pipeline using GitHub
Actions and Kubernetes. This exercise combines pipeline design (Module 57)
with blue-green deployment to create a production-grade deployment system.

## Scenario

You are building the deployment pipeline for `payment-service`. The
requirements are:

- CI pipeline builds, tests, and pushes a Docker image
- CD pipeline deploys to the idle (green/blue) environment
- Health checks verify the new version before switching traffic
- Traffic switch is instant (Service selector update)
- Automatic rollback if production health checks fail
- Previous environment stays running for 30 minutes after switch

## Tasks

### Part A: Design the Deployment Flow

Draw an ASCII diagram showing the complete flow from image push to traffic
switch, including the rollback path. Label each step with the Kubernetes
resource involved.

<details>
<summary>Hint</summary>

The flow is:
1. Push image to registry
2. Update the idle Deployment with the new image
3. Wait for all pods in the idle Deployment to be ready
4. Run smoke tests against the idle pods directly (via pod port-forward
   or a test Service)
5. Patch the main Service to point to the idle environment
6. Run production health checks
7. If health checks fail, patch the Service back to the previous environment

</details>

### Part B: Write the Deployment Job

Write the GitHub Actions job that deploys the new image to the idle
environment. It should:
1. Determine which environment is currently active
2. Deploy to the idle environment
3. Wait for pods to be ready
4. Run smoke tests against the idle environment

<details>
<summary>Hint 1</summary>

Determine the active environment by checking the Service selector:

```bash
ACTIVE=$(kubectl get service payment-service -o jsonpath='{.spec.selector.version}')
if [ "$ACTIVE" = "blue" ]; then
    IDLE="green"
else
    IDLE="blue"
fi
```

</details>

<details>
<summary>Hint 2</summary>

To run smoke tests against the idle environment without switching traffic,
use a temporary headless Service or `kubectl port-forward`:

```bash
kubectl port-forward deployment/payment-$IDLE 8080:8080 &
sleep 2
curl -sf http://localhost:8080/health
```

</details>

### Part C: Write the Traffic Switch and Rollback

Write the traffic switch step and the automatic rollback step. The
rollback should trigger if health checks fail after the switch.

<details>
<summary>Hint</summary>

Use `if: failure()` for the rollback step. Capture the previous
environment before switching:

```bash
echo "previous=$ACTIVE" >> "$GITHUB_OUTPUT"

# Switch
kubectl patch service payment-service \
  -p '{"spec":{"selector":{"version":"'$IDLE'"}}}'

# Health check
curl -sf https://payment.example.com/health
```

On failure, switch back:

```bash
kubectl patch service payment-service \
  -p '{"spec":{"selector":{"version":"'$PREVIOUS'"}}}'
```

</details>

### Part D: Handle Edge Cases

Design handling for these edge cases:
1. What if the deployment to the idle environment times out?
2. What if the smoke tests pass but the production health check fails?
3. What if two deployments trigger simultaneously?
4. What if the rollback itself fails?

<details>
<summary>Hint summary>

1. Timeout: Set a generous `--timeout` on `kubectl rollout status`. If it
   fails, do not switch traffic. The active environment is unaffected.
2. Smoke pass but prod fail: This can happen if the idle environment
   behaves differently without real traffic patterns. The rollback path
   handles this.
3. Concurrent deploys: Use GitHub Actions `concurrency` to prevent
   parallel deployments to the same environment.
4. Rollback failure: This is a critical scenario. Alert the on-call
   engineer. The previous environment should still be healthy since you
   did not modify it.

</details>

## Success Criteria

- [ ] ASCII diagram shows complete flow including rollback path
- [ ] Deployment job correctly identifies active and idle environments
- [ ] Smoke tests run against idle environment before switching traffic
- [ ] Traffic switch uses kubectl patch on the Service selector
- [ ] Rollback triggers automatically on health check failure

## What You Should Understand After This Exercise

Blue-green deployment in a CI/CD pipeline is a sequence of controlled
steps: deploy to idle, verify, switch, verify again. Each step has a
failure path that either aborts (before switch) or rolls back (after
switch). The key insight is that the idle environment is your safety net
-- you can test thoroughly before exposing users to the new version.
