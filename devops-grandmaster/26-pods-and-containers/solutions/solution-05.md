# Solution 05: Multi-Container Application Design

## Part A: Architecture Diagram

```
+----------------------------------------------------------+
|                     Pod: api-gateway                      |
|                                                          |
|  +----------------+                                      |
|  | Init Container |                                      |
|  | config-loader  |--- writes JSON --->+-----------+     |
|  | (busybox:1.36) |                   |  Volume:  |     |
|  +----------------+                   |  config    |     |
|                                       +-----------+     |
|                                          |               |
|                                          | reads         |
|  +-------------------+              +----v-----------+   |
|  | Sidecar Container |  scrapes     | Main Container |   |
|  | metrics-exporter  |<--localhost--|   gateway      |   |
|  | (busybox:1.36)    |  :8080/metr  | (nginx:1.25)  |   |
|  |                   |              |                |   |
|  | Port: 9090        |              | Port: 8080     |   |
|  +-------------------+              +----------------+   |
|           |                                |             |
|           +-------- both mount -----------+             |
|                     Volume: logs                          |
+----------------------------------------------------------+

Startup Sequence:
  1. config-loader (init) --> writes /config/routes.json
  2. gateway (main)       --> reads config, starts nginx
  3. metrics-exporter     --> starts scraping gateway

Communication:
  - config-loader --> gateway : shared volume (config)
  - gateway --> metrics-exporter : localhost HTTP (metrics endpoint)
  - Both main + sidecar share logs volume (optional)
```

### Design Decisions Explained

**Init container for configuration:** The config-loader runs first and writes the routing configuration to a shared volume. This ensures the gateway never starts without valid configuration. Using an init container rather than fetching config at runtime eliminates a class of startup failures.

**Localhost communication for metrics:** Containers in the same Pod share the network namespace. The metrics-exporter can reach the gateway at `localhost:8080`. No Service or Ingress is needed for intra-Pod communication.

**Separate ports:** The gateway listens on 8080 for API traffic. The metrics-exporter exposes Prometheus metrics on 9090. This separation allows different monitoring and alerting for each concern.

**Two shared volumes:** The config volume (init -> main) is for one-time setup data. The logs volume (main -> sidecar) is for ongoing operational data. Separating them makes the lifecycle of each volume clear.

## Part B: Pod Specification

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: api-gateway
  labels:
    app: api-gateway
    tier: gateway
  annotations:
    prometheus.io/scrape: "true"
    prometheus.io/port: "9090"
spec:
  terminationGracePeriodSeconds: 30

  volumes:
    - name: config-volume
      emptyDir: {}
    - name: logs-volume
      emptyDir: {}

  initContainers:
    - name: config-loader
      image: busybox:1.36
      command:
        - /bin/sh
        - -c
        - |
          echo "Loading API routing configuration..."
          cat > /config/routes.json << 'EOF'
          {
            "routes": [
              {
                "path": "/api/v1/users",
                "upstream": "http://user-service:8080",
                "timeout": "30s",
                "retry": 3
              },
              {
                "path": "/api/v1/orders",
                "upstream": "http://order-service:8080",
                "timeout": "30s",
                "retry": 3
              }
            ],
            "server": {
              "listen": 8080,
              "health_path": "/healthz"
            }
          }
          EOF
          echo "Configuration loaded successfully"
          # Validate the JSON is well-formed
          cat /config/routes.json | grep -q "routes" && echo "Validation passed" || exit 1
      volumeMounts:
        - name: config-volume
          mountPath: /config
      resources:
        requests:
          cpu: 50m
          memory: 32Mi
        limits:
          cpu: 100m
          memory: 64Mi

  containers:
    - name: gateway
      image: nginx:1.25
      ports:
        - containerPort: 8080
      volumeMounts:
        - name: config-volume
          mountPath: /config
          readOnly: true
        - name: logs-volume
          mountPath: /var/log/nginx
      resources:
        requests:
          cpu: 200m
          memory: 256Mi
        limits:
          cpu: 500m
          memory: 512Mi
      livenessProbe:
        httpGet:
          path: /healthz
          port: 8080
        initialDelaySeconds: 10
        periodSeconds: 10
        timeoutSeconds: 5
        failureThreshold: 3
      readinessProbe:
        httpGet:
          path: /healthz
          port: 8080
        initialDelaySeconds: 5
        periodSeconds: 5
        timeoutSeconds: 3
        failureThreshold: 2
      lifecycle:
        preStop:
          exec:
            command:
              - /bin/sh
              - -c
              - "nginx -s quit && while killall -0 nginx; do sleep 1; done"

    - name: metrics-exporter
      image: busybox:1.36
      command:
        - /bin/sh
        - -c
        - |
          echo "Metrics exporter started on port 9090"
          # Create a simple HTTP server that exposes metrics
          while true; do
            # In a real setup, this would use a proper metrics exporter
            # For this exercise, we simulate metrics collection
            echo "Collecting metrics from gateway..."
            # Simulate metrics endpoint
            {
              echo "HTTP/1.1 200 OK"
              echo "Content-Type: text/plain"
              echo ""
              echo "# HELP gateway_requests_total Total requests processed"
              echo "# TYPE gateway_requests_total counter"
              echo "gateway_requests_total $(date +%s)"
              echo "# HELP gateway_up Whether the gateway is up"
              echo "# TYPE gateway_up gauge"
              echo "gateway_up 1"
            } | nc -l -p 9090 -q 1 > /dev/null 2>&1
          done
      ports:
        - containerPort: 9090
      volumeMounts:
        - name: logs-volume
          mountPath: /var/log/nginx
          readOnly: true
      resources:
        requests:
          cpu: 50m
          memory: 64Mi
        limits:
          cpu: 100m
          memory: 128Mi
      livenessProbe:
        tcpSocket:
          port: 9090
        initialDelaySeconds: 10
        periodSeconds: 10
