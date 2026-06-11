# Solution 05: Set Up Jaeger with Kubernetes and Correlate with Metrics

## Part A: Deploy Jaeger on Kubernetes

### helm-values.yaml

```yaml
# Custom values for Jaeger Helm chart
storage:
  type: memory

agent:
  enabled: true

collector:
  service:
    type: ClusterIP

query:
  service:
    type: NodePort
    nodePort: 30686

provisionDataStore:
  cassandra: false
  elasticsearch: false
```

### Deployment Commands

```bash
# Add Helm repo and install
helm repo add jaegertracing https://jaegertracing.github.io/helm-charts
helm repo update

kubectl create namespace tracing

helm install jaeger jaegertracing/jaeger \
  --namespace tracing \
  --values helm-values.yaml

# Verify
kubectl get pods -n tracing
# Expected output:
# NAME                                READY   STATUS    RESTARTS   AGE
# jaeger-collector-xxxxx              1/1     Running   0          30s
# jaeger-query-xxxxx                  1/1     Running   0          30s
# jaeger-agent-xxxxx                  1/1     Running   0          30s

# Get Jaeger UI URL
minikube service jaeger-query -n tracing --url
# Or port-forward:
kubectl port-forward -n tracing svc/jaeger-query 16686:16686
```

## Part B: Sample Application

### Dockerfile

```dockerfile
FROM python:3.11-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY app.py .

EXPOSE 5000

CMD ["python", "app.py"]
```

### requirements.txt

```
flask==3.0.0
requests==2.31.0
prometheus_client==0.19.0
opentelemetry-api==1.21.0
opentelemetry-sdk==1.21.0
opentelemetry-exporter-otlp-proto-grpc==1.21.0
opentelemetry-instrumentation-flask==0.42b0
opentelemetry-instrumentation-requests==0.42b0
```

### app.py

```python
import os
import time
import random
from flask import Flask, jsonify, Response
from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from opentelemetry.sdk.resources import Resource, SERVICE_NAME
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.instrumentation.flask import FlaskInstrumentor
from opentelemetry.instrumentation.requests import RequestsInstrumentor
from prometheus_client import Histogram, Counter, generate_latest, CONTENT_TYPE_LATEST

# --- Prometheus Metrics ---
REQUEST_DURATION = Histogram(
    "http_request_duration_seconds",
    "HTTP request duration in seconds",
    ["method", "endpoint", "status"],
    buckets=[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
)

REQUEST_COUNT = Counter(
    "http_requests_total",
    "Total HTTP requests",
    ["method", "endpoint", "status"],
)


def init_tracing():
    """Configure OpenTelemetry with OTLP exporter."""
    otlp_endpoint = os.environ.get(
        "OTEL_EXPORTER_OTLP_ENDPOINT", "localhost:4317"
    )
    service_name = os.environ.get("OTEL_SERVICE_NAME", "order-service")

    resource = Resource.create({SERVICE_NAME: service_name})
    provider = TracerProvider(resource=resource)

    otlp_exporter = OTLPSpanExporter(
        endpoint=otlp_endpoint,
        insecure=True,
    )
    provider.add_span_processor(BatchSpanProcessor(otlp_exporter))
    trace.set_tracer_provider(provider)

    RequestsInstrumentor().instrument()

    return provider


app = Flask(__name__)
provider = init_tracing()
FlaskInstrumentor().instrument_app(app)

tracer = trace.get_tracer(__name__)


def record_metrics(method, endpoint, status, duration):
    """Record Prometheus metrics with exemplar."""
    span = trace.get_current_span()
    ctx = span.get_span_context()
    trace_id = format(ctx.trace_id, "032x") if ctx.trace_id else "0"

    REQUEST_DURATION.labels(
        method=method, endpoint=endpoint, status=str(status)
    ).observe(duration, exemplar={"trace_id": trace_id})

    REQUEST_COUNT.labels(
        method=method, endpoint=endpoint, status=str(status)
    ).inc()


@app.route("/health")
def health():
    return jsonify({"status": "healthy"}), 200


@app.route("/api/order")
def create_order():
    start = time.time()

    with tracer.start_as_current_span("validate-order") as span:
        span.set_attribute("order.item_count", random.randint(1, 5))
        time.sleep(random.uniform(0.01, 0.05))

    with tracer.start_as_current_span("process-order") as span:
        order_id = f"ORD-{random.randint(10000, 99999)}"
        span.set_attribute("order.id", order_id)

        with tracer.start_as_current_span("db-insert"):
            time.sleep(random.uniform(0.02, 0.08))

        with tracer.start_as_current_span("db-commit"):
            time.sleep(random.uniform(0.005, 0.02))

    with tracer.start_as_current_span("send-confirmation"):
        time.sleep(random.uniform(0.01, 0.03))

    duration = time.time() - start
    record_metrics("GET", "/api/order", 200, duration)

    return jsonify({"order_id": order_id, "status": "created"}), 201


@app.route("/api/order/status")
def order_status():
    start = time.time()

    with tracer.start_as_current_span("lookup-order"):
        time.sleep(random.uniform(0.01, 0.05))

    duration = time.time() - start
    record_metrics("GET", "/api/order/status", 200, duration)

    return jsonify({"status": "processing"}), 200


@app.route("/metrics")
def metrics():
    return Response(generate_latest(), mimetype=CONTENT_TYPE_LATEST)


if __name__ == "__main__":
    app.run(host="0.0.0.0", port=5000)
```

