# Solution 05: Blue-Green with CI/CD Integration

## Part A: Deployment Flow

```
  Push image to registry
           |
           v
  ┌────────────────────┐
  │ Determine active    │  kubectl get service selector
  │ environment         │
  └────────┬───────────┘
           |
           v
  ┌────────────────────┐
  │ Deploy to idle      │  kubectl set image deployment/payment-$IDLE
  │ environment         │
  └────────┬───────────┘
           |
           v
  ┌────────────────────┐
  │ Wait for pods       │  kubectl rollout status + kubectl wait
  │ ready               │
  └────────┬───────────┘
           |
           v
  ┌────────────────────┐
  │ Smoke tests         │  kubectl port-forward + curl /health
  │ against idle        │
  └────────┬───────────┘
           |
           v
  ┌────────────────────┐
  │ Switch traffic      │  kubectl patch service selector
  └────────┬───────────┘
           |
           v
  ┌────────────────────┐
  │ Production health   │  curl https://payment.example.com/health
  │ check               │
  └────────┬───────────┘
           |
       ┌───┴───┐
       |       |
    success  failure
       |       |
       v       v
  ┌────────┐ ┌────────────┐
  │  Done  │ │  Rollback  │  kubectl patch service selector back
  └────────┘ └────────────┘
```

## Part B: Deployment Job

```yaml
  deploy-blue-green:
    runs-on: ubuntu-latest
    needs: build
    if: github.ref == 'refs/heads/main'
    environment: production
    concurrency:
      group: production-deployment
      cancel-in-progress: false
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup kubeconfig
        run: |
          mkdir -p ~/.kube
          echo "${{ secrets.KUBECONFIG }}" | base64 -d > ~/.kube/config

      - name: Determine active environment
        id: env
        run: |
          ACTIVE=$(kubectl get service payment-service \
            -o jsonpath='{.spec.selector.version}' 2>/dev/null || echo "blue")

          if [ "$ACTIVE" = "blue" ]; then
              IDLE="green"
          else
              IDLE="blue"
          fi

          echo "active=$ACTIVE" >> "$GITHUB_OUTPUT"
          echo "idle=$IDLE" >> "$GITHUB_OUTPUT"
          echo "Active: $ACTIVE, Deploying to: $IDLE"

      - name: Deploy to idle environment
        run: |
          IMAGE="ghcr.io/${{ github.repository }}:${{ github.sha }}"
          echo "Deploying $IMAGE to ${{ steps.env.outputs.idle }}"

          kubectl set image deployment/payment-${{ steps.env.outputs.idle }} \
            payment-service="$IMAGE"

          kubectl rollout status deployment/payment-${{ steps.env.outputs.idle }} \
            --timeout=180s

      - name: Wait for pods to be ready
        run: |
          kubectl wait --for=condition=ready pod \
            -l app=payment-service,version=${{ steps.env.outputs.idle }} \
            --timeout=120s

          READY=$(kubectl get pods \
            -l app=payment-service,version=${{ steps.env.outputs.idle }} \
            --field-selector=status.phase=Running \
            --no-headers | wc -l)
          echo "$READY pods ready in ${{ steps.env.outputs.idle }} environment"

      - name: Smoke test idle environment
        id: smoke
        run: |
          IDLE=${{ steps.env.outputs.idle }}

          # Port-forward to the idle deployment
          kubectl port-forward deployment/payment-$IDLE 8080:8080 &
          PF_PID=$!
          sleep 3

          # Health check
          echo "Testing /health endpoint..."
          HEALTH=$(curl -sf http://localhost:8080/health)
          echo "Health response: $HEALTH"

          # Version check
          VERSION=$(echo "$HEALTH" | jq -r '.version')
          echo "Version: $VERSION"

          # API contract test
          echo "Testing /api/v1/status endpoint..."
          STATUS=$(curl -sf http://localhost:8080/api/v1/status)
          echo "Status response: $STATUS"

          kill $PF_PID 2>/dev/null || true

          if echo "$HEALTH" | grep -q "healthy"; then
              echo "Smoke tests passed"
          else
              echo "Smoke tests failed"
              exit 1
          fi

      - name: Capture current image
        id: current
        run: |
          CURRENT_IMAGE=$(kubectl get deployment/payment-${{ steps.env.outputs.active }} \
            -o jsonpath='{.spec.template.spec.containers[0].image}')
          echo "image=$CURRENT_IMAGE" >> "$GITHUB_OUTPUT"
          echo "Current production image: $CURRENT_IMAGE"

      - name: Switch traffic to new environment
        id: switch
        run: |
          IDLE=${{ steps.env.outputs.idle }}
          echo "Switching traffic to $IDLE..."

          kubectl patch service payment-service \
            -p '{"spec":{"selector":{"version":"'$IDLE'"}}}'

          # Verify the switch
          ACTIVE_NOW=$(kubectl get service payment-service \
            -o jsonpath='{.spec.selector.version}')
          echo "Traffic now routed to: $ACTIVE_NOW"

          if [ "$ACTIVE_NOW" != "$IDLE" ]; then
              echo "ERROR: Traffic switch failed"
              exit 1
          fi

      - name: Production health check
        id: prod-health
        run: |
          echo "Waiting for production health check..."
          sleep 10

          for i in $(seq 1 12); do
            HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
              https://payment.example.com/health)

            if [ "$HTTP_CODE" = "200" ]; then
              echo "Production health check passed (attempt $i)"
              exit 0
            fi

            echo "Attempt $i/12 - HTTP $HTTP_CODE"
            sleep 5
          done

          echo "Production health check failed after 12 attempts"
          exit 1

      - name: Rollback on failure
        if: failure() && steps.switch.outputs.success != ''
        run: |
          ACTIVE=${{ steps.env.outputs.active }}
          echo "Rolling back to $ACTIVE..."

          kubectl patch service payment-service \
            -p '{"spec":{"selector":{"version":"'$ACTIVE'"}}}'

          # Verify rollback
          ROLLBACK_ENV=$(kubectl get service payment-service \
            -o jsonpath='{.spec.selector.version}')
          echo "Traffic rolled back to: $ROLLBACK_ENV"

          # Wait for rollback to stabilize
          sleep 10
          curl -sf https://payment.example.com/health
          echo "Rollback verified"

      - name: Deployment summary
        if: always()
        run: |
          echo "=== Deployment Summary ==="
          echo "Image: ghcr.io/${{ github.repository }}:${{ github.sha }}"
          echo "Active environment: ${{ steps.env.outputs.idle }}"
          echo "Previous environment: ${{ steps.env.outputs.active }}"
          echo "Previous image: ${{ steps.current.outputs.image }}"
          echo "Status: ${{ job.status }}"
```

