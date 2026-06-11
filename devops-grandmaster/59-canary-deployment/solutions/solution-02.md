# Solution 02: Build a Canary with Argo Rollouts

## Part A: Rollout Resource

```yaml
# rollout.yaml
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: search-service
  labels:
    app: search-service
spec:
  replicas: 5
  revisionHistoryLimit: 3
  selector:
    matchLabels:
      app: search-service
  template:
    metadata:
      labels:
        app: search-service
    spec:
      containers:
        - name: search-service
          image: search-service:v1
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
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 500m
              memory: 256Mi
  strategy:
    canary:
      steps:
        - setWeight: 20
        - pause: {duration: 2m}
        - setWeight: 40
        - pause: {duration: 2m}
        - setWeight: 60
        - pause: {duration: 2m}
        - setWeight: 80
        - pause: {duration: 2m}
      canaryService: search-service-canary
      stableService: search-service-stable
      trafficRouting:
        nginx:
          stableIngress: search-service-ingress
```

### Why This Works

The `Rollout` resource is a drop-in replacement for a Kubernetes
Deployment. The `strategy.canary.steps` define the progression:
set weight, pause to observe, repeat. Argo Rollouts manages the replica
counts to achieve the desired weight percentage.

The `canaryService` and `stableService` fields tell Argo Rollouts to
create two Services: one for canary pods and one for stable pods. The
ingress controller routes traffic between them based on weight.

## Part B: Analysis Template

```yaml
# analysis-template.yaml
apiVersion: argoproj.io/v1alpha1
kind: AnalysisTemplate
metadata:
  name: error-rate
spec:
  args:
    - name: service-name
  metrics:
    - name: error-rate
      interval: 1m
      count: 5
      successCondition: result[0] <= 0.01
      failureLimit: 3
      provider:
        prometheus:
          address: http://prometheus:9090
          query: |
            sum(rate(http_requests_total{
              service="{{args.service-name}}",
              status=~"5.."
            }[5m])) /
            sum(rate(http_requests_total{
              service="{{args.service-name}}"
            }[5m]))
```

### Why This Works

The `successCondition` uses `result[0]` because Prometheus returns an
instant vector, which is an array. `result[0]` is the first (and only)
value.

The `count: 5` means the analysis runs 5 times at `interval: 1m`
(5 minutes total). The `failureLimit: 3` allows up to 3 failures
before the analysis is considered failed. This handles transient spikes
that might cause a single bad measurement.

The query calculates the ratio of 5xx responses to total responses over
a 5-minute window. If more than 1% of requests return 5xx, the condition
fails.

## Part C: Integrated Rollout with Analysis

```yaml
# rollout-with-analysis.yaml
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: search-service
  labels:
    app: search-service
spec:
  replicas: 5
  revisionHistoryLimit: 3
  selector:
    matchLabels:
      app: search-service
  template:
    metadata:
      labels:
        app: search-service
    spec:
      containers:
        - name: search-service
          image: search-service:v1
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
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 500m
              memory: 256Mi
  strategy:
    canary:
      steps:
        - setWeight: 20
        - pause: {duration: 2m}
        - analysis:
            templates:
              - templateName: error-rate
            args:
              - name: service-name
                value: search-service
        - setWeight: 40
        - pause: {duration: 2m}
        - analysis:
            templates:
              - templateName: error-rate
            args:
              - name: service-name
                value: search-service
        - setWeight: 60
        - pause: {duration: 2m}
        - analysis:
            templates:
              - templateName: error-rate
            args:
              - name: service-name
                value: search-service
        - setWeight: 80
        - pause: {duration: 2m}
        - analysis:
            templates:
              - templateName: error-rate
            args:
              - name: service-name
                value: search-service
      canaryService: search-service-canary
      stableService: search-service-stable
      trafficRouting:
        nginx:
          stableIngress: search-service-ingress
```

### Why This Works

Each `analysis` step creates an `AnalysisRun` resource that queries
Prometheus over the specified interval. If the analysis passes, the
rollout proceeds to the next weight step. If it fails (exceeds
`failureLimit`), the rollout automatically aborts and rolls back.

The analysis runs after the pause, so the canary has 2 minutes of
traffic at the new weight before metrics are evaluated. This gives the
metrics time to stabilize after the weight change.

## Part D: Monitoring Commands

```bash
# Watch canary progress in real-time
kubectl argo rollouts get rollout search-service --watch

# Example output:
# Name:            search-service
# Namespace:       default
# Status:          ॥ Paused
# Message:         CanaryPauseStep
# Strategy:        Canary
#  Step 3/8: 40%
#  Images:          search-service:v1 (stable)
#                   search-service:v2 (canary)
#  Replicas:
#    Desired: 5
#    Current: 5
#  Conditions:
#    Type           Status  Reason
#    ----           ------  ------
#    Progressing    True    CanaryPauseStep
#    Available      True    MinimumReplicasAvailable

# Manually promote to next step (skip remaining pause time)
kubectl argo rollouts promote search-service

# Abort canary and roll back to stable
kubectl argo rollouts abort search-service

# Check rollout history
kubectl argo rollouts history search-service

# Get current rollout status as JSON
kubectl argo rollouts get rollout search-service -o json | jq '.status'
```

### Why This Works

The `--watch` flag streams live updates as the canary progresses through
steps. `promote` skips the current pause and moves to the next step.
`abort` immediately sets the canary weight to 0, routing all traffic
back to stable.

## Common Mistakes to Avoid

- **Not defining canaryService and stableService.** Without these, Argo
  Rollouts cannot create separate Services for canary and stable pods.
  Traffic splitting requires two Services.
- **Analysis without a count.** Without `count`, the analysis runs once.
  A single Prometheus query might return an anomalous value. Running
  multiple times with `count` provides more reliable results.
- **Too-low failureLimit.** Setting `failureLimit: 0` means a single
  failed measurement aborts the canary. Transient network issues can
  cause false positives. Use `failureLimit: 2-3` for reliability.
- **Not running analysis at every step.** If you only analyze at the
  final step, a bug that appears at 25% traffic is not caught until
  100%. Analyze at every weight step.

## Key Takeaway

Argo Rollouts automates the canary process by defining weight steps,
pause durations, and analysis templates. The controller manages traffic
splitting and metric evaluation. If analysis fails at any step, the
canary is automatically aborted and rolled back. This turns canary
deployment from a manual, error-prone process into a safe, automated one.
