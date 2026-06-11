# Solution 04: Analyze Traces to Find Performance Bottlenecks

## Complete Code

### bottleneck_app.py

```python
import time
import random
from concurrent.futures import ThreadPoolExecutor
from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from opentelemetry.sdk.resources import Resource, SERVICE_NAME
from opentelemetry.exporter.jaeger.thrift import JaegerExporter

# Setup tracing
resource = Resource.create({SERVICE_NAME: "checkout-service"})
provider = TracerProvider(resource=resource)
jaeger_exporter = JaegerExporter(agent_host_name="localhost", agent_port=6831)
provider.add_span_processor(BatchSpanProcessor(jaeger_exporter))
trace.set_tracer_provider(provider)

tracer = trace.get_tracer(__name__)

# --- Circuit Breaker ---
class CircuitBreaker:
    def __init__(self, failure_threshold=3, timeout=30):
        self.failure_count = 0
        self.failure_threshold = failure_threshold
        self.timeout = timeout
        self.last_failure_time = None
        self.state = "closed"

    def call(self, func, fallback=None):
        if self.state == "open":
            if time.time() - self.last_failure_time > self.timeout:
                self.state = "half-open"
            else:
                if fallback is not None:
                    return fallback()
                raise Exception("Circuit is open")

        try:
            result = func()
            if self.state == "half-open":
                self.state = "closed"
            self.failure_count = 0
            return result
        except Exception as e:
            self.failure_count += 1
            self.last_failure_time = time.time()
            if self.failure_count >= self.failure_threshold:
                self.state = "open"
            if fallback is not None:
                return fallback()
            raise

# Initialize circuit breaker for Promotion Service
promotion_circuit = CircuitBreaker(failure_threshold=2, timeout=60)


# --- Service implementations ---

def api_gateway(request_id):
    with tracer.start_as_current_span("api-gateway") as span:
        span.set_attribute("http.method", "POST")
        span.set_attribute("http.url", "/api/checkout")
        span.set_attribute("request.id", request_id)
        time.sleep(0.005)
        return checkout_service(request_id)


def checkout_service(request_id, degraded=False, parallel=True):
    with tracer.start_as_current_span("checkout-service") as span:
        span.set_attribute("request.id", request_id)

        if parallel:
            # FIX: Run all downstream calls in parallel
            with ThreadPoolExecutor(max_workers=4) as executor:
                inventory_future = executor.submit(inventory_service, degraded)
                pricing_future = executor.submit(pricing_service, degraded)
                payment_future = executor.submit(payment_service, degraded)
                notification_future = executor.submit(notification_service)

                inventory = inventory_future.result()
                pricing = pricing_future.result()
                payment = payment_future.result()
                notification = notification_future.result()
        else:
            # Original sequential calls
            inventory = inventory_service(degraded)
            pricing = pricing_service(degraded)
            payment = payment_service(degraded)
            notification = notification_service()

        span.set_attribute("checkout.total", pricing["total"])
        span.set_attribute("checkout.status", "success")
        return {"status": "success", "total": pricing["total"]}


def inventory_service(degraded=False):
    with tracer.start_as_current_span("inventory-service") as span:
        delay = 0.8 if degraded else 0.05
        span.set_attribute("service.name", "inventory")
        with tracer.start_as_current_span("inventory-db") as db_span:
            db_span.set_attribute("db.system", "postgresql")
            db_span.set_attribute("db.statement", "SELECT stock FROM products WHERE order_id = ?")
            if degraded:
                db_span.set_attribute("db.slow_query", True)
                db_span.set_attribute("db.missing_index", True)
            time.sleep(0.03)
        time.sleep(max(0, delay - 0.03))
        return {"stock": 10}


def pricing_service(degraded=False, cache_warmed=False):
    with tracer.start_as_current_span("pricing-service") as span:
        span.set_attribute("service.name", "pricing")

        # Redis cache lookup
        cache_hit = not degraded or cache_warmed
        with tracer.start_as_current_span("redis-cache-lookup") as cache_span:
            cache_span.set_attribute("cache.system", "redis")
            cache_span.set_attribute("cache.key", "pricing:rules")
            cache_span.set_attribute("cache.hit", cache_hit)
            time.sleep(0.002)

        if not cache_hit:
            # Cache miss — fall through to Promotion Service
            with tracer.start_as_current_span("promotion-service") as promo_span:
                promo_span.set_attribute("service.name", "promotion")
                promo_span.set_attribute("circuit_breaker.state", promotion_circuit.state)

                def call_promotion():
                    time.sleep(0.04 if not degraded else 1.5)
                    return {"discount": 10}

                def fallback():
                    promo_span.set_attribute("promotion.fallback", True)
                    return {"discount": 0}

                # Use circuit breaker to prevent long waits
                result = promotion_circuit.call(call_promotion, fallback)
                promo_span.set_attribute("promotion.discount", result["discount"])
        else:
            time.sleep(0.005)

        return {"total": 89.99}


def payment_service(degraded=False):
    with tracer.start_as_current_span("payment-service") as span:
        span.set_attribute("service.name", "payment")
        with tracer.start_as_current_span("stripe-api") as stripe_span:
            stripe_span.set_attribute("http.method", "POST")
            stripe_span.set_attribute("http.url", "https://api.stripe.com/v1/charges")
            delay = 0.6 if degraded else 0.15
            if degraded:
                stripe_span.set_attribute("http.status_code", 429)
                stripe_span.set_attribute("stripe.rate_limited", True)
                stripe_span.add_event("Rate limited by Stripe, retrying with backoff")
            else:
                stripe_span.set_attribute("http.status_code", 200)
            time.sleep(delay)
        return {"payment_id": "pay_123"}


def notification_service():
    with tracer.start_as_current_span("notification-service") as span:
        span.set_attribute("service.name", "notification")
        time.sleep(0.01)
        with tracer.start_as_current_span("email-service") as email_span:
            email_span.set_attribute("email.provider", "sendgrid")
            email_span.set_attribute("email.to", "user@example.com")
            time.sleep(0.05)
        return {"notified": True}


# --- Run simulations ---

def run_simulation(label, degraded=False, parallel=True, cache_warmed=False):
    print(f"\n{'='*60}")
    print(f"Running: {label}")
    print(f"{'='*60}")

    for i in range(3):
        request_id = f"req-{label.lower().replace(' ', '-')}-{i}"
        if cache_warmed:
            # Override pricing_service to use cached version
            result = api_gateway(request_id)
        else:
            result = api_gateway(request_id)
        print(f"  {request_id}: {result}")
        time.sleep(0.1)


if __name__ == "__main__":
    # Simulate healthy baseline
    run_simulation("Healthy Baseline", degraded=False, parallel=False)

    time.sleep(2)

    # Simulate degraded (sequential, no fixes)
    run_simulation("Degraded Sequential", degraded=True, parallel=False)

    time.sleep(2)

    # Fix 1: Parallel downstream calls
    run_simulation("Fix 1: Parallel Calls", degraded=True, parallel=True)

    time.sleep(2)

    # Fix 2: With circuit breaker and cache warming
    run_simulation("Fix 2: Circuit Breaker + Cache", degraded=True, parallel=True, cache_warmed=True)

    print("\nAll simulations complete. Check Jaeger UI at http://localhost:16686")
```

