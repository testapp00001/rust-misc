# 59 - Canary Deployment

## Problem

You have a new version of your service ready to ship. Blue-Green deployment lets you switch all traffic at once, but that means every user hits the new version simultaneously. If there is a bug, a performance regression, or an unexpected error spike, 100% of your users are affected before you can react. You need a way to expose the new version to a small subset of users first, observe its behavior under real traffic, and only then promote it to everyone.

Canary deployment solves this by routing a controlled fraction of traffic (say 5%) to the new version while the remaining 95% stays on the proven version. You monitor the canary for errors, latency, and business metrics. If it looks healthy, you gradually increase the percentage. If it degrades, you roll back instantly with minimal user impact.

## Naive Way

Manually deploy the new version alongside the old one and use a load balancer rule to send a fixed percentage of traffic to it.

```nginx
# nginx.conf - manual traffic splitting
upstream backend {
    server old-version:8080 weight=95;
    server new-version:8080 weight=5;
}
```

Deploy the canary pod and update the load balancer config by hand:

```bash
# Deploy canary alongside production
kubectl apply -f canary-deployment.yaml

# Manually edit nginx configmap to add canary backend
kubectl edit configmap nginx-config
# Add weight=5 line for new version

# Reload nginx
kubectl rollout restart deployment/nginx-ingress
```

This works but has serious problems. Every percentage change requires a manual config edit and reload. There is no automated health analysis. Rolling back means another manual edit. If you forget to remove the canary deployment after promotion, you have two full copies running. The entire process is error-prone and slow.

## Right Way

Use a progressive delivery tool like Flagger or Argo Rollouts that automates the canary process with built-in metric analysis.

**Using Argo Rollouts:**

```yaml
# rollout.yaml
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: my-service
spec:
  replicas: 3
  strategy:
    canary:
      steps:
        - setWeight: 5
        - pause: { duration: 5m }
        - analysis:
            templates:
              - templateName: success-rate
            args:
              - name: service-name
                value: my-service
        - setWeight: 25
        - pause: { duration: 5m }
        - analysis:
            templates:
              - templateName: success-rate
        - setWeight: 50
        - pause: { duration: 5m }
        - setWeight: 100
  selector:
    matchLabels:
      app: my-service
  template:
    metadata:
      labels:
        app: my-service
    spec:
      containers:
        - name: my-service
          image: my-service:v2.0.0
          ports:
            - containerPort: 8080
---
# Analysis template - decides if canary is healthy
apiVersion: argoproj.io/v1alpha1
kind: AnalysisTemplate
metadata:
  name: success-rate
spec:
  args:
    - name: service-name
  metrics:
    - name: success-rate
      interval: 1m
      successCondition: result[0] >= 0.99
      failureLimit: 3
      provider:
        prometheus:
          address: http://prometheus:9090
          query: |
            sum(rate(http_requests_total{
              service="{{args.service-name}}",
              status=~"2.."
            }[5m])) /
            sum(rate(http_requests_total{
              service="{{args.service-name}}"
            }[5m]))
```

Deploy and let Argo Rollouts handle the progression:

```bash
# Install Argo Rollouts controller
kubectl create namespace argo-rollouts
kubectl apply -n argo-rollouts -f https://github.com/argoproj/argo-rollouts/releases/latest/download/install.yaml

# Deploy the rollout
kubectl apply -f rollout.yaml

# Watch the canary progress
kubectl argo rollouts get rollout my-service --watch

# Manual promotion if needed
kubectl argo rollouts promote my-service

# Abort and rollback
kubectl argo rollouts abort my-service
```

**Using Flagger with Istio:**

```yaml
# flagger-canary.yaml
apiVersion: flagger.app/v1beta1
kind: Canary
metadata:
  name: my-service
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: my-service
  progressDeadlineSeconds: 600
  service:
    port: 8080
    gateways:
      - public-gateway.istio-system.svc.cluster.local
    hosts:
      - my-service.example.com
    trafficPolicy:
      tls:
        mode: ISTIO_MUTUAL
  analysis:
    interval: 1m
    threshold: 5
    maxWeight: 50
    stepWeight: 10
    metrics:
      - name: request-success-rate
        thresholdRange:
          min: 99
        interval: 1m
      - name: request-duration
        thresholdRange:
          max: 500
        interval: 1m
    webhooks:
      - name: load-test
        type: rollout
        url: http://flagger-loadtester.test/
        timeout: 5s
        metadata:
          cmd: "hey -z 1m -q 10 -c 2 http://my-service-canary.test:8080/"
```

## Production Way

In production, canary deployments integrate with your observability stack, support automated rollback, and handle edge cases like session affinity and header-based routing for internal testing.

**Full production canary with Istio VirtualService for traffic splitting:**

