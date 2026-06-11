# Exercise 02: Deploy a Logging Agent as a DaemonSet

## Objective

Deploy a logging agent (Fluentd) as a DaemonSet to collect logs from every node in your Kubernetes cluster and forward them to a central location.

## Background

Logging agents need to run on every node to collect container logs. Using a DaemonSet ensures that:
- Every node has exactly one logging agent pod
- New nodes automatically get the agent when they join the cluster
- Removed nodes automatically have their agent cleaned up

## Instructions

### Step 1: Create the Namespace

```bash
kubectl create namespace logging
```

### Step 2: Create a ConfigMap for Fluentd Configuration

Create a ConfigMap that configures Fluentd to:
- Read container logs from `/var/log/containers/`
- Add metadata (node name, pod name, namespace)
- Output logs to stdout (for testing; in production, you'd send to Elasticsearch or similar)

```yaml
# fluentd-configmap.yaml
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

Create the ConfigMap:

```bash
kubectl apply -f fluentd-configmap.yaml
```

### Step 3: Create the DaemonSet

Create a DaemonSet that:
- Runs Fluentd on every node
- Mounts the necessary host paths for log access
- Mounts the ConfigMap for configuration
- Uses resource limits to prevent runaway memory usage

Your DaemonSet should:
1. Be named `fluentd-agent` in the `logging` namespace
2. Use the `fluent/fluentd-kubernetes-daemonset:v1.16-debian-elasticsearch8-1` image
3. Mount `/var/log` from the host as read-only
4. Mount `/var/lib/docker/containers` from the host as read-only (for container logs)
5. Mount the ConfigMap at `/fluentd/etc/`
6. Set resource requests: 100m CPU, 200Mi memory
7. Set resource limits: 500m CPU, 500Mi memory
8. Use a toleration to run on all nodes (including control plane nodes)

### Step 4: Verify the DaemonSet

After deploying, verify:
1. The DaemonSet is created
2. Pods are running on all nodes
3. Logs are being collected (check pod logs)

### Step 5: Test Node Scaling

Add a new node to your cluster (if possible) and verify that the DaemonSet automatically deploys a pod to the new node.

## Deliverables

Create the following files:
1. `fluentd-configmap.yaml` - The ConfigMap configuration
2. `fluentd-daemonset.yaml` - The DaemonSet manifest

## Success Criteria

- [ ] DaemonSet is created and shows correct desired count matching node count
- [ ] Pods are running on all nodes (including control plane if tolerations are set)
- [ ] Pods have the correct volume mounts for host log access
- [ ] Fluentd is successfully reading container logs (check pod logs for activity)
- [ ] Resource limits are set to prevent memory issues

## Hints

<details>
<summary>Hint 1: Required Volume Mounts</summary>

Fluentd needs access to:
- `/var/log/containers/*.log` - Container log files
- `/var/lib/docker/containers/` - Docker container logs (if using Docker runtime)

Use `hostPath` volumes with `readOnly: true` for security.
</details>

<details>
<summary>Hint 2: Tolerations for Control Plane</summary>

To run on control plane nodes, add tolerations:

```yaml
tolerations:
  - key: node-role.kubernetes.io/control-plane
    operator: Exists
    effect: NoSchedule
  - key: node-role.kubernetes.io/master
    operator: Exists
    effect: NoSchedule
```

The second toleration is for older Kubernetes versions.
</details>

<details>
<summary>Hint 3: DaemonSet Template</summary>

Basic DaemonSet structure:

```yaml
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: fluentd-agent
  namespace: logging
spec:
  selector:
    matchLabels:
      app: fluentd
  template:
    metadata:
      labels:
        app: fluentd
    spec:
      containers:
        - name: fluentd
          image: fluent/fluentd-kubernetes-daemonset:v1.16-debian-elasticsearch8-1
          # ... rest of container spec
```

Note: DaemonSets don't have a `replicas` field - they automatically run on all nodes.
</details>

<details>
<summary>Hint 4: Checking DaemonSet Status</summary>

Use these commands to check your DaemonSet:

```bash
# Check DaemonSet status
kubectl get daemonset -n logging

# Check pods on each node
kubectl get pods -n logging -o wide

# Check pod logs for activity
kubectl logs -n logging <pod-name>

# Describe DaemonSet for events
kubectl describe daemonset fluentd-agent -n logging
```
</details>

<details>
<summary>Hint 5: Environment Variables</summary>

You may need to set these environment variables for Fluentd:

```yaml
env:
  - name: FLUENTD_CONF
    value: "fluent.conf"
  - name: FLUENT_ELASTICSEARCH_HOST
    value: "elasticsearch.logging.svc.cluster.local"
  - name: FLUENT_ELASTICSEARCH_PORT
    value: "9200"
```

For this exercise, we're outputting to stdout, so these aren't required.
</details>

## Common Issues

1. **Pods not running on control plane**: Add tolerations for the control plane taint
2. **Permission denied errors**: Ensure host paths are mounted correctly
3. **No logs collected**: Check that Fluentd is configured to read the correct log path
4. **High memory usage**: Set resource limits to prevent Fluentd from consuming too much memory
