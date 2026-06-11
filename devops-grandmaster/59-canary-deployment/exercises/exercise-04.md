# Exercise 04: Canary with Header-Based Routing

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Implement a canary deployment that supports header-based routing for
internal testing before exposing real users. This exercise trains you to
combine canary traffic splitting with targeted routing for QA and
stakeholder validation.

## Scenario

Your team wants to test the canary version internally before it receives
real user traffic. The requirements are:

1. Internal users with header `x-canary: true` always hit the canary
2. QA team members with header `x-qa-user: <email>` hit the canary
3. Real users are split by weight (5% canary, 95% stable)
4. The canary must pass internal testing before weight-based splitting begins

## Tasks

### Part A: Design the Traffic Routing Rules

Design the routing rules using Istio VirtualService. Show how header-based
routing takes priority over weight-based routing.

<details>
<summary>Hint</summary>

Istio VirtualService evaluates `http` rules in order. Put header-based
matches first, then weight-based routing as the default:

```yaml
http:
  # Header-based routing (evaluated first)
  - match:
      - headers:
          x-canary:
            exact: "true"
    route:
      - destination:
          host: my-service
          subset: canary
  # Weight-based routing (default)
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

</details>

### Part B: Write the DestinationRule

Create an Istio DestinationRule that defines the `stable` and `canary`
subsets based on pod labels. Include connection pool limits and outlier
detection.

<details>
<summary>Hint</summary>

The DestinationRule defines subsets by matching pod labels:

```yaml
subsets:
  - name: stable
    labels:
      version: v1
  - name: canary
    labels:
      version: v2
```

Outlier detection ejects pods that return too many 5xx errors:

```yaml
outlierDetection:
  consecutive5xxErrors: 5
  interval: 30s
  baseEjectionTime: 30s
```

</details>

### Part C: Design the Two-Phase Canary Process

Design a deployment process with two phases:
1. **Internal testing phase:** Header-based routing only, no real user traffic
2. **Gradual rollout phase:** Weight-based traffic splitting

Write the sequence of operations and the conditions for transitioning
from phase 1 to phase 2.

<details>
<summary>Hint</summary>

Phase 1:
- Deploy canary pods
- Configure header-based routing (weight = 0 for real traffic)
- QA tests via `x-canary: true` header
- Monitor error rate and latency from QA traffic
- Gate: QA sign-off + error rate < 0.1% over 30 minutes

Phase 2:
- Enable weight-based routing (start at 5%)
- Run Argo Rollouts analysis at each weight step
- Monitor real user metrics
- Automatic promotion or rollback based on analysis

</details>

### Part D: Handle Session Affinity

Users who hit the canary during header-based testing should continue
to hit the canary for the duration of their session. Design a session
affinity strategy that works with canary routing.

<details>
<summary>Hint</summary>

Use consistent hashing based on a session cookie or user ID:

```yaml
route:
  - destination:
      host: my-service
      subset: stable
    weight: 95
  - destination:
      host: my-service
      subset: canary
    weight: 5
    headers:
      request:
        set:
          x-backend-version: canary
```

Combined with a consistent hash load balancer in the DestinationRule:

```yaml
trafficPolicy:
  loadBalancer:
    consistentHash:
      httpCookie:
        name: session-id
        ttl: 30m
```

This ensures a user who starts on canary stays on canary for 30 minutes.

</details>

## Success Criteria

- [ ] Header-based routing sends internal users to canary regardless of weight
- [ ] DestinationRule defines stable and canary subsets with outlier detection
- [ ] Two-phase process separates internal testing from gradual rollout
- [ ] Session affinity keeps users on the same version during a session
- [ ] Transition from phase 1 to phase 2 has clear, measurable gates

## What You Should Understand After This Exercise

Header-based routing lets you test canary versions with internal users
before exposing real traffic. This two-phase approach (internal testing,
then gradual rollout) reduces risk further than canary alone. Session
affinity ensures users have a consistent experience even as traffic
weights change.
