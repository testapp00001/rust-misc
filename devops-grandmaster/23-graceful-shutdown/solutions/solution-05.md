# Solution 05: Zero-Downtime Deployment Design

## Part A: Design the Full Shutdown Sequence

Here is the complete sequence from `kubectl apply` to pod termination:

```
T=0s     kubectl apply -f deployment.yaml
         Kubernetes API server updates the Deployment spec
         Deployment controller sees the change, begins rolling update

T=0s     Deployment controller creates 1 new pod (maxSurge: 1)
         New pod: ContainerCreating -> ImagePull -> ContainerCreating

T=5s     New pod container starts
         Application begins initialization (DB connect, cache warmup)
         Readiness probe: FAILING (initialDelaySeconds: 5)

T=8s     New pod application ready
         Readiness probe: PASSING
         kube-proxy adds new pod to Service endpoints
         New pod starts receiving traffic

T=9s     Deployment controller marks old pod for termination
         (because new pod is ready and maxUnavailable: 0)

T=9s     OLD POD TERMINATION BEGINS (two things happen in parallel):

         PARALLEL TRACK A (Kubernetes):            PARALLEL TRACK B (Container):
         ─────────────────────────────             ──────────────────────────────
         T=9s  Pod status: Terminating             T=9s  preStop hook starts
         T=9s  Endpoint removal starts             T=9s  sleep 5
         T=9s  kube-proxy update propagates
         T=11s Endpoint fully removed
         T=14s preStop hook finishes
                                                  T=14s SIGTERM sent to PID 1

T=14s    APPLICATION SHUTDOWN HANDLER:
         - shutting_down = True
         - /health/ready returns 503 (redundant, already removed from endpoints)
         - /health/live returns 200 (do not restart a shutting pod)
         - New requests rejected with 503
         - In-flight requests continue processing

T=14s    DATABASE POOL SHUTDOWN:
         - Idle connections closed immediately (7 connections)
         - Active connections wait for their requests to complete

T=14-29s IN-FLIGHT REQUESTS DRAIN:
         - Request A: INSERT + COMMIT (2s) -> session returned -> done at T=16s
         - Request B: SELECT (3s) -> session returned -> done at T=17s
         - Request C: UPDATE (waiting for lock, 8s) -> COMMIT -> done at T=22s
         - All in-flight requests complete

T=29s    FINAL CLEANUP:
         - Session.remove() (thread-local cleanup)
         - engine.dispose() (close pool)
         - Flush logs
         - sys.exit(0)

T=29s    Pod terminates (well within 60s grace period)

T=29s    Deployment controller creates next old pod for termination
         (repeat for remaining 2 pods)
```

### Why This Works

The key insight is that Kubernetes and the application handle different parts
of the shutdown in parallel:

- **Kubernetes handles infrastructure**: endpoint removal, kube-proxy updates,
  pod lifecycle management
- **The application handles business logic**: in-flight request completion,
  database transaction cleanup, log flushing
- **The preStop hook bridges the gap**: it creates a buffer between
  infrastructure changes and application shutdown

The `maxUnavailable: 0` strategy ensures that at no point during this sequence
are fewer than 3 pods available. The load balancer always has healthy backends.

### Common Mistakes to Avoid

- **Not accounting for the parallel tracks.** The preStop sleep and endpoint
  removal happen at the same time. If you forget this, you miscalculate the
  total shutdown time.
- **Setting terminationGracePeriodSeconds too low.** If the total sequence
  takes 29 seconds and the grace period is 30 seconds, you have only 1 second
  of margin. Any variation (slow database, network latency) causes SIGKILL.
- **Forgetting that the sequence repeats for each pod.** With 3 replicas and
  maxSurge: 1, the full deployment takes 3 iterations of this sequence.

## Part B: Write the Application Code

Here is the complete shutdown handler with phased time budgets:

