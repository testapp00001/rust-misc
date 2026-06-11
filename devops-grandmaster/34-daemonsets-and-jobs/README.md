# Module 34: DaemonSets & Jobs — Background Tasks and Node-Level Agents

## The Problem: Not Everything Is a Long-Running Web Server

Deployments and StatefulSets are designed for applications that run continuously. But two categories of work don't fit:

**Category 1: Node-Level Agents**
Every node needs specific pods running:
- Log collectors (Fluentd, Filebeat) — must read logs from every node
- Monitoring agents (Prometheus node-exporter) — must scrape metrics from every node
- Network plugins (CNI daemons) — must manage networking on every node
- Storage drivers — must attach volumes on every node

You can't use a Deployment because you can't control which nodes get pods, and scaling a Deployment is based on replica count, not node count.

**Category 2: Batch and Scheduled Tasks**
Some work runs once and stops:
- Database backups at 2 AM
- Image processing batch jobs
- Data migration scripts
- Report generation

A Deployment keeps restarting pods that exit. You need something that says "run this task, and when it finishes, stop."

## The Naive Way: Cron on Nodes / Deployment Hacks

### Naive Node-Level Agent

```yaml
# WRONG: Deployment for a log collector
apiVersion: apps/v1
kind: Deployment
metadata:
  name: fluentd
spec:
  replicas: 5              # How many? What if I add a 6th node?
  selector:
    matchLabels:
      app: fluentd
  template:
    spec:
      containers:
      - name: fluentd
        image: fluentd:latest
```

**Problems:**
- `replicas: 5` doesn't guarantee one per node
- Pods might stack on the same node (3 on node-1, 2 on node-2, 0 on node-3)
- Adding a node doesn't automatically get a pod
- Removing a node doesn't automatically clean up

### Naive Batch Job

```yaml
# WRONG: Deployment for a one-off task
apiVersion: apps/v1
kind: Deployment
metadata:
  name: db-backup
spec:
  replicas: 1
  template:
    spec:
      containers:
      - name: backup
        image: postgres:16
        command: ["pg_dump", "-h", "db", "-U", "postgres", ">", "/backup/dump.sql"]
```

**Problem:** When the backup finishes, the pod exits. The Deployment restarts it. It runs the backup again. Infinite loop.

## The Right Way: DaemonSet and Job

### DaemonSet: One Pod Per Node

A DaemonSet ensures that **exactly one copy** of a pod runs on every node (or a specific subset of nodes).

```
Cluster with 3 nodes:

Deployment (replicas: 3):        DaemonSet:
  Node 1: pod-aaa, pod-bbb       Node 1: fluentd-xyz
  Node 2: pod-ccc                 Node 2: fluentd-abc
  Node 3: (empty)                 Node 3: fluentd-def

Deployment: pods distributed unevenly, might miss nodes
DaemonSet: exactly one pod on every node
```

```yaml
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: fluentd
  namespace: kube-system
spec:
  selector:
    matchLabels:
      app: fluentd
  template:
    metadata:
      labels:
        app: fluentd
    spec:
      tolerations:
      - key: node-role.kubernetes.io/control-plane
        operator: Exists
        effect: NoSchedule     # Also run on control plane nodes
      containers:
      - name: fluentd
        image: fluentd/fluentd:v1.16
        volumeMounts:
        - name: varlog
          mountPath: /var/log
          readOnly: true
        - name: containers
          mountPath: /var/lib/docker/containers
          readOnly: true
      volumes:
      - name: varlog
        hostPath:
          path: /var/log
      - name: containers
        hostPath:
          path: /var/lib/docker/containers
```

DaemonSet behavior:
- When a new node joins the cluster, a pod is automatically scheduled on it
- When a node is removed, its pod is garbage collected
- You don't set `replicas` — the count equals the number of matching nodes
- Pods are created directly (no ReplicaSet in between)

### Job: Run to Completion

