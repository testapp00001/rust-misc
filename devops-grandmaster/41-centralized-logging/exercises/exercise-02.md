# Exercise 02: Set Up Loki with Promtail for Kubernetes Log Collection

**Type:** Guided
**Time:** 45 minutes
**Difficulty:** Easy-Medium

## Objective

Deploy Loki and Promtail on a Kubernetes cluster using Helm, configure Promtail
to collect pod logs with appropriate labels, verify that logs are flowing into
Loki, and query them using basic LogQL.

## Background

In Kubernetes, every container writes logs to stdout and stderr. The container
runtime (containerd, CRI-O) captures these and writes them to files on the
node, typically under `/var/log/containers/`. Promtail is a log collection
agent that runs on every node as a DaemonSet, tails these log files, attaches
Kubernetes metadata as labels, and ships the logs to Loki. This exercise walks
you through deploying this pipeline on a local Kubernetes cluster.

---

## Tasks

### Part A: Set Up the Kubernetes Cluster

Create a local Kubernetes cluster and prepare the logging namespace.

1. Create a cluster using your preferred tool (minikube, kind, or k3d). If you
   already have a running cluster, you may use it.
2. Create a namespace called `logging`.
3. Verify the cluster is running and the namespace exists.

```bash
# Example for kind
kind create cluster --name loki-lab

# Create namespace
kubectl create namespace logging
```

<details>
<summary>Hint</summary>

- If using minikube: `minikube start --cpus=2 --memory=4096`
- If using kind: `kind create cluster --name loki-lab`
- If using k3d: `k3d cluster create loki-lab`
- Verify with `kubectl get nodes` and `kubectl get namespace logging`

</details>

### Part B: Deploy Loki Using Helm

Install Loki in simple (single-binary) mode using the official Grafana Helm
chart.

1. Add the Grafana Helm repository.
2. Create a `loki-values.yaml` file with configuration for simple mode,
   filesystem storage, and a 7-day retention period.
3. Install Loki in the `logging` namespace.
4. Verify Loki is running and its API responds.

```bash
# Add the Grafana Helm repo
helm repo add grafana https://grafana.github.io/helm-charts
helm repo update
```

Create the values file and install:

```bash
# Install Loki
helm install loki grafana/loki \
  --namespace logging \
  --values loki-values.yaml
```

Verify:

```bash
# Check pod is running
kubectl -n logging get pods

# Test the Loki API
kubectl -n logging port-forward svc/loki 3100:3100 &
curl http://localhost:3100/ready
```

<details>
<summary>Hint</summary>

- For Helm chart values, look at the `grafana/loki` chart documentation.
- Simple mode uses `deploymentMode: Simplescalable` or the legacy
  `singleBinary` configuration depending on chart version.
- Set `loki.storage.type: filesystem` for a local lab.
- Set `loki.limits_config.retention_period: 168h` for 7-day retention.
- The `/ready` endpoint should return "ready" when Loki is healthy.

</details>

### Part C: Deploy Promtail

Install Promtail as a DaemonSet that collects logs from all namespaces and
ships them to Loki.

1. Create a `promtail-values.yaml` file with configuration for Kubernetes
   log collection.
2. Install Promtail in the `logging` namespace.
3. Verify Promtail is running on every node.

```bash
# Install Promtail
helm install promtail grafana/promtail \
  --namespace logging \
  --values promtail-values.yaml
```

Verify:

```bash
# Check DaemonSet -- should have one pod per node
kubectl -n logging get daemonset promtail
kubectl -n logging get pods -l app.kubernetes.io/name=promtail
```

<details>
<summary>Hint</summary>

- The Promtail Helm chart configures Kubernetes service discovery by default.
- Set `config.clients[0].url` to `http://loki:3100/loki/api/v1/push`.
- Ensure Promtail has the correct RBAC permissions (the chart handles this).
- Check that the Promtail config includes `kubernetes_sd_configs` with
  `role: pod`.

</details>

### Part D: Generate Logs and Verify Collection

Deploy a sample application that produces structured JSON logs, then verify
the logs appear in Loki.

