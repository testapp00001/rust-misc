# Solution 03: Traffic Management

## Part A: Canary Deployment

### DestinationRule (define subsets)

```yaml
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: api-service
  namespace: production
spec:
  host: api-service
  subsets:
  - name: v1
    labels:
      version: v1
  - name: v2
    labels:
      version: v2
```

### VirtualService (traffic splitting)

```yaml
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: api-service
  namespace: production
spec:
  hosts:
  - api-service
  http:
  - route:
    - destination:
        host: api-service
        subset: v1
      weight: 90
    - destination:
        host: api-service
        subset: v2
      weight: 10
```

### How It Works

```
100 requests arrive:
  90 requests → api-service pods with label version=v1 (5 replicas)
  10 requests → api-service pods with label version=v2 (1 replica)

Load balancing within each subset:
  v1 subset: round-robin across 5 pods (18 requests each)
  v2 subset: round-robin across 1 pod (10 requests to that one pod)
```

### Why This Works

The `weight` field distributes traffic proportionally. It is applied at the
VirtualService level (before the request reaches any pod). The DestinationRule
defines which pods belong to which subset based on labels. Kubernetes labels
on pods (`version: v1`, `version: v2`) are the mapping mechanism.

## Part B: Retries and Timeouts

```yaml
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: api-service
  namespace: production
spec:
  hosts:
  - api-service
  http:
  - timeout: 10s
    retries:
      attempts: 3
      perTryTimeout: 3s
      retryOn: "5xx,reset,connect-failure"
    route:
    - destination:
        host: api-service
        subset: v1
      weight: 90
    - destination:
        host: api-service
        subset: v2
      weight: 10
```

### Retry Behavior

```
Request arrives:
  1. Send to v2 backend
  2. Backend returns 500 (error)
  3. Wait 25ms (backoff)
  4. Retry: send to v2 backend again
  5. Backend returns 500 again
  6. Wait 50ms (backoff)
  7. Retry: send to v2 backend again
  8. Backend returns 500 again
  9. Wait 100ms (backoff)
  10. Retry: send to v1 backend (different instance)
  11. Backend returns 200 (success)
  12. Total time: 175ms + 3 failed attempts + 1 success
```

### Why Not Retry on 4xx?

4xx errors (400 Bad Request, 401 Unauthorized, 404 Not Found) are **client
errors**. Retrying the same request will produce the same error. Retries
should only be for **transient failures**:
- 5xx: server error (may be transient)
- reset: TCP connection reset (network issue)
- connect-failure: cannot connect (server may be starting up)
- refused-stream: HTTP/2 stream refused (server overloaded)

### perTryTimeout

The `perTryTimeout: 3s` means each individual attempt times out after 3
seconds. Combined with 3 retries, the maximum total time is:
3s (attempt 1) + 3s (attempt 2) + 3s (attempt 3) = 9 seconds.
This stays within the overall `timeout: 10s`.

## Part C: Circuit Breaker

```yaml
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: api-service
  namespace: production
spec:
  host: api-service
  trafficPolicy:
    connectionPool:
      tcp:
        maxConnections: 100
      http:
        h2UpgradePolicy: DEFAULT
        http1MaxPendingRequests: 100
        http2MaxRequests: 1000
        maxRequestsPerConnection: 10
    outlierDetection:
      consecutive5xxErrors: 5
      interval: 10s
      baseEjectionTime: 30s
      maxEjectionPercent: 50
  subsets:
  - name: v1
    labels:
      version: v1
  - name: v2
    labels:
      version: v2
```

### Circuit Breaker Behavior

```
Normal state (circuit closed):
  All requests flow to all backend instances

5 consecutive errors detected on instance X:
  Circuit OPENS for instance X
  Instance X is ejected from the load balancing pool
  All requests go to remaining healthy instances

After 30 seconds (baseEjectionTime):
  Circuit enters HALF-OPEN state
  1 test request is sent to instance X

If test request succeeds:
  Circuit CLOSES
  Instance X returns to the pool

If test request fails:
  Circuit reopens for another 30 seconds
  Process repeats

maxEjectionPercent: 50%
  At most 50% of instances can be ejected
  Prevents cascading ejection (all instances ejected = no capacity)
```

### Circuit Breaker vs Retries

| Aspect | Retries | Circuit Breaker |
|--------|---------|-----------------|
| **Purpose** | Recover from transient failures | Stop sending traffic to unhealthy instances |
| **When to use** | Request fails, try again | Instance fails repeatedly, stop trying |
| **Scope** | Per-request | Per-instance |
| **Time scale** | Milliseconds | Seconds to minutes |
| **Effect** | Hides failures from client | Removes unhealthy instances from pool |

**They work together:**
1. Retries handle single-request failures (try again immediately)
2. Circuit breaker handles persistent instance failures (stop sending traffic)
3. If retries fail 5 times, circuit breaker ejects the instance

## Part D: Traffic Splitting Strategies

| Strategy | How It Works | Best For |
|----------|-------------|----------|
| **Weight-based splitting** | Percentage of traffic to each version (e.g., 90/10) | Canary deployments, gradual rollouts |
| **Header-based routing** | Route based on HTTP headers (e.g., `x-user-id: beta`) | Beta testing, internal dogfooding, debugging |
| **Mirroring (shadowing)** | Copy live traffic to new version, discard response | Testing new version with real traffic without affecting users |
| **Rate-based splitting** | Fixed request rate to canary (e.g., 100 req/min) | Load testing, controlled exposure |

### Header-Based Routing Example

```yaml
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: api-service
spec:
  hosts:
  - api-service
  http:
  # Route beta testers to v2
  - match:
    - headers:
        x-beta-user:
          exact: "true"
    route:
    - destination:
        host: api-service
        subset: v2
  # Route everyone else to v1
  - route:
    - destination:
        host: api-service
        subset: v1
```

### Mirroring Example

```yaml
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: api-service
spec:
  hosts:
  - api-service
  http:
  - route:
    - destination:
        host: api-service
        subset: v1
      weight: 100
    mirror:
      host: api-service
      subset: v2
    mirrorPercentage:
      value: 100.0
```

**Mirroring sends a copy of every request to v2 but discards v2's response.
The user always gets v1's response. This tests v2 with real traffic without
any user impact.**

### Common Mistakes to Avoid

- **Retrying non-idempotent requests.** Retrying a POST /payment may charge
  the customer twice. Only retry idempotent operations (GET, PUT, DELETE)
  or operations with idempotency keys.
- **Circuit breaker too aggressive.** If `consecutive5xxErrors` is set to 1,
  a single error ejects the instance. This causes unnecessary ejections
  during normal operation. Use 3-5 as the threshold.
- **Not testing canary with enough traffic.** A 10% canary with 100 requests/
  second means only 10 requests/second to the canary. This may not reveal
  issues that only appear under load.
- **Forgetting about connection pool limits.** If `maxConnections` is too
  low, the circuit breaker opens when the service is actually healthy but
  just busy. Set limits based on actual service capacity.

## Key Takeaway

Traffic management in a service mesh enables safe, progressive deployments.
Canary deployments use weighted routing to test new versions. Retries handle
transient failures automatically. Circuit breakers prevent cascading failures
by ejecting unhealthy instances. The combination of retries + circuit breakers
provides both immediate recovery (retry) and long-term protection (circuit
breaker). The key is choosing the right strategy for each use case.
