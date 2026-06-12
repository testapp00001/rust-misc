# Module 15: Capstone — Deployment

> The flash sale system is built. Now make it production-ready: containerize it,
> deploy it to Kubernetes, set up monitoring, and write the operational scripts
> that keep it alive during the sale.

## Motivation

Modules 1 through 14 built every piece of the flash sale puzzle — Redis
primitives, Lua scripts, atomic counters, traffic shaping, idempotency, event
sourcing, resilience, load testing, observability, caching, the API, the order
worker, reconciliation, and integration tests. None of that matters if the
system cannot be deployed, scaled, monitored, and operated under pressure.

Deployment is the final exam. A flash sale has one shot: the sale window is
minutes long, traffic spikes are extreme, and there is no "we'll fix it next
sprint." This module teaches you to package everything into containers, run it
on Kubernetes with autoscaling, watch it through Grafana dashboards, and operate
it with battle-tested scripts.

## Concept Map

```
                    ┌─────────────────────────────────────────┐
                    │          Deployment Pipeline            │
                    └─────────────────────────────────────────┘
                                      │
          ┌───────────────────────────┼───────────────────────────┐
          v                           v                           v
  ┌───────────────┐         ┌─────────────────┐         ┌─────────────────┐
  │   Docker      │         │   Kubernetes    │         │   Monitoring    │
  │   Compose     │         │   (Production)  │         │   Stack         │
  │   (Dev)       │         │                 │         │                 │
  │               │         │  Deployment     │         │  Prometheus     │
  │  redis        │         │  Service        │         │  Grafana        │
  │  postgres     │         │  HPA            │         │  Alerts         │
  │  api          │         │  ConfigMap      │         │                 │
  │  worker       │         │                 │         │                 │
  │  prometheus   │         └────────┬────────┘         └────────┬────────┘
  │  grafana      │                  │                           │
  └───────┬───────┘                  │                           │
          │                          v                           v
          │                 ┌─────────────────┐         ┌─────────────────┐
          └────────────────>│  Operational    │<────────│  Dashboards &   │
                            │  Scripts        │         │  Alerts         │
                            │                 │         │                 │
                            │  pre-sale       │         │  stock gauge    │
                            │  load-test      │         │  req/sec        │
                            │  reconciliation │         │  latency p50/   │
                            │                 │         │    p95/p99      │
                            └─────────────────┘         │  redis latency  │
                                                        │  connections    │
                                                        └─────────────────┘
```

## Theory

### Multi-Stage Docker Builds

A Rust release binary is self-contained, but the build toolchain is 1+ GB. A
multi-stage build separates the build environment from the runtime:

```
Stage 1 (builder):  rust:1.82-bookworm  →  cargo build --release  →  1.5 GB
Stage 2 (runtime):  debian:bookworm-slim + binary  →  ~80 MB
```

The key insight: only copy the artifact, not the toolchain. This also means the
runtime image has no compiler, no source code, and a minimal attack surface.

**Dependency caching trick**: Copy `Cargo.toml` and `Cargo.lock` first, create a
dummy `src/main.rs`, build once to cache dependencies, then copy the real source
and rebuild. Only source changes trigger a full rebuild.

### Kubernetes Autoscaling

The Horizontal Pod Autoscaler (HPA) watches metrics and adjusts replica count:

- **CPU-based scaling**: When average CPU utilization exceeds 70%, add pods.
- **Custom metrics**: Scale on `http_requests_per_second` for traffic-aware scaling.
- **Scaling behavior**: Fast scale-up (30s stabilization), slow scale-down (5min)
  to avoid flapping during a flash sale's traffic spikes.

For a flash sale, pre-scaling is essential — reactive scaling is too slow for
the 0-to-100K RPS ramp that happens in seconds.

### Monitoring Stack

```
Application  ──metrics──>  Prometheus  ──query──>  Grafana
     │                        │
     └──logs──────────────────┴──>  Alertmanager  ──>  PagerDuty / Slack
```

- **Prometheus** scrapes `/metrics` endpoints every 15s and stores time-series data.
- **Grafana** queries Prometheus and renders dashboards (stock, RPS, latency).
- **Alert rules** fire when thresholds are breached (error rate, latency, stock).

## Trade-offs

| Dimension | Docker Compose (Dev) | Kubernetes (Prod) |
|---|---|---|
| Setup complexity | Low — single file | High — manifests, cluster, registry |
| Scaling | Manual — change `replicas` in YAML | Automatic — HPA adjusts in real-time |
| Networking | Docker bridge DNS | ClusterIP, Ingress, service mesh |
| Secrets | Env vars in compose file | K8s Secrets or external vault |
| Rolling updates | `docker-compose up --build` | `kubectl rollout` with zero downtime |
| Local development | Fast iteration | Requires minikube/kind/Docker Desktop |

