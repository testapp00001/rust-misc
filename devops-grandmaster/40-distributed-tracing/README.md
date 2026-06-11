# Module 40: Distributed Tracing — Jaeger, OpenTelemetry, Trace Context Propagation

> **Previous Module:** [39 — Metrics Collection](../39-metrics-collection/README.md)
> **Next Module:** [41 — Centralized Logging](../41-centralized-logging/README.md)
> **Phase:** 5 — Observability

---

## 1. The Problem

Your system is a constellation of microservices. A user request enters the API gateway, hits the user service, which calls the order service, which queries the database and calls the payment service, which calls an external Stripe API. The request takes 4 seconds.

Your metrics dashboard shows that p99 latency spiked. But which service is the bottleneck? The metrics for each individual service look fine — they all have low average latency. The problem is somewhere in the interactions between services, but you cannot see the full path of a single request.

Metrics tell you **that** something is slow. Logs tell you what happened in one service. But nothing tells you the **journey** of a request across all services — until you have distributed tracing.

Distributed tracing captures the lifecycle of a request as it traverses multiple services, showing you exactly where time is spent, which services are involved, and where errors occur.

---

## 2. The Naive Way

### Anti-Pattern: Manual Request ID Tracking

```python
# Service A
@app.route('/api/orders')
def create_order():
    request_id = str(uuid.uuid4())
    logger.info(f"[{request_id}] Creating order")

    # Call Service B
    response = requests.get(f"http://service-b/api/users/{user_id}",
        headers={"X-Request-ID": request_id})
    logger.info(f"[{request_id}] Got user info")

    # Call Service C
    response = requests.post(f"http://service-c/api/payments",
        headers={"X-Request-ID": request_id},
        json={"order_id": order_id})
    logger.info(f"[{request_id}] Payment processed")

    return jsonify({"order_id": order_id})
```

Problems:
- Every service must manually propagate the request ID
- You only get a list of log lines, not a visual trace
- No timing information per step
- No visibility into parallel calls
- Breaks when a service forgets to pass the header
- No standard format — every team implements it differently

### Anti-Pattern: Correlating Logs Manually

```bash
# Finding the trace of a slow request
grep "abc-123-def" service-a.log service-b.log service-c.log | sort
# Output: a wall of unsorted text across different log formats
```

This is tedious, error-prone, and impossible to do at scale. With thousands of requests per second, manual log correlation does not work.

### Anti-Pattern: Tracing One Service Only

Using Spring Cloud Sleuth or similar per-framework tracing without propagating context to other services. You get a trace that shows spans within one service but loses visibility as soon as the request crosses a service boundary.

---

## 3. The Right Way

### Core Concepts

**Trace** — the complete journey of a request through the system. A trace has a unique trace ID.

**Span** — a single unit of work within a trace. Each span represents an operation (HTTP call, database query, function call). A span has:
- A unique span ID
- A parent span ID (except the root span)
- Start time and duration
- Service name, operation name
- Tags (key-value metadata)
- Logs (timestamped events within the span)

**Trace Context** — the metadata (trace ID, span ID, flags) that is propagated across service boundaries via HTTP headers.

```
Trace ID: abc123
============================================
|  Service A: POST /api/orders (1200ms)    |
|  ┌─────────────────────────────────────┐ |
|  | Span A1: authenticate (5ms)         | |
|  | ┌─────────────────────────────────┐ | |
|  | | Service B: GET /api/users (200ms)| | |
|  | | Span B1: db query (180ms)       | | |
|  | └─────────────────────────────────┘ | |
|  | ┌─────────────────────────────────┐ | |
|  | | Service C: POST /payments (900ms)| | |
|  | | Span C1: stripe charge (850ms)  | | |
|  | └─────────────────────────────────┘ | |
|  | Span A2: send confirmation (10ms)   | |
|  └─────────────────────────────────────┘ |
============================================
```

The trace above reveals that the Stripe charge (850ms) is the bottleneck, not any of your services. This is the insight tracing provides.

### W3C Trace Context (Standard Headers)

The W3C Trace Context standard defines two headers:

