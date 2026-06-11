# Module 40: Distributed Tracing — Exercises

## Overview

Distributed tracing allows you to follow a single request as it travels through multiple microservices, revealing latency bottlenecks, error origins, and service dependencies. This module covers Jaeger, OpenTelemetry, trace context propagation, spans, and trace analysis.

## Prerequisites

- Python 3.9+
- Docker and Docker Compose
- Kubernetes cluster (minikube or kind) for Exercise 05
- Basic understanding of microservices architecture
- Familiarity with HTTP and gRPC concepts

## Exercises

| Exercise | Type | Topic | Estimated Time |
|----------|------|-------|----------------|
| 01 | Conceptual | Distributed tracing fundamentals (traces, spans, context propagation) | 30 min |
| 02 | Guided | Instrument a Python app with OpenTelemetry | 45 min |
| 03 | Independent | Trace a request across multiple microservices | 60 min |
| 04 | Challenge | Analyze traces to find performance bottlenecks | 60 min |
| 05 | Integration | Set up Jaeger with Kubernetes and correlate with metrics | 90 min |

## Getting Started

Work through the exercises in order. Each exercise builds on concepts from the previous one. Solutions are available in the `solutions/` directory — attempt each exercise before reviewing the solution.

## Environment Setup

Install the base dependencies before starting:

```bash
pip install opentelemetry-api \
    opentelemetry-sdk \
    opentelemetry-exporter-jaeger \
    opentelemetry-exporter-otlp \
    opentelemetry-instrumentation-flask \
    opentelemetry-instrumentation-requests \
    opentelemetry-instrumentation-sqlalchemy \
    flask \
    requests \
    sqlalchemy
```

For Docker-based Jaeger:

```bash
docker pull jaegertracing/all-in-one:1.51
```
