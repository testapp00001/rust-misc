# Solution 03: Trace a Request Across Multiple Microservices

## Complete Code

### tracing.py (shared helper)

```python
from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor, ConsoleSpanExporter
from opentelemetry.sdk.resources import Resource, SERVICE_NAME
from opentelemetry.exporter.jaeger.thrift import JaegerExporter
from opentelemetry.instrumentation.flask import FlaskInstrumentor
from opentelemetry.instrumentation.requests import RequestsInstrumentor


def init_tracing(service_name, app=None):
    """Configure OpenTelemetry tracing for a service."""

    resource = Resource.create({SERVICE_NAME: service_name})
    provider = TracerProvider(resource=resource)

    jaeger_exporter = JaegerExporter(
        agent_host_name="localhost",
        agent_port=6831,
    )
    provider.add_span_processor(BatchSpanProcessor(jaeger_exporter))

    # Optional: also print spans to console for debugging
    # provider.add_span_processor(BatchSpanProcessor(ConsoleSpanExporter()))

    trace.set_tracer_provider(provider)

    # Instrument requests library for outgoing HTTP calls (context propagation)
    RequestsInstrumentor().instrument()

    # Instrument Flask app if provided
    if app:
        FlaskInstrumentor().instrument_app(app)

    return provider
```

### service_db.py (port 5003)

```python
from flask import Flask, jsonify
import time
from opentelemetry import trace
from tracing import init_tracing

app = Flask(__name__)
init_tracing("database-service", app)

tracer = trace.get_tracer(__name__)

# Simulated database
PRODUCTS = [
    {"id": 1, "name": "Laptop", "price": 999.99},
    {"id": 2, "name": "Mouse", "price": 29.99},
    {"id": 3, "name": "Keyboard", "price": 79.99},
]

CARTS = {
    "user123": [{"product_id": 1, "quantity": 1}, {"product_id": 2, "quantity": 2}],
    "user456": [{"product_id": 3, "quantity": 1}],
}


@app.route("/products")
def get_products():
    with tracer.start_as_current_span("db-query") as span:
        span.set_attribute("db.system", "postgresql")
        span.set_attribute("db.operation", "SELECT")
        span.set_attribute("db.statement", "SELECT * FROM products")
        time.sleep(0.05)  # Simulate 50ms DB query
        return jsonify(PRODUCTS)


@app.route("/cart/<user_id>")
def get_cart(user_id):
    with tracer.start_as_current_span("db-query") as span:
        span.set_attribute("db.system", "postgresql")
        span.set_attribute("db.operation", "SELECT")
        span.set_attribute("db.statement", f"SELECT * FROM cart_items WHERE user_id = '{user_id}'")
        time.sleep(0.03)  # Simulate 30ms DB query
        cart = CARTS.get(user_id, [])
        span.set_attribute("db.rows_returned", len(cart))
        return jsonify(cart)


if __name__ == "__main__":
    app.run(port=5003)
```

### service_products.py (port 5002)

```python
from flask import Flask, jsonify
import requests as http_requests
from opentelemetry import trace
from tracing import init_tracing

app = Flask(__name__)
init_tracing("product-service", app)

tracer = trace.get_tracer(__name__)


@app.route("/products")
def get_products():
    with tracer.start_as_current_span("process-products") as span:
        # Context is automatically propagated via the instrumented requests library
        response = http_requests.get("http://localhost:5003/products")
        products = response.json()

        span.set_attribute("product.count", len(products))
        return jsonify(products)


if __name__ == "__main__":
    app.run(port=5002)
```

### service_cart.py (port 5004)

```python
from flask import Flask, jsonify
import requests as http_requests
from opentelemetry import trace
from tracing import init_tracing

app = Flask(__name__)
init_tracing("cart-service", app)

tracer = trace.get_tracer(__name__)


@app.route("/cart/<user_id>")
def get_cart(user_id):
    with tracer.start_as_current_span("process-cart") as span:
        response = http_requests.get(f"http://localhost:5003/cart/{user_id}")
        cart = response.json()

        span.set_attribute("cart.user_id", user_id)
        span.set_attribute("cart.item_count", len(cart))
        return jsonify(cart)


if __name__ == "__main__":
    app.run(port=5004)
```

### service_frontend.py (port 5001)

```python
from flask import Flask, jsonify
import requests as http_requests
from concurrent.futures import ThreadPoolExecutor
from opentelemetry import trace
from tracing import init_tracing

app = Flask(__name__)
init_tracing("frontend-service", app)

tracer = trace.get_tracer(__name__)


@app.route("/dashboard/<user_id>")
def dashboard(user_id):
    with tracer.start_as_current_span("build-dashboard") as span:
        span.set_attribute("user.id", user_id)

        # Call Product Service and Cart Service in parallel
        with ThreadPoolExecutor(max_workers=2) as executor:
            products_future = executor.submit(
                http_requests.get, "http://localhost:5002/products"
            )
            cart_future = executor.submit(
                http_requests.get, f"http://localhost:5004/cart/{user_id}"
            )

            products = products_future.result().json()
            cart = cart_future.result().json()

        span.set_attribute("dashboard.products_count", len(products))
        span.set_attribute("dashboard.cart_items_count", len(cart))

        return jsonify({
            "user_id": user_id,
            "products": products,
            "cart": cart,
        })


if __name__ == "__main__":
    app.run(port=5001)
```

## How Context Propagation Works

The trace context flows through the system like this:

