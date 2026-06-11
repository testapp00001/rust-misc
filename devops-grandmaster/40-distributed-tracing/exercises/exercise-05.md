# Exercise 05: Set Up Jaeger with Kubernetes and Correlate with Metrics

## Type: Integration

## Objective

Deploy Jaeger on a Kubernetes cluster, configure a sample application to send traces, set up trace-metric correlation using exemplars, and build a monitoring dashboard that links metrics to traces.

## Prerequisites

- A running Kubernetes cluster (minikube, kind, or cloud-managed)
- `kubectl` configured
- Helm 3 installed
- Docker installed
- Completion of Exercises 01-04

## Part A: Deploy Jaeger on Kubernetes

### Task 1: Install Jaeger Operator (or use Helm)

Using Helm:

```bash
helm repo add jaegertracing https://jaegertracing.github.io/helm-charts
helm repo update

kubectl create namespace tracing

helm install jaeger jaegertracing/jaeger \
  --namespace tracing \
  --set storage.type=memory \
  --set query.serviceType=NodePort
```

### Task 2: Verify Deployment

1. Check that all Jaeger pods are running:
   ```bash
   kubectl get pods -n tracing
   ```

2. Get the Jaeger UI URL:
   ```bash
   # For minikube
   minikube service jaeger-query -n tracing --url

   # For kind or other clusters
   kubectl port-forward -n tracing svc/jaeger-query 16686:16686
   ```

3. Open the Jaeger UI and verify it loads.

## Part B: Deploy a Sample Application

### Task 3: Create a Traced Microservice

Create a Kubernetes deployment for a Python app that generates traces. Write the following files:

**Dockerfile:**

```dockerfile
FROM python:3.11-slim

WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY app.py .

CMD ["python", "app.py"]
```

**requirements.txt:**

```
flask
requests
opentelemetry-api
opentelemetry-sdk
opentelemetry-exporter-otlp
opentelemetry-instrumentation-flask
opentelemetry-instrumentation-requests
```

**app.py:**

Write a Flask application that:

1. Reads the Jaeger collector endpoint from an environment variable `OTEL_EXPORTER_OTLP_ENDPOINT`.
2. Instruments itself with OpenTelemetry using the OTLP exporter (gRPC).
3. Has three endpoints:
   - `GET /health` — returns 200 OK
   - `GET /api/order` — simulates placing an order (creates spans for validation, processing, and database operations)
   - `GET /api/order/status` — simulates checking order status

**k8s-deployment.yaml:**

Write a Kubernetes manifest that:

1. Creates a Deployment with 2 replicas
2. Sets the `OTEL_EXPORTER_OTLP_ENDPOINT` environment variable to point to the Jaeger collector service (e.g., `jaeger-collector.tracing.svc.cluster.local:4317`)
3. Sets `OTEL_SERVICE_NAME` to `"order-service"`
4. Exposes port 5000
5. Creates a Service of type ClusterIP
6. Includes resource requests and limits
7. Includes liveness and readiness probes

### Task 4: Build and Deploy

1. Build the Docker image:
   ```bash
   docker build -t order-service:1.0 .
   # For kind: kind load docker-image order-service:1.0
   # For minikube: eval $(minikube docker-env) && docker build -t order-service:1.0 .
   ```

2. Apply the manifests:
   ```bash
   kubectl apply -f k8s-deployment.yaml
   ```

3. Verify pods are running and generate traffic:
   ```bash
   kubectl port-forward svc/order-service 5000:5000
   for i in $(seq 1 50); do curl http://localhost:5000/api/order; done
   ```

4. Check the Jaeger UI for traces from `order-service`.

## Part C: Correlate Traces with Metrics

### Task 5: Deploy Prometheus

Install Prometheus using Helm:

```bash
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm install prometheus prometheus-community/prometheus \
  --namespace monitoring \
  --create-namespace
```

### Task 6: Add Metrics to the Application

Modify your `app.py` to expose a `/metrics` endpoint using `prometheus_client`:

