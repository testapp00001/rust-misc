# Module 07: Resilience Patterns

## Motivation

During Black Friday, your downstream services **will** fail. Redis will timeout under
load. The payment gateway will return 503s. The notification service will backlog.
The database connection pool will exhaust. These are not edge cases -- they are
certainties.

The question is not whether failures happen, but how your system responds. A system
without resilience patterns will cascade: one slow service blocks threads, which
exhausts the connection pool, which causes timeouts across all services, which takes
down the entire platform. A system with resilience patterns degrades gracefully:
failing services are isolated, fast-failed, and recovered automatically.

This module teaches you to build systems that survive the storm.

## Concept Map

```
Request Flow with Resilience Patterns
======================================

  Client Request
       |
       v
  +------------------+
  | Timeout Budget   |  <-- Never wait forever
  | (overall: 8s)    |
  +--------+---------+
           |
           v
  +------------------+
  | Circuit Breaker  |  <-- Stop calling broken services
  | (threshold: 5)   |
  +--------+---------+
           |
           v
  +------------------+
  | Bulkhead         |  <-- Isolate failures per operation
  | (max: 50 slots)  |
  +--------+---------+
           |
           v
  +------------------+
  | Retry + Backoff  |  <-- Recover from transient failures
  | (max: 3 retries) |
  +--------+---------+
           |
           v
  +------------------+
  | Fallback         |  <-- Graceful degradation
  | (cached/default) |
  +--------+---------+
           |
           v
      Response
```

## Theory

### Circuit Breaker

Like an electrical circuit breaker, this pattern monitors failure rates. When failures
exceed a threshold, the "circuit opens" and all requests are rejected immediately
without calling the downstream service. After a recovery timeout, one "probe" request
is allowed through. If it succeeds, the circuit closes; if it fails, it reopens.

```
State Diagram:
  Closed ──(failures >= threshold)──> Open
    ^                                   |
    |                                   v
    └──(probe success)── HalfOpen <──(timeout elapsed)
                           |
                           └──(probe failure)──> Open
```

### Bulkhead

Named after ship bulkheads that prevent a hull breach from flooding the entire vessel.
Each operation type (stock check, order creation, notification) gets its own concurrency
limit. If the notification service is overloaded, it cannot steal connections from
order creation.

### Retry with Exponential Backoff

Transient failures (network blips, brief overloads) often resolve on their own. Retry
logic automatically retries failed operations. Exponential backoff increases delay
between retries (100ms, 200ms, 400ms...) to avoid overwhelming a recovering service.
Jitter randomizes delays to prevent the "thundering herd" problem where thousands of
clients retry simultaneously.

### Timeout Management

Every call to a downstream service must have a bounded execution time. Different
operations have different latency expectations:
- Redis: 50ms (in-memory, should be fast)
- Database: 500ms (disk I/O, moderate)
- Payment: 5s (external API, slow but bounded)

Cascading timeouts ensure that inner operations timeout before outer ones.

### Fallback Strategies

When a primary operation fails, a fallback provides an alternative response:
- Stock check fails -> "temporarily unavailable"
- DB write fails -> queue for later processing
- Cache miss -> serve stale data

The key insight: a degraded response is better than no response.

## Trade-offs

### Aggressive vs Conservative Thresholds

| Parameter | Aggressive | Conservative |
|-----------|-----------|--------------|
| Circuit breaker threshold | 3 failures | 10 failures |
| Recovery timeout | 5s | 60s |
| Max retries | 5 | 2 |
| Base retry delay | 50ms | 500ms |
| Redis timeout | 20ms | 100ms |

**Aggressive** thresholds detect problems faster but may trip on transient spikes.
**Conservative** thresholds allow more failures through but provide better availability
during minor issues.

### Timeout Budget Allocation

With an overall 8s timeout, how do you allocate?
- Stock (50ms) + DB (500ms) + Payment (5s) = 5.55s total, leaving 2.45s buffer
- If you allocate too tightly, a slow stock check eats into payment time
- If you allocate too loosely, the user waits too long for a doomed request

## Failure Modes

### Cascading Failure

```
  Payment Service slows down
         |
         v
  Order threads block on payment timeout (5s)
         |
         v
  Thread pool exhausted (all 200 threads waiting)
         |
         v
  Stock check requests queue up (no threads available)
         |
         v
  ALL requests timeout, not just payment
         |
         v
  Load balancer marks instance unhealthy
         |
         v
  Traffic shifts to other instances (which also fail)
         |
         v
  Complete platform outage
```

**Prevention**: Circuit breaker on payment + bulkhead isolating payment from stock +
short timeouts to fail fast.

### Resource Exhaustion

Without bulkheads, a surge in non-critical operations (notifications) can consume all
available connections, starving critical operations (order creation). The system appears
"busy" but isn't doing useful work.

**Prevention**: Bulkheads with separate concurrency limits per operation type.

## Connections to Other Modules

- **Module 02 (Redis Lua Scripting)**: Lua scripts in Redis are atomic, but the
  connection can still fail. Resilience patterns wrap Redis operations with retry,
  timeout, and circuit breaker.

- **Module 04 (Traffic Shaping)**: Traffic shaping prevents overload at the entry
  point. Resilience patterns handle failures that occur deeper in the system. Together,
  they form a defense-in-depth strategy.

- **Module 11 (Flash Sale API)**: The API layer is where all resilience patterns are
  composed together. Each incoming request flows through the full resilience stack
  before reaching downstream services.

## Exercise List

| # | Exercise | Pattern | Key Concept |
|---|----------|---------|-------------|
| 01 | Circuit Breaker | State machine | Fail-fast when service is down |
| 02 | Bulkhead | Concurrency isolation | Prevent cascade between operation types |
| 03 | Retry with Backoff | Transient recovery | Exponential backoff with jitter |
| 04 | Timeout Management | Bounded latency | Cascading timeout budgets |
| 05 | Fallback Strategies | Graceful degradation | Alternative responses on failure |
| 06 | Graceful Shutdown | Clean exit | Drain in-flight requests |
| 07 | Health Check | Observability | Deep dependency probing |
| 08 | Failure Injection | Testing | Verify resilience in tests |

## Running

```bash
# Run exercise stubs (tests will fail with todo!())
cargo test -p resilience

# Run completed solutions
cargo test -p resilience --features solution

# Run a specific exercise's tests
cargo test -p resilience --features solution p01_circuit_breaker
```

## References

- *Release It!* by Michael T. Nygard -- the definitive guide to resilience patterns
- *Designing Data-Intensive Applications* by Martin Kleppmann -- fault tolerance theory
- Netflix Hystrix documentation -- the library that popularized circuit breakers
- AWS Architecture Blog: "Using bulkheads with microservices"
- Google SRE Book: "Managing Overload" and "Addressing Cascading Failures"