```yaml
# Istio VirtualService for fine-grained traffic control
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: my-service
spec:
  hosts:
    - my-service
  http:
    # Route internal testers to canary always
    - match:
        - headers:
            x-canary:
              exact: "true"
      route:
        - destination:
            host: my-service
            subset: canary
          weight: 100
    # Normal traffic split
    - route:
        - destination:
            host: my-service
            subset: stable
          weight: 95
        - destination:
            host: my-service
            subset: canary
          weight: 5
      retries:
        attempts: 3
        perTryTimeout: 2s
---
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: my-service
spec:
  host: my-service
  subsets:
    - name: stable
      labels:
        version: v1
    - name: canary
      labels:
        version: v2
  trafficPolicy:
    connectionPool:
      tcp:
        maxConnections: 100
      http:
        h2UpgradePolicy: DEFAULT
        http1MaxPendingRequests: 100
        http2MaxRequests: 1000
    outlierDetection:
      consecutive5xxErrors: 5
      interval: 30s
      baseEjectionTime: 30s
```

**Canary with automated metric-based promotion in CI/CD:**

```yaml
# .github/workflows/canary-deploy.yaml
name: Canary Deploy
on:
  push:
    branches: [main]

jobs:
  canary:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build and push image
        run: |
          docker build -t my-service:${{ github.sha }} .
          docker push my-service:${{ github.sha }}

      - name: Update canary image
        run: |
          kubectl argo rollouts set image my-service \
            my-service=my-service:${{ github.sha }}

      - name: Wait for canary analysis
        run: |
          kubectl argo rollouts status my-service \
            --timeout=600s

      - name: Check rollout status
        run: |
          STATUS=$(kubectl argo rollouts get rollout my-service \
            -o json | jq -r '.status.phase')
          if [ "$STATUS" != "Healthy" ]; then
            echo "Canary failed analysis, rolling back"
            kubectl argo rollouts abort my-service
            exit 1
          fi
```

**Key production considerations:**

- **Session affinity**: Use consistent hashing so users stay on the same version during a session.
- **Header-based routing**: Let QA and internal users hit the canary via special headers before opening to real traffic.
- **Metric thresholds**: Define SLO-based thresholds (p99 latency < 500ms, error rate < 1%) rather than generic checks.
- **Automatic rollback**: If failure threshold is crossed, roll back within seconds, not minutes.
- **Canary in the canary**: For critical services, canary your canary process itself with a small subset before enabling it broadly.

## Hands-On Lab

**Exercise: Deploy a canary with Argo Rollouts**

1. Install Argo Rollouts in a local cluster:

```bash
# Create a kind cluster
kind create cluster --name canary-lab

# Install Argo Rollouts
kubectl create namespace argo-rollouts
kubectl apply -n argo-rollouts \
  -f https://github.com/argoproj/argo-rollouts/releases/latest/download/install.yaml

# Install Prometheus for metric analysis
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm install prometheus prometheus-community/kube-prometheus-stack \
  -n monitoring --create-namespace
```

2. Deploy a baseline application:

```yaml
# baseline.yaml
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: web-app
spec:
  replicas: 3
  strategy:
    canary:
      steps:
        - setWeight: 20
        - pause: { duration: 30s }
        - setWeight: 40
        - pause: { duration: 30s }
        - setWeight: 60
        - pause: { duration: 30s }
        - setWeight: 80
        - pause: { duration: 30s }
  selector:
    matchLabels:
      app: web-app
  template:
    metadata:
      labels:
        app: web-app
    spec:
      containers:
        - name: web-app
          image: nginx:1.24
          ports:
            - containerPort: 80
          resources:
            requests:
              cpu: 50m
              memory: 64Mi
---
apiVersion: v1
kind: Service
metadata:
  name: web-app
spec:
  selector:
    app: web-app
  ports:
    - port: 80
      targetPort: 80
```

```bash
kubectl apply -f baseline.yaml
```

3. Trigger a canary update and watch it progress:

```bash
# Update the image to trigger canary
kubectl argo rollouts set image web-app web-app=nginx:1.25

# Watch the rollout in real time
kubectl argo rollouts get rollout web-app --watch
```

4. Practice manual promotion and rollback:

```bash
# In another terminal, manually promote the canary
kubectl argo rollouts promote web-app

# Or abort and roll back
kubectl argo rollouts abort web-app

# Verify rollback
kubectl argo rollouts get rollout web-app
```

5. Add header-based routing with an Ingress:

```yaml
# canary-ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: web-app-canary
  annotations:
    nginx.ingress.kubernetes.io/canary: "true"
    nginx.ingress.kubernetes.io/canary-by-header: "x-canary"
spec:
  ingressClassName: nginx
  rules:
    - host: web-app.local
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: web-app-canary
                port:
                  number: 80
```

Test with:

```bash
# Normal traffic goes to stable
curl http://web-app.local

# Header forces canary
curl -H "x-canary: true" http://web-app.local
```

## Limitation

Canary deployments handle gradual rollout and metric-based promotion well, but they assume your infrastructure and application configuration are managed declaratively. When you have dozens of services, each with their own rollout configs, Istio VirtualServices, DestinationRules, and analysis templates, manually applying YAML files becomes unmanageable. You need a system that watches your Git repository and automatically synchronizes the desired state to your cluster. This leads directly to GitOps.

## Next Topic

[60 - GitOps](../60-gitops/README.md) - Declarative infrastructure management with ArgoCD and Flux, where your Git repository becomes the single source of truth for your entire cluster state.