```python
import signal
import sys
import time
import threading
from flask import Flask, jsonify
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker, scoped_session

app = Flask(__name__)
engine = create_engine(
    "postgresql://user:pass@db:5432/mydb",
    pool_size=10,
    pool_pre_ping=True
)
Session = scoped_session(sessionmaker(bind=engine))

shutting_down = False
in_flight = 0
lock = threading.Lock()

@app.before_request
def before_request():
    global in_flight
    with lock:
        if shutting_down:
            return jsonify({"error": "Service shutting down"}), 503
        in_flight += 1

@app.after_request
def after_request(response):
    global in_flight
    with lock:
        in_flight -= 1
    return response

@app.route('/health/live')
def health_live():
    """Liveness: always 200 unless the process is deadlocked."""
    return jsonify({"status": "ok"}), 200

@app.route('/health/ready')
def health_ready():
    """Readiness: 503 during shutdown or if not initialized."""
    if shutting_down:
        return jsonify({"status": "not_ready", "reason": "shutting_down"}), 503
    # Check database connectivity
    try:
        Session.execute("SELECT 1")
        return jsonify({"status": "ready"}), 200
    except Exception:
        return jsonify({"status": "not_ready", "reason": "database_unavailable"}), 503

@app.route('/order', methods=['POST'])
def create_order():
    session = Session()
    try:
        session.execute(
            "INSERT INTO orders (user_id, total) VALUES (:uid, :total)",
            {"uid": 1, "total": 99.99}
        )
        session.commit()
        return jsonify({"status": "created"})
    except Exception:
        session.rollback()
        return jsonify({"error": "order creation failed"}), 500
    finally:
        Session.remove()

def graceful_shutdown(signum, frame):
    global shutting_down
    shutting_down = True
    shutdown_start = time.time()

    print(f"[SHUTDOWN] Signal {signum} received at {shutdown_start}")
    print(f"[SHUTDOWN] {in_flight} in-flight requests")

    # ── Phase 1: Drain in-flight requests (budget: 25 seconds) ──
    phase1_budget = 25
    phase1_start = time.time()
    while in_flight > 0 and (time.time() - phase1_start) < phase1_budget:
        remaining = phase1_budget - (time.time() - phase1_start)
        print(f"[SHUTDOWN] Phase 1: {in_flight} requests remaining, "
              f"{remaining:.0f}s budget left")
        time.sleep(1)

    if in_flight > 0:
        print(f"[SHUTDOWN] Phase 1 TIMEOUT: {in_flight} requests abandoned")
    else:
        elapsed = time.time() - phase1_start
        print(f"[SHUTDOWN] Phase 1 complete in {elapsed:.1f}s")

    # ── Phase 2: Close database connections (budget: 3 seconds) ──
    print("[SHUTDOWN] Phase 2: Closing database connections...")
    try:
        Session.remove()
    except Exception as e:
        print(f"[SHUTDOWN] Session.remove() error: {e}")

    try:
        engine.dispose()
    except Exception as e:
        print(f"[SHUTDOWN] engine.dispose() error: {e}")

    print("[SHUTDOWN] Phase 2 complete: database pool closed")

    # ── Phase 3: Flush logs and metrics (budget: 2 seconds) ──
    print("[SHUTDOWN] Phase 3: Flushing logs...")
    sys.stdout.flush()
    sys.stderr.flush()
    print("[SHUTDOWN] Phase 3 complete: logs flushed")

    # ── Summary ──
    total_elapsed = time.time() - shutdown_start
    print(f"[SHUTDOWN] Shutdown complete in {total_elapsed:.1f}s. Exiting.")
    sys.exit(0)

signal.signal(signal.SIGTERM, graceful_shutdown)

if __name__ == '__main__':
    print("[STARTUP] Application starting on port 8080")
    app.run(host='0.0.0.0', port=8080, threaded=True)
```

### Why This Works

The shutdown is broken into three phases with explicit time budgets:

