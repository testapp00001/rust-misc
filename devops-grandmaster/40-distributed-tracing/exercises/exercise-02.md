# Exercise 02: Instrument a Python App with OpenTelemetry

## Type: Guided

## Objective

Add OpenTelemetry instrumentation to a Python Flask application so that every HTTP request produces a trace with spans, which are exported to a local Jaeger instance.

## Prerequisites

- Python 3.9+ installed
- Docker installed (for Jaeger)
- Exercise 01 concepts understood

## Part A: Start Jaeger

Run Jaeger all-in-one with Docker:

```bash
docker run -d --name jaeger \
  -p 16686:16686 \
  -p 6831:6831/udp \
  -p 14268:14268 \
  -p 4317:4317 \
  -p 4318:4318 \
  jaegertracing/all-in-one:1.51
```

Verify Jaeger is running by opening http://localhost:16686 in your browser.

## Part B: The Application

Create a file called `app.py` with the following code. This is a simple Flask API with two endpoints — do NOT modify it yet:

```python
from flask import Flask, jsonify
import time
import random

app = Flask(__name__)

@app.route("/")
def home():
    return jsonify({"message": "Hello, traced world!"})

@app.route("/slow")
def slow_endpoint():
    # Simulate a slow operation
    delay = random.uniform(0.5, 2.0)
    time.sleep(delay)
    return jsonify({"message": f"Slept for {delay:.2f}s"})

@app.route("/error")
def error_endpoint():
    # Simulate an error
    raise ValueError("Something went wrong!")

if __name__ == "__main__":
    app.run(port=5000, debug=True)
```

## Part C: Add OpenTelemetry Instrumentation

Create a file called `tracing.py` that:

1. Configures a **TracerProvider** with a **Resource** that includes the service name `"my-flask-app"` and the service version `"1.0.0"`.
2. Sets up a **JaegerExporter** that sends traces to `localhost:6831` (UDP, agent mode).
3. Adds a **BatchSpanProcessor** with the Jaeger exporter to the TracerProvider.
4. Uses the **FlaskInstrumentor** to automatically instrument the Flask app (this creates a span for every incoming HTTP request).
5. Registers the TracerProvider as the global default.

Then modify `app.py` to import and call your tracing setup function BEFORE the Flask app starts.

## Part D: Manual Spans

Add a custom span to the `/slow` endpoint:

1. Get a tracer from `opentelemetry.trace.get_tracer(__name__)`.
2. Create a child span named `"simulate-delay"` inside the `/slow` endpoint.
3. Add the attribute `delay.seconds` with the actual delay value.
4. Record the exception in the `/error` endpoint using `span.record_exception(e)`.

## Tasks

1. Complete the `tracing.py` file as described in Part C.
2. Modify `app.py` to use manual spans as described in Part D.
3. Start the app: `python app.py`
4. Send requests to all three endpoints:
   ```bash
   curl http://localhost:5000/
   curl http://localhost:5000/slow
   curl http://localhost:5000/error
   ```
5. Open the Jaeger UI and find your traces.

## Questions to Answer

1. How many spans appear for a request to `/slow`? Name each span.
2. What attributes are automatically added by the Flask instrumentor?
3. What does the trace for `/error` look like compared to a successful request?
4. What is the purpose of `BatchSpanProcessor` vs `SimpleSpanProcessor`?

## Success Criteria

- [ ] `tracing.py` correctly configures TracerProvider, JaegerExporter, and BatchSpanProcessor
- [ ] `app.py` imports and initializes tracing before the Flask app starts
- [ ] The `/slow` endpoint has a child span named `simulate-delay` with a `delay.seconds` attribute
- [ ] The `/error` endpoint records the exception on the active span
- [ ] Traces appear in the Jaeger UI for all three endpoints

## Hints

<details>
<summary>Hint 1: tracing.py skeleton</summary>

```python
from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from opentelemetry.sdk.resources import Resource
from opentelemetry.exporter.jaeger.thrift import JaegerExporter
from opentelemetry.instrumentation.flask import FlaskInstrumentor

def init_tracing(app):
    resource = Resource.create({
        "service.name": "my-flask-app",
        "service.version": "1.0.0",
    })
    provider = TracerProvider(resource=resource)
    # Create exporter and processor here
    # Register provider and instrument Flask here
```

</details>

<details>
<summary>Hint 2: Jaeger exporter setup</summary>

```python
jaeger_exporter = JaegerExporter(
    agent_host_name="localhost",
    agent_port=6831,
)
provider.add_span_processor(BatchSpanProcessor(jaeger_exporter))
trace.set_tracer_provider(provider)
FlaskInstrumentor().instrument_app(app)
```

</details>

<details>
<summary>Hint 3: Manual span creation</summary>

```python
tracer = trace.get_tracer(__name__)

@app.route("/slow")
def slow_endpoint():
    with tracer.start_as_current_span("simulate-delay") as span:
        delay = random.uniform(0.5, 2.0)
        span.set_attribute("delay.seconds", delay)
        time.sleep(delay)
        return jsonify({"message": f"Slept for {delay:.2f}s"})
```

Use a `with` block so the span automatically ends when the block exits.

</details>

<details>
<summary>Hint 4: Recording exceptions</summary>

```python
from opentelemetry.trace import StatusCode

@app.route("/error")
def error_endpoint():
    try:
        raise ValueError("Something went wrong!")
    except ValueError as e:
        span = trace.get_current_span()
        span.record_exception(e)
        span.set_status(StatusCode.ERROR, str(e))
        raise
```

Setting the status to ERROR and recording the exception ensures the span is marked as failed in Jaeger.

</details>
