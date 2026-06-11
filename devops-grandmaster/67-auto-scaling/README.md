# Module 67: Auto-Scaling — HPA, VPA, Cluster Autoscaler

> **Previous Module:** [66 - Network Troubleshooting](../66-network-troubleshooting/README.md)
> **Next Module:** [68 - Traffic Surge Handling](../68-traffic-surge-handling/README.md)

## The Problem

Your application runs fine at 2 AM with 100 users. At 2 PM, traffic spikes 10x. Your pods get overwhelmed, response times climb to 30 seconds, and users abandon carts. You could manually scale at 1:30 PM every day, but that is not automation — that is a cron job for a human.

The real problem: **workload demand is not constant**. Static provisioning means you either overpay for idle resources during off-peak or under-provision and lose customers during peak.

## The Naive Way

```bash
# "We'll just set replicas to 50 and hope for the best"
kubectl scale deployment web --replicas=50

# Or worse, a cron job
# crontab
0 8 * * * kubectl scale deployment web --replicas=30
0 20 * * * kubectl scale deployment web --replicas=5
```

**Why this fails:**
- Manual scaling does not react to unexpected traffic
- Cron-based scaling assumes predictable patterns — real traffic is not predictable
- You still over-provision during off-peak
- No reaction to sudden spikes (viral content, flash sales)

## The Right Way

### Horizontal Pod Autoscaler (HPA)

HPA automatically adjusts the number of pod replicas based on observed metrics.

```yaml
# hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: web-hpa
  namespace: production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: web
  minReplicas: 2
  maxReplicas: 20
  metrics:
    # Metric type: Resource (CPU/Memory of pods)
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70    # Scale when average CPU > 70%
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 80    # Scale when average Memory > 80%
  behavior:
    # Scale-up: fast and aggressive
    scaleUp:
      stabilizationWindowSeconds: 30
      policies:
        - type: Percent
          value: 100                # Can double replicas
          periodSeconds: 30
        - type: Pods
          value: 4                  # Or add 4 pods
          periodSeconds: 30
      selectPolicy: Max             # Use the policy that scales more
    # Scale-down: slow and cautious
    scaleDown:
      stabilizationWindowSeconds: 300   # Wait 5 min before scaling down
      policies:
        - type: Percent
          value: 10                 # Remove at most 10% of pods
          periodSeconds: 60
```

**Key insight:** Scale up fast, scale down slow. Users notice slow scale-up (latency). You notice fast scale-down (flapping).

### Custom Metrics HPA

CPU and memory are often poor proxies for actual load. A web server might have low CPU but be waiting on database connections. Custom metrics let you scale on what matters.

```yaml
# hpa-custom-metrics.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: web-hpa-custom
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: web
  minReplicas: 2
  maxReplicas: 50
  metrics:
    # Scale on requests per second
    - type: Pods
      pods:
        metric:
          name: http_requests_per_second
        target:
          type: AverageValue
          averageValue: "1000"      # 1000 RPS per pod
    # Scale on queue depth (external metric)
    - type: External
      external:
        metric:
          name: sqs_queue_messages_visible
          selector:
            matchLabels:
              queue: "order-processing"
        target:
          type: AverageValue
          averageValue: "10"        # 10 messages per pod
```

### Vertical Pod Autoscaler (VPA)

VPA adjusts CPU and memory requests/limits for individual pods.

```yaml
# vpa.yaml
apiVersion: autoscaling.k8s.io/v1
kind: VerticalPodAutoscaler
metadata:
  name: web-vpa
  namespace: production
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: web
  updatePolicy:
    updateMode: "Auto"             # Auto, Off (recommendation only), Initial
  resourcePolicy:
    containerPolicies:
      - containerName: web
        minAllowed:
          cpu: 100m
          memory: 128Mi
        maxAllowed:
          cpu: 4
          memory: 8Gi
        controlledResources: ["cpu", "memory"]
```

**HPA vs VPA — when to use which:**

| Aspect | HPA | VPA |
|--------|-----|-----|
| What it scales | Number of replicas | Resource requests/limits per pod |
| Best for | Stateless, horizontally scalable | Stateful, vertically scalable |
| Requires | Multiple replicas | Pod restart (brief disruption) |
| Use together? | Yes, but only on CPU (not memory) | VPA for memory, HPA for CPU |