| Phase | Duration | Action | Failure mode |
|-------|----------|--------|--------------|
| 1 | 0-25s | Drain in-flight requests | Timeout: force-proceed to phase 2 |
| 2 | 25-28s | Close database connections | Exception: log and continue |
| 3 | 28-30s | Flush logs | Exception: log and continue |

Each phase has a budget. If phase 1 takes less than 25 seconds, the remaining
time is available for phases 2 and 3. If phase 1 times out, phases 2 and 3
still run but with less time before the grace period expires.

The total budget (30 seconds) is well within the `terminationGracePeriodSeconds`
of 60 seconds, giving a 30-second safety margin for the preStop sleep and
any unexpected delays.

### Common Mistakes to Avoid

- **Not having phase budgets.** Without budgets, one slow phase can consume
  the entire grace period, leaving no time for cleanup.
- **Letting exceptions in one phase prevent subsequent phases.** Each phase
  must be wrapped in try/except so that a failure in database cleanup does
  not prevent log flushing.
- **Not flushing stdout/stderr.** Python buffers stdout by default. If the
  process exits before the buffer is flushed, you lose the shutdown log
  messages.

## Part C: Write the Kubernetes Manifests

### Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ecommerce-app
  labels:
    app: ecommerce
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 0
      maxSurge: 1
  selector:
    matchLabels:
      app: ecommerce
  template:
    metadata:
      labels:
        app: ecommerce
    spec:
      terminationGracePeriodSeconds: 60
      containers:
        - name: app
          image: ecommerce:1.0
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
            initialDelaySeconds: 10
            periodSeconds: 10
            failureThreshold: 3
          readinessProbe:
            httpGet:
              path: /health/ready
              port: 8080
            initialDelaySeconds: 10
            periodSeconds: 5
            failureThreshold: 1
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: db-credentials
                  key: url
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 500m
              memory: 256Mi
```

### Service

```yaml
apiVersion: v1
kind: Service
metadata:
  name: ecommerce-service
spec:
  selector:
    app: ecommerce
  ports:
    - port: 80
      targetPort: 8080
  type: ClusterIP
```

### HorizontalPodAutoscaler

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: ecommerce-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: ecommerce-app
  minReplicas: 3
  maxReplicas: 10
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
```

### How HPA interacts with graceful shutdown during scale-down

When the HPA scales down (e.g., from 5 replicas to 3), it deletes 2 pods.
The deletion follows the standard Kubernetes termination sequence:

1. Pod status set to Terminating
2. Endpoint removal starts
3. preStop hook runs (sleep 5)
4. SIGTERM sent
5. Application shutdown handler runs
6. terminationGracePeriodSeconds respected

**The shutdown handler does not need to distinguish between a deployment
update and an HPA scale-down.** The same code handles both. This is a
strength of the design: the application only knows it received SIGTERM,
and it shuts down gracefully regardless of the reason.

However, there is one consideration: during scale-down, Kubernetes may choose
to delete pods that have the fewest connections or the least recent activity.
This is not guaranteed -- Kubernetes does not have built-in "connection-aware"
pod selection. If you need this behavior, you can use a custom controller or
PodDisruptionBudgets.

### Common Mistakes to Avoid

- **Not setting resource requests.** Without them, the HPA cannot calculate
  CPU utilization and will not scale.
- **Using `terminationGracePeriodSeconds` shorter than the shutdown sequence.**
  The total sequence (preStop sleep + drain + cleanup) must fit within the
  grace period.
- **Not using PodDisruptionBudgets.** During voluntary disruptions (node drain,
  HPA scale-down), a PDB ensures a minimum number of pods remain available.

## Part D: Handle the Edge Cases

### 1. A request takes 60 seconds (grace period is 30 seconds)

**What happens:**
- Phase 1 times out at 25 seconds
- The 60-second request is abandoned
- The client receives a connection error
- Database transaction is rolled back (Session.remove() in the force-close)