### k8s-deployment.yaml

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: order-service
  labels:
    app: order-service
spec:
  replicas: 2
  selector:
    matchLabels:
      app: order-service
  template:
    metadata:
      labels:
        app: order-service
    spec:
      containers:
        - name: order-service
          image: order-service:1.0
          imagePullPolicy: IfNotPresent
          ports:
            - containerPort: 5000
              name: http
          env:
            - name: OTEL_EXPORTER_OTLP_ENDPOINT
              value: "jaeger-collector.tracing.svc.cluster.local:4317"
            - name: OTEL_SERVICE_NAME
              value: "order-service"
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 250m
              memory: 256Mi
          livenessProbe:
            httpGet:
              path: /health
              port: 5000
            initialDelaySeconds: 10
            periodSeconds: 15
            timeoutSeconds: 3
          readinessProbe:
            httpGet:
              path: /health
              port: 5000
            initialDelaySeconds: 5
            periodSeconds: 10
            timeoutSeconds: 3
---
apiVersion: v1
kind: Service
metadata:
  name: order-service
  labels:
    app: order-service
spec:
  selector:
    app: order-service
  ports:
    - port: 5000
      targetPort: 5000
      name: http
  type: ClusterIP
```

### service-monitor.yaml

```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: order-service-monitor
  namespace: monitoring
  labels:
    release: prometheus
spec:
  selector:
    matchLabels:
      app: order-service
  namespaceSelector:
    matchNames:
      default
  endpoints:
    - port: http
      path: /metrics
      interval: 15s
```

### Build and Deploy

```bash
# Build the Docker image
docker build -t order-service:1.0 .

# For minikube: use minikube's Docker daemon
eval $(minikube docker-env)
docker build -t order-service:1.0 .

# For kind: load image into cluster
kind load docker-image order-service:1.0

# Deploy
kubectl apply -f k8s-deployment.yaml

# Verify
kubectl get pods -l app=order-service
kubectl get svc order-service

# Generate traffic
kubectl port-forward svc/order-service 5000:5000 &
for i in $(seq 1 50); do
  curl -s http://localhost:5000/api/order > /dev/null
  curl -s http://localhost:5000/api/order/status > /dev/null
  sleep 0.1
done

# Check Jaeger UI — select "order-service" from the Service dropdown
```

## Part C: Metric-Trace Correlation

### Deploy Prometheus

```bash
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo update

helm install prometheus prometheus-community/prometheus \
  --namespace monitoring \
  --create-namespace \
  --set server.service.type=NodePort

# Verify
kubectl get pods -n monitoring
```

### How Exemplars Work

Exemplars link a metric data point to a specific trace. Here is the flow:

```
1. HTTP request arrives at /api/order
2. OpenTelemetry creates a span (traceId=abc123, spanId=def456)
3. Request processing completes
4. Prometheus metric is recorded with an exemplar:
   http_request_duration_seconds_bucket{method="GET",endpoint="/api/order",le="0.1"} 42
   # exemplar: {trace_id="abc123"} 0.087
5. Prometheus scrapes /metrics and stores the exemplar
6. In Grafana, hover over a metric data point
7. Click the exemplar dot — it opens Jaeger with traceId=abc123
```

### Key Code: Recording Exemplars

```python
from opentelemetry import trace

