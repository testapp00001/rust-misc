# Solution 04: Debugging CrashLoopBackOff

## Part B: Diagnose the Problems

The broken Pod manifest has three distinct problems:

| Problem # | What You Observed | Root Cause | Which Component |
|-----------|-------------------|------------|-----------------|
| 1 | Container logs show "ERROR: CONFIG_VALUE is not set" followed by exit code 1 | The environment variable `CONFIG_VALUE` is referenced in the container args but is never defined in the Pod spec | Container environment configuration |
| 2 | Container logs show "Health service running on port 8080" but liveness probe fails with connection refused on /healthz | The container is a shell script that prints to stdout. It does NOT run an HTTP server. The httpGet probe on port 8080 fails because nothing is listening on that port | Liveness and readiness probes |
| 3 | Pod shows increasing restart counts and eventually CrashLoopBackOff with exponential backoff intervals (0s, 10s, 20s, 40s...) | Even if Problem 1 were fixed, the container would start successfully but the liveness probe would kill it after initialDelaySeconds (5s), causing a restart cycle | Interaction between probes and restartPolicy |

### Detailed Diagnosis Walkthrough

**`kubectl get pod health-service`** shows:

```
NAME             READY   STATUS             RESTARTS      AGE
health-service   0/1     CrashLoopBackOff   5 (32s ago)   3m
```

The high restart count and CrashLoopBackOff status tell you the container keeps crashing and Kubernetes is backing off.

**`kubectl logs health-service`** shows:

```
Starting health service...
Config value:
ERROR: CONFIG_VALUE is not set
```

This is Problem 1. The script checks `$CONFIG_VALUE`, finds it empty, and exits with code 1.

**`kubectl describe pod health-service`** shows in the Events section:

```
Warning  Unhealthy  ...  Liveness probe failed: Get "http://10.244.0.5:8080/healthz": dial tcp 10.244.0.5:8080: connect: connection refused
Warning  BackOff    ...  Back-off restarting failed container
```

This reveals Problems 2 and 3. Even if the environment variable were set, the httpGet probe would fail because there is no HTTP server.

**Problem Interaction:**

The three problems interact to create the CrashLoopBackOff:
1. The container starts and immediately exits (Problem 1: missing env var).
2. Kubernetes restarts the container (restartPolicy: Always).
3. If the env var were fixed, the container would start and run.
4. After 5 seconds (initialDelaySeconds), the liveness probe would fire.
5. The probe sends an HTTP GET to /healthz on port 8080.
6. Nothing is listening on port 8080 (Problem 2: no HTTP server).
7. The probe fails, Kubernetes kills the container.
8. Restart, repeat, exponential backoff (Problem 3).

## Part C: Fixed Manifest

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: health-service
  labels:
    app: health-service
    version: v2
spec:
  containers:
    - name: health-check
      image: busybox:1.36
      command: ["/bin/sh", "-c"]
      args:
        - |
          echo "Starting health service..."
          echo "Config value: $CONFIG_VALUE"
          if [ -z "$CONFIG_VALUE" ]; then
            echo "ERROR: CONFIG_VALUE is not set"
            exit 1
          fi
          echo "Health service running on port 8080"
          # Create a health marker file that the exec probe can check
          touch /tmp/healthy
          while true; do
            echo "OK - $(date)"
            sleep 10
          done
      env:
        # FIX 1: Define the missing environment variable
        - name: CONFIG_VALUE
          value: "production-config-v2"
      ports:
        - containerPort: 8080
      resources:
        requests:
          cpu: 100m
          memory: 64Mi
        limits:
          cpu: 200m
          memory: 128Mi
      # FIX 2: Replace httpGet probes with exec probes
      # The container is a shell script, not an HTTP server.
      # Use exec probes to check if the process is running and healthy.
      livenessProbe:
        exec:
          command:
            - cat
            - /tmp/healthy
        initialDelaySeconds: 10
        periodSeconds: 10
      readinessProbe:
        exec:
          command:
            - cat
            - /tmp/healthy
        initialDelaySeconds: 5
        periodSeconds: 5
  restartPolicy: Always
```

### What Each Fix Does and Why

**Fix 1: Added `env` section with `CONFIG_VALUE`**

```yaml
env:
  - name: CONFIG_VALUE
    value: "production-config-v2"
```

The container script checks for this variable and exits with code 1 if it is not set. By defining it in the Pod spec, the script passes the check and continues to the main loop. In production, you would use a ConfigMap or Secret instead of a hardcoded value, but the principle is the same.

**Fix 2: Replaced httpGet probes with exec probes**

```yaml
livenessProbe:
  exec:
    command:
      - cat
      - /tmp/healthy