**traceparent** — required:
```
traceparent: {version}-{trace-id}-{parent-id}-{trace-flags}
# Example:
traceparent: 00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01
#  version:   00
#  trace-id:  0af7651916cd43dd8448eb211c80319c (32 hex chars)
#  span-id:   b7ad6b7169203331 (16 hex chars)
#  flags:     01 (sampled)
```

**tracestate** — optional vendor-specific data:
```
tracestate: congo=t61rcWkgMzE,rojo=00f067aa0ba902b7
```

Older formats (B3 headers from Zipkin, Jaeger headers) are still in use:

```
# B3 format (Zipkin)
X-B3-TraceId: 0af7651916cd43dd8448eb211c80319c
X-B3-SpanId: b7ad6b7169203331
X-B3-ParentSpanId: 0020000000000001
X-B3-Sampled: 1

# Jaeger format
uber-trace-id: 0af7651916cd43dd8448eb211c80319c:b7ad6b7169203331:0:01
```

### OpenTelemetry

OpenTelemetry (OTel) is the CNCF standard for observability instrumentation. It provides:

- A vendor-neutral API and SDK for traces, metrics, and logs
- An OpenTelemetry Collector that receives, processes, and exports telemetry data
- Auto-instrumentation libraries for popular languages and frameworks
- The OTLP (OpenTelemetry Protocol) for data transport

```
Application --> OTel SDK --> OTel Collector --> Backend (Jaeger, Zipkin, etc.)
                         --> Backend (Prometheus)
                         --> Backend (Loki)
```

**Why OpenTelemetry matters:**

- One standard replaces Jaeger client, Zipkin client, Datadog client, etc.
- Auto-instrumentation reduces code changes
- Collector provides a single pipeline for all telemetry
- Vendor-neutral — switch backends without changing application code

### OpenTelemetry Architecture

```
+-----------------+     +------------------+     +-----------------+
|   Application   |     |  OTel Collector  |     |    Backend      |
|                 |     |                  |     |                 |
|  OTel SDK       |     |  Receivers:      |     |  Jaeger         |
|  ├── Tracer     |---->|  ├── OTLP        |---->|  Prometheus     |
|  ├── Meter      |     |  ├── Jaeger      |     |  Loki          |
|  └── Logger     |     |  └── Zipkin      |     |  Elasticsearch  |
|                 |     |                  |     |                 |
|  Auto-instrum.  |     |  Processors:     |     |                 |
|  ├── HTTP       |     |  ├── Batch       |     |                 |
|  ├── gRPC       |     |  ├── Filter      |     |                 |
|  ├── DB         |     |  └── Attributes  |     |                 |
|  └── Messaging  |     |                  |     |                 |
|                 |     |  Exporters:       |     |                 |
|                 |     |  ├── OTLP        |     |                 |
|                 |     |  ├── Prometheus  |     |                 |
|                 |     |  └── Loki        |     |                 |
+-----------------+     +------------------+     +-----------------+
```

### Sampling Strategies

At high traffic volumes, tracing every request is expensive. Sampling controls what percentage of requests get traced.

**Head-based sampling** — the decision is made at the start of the trace:

```yaml
# Sample 10% of all traces
processors:
  probabilistic_sampler:
    sampling_percentage: 10
```

**Tail-based sampling** — the decision is made after the trace completes:

```yaml
# Keep all traces with errors, plus 5% of normal traces
processors:
  tail_sampling:
    decision_wait: 10s
    policies:
      - name: errors
        type: status_code
        status_code:
          status_codes: [ERROR]
      - name: slow-requests
        type: latency
        latency:
          threshold_ms: 1000
      - name: sample-rest
        type: probabilistic
        probabilistic:
          sampling_percentage: 5
```

**Rate limiting sampling** — keep a fixed number per second:

```yaml
processors:
  rate_limiting:
    spans_per_second: 100
```

### Baggage Propagation

Baggage is key-value metadata that propagates across all services in a trace, beyond just the trace context:

```python
from opentelemetry import baggage, context

# Set baggage in Service A
ctx = baggage.set_baggage("user.id", "12345", context.get_current())
ctx = baggage.set_baggage("tenant", "acme-corp", context=ctx)

# Read baggage in Service C (after passing through Service B)
user_id = baggage.get_baggage("user.id")  # "12345"
tenant = baggage.get_baggage("tenant")     # "acme-corp"
```