**Does the design handle it?** Yes. The timeout in phase 1 ensures the shutdown
does not hang forever. The force-close in phase 2 cleans up the database
connection. The client gets an error, but the system remains consistent.

**How to mitigate:** If 60-second requests are expected, increase the grace
period. Or, break long requests into shorter, resumable operations.

### 2. The database is slow (active transactions take 45 seconds)

**What happens:**
- Phase 1 waits up to 25 seconds for in-flight requests
- The 45-second transaction is still running when the timeout expires
- Phase 2 force-closes the connection, which rolls back the transaction
- The client receives an error

**Does the design handle it?** Yes. The transaction is rolled back, not left
in an ambiguous state. The client can retry.

**How to mitigate:** Set database statement timeouts shorter than the shutdown
drain period. If the database query itself takes 45 seconds, that is a
separate problem (slow query, missing index).

### 3. Multiple deploys in quick succession

**What happens:**
- First `kubectl apply` starts a rolling update
- Second `kubectl apply` is issued 10 seconds later
- Kubernetes' Deployment controller handles this: it updates the desired
  state and the rolling update continues from the current state
- If the first update is still in progress, the controller adjusts: it may
  terminate pods that were already updated in the first rollout

**Does the design handle it?** Yes. The application does not need to know
about the deployment history. Each pod receives SIGTERM and shuts down
gracefully, regardless of how many deployments have occurred.

**How to mitigate:** Use `kubectl rollout status` to wait for one deployment
to complete before starting another. Or use CI/CD that serializes deployments.

### 4. The preStop hook fails (the `sleep 5` errors out)

**What happens:**
- The preStop hook exits with a non-zero status
- Kubernetes still sends SIGTERM to the container
- The application receives SIGTERM and runs its shutdown handler
- The only consequence is that the 5-second buffer is lost

**Does the design handle it?** Yes. The preStop hook failure does not prevent
SIGTERM delivery. The application still shuts down gracefully. The risk is
that the endpoint removal race condition is not mitigated, so a few requests
might hit the shutting-down pod and receive 503.

**How to mitigate:** Keep the preStop command simple (`sleep 5` is very
unlikely to fail). For extra safety, you can implement the sleep in the
application's SIGTERM handler instead of the preStop hook.

### 5. The pod is force-deleted (`kubectl delete pod --force`)

**What happens:**
- `--force` sets the grace period to 0
- Kubernetes sends SIGTERM and immediately sends SIGKILL
- The application has no time to shut down gracefully
- In-flight requests are dropped
- Database transactions may be left incomplete

**Does the design handle it?** No. This is by design -- `--force` is an
emergency escape hatch. It bypasses all graceful shutdown mechanisms.

**How to mitigate:** Do not use `--force` in normal operations. Use it only
when a pod is stuck in Terminating state and you need to force-remove it.
If you must use it, accept that some requests will be dropped. Monitor for
connection errors and retry at the client level.

### Common Mistakes to Avoid

- **Designing only for the happy path.** The edge cases are where production
  incidents happen. Every timeout, every force-kill, every slow database is
  a potential source of dropped requests.
- **Not having client-level retry logic.** Even with perfect graceful shutdown,
  some requests will fail (network issues, force-deletes, unexpected bugs).
  The client must retry.

## Part E: Monitoring and Observability

### Metrics to emit during shutdown

```python
# Using Prometheus client library
from prometheus_client import Counter, Histogram, Gauge

shutdown_duration = Histogram(
    'app_shutdown_duration_seconds',
    'Time spent in shutdown handler',
    buckets=[1, 5, 10, 15, 20, 25, 30, 45, 60]
)
shutdown_timeouts = Counter(
    'app_shutdown_timeouts_total',
    'Number of shutdowns that hit the drain timeout'
)
shutdown_requests_abandoned = Counter(
    'app_shutdown_requests_abandoned_total',
    'Requests abandoned during shutdown timeout'
)
shutdown_connections_closed = Counter(
    'app_shutdown_connections_closed_total',
    'Database connections closed during shutdown'
)
in_flight_at_shutdown = Gauge(
    'app_in_flight_at_shutdown',
    'Number of in-flight requests when SIGTERM was received'
)
```