```

The container is a shell script that prints to stdout. It does not run an HTTP server. An httpGet probe on port 8080 will always fail with "connection refused" because nothing is listening on that port. The exec probe runs a command inside the container. The command `cat /tmp/healthy` succeeds (exit code 0) if the file exists, and fails (exit code 1) if it does not. The script creates `/tmp/healthy` after passing the configuration check, so the probe succeeds only when the script is running and has initialized properly.

**Fix 3: Added `touch /tmp/healthy` after the config check**

This creates the marker file that the exec probe checks. It acts as a simple health indicator: if the file exists, the application has started successfully. This is a minimal example; a real application might write a PID file, check its internal state, or verify dependencies.

## Part D: Verification

After deploying the fixed Pod:

1. The Pod transitions from Pending to ContainerCreating to Running within 15-20 seconds.
2. `kubectl describe pod` shows the liveness probe passing with "cat /tmp/healthy" returning exit code 0.
3. The restart count remains at 0 (or at whatever it was when the Pod started).
4. `kubectl logs health-service` shows the startup messages and periodic "OK" lines.

**Why removing probes entirely would be problematic:**

Without probes, Kubernetes has no way to know if the container is healthy. The container could be stuck in a deadlock, running but unable to serve requests, or leaking memory. In production, this means traffic would continue to be sent to a broken Pod. Probes are essential for self-healing.

**Why Kubernetes uses exponential backoff:**

Immediate restart would cause a tight loop of failed starts, consuming CPU, disk I/O (for logs), and API server resources. Exponential backoff (10s, 20s, 40s, 80s, ..., capped at 5 minutes) reduces the load while still allowing the container to recover if the underlying issue is resolved (for example, a dependency coming back online).

## Part E: Prevention Practices

**1. Pre-deployment validation:**
Run `kubectl apply --dry-run=client -f manifest.yaml` to catch syntax errors. Use tools like kubeval or kube-linter to validate manifests against the Kubernetes schema and best practices. Example: kube-linter would flag that an httpGet probe targets a port that has no corresponding container port configuration.

**2. Staged rollout with verification:**
Deploy to a staging namespace first. Run a smoke test that checks if the Pod starts, passes its probes, and serves traffic. Only then promote to production. This catches runtime issues like missing environment variables that static analysis cannot detect.

**3. Automated probe testing in CI/CD:**
Add a CI step that deploys the Pod to a test cluster, waits for it to become Ready, and runs a health check. If the Pod does not become Ready within a timeout, the pipeline fails. This catches probe misconfigurations before they reach production.

**4. Admission controllers:**
Use an admission controller (like OPA/Gatekeeper or Kyverno) to enforce that all Pods have resource limits, probes, and environment variables for known dependencies. This prevents incomplete manifests from being accepted by the API server.

**5. Manifest templates and Helm charts:**
Do not write raw Pod manifests by hand. Use Helm charts or Kustomize bases that include validated templates with correct probe configurations. New deployments inherit the tested patterns instead of reimplementing them.

### Common Mistakes to Avoid

- **Fixing only the most obvious problem.** The env var missing is the first thing you see in logs, but the probe misconfiguration would still cause CrashLoopBackOff even with the env var fixed. Always check for multiple issues.

- **Using httpGet probes on non-HTTP containers.** This is extremely common. If your container is a worker, a script, or a sidecar that does not serve HTTP, use exec or tcpSocket probes instead.

- **Setting initialDelaySeconds too low.** If the application takes 10 seconds to start but initialDelaySeconds is 5, the liveness probe will fire before the app is ready, killing it immediately. Set initialDelaySeconds to be longer than the expected startup time.

- **Not checking --previous logs.** The `--previous` flag shows logs from the last crashed instance of the container. Without it, you only see the current instance, which may not have the same error.

- **Deleting the Pod to "fix" it.** Deleting and recreating a Pod from the same manifest will produce the same CrashLoopBackOff. The fix must be in the manifest, not in the Pod lifecycle.

## Key Takeaway

CrashLoopBackOff always has a root cause. The debugging workflow is systematic: get status, describe the Pod, read current and previous logs, check events. The most common causes are missing configuration, misconfigured probes, and application errors. In this exercise, all three were present simultaneously, which is realistic -- production failures often involve multiple interacting issues. The key skill is not memorizing fixes but understanding the diagnostic workflow that leads you to the root cause.
