# Module 09: Observability

## Phase: Validation

## Motivation

When 500,000 requests hit your flash sale system in 60 seconds and something goes
wrong, you have minutes -- sometimes seconds -- to diagnose and respond. Without
proper observability, you are flying blind. Structured tracing tells you *where*
the failure is, metrics tell you *how bad* it is, and dashboards tell you *when*
it started. This module builds the observability stack that turns a catastrophic
incident into a manageable situation.

## Concept Map

```
Tracing          Metrics           Alerts           Dashboards
   |                |                |                  |
   v                v                v                  v
Spans & Events   Counters &      Threshold Rules    Panels &
Context Prop.    Histograms      Severity Levels    Queries
   |                |                |                  |
   +--------+-------+--------+-------+------------------+
            |                |
            v                v
     Prometheus          Grafana
   Exposition Format   Visualization
```

## Theory

### Distributed Tracing (OpenTelemetry)

A **trace** represents a single request flowing through the system. Each service
creates **spans** -- timed operations with metadata. Spans form a tree via
parent-child relationships. The W3C Trace Context standard propagates trace IDs
across service boundaries using the `traceparent` header:

```
traceparent: {version}-{trace_id}-{parent_id}-{trace_flags}
```

- **trace_id**: 128-bit globally unique identifier
- **parent_id**: 64-bit span identifier of the parent
- **trace_flags**: 8-bit field (01 = sampled)

### Prometheus Metrics

Prometheus scrapes metrics in a text-based exposition format. Three metric types:

- **Counter**: Monotonically increasing value (e.g., total requests)
- **Gauge**: Value that can go up or down (e.g., current stock level)
- **Histogram**: Distribution of values in configurable buckets (e.g., latency)

Labels add dimensions: `purchase_attempts_total{result="success"} 42`

### Observability Pillars

1. **Logs**: Discrete events with structured fields (who, what, when)
2. **Metrics**: Aggregated numerical measurements over time
3. **Traces**: Request flow through distributed services

Together they answer: "What happened?" (logs), "How is the system?" (metrics),
"Where did it go wrong?" (traces).

## Trade-offs

### Sampling Strategies

| Strategy | Pros | Cons |
|----------|------|------|
| **Probabilistic** | Simple, uniform | May miss rare errors |
| **Rate-limiting** | Predictable cost | Loses burst visibility |
| **Tail-based** | Captures all errors | Requires buffering, higher latency |
| **Head-based** | Low overhead | May drop interesting traces |

### Observability Overhead

- Tracing adds ~1-5% latency per span
- High-cardinality metrics labels can explode memory
- Structured logging produces 2-5x more bytes than plain text
- Solution: sample traces, limit label cardinality, use async logging

### Metrics Cardinality

`product_id` as a label is fine for 100 products. `user_id` as a label with
1M users creates 1M time series -- this will crash Prometheus. Always bound
label cardinality.

## Failure Modes

1. **Observability overhead becomes the bottleneck**: Too many spans/metrics
   degrade the system you are trying to monitor
2. **Cardinality explosion**: Unbounded labels fill Prometheus memory
3. **Trace context loss**: Missing propagation breaks distributed traces
4. **Alert fatigue**: Too many alerts cause operators to ignore them
5. **Clock skew**: Distributed traces assume synchronized clocks
6. **Metric gaps**: If your monitoring system goes down during the incident,
   you lose the data you need most

## Connection to Other Modules

- **01-redis-fundamentals**: Redis operations are traced and timed
- **02-redis-lua-scripting**: Lua script execution metrics
- **03-atomic-counters**: Counter operations feed into metrics
- **04-traffic-shaping**: Rate limiter decisions are traced
- **05-idempotency**: Duplicate detection events are logged
- **06-event-sourcing**: Event stream is a trace source
- **07-resilience**: Circuit breaker state changes are metric events
- **08-load-testing**: Load test results validate observability coverage
- **10-caching-strategy**: Cache hit/miss ratios are key metrics
- **11-flash-sale-api**: API layer produces request traces
- **12-order-worker**: Worker processing is traced end-to-end
- **13-reconciliation**: Discrepancy detection generates alerts
- **14-integration-tests**: Tests validate observability instrumentation
- **15-capstone-deployment**: Production monitoring setup

## Exercises

| # | Exercise | Description |
|---|----------|-------------|
| 01 | Tracing Setup | Configure structured logging with `tracing` |
| 02 | Span Propagation | W3C trace context injection and extraction |
| 03 | Custom Metrics | Prometheus counters, gauges, histograms |
| 04 | Redis Metrics | Redis-specific operation metrics |
| 05 | Business Metrics | Revenue, conversion, active user tracking |
| 06 | Alert Rules | Define and evaluate threshold-based alerts |
| 07 | Dashboard Spec | Rust structs defining monitoring dashboards |
| 08 | Log Analysis | Parse, filter, and aggregate structured logs |

## References

- [OpenTelemetry Specification](https://opentelemetry.io/docs/specs/)
- [W3C Trace Context](https://www.w3.org/TR/trace-context/)
- [Prometheus Data Model](https://prometheus.io/docs/concepts/data_model/)
- [tracing crate documentation](https://docs.rs/tracing/)
- [prometheus crate documentation](https://docs.rs/prometheus/)
- [Google SRE Book - Monitoring](https://sre.google/sre-book/practical-alerting/)