```
1. Client sends: GET /dashboard/user123
   (no trace headers)

2. Frontend Service (FlaskInstrumentor):
   - Extracts trace context from headers (none found)
   - Creates root span (traceId=abc, spanId=001)
   - Sets root span as current context

3. Frontend Service (RequestsInstrumentor + ThreadPoolExecutor):
   - requests.get("http://localhost:5002/products")
   - RequestsInstrumentor reads current context (traceId=abc, spanId=001)
   - Injects header: traceparent: 00-abc-001-01
   - Sends HTTP request

4. Product Service (FlaskInstrumentor):
   - Extracts header: traceparent: 00-abc-001-01
   - Creates child span (traceId=abc, spanId=002, parentSpanId=001)
   - Sets as current context

5. Product Service calls Database Service:
   - RequestsInstrumentor injects: traceparent: 00-abc-002-01

6. Database Service (FlaskInstrumentor):
   - Extracts header: traceparent: 00-abc-002-01
   - Creates child span (traceId=abc, spanId=003, parentSpanId=002)
```

The same flow happens for the Cart Service path. Both paths share the same `traceId=abc` because they originated from the same root span.

## Why It Works

### RequestsInstrumentor

This is the key to automatic context propagation. When you call `http_requests.get(url)`, the instrumentor:

1. Reads the current span from the execution context.
2. Injects `traceparent` and `tracestate` headers into the outgoing HTTP request.
3. Creates a child span for the outgoing HTTP call.

The receiving service's `FlaskInstrumentor` extracts these headers and continues the trace.

### ThreadPoolExecutor and Context

Python's `contextvars` (used by OpenTelemetry for context propagation) is inherited by threads spawned by `ThreadPoolExecutor`. This means the child threads in the executor can access the same current context as the parent thread. The `RequestsInstrumentor` correctly picks up the context in each thread.

This would NOT work with `multiprocessing.Process` because processes do not share memory.

### Parallel vs Sequential Calls

Using `ThreadPoolExecutor` with 2 workers means the Product Service and Cart Service calls happen concurrently:

```
Sequential (without ThreadPoolExecutor):
[build-dashboard] ───────────────────────────────── 110ms
  [process-products] ───── 80ms
    [db-query] ─ 50ms
  [process-cart] ──────── 60ms
    [db-query] ─ 30ms

Parallel (with ThreadPoolExecutor):
[build-dashboard] ──────────────────── 80ms
  [process-products] ───── 80ms
    [db-query] ─ 50ms
  [process-cart] ──────── 60ms
    [db-query] ─ 30ms
```

In the parallel case, the total time is max(80, 60) = 80ms instead of 80 + 60 = 140ms.

## Answers to Questions

**1. How many spans are in the complete trace?**

8 spans:
1. `GET /dashboard/user123` (Frontend Service, auto-instrumented)
2. `build-dashboard` (Frontend Service, manual)
3. `GET /products` (Frontend -> Product Service, auto-instrumented outgoing)
4. `process-products` (Product Service, manual)
5. `GET /products` (Product -> Database Service, auto-instrumented outgoing)
6. `db-query` (Database Service, manual)
7. `GET /cart/user123` (Frontend -> Cart Service, auto-instrumented outgoing)
8. `process-cart` (Cart Service, manual)
9. `GET /cart/user123` (Cart -> Database Service, auto-instrumented outgoing)
10. `db-query` (Database Service, manual)

(Note: the exact count depends on which instrumentors create spans. The Flask instrumentor creates one span per incoming request, and the Requests instrumentor creates one span per outgoing request.)

**2. Do all spans share the same traceId?**

Yes. All spans share `traceId=abc` because the context was propagated through HTTP headers at each service boundary. This is the fundamental requirement for distributed tracing — without a shared trace ID, you cannot correlate spans across services.

**3. Waterfall view:**

In Jaeger, the waterfall shows:
- The root `GET /dashboard/user123` span spanning the full duration.
- `build-dashboard` nested inside it.
- `process-products` and `process-cart` running in parallel (overlapping time ranges).
- Each `db-query` nested inside its parent service span.

**4. If Database Service takes 200ms:**

Both `process-products` and `process-cart` are affected because they both call the Database Service. The Product Service call goes from ~80ms to ~230ms. The Cart Service call goes from ~60ms to ~230ms. Since they run in parallel, the total request time goes from ~80ms to ~230ms. If they were sequential, it would go from ~140ms to ~460ms. This demonstrates why parallel downstream calls are important.

## Common Mistakes

1. **Not instrumenting the requests library**: Without `RequestsInstrumentor().instrument()`, outgoing HTTP calls do not inject trace headers. The receiving service creates a new root span, breaking the trace chain.

2. **Instrumenting requests before setting TracerProvider**: The instrumentor needs the global TracerProvider to be registered. Order matters: `trace.set_tracer_provider(provider)` must come before `RequestsInstrumentor().instrument()`.

3. **Using multiprocessing instead of threading**: `multiprocessing.Process` does not share `contextvars`, so trace context is lost. Use `ThreadPoolExecutor` or `asyncio` for concurrent operations.

4. **Creating a new TracerProvider in each service call**: Each service is a separate process, so each needs its own TracerProvider. But do not create a new one per request — create it once at startup.

5. **Forgetting to instrument leaf services**: Even the Database Service needs a TracerProvider and FlaskInstrumentor. Without it, the `db-query` spans would not be exported to Jaeger.

6. **Running services on wrong ports**: Each service must listen on its expected port. If the Product Service runs on 5003 instead of 5002, the Frontend's request to `localhost:5002` fails.