### Cluster Autoscaler

HPA creates more pods, but what if there are no nodes to run them? Cluster Autoscaler adds/removes nodes.

```yaml
# cluster-autoscaler deployment (AWS EKS example)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cluster-autoscaler
  namespace: kube-system
spec:
  replicas: 1
  selector:
    matchLabels:
      app: cluster-autoscaler
  template:
    metadata:
      labels:
        app: cluster-autoscaler
    spec:
      serviceAccountName: cluster-autoscaler
      containers:
        - image: registry.k8s.io/autoscaling/cluster-autoscaler:v1.28.0
          name: cluster-autoscaler
          command:
            - ./cluster-autoscaler
            - --v=4
            - --cloud-provider=aws
            - --skip-nodes-with-local-storage=false
            - --expander=least-waste
            - --node-group-auto-discovery=asg:tag=k8s.io/cluster-autoscaler/enabled,k8s.io/cluster-autoscaler/my-cluster
            - --balance-similar-node-groups
            - --scale-down-utilization-threshold=0.5
            - --scale-down-delay-after-add=10m
            - --scale-down-unneeded-time=10m
```

**Scaling flow:**

```
Traffic increase
    -> HPA detects high CPU
    -> HPA creates more pods
    -> Pods Pending (no node capacity)
    -> Cluster Autoscaler adds nodes
    -> Pods scheduled on new nodes
    -> Traffic decreases
    -> HPA removes pods
    -> Nodes become underutilized
    -> Cluster Autoscaler removes nodes (after delay)
```

### KEDA — Event-Driven Autoscaling

KEDA (Kubernetes Event-Driven Autoscaling) scales based on event sources — Kafka lag, RabbitMQ queue depth, AWS SQS messages, and 50+ other sources.

```yaml
# keda-scaledobject.yaml
apiVersion: keda.sh/v1alpha1
kind: ScaledObject
metadata:
  name: order-processor
  namespace: production
spec:
  scaleTargetRef:
    name: order-processor
  pollingInterval: 15               # Check queue every 15 seconds
  cooldownPeriod: 300               # Wait 5 min before scaling to zero
  minReplicaCount: 0                # Can scale to ZERO (serverless-style)
  maxReplicaCount: 50
  triggers:
    - type: rabbitmq
      metadata:
        host: amqp://rabbitmq.production.svc:5672
        queueName: orders
        queueLength: "5"            # One pod per 5 messages
    - type: cron
      metadata:
        timezone: America/New_York
        start: "0 8 * * *"         # Scale up at 8 AM
        end: "0 20 * * *"          # Scale down at 8 PM
        desiredReplicas: "10"
```

## The Production Way

### Multi-Metric Scaling Strategy

```yaml
# production-hpa.yaml — Production-grade HPA with multiple signals
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: web-production
  namespace: production
  annotations:
    # Link to runbook
    devops/runbook: "https://wiki.internal/runbooks/hpa-scaling"
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: web
  minReplicas: 3                   # Always maintain HA (3 AZs)
  maxReplicas: 100
  metrics:
    # Primary: CPU-based scaling
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 65
    # Secondary: Custom metric (request latency)
    - type: Pods
      pods:
        metric:
          name: http_request_duration_p99
        target:
          type: AverageValue
          averageValue: "500m"      # 500ms p99 latency
    # Tertiary: External (queue depth)
    - type: External
      external:
        metric:
          name: aws_sqs_approximate_number_of_messages_visible
          selector:
            matchLabels:
              queue: order-processing
        target:
          type: AverageValue
          averageValue: "20"
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 0   # React immediately to scale up
      policies:
        - type: Percent
          value: 100
          periodSeconds: 15
        - type: Pods
          value: 10
          periodSeconds: 15
      selectPolicy: Max
    scaleDown:
      stabilizationWindowSeconds: 600  # Wait 10 minutes
      policies:
        - type: Percent
          value: 10
          periodSeconds: 60
```

### Scaling Stateful Workloads