**When to use which**: Docker Compose for local development, integration testing,
and demos. Kubernetes for production, staging, and any environment that needs
autoscaling, self-healing, or multi-node deployment.

## Failure Modes

### Pod Crashes

- **OOMKilled**: The binary exceeded its memory limit. Increase `resources.limits.memory`
  or investigate memory leaks (common with unbounded connection pools).
- **CrashLoopBackOff**: The app crashes on startup. Check `kubectl logs` — likely a
  missing ConfigMap value or unreachable Redis/Postgres.
- **Mitigation**: `startupProbe` gives the app time to initialize before liveness
  checks start killing it.

### Scaling Delays

- HPA reacts to metrics with a 15-30s delay (Prometheus scrape interval + HPA sync).
- Pod startup takes 5-15s for a Rust binary.
- **Mitigation**: Pre-scale before the sale starts using `scripts/pre-sale-warmup.sh`.

### Monitoring Gaps

- Prometheus scrape interval (15s) means you can miss short-lived spikes.
- Grafana dashboards only show what you measure — if a metric is not exported,
  you are blind.
- **Mitigation**: Use recording rules for fast-moving counters, export RED metrics
  (Rate, Errors, Duration) for every endpoint.

### Network Partitions

- Redis or Postgres becoming unreachable causes cascading failures.
- **Mitigation**: Circuit breakers (module 07) prevent thundering herds; retry with
  backoff; degrade gracefully (return 503 instead of hanging).

## Connection to Other Modules

| Module | What it provides | How deployment uses it |
|---|---|---|
| 01-redis-fundamentals | Redis data structures | Redis container in compose/k8s |
| 02-redis-lua-scripting | Atomic Lua operations | Runs inside Redis — no deployment change |
| 03-atomic-counters | Stock decrement logic | Metrics: `flash_sale_stock_remaining` |
| 04-traffic-shaping | Rate limiting | ConfigMap: `RATE_LIMIT_RPS` |
| 05-idempotency | Duplicate prevention | Runs in the API — containerized |
| 06-event-sourcing | Event log | Postgres volume in compose/k8s |
| 07-resilience | Circuit breakers | Health probes + retry config |
| 08-load-testing | Load test harness | `scripts/load-test.sh` drives it |
| 09-observability | Metrics + tracing | Prometheus scrape + Grafana dashboards |
| 10-caching-strategy | Cache warming | `scripts/pre-sale-warmup.sh` |
| 11-flash-sale-api | HTTP API server | `flash-sale-api` container/pod |
| 12-order-worker | Background worker | `order-worker` container/pod |
| 13-reconciliation | Post-sale audit | `scripts/reconciliation.sh` |
| 14-integration-tests | End-to-end tests | Run in CI against compose stack |

## Deployment Checklist

### Pre-Sale (T-24h)

- [ ] Docker image built and pushed to registry
- [ ] Kubernetes manifests applied (deployment, service, HPA, configmap)
- [ ] Prometheus and Grafana are running and scraping metrics
- [ ] Alert rules loaded in Alertmanager
- [ ] Run integration tests against the staging environment
- [ ] Verify database migrations are applied

### Pre-Sale (T-1h)

- [ ] Run `scripts/pre-sale-warmup.sh` to warm cache and scale pods
- [ ] Verify all pods are healthy (readiness probes passing)
- [ ] Check Grafana dashboards — stock levels, connections, latency baseline
- [ ] Confirm HPA min replicas match expected traffic

### During Sale (T-0 to T+15min)

- [ ] Monitor Grafana dashboard in real-time
- [ ] Watch for `oversell_detected` alerts (CRITICAL)
- [ ] Watch for `error_rate_exceeded` and `latency_exceeded` alerts
- [ ] Be ready to manually scale if HPA reacts too slowly
- [ ] Keep communication channel open with the team

### Post-Sale (T+15min)

- [ ] Run `scripts/reconciliation.sh` to close the sale and audit orders
- [ ] Review reconciliation report for oversells or data inconsistencies
- [ ] Check final stock levels in Grafana
- [ ] Scale down pods to baseline
- [ ] Archive logs and metrics for post-mortem

### Post-Mortem (T+24h)

- [ ] Review load test results vs actual traffic
- [ ] Identify bottlenecks (CPU, memory, Redis latency, DB connections)
- [ ] Update alert thresholds based on observed baselines
- [ ] Document lessons learned
- [ ] Plan improvements for the next sale