A Job creates one or more pods that run until they **successfully complete**.

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: db-backup
spec:
  template:
    spec:
      containers:
      - name: backup
        image: postgres:16
        command:
        - /bin/sh
        - -c
        - |
          pg_dump -h postgres -U postgres mydb > /backup/dump.sql
          echo "Backup complete at $(date)"
        volumeMounts:
        - name: backup-volume
          mountPath: /backup
      volumes:
      - name: backup-volume
        persistentVolumeClaim:
          claimName: backup-pvc
      restartPolicy: Never      # REQUIRED for Jobs (not Always)
  backoffLimit: 3               # Retry up to 3 times on failure
```

Job behavior:
- Creates a pod that runs the command
- If the pod exits with code 0, the Job is **Complete**
- If the pod exits with non-zero, the Job restarts it (up to `backoffLimit`)
- When the Job is complete, the pod is kept (for logs) until TTL cleans it

### CronJob: Scheduled Jobs

A CronJob runs a Job on a schedule, just like Unix cron.

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: nightly-backup
spec:
  schedule: "0 2 * * *"        # Every day at 2:00 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: backup
            image: postgres:16
            command: ["/bin/sh", "-c", "pg_dump -h postgres -U postgres mydb > /backup/dump.sql"]
          restartPolicy: Never
  successfulJobsHistoryLimit: 3    # Keep last 3 successful jobs
  failedJobsHistoryLimit: 1        # Keep last 1 failed job
  concurrencyPolicy: Forbid        # Don't start new job if previous is still running
  startingDeadlineSeconds: 200     # If missed by 200s, skip it
```

Cron schedule syntax:

```
┌───────────── minute (0-59)
│ ┌───────────── hour (0-23)
│ │ ┌───────────── day of month (1-31)
│ │ │ ┌───────────── month (1-12)
│ │ │ │ ┌───────────── day of week (0-6, Sun=0)
│ │ │ │ │
* * * * *

"0 2 * * *"      = every day at 2:00 AM
"*/5 * * * *"    = every 5 minutes
"0 0 * * 0"      = every Sunday at midnight
"0 2 * * 1-5"    = weekdays at 2:00 AM
"0 */6 * * *"    = every 6 hours
```

## The Production Way: Advanced Patterns

### DaemonSet Update Strategy

```yaml
spec:
  updateStrategy:
    type: RollingUpdate          # Update pods one at a time
    rollingUpdate:
      maxUnavailable: 1          # Only one node's pod is down at a time
```

Or for critical agents that should never be down:

```yaml
spec:
  updateStrategy:
    type: OnDelete               # Only update when you manually delete the pod
```

### DaemonSet Node Selection

Run a DaemonSet only on specific nodes:

```yaml
spec:
  template:
    spec:
      nodeSelector:
        node-type: gpu          # Only run on GPU nodes
      # OR
      affinity:
        nodeAffinity:
          requiredDuringSchedulingIgnoredDuringExecution:
            nodeSelectorTerms:
            - matchExpressions:
              - key: kubernetes.io/os
                operator: In
                values:
                - linux
```

### Job Parallelism

Run multiple pods in parallel for batch processing:

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: image-processor
spec:
  completions: 10       # Need 10 successful completions
  parallelism: 3        # Run 3 pods at a time
  template:
    spec:
      containers:
      - name: processor
        image: my-image-processor:latest
        env:
        - name: BATCH_ID
          valueFrom:
            fieldRef:
              fieldPath: metadata.name    # Each pod gets a unique ID
      restartPolicy: Never
```

```
Parallelism flow:
  Wave 1: [pod-1] [pod-2] [pod-3]     ← 3 running
  Wave 2: [pod-4] [pod-5] [pod-6]     ← when wave 1 finishes
  Wave 3: [pod-7] [pod-8] [pod-9]     ← when wave 2 finishes
  Wave 4: [pod-10]                     ← final
  Total completions needed: 10
```

### Job Completion Modes

```yaml
# Mode 1: Non-indexed (default) — all pods are identical
spec:
  completions: 5
  parallelism: 3

# Mode 2: Indexed — each pod gets a unique index (0, 1, 2, ...)
spec:
  completions: 5
  parallelism: 5
  completionMode: Indexed       # K8s 1.21+
