# Exercise 04: Secrets Rotation with Zero Downtime

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Design and implement a secrets rotation strategy that updates a database password without any dropped requests or failed connections. This exercise trains you to think about the gap between changing a secret in the secret store and the application actually picking up the new value.

## Scenario

Your production application connects to a PostgreSQL database using a password stored in a Kubernetes Secret. The security policy requires password rotation every 30 days. The last rotation caused a 30-second outage because the application kept using the old password after the secret was updated. Your task is to design a rotation process that eliminates this downtime window.

## Architecture

```
                         Rotation Process
                              |
                              v
+----------+    +-----------------+    +----------+
|   App    |--->|  Kubernetes     |--->|  Postgres|
|  Pods    |    |  Secret         |    |  DB      |
| (N=3)    |    |  (db-password)  |    |          |
+----------+    +-----------------+    +----------+
      |                                      |
      |  Connection pool                     |  Old + New passwords
      |  uses cached password                |  both valid during
      |                                      |  rotation window
      +--------------------------------------+
```

## Tasks

### Part A: Analyze the Failure

The last rotation followed this process:

1. Update the Kubernetes Secret with the new password.
2. Restart the application pods.
3. The application connects with the new password.

Explain why this caused a 30-second outage. Identify the specific window during which requests fail and why. Consider what happens to in-flight requests, connection pools, and the timing between secret update and pod restart.

<details>
<summary>Hint</summary>

Between step 1 and step 2, the pods are still running with the old password cached in their connection pool. Between step 2 and when the new pods are ready, there are no healthy pods to serve requests. If the old password is already invalidated in the database, connections fail during this window.

</details>

### Part B: Design a Zero-Downtime Rotation Process

Design a rotation process that never drops a request. Your process must handle these constraints:

1. The database only allows one valid password at a time (you cannot have two passwords active simultaneously).
2. Kubernetes Secrets are eventually consistent -- there is a delay between updating a Secret and the volume mount reflecting the change.
3. The application uses a connection pool that caches the password at startup.

Write a step-by-step rotation plan with at least six steps. For each step, describe what happens and why it does not cause downtime.

<details>
<summary>Hint</summary>

The key insight is ordering: you must update the application before you invalidate the old password. Think about running old and new versions of the application simultaneously (blue-green or rolling update), then cutting over. Consider using a sidecar or init container that monitors the secret file for changes.

</details>

### Part C: Implement a Rolling Secret Update

Write the Kubernetes manifests for a Deployment that supports rolling secret updates without downtime. The deployment should:

1. Use a `RollingUpdate` strategy with `maxUnavailable: 0` and `maxSurge: 1`.
2. Mount the database password from a Kubernetes Secret as a file.
3. Include a readiness probe that verifies the database connection is working.
4. Include a pre-stop hook that drains connections before the pod terminates.

<details>
<summary>Hint</summary>

`maxUnavailable: 0` ensures that old pods keep running until new pods are ready. The readiness probe ensures that traffic is only sent to pods that can actually connect to the database. The pre-stop hook gives the pod time to finish in-flight requests before Kubernetes sends SIGTERM.

</details>

### Part D: Write the Rotation Script

Write a shell script (`rotate-password.sh`) that performs the full rotation process:

1. Generate a new random password.
2. Update the PostgreSQL user's password (while the old password is still valid for existing connections).
3. Update the Kubernetes Secret with the new password.
4. Trigger a rolling restart of the deployment.
5. Wait for all pods to be ready with the new password.
6. Verify that the application is responding correctly.

The script should include error handling: if any step fails, it should roll back to the previous password.

<details>
<summary>Hint</summary>

PostgreSQL allows you to change a user's password with `ALTER USER appuser PASSWORD 'newpass';`. Existing connections continue to work until they are closed -- only new connections use the new password. Use `kubectl rollout restart deployment/app` to trigger a rolling restart. Use `kubectl rollout status deployment/app` to wait for completion.

</details>

### Part E: Handle the Connection Pool Edge Case

Even with a rolling update, there is a subtle edge case: a pod might establish a connection to the database using the old password, cache it in the connection pool, and then the secret file is updated -- but the pod does not restart because the Deployment hash did not change (the secret content changed, but the Pod spec did not).

Describe two approaches to handle this edge case:

1. A sidecar container that watches the secret file and triggers a graceful reload.
2. An application-level approach where the app detects secret file changes and refreshes the connection pool.

<details>
<summary>Hint</summary>

For the sidecar approach, use a container that runs `inotifywait` on the secret file and sends a signal (e.g., SIGHUP) to the main process. For the application-level approach, the app can periodically stat the file and compare the mtime to detect changes. Both approaches must gracefully close existing connections and establish new ones.

</details>

## Success Criteria

- [ ] You explained why the naive rotation process causes downtime (specific timing analysis).
- [ ] You designed a six-step rotation plan that never drops a request.
- [ ] Your Kubernetes manifests use `maxUnavailable: 0` and include a readiness probe.
- [ ] Your rotation script handles all six steps with rollback on failure.
- [ ] You described two approaches for the connection pool edge case.
- [ ] You can articulate the difference between secret store update and application secret consumption.

## What You Should Understand After This Exercise

Secrets rotation is not a single action -- it is a multi-step process with a critical ordering constraint: you must update consumers before you invalidate the old secret. Kubernetes makes this harder because changing a Secret does not automatically restart pods, and connection pools cache credentials. Zero-downtime rotation requires rolling updates, readiness probes, connection draining, and a mechanism to detect and respond to secret file changes. The rotation script must be idempotent and support rollback because failed rotations in production are worse than delayed rotations.
