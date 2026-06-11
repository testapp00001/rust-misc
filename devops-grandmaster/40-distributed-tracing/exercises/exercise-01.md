# Exercise 01: Distributed Tracing Concepts

## Type: Conceptual

## Objective

Understand the core building blocks of distributed tracing — traces, spans, context propagation, and how they relate to each other in a microservices architecture.

## Background

When a user clicks "Place Order" on an e-commerce site, that single action might trigger calls to 10+ microservices: authentication, inventory, pricing, payment, shipping, notifications, and more. Without distributed tracing, debugging a slow request means manually correlating logs across all those services — a nightmare at scale.

Distributed tracing solves this by assigning a unique identifier to each request and tracking its journey through every service it touches.

## Tasks

### Task 1: Trace Anatomy

A distributed trace is a directed acyclic graph (DAG) of spans. Draw (on paper or in a text diagram) a trace for the following scenario:

> A user submits an order. The API Gateway receives the request, calls the Order Service, which in turn calls both the Inventory Service and the Payment Service. The Payment Service calls an external Stripe API.

For each component, show:
- The span name
- Parent-child relationships
- Approximate timing (make reasonable guesses)

### Task 2: Span Attributes

For each span in your diagram from Task 1, list at least 3 semantic attributes you would attach. Use the OpenTelemetry semantic conventions where applicable.

Reference categories:
- `http.method`, `http.url`, `http.status_code`
- `db.system`, `db.statement`, `db.operation`
- `rpc.service`, `rpc.method`, `rpc.grpc.status_code`
- `net.peer.name`, `net.peer.port`

### Task 3: Context Propagation

Context propagation is the mechanism that links spans across service boundaries. Answer the following:

1. What is the difference between **in-process context propagation** and **cross-process context propagation**?
2. Which HTTP headers are used by the W3C Trace Context standard to propagate trace information?
3. Why can't you simply pass a trace ID as a query parameter instead of using headers?
4. What happens if an intermediate service (like a message queue or proxy) strips the trace context headers?

### Task 4: Sampling

Tracing every single request in a high-traffic system is expensive. Explain:

1. What is **head-based sampling** and where does the decision get made?
2. What is **tail-based sampling** and what advantage does it have over head-based sampling?
3. If you run a system handling 100,000 requests/second and your tracing backend can handle 5,000 spans/second, what sampling rate would you configure? What trade-offs does this introduce?

### Task 5: Trace vs Metrics vs Logs

Fill in the comparison table:

| Aspect | Metrics | Logs | Traces |
|--------|---------|------|--------|
| Data shape | ??? | ??? | ??? |
| Best for detecting | ??? | ??? | ??? |
| Cardinality concern | ??? | ??? | ??? |
| Storage cost | ??? | ??? | ??? |
| Relationship to a request | ??? | ??? | ??? |

## Success Criteria

- [ ] You can explain what a trace, span, and span context are in your own words
- [ ] You can describe how trace context propagates across HTTP service boundaries
- [ ] You understand the trade-offs between different sampling strategies
- [ ] You can articulate when to use traces vs metrics vs logs
- [ ] You can list the W3C Trace Context headers

## Hints

<details>
<summary>Hint 1: Span relationships</summary>

Each span has a `traceId` (shared by all spans in the same trace), a `spanId` (unique to that span), and a `parentSpanId` (pointing to the span that caused this one). The root span has no parent. This forms a tree structure within the trace.

```
[API Gateway (root)] traceId=abc, spanId=1, parentSpanId=null
  └── [Order Service] traceId=abc, spanId=2, parentSpanId=1
        ├── [Inventory Service] traceId=abc, spanId=3, parentSpanId=2
        └── [Payment Service] traceId=abc, spanId=4, parentSpanId=2
              └── [Stripe API] traceId=abc, spanId=5, parentSpanId=4
```

</details>

<details>
<summary>Hint 2: W3C Trace Context headers</summary>

The W3C Trace Context standard defines two headers:
- `traceparent`: Contains the trace ID, parent span ID, and trace flags
- `tracestate`: Vendor-specific trace data (e.g., `vendor1=value1`)

The `traceparent` format is: `version-traceId-parentSpanId-traceFlags`
Example: `00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01`

The last two hex digits (`01`) indicate the trace is sampled.

</details>

<details>
<summary>Hint 3: Context propagation mechanisms</summary>

In-process propagation uses thread-local storage (Python contextvars, Java ThreadLocal, Go context.Context). Cross-process propagation serializes context into carrier objects — HTTP headers for synchronous calls, message headers or metadata for async messaging systems like Kafka or RabbitMQ.

</details>

<details>
<summary>Hint 4: Sampling math</summary>

5,000 / 100,000 = 0.05 = 5% sampling rate. You would only trace 1 in 20 requests. The trade-off: you might miss rare errors or slow requests that occur in the 95% you did not sample. Tail-based sampling solves this by making the decision after the trace completes, keeping only interesting traces (errors, slow).

</details>
