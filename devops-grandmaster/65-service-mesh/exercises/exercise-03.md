# Exercise 03: Traffic Management

**Type:** Independent
**Time:** 35 min
**Difficulty:** Medium

## Objective

Configure traffic management policies in a service mesh for canary deployments, traffic splitting, retries, timeouts, and circuit breaking.

## Scenario

You are managing traffic for an API service with the following deployment:

```
Current deployment:
  api-service v1 (stable) → 5 replicas
  api-service v2 (canary) → 1 replica (new version, testing)

Requirements:
- Route 90% of traffic to v1, 10% to v2
- If v2 returns errors, automatically shift traffic back to v1
- Retry failed requests up to 3 times with exponential backoff
- Timeout requests after 10 seconds
- Circuit breaker: open after 5 consecutive errors, half-open after 30s
```

## Tasks

### Part A: Canary Deployment

Write the Istio VirtualService and DestinationRule for the canary deployment:

1. Define subsets for v1 and v2 based on version labels
2. Route 90% traffic to v1, 10% to v2
3. Ensure the traffic split is based on weight, not random

```yaml
# Your Istio configuration here
```

<details>
<summary>Hint</summary>

Use `VirtualService` with `http.route.weight` to split traffic. Use `DestinationRule` with `subsets` to define v1 and v2 based on pod labels. The weight is relative (90:10 means 90% and 10%).

</details>

### Part B: Retries and Timeouts

Configure retry and timeout policies:

1. Timeout: 10 seconds per request
2. Retries: up to 3 attempts on 5xx errors
3. Retry backoff: 25ms, 50ms, 100ms (exponential)
4. Do not retry on 4xx errors (client errors are not transient)

<details>
<summary>Hint</summary>

In Istio VirtualService, use `timeout` for request timeout and `retries` for retry policy. The `retryOn` field specifies which conditions trigger retries (e.g., `5xx,reset,connect-failure`). Retries should only be for idempotent operations.

</details>

### Part C: Circuit Breaker

Design a circuit breaker configuration:

1. Track consecutive errors per service instance
2. Open the circuit after 5 consecutive errors
3. Keep the circuit open for 30 seconds
4. Half-open: allow 1 request through to test recovery
5. If the test request succeeds, close the circuit

Write the configuration and explain how it differs from retries.

<details>
<summary>Hint</summary>

In Istio, circuit breaking is configured in `DestinationRule` using `outlierDetection`. The `consecutive5xxErrors` threshold opens the circuit. `baseEjectionTime` controls how long the instance is ejected. `maxEjectionPercent` limits how many instances can be ejected.

</details>

### Part D: Traffic Splitting Strategies

Compare these traffic splitting strategies and identify when to use each:

| Strategy | How It Works | Best For |
|----------|-------------|----------|
| Weight-based splitting | | |
| Header-based routing | | |
| Mirroring (shadowing) | | |
| Rate-based splitting | | |

<details>
<summary>Hint</summary>

Weight-based: percentage of traffic (canary). Header-based: route specific users (beta testers). Mirroring: copy traffic to new version without affecting users (testing). Rate-based: fixed request rate to canary (load testing).

</details>

## Success Criteria

- [ ] You can configure canary deployments with weighted traffic splitting
- [ ] You can set up retry and timeout policies
- [ ] You can design circuit breaker configurations
- [ ] You understand different traffic splitting strategies

## What You Should Understand After This Exercise

Traffic management in a service mesh enables sophisticated deployment strategies without changing application code. Canary deployments use weighted routing to test new versions with a small percentage of traffic. Retries and timeouts handle transient failures. Circuit breakers prevent cascading failures by stopping requests to unhealthy services. The combination enables safe, progressive rollouts with automatic rollback.