Use cases:
- Propagating user ID for user-specific sampling decisions
- Propagating tenant ID in multi-tenant systems
- Propagating feature flags across services
- Priority tagging for critical requests

---

## 4. The Production Way

### Auto-Instrumentation with OpenTelemetry

Auto-instrumentation captures traces without changing application code.

**Python (auto-instrumentation):**

```bash
# Install
pip install opentelemetry-distro opentelemetry-exporter-otlp
opentelemetry-bootstrap -a install
```

```python
# app.py — no tracing code needed!
from flask import Flask, jsonify
import requests

app = Flask(__name__)

@app.route('/api/orders')
def create_order():
    # This HTTP call is automatically traced
    user = requests.get("http://user-service:8081/api/users/1").json()

    # This HTTP call is automatically traced
    payment = requests.post("http://payment-service:8082/api/charge",
        json={"amount": 99.99}).json()

    return jsonify({"order_id": 1, "user": user, "payment": payment})
```

```bash
# Run with auto-instrumentation
opentelemetry-instrument \
  --service_name order-service \
  --exporter_otlp_endpoint http://otel-collector:4317 \
  python app.py
```

**Go (auto-instrumentation):**

```go
package main

import (
    "net/http"

    "go.opentelemetry.io/contrib/instrumentation/net/http/otelhttp"
)

func main() {
    // Wrap the default HTTP transport for outbound calls
    client := &http.Client{
        Transport: otelhttp.NewTransport(http.DefaultTransport),
    }

    // Wrap the handler for inbound calls
    handler := otelhttp.NewHandler(
        http.HandlerFunc(handleRequest),
        "api/orders",
    )

    http.Handle("/api/orders", handler)
    http.ListenAndServe(":8080", nil)
}
```

**Rust (manual instrumentation):**

```rust
use opentelemetry::{
    global,
    sdk::{trace, Resource},
    trace::{Span, SpanKind, Tracer, TraceContextExt},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;

fn init_tracer() -> trace::Tracer {
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint("http://otel-collector:4317");

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            trace::config()
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", "order-service"),
                ]))
        )
        .install_batch(opentelemetry::runtime::Tokio)
        .unwrap()
}

async fn handle_request(req: Request) -> Response {
    let tracer = global::tracer("order-service");
    let mut span = tracer
        .span_builder("create_order")
        .with_kind(SpanKind::Server)
        .start(&tracer);

    // Add attributes
    span.set_attribute(KeyValue::new("order.id", "12345"));

    // Create child span for database call
    let db_span = tracer
        .span_builder("db_query")
        .with_kind(SpanKind::Client)
        .start(&tracer);
    let result = query_database().await;
    db_span.end();

    span.end();
    Response::new(result)
}
```

### OpenTelemetry Collector Configuration

```yaml
# otel-collector-config.yml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    timeout: 5s
    send_batch_size: 1024

  # Add resource attributes
  resource:
    attributes:
      - key: environment
        value: production
        action: upsert

  # Filter out health check spans
  filter:
    error_mode: ignore
    traces:
      exclude:
        match_type: strict
        span_names:
          - "GET /health"
          - "GET /ready"

  # Tail-based sampling
  tail_sampling:
    decision_wait: 10s
    num_traces: 100000
    policies:
      - name: errors-policy
        type: status_code
        status_code:
          status_codes: [ERROR]
      - name: slow-traces
        type: latency
        latency:
          threshold_ms: 2000
      - name: probabilistic-policy
        type: probabilistic
        probabilistic:
          sampling_percentage: 10

exporters:
  otlp/jaeger:
    endpoint: jaeger:4317
    tls:
      insecure: true

  prometheus:
    endpoint: 0.0.0.0:8889

  logging:
    loglevel: info

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [filter, tail_sampling, batch]
      exporters: [otlp/jaeger, logging]
    metrics:
      receivers: [otlp]
      processors: [batch]
      exporters: [prometheus]
```

### Jaeger Deployment

