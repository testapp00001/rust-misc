# Solution 04: Secrets Rotation with Zero Downtime

## Part A: Analyze the Failure

The naive rotation process caused a 30-second outage due to a critical ordering problem:

```
Timeline:
  T+0s   Update Kubernetes Secret with new password
         --> Old pods still running, using old password from connection pool
         --> Old password is NOW INVALID in the database
         --> All in-flight connections FAIL
  T+0s   Restart pods (kubectl rollout restart)
         --> Old pods begin terminating
         --> New pods begin starting (image pull, init, readiness probe)
  T+15s  New pods starting up, not yet ready
         --> No healthy pods to serve requests
         --> 503 errors from the ingress
  T+25s  New pods become ready
         --> They read the new password from the updated Secret
         --> They connect to the database successfully
  T+30s  All pods running with new password
         --> Service recovers
```

**The specific failure window is T+0s to T+25s.** Three things go wrong simultaneously:

1. **Immediate failure (T+0s):** The database password is changed before the pods are restarted. Existing pods have the old password cached in their connection pool. Every new connection attempt fails because the database rejects the old password.

2. **Unavailability (T+0s to T+25s):** Old pods are terminating and new pods are not ready. During this window, the service has zero healthy replicas to serve traffic.

3. **No rollback path:** If the new password has a typo, the pods will fail to start. There is no easy way to roll back because the old password is already invalidated in the database.

### Common mistakes

- Thinking the problem is just "pods need to restart." The real problem is the ordering: the database password was invalidated before the application was updated.
- Not considering connection pools. Even without a restart, a pod with a cached connection using the old password would eventually fail when the connection is recycled.
- Ignoring the readiness gap. Even with correct ordering, there is a window where no pods are ready if the restart is not rolling.

---

## Part B: Design a Zero-Downtime Rotation Process

### Step-by-step rotation plan

**Step 1: Generate a new password.**
Generate a cryptographically random password and store it temporarily (not in the Secret yet). This does not affect any running systems.

**Step 2: Update the database to accept BOTH the old and new passwords.**
This is the critical insight. If your database does not support multiple valid passwords, use a connection proxy (e.g., PgBouncer) that accepts both passwords during the rotation window. Alternatively, create a new database user with the new password and grant it the same permissions.

```
Database accepts: old_password AND new_password
Running pods use: old_password (still working)
```

**Step 3: Update the Kubernetes Secret with the new password.**
The Secret now contains the new password. Running pods still have the old password cached in their connection pool. Because the database accepts both passwords, existing connections continue to work.

**Step 4: Perform a rolling restart of the deployment.**
Use `kubectl rollout restart deployment/app` with a `RollingUpdate` strategy (`maxUnavailable: 0`, `maxSurge: 1`). Old pods keep serving traffic while new pods start up and pass readiness probes.

```
Old pods:  using old_password (still accepted by database)
New pods:  using new_password (also accepted by database)
```

**Step 5: Wait for all new pods to be ready.**
Use `kubectl rollout status deployment/app` to wait. Once all new pods are ready and passing readiness probes, traffic is routed exclusively to new pods.

```
All pods:  using new_password
Database:  still accepts both passwords
```

**Step 6: Invalidate the old password.**
Once all pods are confirmed to use the new password, update the database to reject the old password. If anything goes wrong, you can still roll back by updating the Secret to the old password and restarting again.

```
All pods:       using new_password
Database:       only accepts new_password
Rotation:       complete
```

### Why this works

The key principle is **update consumers before invalidating credentials**. By keeping the old password valid until all pods have restarted, there is never a moment where a pod cannot connect to the database. The `maxUnavailable: 0` strategy ensures that old pods continue serving traffic until new pods are confirmed healthy.

### Common mistakes

- Invalidating the old password too early (Step 6 before Step 5 is complete). If you invalidate before all pods have restarted, the remaining old pods will lose their database connections.
- Not having a rollback plan. If Step 4 fails (new pods cannot start), you need to revert the Secret and restart with the old password while it is still valid.
- Assuming the database supports multiple valid passwords. PostgreSQL does not -- you need PgBouncer or a similar proxy, or you must use a different approach (e.g., create a new user).

