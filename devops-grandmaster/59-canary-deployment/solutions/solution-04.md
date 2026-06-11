# Solution 04: Canary with Header-Based Routing

## Part A: Traffic Routing Rules

```yaml
# virtual-service.yaml
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: my-service
spec:
  hosts:
    - my-service
    - my-service.example.com
  http:
    # Rule 1: Internal testers with x-canary header always hit canary
    - match:
        - headers:
            x-canary:
              exact: "true"
      route:
        - destination:
            host: my-service
            subset: canary
          weight: 100
      headers:
        response:
          set:
            x-served-by: canary

    # Rule 2: QA users with x-qa-user header hit canary
    - match:
        - headers:
            x-qa-user:
              regex: ".*@mycompany\\.com"
      route:
        - destination:
            host: my-service
            subset: canary
          weight: 100
      headers:
        response:
          set:
            x-served-by: canary

    # Rule 3: Default weight-based routing for real users
    - route:
        - destination:
            host: my-service
            subset: stable
          weight: 95
        - destination:
            host: my-service
            subset: canary
          weight: 5
      retries:
        attempts: 3
        perTryTimeout: 2s
        retryOn: 5xx
```

### Why This Works

Istio evaluates `http` rules in order. The first matching rule wins.
Header-based rules are listed first, so `x-canary: true` or
`x-qa-user: alice@mycompany.com` always routes to canary regardless of
the weight split.

The `x-served-by: canary` response header lets testers verify they are
actually hitting the canary version.

The default rule at the bottom handles all other traffic with the
configured weight split (95/5).

## Part B: DestinationRule

```yaml
# destination-rule.yaml
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: my-service
spec:
  host: my-service
  subsets:
    - name: stable
      labels:
        version: v1
    - name: canary
      labels:
        version: v2
  trafficPolicy:
    connectionPool:
      tcp:
        maxConnections: 100
      http:
        h2UpgradePolicy: DEFAULT
        http1MaxPendingRequests: 100
        http2MaxRequests: 1000
    outlierDetection:
      consecutive5xxErrors: 5
      interval: 30s
      baseEjectionTime: 30s
      maxEjectionPercent: 50
    loadBalancer:
      simple: ROUND_ROBIN
```

### Why This Works

The `subsets` define which pods are `stable` (version: v1) and which
are `canary` (version: v2). The VirtualService routes to these subset
names.

`outlierDetection` ejects pods that return 5 consecutive 5xx errors.
Ejected pods are removed from the load balancing pool for
`baseEjectionTime` (30 seconds). If 50% of pods are ejected
(`maxEjectionPercent: 50`), Istio stops ejecting to prevent total
outage.

`connectionPool` limits prevent the canary from overwhelming downstream
services. If the canary has a connection leak, the limits contain the
damage.

## Part C: Two-Phase Canary Process

### Phase 1: Internal Testing (Header-Based Only)

```yaml
# virtual-service-phase1.yaml
# Weight is 100/0 -- no real traffic goes to canary
http:
  # Internal testers
  - match:
      - headers:
          x-canary:
            exact: "true"
    route:
      - destination:
          host: my-service
          subset: canary

  # All other traffic goes to stable
  - route:
      - destination:
          host: my-service
          subset: stable
        weight: 100
```

```bash
#!/bin/bash
# phase1-internal-testing.sh

echo "=== Phase 1: Internal Testing ==="

# Deploy canary pods
kubectl apply -f deployment-canary.yaml
kubectl wait --for=condition=ready pod -l version=v2 --timeout=120s

# Apply header-only routing (no real user traffic to canary)
kubectl apply -f virtual-service-phase1.yaml
kubectl apply -f destination-rule.yaml

echo "Canary deployed. Only header-based traffic reaches it."
echo "QA: curl -H 'x-canary: true' https://my-service.example.com/health"

# Monitor canary metrics from internal traffic
echo "Monitoring canary error rate from internal traffic..."
for i in $(seq 1 30); do
    ERROR_RATE=$(curl -s "http://prometheus:9090/api/v1/query" \
        --data-urlencode 'query=sum(rate(http_requests_total{service="my-service",version="canary",status=~"5.."}[5m])) / sum(rate(http_requests_total{service="my-service",version="canary"}[5m]))' \
        | jq -r '.data.result[0].value[1] // "0"')

    echo "Minute $i: Canary error rate = $ERROR_RATE"

    if [ "$(echo "$ERROR_RATE > 0.001" | bc -l)" = "1" ]; then
        echo "ERROR: Canary error rate exceeds 0.1%. Aborting."
        kubectl delete -f deployment-canary.yaml
        exit 1
    fi

    sleep 60
done

echo "Phase 1 complete. Canary passed internal testing."
echo "Proceed to Phase 2? (requires QA sign-off)"
```