```yaml
# docker-compose.yml for Jaeger
version: '3.8'
services:
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"  # Jaeger UI
      - "4317:4317"    # OTLP gRPC
      - "4318:4318"    # OTLP HTTP
      - "14250:14250"  # gRPC for jaeger-agent
    environment:
      - COLLECTOR_OTLP_ENABLED=true
      - SPAN_STORAGE_TYPE=badger
      - BADGER_EPHEMERAL=false
      - BADGER_DIRECTORY_VALUE=/badger/data
      - BADGER_DIRECTORY_KEY=/badger/key
    volumes:
      - jaeger-data:/badger

  # For production, use Elasticsearch backend:
  # jaeger-query:
  #   image: jaegertracing/jaeger-query:latest
  #   environment:
  #     - SPAN_STORAGE_TYPE=elasticsearch
  #     - ES_SERVER_URLS=http://elasticsearch:9200
  # elasticsearch:
  #   image: docker.elastic.co/elasticsearch/elasticsearch:8.10.0
  #   environment:
  #     - discovery.type=single-node

volumes:
  jaeger-data:
```

### Kubernetes Deployment with Helm

```bash
# Add Jaeger Helm repo
helm repo add jaegertracing https://jaegertracing.github.io/helm-charts

# Install Jaeger
helm install jaeger jaegertracing/jaeger \
  --namespace observability \
  --create-namespace \
  --set storage.type=elasticsearch \
  --set storage.options.es.server-urls=http://elasticsearch:9200 \
  --set collector.otlp.enabled=true

# Install OpenTelemetry Collector
helm repo add open-telemetry https://open-telemetry.github.io/opentelemetry-helm-charts

helm install otel-collector open-telemetry/opentelemetry-collector \
  --namespace observability \
  --set mode=deployment \
  --set config.exporters.otlp.endpoint=jaeger-collector:4317
```

### Instrumentation Best Practices

**Span naming conventions:**

```python
# GOOD: Parameterized names (bounded cardinality)
tracer.start_span(f"GET /api/users/{user_id}")  # BAD: high cardinality
tracer.start_span("GET /api/users/{id}")          # GOOD: bounded

# GOOD: Named operations
tracer.start_span("db.query.users.find_by_id")
tracer.start_span("cache.get.session")
tracer.start_span("http.client.stripe.charge")
```

**Adding useful attributes:**

```python
from opentelemetry import trace

tracer = trace.get_tracer(__name__)

with tracer.start_as_current_span("process_order") as span:
    # Add business context
    span.set_attribute("order.id", order_id)
    span.set_attribute("order.total", 99.99)
    span.set_attribute("order.item_count", 3)
    span.set_attribute("customer.tier", "premium")

    # Add error information
    try:
        process_payment(order)
    except PaymentError as e:
        span.set_status(trace.StatusCode.ERROR, str(e))
        span.record_exception(e)
        raise

    # Add events (timestamped logs within the span)
    span.add_event("payment_initiated", {
        "payment.method": "credit_card",
        "payment.amount": 99.99,
    })
    span.add_event("payment_completed", {
        "payment.transaction_id": "txn_abc123",
    })
```

### Correlating Traces with Logs

Inject trace context into log records so you can jump from a log line to its trace.

**Python with structlog:**

```python
import structlog
from opentelemetry import trace

def add_trace_context(logger, method_name, event_dict):
    span = trace.get_current_span()
    ctx = span.get_span_context()
    if ctx.is_valid:
        event_dict["trace_id"] = format(ctx.trace_id, '032x')
        event_dict["span_id"] = format(ctx.span_id, '016x')
    return event_dict

structlog.configure(
    processors=[
        add_trace_context,
        structlog.processors.JSONRenderer(),
    ]
)

# Log output includes trace context:
# {"event": "order_created", "trace_id": "0af76519...", "span_id": "b7ad6b71...", "order_id": "12345"}
```

**Go with zap:**

```go
import (
    "go.opentelemetry.io/otel/trace"
    "go.uber.org/zap"
)

func TraceFields(ctx context.Context) zap.Field {
    spanCtx := trace.SpanContextFromContext(ctx)
    if spanCtx.IsValid() {
        return zap.Fields(
            zap.String("trace_id", spanCtx.TraceID().String()),
            zap.String("span_id", spanCtx.SpanID().String()),
        )
    }
    return zap.Fields()
}
```

---

## 5. Hands-On Lab