---

## Part C: Implementing a Rolling Secret Update

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: app
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 0
      maxSurge: 1
  selector:
    matchLabels:
      app: myapp
  template:
    metadata:
      labels:
        app: myapp
    spec:
      terminationGracePeriodSeconds: 60
      volumes:
        - name: db-secrets
          secret:
            secretName: db-credentials
      containers:
        - name: app
          image: myapp:1.0.0
          ports:
            - containerPort: 8080
          volumeMounts:
            - name: db-secrets
              mountPath: /run/secrets
              readOnly: true
          env:
            - name: DB_PASSWORD_FILE
              value: /run/secrets/db-password
            - name: DB_HOST
              value: "postgres.default.svc.cluster.local"
          readinessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 5
            failureThreshold: 3
          lifecycle:
            preStop:
              exec:
                command:
                  - sh
                  - -c
                  - |
                    echo "Draining connections..."
                    sleep 10
                    echo "Shutdown complete."
```

### Why this works

- `maxUnavailable: 0` ensures that the number of ready pods never drops below the desired count during the rollout. Old pods keep running until new pods pass their readiness probe.
- `maxSurge: 1` allows one extra pod temporarily during the rollout, so there are at most 4 pods running (3 old + 1 new) while the new pod is starting.
- The `readinessProbe` on `/health` ensures that the new pod only receives traffic after it has verified its database connection. If the new password is wrong, the readiness probe fails and Kubernetes does not route traffic to the broken pod.
- The `preStop` hook gives the pod 10 seconds to drain in-flight requests before Kubernetes sends SIGTERM. Combined with `terminationGracePeriodSeconds: 60`, the pod has up to 60 seconds to shut down gracefully.
- The secret is mounted as a volume (`/run/secrets`) rather than injected as an environment variable. This makes rotation easier because volume mounts can be updated by recreating the pod, while environment variables require a Deployment spec change.

### Common mistakes

- Setting `maxUnavailable: 1` (the default). This means one old pod can be terminated before a new pod is ready, creating a brief window with reduced capacity.
- Not having a readiness probe. Without it, Kubernetes routes traffic to the new pod as soon as the container starts, even if the application has not yet connected to the database.
- Setting `terminationGracePeriodSeconds` too low. If in-flight requests take 30 seconds and the grace period is 10 seconds, requests will be dropped.
- Using environment variables for secrets in a rotation scenario. Volume mounts are better because the secret file is updated when the pod is recreated, but env vars require a Deployment spec change to trigger a rollout.

---

## Part D: The Rotation Script

```bash
#!/bin/bash
# rotate-password.sh -- Zero-downtime database password rotation
set -euo pipefail

# Configuration
NAMESPACE="default"
DEPLOYMENT="app"
SECRET_NAME="db-credentials"
DB_USER="appuser"
DB_HOST="postgres.default.svc.cluster.local"
DB_PORT="5432"
DB_NAME="myapp"
OLD_PASSWORD_FILE="/tmp/old_password"
NEW_PASSWORD_FILE="/tmp/new_password"
ROLLBACK_NEEDED=false

echo "=== Starting password rotation ==="

# Step 1: Generate new password
echo "Step 1: Generating new password..."
NEW_PASSWORD=$(openssl rand -base64 32 | tr -d '/+=' | head -c 32)
echo "$NEW_PASSWORD" > "$NEW_PASSWORD_FILE"
echo "  New password generated (${#NEW_PASSWORD} chars)"

# Step 2: Save old password (for rollback)
echo "Step 2: Saving current password for rollback..."
OLD_PASSWORD=$(kubectl get secret "$SECRET_NAME" -n "$NAMESPACE" \
  -o jsonpath='{.data.db-password}' | base64 -d)
echo "$OLD_PASSWORD" > "$OLD_PASSWORD_FILE"
echo "  Old password saved"