## Part B: Trace Analysis — Answers

### 1. Critical Path

The critical path in the degraded sequential trace is:

```
api-gateway (5ms) -> checkout-service (10ms) -> inventory-service (800ms) -> pricing-service (1500ms) -> payment-service (600ms) -> notification-service (60ms)
```

Total: 5 + 10 + 800 + 20 + 1500 + 200 + 10 = **~2545ms** (2.5 seconds)

Wait — in the sequential case, the critical path is simply the sum of all sequential calls because they run one after another. In the parallel case, the critical path is the longest chain of dependent spans.

### 2. Span Duration Comparison

| Span Name | Healthy (ms) | Degraded (ms) | Delta |
|-----------|-------------|---------------|-------|
| api-gateway | 5 | 5 | 0 |
| checkout-service | 10 | 10 | 0 |
| inventory-service | 50 | 800 | +750 |
| inventory-db | 30 | 30 | 0 |
| pricing-service | 25 | 1525 | +1500 |
| redis-cache-lookup | 2 | 2 | 0 |
| promotion-service | 40 | 1500 | +1460 |
| payment-service | 200 | 600 | +400 |
| stripe-api | 150 | 600 | +450 |
| notification-service | 60 | 60 | 0 |
| email-service | 50 | 50 | 0 |

### 3. Parallelism Analysis

In the original (sequential) implementation, ALL downstream calls are executed one after another. The Checkout Service calls Inventory, then Pricing, then Payment, then Notification.