### Lab: Instrument a Microservices App with OpenTelemetry

**Objective:** Trace a request across three services, view traces in Jaeger, and correlate traces with logs.

**Step 1: Create the project**

```bash
mkdir tracing-lab && cd tracing-lab
mkdir -p services/order-service services/user-service services/payment-service
```

**Step 2: Create the Order Service (Python/Flask)**

```python
# services/order-service/app.py
from flask import Flask, jsonify
from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.sdk.resources import Resource
from opentelemetry.instrumentation.flask import FlaskInstrumentor
from opentelemetry.instrumentation.requests import RequestsInstrumentor
import requests
import logging
import time

# Configure tracing
resource = Resource.create({"service.name": "order-service"})
provider = TracerProvider(resource=resource)
processor = BatchSpanProcessor(OTLPSpanExporter(endpoint="otel-collector:4317"))
provider.add_span_processor(processor)
trace.set_tracer_provider(provider)
tracer = trace.get_tracer(__name__)

app = Flask(__name__)
FlaskInstrumentor().instrument_app(app)
RequestsInstrumentor().instrument()

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@app.route('/api/orders', methods=['POST'])
def create_order():
    with tracer.start_as_current_span("create_order") as span:
        span.set_attribute("order.id", "order-123")
        logger.info("Creating order", extra={
            "trace_id": format(trace.get_current_span().get_span_context().trace_id, '032x'),
            "order_id": "order-123"
        })

        # Call user service
        with tracer.start_as_current_span("fetch_user"):
            user = requests.get("http://user-service:8081/api/users/1").json()
            span.set_attribute("user.id", user.get("id", "unknown"))

        # Call payment service
        with tracer.start_as_current_span("process_payment"):
            payment = requests.post("http://payment-service:8082/api/charge",
                json={"amount": 99.99, "currency": "USD"}).json()

        return jsonify({
            "order_id": "order-123",
            "user": user,
            "payment": payment,
            "status": "created"
        })

@app.route('/health')
def health():
    return jsonify({"status": "healthy"})

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=8080)
```

**Step 3: Create the User Service**

```python
# services/user-service/app.py
from flask import Flask, jsonify
from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.sdk.resources import Resource
from opentelemetry.instrumentation.flask import FlaskInstrumentor
import time
import random

resource = Resource.create({"service.name": "user-service"})
provider = TracerProvider(resource=resource)
processor = BatchSpanProcessor(OTLPSpanExporter(endpoint="otel-collector:4317"))
provider.add_span_processor(processor)
trace.set_tracer_provider(provider)
tracer = trace.get_tracer(__name__)

app = Flask(__name__)
FlaskInstrumentor().instrument_app(app)

@app.route('/api/users/<user_id>')
def get_user(user_id):
    with tracer.start_as_current_span("db_query") as span:
        span.set_attribute("db.system", "postgresql")
        span.set_attribute("db.statement", "SELECT * FROM users WHERE id = ?")
        time.sleep(random.uniform(0.05, 0.15))  # Simulate DB query

    return jsonify({"id": user_id, "name": "Alice", "email": "alice@example.com"})

@app.route('/health')
def health():
    return jsonify({"status": "healthy"})

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=8081)
```

**Step 4: Create the Payment Service**

```python
# services/payment-service/app.py
from flask import Flask, request, jsonify
from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.sdk.resources import Resource
from opentelemetry.instrumentation.flask import FlaskInstrumentor
import time
import random

resource = Resource.create({"service.name": "payment-service"})
provider = TracerProvider(resource=resource)
processor = BatchSpanProcessor(OTLPSpanExporter(endpoint="otel-collector:4317"))
provider.add_span_processor(processor)
trace.set_tracer_provider(provider)
tracer = trace.get_tracer(__name__)

app = Flask(__name__)
FlaskInstrumentor().instrument_app(app)

@app.route('/api/charge', methods=['POST'])
def charge():
    body = request.json
    amount = body.get("amount", 0)

    with tracer.start_as_current_span("stripe_charge") as span:
        span.set_attribute("payment.provider", "stripe")
        span.set_attribute("payment.amount", amount)
        span.set_attribute("payment.currency", "USD")
        time.sleep(random.uniform(0.3, 0.8))  # Simulate external API call

        if random.random() < 0.05:  # 5% failure rate
            span.set_status(trace.StatusCode.ERROR, "Stripe declined")
            return jsonify({"error": "Payment declined"}), 402

    return jsonify({
        "transaction_id": f"txn_{random.randint(1000, 9999)}",
        "status": "success",
        "amount": amount
    })

@app.route('/health')
def health():
    return jsonify({"status": "healthy"})

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=8082)
```

