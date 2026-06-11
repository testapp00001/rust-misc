# Solution 05: Full Canary Pipeline with Automated Rollback

## Part A: Pipeline Architecture

```
  push to main
       |
       v
  ┌─────────┐
  │  Build   │  Build Docker image, push to registry
  └────┬─────┘
       |
       ├────────────┬────────────┐
       v            v            v
  ┌──────────┐ ┌──────────┐ ┌──────────┐
  │  Tests   │ │  Scan    │ │  Lint    │
  └────┬─────┘ └────┬─────┘ └────┬─────┘
       │             │            │
       └─────────────┴────────────┘
                     |
                     v
              ┌──────────────┐
              │Quality Gate  │
              └──────┬───────┘
                     |
                     v
  ┌──────────────────────────────────────────────────┐
  │              Canary Deployment                    │
  │                                                  │
  │  ┌─────────┐   ┌─────────┐   ┌─────────┐       │
  │  │  5%     │   │  25%    │   │  50%    │  100% │
  │  │ Deploy  │──>│ Deploy  │──>│ Deploy  │──>     │
  │  │ Pause   │   │ Pause   │   │ Pause   │       │
  │  │ Analyze │   │ Analyze │   │ Analyze │       │
  │  └────┬────┘   └────┬────┘   └────┬────┘       │
  │       |              |              |            │
  │    failure        failure        failure         │
  │       |              |              |            │
  │       v              v              v            │
  │  ┌─────────┐   ┌─────────┐   ┌─────────┐       │
  │  │Rollback │   │Rollback │   │Rollback │       │
  │  │weight=0 │   │weight=0 │   │weight=0 │       │
  │  └─────────┘   └─────────┘   └─────────┘       │
  └──────────────────────────────────────────────────┘
                     |
                     v
              ┌──────────────┐
              │  Notify      │  Slack notification at each stage
              └──────────────┘
```

## Part B: Argo Rollouts Configuration

```yaml
# rollout.yaml
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: api-service
  labels:
    app: api-service
  annotations:
    notifications.argoproj.io/subscribe.on-promoted.slack: deployments
    notifications.argoproj.io/subscribe.on-aborted.slack: deployments
    notifications.argoproj.io/subscribe.on-step-completed.slack: deployments
spec:
  replicas: 20
  revisionHistoryLimit: 5
  progressDeadlineSeconds: 1800  # 30 minutes total deadline
  selector:
    matchLabels:
      app: api-service
  template:
    metadata:
      labels:
        app: api-service
    spec:
      containers:
        - name: api-service
          image: api-service:v1
          ports:
            - containerPort: 8080
          readinessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 10
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 15
            periodSeconds: 20
          resources:
            requests:
              cpu: 200m
              memory: 256Mi
            limits:
              cpu: 1000m
              memory: 512Mi
  strategy:
    canary:
      steps:
        # Step 1: 5% traffic, analyze error rate and latency
        - setWeight: 5
        - pause: {duration: 2m}
        - analysis:
            templates:
              - templateName: canary-analysis
            args:
              - name: service-name
                value: api-service

        # Step 2: 25% traffic
        - setWeight: 25
        - pause: {duration: 3m}
        - analysis:
            templates:
              - templateName: canary-analysis
            args:
              - name: service-name
                value: api-service

        # Step 3: 50% traffic
        - setWeight: 50
        - pause: {duration: 3m}
        - analysis:
            templates:
              - templateName: canary-analysis
            args:
              - name: service-name
                value: api-service

        # Step 4: 100% traffic (promotion)
        - setWeight: 100
      canaryService: api-service-canary
      stableService: api-service-stable
      trafficRouting:
        nginx:
          stableIngress: api-service-ingress
      abortScaleDownDelaySeconds: 30
```