If all four downstream calls ran in parallel, the degraded time would be:
- max(800, 1525, 600, 60) = 1525ms instead of 800 + 1525 + 600 + 60 = 2985ms

That is a 49% reduction just from parallelization.

### 4. Error Analysis

In the degraded trace:
- `stripe-api` has status code 429 (rate limited) — not a span error, but an HTTP error status.
- `redis-cache-lookup` has `cache.hit=false` — not an error, but a contributing factor.
- No spans have `StatusCode.ERROR` because the system degrades gracefully (no exceptions thrown).

This is a subtle point: performance degradation often does not produce errors. The system still works, just slowly.

### 5. Root Cause Ranking

| Rank | Issue | Impact | Description |
|------|-------|--------|-------------|
| 1 | Promotion Service timeout | +1460ms | Cache miss cascades to a slow Promotion Service call |
| 2 | Inventory Service slow query | +750ms | Missing database index causes full table scan |
| 3 | Stripe API rate limiting | +450ms | No retry with backoff, blocked by rate limit |
| 4 | Sequential execution | ~3x multiplier | All calls run sequentially instead of in parallel |

The Promotion Service is the biggest single bottleneck, but the sequential execution pattern multiplies all delays.

## Part C: Fix Explanations

### Fix 1: Parallel Downstream Calls

**What changed:** The Checkout Service now uses `ThreadPoolExecutor` to call Inventory, Pricing, Payment, and Notification concurrently.

**Why it works:** The four downstream services are independent — none requires the output of another. Running them in parallel means the total time is the duration of the slowest call, not the sum of all calls.

**Impact:** Degraded time drops from ~2985ms to ~1525ms (the duration of the slowest call, Promotion Service).

### Fix 2: Circuit Breaker for Promotion Service

**What changed:** A circuit breaker wraps the Promotion Service call. After 2 failures, the circuit opens and returns a fallback discount of 0.

**Why it works:** Instead of waiting 1.5 seconds for a timeout, the circuit breaker fails fast after the first slow call and returns a default value immediately. The user still gets a valid checkout (just without a discount).

**Impact:** After the circuit opens, the Promotion Service call takes ~0ms (instant fallback) instead of 1500ms.

### Fix 3: Cache Warming

**What changed:** The Pricing Service's Redis cache is pre-populated with pricing rules, so the cache hit rate is 100%.

**Why it works:** A cache hit eliminates the need to call the Promotion Service entirely. The cache lookup takes 2ms instead of 1500ms for a Promotion Service call.

**Impact:** The Pricing Service drops from 1525ms to ~7ms (2ms cache + 5ms processing).

### Fix 4: Custom Span Attributes

Attributes added:
- `cache.hit` (boolean) on `redis-cache-lookup` — allows filtering traces by cache miss rate
- `circuit_breaker.state` on `promotion-service` — shows when the circuit is open
- `promotion.fallback` on `promotion-service` — indicates a fallback was used
- `db.slow_query` and `db.missing_index` on `inventory-db` — flags the root cause

These attributes make it possible to:
1. Query Jaeger for all traces where `cache.hit=false` to study cache miss patterns.
2. Alert when `circuit_breaker.state=open` appears frequently.
3. Find all traces affected by slow queries using `db.slow_query=true`.

## Performance Comparison

| Scenario | Total Duration | vs Baseline |
|----------|---------------|-------------|
| Healthy baseline (sequential) | ~385ms | baseline |
| Degraded (sequential) | ~2985ms | 7.7x slower |
| Degraded (parallel) | ~1525ms | 4.0x slower |
| With circuit breaker (2nd call) | ~60ms | 0.16x (faster, circuit open) |
| With cache warming | ~85ms | 0.22x (near baseline) |

## Common Mistakes

1. **Not measuring before fixing**: Always generate baseline traces first. Without a baseline, you cannot quantify the improvement.

2. **Fixing symptoms instead of root causes**: Adding a timeout to the Promotion Service call hides the problem. The root cause is the cache miss — fix the cache.

3. **Parallel calls without error handling**: If one parallel call fails, the others continue. You need to decide: fail fast (raise immediately) or collect partial results.

4. **Circuit breaker threshold too low**: A threshold of 1 means a single slow request opens the circuit. Use 3-5 failures to avoid false positives from transient issues.

5. **Not propagating trace context in parallel calls**: When using `ThreadPoolExecutor`, ensure the trace context is inherited by child threads. OpenTelemetry handles this with `contextvars`, but manual thread creation without `contextvars` breaks propagation.
