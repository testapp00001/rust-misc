# Module 11: Flash Sale API

## Motivation

This is the **integration module** that brings together every concept from
Modules 01-10 into a single, runnable Axum HTTP API. Rather than isolated
exercises, you now assemble a complete request pipeline: middleware layers,
service calls, Redis Lua scripts, circuit breakers, and async order processing.

By the end of this module you will understand how a production flash sale API
handles 100k+ requests/second without overselling, duplicating vouchers, or
cascading failures.

## Concept Map

```
                         Module 11: Flash Sale API
 ┌─────────────────────────────────────────────────────────────────┐
 │                                                                 │
 │  Client Request                                                 │
 │       │                                                         │
 │       ▼                                                         │
 │  ┌──────────────────┐  Module 04: Traffic Shaping               │
 │  │  Rate Limit MW   │  (per-account sliding window)             │
 │  └────────┬─────────┘                                           │
 │           ▼                                                     │
 │  ┌──────────────────┐  Module 05: Idempotency                   │
 │  │  Idempotency MW  │  (Idempotency-Key header -> cached resp)  │
 │  └────────┬─────────┘                                           │
 │           ▼                                                     │
 │  ┌──────────────────┐  Module 09: Observability                 │
 │  │  Tracing MW      │  (request_id, spans, latency)             │
 │  └────────┬─────────┘                                           │
 │           ▼                                                     │
 │  ┌──────────────────┐                                           │
 │  │  Route Handler   │  Axum handler                             │
 │  └────────┬─────────┘                                           │
 │           ▼                                                     │
 │  ┌──────────────────┐  Module 02: Lua Scripting                 │
 │  │  Stock Service   │  (atomic check-and-decrement in Redis)     │
 │  └────────┬─────────┘                                           │
 │           ├──────────────────┐                                  │
 │           ▼                  ▼                                  │
 │  ┌──────────────────┐  ┌──────────────────┐                     │
 │  │  Voucher Service │  │  Order Service   │  Module 06: Events  │
 │  │  (UUID codes)    │  │  (async queue)   │                     │
 │  └──────────────────┘  └──────────────────┘                     │
 │           │                                                     │
 │           ▼                                                     │
 │  ┌──────────────────┐  Module 07: Resilience                    │
 │  │  Circuit Breaker │  (open/half-open/closed)                  │
 │  └──────────────────┘                                           │
 │                                                                 │
 └─────────────────────────────────────────────────────────────────┘
```

## Theory

### Axum Middleware Pipeline

Axum uses Tower's `Layer` and `Service` traits to compose middleware. Layers
wrap the inner service, transforming requests before they reach the handler
and responses after:

```rust
Router::new()
    .route("/purchase", post(handler))
    .layer(InnermostLayer)   // applied last  (closest to handler)
    .layer(OutermostLayer)  // applied first (sees request first)
```

Each middleware implements `Service<Request<Body>>` and can:
- **Short-circuit**: return a response without calling the inner service
  (rate limiting, idempotency cache hit).
- **Transform**: modify the request or response (add headers, log).
- **Wrap errors**: catch panics and format them as JSON.

### Service Layer Design

Business logic is separated from HTTP concerns:

| Layer    | Responsibility                            |
|----------|-------------------------------------------|
| Routes   | Parse input, call services, format output |
| Services | Business rules, Redis/DB interaction      |
| Models   | Shared data types                         |
| Resilience | Circuit breaker, fallbacks             |

This separation makes each piece independently testable.

### Lua Script (Module 02 Integration)

The stock service embeds the Lua script from Module 02 as a `const` string.
The script runs atomically in Redis, performing six checks in a single round
trip -- eliminating race conditions between concurrent requests.

## Trade-offs

### In-Memory vs. Distributed Rate Limiting

| Aspect            | In-Memory (this module)     | Distributed (Redis)            |
|-------------------|-----------------------------|--------------------------------|
| Latency           | ~1 microsecond              | ~1 millisecond                 |
| Accuracy          | Per-node only               | Global across all nodes        |
| Complexity        | Simple DashMap              | Lua script + sorted sets       |
| Failure mode      | Limits reset on restart     | Requires Redis availability    |
| Best for          | Single-node, learning       | Production multi-node          |

This module uses in-memory rate limiting for simplicity. In production, you
would use Redis-based sliding window or token bucket (Module 04).

### Idempotency: Middleware vs. Handler

This module implements idempotency at the **middleware** level (caching full
HTTP responses). Module 05 implemented it at the **Lua script** level (inside
the atomic operation). Both are valid; the Lua approach is more precise
(only caches the business outcome), while the middleware approach catches
duplicate retries transparently.

### Circuit Breaker Thresholds

- **failure_threshold = 5**: After 5 consecutive failures, the circuit opens.
  Too low risks false positives from transient errors; too high allows
  cascading failures to propagate.
- **recovery_timeout = 30s**: How long to wait before probing again.
  Must be long enough for the downstream to recover.

## Failure Modes

| Component          | Failure               | Behavior                                   |
|--------------------|-----------------------|--------------------------------------------|
| Redis down         | Connection refused    | Circuit breaker opens; stock queries return sold-out fallback; purchases rejected with 503 |
| Redis slow         | High latency          | Tracing records latency; circuit breaker trips if >5 slow calls |
| Lua script error   | Bad key/value         | Stock service returns error; handler logs and returns generic 500 |
| Order queue full   | Backpressure          | `queue_order` returns error; purchase still succeeds (order loss accepted) |
| Notification fail  | Email/SMS down        | Fire-and-forget; logged but purchase is not blocked |
| Panic in handler   | Bug in code           | Error handler middleware catches it; returns safe JSON 500 |