```yaml
# analysis-template.yaml
apiVersion: argoproj.io/v1alpha1
kind: AnalysisTemplate
metadata:
  name: canary-analysis
spec:
  args:
    - name: service-name
  metrics:
    - name: error-rate
      interval: 30s
      count: 10
      failureLimit: 1
      successCondition: result[0] <= 0.01
      provider:
        prometheus:
          address: http://prometheus:9090
          query: |
            sum(rate(http_requests_total{
              service="{{args.service-name}}",
              version="canary",
              status=~"5.."
            }[5m])) /
            sum(rate(http_requests_total{
              service="{{args.service-name}}",
              version="canary"
            }[5m]))

    - name: latency-p99
      interval: 1m
      count: 5
      failureLimit: 2
      successCondition: result[0] <= 2.0
      provider:
        prometheus:
          address: http://prometheus:9090
          query: |
            histogram_quantile(0.99,
              sum(rate(http_request_duration_seconds_bucket{
                service="{{args.service-name}}",
                version="canary"
              }[5m])) by (le)
            ) /
            histogram_quantile(0.99,
              sum(rate(http_request_duration_seconds_bucket{
                service="{{args.service-name}}",
                version="stable"
              }[5m])) by (le)
            )

    - name: throughput
      interval: 1m
      count: 5
      failureLimit: 2
      successCondition: result[0] >= 500
      provider:
        prometheus:
          address: http://prometheus:9090
          query: |
            sum(rate(http_requests_total{
              service="{{args.service-name}}",
              version="canary"
            }[5m]))
```

### Why This Works

The `progressDeadlineSeconds: 1800` (30 minutes) ensures the canary does
not hang indefinitely. If the rollout does not complete within 30 minutes
(e.g., Prometheus is down and analysis never finishes), Argo Rollouts
automatically aborts.

The `abortScaleDownDelaySeconds: 30` keeps canary pods running for 30
seconds after abort, allowing in-flight requests to complete before
the pods are terminated.

Slack notifications are configured via annotations. The Argo Rollouts
notification controller sends messages when the canary is promoted,
aborted, or completes a step.

## Part C: CI/CD Pipeline Job

```yaml
# .github/workflows/canary-deploy.yaml
name: Canary Deploy

on:
  push:
    branches: [main]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  build:
    runs-on: ubuntu-latest
    outputs:
      image-tag: ${{ steps.meta.outputs.version }}
    steps:
      - uses: actions/checkout@v4

      - name: Setup Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Login to GHCR
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=sha,prefix=

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          cache-from: type=gha
          cache-to: type=gha,mode=max

  test:
    runs-on: ubuntu-latest
    needs: build
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Run tests
        run: cargo test

  canary-deploy:
    runs-on: ubuntu-latest
    needs: [build, test]
    if: github.ref == 'refs/heads/main'
    environment: production
    concurrency:
      group: canary-deployment
      cancel-in-progress: false
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup kubeconfig
        run: |
          mkdir -p ~/.kube
          echo "${{ secrets.KUBECONFIG }}" | base64 -d > ~/.kube/config

      - name: Install Argo Rollouts plugin
        run: |
          curl -LO https://github.com/argoproj/argo-rollouts/releases/latest/download/kubectl-argo-rollouts-linux-amd64
          chmod +x kubectl-argo-rollouts-linux-amd64
          sudo mv kubectl-argo-rollouts-linux-amd64 /usr/local/bin/kubectl-argo-rollouts

      - name: Start canary deployment
        run: |
          IMAGE="${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ github.sha }}"
          echo "Starting canary deployment with image: $IMAGE"

          kubectl argo rollouts set image api-service \
            api-service="$IMAGE"

          echo "Canary deployment triggered"

      - name: Monitor canary progress
        id: canary
        run: |
          echo "Monitoring canary deployment (timeout: 30 minutes)..."

          # Wait for rollout to complete or fail
          if kubectl argo rollouts status api-service --timeout=1800s; then
            echo "status=success" >> "$GITHUB_OUTPUT"
            echo "Canary deployment completed successfully"
          else
            echo "status=failed" >> "$GITHUB_OUTPUT"
            echo "Canary deployment failed or was aborted"
            exit 1
          fi

      - name: Get final status
        if: always()
        run: |
          echo "=== Canary Deployment Summary ==="
          kubectl argo rollouts get rollout api-service

          STATUS=$(kubectl argo rollouts get rollout api-service \
            -o json | jq -r '.status.phase')
          echo "Final status: $STATUS"

          if [ "$STATUS" = "Degraded" ] || [ "$STATUS" = "Abort" ]; then
            echo "Canary was rolled back"
            echo "Check Argo Rollouts logs for analysis details"
          fi
```