### Phase 2: Gradual Rollout (Weight-Based)

```bash
#!/bin/bash
# phase2-gradual-rollout.sh

echo "=== Phase 2: Gradual Rollout ==="

# Apply weight-based routing (5% to canary)
kubectl apply -f virtual-service-phase2.yaml

# Trigger Argo Rollouts canary progression
kubectl argo rollouts set image my-service my-service=my-service:v2

# Watch progress
kubectl argo rollouts get rollout my-service --watch
```

```yaml
# virtual-service-phase2.yaml
# Weight-based routing begins
http:
  - match:
      - headers:
          x-canary:
            exact: "true"
    route:
      - destination:
          host: my-service
          subset: canary

  - route:
      - destination:
          host: my-service
          subset: stable
        weight: 95
      - destination:
          host: my-service
          subset: canary
        weight: 5
```

### Transition Gates

| Gate | Condition | Verification |
|------|-----------|-------------|
| Phase 1 -> Phase 2 | QA sign-off | Manual approval |
| Phase 1 -> Phase 2 | Error rate < 0.1% | Prometheus query |
| Phase 1 -> Phase 2 | Latency p99 < 2x stable | Prometheus query |
| Phase 1 -> Phase 2 | No P0/P1 bugs filed | Issue tracker check |

## Part D: Session Affinity

```yaml
# destination-rule-with-session-affinity.yaml
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: my-service
spec:
  host: my-service
  subsets:
    - name: stable
      labels:
        version: v1
    - name: canary
      labels:
        version: v2
  trafficPolicy:
    loadBalancer:
      consistentHash:
        httpCookie:
          name: session-id
          ttl: 30m
    outlierDetection:
      consecutive5xxErrors: 5
      interval: 30s
      baseEjectionTime: 30s
```

```yaml
# virtual-service-with-session-affinity.yaml
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: my-service
spec:
  hosts:
    - my-service
  http:
    # Header-based routing (always canary)
    - match:
        - headers:
            x-canary:
              exact: "true"
      route:
        - destination:
            host: my-service
            subset: canary

    # Weight-based routing with session affinity
    - route:
        - destination:
            host: my-service
            subset: stable
          weight: 95
        - destination:
            host: my-service
            subset: canary
          weight: 5
      headers:
        response:
          set:
            set-cookie: "session-id=%REQUEST_ID%; Path=/; Max-Age=1800"
```

### Why This Works

The `consistentHash` with `httpCookie` ensures that requests with the
same `session-id` cookie always go to the same backend pod. If a user's
first request goes to a canary pod, subsequent requests with the same
cookie continue to hit canary for 30 minutes (the cookie TTL).

The response header automatically sets the `session-id` cookie if it
does not exist. On the first request, Istio assigns the user to stable
or canary based on the weight. On subsequent requests, the cookie
ensures consistency.

## Common Mistakes to Avoid

- **Not putting header rules first.** If weight-based routing is listed
  before header-based routing, the weight rule matches first and the
  header rule is never evaluated. Always put specific rules before
  general rules.
- **Forgetting the response header for verification.** Without
  `x-served-by: canary`, testers cannot confirm they are actually
  hitting the canary. Always add a response header for debugging.
- **Session affinity without TTL.** Without a TTL, the cookie persists
  forever. If the canary is rolled back, users with stale canary cookies
  might hit pods that no longer exist. Use a reasonable TTL (15-30 minutes).
- **Not testing the header-based routing.** Deploying header-based routing
  without verifying it works means you might think QA is testing the
  canary when they are actually hitting stable.

## Key Takeaway

Header-based routing enables a two-phase canary process: internal testing
with zero real-user risk, followed by gradual weight-based rollout. This
is strictly safer than weight-based canary alone because the first phase
has zero blast radius. Session affinity ensures users have a consistent
experience as traffic weights change.