# Step 3: Update database password
echo "Step 3: Updating database password..."
# Note: In a real system, you would use PgBouncer to accept both passwords.
# This example assumes the database accepts the ALTER USER command.
if ! kubectl exec -n "$NAMESPACE" deploy/postgres -- \
  psql -U postgres -c "ALTER USER $DB_USER PASSWORD '$NEW_PASSWORD';" 2>/dev/null; then
  echo "ERROR: Failed to update database password. Aborting."
  exit 1
fi
ROLLBACK_NEEDED=true
echo "  Database password updated"

# Step 4: Update Kubernetes Secret
echo "Step 4: Updating Kubernetes Secret..."
NEW_PASSWORD_B64=$(echo -n "$NEW_PASSWORD" | base64)
kubectl get secret "$SECRET_NAME" -n "$NAMESPACE" -o json | \
  jq --arg pw "$NEW_PASSWORD_B64" '.data["db-password"] = $pw' | \
  kubectl apply -f -
echo "  Secret updated"

# Step 5: Rolling restart
echo "Step 5: Performing rolling restart..."
kubectl rollout restart deployment/"$DEPLOYMENT" -n "$NAMESPACE"

# Step 6: Wait for rollout to complete
echo "Step 6: Waiting for pods to be ready..."
if ! kubectl rollout status deployment/"$DEPLOYMENT" -n "$NAMESPACE" \
  --timeout=120s; then
  echo "ERROR: Rollout failed or timed out."
  if [ "$ROLLBACK_NEEDED" = true ]; then
    echo "Rolling back..."
    # Restore old password in database
    kubectl exec -n "$NAMESPACE" deploy/postgres -- \
      psql -U postgres -c "ALTER USER $DB_USER PASSWORD '$OLD_PASSWORD';"
    # Restore old secret
    OLD_PASSWORD_B64=$(echo -n "$OLD_PASSWORD" | base64)
    kubectl get secret "$SECRET_NAME" -n "$NAMESPACE" -o json | \
      jq --arg pw "$OLD_PASSWORD_B64" '.data["db-password"] = $pw' | \
      kubectl apply -f -
    # Restart with old password
    kubectl rollout restart deployment/"$DEPLOYMENT" -n "$NAMESPACE"
    kubectl rollout status deployment/"$DEPLOYMENT" -n "$NAMESPACE" --timeout=120s
    echo "Rollback complete."
  fi
  exit 1
fi

# Step 7: Verify
echo "Step 7: Verifying application health..."
sleep 5
HEALTH=$(curl -s -o /dev/null -w "%{http_code}" \
  "http://$DEPLOYMENT.$NAMESPACE.svc.cluster.local:8080/health" 2>/dev/null || echo "000")
if [ "$HEALTH" = "200" ]; then
  echo "  Application is healthy (HTTP 200)"
else
  echo "WARNING: Application health check returned HTTP $HEALTH"
  echo "Consider manual verification."
fi

# Cleanup temp files
rm -f "$OLD_PASSWORD_FILE" "$NEW_PASSWORD_FILE"

echo ""
echo "=== Rotation complete ==="
echo "  Old password: invalidated"
echo "  New password: active"
echo "  All pods: running with new password"
```

### Why this works

- The script follows the six-step process from Part B, ensuring that the database and the Kubernetes Secret are updated atomically (Steps 3 and 4 happen close together).
- Step 2 saves the old password before overwriting, enabling rollback.
- The rollback block (triggered if `rollout status` fails) restores both the database password and the Kubernetes Secret, then restarts the pods with the old password.
- `openssl rand -base64 32` generates cryptographically random passwords. The `tr -d '/+='` removes characters that cause problems in shell escaping and URLs.
- The script uses `kubectl get secret ... -o json | jq ... | kubectl apply -f -` to update the secret in-place without deleting and recreating it.

### Common mistakes

- Not cleaning up temp files. The script stores passwords in `/tmp` files during rotation. If the script is interrupted, these files remain. Use `trap` to clean up on exit.
- Not testing rollback. A rollback that has never been tested is not a rollback -- it is a hope. Test the rollback path in a non-production environment before running this in production.
- Using `kubectl delete secret` and `kubectl create secret` instead of updating in-place. Deleting and recreating can cause a brief window where the secret does not exist, and pods that restart during this window will fail to mount it.
- Not setting a timeout on `rollout status`. Without `--timeout`, the script waits forever if the rollout is stuck.

---

## Part E: Handle the Connection Pool Edge Case

### Approach 1: Sidecar container that watches the secret file

```yaml
containers:
  - name: secret-watcher
    image: alpine:latest
    command:
      - sh
      - -c
      - |
        apk add --no-cache inotify-tools
        echo "Watching /run/secrets/db-password for changes..."
        while true; do
          inotifywait -e modify /run/secrets/db-password
          echo "Secret file changed, sending SIGHUP to app..."
          # Send SIGHUP to the main process (PID 1 in the pod)
          # The app must handle SIGHUP by reloading its connection pool
          kill -HUP 1
          sleep 2  # Debounce -- avoid rapid restarts
        done
    volumeMounts:
      - name: db-secrets
        mountPath: /run/secrets
        readOnly: true