### Why This Works

The `concurrency` group with `cancel-in-progress: false` prevents
simultaneous canary deployments. If a second push happens while a canary
is in progress, the second deployment waits.

The `kubectl argo rollouts status` command blocks until the rollout
completes, fails, or times out. The 30-minute timeout matches the
`progressDeadlineSeconds` on the Rollout resource.

The `environment: production` key enables manual approval if configured
in the repository settings.

## Part D: Rollback Strategy

### Failure Mode 1: Analysis fails at 5% weight

**Automatic behavior:** Argo Rollouts sets canary weight to 0. All traffic
returns to stable. Canary pods are scaled down after
`abortScaleDownDelaySeconds`.

**Impact:** 5% of users (1,000 req/s) experienced issues for the duration
of the analysis (up to 5 minutes). Total affected requests: ~300,000.

**Recovery time:** < 1 second (traffic switch) + 30 seconds (pod scale-down).

### Failure Mode 2: Analysis fails at 50% weight

**Automatic behavior:** Same as above -- weight set to 0.

**Impact:** 50% of users (25,000 req/s) experienced issues for the duration
of the analysis at 50% (up to 3 minutes). Total affected requests:
~4,500,000.

**Recovery time:** < 1 second + 30 seconds.

**Why this is acceptable:** The analysis at 25% passed, meaning the canary
was healthy at lower traffic. The issue appeared at 50%, suggesting a
load-dependent problem. The analysis caught it within 3 minutes.

### Failure Mode 3: Rollout controller crashes during analysis

**Automatic behavior:** The `progressDeadlineSeconds: 1800` on the Rollout
resource triggers an automatic abort if the rollout does not progress
within 30 minutes. When the controller recovers, it sees the deadline
exceeded and aborts the canary.

**Impact:** Depends on the traffic weight when the controller crashed.
If it crashed at 25%, 25% of traffic continues hitting canary pods until
the deadline expires.

**Mitigation:** The controller is a single point of failure. In production,
run multiple replicas of the Argo Rollouts controller with leader election.

### Failure Mode 4: Prometheus is down during analysis

**Automatic behavior:** The analysis metric returns an error (no data),
which counts as a failure. After `failureLimit` failures (1 for error
rate, 2 for latency), the analysis fails and the canary is aborted.

**Impact:** The canary is aborted even though it might be healthy. This is
the safe default -- if you cannot measure, you cannot promote.

**Mitigation:** Increase `failureLimit` for metrics that depend on external
infrastructure, or add a fallback metric that does not require Prometheus
(e.g., a Kubernetes event count).

```yaml
# Fallback metric: pod restart count (no Prometheus needed)
- name: pod-restarts
  interval: 1m
  count: 3
  failureLimit: 0
  successCondition: result[0] == 0
  provider:
    kubernetes:
      resource:
        apiVersion: v1
        kind: Pod
        namespace: default
        labelSelector: app=api-service,version=canary
      jsonPath: "{.status.containerStatuses[0].restartCount}"
```

## Common Mistakes to Avoid

- **Not setting progressDeadlineSeconds.** Without it, a stuck canary
  (e.g., Prometheus down) hangs indefinitely. Always set a deadline.
- **Too-short total timeout.** With 4 steps and 2-3 minute pauses plus
  5-10 minute analysis windows, the total time is 20-40 minutes. Set
  `progressDeadlineSeconds` to at least 30 minutes.
- **Not handling the Prometheus-down scenario.** If Prometheus is down,
  all analysis metrics fail. This aborts the canary, which is safe but
  might be frustrating if the canary is actually healthy. Consider a
  fallback metric.
- **No notification on abort.** If the canary aborts silently, nobody
  knows the deployment failed. Always configure notifications for abort
  and promote events.

## Key Takeaway

A production canary pipeline is an automated feedback loop: deploy,
measure, decide, repeat. Each step is gated by multi-dimensional metric
analysis. The pipeline handles all failure modes automatically: analysis
failure triggers rollback, controller crash triggers deadline abort,
Prometheus failure triggers metric-level abort. The result is a deployment
system that is safe by default and provides fast feedback at every step.