```

Indexed jobs are useful when each pod processes a specific partition:

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: partition-processor
spec:
  completions: 4
  parallelism: 4
  completionMode: Indexed
  template:
    spec:
      containers:
      - name: worker
        image: my-worker:latest
        env:
        - name: JOB_COMPLETION_INDEX    # Kubernetes sets this automatically
          valueFrom:
            fieldRef:
              fieldPath: metadata.annotations['batch.kubernetes.io/job-completion-index']
      restartPolicy: Never
```

### Job Failure Handling

```yaml
spec:
  backoffLimit: 3               # Retry up to 3 times
  activeDeadlineSeconds: 3600   # Kill job after 1 hour (timeout)
  ttlSecondsAfterFinished: 86400  # Auto-delete completed job after 24 hours
```

```
Job States:
  Running    → pod is executing
  Complete   → all pods exited with code 0
  Failed     → backoffLimit exceeded or deadline exceeded
  Suspended  → job is paused (spec.suspend: true)
```

### CronJob Concurrency Policies

```yaml
spec:
  concurrencyPolicy: Forbid     # Skip new run if previous is still running
  # concurrencyPolicy: Allow    # Allow concurrent runs (default)
  # concurrencyPolicy: Replace  # Kill previous run, start new one
```

```
Forbid:  Run 1 [========]
         Run 2 (skipped)
         Run 3 [========]

Allow:   Run 1 [========]
         Run 2      [========]    ← runs concurrently
         Run 3            [========]

Replace: Run 1 [====]KILLED
         Run 2      [========]    ← replaces run 1
```

## Hands-On Lab: DaemonSet and CronJob

### Lab 1: Deploy a Log Collector DaemonSet

```bash
# Create namespace
kubectl create namespace logging

# Deploy a DaemonSet that runs on every node
cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: log-collector
  namespace: logging
  labels:
    app: log-collector
spec:
  selector:
    matchLabels:
      app: log-collector
  template:
    metadata:
      labels:
        app: log-collector
    spec:
      tolerations:
      - key: node-role.kubernetes.io/control-plane
        operator: Exists
        effect: NoSchedule
      containers:
      - name: logger
        image: busybox
        command:
        - /bin/sh
        - -c
        - |
          echo "Log collector started on \$(hostname)"
          while true; do
            echo "[\$(date)] Collecting logs from \$(hostname)..."
            ls /var/log/ 2>/dev/null | head -5
            sleep 60
          done
        volumeMounts:
        - name: host-logs
          mountPath: /var/log
          readOnly: true
        resources:
          requests:
            cpu: 10m
            memory: 16Mi
          limits:
            cpu: 50m
            memory: 64Mi
      volumes:
      - name: host-logs
        hostPath:
          path: /var/log
EOF

# Verify one pod per node
kubectl get pods -n logging -o wide
# You should see one log-collector pod on each node

# Check the logs
kubectl logs -n logging -l app=log-collector --tail=5

# Add a new node (if using kind/minikube) and watch the DaemonSet
# A new pod is automatically scheduled on the new node
```

### Lab 2: Deploy a Monitoring DaemonSet

```bash
# Deploy node-exporter on every node
cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: node-exporter
  namespace: logging
spec:
  selector:
    matchLabels:
      app: node-exporter
  template:
    metadata:
      labels:
        app: node-exporter
    spec:
      hostNetwork: true         # Use host network for metrics endpoint
      hostPID: true
      tolerations:
      - key: node-role.kubernetes.io/control-plane
        operator: Exists
        effect: NoSchedule
      containers:
      - name: node-exporter
        image: busybox
        command:
        - /bin/sh
        - -c
        - |
          echo "Node exporter on \$(hostname)"
          echo "Metrics available on port 9100"
          while true; do
            echo "# HELP node_cpu_seconds_total CPU seconds"
            echo "node_cpu_seconds_total \$(date +%s)"
            sleep 30
          done
        ports:
        - containerPort: 9100
          hostPort: 9100
        resources:
          requests:
            cpu: 10m
            memory: 16Mi
EOF

# Verify it runs on every node
kubectl get pods -n logging -l app=node-exporter -o wide
```

### Lab 3: Run a One-Off Backup Job