REQUEST_DURATION.labels(
    method="GET", endpoint="/api/order", status="200"
).observe(duration, exemplar={"trace_id": trace_id})
```

The `exemplar` parameter is a dictionary. Prometheus stores it alongside the metric sample. The `trace_id` key is conventional — Grafana recognizes it and creates a clickable link to your tracing backend (Jaeger, Tempo, etc.).

### Accessing the Metrics

```bash
# Port-forward to Prometheus
kubectl port-forward -n monitoring svc/prometheus-server 9090:80

# Query: http_request_duration_seconds_count
# Look for exemplar dots on the graph

# Port-forward to Grafana (if installed)
kubectl port-forward -n monitoring svc/grafana 3000:80
```

## Part D: Analysis — Answers

**1. Jaeger collector endpoint in Kubernetes vs local:**

Locally, you use `localhost:6831` (agent) or `localhost:4317` (OTLP). In Kubernetes, you use the fully qualified domain name of the Jaeger collector service: `jaeger-collector.tracing.svc.cluster.local:4317`.

The service discovery mechanism is Kubernetes DNS. The format is `<service-name>.<namespace>.svc.cluster.local`. The `svc.cluster.local` suffix is the cluster's internal DNS domain. Services in the same namespace can use just the service name (`jaeger-collector`), but using the FQDN is more explicit and works across namespaces.

**2. Collector vs Agent:**

- **Agent** (UDP, port 6831): A sidecar or daemonset that runs alongside the application. The app sends spans to the agent via UDP (fire-and-forget). The agent batches and forwards to the collector. Use this for high-throughput applications where you do not want the app to block on export.

- **Collector** (gRPC, port 4317 or HTTP, port 4318): A centralized service that receives spans directly. Use this when you want reliable delivery (gRPC has acknowledgments), need to process spans (sampling, enrichment), or are running the OpenTelemetry Collector as an intermediary.

In Kubernetes, the recommended pattern is:
- Application -> OTLP/gRPC -> Jaeger Collector
- The collector handles storage, sampling, and fan-out to multiple backends.

**3. How exemplars bridge metrics and traces:**

Metrics are aggregated — a P99 latency of 500ms tells you something is slow but not which specific request. Traces show individual request flows but are too numerous to browse manually.

Exemplars solve this by annotating specific metric samples with trace IDs. When you see a spike in the latency metric, the exemplars on those high data points link directly to the traces that caused the spike. You can click through to see exactly what happened in that slow request.

This eliminates the "which request was slow?" problem. Without exemplars, you would have to:
1. Notice the metric spike.
2. Open the tracing UI.
3. Manually search for traces in the same time window.
4. Hope to find the slow one.

With exemplars, step 2-4 is a single click.

**4. Finding slow traces from a metric spike:**

1. Open Grafana and view the `http_request_duration_seconds` metric.
2. Find the spike at 3:00 PM.
3. Hover over the data points at the spike — exemplar dots appear.
4. Click an exemplar — Grafana opens Jaeger with the specific trace ID.
5. In Jaeger, examine the waterfall view to see which spans are slow.
6. Use Jaeger's "Compare" feature to compare this trace with a healthy baseline trace.

## Common Mistakes

1. **Using localhost for OTLP endpoint in Kubernetes**: Inside a pod, `localhost` refers to the pod itself, not the host machine. You must use the Kubernetes service DNS name.

2. **Not creating the `tracing` namespace**: The Jaeger Helm chart creates resources in a specific namespace. If you install it in `default` but reference `tracing.svc.cluster.local`, the services will not be found.

3. **Missing `insecure=True` for gRPC**: The Jaeger collector in a dev cluster typically does not have TLS configured. Without `insecure=True`, the gRPC client will fail to connect because it expects TLS.

4. **Exemplar labels not matching Prometheus format**: The exemplar dictionary must use string keys and values. Using integer trace IDs or missing the `trace_id` key means Grafana cannot create the link.

5. **imagePullPolicy issues**: When building images locally with minikube's Docker daemon, set `imagePullPolicy: IfNotPresent`. Otherwise, Kubernetes tries to pull from Docker Hub and fails.

6. **Prometheus not scraping the application**: The ServiceMonitor selector must match the Service labels. If the Service does not have a label matching `app: order-service`, Prometheus will not discover it.

7. **Forgetting to open the firewall/port-forward**: In minikube, `minikube service <name>` opens a tunnel. In kind, you must use `kubectl port-forward`. Without this, you cannot reach the Jaeger UI from your browser.