1. Add a Histogram metric `http_request_duration_seconds` with labels for method, endpoint, and status.
2. Record each request's duration in this histogram.
3. Add a Counter metric `http_requests_total` with the same labels.
4. Enable exemplars on the histogram — each bucket should include the trace ID and span ID from the current OpenTelemetry context.

Example of recording an exemplar:

```python
from prometheus_client import Histogram, generate_latest
from opentelemetry import trace

REQUEST_DURATION = Histogram(
    "http_request_duration_seconds",
    "Request duration in seconds",
    ["method", "endpoint", "status"],
)

@app.route("/api/order")
def order():
    start = time.time()
    # ... do work ...
    duration = time.time() - start
    span = trace.get_current_span()
    ctx = span.get_span_context()
    trace_id = format(ctx.trace_id, "032x")
    REQUEST_DURATION.labels(
        method="GET", endpoint="/api/order", status="200"
    ).observe(duration, exemplar={"trace_id": trace_id})
```

### Task 7: Configure Prometheus to Scrape the Application

Create a `ServiceMonitor` (if using the Prometheus Operator) or add a scrape config to Prometheus:

**service-monitor.yaml:**

```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: order-service-monitor
  namespace: monitoring
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

### Task 8: Explore Trace-Metric Correlation

1. Open Prometheus UI and query `http_request_duration_seconds`.
2. In Grafana (if installed) or Prometheus, look for exemplar links on metric data points.
3. Click an exemplar to jump to the corresponding trace in Jaeger.
4. Verify that the trace ID in the metric exemplar matches a trace in Jaeger.

## Part D: Analysis

Answer the following:

1. How does the Jaeger collector endpoint differ in Kubernetes vs running locally? What service discovery mechanism is used?
2. What is the difference between `OTEL_EXPORTER_OTLP_ENDPOINT` pointing to the collector vs the agent? When would you use each?
3. How do exemplars bridge the gap between metrics and traces? What problems does this solve?
4. If you see a spike in `http_request_duration_seconds` at 3:00 PM, how would you use exemplars to find the slow traces?

## Success Criteria

- [ ] Jaeger is deployed and accessible in the Kubernetes cluster
- [ ] The order-service application is running with 2 replicas
- [ ] Traces from order-service appear in the Jaeger UI
- [ ] The application exposes Prometheus metrics with exemplars
- [ ] Prometheus is scraping the application's /metrics endpoint
- [ ] You can click an exemplar in Prometheus/Grafana to view the corresponding trace in Jaeger
- [ ] You can explain the collector vs agent deployment pattern

## Hints

<details>
<summary>Hint 1: Jaeger collector endpoint in Kubernetes</summary>

The Jaeger Helm chart creates several services. The OTLP gRPC endpoint is typically:

```
jaeger-collector.tracing.svc.cluster.local:4317
```

The OTLP HTTP endpoint is:

```
jaeger-collector.tracing.svc.cluster.local:4318
```

Verify with:

```bash
kubectl get svc -n tracing
```

</details>

<details>
<summary>Hint 2: OTLP exporter configuration</summary>

```python
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter

exporter = OTLPSpanExporter(
    endpoint=os.environ.get("OTEL_EXPORTER_OTLP_ENDPOINT", "localhost:4317"),
    insecure=True,
)
```

Set `insecure=True` because the Jaeger collector in a dev cluster typically does not use TLS.

</details>

<details>
<summary>Hint 3: Exemplar structure</summary>

Exemplars are data points attached to metric buckets that contain trace context:

```python
exemplar = {
    "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
    "span_id": "00f067aa0ba902b7",
}
```

The `prometheus_client` library supports exemplars natively on Histogram and Counter metrics since version 0.14+. The observe() method accepts an `exemplar` keyword argument.

</details>

<details>
<summary>Hint 4: Kubernetes health probes</summary>

```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 5000
  initialDelaySeconds: 10
  periodSeconds: 15
readinessProbe:
  httpGet:
    path: /health
    port: 5000
  initialDelaySeconds: 5
  periodSeconds: 10
```

The readiness probe ensures traffic is not sent to a pod until it can accept requests. The liveness probe restarts the pod if it becomes unresponsive.

</details>
