# Solution 03: Configure Kubernetes Shutdown Policies

## Part A: The Deployment Manifest

Here is the complete manifest with all values filled in:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-app
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 0
      maxSurge: 1
  selector:
    matchLabels:
      app: web-app
  template:
    metadata:
      labels:
        app: web-app
    spec:
      terminationGracePeriodSeconds: 60
      containers:
        - name: web-app
          image: myapp:1.0
          ports:
            - containerPort: 8080
          lifecycle:
            preStop:
              exec:
                command: ["/bin/sh", "-c", "sleep 5"]
          livenessProbe:
            httpGet:
              path: /health/live
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 10
            failureThreshold: 3
          readinessProbe:
            httpGet:
              path: /health/ready
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 5
            failureThreshold: 1
```

### Why Each Value Was Chosen

**`terminationGracePeriodSeconds: 60`**

Calculation:
```
preStop sleep:                     5 seconds
p99 in-flight request completion: 15 seconds
Database + log cleanup:            2 seconds
                                ----------
Subtotal:                         22 seconds
Safety margin:                    38 seconds
                                ----------
Total:                            60 seconds
```

60 seconds gives ample time for the worst case. It is long enough to handle
p99 requests with room to spare, but short enough that pod deletions do not
take unreasonably long during deployments or HPA scale-down events.

**`preStop: sleep 5`**

This sleep exists to work around a race condition. When Kubernetes decides to
terminate a pod, it does two things in parallel:

1. Remove the pod from the Service endpoints (so kube-proxy stops routing traffic)
2. Run the preStop hook

The endpoint removal takes time to propagate through kube-proxy and any ingress
controllers. Without the sleep, SIGTERM might arrive while traffic is still
being routed to the pod. The 5-second sleep gives the endpoint removal time to
propagate before the application starts shutting down.

**Liveness probe: `initialDelaySeconds: 5, periodSeconds: 10, failureThreshold: 3`**

The liveness probe checks if the process is alive. It should NOT fail during
shutdown -- if it does, Kubernetes restarts the pod, which is worse than letting
it shut down gracefully. The `failureThreshold: 3` with `periodSeconds: 10`
means the pod must fail for 30 consecutive seconds before being restarted.
This is generous enough that a slow shutdown will not trigger a restart.

**Readiness probe: `initialDelaySeconds: 5, periodSeconds: 5, failureThreshold: 1`**

The readiness probe checks if the pod can handle traffic. During startup, the
`initialDelaySeconds: 5` (matching the 3-second startup time with margin)
gives the application time to initialize. During shutdown, the application
returns 503 from `/health/ready`, and `failureThreshold: 1` means the pod is
marked not-ready on the first failed check. This ensures the pod is removed
from the Service endpoints quickly.

### Common Mistakes to Avoid

- **Setting terminationGracePeriodSeconds too low.** If set to 10 seconds,
  the preStop sleep alone uses 5 seconds, leaving only 5 seconds for request
  completion and cleanup.
- **Not having a preStop hook.** Without it, there is a race condition between
  endpoint removal and SIGTERM delivery.
- **Using the same path for liveness and readiness.** The readiness endpoint
  must return 503 during shutdown. If liveness uses the same endpoint, Kubernetes
  might restart the pod during shutdown.

## Part B: Explain the Rolling Update Strategy

```yaml
strategy:
  type: RollingUpdate
  rollingUpdate:
    maxUnavailable: 0
    maxSurge: 1
```

### What does `maxUnavailable: 0` guarantee?

It guarantees that Kubernetes will never reduce the number of available pods
below the desired replica count (3). During a rolling update, Kubernetes will
not terminate an old pod until a new pod is running and passing its readiness
check. At any point during the deployment, all 3 replicas are serving traffic.

### What does `maxSurge: 1` mean?

It means Kubernetes can create 1 extra pod beyond the desired count during a
rollout. With `replicas: 3`, Kubernetes can have up to 4 pods running during
the deployment: 3 old pods still serving + 1 new pod starting up. Once the
new pod passes its readiness check, Kubernetes terminates one of the old pods.

### Why is this combination important?

This combination implements the "ramp up before draining" pattern:

```
Before deploy:    3 old pods serving
During deploy:    3 old pods + 1 new pod (4 total)
After new ready:  2 old pods + 1 new pod (3 total, one old draining)
Continue:         1 old pod + 2 new pods (3 total)
Complete:         3 new pods serving
```

At no point are fewer than 3 pods available.

### What would happen with `maxUnavailable: 1`?

With `maxUnavailable: 1`, Kubernetes can terminate an old pod before the new
pod is ready. This creates a window where only 2 pods are serving traffic:

```
maxUnavailable: 1:
  Before deploy:    3 old pods serving
  During deploy:    2 old pods + 1 new pod starting (3 total, but new not ready)
  Brief window:     2 old pods serving (capacity reduced by 33%)
