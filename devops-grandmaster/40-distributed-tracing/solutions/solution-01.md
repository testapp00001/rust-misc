# Solution 01: Distributed Tracing Concepts

## Task 1: Trace Anatomy — Text Diagram

```
Trace ID: abc123def456
Total Duration: ~350ms

[API Gateway] ───────────────────────────── 350ms (root span)
  traceId=abc123, spanId=1, parentSpanId=null
  |
  └── [Order Service] ───────────────────── 300ms
        traceId=abc123, spanId=2, parentSpanId=1
        |
        ├── [Inventory Service] ─────────── 80ms
        |     traceId=abc123, spanId=3, parentSpanId=2
        |
        └── [Payment Service] ───────────── 200ms
              traceId=abc123, spanId=4, parentSpanId=2
              |
              └── [Stripe API] ──────────── 150ms
                    traceId=abc123, spanId=5, parentSpanId=4

Timeline (parallel execution):
0ms          100ms         200ms         300ms    350ms
|------------|-------------|-------------|--------|
[API Gateway (overhead ~50ms)]
  [Order Service]
    [Inventory Service (80ms)]
    [Payment Service ────────────]
      [Stripe API ───────]
```

### Why this structure?

- The API Gateway is the root span because it receives the external request.
- The Order Service is a child of the API Gateway because the gateway forwards the request.
- Inventory and Payment are children of Order Service because the order service orchestrates them.
- Stripe is a child of Payment because the payment service makes the external call.
- Inventory and Payment can run in parallel since neither depends on the other's output.

## Task 2: Span Attributes

**API Gateway span:**
- `http.method`: `POST`
- `http.url`: `/api/orders`
- `http.status_code`: `201`
- `http.request_content_length`: `256`
- `net.host.name`: `api-gateway.internal`

**Order Service span:**
- `rpc.service`: `OrderService`
- `rpc.method`: `CreateOrder`
- `order.id`: `ORD-98765`
- `order.item_count`: `3`
- `order.total`: `149.97`

**Inventory Service span:**
- `db.system`: `postgresql`
- `db.operation`: `SELECT`
- `db.statement`: `SELECT stock FROM products WHERE id IN (...)`
- `db.rows_affected`: `3`
- `net.peer.name`: `inventory-db.internal`

**Payment Service span:**
- `rpc.service`: `PaymentService`
- `rpc.method`: `ProcessPayment`
- `payment.amount`: `149.97`
- `payment.currency`: `USD`
- `payment.method`: `credit_card`

**Stripe API span:**
- `http.method`: `POST`
- `http.url`: `https://api.stripe.com/v1/charges`
- `http.status_code`: `200`
- `http.response_content_length`: `512`
- `net.peer.name`: `api.stripe.com`

### Why these attributes?

OpenTelemetry defines semantic conventions for common operations (HTTP, database, RPC). Using these conventions ensures consistency across services and enables tools like Jaeger to automatically parse and display them. Custom attributes (like `order.id`) provide business context.

## Task 3: Context Propagation

**1. In-process vs cross-process propagation:**

In-process propagation passes context within a single application, typically using thread-local storage (Python `contextvars`, Java `ThreadLocal`, Go `context.Context`). When a span is created, it is stored in the current thread's context. Any code running on that thread can access the active span.

Cross-process propagation serializes the context into a carrier (HTTP headers, message metadata) so that another process can reconstruct it. This is necessary because threads do not share memory across process boundaries.

**2. W3C Trace Context headers:**

- `traceparent`: Contains version, trace ID, parent span ID, and trace flags.
  Format: `{version}-{trace-id}-{parent-span-id}-{trace-flags}`
  Example: `00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01`

- `tracestate`: Vendor-specific key-value pairs for additional trace metadata.
  Example: `congo=t61rcWkgMzE,rojo=00f067aa0ba902b7`

**3. Why not use query parameters?**

Query parameters are part of the URL and are:
- Logged by proxies, CDNs, and web servers (leaking trace IDs in access logs).
- Visible in browser history and bookmarks.
- Cachable by intermediaries (a cached response with one trace ID would be wrong for another request).
- Limited in size and format (headers support arbitrary key-value pairs).
- Not designed for metadata that should not affect routing or caching.