```yaml
# statefulset-hpa.yaml — HPA for StatefulSets
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: kafka-broker-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: StatefulSet
    name: kafka-broker
  minReplicas: 3                    # Minimum for quorum
  maxReplicas: 12
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
    - type: Pods
      pods:
        metric:
          name: kafka_consumer_group_lag
        target:
          type: AverageValue
          averageValue: "10000"
```

**Stateful scaling considerations:**
- StatefulSets scale up one pod at a time (ordered)
- Scale-down removes highest ordinal first
- PVCs are retained by default (data is not lost)
- Consider rebalancing time (Kafka partition reassignment)

### Monitoring Auto-Scaling

```yaml
# PrometheusRule for HPA monitoring
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: hpa-alerts
  namespace: monitoring
spec:
  groups:
    - name: hpa.rules
      rules:
        # Alert when HPA is at max capacity
        - alert: HPAAtMaxCapacity
          expr: |
            kube_horizontalpodautoscaler_status_current_replicas
            == kube_horizontalpodautoscaler_spec_max_replicas
          for: 10m
          labels:
            severity: warning
          annotations:
            summary: "HPA {{ $labels.horizontalpodautoscaler }} is at max replicas"
            description: "HPA has been at maximum capacity for 10 minutes. Consider increasing maxReplicas."

        # Alert when HPA cannot scale up (no node capacity)
        - alert: HPAScaleUpStalled
          expr: |
            kube_horizontalpodautoscaler_status_condition{condition="ScalingLimited",status="true"} == 1
          for: 5m
          labels:
            severity: critical
          annotations:
            summary: "HPA {{ $labels.horizontalpodautoscaler }} cannot scale up"

        # Metric: Scaling efficiency
        - record: hpa:scaling_efficiency
          expr: |
            kube_horizontalpodautoscaler_status_current_replicas
            / kube_horizontalpodautoscaler_spec_max_replicas
```

```python
# Grafana dashboard query for HPA metrics
# Panel: HPA Replica Count vs Target
"""
# Current replicas
kube_horizontalpodautoscaler_status_current_replicas{namespace="production"}

# Desired replicas
kube_horizontalpodautoscaler_status_desired_replicas{namespace="production"}

# Min/Max boundaries
kube_horizontalpodautoscaler_spec_max_replicas{namespace="production"}
kube_horizontalpodautoscaler_spec_min_replicas{namespace="production"}
"""
```

### Scaling Policies Best Practices

```yaml
# anti-flap-policy.yaml — Prevent scaling oscillation
behavior:
  scaleUp:
    # Never scale up faster than this
    stabilizationWindowSeconds: 60
    policies:
      - type: Pods
        value: 2
        periodSeconds: 60
  scaleDown:
    # Very conservative scale-down
    stabilizationWindowSeconds: 900    # 15 minutes
    policies:
      - type: Percent
        value: 5                       # Remove at most 5% at a time
        periodSeconds: 120
```

**Scaling anti-patterns to avoid:**
1. Symmetric scale-up/scale-down policies (causes flapping)
2. Scaling on noisy metrics (raw CPU spikes)
3. Too aggressive scale-down (premature resource reclamation)
4. Not accounting for pod startup time in stabilization windows
5. Ignoring Cluster Autoscaler node provisioning delay (2-5 minutes on cloud)

## Hands-On Lab: HPA with Custom Metrics

### Prerequisites

```bash
# Ensure metrics-server is installed
kubectl apply -f https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/components.yaml

# Install Prometheus Adapter for custom metrics
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm install prometheus-adapter prometheus-community/prometheus-adapter \
  --namespace monitoring \
  --values prometheus-adapter-values.yaml
```

### Step 1: Deploy a Sample Application

```yaml
# app-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sample-web
  namespace: default
spec:
  replicas: 2
  selector:
    matchLabels:
      app: sample-web
  template:
    metadata:
      labels:
        app: sample-web
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8080"
    spec:
      containers:
        - name: web
          image: registry.k8s.io/hpa-example
          ports:
            - containerPort: 80
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 500m
              memory: 256Mi
---
apiVersion: v1
kind: Service
metadata:
  name: sample-web
  namespace: default
spec:
  selector:
    app: sample-web
  ports:
    - port: 80
      targetPort: 80
```

### Step 2: Deploy Application with Custom Metrics Endpoint

