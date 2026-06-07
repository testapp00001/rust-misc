# Module 3: Logging & Tracing

Build production-grade observability with the tracing ecosystem. Learn
structured logging, span-based context propagation, subscriber configuration,
error tracing, metrics integration, distributed tracing with OpenTelemetry,
and production observability patterns.

## Lesson Index

| # | File | Topic |
|---|------|-------|
| 1 | `p01_tracing_fundamentals.rs` | tracing crate overview, Subscriber trait, global dispatcher |
| 2 | `p02_spans_events.rs` | Creating spans, entering spans, events, fields, span lifecycle |
| 3 | `p03_structured_logging.rs` | Structured fields, Debug/Display formatting, custom field types |
| 4 | `p04_subscriber_config.rs` | tracing-subscriber layers, fmt layer, filter layer, env-filter |
| 5 | `p05_error_tracing.rs` | Tracing errors, span context for errors, error events, tracing::error! |
| 6 | `p06_debug_techniques.rs` | Debug tracing, conditional tracing, trace-level debugging, inspection |
| 7 | `p07_log_filtering.rs` | Env-filter directives, per-module filtering, dynamic filtering, reload |
| 8 | `p08_metrics_integration.rs` | metrics crate, counters, gauges, histograms, tracing-metrics bridge |
| 9 | `p09_distributed_tracing.rs` | OpenTelemetry, trace propagation, span context, B3/W3C headers |
| 10 | `p10_production_observability.rs` | Log aggregation, alerting, dashboards, structured JSON output, log levels |

## Running

```bash
cargo test -p logging_tracing
```
