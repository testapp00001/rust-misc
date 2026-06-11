# Solution 03: Sidecar Logging Pattern

## Part A: Pod Specification

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: sidecar-logging
  labels:
    app: sidecar-demo
spec:
  volumes:
    - name: app-logs
      emptyDir: {}
  containers:
    - name: app
      image: busybox:1.36
      command:
        - /bin/sh
        - -c
        - |
          mkdir -p /var/log/app
          while true; do
            echo "$(date '+%Y-%m-%d %H:%M:%S') [INFO] Application heartbeat - processing request" >> /var/log/app/application.log
            sleep 5
          done
      volumeMounts:
        - name: app-logs
          mountPath: /var/log/app
      resources:
        requests:
          cpu: 200m
          memory: 128Mi
        limits:
          cpu: 500m
          memory: 256Mi
    - name: log-shipper
      image: busybox:1.36
      command:
        - /bin/sh
        - -c
        - |
          # Wait for the log file to be created by the app container
          while [ ! -f /var/log/app/application.log ]; do
            echo "Waiting for log file..."
            sleep 1
          done
          tail -f /var/log/app/application.log
      volumeMounts:
        - name: app-logs
          mountPath: /var/log/app
      resources:
        requests:
          cpu: 50m
          memory: 64Mi
        limits:
          cpu: 100m
          memory: 128Mi
```

### Why This Works

**Shared volume mechanism:** Both containers mount the same `emptyDir` volume named `app-logs` at the same path (`/var/log/app`). An emptyDir volume is created when the Pod is assigned to a node and exists as long as the Pod is running. It is shared between all containers that mount it, providing a common filesystem for inter-container communication.

**App container:** Runs an infinite loop that appends timestamped log lines to `/var/log/app/application.log` every 5 seconds. The `>>` operator appends rather than overwrites. The `mkdir -p` ensures the directory exists.

**Log-shipper container:** First waits for the log file to exist (since the app container may not have created it yet), then uses `tail -f` to follow the file. `tail -f` outputs new lines as they are appended and does not exit, keeping the container running. Any output to stdout from this container is captured by the kubelet and available via `kubectl logs sidecar-logging -c log-shipper`.

**Resource asymmetry:** The sidecar has significantly lower resource requests and limits than the main container. This is correct because the sidecar performs a simple operation (reading and forwarding text) while the main application does actual work.

## Part B: Resource Configuration

The resource definitions are included in the manifest above. Key observations:

- The app container gets 200m-500m CPU and 128Mi-256Mi memory.
- The log-shipper gets 50m-100m CPU and 64Mi-128Mi memory.
- The ratio reflects the actual work each container performs.
- In production, you would monitor actual usage with `kubectl top pod` and adjust these values based on observed patterns.

## Part C: Verification Answers

**1. Can you see the application's log output through the sidecar?**

Yes. Running `kubectl logs sidecar-logging -c log-shipper` shows the timestamped lines written by the app container. This is the entire point of the sidecar pattern -- the operations team can view application logs by querying the sidecar container's stdout, without needing to exec into the app container or access the volume directly.

**2. What if the main container writes logs faster than the sidecar can read?**

With `emptyDir`, this is generally not a concern for moderate log volumes because:
- Both containers share the same filesystem. There is no network overhead or serialization bottleneck.
- `tail -f` uses inotify on Linux, so it receives file change notifications immediately.
- The bottleneck would be stdout I/O of the sidecar container, which is very fast for text.

However, at extreme volumes (thousands of lines per second), the log file could grow until it fills the emptyDir volume, which uses the node's ephemeral storage. In production, you would configure log rotation at the application level or use a more sophisticated sidecar that compresses and ships logs to an external system.

**3. Why use a shared volume instead of reading from the same stdout stream?**

Several reasons:
- The application may write structured logs (JSON, logfmt) that need transformation before output.
- The application may not write to stdout by design (legacy code, third-party images, compliance requirements that mandate file-based logging).
- The sidecar can perform additional processing: filtering sensitive data, enriching logs with metadata, buffering during spikes, or shipping to multiple destinations.
- Decoupling: the application and the log infrastructure evolve independently. You can swap the log-shipper sidecar for a Fluentd sidecar without touching the application.

## Part D: Production Concerns

In a production environment, additional concerns for this sidecar pattern include:

**Log rotation:** The `emptyDir` volume has no built-in rotation. If the application writes logs indefinitely, the volume will fill up and the Pod will be evicted when ephemeral storage limits are exceeded. Solutions: configure the application to rotate logs, or use a sidecar that rotates the file.

**Volume size:** `emptyDir` volumes can optionally have a `sizeLimit` field. Without it, they can grow until the node's disk fills up. Always set a sizeLimit in production.

**Crash recovery:** If the app container crashes and restarts, it will start writing to the log file again. If it truncates the file on startup (using `>` instead of `>>`), the sidecar's `tail -f` may behave unexpectedly. The sidecar should be resilient to the file being truncated or replaced.

**Log format transformation:** A production sidecar might use Fluentd or Fluent Bit instead of a simple `tail -f`. These tools can parse structured logs, add metadata, filter events, and ship to multiple destinations (Elasticsearch, S3, stdout).

**Resource limits:** Without proper resource limits, a misbehaving sidecar could consume resources needed by the main application. Always set limits on sidecar containers.

### Common Mistakes to Avoid

- **Mounting volumes at conflicting paths.** If the app writes to `/var/log/app` and the sidecar mounts the same volume at a different path, the sidecar will not see the files. Both containers must mount the volume at the same path (or the sidecar must know the app's path).

- **Using `tail -F` instead of `tail -f`.** The `-F` flag follows the file by name (useful for rotation), but it also retries if the file is inaccessible. For this pattern, either flag works, but understanding the difference matters when log rotation is involved.

- **Not waiting for the log file.** If the sidecar starts `tail -f` before the app creates the file, `tail` may fail or show an error. The solution shown above waits for the file to exist before tailing.

- **Forgetting that emptyDir is ephemeral.** When the Pod is deleted, the emptyDir volume and all its contents are lost. This sidecar pattern is for real-time log forwarding, not log persistence. For durable storage, use a PersistentVolumeClaim or ship logs to an external system.

- **Running the sidecar as root unnecessarily.** Both containers need read/write access to the shared volume. In production, use a securityContext with specific UID/GID rather than running as root.

## Key Takeaway

The sidecar pattern is one of the fundamental multi-container patterns in Kubernetes. It separates cross-cutting concerns (logging, monitoring, networking) from business logic. The shared volume mechanism is simple, fast, and requires no network configuration. In production, replace the simple `tail -f` sidecar with a proper log shipper like Fluentd, Fluent Bit, or Vector for reliability and feature richness.