```python
# app_with_metrics.py — Flask app exposing custom metrics
from flask import Flask, Response
import prometheus_client
from prometheus_client import Counter, Histogram, Gauge
import random
import time

app = Flask(__name__)

# Custom metrics
REQUEST_COUNT = Counter(
    'http_requests_total',
    'Total HTTP requests',
    ['method', 'endpoint', 'status']
)
REQUEST_LATENCY = Histogram(
    'http_request_duration_seconds',
    'HTTP request latency',
    ['method', 'endpoint']
)
ACTIVE_CONNECTIONS = Gauge(
    'active_connections',
    'Number of active connections'
)

@app.route('/')
def index():
    start = time.time()
    ACTIVE_CONNECTIONS.inc()
    try:
        # Simulate work
        time.sleep(random.uniform(0.01, 0.1))
        REQUEST_COUNT.labels('GET', '/', '200').inc()
        return 'Hello, World!'
    finally:
        REQUEST_LATENCY.labels('GET', '/').observe(time.time() - start)
        ACTIVE_CONNECTIONS.dec()

@app.route('/metrics')
def metrics():
    return Response(
        prometheus_client.generate_latest(),
        mimetype='text/plain'
    )

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=8080)
```

### Step 3: Create HPA with Custom Metrics

```yaml
# custom-metrics-hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: sample-web-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: sample-web
  minReplicas: 2
  maxReplicas: 10
  metrics:
    # CPU-based scaling
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 50
    # Custom metric: requests per second
    - type: Pods
      pods:
        metric:
          name: http_requests_per_second
        target:
          type: AverageValue
          averageValue: "100"
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 30
      policies:
        - type: Percent
          value: 100
          periodSeconds: 30
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
        - type: Percent
          value: 25
          periodSeconds: 60
```

### Step 4: Generate Load and Observe Scaling

```bash
# Install load testing tool
brew install k6   # or: sudo snap install k6

# Create load test script
cat > load-test.js << 'EOF'
import http from 'k6/http';
import { sleep } from 'k6';

export let options = {
  stages: [
    { duration: '2m', target: 50 },    # Ramp up to 50 VUs
    { duration: '5m', target: 50 },    # Stay at 50 VUs
    { duration: '2m', target: 200 },   # Spike to 200 VUs
    { duration: '5m', target: 200 },   # Stay at 200 VUs
    { duration: '2m', target: 0 },     # Ramp down
  ],
};

export default function () {
  http.get('http://sample-web.default.svc.cluster.local/');
  sleep(0.1);
}
EOF

# Run the load test
k6 run load-test.js

# Watch HPA in another terminal
watch -n 2 kubectl get hpa sample-web-hpa

# Watch pods being created
watch -n 2 kubectl get pods -l app=sample-web

# Check HPA events
kubectl describe hpa sample-web-hpa
```

### Step 5: Verify Scaling Behavior

```bash
# Check metrics being used
kubectl get --raw "/apis/custom.metrics.k8s.io/v1beta1/namespaces/default/pods/*/http_requests_per_second?selector=app%3Dsample-web"

# View HPA status
kubectl get hpa sample-web-hpa -o yaml

# Check cluster autoscaler logs (if configured)
kubectl logs -n kube-system -l app=cluster-autoscaler --tail=50
```

### Lab Validation Checklist

- [ ] HPA created and attached to deployment
- [ ] Metrics-server providing CPU/memory metrics
- [ ] Custom metrics visible via Prometheus Adapter
- [ ] Pods scale up when load increases
- [ ] Pods scale down after stabilization window
- [ ] No scaling oscillation observed
- [ ] Cluster Autoscaler adds nodes if needed

## Limitation -> Next Topic

Auto-scaling handles the **reactive** side — it responds after traffic arrives. But during a Black Friday flash sale, traffic does not trickle in; it floods in. By the time HPA reacts, pods are starting, connections are queuing, and users are seeing errors.

We need **proactive** strategies: pre-warming infrastructure, queue-based architectures to buffer surges, rate limiting to protect backends, and graceful degradation to serve something rather than nothing.

**Next: [Module 68 — Traffic Surge Handling](../68-traffic-surge-handling/README.md)**
