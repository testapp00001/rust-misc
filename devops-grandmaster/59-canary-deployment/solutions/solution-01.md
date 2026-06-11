# Solution 01: Canary vs Blue-Green Analysis

## Part A: Risk Exposure Comparison

| Strategy | Traffic to v2 | Requests/sec Affected | User Impact |
|----------|--------------|----------------------|-------------|
| Blue-green | 100% | 10,000 req/s | All users |
| Canary at 5% | 5% | 500 req/s | 1 in 20 users |
| Canary at 25% | 25% | 2,500 req/s | 1 in 4 users |

With 50 pods and blue-green, all 50 pods switch simultaneously. With
canary at 5%, approximately 2-3 pods run the new version while 47-48
pods run the stable version.

### Why This Matters

The blast radius of a bug is directly proportional to the traffic
percentage. At 5%, a bug in v2 affects 500 req/s -- enough to detect
statistically but not enough to cause a widespread outage.

## Part B: Feedback Speed Comparison

To detect a 10% error rate increase (from 2% to 12%) with 95% confidence:

| Strategy | Traffic to v2 | Time to Detect (statistical) | Users Affected Before Detection |
|----------|--------------|------------------------------|--------------------------------|
| Blue-green | 100% (10,000 req/s) | ~10 seconds (100 requests) | 10,000 users |
| Canary at 5% | 500 req/s | ~20 seconds (10 requests) | 500 users |
| Canary at 25% | 2,500 req/s | ~4 seconds (10 requests) | 250 users |

**Key insight:** Both strategies detect the problem quickly in absolute
time. The difference is how many users are affected during that detection
window. Blue-green affects 10,000 users in 10 seconds. Canary at 5%
affects 500 users in 20 seconds.

### Why This Works

Error rate detection requires a minimum number of requests to be
statistically significant. With 10% expected errors, you need roughly
10 requests to see 1 error. At 10,000 req/s, this takes milliseconds.
At 500 req/s, it takes ~20ms. Both are fast, but the blast radius
differs by 20x.

## Part C: Rollback Characteristics

| Aspect | Blue-Green | Canary at 5% | Canary at 25% |
|--------|-----------|--------------|---------------|
| **Rollback time** | < 1 second | < 1 second | < 1 second |
| **Users affected during rollback** | 100% (all users) | 5% (canary slice) | 25% (canary slice) |
| **In-flight requests** | Complete normally | Complete normally | Complete normally |
| **Rollback mechanism** | Switch Service selector | Set weight to 0 | Set weight to 0 |

All strategies have instant rollback because the stable version is still
running. The difference is how many users were on the bad version when
the bug was detected.

### Why This Works

In both blue-green and canary, the stable version's pods are still
running. Rollback is just redirecting traffic back to them. In-flight
requests to the canary pods complete normally because the pods are not
killed -- they just stop receiving new traffic.

## Part D: When to Use Each Strategy

### Scenario 1: Stateless API, any error rate increase unacceptable

**Best choice: Canary at 5%**

A 5% canary limits the blast radius to 500 req/s. If the error rate
increases by even 0.5%, the analysis detects it and rolls back before
widespread impact. Blue-green would expose all 10,000 req/s to the
error.

### Scenario 2: Subtle performance regressions

**Best choice: Canary with gradual progression (5% -> 25% -> 50% -> 100%)**

Performance regressions often only appear under load. A 5% canary might
not show the regression because the load is too low. Progressing through
weights lets you observe behavior at each load level. If p99 latency
increases at 25% but not at 5%, the analysis catches it at the 25% step.

### Scenario 3: Database schema migration requiring atomic switchover

**Best choice: Blue-green**

Database migrations (especially the expand-and-contract pattern from
Module 58) require both versions to coexist during the transition. A
canary at 5% means 5% of writes go to the new schema while 95% go to
the old schema. This creates data inconsistency. Blue-green lets you
switch all writes atomically.

### Scenario 4: UI redesign, testing user engagement

**Best choice: Canary (or A/B test)**

You want to observe real user behavior (click-through rate, time on page,
conversion rate) before committing to the new design. A canary at 5%
gives you real user data without risking the entire user base. If
engagement drops, you roll back with minimal impact.

## Common Mistakes to Avoid

- **Assuming canary is always better than blue-green.** Canary requires
  traffic splitting infrastructure (service mesh, ingress controller)
  and metric analysis (Prometheus, analysis templates). For simple
  services, blue-green's simplicity is an advantage.
- **Using canary without metric analysis.** A canary without automated
  analysis is just a slow blue-green. The value of canary is the
  feedback loop at each weight step.
- **Not considering the total time.** A canary with 5 steps and 5-minute
  pauses takes 25 minutes to reach 100%. Blue-green takes seconds. If
  you need fast deployments, canary's safety comes at the cost of speed.

## Key Takeaway

Canary deployment limits blast radius by exposing the new version to a
small percentage of traffic first. It provides faster feedback with less
risk than blue-green, but takes longer to reach 100% and requires more
infrastructure. The right strategy depends on the cost of errors, the
need for real-user feedback, and the complexity of the deployment.
