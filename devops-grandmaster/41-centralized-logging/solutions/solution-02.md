# Solution 02: Set Up Loki with Promtail for Kubernetes Log Collection

## Part A: Set Up the Kubernetes Cluster

```bash
# Create cluster (choose one)
kind create cluster --name loki-lab
# OR
minikube start --cpus=2 --memory=4096
# OR
k3d cluster create loki-lab

# Create namespace
kubectl create namespace logging

# Verify
kubectl get nodes
kubectl get namespace logging
```

**Why this works:** Each tool creates a local Kubernetes cluster with slightly
different characteristics. kind runs Kubernetes in Docker containers, minikube
runs a single-node VM, and k3d runs a lightweight k3s distribution in Docker.
For a logging lab, any of them work. The `logging` namespace isolates Loki
and Promtail from your application workloads.

## Part B: Deploy Loki Using Helm

Add the Helm repository:

```bash
helm repo add grafana https://grafana.github.io/helm-charts
helm repo update
```

Create `loki-values.yaml`:

```yaml
deploymentMode: SingleBinary

loki:
  auth_enabled: false

  commonConfig:
    replication_factor: 1

  storage:
    type: filesystem

  schemaConfig:
    configs:
      - from: "2024-01-01"
        store: tsdb
        object_store: filesystem
        schema: v13
        index:
          prefix: index_
          period: 24h

  limits_config:
    retention_period: 168h
    allow_structured_metadata: true

singleBinary:
  replicas: 1
  persistence:
    size: 10Gi

gateway:
  enabled: false

monitoring:
  selfMonitoring:
    enabled: false
    grafanaAgent:
      installOperator: false

backend:
  replicas: 0
read:
  replicas: 0
write:
  replicas: 0
```

Install:

```bash
helm install loki grafana/loki \
  --namespace logging \
  --values loki-values.yaml
```

Verify:

```bash
kubectl -n logging get pods
# Wait for the pod to be Running

kubectl -n logging port-forward svc/loki-gateway 3100:80 &
curl http://localhost:3100/ready
# Should return "ready"
```

**Why this works:** Single-binary mode runs Loki as one process instead of
separate read/write/backend components. This is appropriate for a lab
environment. The filesystem storage backend uses a PersistentVolume for log
data. The 168h retention period keeps logs for 7 days. The schema config uses
TSDB (time-series database) index format, which is the current recommended
schema for Loki.

## Part C: Deploy Promtail

Create `promtail-values.yaml`:

```yaml
config:
  clients:
    - url: http://loki:3100/loki/api/v1/push

  snippets:
    pipelineStages:
      - cri: {}
      - json:
          expressions:
            level: level
            service: service
            trace_id: trace_id
            message: message
      - labels:
          level:
          service:

tolerations:
  - effect: NoSchedule
    operator: Exists
```

Install:

```bash
helm install promtail grafana/promtail \
  --namespace logging \
  --values promtail-values.yaml
```

Verify:

```bash
kubectl -n logging get daemonset promtail
kubectl -n logging get pods -l app.kubernetes.io/name=promtail
```

**Why this works:** The Promtail Helm chart deploys a DaemonSet, which ensures
one Promtail pod runs on every node in the cluster. By default, the chart
configures Kubernetes service discovery with the `pod` role, which means
Promtail automatically discovers all pods and tails their log files under
`/var/log/containers/`. The pipeline stages parse CRI log format, then parse
JSON, and extract `level` and `service` as labels for efficient querying.

## Part D: Generate Logs and Verify Collection

Create the log generator deployment:

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
              i=0
              while true; do
                i=$((i + 1))
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
                echo "{\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\"level\":\"${LEVEL}\",\"service\":\"order-api\",\"namespace\":\"default\",\"message\":\"${MSG}\",\"user_id\":\"user-$((RANDOM % 100))\",\"request_id\":\"req-${i}\"}"
                sleep 1
              done
```

Apply and verify:

```bash
kubectl apply -f log-generator.yaml

# Wait for pods to start
kubectl get pods -l app=log-generator

# Check logs are being produced
kubectl logs -l app=log-generator --tail=5
```

Verify in Loki:

```bash
# Port-forward Loki
kubectl -n logging port-forward svc/loki 3100:3100 &

# Query all logs from log-generator
curl -G http://localhost:3100/loki/api/v1/query_range \
  --data-urlencode 'query={app="log-generator"}' \
  --data-urlencode 'limit=5' | jq '.data.result[0].values'

# If no results, check Promtail targets
kubectl -n logging port-forward svc/promtail 9080:9080 &
# Open http://localhost:9080/targets in a browser
```

**Why this works:** The busybox container runs a shell loop that produces one
structured JSON log line per second. The `app=log-generator` Kubernetes label
is automatically discovered by Promtail and becomes a Loki label. When you
query `{app="log-generator"}`, Loki uses the label index to find the relevant
chunks and returns the matching log lines.

**Troubleshooting:** If logs do not appear in Loki:
1. Check Promtail pods are running: `kubectl -n logging get pods`
2. Check Promtail logs for errors: `kubectl -n logging logs -l app.kubernetes.io/name=promtail`
3. Check Promtail targets at `http://localhost:9080/targets` -- the log-generator
   pods should appear as active targets.
4. Wait 15-30 seconds -- Loki batches writes and needs time to ingest.

## Part E: Basic LogQL Queries

### 1. All logs from log-generator

```logql
{app="log-generator"}
```

### 2. Only ERROR logs

```logql
{app="log-generator"} | json | level="ERROR"
```

### 3. Logs where user_id is user-42

```logql
{app="log-generator"} | json | user_id="user-42"
```

### 4. Count per level over 5 minutes

```logql
sum by (level) (count_over_time({app="log-generator"} | json [5m]))
```

### 5. Most recent 5 ERROR logs with message field

```logql
{app="log-generator"} | json | level="ERROR" | line_format "{{.timestamp}} {{.message}} {{.user_id}}"
```

With `limit=5` set in the query parameters, or use the Grafana Explore UI
which lets you set the line limit.

**Why this works:** LogQL has two parts: a *stream selector* in curly braces
that filters by labels, and a *pipeline* (after `|`) that processes the log
lines. The `json` stage parses the log line into fields, and subsequent
stages filter or format those fields. Metric queries like `count_over_time`
and `sum by` aggregate log lines into time-series data.

## Common Mistakes

- **Not waiting long enough before querying.** Loki batches log ingestion.
  After deploying the log generator, wait 15-30 seconds before querying.
  If you query immediately, you may get empty results and think something
  is broken.
- **Using the wrong Loki service name.** The Helm chart may create a service
  named `loki-gateway` or `loki` depending on the chart version. Check
  `kubectl -n logging get svc` to find the correct name.
- **Forgetting the `/loki/api/v1/push` path in Promtail config.** Promtail
  sends logs to this endpoint. If the URL is wrong, Promtail will log
  connection errors but keep retrying silently.
- **Not setting `auth_enabled: false` in Loki.** By default, Loki requires
  an `X-Scope-OrgID` header for multi-tenant authentication. In a lab
  environment, disable this to avoid confusing "no org id" errors.
- **Installing Promtail in a different namespace than Loki.** Promtail needs
  to reach Loki by service name. If they are in different namespaces, use the
  fully qualified DNS name (e.g., `loki.logging.svc.cluster.local`).