### Why This Works

The deployment job follows the blue-green pattern with these key elements:

1. **Concurrency group:** `cancel-in-progress: false` prevents simultaneous
   deployments. If a second deployment triggers while one is in progress,
   it waits rather than canceling the first.

2. **Environment detection:** The Service selector tells us which environment
   is active. The idle environment is the opposite.

3. **Smoke tests against idle:** Port-forwarding lets us test the idle
   pods without switching traffic. This catches issues before users are
   affected.

4. **Image capture:** Before switching, we record the current image so we
   can roll back to it if needed.

5. **Rollback with `if: failure()`:** If any step after the traffic switch
   fails, the rollback step runs automatically. It patches the Service
   back to the previous environment.

## Common Mistakes to Avoid

- **Not using concurrency groups.** Without them, two pushes in quick
   succession can trigger simultaneous deployments, causing conflicts
   when both try to patch the same Service.
- **Smoke testing against production URL instead of idle pods.** If you
   test against the production URL, you are testing the old version, not
   the new one. Always test against the idle pods directly.
- **Not capturing the current image before switching.** If the rollback
   step does not know the previous image, it can only switch environments
   (which may be running an old image from a previous deployment).
- **Rollback step that does not verify success.** A rollback that patches
   the Service but does not verify the health check can leave you in a
   state where the rollback itself is broken.

## Key Takeaway

Integrating blue-green deployment into CI/CD is about creating a controlled
sequence of steps where each step has a clear success/failure path. The
idle environment is your staging area -- you deploy, test, and verify
before any user sees the new version. If anything fails after the switch,
the rollback is instant because the previous environment is still running.
