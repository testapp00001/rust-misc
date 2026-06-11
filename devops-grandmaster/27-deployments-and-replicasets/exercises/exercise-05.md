# Exercise 05: Deployment Pipeline with Health Checks

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Design a complete deployment pipeline that combines Deployments, health checks, rollout controls, and automated verification. You will create a production-grade Deployment manifest with liveness probes, readiness probes, resource limits, rollout strategy parameters, and minReadySeconds. Then you will simulate a deployment pipeline that verifies health before and after updates, handles failures gracefully, and provides observability into the deployment process.

## Background

In production, deploying software is not just `kubectl apply`. A safe deployment pipeline must verify that new Pods are healthy before shifting traffic, detect failures quickly, and roll back automatically if something goes wrong. Kubernetes provides the building blocks -- probes, `minReadySeconds`, `progressDeadlineSeconds`, and `readinessGates` -- but you must assemble them correctly. This exercise integrates all these concepts into a single, cohesive deployment workflow.

## Tasks

### Part A: Production-Grade Deployment Manifest

Create a file named `production-app.yaml` with a Deployment that meets all of the following requirements:

**Application details:**
- Name: `prod-api`
- Image: `nginx:1.25` (placeholder for your actual application)
- 4 replicas
- Labels: `app: prod-api`, `tier: backend`, `env: production`

**Container configuration:**
- Container name: `api`
- Port: 8080
- Resource requests: 250m CPU, 128Mi memory
- Resource limits: 500m CPU, 256Mi memory

**Health checks:**
- Readiness probe: HTTP GET on `/ready` port 8080, initial delay 5 seconds, period 5 seconds, failure threshold 3
- Liveness probe: HTTP GET on `/healthz` port 8080, initial delay 15 seconds, period 10 seconds, failure threshold 3

**Rollout controls:**
- Strategy: RollingUpdate
- maxSurge: 1
- maxUnavailable: 0
- minReadySeconds: 10
- progressDeadlineSeconds: 300
- revisionHistoryLimit: 5

<details>
<summary>Hint -- Readiness vs Liveness Timing</summary>
The readiness probe should start checking sooner than the liveness probe. A Pod that is not ready simply does not receive traffic -- it is not killed. A Pod that fails its liveness probe is restarted. Your application needs time to initialize before the liveness probe starts checking, or Kubernetes will restart it during startup. Set `initialDelaySeconds` for liveness probe higher than for readiness probe.
</details>

<details>
<summary>Hint -- minReadySeconds</summary>
`minReadySeconds` is the time a Pod must be ready (passing readiness probe) before it is considered available. This prevents Kubernetes from marking a Pod as available immediately after the readiness probe passes, giving your application time to fully warm up (load caches, establish database connections, etc.). If the Pod crashes during this period, it does not count as available.
</details>

Apply the Deployment:

```bash
kubectl apply -f production-app.yaml
```

### Part B: Deployment Verification Script

Write a shell script named `verify-deployment.sh` that performs the following checks after a deployment. The script should:

1. Wait for the rollout to complete (with a timeout)
2. Verify that all Pods are in Running state
3. Verify that all Pods are Ready (readiness probe passing)
4. Verify that the Deployment has the Available condition set to True
5. Verify that the correct number of replicas are available
6. Print a summary of the deployment status

The script should exit with code 0 on success and non-zero on failure.

<details>
<summary>Hint -- kubectl rollout status</summary>
`kubectl rollout status deployment <name> --timeout=300s` will wait for the rollout to complete and exit with code 0 on success or non-zero on timeout/failure. Combine this with `kubectl get` commands and `jsonpath` to extract specific fields for verification.
</details>

<details>
<summary>Hint -- Checking Pod Readiness</summary>
You can check if all Pods are ready by looking at the `Ready` condition or by checking that the `readyReplicas` field in the Deployment status equals the `replicas` field. Use `kubectl get deployment <name> -o jsonpath='{.status.readyReplicas}'` to get the ready count.
</details>

### Part C: Simulate a Deployment Pipeline

Perform the following steps in order. Record the output of each command.