```

**How it works:**

- `inotifywait` blocks until the specified file is modified. When Kubernetes updates the Secret volume (which happens when the Secret is updated and the pod is recreated), the file changes and `inotifywait` returns.
- The sidecar sends `SIGHUP` to PID 1 (the main application process). The application must be designed to handle `SIGHUP` by re-reading the secret file and refreshing its connection pool.
- The `sleep 2` prevents rapid repeated triggers if the file is modified multiple times in quick succession.

**Limitation:** This only works if the application is designed to handle `SIGHUP`. Many applications are not. You would need to modify the application code to add a signal handler.

### Approach 2: Application-level file change detection

```rust
use std::fs;
use std::time::SystemTime;

struct SecretWatcher {
    path: String,
    last_modified: SystemTime,
}

impl SecretWatcher {
    fn new(path: &str) -> Self {
        let metadata = fs::metadata(path).expect("Secret file not found");
        SecretWatcher {
            path: path.to_string(),
            last_modified: metadata.modified().unwrap(),
        }
    }

    fn has_changed(&mut self) -> bool {
        let metadata = fs::metadata(&self.path).unwrap();
        let current_modified = metadata.modified().unwrap();
        if current_modified > self.last_modified {
            self.last_modified = current_modified;
            true
        } else {
            false
        }
    }
}

// In the main application loop:
let mut secret_watcher = SecretWatcher::new("/run/secrets/db-password");

loop {
    if secret_watcher.has_changed() {
        println!("Secret changed, refreshing connection pool...");
        let new_password = read_secret("/run/secrets/db-password")?;
        connection_pool.update_password(&new_password);
    }

    // Normal application logic
    handle_request(&connection_pool).await;
}
```

**How it works:**

- The application periodically checks the modification time of the secret file. If it changes, the application re-reads the file and updates its connection pool.
- This approach does not require a sidecar container or signal handling. The application is self-contained.
- The check can happen on every request (expensive), on a timer (e.g., every 30 seconds), or on a connection failure (reactive).

**Limitation:** The application must be modified to support this. This is a code-level change, not an infrastructure-level change.

### Comparison

| Aspect | Sidecar (SIGHUP) | Application-level |
|--------|-------------------|-------------------|
| Application changes | Must handle SIGHUP | Must add file watcher |
| Infrastructure changes | Adds a sidecar container | None |
| Reliability | Depends on signal delivery | Depends on polling frequency |
| Complexity | Low (shell script) | Medium (application code) |
| Works with any app | Only if app handles SIGHUP | No -- requires app modification |

### Common mistakes

- Not handling the case where the secret file is temporarily empty during a volume update. Kubernetes may briefly remove and recreate the file. The application should retry reads with backoff.
- Polling the file on every request. This adds I/O overhead to every request. Use a timer-based check (e.g., every 30 seconds) instead.
- Not closing and reopening database connections when the password changes. Some connection pools cache the password and do not re-authenticate existing connections. You may need to drain the pool and create new connections.
