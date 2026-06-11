# Cheatsheet: Distributed Tracing

## Key Concepts

| Term | Definition |
|------|-----------|
| Trace | Full journey of a request |
| Span | Single unit of work |
| Context | Propagated trace info |
| Sampling | Which traces to collect |

## OpenTelemetry (Standard)
```python
from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider

provider = TracerProvider()
trace.set_tracer_provider(provider)
tracer = trace.get_tracer(__name__)

with tracer.start_as_current_span("my-operation") as span:
    span.set_attribute("user.id", 123)
    span.add_event("processing started")
    # ... do work ...
```

## Jaeger
```bash
# Run Jaeger
docker run -d --name jaeger \
  -p 16686:16686 \
  -p 6831:6831/udp \
  jaegertracing/all-in-one:latest

# View traces at http://localhost:16686
```

## Trace Visualization
```
Request → [API Gateway: 50ms]
              ↓
         [Auth Service: 10ms]
              ↓
         [User Service: 30ms]
              ↓
         [Database: 15ms]
```