```

If traffic is at capacity, this causes request failures during the deployment.
For zero-downtime deployments, `maxUnavailable: 0` is essential.

### Common Mistakes to Avoid

- **Using the default strategy.** The default is `maxUnavailable: 25%`, which
  can reduce capacity during deployment.
- **Not setting maxSurge.** Without maxSurge, Kubernetes must wait for an old
  pod to terminate before creating a new one, which extends deployment time.

## Part C: The preStop Race Condition

### The race condition without a preStop hook

When a pod is deleted, Kubernetes does these things:

1. The pod status is set to "Terminating"
2. The pod is removed from Service endpoints (kube-proxy update)
3. SIGTERM is sent to the container

Steps 2 and 3 happen at roughly the same time. But step 2 is not instant --
kube-proxy must update iptables or IPVS rules, and ingress controllers must
update their routing tables. This propagation can take 1-5 seconds.

Without a preStop hook:
```
T=0s:   Pod marked for deletion
T=0s:   Endpoint removal starts (async)
T=0s:   SIGTERM sent immediately
T=0s:   Application starts shutting down, returns 503 from readiness probe
T=0-5s: Traffic still arriving (endpoint propagation not complete)
T=0-5s: These requests hit a shutting-down application -> 503 errors
T=5s:   Endpoint propagation complete, no more traffic
```

### How the sleep solves it

With a preStop hook of `sleep 5`:
```
T=0s:   Pod marked for deletion
T=0s:   Endpoint removal starts (async)
T=0s:   preStop hook runs: sleep 5
T=0-5s: Endpoint propagation completes while the pod is still running normally
T=5s:   preStop hook finishes, SIGTERM sent
T=5s:   Application starts shutting down (no new traffic is arriving)
T=5-60s: In-flight requests complete, cleanup happens
T=60s:  SIGKILL if still running
```

The sleep creates a buffer between endpoint removal and application shutdown.

### When 5 seconds is not enough

You might need more than 5 seconds if:
- You use a service mesh (Istio, Linkerd) that has its own endpoint propagation
- Your ingress controller has slow reconciliation loops
- You have a large cluster with many endpoints to update
- You use external load balancers (AWS ALB, GCP LB) with their own health check
  intervals (which can be 5-30 seconds)

For external load balancers, you may need a preStop sleep of 10-30 seconds,
and correspondingly longer `terminationGracePeriodSeconds`.

### Common Mistakes to Avoid

- **Assuming endpoint removal is instant.** It is not. The propagation delay
  is real and causes 503 errors during deployment.
- **Using a fixed sleep without measuring.** If your ingress takes 10 seconds
  to propagate, a 5-second sleep is not enough.

## Part D: Liveness vs Readiness During Shutdown

### 1. Should the liveness probe pass or fail?

**The liveness probe should PASS (return 200) during shutdown.**

If the liveness probe fails during shutdown, Kubernetes interprets this as
"the process is unhealthy" and restarts it. But the process is already
shutting down -- restarting it would create a new instance that immediately
starts and then has to deal with the old instance still draining. This can
cause cascading issues.

The liveness probe is for detecting deadlocks and unrecoverable errors, not
for shutdown.

### 2. Should the readiness probe pass or fail?

**The readiness probe should FAIL (return 503) immediately during shutdown.**

The readiness probe tells Kubernetes whether this pod should receive traffic.
During shutdown, the pod should not receive new traffic. Returning 503 from
the readiness probe causes Kubernetes to remove the pod from the Service
endpoints, directing new traffic to other pods.

### 3. What happens if the liveness probe fails during shutdown?

```
T=0s:   SIGTERM received, app starts shutting down
T=0s:   Readiness probe returns 503 (removed from endpoints)
T=10s:  Liveness probe returns 503 (app is shutting down)
T=20s:  Liveness probe fails again
T=30s:  Liveness probe fails third time (failureThreshold: 3)
T=30s:  Kubernetes restarts the pod
T=30s:  New pod starts, old pod is still terminating
T=30s:  Two instances of the same app running simultaneously
T=30s:  Potential port conflicts, database connection issues
```

This is why the liveness probe should always return 200 during shutdown.

### 4. What happens if the readiness probe continues passing during shutdown?

```
T=0s:   SIGTERM received, app starts shutting down
T=0s:   Readiness probe returns 200 (Kubernetes thinks: "still healthy!")
T=1s:   New request routed to this pod
T=1s:   App rejects it with 503 (shutting_down flag is true)
T=2s:   Another new request routed to this pod
T=2s:   Also rejected with 503
...     This continues until the pod is terminated
```

Users get 503 errors for every request routed to this pod. The load balancer
does not know the pod is shutting down because the readiness probe says it
is fine.

### Common Mistakes to Avoid

- **Using the same endpoint for liveness and readiness.** If the readiness
  endpoint returns 503 during shutdown and liveness uses the same endpoint,
  Kubernetes restarts the pod. Use separate endpoints.
- **Not having a readiness probe at all.** Without it, Kubernetes considers
  the pod ready as soon as the container starts, even before the application
  has initialized.

## Key Takeaway

Kubernetes shutdown is a coordinated dance between the API server, kube-proxy,
endpoint controller, and the application. The preStop hook solves the race
condition between endpoint removal and SIGTERM delivery. The liveness probe
must not trigger during shutdown. The readiness probe must fail immediately
to stop traffic. The terminationGracePeriodSeconds must account for all of
these steps. Getting any one of these wrong causes dropped requests during
deployment.
