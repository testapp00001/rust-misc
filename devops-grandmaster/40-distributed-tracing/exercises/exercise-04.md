# Exercise 04: Analyze Traces to Find Performance Bottlenecks

## Type: Challenge

## Objective

Given a set of trace data from a production-like system, identify performance bottlenecks, diagnose root causes, and propose fixes.

## Background

You are an SRE investigating a latency regression. Users report that the checkout flow, which used to take ~500ms, now takes 2-5 seconds. You have access to traces in Jaeger.

The checkout flow involves:

```
[API Gateway] --> [Checkout Service]
                       |
                       +--> [Inventory Service] --> [Inventory DB]
                       |
                       +--> [Pricing Service] --> [Redis Cache]
                       |         |
                       |         +--> [Promotion Service]
                       |
                       +--> [Payment Service] --> [Stripe API]
                       |
                       +--> [Notification Service] --> [Email Service]
```

## Part A: Simulate the Problem

Create `bottleneck_app.py` that simulates all the services above in a single process. Each service is a function that creates spans with realistic delays:

```python
# Healthy baseline delays (normal operation):
# API Gateway:          5ms
# Checkout Service:     10ms
# Inventory Service:    50ms
# Inventory DB:         30ms
# Pricing Service:      20ms
# Redis Cache:          2ms (cache hit)
# Promotion Service:    40ms
# Payment Service:      200ms
# Stripe API:           150ms
# Notification Service: 10ms
# Email Service:        50ms

# Degraded delays (current production):
# Inventory Service:    800ms  (was 50ms — slow query)
# Redis Cache:          MISS — falls through to Promotion Service
# Promotion Service:    1500ms (was 40ms — timeout retry)
# Stripe API:           600ms  (was 150ms — rate limiting)
```

Instrument the simulation with OpenTelemetry and export to Jaeger.

Run the simulation twice:
1. First with the **healthy** delays to generate baseline traces.
2. Then with the **degraded** delays to generate regression traces.

## Part B: Trace Analysis

Using the Jaeger UI, answer the following questions:

1. **Critical Path**: For the degraded trace, what is the critical path (the longest chain of dependent spans)? What is its total duration?

2. **Span Duration Comparison**: Create a table comparing each span's duration between the healthy and degraded traces:

   | Span Name | Healthy (ms) | Degraded (ms) | Delta |
   |-----------|-------------|---------------|-------|

3. **Parallelism Analysis**: Which spans in the Checkout Service are executed in parallel vs sequentially? If the Checkout Service made all downstream calls in parallel, what would the total duration be?

4. **Error Analysis**: Are there any spans with error status in the degraded trace? What caused them?

5. **Root Cause Ranking**: Rank the performance issues by impact (which one adds the most latency to the overall request).

## Part C: Propose Fixes

For each identified bottleneck, propose a specific fix:

1. For the Inventory Service slow query (800ms vs 50ms):
   - What instrumentation would help diagnose the specific slow query?
   - What architectural change would fix it?

2. For the Redis cache miss + Promotion Service timeout:
   - How would you detect cache miss patterns from traces alone?
   - What circuit breaker configuration would prevent the 1.5s penalty?

3. For the Stripe API rate limiting:
   - What trace attributes would you add to track rate limit status?
   - How would you implement exponential backoff and reflect it in the trace?

## Part D: Implement Fixes

Modify `bottleneck_app.py` to implement:

1. **Parallel downstream calls** in the Checkout Service using `ThreadPoolExecutor`.
2. **A circuit breaker** for the Promotion Service (if it fails, return a default discount of 0).
3. **Cache warming** for the Pricing Service (pre-populate Redis to avoid misses).
4. **Custom span attributes** that track cache hit/miss status and circuit breaker state.

Run the fixed version and compare traces.

## Success Criteria

- [ ] You can identify the critical path in the degraded trace
- [ ] You correctly rank the bottlenecks by impact
- [ ] Your fix reduces the checkout time from 2-5 seconds to under 500ms
- [ ] Fixed traces show parallel downstream calls in the waterfall view
- [ ] Circuit breaker spans show the state transition (closed -> open)
- [ ] Cache hit/miss is visible as a span attribute

## Hints

<details>
<summary>Hint 1: Simulating services with spans</summary>

```python
from opentelemetry import trace
import time

tracer = trace.get_tracer("checkout-simulator")

def inventory_service(degraded=False):
    with tracer.start_as_current_span("inventory-service") as span:
        delay = 0.8 if degraded else 0.05
        span.set_attribute("service.delay.ms", delay * 1000)
        # Call database
        with tracer.start_as_current_span("inventory-db"):
            time.sleep(0.03)
        time.sleep(delay - 0.03)
        return {"stock": 10}
```

</details>

<details>
<summary>Hint 2: Detecting the critical path</summary>

The critical path is the longest path from the root span's start to the last span's end. In Jaeger, this is highlighted in the trace timeline view. In code, you can compute it by finding the span with the latest `endTime` and tracing back through its parent chain.

To extract trace data programmatically, query the Jaeger API:

```bash
curl http://localhost:16686/api/traces?service=checkout-service&limit=1 | python -m json.tool
```

</details>

<details>
<summary>Hint 3: Circuit breaker pattern</summary>

```python
class CircuitBreaker:
    def __init__(self, failure_threshold=3, timeout=30):
        self.failure_count = 0
        self.failure_threshold = failure_threshold
        self.timeout = timeout
        self.last_failure_time = None
        self.state = "closed"  # closed, open, half-open

    def call(self, func, fallback):
        if self.state == "open":
            if time.time() - self.last_failure_time > self.timeout:
                self.state = "half-open"
            else:
                return fallback()

        try:
            result = func()
            self.failure_count = 0
            self.state = "closed"
            return result
        except Exception as e:
            self.failure_count += 1
            self.last_failure_time = time.time()
            if self.failure_count >= self.failure_threshold:
                self.state = "open"
            raise
```

Add `circuit_breaker.state` as a span attribute.

</details>

<details>
<summary>Hint 4: Parallel calls with tracing context</summary>

```python
from concurrent.futures import ThreadPoolExecutor

def checkout(user_id, degraded=False):
    with tracer.start_as_current_span("checkout-service") as span:
        with ThreadPoolExecutor(max_workers=4) as executor:
            inventory_future = executor.submit(inventory_service, degraded)
            pricing_future = executor.submit(pricing_service, degraded)
            payment_future = executor.submit(payment_service, degraded)
            notification_future = executor.submit(notification_service)

            inventory = inventory_future.result()
            pricing = pricing_future.result()
            payment = payment_future.result()
            notification = notification_future.result()

        span.set_attribute("checkout.total", pricing["total"])
        return {"status": "success"}
```

The OpenTelemetry context propagation handles cross-thread span parenting correctly when using `start_as_current_span`.

</details>
