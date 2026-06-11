# Exercise 04: Debugging CrashLoopBackOff

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Diagnose and fix a Pod that is stuck in CrashLoopBackOff. You will be given a broken Pod manifest, must identify the root cause using only kubectl commands and Pod status information, fix the manifest, and explain why each problem causes the observed behavior. This exercise simulates a real production debugging scenario where you did not write the original manifest.

## Scenario

A teammate deployed a Pod to your cluster. It was working yesterday, but now it is stuck in CrashLoopBackOff. The teammate is unavailable. You need to figure out what is wrong and fix it. The application is a simple HTTP health-check endpoint that reads its configuration from an environment variable.

## Tasks

### Part A: Deploy the Broken Pod

Create a file named `broken-pod.yaml` with the following manifest. **Do not fix anything yet.** Deploy it as-is.

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
          while true; do
            echo "OK - $(date)"
            sleep 10
          done
      ports:
        - containerPort: 8080
      resources:
        requests:
          cpu: 100m
          memory: 64Mi
        limits:
          cpu: 200m
          memory: 128Mi
      livenessProbe:
        httpGet:
          path: /healthz
          port: 8080
        initialDelaySeconds: 5
        periodSeconds: 10
      readinessProbe:
        httpGet:
          path: /healthz
          port: 8080
        initialDelaySeconds: 3
        periodSeconds: 5
  restartPolicy: Always
```

```bash
kubectl apply -f broken-pod.yaml
```

### Part B: Diagnose the Problems

Use the following commands to investigate. Record your findings for each command.

```bash
# Check the Pod status
kubectl get pod health-service

# Get detailed status information
kubectl describe pod health-service

# Check container logs (including previous restart)
kubectl logs health-service
kubectl logs health-service --previous

# Check events
kubectl get events --field-selector involvedObject.name=health-service
```

The Pod has **three distinct problems** that contribute to the CrashLoopBackOff. Identify all three and fill in the table:

| Problem # | What You Observed | Root Cause | Which Component |
|-----------|-------------------|------------|-----------------|
| 1 | | | |
| 2 | | | |
| 3 | | | |

<details>
<summary>Hint -- Problem 1</summary>
Read the container logs carefully. The script checks for an environment variable. Is that variable defined anywhere in the Pod spec?
</details>

<details>
<summary>Hint -- Problem 2</summary>
Look at the liveness and readiness probes. What protocol and path are they using? What is the container actually doing -- is it running an HTTP server?
</details>

<details>
<summary>Hint -- Problem 3</summary>
The container exits with code 1, Kubernetes restarts it (restartPolicy: Always), the same thing happens again. After several restarts, Kubernetes backs off. But think about the probes: even if the environment variable were fixed, what would the liveness probe do after the container starts?
</details>

### Part C: Fix the Manifest

Create a file named `fixed-pod.yaml` that resolves all three problems. For each fix, add a YAML comment explaining what you changed and why.

Guidelines for your fixes:
- The application does NOT serve HTTP. It runs as a simple shell process that prints to stdout.
- The environment variable must be set to a meaningful value.
- The probes must match what the container actually does.

<details>
<summary>Hint -- Probe Alternatives</summary>
When a container does not serve HTTP, you can use an exec probe that runs a command inside the container, or a tcpSocket probe that checks if a port is open. An exec probe with a simple command like `cat /tmp/healthy` or checking if a process is running is a common pattern for non-HTTP workloads.
</details>

### Part D: Deploy and Verify the Fix

```bash
# Remove the broken Pod
kubectl delete -f broken-pod.yaml

# Deploy the fixed version
kubectl apply -f fixed-pod.yaml

# Verify it is running
kubectl get pod health-service -w

# Confirm no restarts are occurring
kubectl describe pod health-service | grep -A 5 "Last State"

# Check logs are clean
kubectl logs health-service
```

Answer these questions:

1. How many restarts occurred before the Pod stabilized?
2. What would happen if you removed the probes entirely instead of fixing them?
3. Why does Kubernetes use exponential backoff for restarts instead of immediate restarts?

### Part E: Prevention

Write three specific practices that would have prevented this Pod from being deployed in a broken state. Think about validation, testing, and deployment processes.

<details>
<summary>Hint -- Prevention Approaches</summary>
Consider: schema validation before apply, testing manifests in a staging namespace, using `kubectl apply --dry-run=client`, admission controllers, CI/CD pipeline checks, and linting tools like kubeval or kube-linter.
</details>

## Success Criteria

- [ ] You identified all three problems in the broken manifest
- [ ] Your fixed manifest deploys successfully and the Pod reaches Running state
- [ ] The Pod remains stable with no restarts after the fix
- [ ] You can explain the root cause of each problem and why it produces the observed behavior
- [ ] You proposed concrete prevention practices

## What You Should Understand After This Exercise

CrashLoopBackOff is one of the most common Pod failure modes. It always has a root cause -- Kubernetes does not fail Pods arbitrarily. The debugging pattern is always the same: check status, describe the Pod, read logs (including --previous), and check events. The three most common causes are missing configuration, misconfigured probes, and application errors. Understanding this debugging workflow is essential for operating Kubernetes in production.
