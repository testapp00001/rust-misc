# Exercise 05: Zero-Downtime Deployment Design

**Type:** Integration
**Time:** 60 minutes
**Difficulty:** Hard

## Objective

Design a complete zero-downtime deployment system that combines graceful
shutdown, connection draining, health checks, load balancer configuration,
and database connection management. This exercise integrates everything from
this module with concepts from previous modules (health checks, load balancing,
session management).

## Scenario

You are the platform engineer for an e-commerce company. The application stack
consists of:

```
                    Internet
                       |
                 [Load Balancer]
                 (AWS ALB or Nginx)
                   /        \
            [App v1]      [App v1]
            (3 replicas)
                  |
            [PostgreSQL]
            (Primary + Replica)
```

Requirements:
- Zero user-facing errors during deployments
- No dropped database transactions
- Deployments must complete within 2 minutes
- The system must handle 3 in-flight requests during a deploy
- Health checks must accurately reflect instance readiness

---

## Tasks

### Part A: Design the Full Shutdown Sequence

Draw the complete sequence of events from when `kubectl apply` is run to when
the old pods are fully terminated. Include:

1. What Kubernetes does (creates new pods, terminates old pods)
2. What the preStop hook does
3. What the application does when it receives SIGTERM
4. What the load balancer does (endpoint updates, connection draining)
5. What the database connection pool does

Use a timeline format with parallel events where they occur.

<details>
<summary>Hint 1</summary>

Think about what happens in parallel vs sequentially. Kubernetes starts new
pods and begins terminating old pods at roughly the same time (controlled by
maxSurge and maxUnavailable). The endpoint removal and preStop hook also
happen in parallel.

</details>

<details>
<summary>Hint 2</summary>

The load balancer has its own deregistration delay (for AWS ALB, default is
300 seconds). You need to coordinate this with the Kubernetes
terminationGracePeriodSeconds.

</details>

### Part B: Write the Application Code

Write the complete graceful shutdown handler for the application. It must:

1. Catch SIGTERM
2. Set the readiness probe to return 503
3. Stop accepting new requests
4. Wait for in-flight requests to complete
5. Close idle database connections
6. Wait for active database transactions to complete
7. Close remaining database connections
8. Flush logs and metrics
9. Exit cleanly

Your code should handle the case where any step takes too long.

<details>
<summary>Hint</summary>

Break the shutdown into phases with time budgets. For example: phase 1 (0-5s)
stop accepting requests and close idle connections. Phase 2 (5-25s) wait for
in-flight requests. Phase 3 (25-30s) force-close everything and exit. Each
phase has a deadline.

</details>

### Part C: Write the Kubernetes Manifests

Write the complete Kubernetes manifests including:

1. **Deployment** with preStop hook, terminationGracePeriodSeconds, probes,
   and rolling update strategy
2. **Service** that routes traffic to the pods
3. **HorizontalPodAutoscaler** (describe how it interacts with graceful
   shutdown during scale-down)

For the HPA, explain: when the HPA scales down, does it terminate pods
gracefully or forcefully? How does this interact with your shutdown handler?

<details>
<summary>Hint 1</summary>

When the HPA scales down, it deletes pods using the same termination sequence
as `kubectl delete pod`. This means preStop hooks run and
terminationGracePeriodSeconds is respected. Your shutdown handler handles
this correctly -- it does not need to distinguish between a deployment
scale-down and an HPA scale-down.

</details>

<details>
<summary>Hint 2</summary>

For the rolling update strategy, use `maxUnavailable: 0` and `maxSurge: 1`.
This ensures that during the rollout, the old pods keep serving while new
pods start up and pass their readiness probes.

</details>

### Part D: Handle the Edge Cases

For each edge case below, explain what happens and whether your design handles
it correctly:

1. **A request takes 60 seconds**: Your grace period is 30 seconds.
2. **The database is slow**: Active transactions take 45 seconds to complete.
3. **Multiple deploys in quick succession**: Someone runs `kubectl apply` twice
   in 10 seconds.
4. **The preStop hook fails**: The `sleep 5` command errors out.
5. **The pod is force-deleted**: Someone runs `kubectl delete pod --force`.

<details>
<summary>Hint</summary>

For edge cases 1 and 2, the timeout forces cleanup. For case 3, Kubernetes
handles this with its rollout controller -- it will wait for the first rollout
to complete before starting the second. For case 4, the preStop hook failure
does not prevent SIGTERM from being sent. For case 5, --force bypasses the
graceful termination -- there is nothing you can do.

</details>

### Part E: Monitoring and Observability

Design the metrics and logs you would emit during shutdown so that you can
debug deployment issues in production:

1. What metrics should be emitted during shutdown?
2. What log messages should be emitted at each phase?
3. How would you alert on shutdowns that take too long?
4. How would you detect if requests are being dropped during deploys?

<details>
<summary>Hint</summary>

Track metrics like: shutdown_duration_seconds, requests_during_shutdown,
connections_closed, shutdown_timeout_reached. Log at each phase transition.
Alert if shutdown_duration_seconds approaches terminationGracePeriodSeconds.
Detect dropped requests by comparing error rates during deploy windows to
baseline error rates.

</details>

---

## Success Criteria

- [ ] The shutdown sequence coordinates Kubernetes, load balancer, application, and database
- [ ] The application code handles timeout at every phase of shutdown
- [ ] The Kubernetes manifests include preStop hook, probes, and rolling update strategy
- [ ] You can explain what happens during HPA scale-down
- [ ] You can identify and handle at least 4 edge cases
- [ ] You have a monitoring plan for detecting deployment-related issues
- [ ] The complete design achieves zero dropped requests during a rolling update

## What You Should Understand After This Exercise

Zero-downtime deployment is not a single configuration -- it is a system of
coordinated components. The application must handle SIGTERM correctly. The
Kubernetes manifests must configure the right probes and hooks. The load
balancer must drain connections. The database pool must be cleaned up in order.
If any one of these components is misconfigured, requests will be dropped.
The edge cases are where production incidents happen -- the happy path is
easy, but the timeout and force-kill scenarios require careful design.