### Log messages at each phase

```
[SHUTDOWN] Signal 15 received. 3 in-flight requests.
[SHUTDOWN] Phase 1: Draining in-flight requests (budget: 25s)
[SHUTDOWN] Phase 1: 3 requests remaining, 24s budget left
[SHUTDOWN] Phase 1: 2 requests remaining, 22s budget left
[SHUTDOWN] Phase 1: 1 requests remaining, 19s budget left
[SHUTDOWN] Phase 1 complete in 6.2s
[SHUTDOWN] Phase 2: Closing database connections
[SHUTDOWN] Phase 2: 10 connections closed (7 idle, 3 returned)
[SHUTDOWN] Phase 2 complete in 0.1s
[SHUTDOWN] Phase 3: Flushing logs and metrics
[SHUTDOWN] Phase 3 complete in 0.0s
[SHUTDOWN] Total shutdown: 6.3s (grace period: 60s)
```

### Alerting on shutdowns that take too long

```yaml
# Prometheus alert rule
groups:
  - name: graceful-shutdown
    rules:
      - alert: ShutdownApproachingTimeout
        expr: app_shutdown_duration_seconds > 45
        for: 0m
        labels:
          severity: warning
        annotations:
          summary: "Shutdown took {{ $value }}s, approaching 60s grace period"

      - alert: ShutdownTimeoutHit
        expr: rate(app_shutdown_timeouts_total[5m]) > 0
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "Shutdowns timing out -- requests are being dropped"

      - alert: RequestsAbandonedDuringShutdown
        expr: rate(app_shutdown_requests_abandoned_total[5m]) > 0
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "{{ $value }} requests abandoned during shutdown in last 5m"
```

### Detecting dropped requests during deploys

Compare error rates during deployment windows to baseline:

```python
# PromQL: error rate during deployment vs baseline
# Error rate in the last 5 minutes
rate(http_requests_total{status=~"5.."}[5m])
/
rate(http_requests_total[5m])

# Compare to: error rate in the last 1 hour (baseline)
rate(http_requests_total{status=~"5.."}[1h])
/
rate(http_requests_total[1h])

# Alert if the ratio exceeds 2x
(
  rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m])
)
/
(
  rate(http_requests_total{status=~"5.."}[1h]) / rate(http_requests_total[1h])
) > 2
```

Also track: `Connection reset by peer` errors in load balancer logs, and
`503` responses from the readiness probe during deployment windows.

### Why This Works

Monitoring turns graceful shutdown from a "hope it works" mechanism into a
measurable, alertable system. If you do not measure shutdown duration, you
cannot tune the grace period. If you do not track abandoned requests, you
cannot detect when shutdowns are failing. If you do not compare error rates
during deploys, you cannot prove that zero-downtime deployment is actually
achieving zero downtime.

### Common Mistakes to Avoid

- **Not monitoring shutdown at all.** You will not know requests are being
  dropped until users complain.
- **Monitoring only the happy path.** Track timeouts, abandoned requests,
  and force-closes -- not just successful shutdowns.
- **Not correlating deploys with error spikes.** If you do not tag deploy
  events in your monitoring system, you cannot distinguish deploy-related
  errors from application bugs.

## Key Takeaway

Zero-downtime deployment is a system, not a feature. It requires coordination
between Kubernetes (rolling update strategy, preStop hooks, probes), the load
balancer (endpoint propagation, connection draining), the application (signal
handling, request tracking, phased cleanup), and the database (connection pool
management, transaction safety). Each component has its own timing constraints,
and the overall system is only as strong as the weakest link. The edge cases --
slow requests, slow databases, force-deletes, rapid redeploys -- are where
production incidents happen. Design for them, monitor for them, and have a
plan for when they occur.