**Step 1: Initial deployment verification**

```bash
# Verify the initial deployment is healthy
kubectl rollout status deployment prod-api --timeout=60s
kubectl get deployment prod-api
kubectl get pods -l app=prod-api
```

**Step 2: Update the application**

```bash
# Update the image
kubectl set image deployment prod-api api=nginx:1.26

# Immediately check the rollout status
kubectl rollout status deployment prod-api --timeout=120s
```

**Step 3: Post-deployment verification**

```bash
# Verify all Pods are running the new image
kubectl get pods -l app=prod-api -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.spec.containers[0].image}{"\t"}{.status.phase}{"\n"}{end}'

# Verify the Deployment conditions
kubectl get deployment prod-api -o jsonpath='{.status.conditions[*].type}{"\n"}'
```

**Step 4: Simulate a failure and automatic handling**

```bash
# Update to a bad image
kubectl set image deployment prod-api api=nginx:bad-tag-does-not-exist

# Watch what happens
kubectl rollout status deployment prod-api --timeout=60s
# This will fail -- that is expected

# Check the Deployment status
kubectl describe deployment prod-api | grep -A 10 "Conditions"
```

Answer these questions:

1. What does `progressDeadlineSeconds: 300` do when a deployment is stuck?
2. After the bad update, is the old version still serving traffic? How can you verify?
3. What is the fastest way to recover?

<details>
<summary>Hint -- progressDeadlineSeconds</summary>
If the deployment does not make progress within `progressDeadlineSeconds`, Kubernetes marks the deployment condition as `Progressing: False` with reason `ProgressDeadlineExceeded`. This does NOT automatically roll back -- it just signals that the deployment is stuck. You must roll back manually or use a controller that watches for this condition.
</details>

**Step 5: Rollback and verify recovery**

```bash
# Roll back the bad update
kubectl rollout undo deployment prod-api

# Verify the rollback
kubectl rollout status deployment prod-api --timeout=60s
kubectl get deployment prod-api -o jsonpath='{.spec.template.spec.containers[0].image}'
```

### Part D: Deployment Observability

Write the commands to answer each question. You should know these commands from memory.

1. How do you see all events related to the `prod-api` Deployment?
2. How do you see which ReplicaSet is currently active?
3. How do you see the full rollout history?
4. How do you see the details of revision 2?
5. How do you see which node each Pod is running on?

<details>
<summary>Hint -- Events and Debugging</summary>
Events are the first place to look when something goes wrong. Use `kubectl get events --field-selector involvedObject.name=<deployment-name>` to filter events for a specific Deployment. Events have a TTL and are garbage collected, so check them promptly after an issue occurs.
</details>

### Part E: Production Checklist

Based on everything you have learned, fill in the production deployment checklist. For each item, explain why it matters and what happens if you skip it.

| Checklist Item | Why It Matters | What Happens If Skipped |
|----------------|----------------|-------------------------|
| Resource requests and limits set | | |
| Readiness probe configured | | |
| Liveness probe configured | | |
| maxUnavailable: 0 for critical services | | |
| minReadySeconds > 0 | | |
| progressDeadlineSeconds set | | |
| revisionHistoryLimit set | | |

### Part F: Clean Up

```bash
kubectl delete deployment prod-api
```

## Success Criteria

- [ ] Your Deployment manifest includes all specified configuration (probes, resources, rollout controls)
- [ ] You can write a verification script that checks deployment health
- [ ] You performed a rolling update and verified the new version
- [ ] You simulated a failed update and observed the behavior
- [ ] You rolled back and verified recovery
- [ ] You can explain what each rollout control parameter does
- [ ] You completed the production checklist with accurate explanations

## What You Should Understand After This Exercise

After completing this exercise, you should be able to create production-grade Deployments that are safe to update, detect failures, and recover quickly. You should understand that a deployment pipeline is not just `kubectl apply` -- it is a combination of health checks, rollout controls, verification steps, and observability. The goal is not just to deploy code, but to deploy code safely with confidence that you can detect and recover from failures.