1. Create a Deployment YAML for a simple application that writes JSON log
   lines to stdout. The application should log at various levels (INFO, WARN,
   ERROR) and include fields like `service`, `user_id`, and `request_id`.
2. Apply the Deployment to the `default` namespace.
3. Verify the application is producing logs using `kubectl logs`.
4. Query Loki (via port-forward) to confirm the logs are collected.

Sample application (a simple shell loop that produces JSON logs):

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: log-generator
  namespace: default
spec:
  replicas: 2
  selector:
    matchLabels:
      app: log-generator
  template:
    metadata:
      labels:
        app: log-generator
    spec:
      containers:
        - name: log-gen
          image: busybox
          command:
            - /bin/sh
            - -c
            - |
              while true; do
                LEVEL="INFO"
                MSG="request_processed"
                if [ $((RANDOM % 10)) -eq 0 ]; then
                  LEVEL="ERROR"
                  MSG="payment_failed"
                fi
                if [ $((RANDOM % 20)) -eq 0 ]; then
                  LEVEL="WARN"
                  MSG="slow_query_detected"
                fi
                echo "{\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\"level\":\"${LEVEL}\",\"service\":\"order-api\",\"message\":\"${MSG}\",\"user_id\":\"user-$((RANDOM % 100))\",\"request_id\":\"req-$((RANDOM % 10000))\"}"
                sleep 1
              done
```

Query Loki:

```bash
# Port-forward Loki
kubectl -n logging port-forward svc/loki 3100:3100 &

# Query all logs from log-generator
curl -G http://localhost:3100/loki/api/v1/query_range \
  --data-urlencode 'query={app="log-generator"}' \
  --data-urlencode 'limit=10' | jq .
```

<details>
<summary>Hint</summary>

- The application label `app=log-generator` becomes a Loki label automatically
  via Promtail's Kubernetes service discovery.
- If logs do not appear, check Promtail's targets page:
  `kubectl -n logging port-forward svc/promtail 9080:9080` then open
  `http://localhost:9080/targets`.
- Check Promtail logs for errors: `kubectl -n logging logs -l app.kubernetes.io/name=promtail`
- Loki needs a few seconds to ingest and index logs. Wait 10-15 seconds
  after generating traffic before querying.

</details>

### Part E: Basic LogQL Queries

Using the Grafana UI or the Loki HTTP API, write LogQL queries to answer these
questions. Record each query you write.

1. Fetch all logs from the `log-generator` application.
2. Fetch only ERROR-level logs.
3. Fetch logs where the `user_id` is `user-42`.
4. Count the number of log lines per level over the last 5 minutes.
5. Find the most recent 5 ERROR logs and extract the `message` field.

<details>
<summary>Hint</summary>

- LogQL stream selectors use curly braces: `{app="log-generator"}`
- Use `| json` to parse JSON log lines into fields.
- Filter parsed fields with `|=` (contains) or field-level filters like
  `level="ERROR"`.
- Use `count_over_time({app="log-generator"} | json [5m])` for counts.
- Use `sum by (level) (count_over_time({app="log-generator"} | json [5m]))`
  to group by level.

</details>

---

## Success Criteria

- [ ] Loki is deployed and running in the `logging` namespace, and
      `curl http://localhost:3100/ready` returns "ready"
- [ ] Promtail is running as a DaemonSet with one pod per node, and its
      targets page shows active log sources
- [ ] The sample application produces structured JSON logs visible in
      `kubectl logs`
- [ ] Logs from the sample application appear in Loki when queried via the
      HTTP API or Grafana Explore
- [ ] You can write at least 3 different LogQL queries to filter, search, and
      aggregate logs

## What You Should Understand After This Exercise

Promtail discovers Kubernetes pods automatically via the Kubernetes API and
attaches metadata (namespace, pod name, node name, labels) as Loki labels.
This means you do not need to configure each application individually -- any
pod that writes to stdout is automatically collected. The labels you apply in
Promtail's relabel configuration determine how efficiently you can query logs
in Loki. Too few labels and every query scans everything; too many
high-cardinality labels and the index explodes.
