# Exercise 03: Configure Kubernetes Shutdown Policies

**Type:** Independent
**Time:** 40 minutes
**Difficulty:** Medium

## Objective

Configure a Kubernetes Deployment that handles pod termination correctly using
preStop hooks, terminationGracePeriodSeconds, and proper health check
configuration. You will write the complete YAML manifests and explain every
field that contributes to zero-downtime shutdown.

## Scenario

You are deploying a Python web application to Kubernetes. The application:

- Serves HTTP requests on port 8080
- Has a p95 latency of 5 seconds, p99 latency of 15 seconds
- Takes about 3 seconds to initialize (connect to database, warm caches)
- Needs to flush logs and close database connections on shutdown (takes ~2 seconds)
- Has both `/health/live` (liveness) and `/health/ready` (readiness) endpoints

Your task is to write a complete Kubernetes Deployment manifest that ensures no
requests are dropped during a rolling update.

---

## Tasks

### Part A: Write the Deployment Manifest

Write a complete Kubernetes Deployment YAML with the following requirements:

1. **preStop hook**: The preStop hook should sleep for a short period before
   the application receives SIGTERM. Explain why this sleep is necessary.
2. **terminationGracePeriodSeconds**: Set an appropriate value given the
   application characteristics above.
3. **Liveness probe**: Configure a liveness probe on `/health/live`.
4. **Readiness probe**: Configure a readiness probe on `/health/ready` with
   appropriate timing for the 3-second startup.

Fill in all missing values and explain each one:

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
      terminationGracePeriodSeconds: # TODO: What value? Why?
      containers:
        - name: web-app
          image: myapp:1.0
          ports:
            - containerPort: 8080
          lifecycle:
            preStop:
              exec:
                command: # TODO: What command? Why?
          livenessProbe:
            httpGet:
              path: # TODO
              port: # TODO
            # TODO: timing fields
          readinessProbe:
            httpGet:
              path: # TODO
              port: # TODO
            # TODO: timing fields (consider startup time)
```

<details>
<summary>Hint 1</summary>

The preStop hook should run `sleep 5`. This gives Kubernetes time to remove
the pod from the Service endpoints (stop sending new traffic) before the
application starts shutting down. Without this, there is a race condition
where SIGTERM arrives before the pod is deregistered from the Service.

</details>

<details>
<summary>Hint 2</summary>

`terminationGracePeriodSeconds` should be long enough for: preStop sleep (5s)
+ in-flight request completion (up to 15s at p99) + cleanup (2s) = 22 seconds.
Round up to 30 or 60 for safety margin.

</details>

<details>
<summary>Hint 3</summary>

For the readiness probe, set `initialDelaySeconds` to at least 3 (the startup
time) and use `periodSeconds: 5` with `failureThreshold: 1` so the pod is
marked not-ready quickly during shutdown.

</details>

### Part B: Explain the Rolling Update Strategy

The manifest above uses:
```yaml
strategy:
  type: RollingUpdate
  rollingUpdate:
    maxUnavailable: 0
    maxSurge: 1
```

Explain:
1. What does `maxUnavailable: 0` guarantee?
2. What does `maxSurge: 1` mean?
3. Why is this combination important for zero-downtime deployments?
4. What would happen if you set `maxUnavailable: 1` instead?

<details>
<summary>Hint</summary>

`maxUnavailable: 0` means Kubernetes will never reduce the number of available
pods below the desired count. `maxSurge: 1` means it can create one extra pod
during the rollout. Together, this ensures that at any point during the
deployment, all 3 replicas are serving traffic.

</details>

### Part C: The preStop Race Condition

Without a preStop hook, there is a race condition during pod termination.
Describe the race:

1. What happens if SIGTERM arrives before the pod is removed from the Service
   endpoints?
2. How does the `sleep 5` in the preStop hook solve this?
3. Is 5 seconds always enough? When might you need more?

<details>
<summary>Hint</summary>

Kubernetes does two things in parallel when a pod is deleted: it removes the
pod from the Service endpoints (so the kube-proxy and ingress stop routing
traffic) and it runs the preStop hook. These happen at the same time. The
kube-proxy update can take a few seconds to propagate. The sleep gives time
for that propagation.

</details>

### Part D: Liveness vs Readiness During Shutdown

When a pod is being terminated:

1. Should the liveness probe pass or fail?
2. Should the readiness probe pass or fail?
3. What happens if the liveness probe fails during shutdown?
4. What happens if the readiness probe continues passing during shutdown?

<details>
<summary>Hint</summary>

Liveness should pass during shutdown -- you do not want Kubernetes to restart
a pod that is already shutting down. Readiness should fail immediately -- you
want Kubernetes to stop routing traffic to this pod.

</details>

---

## Success Criteria

- [ ] The Deployment manifest includes a preStop hook with a sleep command
- [ ] terminationGracePeriodSeconds accounts for preStop + request completion + cleanup
- [ ] The liveness probe is configured and will not trigger restarts during shutdown
- [ ] The readiness probe is configured with appropriate startup delay
- [ ] The rolling update strategy uses maxUnavailable: 0 and maxSurge: 1
- [ ] You can explain the preStop race condition and how the sleep solves it

## What You Should Understand After This Exercise

Kubernetes shutdown is not just about the application catching SIGTERM. It is
a coordinated sequence involving the API server, kube-proxy, endpoints
controller, and the application. The preStop hook is critical because it
addresses the race condition between endpoint removal and signal delivery.
The terminationGracePeriodSeconds must be calculated from your application's
actual behavior, not picked arbitrarily.