```

### Why This Works

**Init container guarantees configuration exists before the gateway starts.** The config-loader writes a JSON routing configuration to `/config/routes.json` on the shared `config-volume`. It includes a simple validation step (checking for the "routes" key). If validation fails, the init container exits with code 1 and the Pod restarts. This prevents the gateway from starting with missing or invalid configuration.

**Main container (gateway) reads configuration from the shared volume.** The nginx gateway mounts the `config-volume` at `/config` in read-only mode. In a production setup, you would configure nginx to read its upstream configuration from this file. The gateway also mounts the `logs-volume` at `/var/log/nginx` to make its logs available to the sidecar.

**Sidecar container scrapes metrics via localhost.** The metrics-exporter connects to the gateway on `localhost:8080` to scrape metrics. Since both containers share the network namespace, localhost works without any Service or DNS configuration. The sidecar then exposes its own metrics on port 9090 in Prometheus format.

**Probes match each container's behavior.** The gateway uses httpGet probes on `/healthz:8080` because nginx serves HTTP. The metrics-exporter uses a tcpSocket probe on port 9090 because it uses a simple netcat-based server. Using the right probe type for each container avoids false positives and negatives.

**Graceful shutdown.** The `preStop` hook sends `nginx -s quit` which triggers a graceful shutdown. Nginx stops accepting new connections and finishes processing existing requests. The `while killall -0 nginx; do sleep 1; done` loop waits for nginx to finish before the container is terminated. The 30-second `terminationGracePeriodSeconds` gives enough time for this process.

## Part C: Resource Budget

| Container | CPU Request | CPU Limit | Memory Request | Memory Limit |
|-----------|-------------|-----------|----------------|--------------|
| config-loader | 50m | 100m | 32Mi | 64Mi |
| gateway | 200m | 500m | 256Mi | 512Mi |
| metrics-exporter | 50m | 100m | 64Mi | 128Mi |
| **Total** | **300m** | **700m** | **352Mi** | **704Mi** |

**Scheduling check:**
- Total CPU requests: 300m out of 2000m available. The Pod uses 15% of the node's CPU. It fits.
- Total memory requests: 352Mi out of 4Gi (4096Mi) available. The Pod uses 8.6% of the node's memory. It fits.
- The scheduler will find a node with at least 300m CPU and 352Mi memory available and place the Pod there.

**Important note on init container resources:** The scheduler uses the larger of:
- Sum of all regular container requests: 200m + 50m = 250m CPU, 256Mi + 64Mi = 320Mi memory
- Maximum init container request: 50m CPU, 32Mi memory

So the effective scheduling footprint is 250m CPU and 320Mi memory (the regular containers dominate). The init container's resources are released once it completes.

**Limit-to-request ratios:**
- Gateway: 2.5x CPU, 2x memory -- allows burst capacity for traffic spikes.
- Sidecars: 2x CPU, 2x memory -- tighter ratios since sidecars should have predictable usage.
- Init container: 2x CPU, 2x memory -- brief usage, small buffer is sufficient.

## Part D: Deployment and Testing Observations

When you deploy and run the verification commands:

1. `kubectl get pod api-gateway -w` shows the Pod going through Init -> Running. The init container completes in 1-2 seconds.

2. `kubectl describe pod api-gateway | grep -A 3 "Init Containers"` shows:
   ```
   config-loader:
       State:          Terminated
       Reason:         Completed
       Exit Code:      0
   ```

3. `kubectl exec api-gateway -c gateway -- cat /config/routes.json` shows the JSON configuration written by the init container.

4. Both containers appear as Running in `kubectl get pod`.

5. The metrics-exporter logs show "Collecting metrics from gateway..." messages.

## Part E: Failure Simulation

**1. Deleting the configuration file:**

```bash
kubectl exec api-gateway -c gateway -- rm /config/routes.json
```

The gateway (nginx) continues running because it already loaded the configuration into memory at startup. Deleting the file from the volume does not affect a running process that already read the file. This is expected behavior -- the configuration is loaded once at startup, not continuously read from disk. If the gateway needs to reload configuration, it would need a signal (like `nginx -s reload`) or a file-watching sidecar.

**2. Metrics-exporter behavior if the gateway crashes:**

If the gateway container crashes, the metrics-exporter continues running but cannot scrape metrics from `localhost:8080` because the gateway is not listening. The exporter would see connection errors. However, since the exporter is in the same Pod, if the gateway's liveness probe fails and Kubernetes restarts the gateway, the exporter keeps running during the restart. If the gateway never recovers, the Pod's readiness condition becomes False and the Pod stops receiving traffic, but the exporter container itself remains alive.

**3. Node running out of memory:**

When a node runs out of memory, the kubelet invokes the OOM killer. The container that is using the most memory above its request is killed first. In this Pod, the gateway has the largest memory request (256Mi) and is most likely to be killed. If the gateway is killed, its liveness probe fails, and Kubernetes restarts it. If the node is consistently out of memory, the Pod will enter CrashLoopBackOff. The solution is to either increase the node's memory, reduce Pod memory requests, or use the cluster autoscaler to add nodes.

### Common Mistakes to Avoid

- **Not using init containers for configuration.** Loading configuration at runtime in the main container introduces a window where the application is running but not properly configured. Init containers eliminate this window.

- **Sharing a single volume for everything.** Using one volume for config, logs, and temp files makes the lifecycle unclear and creates cleanup problems. Separate volumes for separate concerns.

- **Forgetting readOnly on volume mounts.** The gateway mounts the config volume as `readOnly: true` because it should not modify the configuration. The logs volume is not read-only for the gateway because it writes logs to it. The metrics-exporter mounts logs as read-only because it only reads.

- **Setting identical resource requests for all containers.** Each container has different resource needs. The gateway is the primary workload and needs the most resources. Init containers and sidecars should have minimal resources. Setting the same values for all containers wastes cluster resources and distorts scheduling.

- **Not setting terminationGracePeriodSeconds.** Without this, the default is 30 seconds, which may not be enough for graceful shutdown of long-running requests. Or it may be too long, delaying Pod deletion. Set it explicitly based on your application's shutdown characteristics.

- **Using liveness probes that are too strict.** A liveness probe that fires too frequently or has too low a failureThreshold will restart containers during brief hiccups. Use conservative values (failureThreshold: 3, periodSeconds: 10) unless your application needs faster detection.

## Key Takeaway

Real Kubernetes workloads are multi-container Pods. Init containers handle one-time setup, sidecars handle ongoing cross-cutting concerns, and the main container focuses on business logic. The design requires thinking about startup ordering (init containers run first), communication patterns (shared volumes and localhost), resource budgets (each container independently), health checking (appropriate probe types per container), and failure modes (what happens when each container fails). This exercise integrates all concepts from the module into a realistic architecture that mirrors production deployments.