**Step 5: Create OTel Collector configuration**

```yaml
# otel-collector-config.yml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317

processors:
  batch:
    timeout: 5s

exporters:
  otlp/jaeger:
    endpoint: jaeger:4317
    tls:
      insecure: true
  logging:
    loglevel: debug

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters: [otlp/jaeger, logging]
```

**Step 6: Docker Compose**

```yaml
version: '3.8'
services:
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"
      - "4317:4317"
    environment:
      - COLLECTOR_OTLP_ENABLED=true

  otel-collector:
    image: otel/opentelemetry-collector-contrib:latest
    ports:
      - "4317:4317"
    volumes:
      - ./otel-collector-config.yml:/etc/otelcol/config.yml
    depends_on:
      - jaeger

  order-service:
    build: ./services/order-service
    ports:
      - "8080:8080"
    depends_on:
      - otel-collector

  user-service:
    build: ./services/user-service
    ports:
      - "8081:8081"
    depends_on:
      - otel-collector

  payment-service:
    build: ./services/payment-service
    ports:
      - "8082:8082"
    depends_on:
      - otel-collector
```

```dockerfile
# services/*/Dockerfile
FROM python:3.11-slim
WORKDIR /app
RUN pip install flask requests opentelemetry-api opentelemetry-sdk \
    opentelemetry-exporter-otlp-proto-grpc opentelemetry-instrumentation-flask \
    opentelemetry-instrumentation-requests opentelemetry-distro
COPY app.py .
CMD ["python", "app.py"]
```

**Step 7: Generate traffic and explore traces**

```bash
# Start the stack
docker-compose up -d --build

# Wait for services to start
sleep 10

# Generate traffic
for i in $(seq 1 20); do
  curl -s -X POST http://localhost:8080/api/orders > /dev/null
  sleep 0.5
done

# Open Jaeger UI
open http://localhost:16686
```

**Step 8: Explore in Jaeger**

1. Select "order-service" from the Service dropdown
2. Click "Find Traces"
3. Click on a trace to see the waterfall view
4. Notice how the trace spans across three services
5. Click on individual spans to see attributes and timing
6. Look for the "stripe_charge" span — this is typically the slowest

**Expected trace waterfall:**

```
order-service: POST /api/orders          ████████████████████████  600ms
  order-service: fetch_user              ████                      100ms
    user-service: GET /api/users/1       ████                      100ms
      user-service: db_query             ███                        80ms
  order-service: process_payment         █████████████████          450ms
    payment-service: POST /api/charge    █████████████████          450ms
      payment-service: stripe_charge     ████████████████           420ms
```

**Step 9: Examine the failed traces**

```bash
# Generate some more traffic (some will fail)
for i in $(seq 1 50); do
  curl -s -X POST http://localhost:8080/api/orders &
  sleep 0.1
done
wait

# In Jaeger, filter by "Error: true"
# Click on an error trace — see the red span indicating the failed payment
```

---

## 6. Limitation → Next Topic

Distributed tracing tells you the journey of a single request across services. You can see where time is spent, which services are involved, and where errors occur. Traces are excellent for debugging latency and understanding service dependencies.

But traces only capture requests that were actually made. When something goes wrong, you often need to understand the **context** — what the application was doing, what values it processed, what decisions it made. That context lives in logs.

Logs provide the detailed narrative that traces and metrics cannot. When a trace shows a span took 5 seconds, logs tell you why — what query was run, what error was encountered, what retry logic kicked in.

The challenge is managing logs across dozens of services and millions of log lines per minute. You need centralized logging.

**Next Module:** [41 — Centralized Logging](../41-centralized-logging/README.md) — Aggregate logs from all services into a searchable system with ELK or Loki, correlate them with traces, and make them actionable.
