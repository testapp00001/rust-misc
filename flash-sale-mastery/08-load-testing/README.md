# Module 08: Load Testing and Capacity Planning

> You cannot ship a flash sale system without knowing its breaking point. Load
> testing reveals where the system bends, and capacity planning tells you how
> many servers you need before it breaks.

## Motivation

A flash sale that attracts 50,000 concurrent users is not the time to discover
your system caps out at 2,000 requests per second. Load testing answers the
critical questions: How many requests can we handle? At what point does latency
become unacceptable? Where is the bottleneck? How many instances do we need?

This module teaches you to build load testing tools in Rust, analyze the
results, and translate them into capacity plans using queuing theory.

## Concept Map

```
Scenario Selection
        |
        v
+---------------------+
| Scenario Orchestrator|  <-- p02: ramp-up, p03: spike, p04: sustained
+---------------------+
        |
        v
+---------------------+
| Load Generator      |  <-- p01: send HTTP requests at controlled rate
+---------------------+
        |
        v
+---------------------+
| Metrics Collector   |  <-- p05: record latency, errors, throughput
+---------------------+
        |
        v
+---------------------+
| Latency Analysis    |  <-- p06: percentiles, outliers, histograms
+---------------------+
        |
   +---------+---------+
   |                   |
   v                   v
+------------+  +-----------------+
| Capacity   |  | Bottleneck      |
| Model      |  | Identifier      |
| (p07)      |  | (p08)           |
+------------+  +-----------------+
   |                   |
   v                   v
 Instance         Fix the weakest
 Sizing           link first
```

## Theory

### Little's Law

The fundamental relationship in queuing theory:

```
L = lambda * W
```

Where:
- **L** = average number of concurrent requests in the system
- **lambda** = arrival rate (requests per second)
- **W** = average time a request spends in the system (latency)

Rearranged for capacity planning:
- **Required concurrency** = arrival_rate * average_latency
- **Max throughput** = max_concurrency / average_latency

### Percentile Math

Percentiles reveal what averages hide. A p99 of 500ms means 1 in 100 requests
takes half a second, even if the average is 50ms. For flash sales:

- **p50** (median): The typical user experience
- **p95**: The experience of 1 in 20 users (bad but tolerable)
- **p99**: The experience of 1 in 100 users (unacceptable for checkout)
- **p99.9**: The worst 1 in 1,000 (likely timeout or retry)

### Queuing Theory and Utilization

As a server approaches 100% utilization, latency does not increase linearly --
it increases hyperbolically. At 50% utilization, average latency is roughly 2x
the base latency. At 90%, it is roughly 10x. At 99%, it approaches infinity.
This is why you target 60-70% utilization in production.

### Load Test Scenarios

| Scenario | Shape | Use Case |
|----------|-------|----------|
| Ramp-up | Gradual increase | Find the inflection point where latency degrades |
| Spike | Instant jump | Test circuit breakers, admission control, auto-scaling |
| Sustained | Flat plateau | Detect memory leaks, connection pool exhaustion |

## Trade-offs

### Accuracy vs Load Generator Overhead

A load generator running on a single machine has its own bottlenecks: CPU for
TLS handshakes, file descriptor limits, ephemeral port exhaustion. If your
generator saturates before your target, the results are invalid.

- **Single machine**: Simple, but limited to ~10-50K concurrent connections
- **Distributed (e.g., Locust, k6)**: Accurate, but requires infrastructure
- **Rust-based**: Lower overhead per request than Python/JS generators

### Synthetic vs Realistic Traffic

- **Synthetic**: Constant rate, uniform endpoints -- easy to reproduce, does not
  reflect real user behavior
- **Realistic**: Variable rate, weighted endpoints, think-time between actions --
  harder to set up, better predictor of production behavior

### Open-Loop vs Closed-Loop

- **Open-loop**: Send requests at a fixed rate regardless of responses. Measures
  how the system behaves under a given load level.
- **Closed-loop**: Send a new request only when the previous one completes.
  Measures throughput but masks queuing effects.

## Failure Modes

1. **Generator Becomes the Bottleneck**: The load test machine runs out of CPU,
   memory, or file descriptors before the target system breaks. Monitor the
   generator itself.

2. **Coordinated Omission**: In closed-loop testing, if requests take 10x
   longer than expected, the generator sends 10x fewer requests, hiding the
   true load. Use open-loop or adjust for this.

3. **Test Environment Does Not Match Production**: Load testing against a
   single instance behind localhost gives meaningless results if production
   uses 8 instances behind a load balancer.

4. **Ignoring Warm-up**: JIT compilation, connection pool filling, and cache
   warming mean the first few seconds of a load test are not representative.
   Always include a warm-up period.

5. **Wrong Metric Focus**: Optimizing for throughput while ignoring tail latency
   can lead to a system that handles 10K rps but times out for 1% of users.

## Connection to Other Modules

- **Module 09 (Observability)**: Load test results feed into dashboards and
  alerting. The metrics you collect here (latency percentiles, error rates)
  are the same metrics you monitor in production.
- **Module 11 (Flash Sale API)**: The API endpoints you load test are the same
  ones users hit during a real sale. Load testing validates the API can handle
  the expected traffic.
- **Module 15 (Capstone Deployment)**: Capacity planning outputs (instance
  counts, resource limits) directly inform deployment configuration and
  Kubernetes resource requests/limits.

## Exercises

| # | Exercise | Focus |
|---|----------|-------|
| 01 | Load Generator | HTTP client, concurrency control, result aggregation |
| 02 | Ramp-up Scenario | Gradual load increase, tokio task spawning |
| 03 | Spike Scenario | Instant load burst, admission control testing |
| 04 | Sustained Load | Steady-state testing, resource leak detection |
| 05 | Metrics Collector | Concurrent recording, histogram aggregation |
| 06 | Latency Analysis | Percentile math, outlier detection, histograms |
| 07 | Capacity Model | Little's Law, utilization curves, instance sizing |
| 08 | Bottleneck Identifier | Decision logic, CPU/IO/DB/Redis bottleneck patterns |

## References

- [Little's Law - Wikipedia](https://en.wikipedia.org/wiki/Little%27s_law)
- [The Art of Application Performance Testing](https://www.oreilly.com/library/view/the-art-of/9781491900550/) by Ian Molyneaux
- [Systems Performance](https://www.brendangregg.com/systems-performance.html) by Brendan Gregg
- [Queuing Theory for System Design](https://ferd.ca/queues-don-t-have-load-averages.html) by Fred Hebert
- [hdrhistogram](https://hdrhistogram.org/) - High Dynamic Range Histogram
- [k6 Load Testing Tool](https://k6.io/) - Open-source load testing tool
