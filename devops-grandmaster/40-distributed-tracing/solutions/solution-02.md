# Solution 02: Instrument a Python App with OpenTelemetry

## Complete Code

### tracing.py

```python
from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from opentelemetry.sdk.resources import Resource, SERVICE_NAME, SERVICE_VERSION
from opentelemetry.exporter.jaeger.thrift import JaegerExporter
from opentelemetry.instrumentation.flask import FlaskInstrumentor


def init_tracing(app):
    """Configure OpenTelemetry tracing and instrument the Flask app."""

    # Define the resource with service identity
    resource = Resource.create({
        SERVICE_NAME: "my-flask-app",
        SERVICE_VERSION: "1.0.0",
    })

    # Create the TracerProvider with the resource
    provider = TracerProvider(resource=resource)

    # Create the Jaeger exporter (UDP agent mode)
    jaeger_exporter = JaegerExporter(
        agent_host_name="localhost",
        agent_port=6831,
    )

    # Add BatchSpanProcessor for efficient span export
    provider.add_span_processor(BatchSpanProcessor(jaeger_exporter))

    # Set as the global default TracerProvider
    trace.set_tracer_provider(provider)

    # Instrument the Flask app (auto-creates spans for each request)
    FlaskInstrumentor().instrument_app(app)

    return provider
```

### app.py

```python
from flask import Flask, jsonify
import time
import random

from opentelemetry import trace
from opentelemetry.trace import StatusCode
from tracing import init_tracing

app = Flask(__name__)

# Initialize tracing BEFORE any routes are defined
init_tracing(app)

# Get a tracer for creating manual spans
tracer = trace.get_tracer(__name__)


@app.route("/")
def home():
    return jsonify({"message": "Hello, traced world!"})


@app.route("/slow")
def slow_endpoint():
    # Create a child span for the simulated delay
    with tracer.start_as_current_span("simulate-delay") as span:
        delay = random.uniform(0.5, 2.0)
        span.set_attribute("delay.seconds", delay)
        time.sleep(delay)
        return jsonify({"message": f"Slept for {delay:.2f}s"})


@app.route("/error")
def error_endpoint():
    try:
        raise ValueError("Something went wrong!")
    except ValueError as e:
        span = trace.get_current_span()
        span.record_exception(e)
        span.set_status(StatusCode.ERROR, str(e))
        raise


if __name__ == "__main__":
    app.run(port=5000, debug=True)
```

## Why It Works

### Resource and Service Name

The `Resource` object identifies *what* is producing telemetry. The `service.name` attribute is the most important — it is how Jaeger groups traces by service. Without it, all traces would appear under an "unknown-service" label.

```python
resource = Resource.create({
    SERVICE_NAME: "my-flask-app",
    SERVICE_VERSION: "1.0.0",
})
```

Using `SERVICE_NAME` from `opentelemetry.sdk.resources` instead of a raw string `"service.name"` avoids typos and follows the convention.

### JaegerExporter in Agent Mode

```python
jaeger_exporter = JaegerExporter(
    agent_host_name="localhost",
    agent_port=6831,
)
```

This uses the Jaeger Agent protocol over UDP on port 6831. The agent batches and forwards spans to the Jaeger Collector. UDP is fire-and-forget — the application does not block waiting for acknowledgment, which keeps latency low. In production, the agent runs as a sidecar container alongside the application.

### BatchSpanProcessor vs SimpleSpanProcessor

`BatchSpanProcessor` collects spans in memory and exports them in batches (default: every 5 seconds or when 512 spans accumulate). This is critical for production:

- **SimpleSpanProcessor** exports each span immediately when it ends. This adds network latency to every request and can overwhelm the collector under load.
- **BatchSpanProcessor** amortizes the export cost across many spans, uses fewer network round trips, and can drop spans gracefully if the collector is unreachable.

The trade-off: if the application crashes, unexported spans in the batch buffer are lost.

### FlaskInstrumentor

```python
FlaskInstrumentor().instrument_app(app)
```

This wraps Flask's request handling to automatically:
1. Extract trace context from incoming `traceparent` headers.
2. Create a root span for each HTTP request with attributes like `http.method`, `http.url`, `http.status_code`.
3. Set the span as the current span in the context.
4. End the span when the response is sent.

### Manual Spans

```python
with tracer.start_as_current_span("simulate-delay") as span:
    span.set_attribute("delay.seconds", delay)
```

`start_as_current_span` does two things:
1. Creates a new span with the given name.
2. Sets it as the "current" span in the execution context.

Any child spans created inside the `with` block will have this span as their parent. When the block exits, the span is ended automatically.

### Exception Recording

```python
span.record_exception(e)
span.set_status(StatusCode.ERROR, str(e))
```

`record_exception` creates an event on the span with the exception type, message, and stack trace. `set_status` marks the span as failed. In Jaeger, this shows as a red/error span with the exception details in the events tab.

## Answers to Questions

**1. How many spans appear for a request to `/slow`?**

Two spans:
- `GET /slow` (created automatically by FlaskInstrumentor)
- `simulate-delay` (created manually, child of the request span)

**2. What attributes are automatically added by the Flask instrumentor?**

- `http.method`: `GET`
- `http.url`: `http://localhost:5000/slow`
- `http.status_code`: `200`
- `http.scheme`: `http`
- `http.target`: `/slow`
- `http.host`: `localhost:5000`
- `net.host.port`: `5000`

**3. What does the trace for `/error` look like?**

The span is marked with a red error indicator in Jaeger. The span status is `ERROR` with the message "Something went wrong!". An event of kind `exception` appears on the span containing:
- `exception.type`: `ValueError`
- `exception.message`: `Something went wrong!`
- `exception.stacktrace`: the full Python traceback

**4. BatchSpanProcessor vs SimpleSpanProcessor?**

SimpleSpanProcessor exports each span synchronously when it ends — blocking the application thread. BatchSpanProcessor buffers spans and exports them asynchronously in batches, reducing overhead and network calls. Use BatchSpanProcessor in production; SimpleSpanProcessor only for debugging or testing.

## Common Mistakes

1. **Instrumenting Flask before setting the TracerProvider**: The FlaskInstrumentor needs a global TracerProvider to be registered first. If you call `FlaskInstrumentor().instrument_app(app)` before `trace.set_tracer_provider(provider)`, spans will be created but not exported.

2. **Using SimpleSpanProcessor in production**: This blocks every request while the span is exported. Under load, this adds significant latency.

3. **Not using the `with` block for manual spans**: If you call `tracer.start_span("name")` without `start_as_current_span` and a `with` block, you must manually call `span.end()`. Forgetting to end spans causes memory leaks and missing data.

4. **Forgetting to record exceptions**: If an exception is raised but not recorded on the span, the span will show as successful in Jaeger even though the request failed. Always use `span.record_exception(e)` and `span.set_status(StatusCode.ERROR)`.

5. **Hardcoding the Jaeger endpoint**: Use environment variables or configuration files so the exporter endpoint can change between development (localhost) and production (jaeger-collector service).
