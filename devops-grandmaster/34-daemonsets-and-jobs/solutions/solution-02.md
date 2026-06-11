# Solution 02: Deploy a Logging Agent as a DaemonSet

## Complete Solution

### fluentd-configmap.yaml

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: fluentd-config
  namespace: logging
data:
  fluent.conf: |
    # Read container logs
    <source>
      @type tail
      path /var/log/containers/*.log
      pos_file /var/log/fluentd-containers.log.pos
      tag kubernetes.*
      read_from_head true
      <parse>
        @type json
        time_key time
        time_format %Y-%m-%dT%H:%M:%S.%NZ
      </parse>
    </source>

    # Add Kubernetes metadata
    <filter kubernetes.**>
      @type record_transformer
      <record>
        hostname "#{Socket.gethostname}"
      </record>
    </filter>

    # Output to stdout (for testing)
    <match **>
      @type stdout
    </match>
```

### fluentd-daemonset.yaml

```yaml
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: fluentd-agent
  namespace: logging
  labels:
    app: fluentd
spec:
  selector:
    matchLabels:
      app: fluentd
  template:
    metadata:
      labels:
        app: fluentd
    spec:
      # Tolerations to run on all nodes, including control plane
      tolerations:
        - key: node-role.kubernetes.io/control-plane
          operator: Exists
          effect: NoSchedule
        - key: node-role.kubernetes.io/master
          operator: Exists
          effect: NoSchedule
      containers:
        - name: fluentd
          image: fluent/fluentd-kubernetes-daemonset:v1.16-debian-elasticsearch8-1
          env:
            - name: FLUENTD_CONF
              value: "fluent.conf"
          resources:
            requests:
              cpu: 100m
              memory: 200Mi
            limits:
              cpu: 500m
              memory: 500Mi
          volumeMounts:
            - name: varlog
              mountPath: /var/log
              readOnly: true
            - name: varlibdockercontainers
              mountPath: /var/lib/docker/containers
              readOnly: true
            - name: config
              mountPath: /fluentd/etc/
      volumes:
        - name: varlog
          hostPath:
            path: /var/log
        - name: varlibdockercontainers
          hostPath:
            path: /var/lib/docker/containers
        - name: config
          configMap:
            name: fluentd-config
```

## Why This Solution Works

### 1. DaemonSet Guarantees Node Coverage

The DaemonSet ensures that exactly one Fluentd pod runs on every node in the cluster. This is critical for log collection because:

- Logs are generated locally on each node
- Container logs are stored in `/var/log/containers/` on the node
- Each node needs its own collector to read local logs

Unlike a Deployment, a DaemonSet automatically adapts when nodes are added or removed.

### 2. Host Volume Mounts for Log Access

Fluentd needs access to two host directories:

- **`/var/log/containers/*.log`**: Where Kubernetes stores container logs
- **`/var/lib/docker/containers/`**: Where Docker stores container logs

These are mounted as read-only (`readOnly: true`) for security - Fluentd should only read logs, not modify them.

### 3. Tolerations for Control Plane Nodes

Control plane nodes are tainted to prevent regular workloads from running on them. However, logging agents should run on ALL nodes, including control plane. The tolerations allow Fluentd to be scheduled on tainted nodes:

```yaml
tolerations:
  - key: node-role.kubernetes.io/control-plane
    operator: Exists
    effect: NoSchedule
  - key: node-role.kubernetes.io/master
    operator: Exists
    effect: NoSchedule
```

The second toleration is for backward compatibility with older Kubernetes versions.

### 4. Resource Limits Prevent Memory Issues

Fluentd can consume significant memory when processing many logs. Setting limits prevents:

- Node resource exhaustion
- OOM kills affecting other pods
- Unbounded memory growth

**Recommended limits:**
- Requests: 100m CPU, 200Mi memory (guaranteed resources)
- Limits: 500m CPU, 500Mi memory (maximum allowed)

### 5. ConfigMap for Flexible Configuration

The ConfigMap allows:
- Easy configuration updates without rebuilding images
- Environment-specific configurations
- Version control of configuration

## Verification Steps

### 1. Check DaemonSet Status

```bash
kubectl get daemonset -n logging
```

Expected output:
```
NAME           DESIRED   CURRENT   READY   UP-TO-DATE   AVAILABLE   NODE SELECTOR   AGE
fluentd-agent   3         3         3       3            3           <none>          5m
```

**Key metrics:**
- `DESIRED`: Number of nodes (should match your node count)
- `CURRENT`: Number of pods currently running
- `READY`: Number of pods that are ready

### 2. Verify Pod Distribution

```bash
kubectl get pods -n logging -o wide
```

Expected output:
```
NAME                 READY   STATUS    RESTARTS   AGE   IP           NODE
fluentd-agent-abc12  1/1     Running   0          5m    10.244.0.5   node-1
fluentd-agent-def34  1/1     Running   0          5m    10.244.1.5   node-2
fluentd-agent-ghi56  1/1     Running   0          5m    10.244.2.5   node-3
```

**Verify:** One pod per node, all running.

### 3. Check Fluentd Logs

```bash
kubectl logs -n logging daemonset/fluentd-agent
```

You should see Fluentd starting up and beginning to tail log files.

### 4. Verify Log Collection

Create a test pod that generates logs:

```bash
kubectl run test-logger --image=busybox --restart=Never -- sh -c 'for i in $(seq 1 10); do echo "Test log message $i"; sleep 1; done'
```

Check Fluentd logs to see the captured logs:

```bash
kubectl logs -n logging daemonset/fluentd-agent | grep "Test log message"
```

## Common Mistakes to Avoid

### 1. Missing Tolerations for Control Plane

**Mistake:** Not including tolerations for control plane nodes.

**Problem:** Fluentd pods won't run on control plane nodes, missing their logs.

**Solution:** Always include tolerations for both `control-plane` and `master` taints.

### 2. Incorrect Volume Mount Paths

**Mistake:** Mounting to wrong paths or missing mounts.

**Problem:** Fluentd can't access container logs.

**Solution:** Verify the exact paths on your nodes:
```bash
# Check if paths exist on nodes
kubectl debug node/<node-name> -it --image=busybox -- ls -la /var/log/containers/
```

### 3. Not Setting Resource Limits

**Mistake:** No resource limits on Fluentd pods.

**Problem:** Fluentd consumes all available memory, causing OOM kills.

**Solution:** Always set both requests and limits based on your log volume.

### 4. Using Wrong Image Tag

**Mistake:** Using a generic Fluentd image without Kubernetes support.

**Problem:** Missing Kubernetes metadata or configuration issues.

**Solution:** Use the official `fluentd-kubernetes-daemonset` images which include:
- Kubernetes metadata filter
- Proper log parsing
- Optimized configuration

### 5. Not Mounting ConfigMap Correctly

**Mistake:** ConfigMap not mounted or wrong path.

**Problem:** Fluentd uses default configuration instead of custom one.

**Solution:** Verify ConfigMap is mounted:
```bash
kubectl describe pod -n logging <pod-name> | grep -A5 "Mounts:"
```

## Production Considerations

### 1. Log Rotation

Ensure log rotation is configured on nodes to prevent disk space issues:

```bash
# Check current log rotation
kubectl debug node/<node-name> -it --image=busybox -- cat /etc/logrotate.d/docker-containers
```

### 2. Buffer Configuration

For high-volume logs, add buffering to Fluentd:

```xml
<match **>
  @type file
  path /var/log/fluentd/buffer
  append true
  <buffer time>
    timekey 1h
    timekey_use_utc true
    chunk_limit_size 256MB
    flush_interval 30s
  </buffer>
</match>
```

### 3. Output Destinations

In production, send logs to a centralized system:

- **Elasticsearch**: For searchable logs
- **Loki**: For Grafana integration
- **Cloud Logging**: For cloud-native setups
- **Kafka**: For log streaming

### 4. Monitoring Fluentd

Add monitoring to track Fluentd health:

```yaml
livenessProbe:
  httpGet:
    path: /fluentd.pod.healthcheck
    port: 9880
  initialDelaySeconds: 30
  periodSeconds: 30
readinessProbe:
  httpGet:
    path: /fluentd.pod.healthcheck
    port: 9880
  initialDelaySeconds: 30
  periodSeconds: 30
```

### 5. Security Best Practices

- Run Fluentd as non-root user
- Use read-only root filesystem
- Drop all capabilities
- Use security context:

```yaml
securityContext:
  runAsUser: 0  # Fluentd needs root to read logs
  readOnlyRootFilesystem: true
  capabilities:
    drop:
      - ALL
```