**4. What happens if headers are stripped?**

The receiving service has no context to continue the trace. It will create a new root span with a new trace ID, resulting in two disconnected traces. The downstream portion is orphaned — you can see it in Jaeger but cannot link it to the upstream request. This is a common problem with API gateways, load balancers, or proxies that do not forward custom headers. Solution: configure the intermediary to pass through `traceparent` and `tracestate` headers.

## Task 4: Sampling

**1. Head-based sampling:**

The decision to sample (record) a trace is made at the root span — at the very start ("head") of the trace, before any processing occurs. The decision is propagated to all child spans via the `trace-flags` field in the `traceparent` header. If the root decides not to sample, no spans in the entire trace are recorded.

Pros: Simple, low overhead, consistent decision across all services.
Cons: Cannot know in advance if the trace will be interesting (e.g., contain errors).

**2. Tail-based sampling:**

The decision is made after the trace completes. All spans are collected by a central component (like the OpenTelemetry Collector), which evaluates the entire trace before deciding whether to keep it. Traces with errors, high latency, or specific attributes are kept; routine successful traces are dropped.

Pros: Keeps only interesting traces, no risk of missing errors.
Cons: Requires a central collector with memory to buffer traces, introduces latency, more complex to operate.

**3. Sampling math:**

Rate = 5,000 / 100,000 = 0.05 = **5% sampling rate**.

Trade-offs:
- You will miss 95% of traces, including potentially important error traces.
- Statistical analysis (P50, P95, P99 latency) is still valid with 5% sampling if the sample is random and representative.
- Rare errors might not appear in the sampled set — you need a way to guarantee error traces are captured (tail-based sampling or a separate error-only pipeline).
- If traffic is bursty, a fixed 5% rate might under-sample during peaks and over-sample during troughs.

## Task 5: Trace vs Metrics vs Logs

| Aspect | Metrics | Logs | Traces |
|--------|---------|------|--------|
| **Data shape** | Numeric time series (counters, gauges, histograms) | Structured or unstructured text entries with timestamps | A directed acyclic graph (DAG) of spans with timing and relationships |
| **Best for detecting** | Trends, thresholds, and aggregate anomalies (CPU spike, error rate increase) | Specific events, errors, and debugging details (stack traces, request params) | Request flow, latency distribution, and cross-service dependencies |
| **Cardinality concern** | High cardinality labels (e.g., user_id) cause metric explosion | High-volume logs overwhelm storage and indexing | High-cardinality attributes increase trace storage but are generally acceptable per-span |
| **Storage cost** | Low — numeric data compresses well | High — text data, especially unstructured, accumulates fast | Medium — spans are structured but carry more data than metrics |
| **Relationship to a request** | Aggregated across many requests; loses individual request identity | Usually tied to a single service's view of a request | Tracks a single request end-to-end across all services |

### Why this matters

The three pillars work together:
- **Metrics** tell you *something is wrong* (latency P99 spiked).
- **Traces** tell you *where it is wrong* (the Payment Service is slow).
- **Logs** tell you *why it is wrong* (Stripe returned a rate limit error).

Exemplars bridge metrics to traces — clicking a high-latency data point takes you to the specific trace that caused it.

## Common Mistakes

1. **Confusing trace ID with span ID**: A trace ID is shared by all spans in one trace. A span ID is unique to each span. You need both to identify a specific span.

2. **Forgetting the root span has no parent**: The root span's `parentSpanId` is null/empty. Do not invent a parent for it.

3. **Assuming all spans are sequential**: Spans can overlap in time. Two child spans of the same parent can run in parallel.

4. **Thinking sampling is all-or-nothing**: You can use probabilistic sampling (5%), rate-limiting sampling (1000 traces/sec), or attribute-based sampling (always sample errors).

5. **Ignoring the cost of high-cardinality attributes**: Adding `user_id` or `request_id` as span attributes is fine, but adding them as metric labels causes cardinality explosion.
