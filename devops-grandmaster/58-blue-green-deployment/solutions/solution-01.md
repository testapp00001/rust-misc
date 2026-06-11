# Solution 01: Blue-Green vs Rolling Update

## Part A: Analyze the Rolling Update Failure

With `maxSurge: 2` and `maxUnavailable: 2`, here is what happens when v2
crashes on startup:

| Step | Action | Healthy Pods | Unhealthy Pods | Total |
|------|--------|-------------|----------------|-------|
| 0 | Initial state | 6 (v1) | 0 | 6 |
| 1 | Create 2 surge pods (v2) | 6 (v1) | 2 (v2 starting) | 8 |
| 2 | v2 pods crash | 6 (v1) | 2 (v2 crashing) | 8 |
| 3 | Kill 2 old v1 pods (maxUnavailable=2) | 4 (v1) | 2 (v2 crashing) | 6 |
| 4 | Create 2 new v2 pods | 4 (v1) | 4 (v2 crashing) | 8 |
| 5 | New v2 pods crash | 4 (v1) | 4 (v2 crashing) | 8 |

At step 3, the cluster drops from 6 healthy pods to 4. That is a 33%
capacity reduction. At 1,000 req/s, 667 req/s now hit 4 pods instead of
6. Each pod goes from 167 req/s to 250 req/s, a 50% increase in load
per pod.

If the remaining v1 pods cannot handle 250 req/s each, they start failing
too, creating a cascading failure. The cluster never recovers because
Kubernetes keeps trying to create v2 pods that keep crashing.

### Why This Matters

The rolling update strategy assumes the new version starts successfully.
When it does not, `maxUnavailable` controls how many healthy pods you lose
before the deployment is rolled back. With `maxUnavailable: 2`, you lose
33% of capacity during the failure window.

## Part B: Compare with Blue-Green

In blue-green deployment:

| Step | Action | Blue (v1) Pods | Green (v2) Pods | Traffic |
|------|--------|----------------|-----------------|---------|
| 0 | Initial state | 6 healthy | 0 | Blue |
| 1 | Deploy v2 to green | 6 healthy | 6 starting | Blue |
| 2 | v2 pods crash | 6 healthy | 6 crashing | Blue |
| 3 | Health check fails | 6 healthy | 6 crashing | Blue |
| 4 | Do not switch | 6 healthy | 6 crashing | Blue |

**Zero user impact.** Blue continues serving all 1,000 req/s with 6
healthy pods throughout the entire process. Green's failure is invisible
to users.

**Rollback:** Not needed -- traffic was never switched. The green
environment is simply left in a failed state for debugging.

### Why This Works

Blue-green separates deployment from traffic switching. The new version
is fully deployed and tested before any user sees it. If deployment fails,
nothing changes for users.

## Part C: Resource Cost Analysis

| Strategy | CPU During Deployment | Memory During Deployment |
|----------|----------------------|-------------------------|
| Rolling update (maxSurge=2) | 8 x 500m = 4,000m (4 cores) | 8 x 512Mi = 4,096Mi |
| Blue-green | 12 x 500m = 6,000m (6 cores) | 12 x 512Mi = 6,144Mi |

Blue-green uses 50% more resources during deployment. However:

- Rolling update peak duration: ~5 minutes (pods cycle one at a time)
- Blue-green peak duration: ~5 minutes (until old environment is scaled down)

**Cost comparison** (assuming $0.05/core-hour):
- Rolling update extra cost: 2 cores x 5 min = $0.008 per deployment
- Blue-green extra cost: 6 cores x 5 min = $0.025 per deployment

The difference is $0.017 per deployment. If downtime costs $10,000/minute
and the blue-green strategy prevents even 1 minute of downtime per 590
deployments, it pays for itself.

## Part D: Choose the Right Strategy

### Scenario 1: Stateless API, 100 req/s, cost-sensitive

**Best choice: Rolling update**

The service is stateless, so partial unavailability during rollout is
acceptable. The cost-sensitive cluster cannot afford double resources.
If v2 fails, the rolling update auto-reverts after the progress deadline.

### Scenario 2: Payment service, 10,000 req/s, $10,000/min downtime

**Best choice: Blue-green**

The cost of downtime ($10,000/minute) far exceeds the cost of extra
resources. Blue-green guarantees zero user impact during deployment and
instant rollback. The extra $0.025 per deployment is negligible.

### Scenario 3: Batch processing, runs once per hour

**Best choice: Rolling update (or even stop-and-start)**

The batch job is not serving real-time traffic. If it fails during
deployment, the next hourly run picks up the work. Blue-green is
overkill for non-user-facing workloads.

### Scenario 4: Real-time trading system

**Best choice: Blue-green**

Any error during deployment could cause financial loss. Blue-green
provides zero-downtime deployment and instant rollback. The resource
cost is irrelevant compared to the risk of a single failed trade.

## Common Mistakes to Avoid

- **Assuming rolling updates are always safe.** The `maxUnavailable`
  setting directly controls how much capacity you lose during a failed
  deployment. Setting it too high can cause cascading failures.
- **Ignoring resource costs of blue-green.** Running two full environments
  is expensive. For non-critical services, the cost may not be justified.
- **Not considering database state.** Blue-green works perfectly for
  stateless services, but shared databases create complications (see
  Exercise 04).

## Key Takeaway

The choice between blue-green and rolling update is a risk/cost trade-off.
Blue-green trades resource cost for zero-downtime safety. Rolling update
trades safety for resource efficiency. The right choice depends on the
cost of downtime relative to the cost of infrastructure.