```bash
# Create a simple Job
cat <<EOF | kubectl apply -f -
apiVersion: batch/v1
kind: Job
metadata:
  name: data-export
  namespace: logging
spec:
  template:
    spec:
      containers:
      - name: export
        image: busybox
        command:
        - /bin/sh
        - -c
        - |
          echo "Starting data export..."
          echo "Exporting records 1-1000"
          sleep 5
          echo "Exporting records 1001-2000"
          sleep 5
          echo "Export complete. 2000 records exported."
      restartPolicy: Never
  backoffLimit: 2
EOF

# Watch the job run
kubectl get job -n logging -w
# STATUS: Running → Complete

# Check pod status
kubectl get pods -n logging -l job-name=data-export

# View the logs
kubectl logs -n logging job/data-export

# Job stays around for inspection
kubectl get jobs -n logging
```

### Lab 4: Run a Parallel Job

```bash
# Run 5 tasks, 3 at a time
cat <<EOF | kubectl apply -f -
apiVersion: batch/v1
kind: Job
metadata:
  name: batch-processor
  namespace: logging
spec:
  completions: 5
  parallelism: 3
  template:
    spec:
      containers:
      - name: processor
        image: busybox
        command:
        - /bin/sh
        - -c
        - |
          echo "Processing batch on \$(hostname)..."
          sleep \$((RANDOM % 10 + 5))
          echo "Batch complete on \$(hostname)"
      restartPolicy: Never
  backoffLimit: 3
EOF

# Watch pods come and go in waves
kubectl get pods -n logging -l job-name=batch-processor -w
# Wave 1: 3 pods running
# Wave 2: 2 pods running (after first 3 complete)
# All done when completions = 5

# Check final status
kubectl get job batch-processor -n logging
# COMPLETIONS: 5/5
```

### Lab 5: Create a Scheduled Backup CronJob

```bash
# Create a CronJob that runs every 2 minutes (for testing)
cat <<EOF | kubectl apply -f -
apiVersion: batch/v1
kind: CronJob
metadata:
  name: scheduled-backup
  namespace: logging
spec:
  schedule: "*/2 * * * *"        # Every 2 minutes
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: backup
            image: busybox
            command:
            - /bin/sh
            - -c
            - |
              echo "Starting backup at \$(date)"
              echo "Backing up database..."
              sleep 10
              echo "Backup complete at \$(date)"
          restartPolicy: Never
  successfulJobsHistoryLimit: 3
  failedJobsHistoryLimit: 1
  concurrencyPolicy: Forbid
  startingDeadlineSeconds: 60
EOF

# Wait 2-4 minutes and check for jobs
kubectl get cronjobs -n logging
kubectl get jobs -n logging

# Check job logs
kubectl logs -n logging -l app=scheduled-backup --tail=10

# See the schedule
kubectl describe cronjob scheduled-backup -n logging

# Suspend the CronJob (stop creating new jobs)
kubectl patch cronjob scheduled-backup -n logging -p '{"spec":{"suspend":true}}'

# Resume
kubectl patch cronjob scheduled-backup -n logging -p '{"spec":{"suspend":false}}'
```

### Lab 6: Cleanup

```bash
kubectl delete namespace logging
```

## Limitation: You Can Deploy Everything Manually, But It Doesn't Scale

You can now run any workload type: long-running services (Deployment), stateful apps (StatefulSet), node agents (DaemonSet), and batch tasks (Job/CronJob). But managing all these YAML files across environments is painful:

- 50+ YAML files for a production app
- Dev, staging, and prod configs differ slightly
- Installing third-party apps (Redis, NGINX Ingress, Prometheus) means copying and pasting YAML
- No versioning, no rollback, no dependency management

**Next problem:** How do you package, version, and deploy complex Kubernetes applications?

→ **Next module:** [35-helm-charts](../35-helm-charts/) — Package management for Kubernetes

## Checklist

- [ ] I understand why Deployments can't handle node-level agents or batch tasks
- [ ] I can create a DaemonSet that runs one pod per node
- [ ] I understand DaemonSet tolerations for control plane nodes
- [ ] I can create a Job that runs to completion
- [ ] I can configure Job parallelism and completion modes
- [ ] I can create a CronJob with proper concurrency policies
- [ ] I know the difference between restartPolicy: Always vs Never vs OnFailure
- [ ] I can handle Job failures with backoffLimit and activeDeadlineSeconds