## Connection to Previous Modules

| Module | What it provides to Module 11 |
|--------|-------------------------------|
| 01 - Redis Fundamentals | Connection pooling via `deadpool-redis` |
| 02 - Lua Scripting | The atomic purchase Lua script embedded in `stock_service` |
| 03 - Atomic Counters | The `DECR`/`INCR` patterns used for stock and voucher counts |
| 04 - Traffic Shaping | The sliding window algorithm used in `rate_limit` middleware |
| 05 - Idempotency | The `Idempotency-Key` pattern used in `idempotency` middleware |
| 06 - Event Sourcing | The `PendingOrder` type and async queue pattern |
| 07 - Resilience | Circuit breaker and fallback strategies |
| 08 - Load Testing | This API is the system under test for Module 08 |
| 09 - Observability | The tracing middleware with spans, request_id, and latency |
| 10 - Caching Strategy | Stock queries use Redis as a fast-path cache |

## Exercise List

| # | Exercise | Difficulty | Description |
|---|----------|------------|-------------|
| 1 | **Stock Service** | Medium | Implement `check_and_decrement_stock` using the Lua script. |
| 2 | **Voucher Service** | Easy | Implement `generate_voucher` and `check_voucher_limit`. |
| 3 | **Order Service** | Easy | Implement `queue_order` and `process_pending_orders`. |
| 4 | **Notification Service** | Easy | Implement fire-and-forget notification dispatch. |
| 5 | **Rate Limit Middleware** | Hard | Implement the Tower `Layer` + `Service` for per-account rate limiting. |
| 6 | **Idempotency Middleware** | Hard | Implement the Tower `Layer` + `Service` for response caching. |
| 7 | **Tracing Middleware** | Medium | Implement the Tower `Layer` + `Service` with spans. |
| 8 | **Error Handler Middleware** | Medium | Implement panic catching and JSON error responses. |
| 9 | **Purchase Handler** | Medium | Wire up the full purchase flow: validate, stock check, voucher, order. |
| 10 | **Stock Query Handler** | Easy | Implement the stock query endpoint with fallback. |
| 11 | **Health Handler** | Easy | Implement the health check endpoint. |
| 12 | **create_app** | Medium | Wire up all components in the `create_app` function. |

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          Flash Sale API                                 │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                      Middleware Stack                            │   │
│  │                                                                 │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌────────────┐ ┌───────────┐  │   │
│  │  │   Error     │ │   Tracing   │ │  Rate      │ │ Idempot.  │  │   │
│  │  │   Handler   │→│   MW        │→│  Limit MW  │→│ MW        │  │   │
│  │  └─────────────┘ └─────────────┘ └────────────┘ └───────────┘  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                   │                                    │
│                                   ▼                                    │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                       Route Handlers                             │   │
│  │                                                                 │   │
│  │  POST /purchase      GET /stock/:id       GET /health           │   │
│  └───────┬─────────────────────┬───────────────────┬───────────────┘   │
│          │                     │                   │                    │
│          ▼                     ▼                   │                    │
│  ┌───────────────────────────────────────────┐     │                    │
│  │             Service Layer                 │     │                    │
│  │                                           │     │                    │
│  │  ┌──────────────┐  ┌──────────────┐       │     │                    │
│  │  │ Stock Svc    │  │ Voucher Svc  │       │     │                    │
│  │  │ (Lua script) │  │ (UUID gen)   │       │     │                    │
│  │  └──────┬───────┘  └──────────────┘       │     │                    │
│  │         │                                  │     │                    │
│  │  ┌──────────────┐  ┌──────────────┐       │     │                    │
│  │  │ Order Svc    │  │ Notification │       │     │                    │
│  │  │ (MPSC queue) │  │ (fire+forget)│       │     │                    │
│  │  └──────────────┘  └──────────────┘       │     │                    │
│  └───────────────────┬───────────────────────┘     │                    │
│                      │                             │                    │
│                      ▼                             ▼                    │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    Resilience Layer                              │   │
│  │                                                                 │   │
│  │  ┌──────────────────────┐  ┌─────────────────────────────────┐  │   │
│  │  │   Circuit Breaker    │  │   Fallback (sold-out default)   │  │   │
│  │  │   (Closed/Open/Half) │  │                                 │  │   │
│  │  └──────────────────────┘  └─────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                      │                                                 │
│                      ▼                                                 │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                       Redis                                      │   │
│  │                                                                 │   │
│  │  product:{id}:stock     (STRING, atomic DECR)                   │   │
│  │  product:{id}:claims    (SET, SISMEMBER check)                  │   │
│  │  product:{id}:vouchers  (STRING, INCR counter)                  │   │
│  │  idempotency:{key}      (STRING, cached result)                 │   │
│  │  sale:config            (HASH, sale metadata)                   │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

## Running

```bash
# Start Redis
docker run -d -p 6379:6379 redis:7

# Run with solution code
cargo run --features solution

# Run tests
cargo test --features solution

# Initialize stock via Redis CLI
redis-cli SET product:prod-1:stock 1000

# Make a purchase
curl -X POST http://localhost:3000/purchase \
  -H 'Content-Type: application/json' \
  -H 'Idempotency-Key: unique-key-001' \
  -H 'X-Account-Id: user-123' \
  -d '{"product_id":"prod-1","account_id":"user-123","idempotency_key":"unique-key-001"}'

# Check stock
curl http://localhost:3000/stock/prod-1

# Health check
curl http://localhost:3000/health
```
