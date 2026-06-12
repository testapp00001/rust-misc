# Flash Sale Mastery

> Build a production-grade flash sale system in Rust that handles Black Friday traffic without data races, overselling, or downtime.

## What You'll Build

A complete e-commerce flash sale system capable of handling 100K+ concurrent requests with:

- **Atomic stock management** — provably cannot oversell
- **Per-product voucher limits** — each product has its own claim cap
- **Per-account limits** — each user can only claim once per product
- **Idempotent purchase flow** — duplicate requests are safely deduplicated
- **Multi-layer rate limiting** — from reverse proxy to Redis
- **Real-time observability** — distributed tracing, metrics, dashboards
- **Resilience patterns** — circuit breaker, bulkhead, fallback
- **Event sourcing** — complete audit trail of every stock change
- **Reconciliation** — automatic sync between Redis and database

## Prerequisites

You should have completed these learning projects before starting:

| Project | Skills You Need |
|---------|----------------|
| rust-dev-mastery | Async Rust, Axum, concurrency, database, tracing |
| rust-security-lab | Rate limiting, input validation, audit logging |
| devops-grandmaster | Load balancing, caching, distributed systems |
| rust-DSA | Algorithmic thinking, data structures |

## Architecture Overview

```
                    ┌─────────────────┐
                    │   Load Balancer  │
                    │   (Nginx/HAProxy)│
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │   Flash Sale API │
                    │   (Axum + Tower) │
                    │                  │
                    │ ┌──────────────┐ │
                    │ │Rate Limiter  │ │
                    │ │Idempotency   │ │
                    │ │Tracing       │ │
                    │ └──────────────┘ │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
     ┌────────▼───────┐ ┌───▼────┐ ┌───────▼───────┐
     │  Redis (Stock)  │ │Postgres│ │  Order Queue  │
     │  Lua Scripts    │ │ (DB)   │ │ (Redis Stream)│
     │  Atomic Ops     │ │        │ │               │
     └────────────────┘ └────────┘ └───────┬───────┘
                                           │
                                  ┌────────▼────────┐
                                  │  Order Worker    │
                                  │  (Async Consumer) │
                                  └─────────────────┘
```

## Module Guide

### Phase 1: Foundation (Modules 01-03)
Learn Redis as an atomic coordination layer. These modules teach the core technology that makes sub-millisecond stock operations possible.

### Phase 2: Traffic Control (Modules 04-05)
Learn to shape, limit, and deduplicate traffic before it reaches your stock service.

### Phase 3: Data Integrity (Modules 06-07)
Learn event sourcing for audit trails and resilience patterns for graceful degradation.

### Phase 4: Validation (Modules 08-10)
Learn load testing, observability, and caching strategies to validate and optimize the system.

### Phase 5: Integration (Modules 11-13)
Build the complete API, order worker, and reconciliation service.

### Phase 6: Production (Modules 14-15)
Integration tests and deployment with Docker, Kubernetes, and monitoring.

## Quick Start

```bash
# Build the entire workspace
cargo build

# Run tests for a specific module
cargo test -p redis-fundamentals

# Run with solution code
cargo test -p redis-fundamentals --features solution

# Run the flash sale API
cargo run -p flash-sale-api

# Run integration tests (requires Docker for Redis + Postgres)
cargo test -p integration-tests
```

## Project Structure

```
flash-sale-mastery/
├── 01-redis-fundamentals/     # Redis data types and operations
├── 02-redis-lua-scripting/    # Atomic Lua scripts for stock ops
├── 03-atomic-counters/        # Distributed counter patterns
├── 04-traffic-shaping/        # Rate limiting and admission control
├── 05-idempotency/            # Exactly-once purchase semantics
├── 06-event-sourcing/         # Audit trail and event replay
├── 07-resilience/             # Circuit breaker, bulkhead, fallback
├── 08-load-testing/           # Load generation and capacity planning
├── 09-observability/          # Tracing, metrics, dashboards
├── 10-caching-strategy/       # Cache warming and invalidation
├── 11-flash-sale-api/         # Complete API (integration module)
├── 12-order-worker/           # Async order processing
├── 13-reconciliation/         # Redis ↔ DB sync
├── 14-integration-tests/      # End-to-end tests
└── 15-capstone-deployment/    # Docker, K8s, monitoring
```

## Success Criteria

After completing all modules, you can:

1. Design a flash sale architecture from scratch
2. Implement atomic stock management that provably cannot oversell
3. Handle 100K+ concurrent requests without degradation
4. Implement idempotent purchase flows
5. Design multi-layer rate limiting
6. Set up observability for real-time monitoring
7. Write load tests that validate system capacity
8. Design reconciliation for data consistency
9. Implement resilience patterns
10. Deploy with Kubernetes and monitoring
