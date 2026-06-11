# Module 26: Pods & Containers — The Smallest Deployable Unit

**Previous:** [Module 25: Architecture](../25-architecture/README.md)

---

## The Problem

You have containerized your application with Docker. You can run it locally, push images to a registry, and deploy to a single server. But in production, you need:

- Multiple instances running for high availability
- Automatic restart when a container crashes
- Scheduling across multiple machines
- Resource guarantees (CPU, memory)
- Containers that need to share a network namespace (e.g., a web server with a log collector)

Running `docker run` on each server manually does not scale. You need an orchestrator, and the fundamental unit Kubernetes orchestrates is **not** a container — it is a **Pod**.

---

## The Naive Way

Deploy containers directly on nodes using Docker commands or even docker-compose:

```bash
# On server 1
docker run -d --name web-1 -p 8080:80 myapp:v1
docker run -d --name web-2 -p 8081:80 myapp:v1

# On server 2
docker run -d --name web-3 -p 8080:80 myapp:v1
```

**What goes wrong:**

- No automatic restart if a container dies
- No resource isolation between workloads
- Port conflicts require manual coordination
- No health checks, no scheduling, no self-healing
- Scaling means SSH-ing into more servers

---

## The Right Way

### What Is a Pod?

A **Pod** is the smallest deployable unit in Kubernetes. It wraps one or more containers that:

- Share the same network namespace (same IP, same ports)
- Share the same volumes
- Are always co-scheduled on the same node
- Are created and destroyed together

**Most pods contain a single container.** Multi-container pods are used when containers are tightly coupled and need to communicate via localhost or shared filesystem.

### Why Not Just Containers?

| Concept | Container | Pod |
|---------|-----------|-----|
| Networking | Isolated per container | Shared namespace across containers in pod |
| Storage | Must be explicitly mounted per container | Shared volumes across containers in pod |
| Lifecycle | Independent | All containers in a pod share lifecycle |
| Scheduling | N/A | Kubernetes schedules pods to nodes |

### Pod Spec (YAML)

```yaml
# pod.yaml
apiVersion: v1
kind: Pod
metadata:
  name: nginx-pod
  labels:
    app: nginx
    environment: development
spec:
  containers:
    - name: nginx
      image: nginx:1.25
      ports:
        - containerPort: 80
      resources:
        requests:
          cpu: "100m"      # 0.1 CPU cores
          memory: "128Mi"
        limits:
          cpu: "250m"      # 0.25 CPU cores
          memory: "256Mi"
      livenessProbe:
        httpGet:
          path: /
          port: 80
        initialDelaySeconds: 10
        periodSeconds: 5
      readinessProbe:
        httpGet:
          path: /
          port: 80
        initialDelaySeconds: 5
        periodSeconds: 3
```

**Key fields:**

- `metadata.name` — unique pod name within the namespace
- `metadata.labels` — key-value pairs for selection and organization
- `spec.containers` — list of containers in the pod
- `resources.requests` — minimum resources the scheduler guarantees
- `resources.limits` — maximum resources allowed (container is throttled/killed if exceeded)
- `livenessProbe` — tells Kubernetes when to restart the container
- `readinessProbe` — tells Kubernetes when the container is ready to accept traffic

### Init Containers

Init containers run **before** the main containers start. They are used for setup tasks:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: app-with-init
spec:
  initContainers:
    - name: wait-for-db
      image: busybox:1.36
      command: ['sh', '-c', 'until nc -z postgres-service 5432; do echo waiting for db; sleep 2; done']
    - name: run-migrations
      image: myapp:v1
      command: ['./migrate.sh']
  containers:
    - name: app
      image: myapp:v1
      ports:
        - containerPort: 8080
```

**Rules for init containers:**
- They run to completion (must exit 0) before the next init container or main container starts
- If an init container fails, the pod restarts it (based on `restartPolicy`)
- They cannot have readiness probes (they are expected to exit)

### Sidecar Containers

Sidecars are auxiliary containers that run alongside the main container for the entire pod lifetime:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: app-with-sidecar
spec:
  containers:
    # Main application container
    - name: app
      image: myapp:v1
      ports:
        - containerPort: 8080
      volumeMounts:
        - name: shared-logs
          mountPath: /var/log/app

    # Sidecar: log collector (ships logs to a central system)
    - name: log-collector
      image: fluentd:v1.16
      volumeMounts:
        - name: shared-logs
          mountPath: /var/log/app
          readOnly: true

  volumes:
    - name: shared-logs
      emptyDir: {}
```

**Common sidecar patterns:**
- Log collectors (Fluentd, Filebeat)
- Service mesh proxies (Envoy, Istio sidecar)
- Configuration watchers (reloading config on change)

### Pod Lifecycle

```
Pending → Running → Succeeded
                    Failed
                    Unknown
```

| Phase | Meaning |
|-------|---------|
| **Pending** | Accepted by the cluster, but containers are not yet running (waiting for scheduling, image pull, or init containers) |
| **Running** | At least one container is running |
| **Succeeded** | All containers terminated with exit code 0 (for jobs) |
| **Failed** | All containers terminated, and at least one exited with non-zero |
| **Unknown** | Node state cannot be obtained (typically a communication issue) |

**Container states:** `Waiting`, `Running`, `Terminated`

### Resource Requests and Limits

```yaml
resources:
  requests:
    cpu: "250m"      # 0.25 cores — guaranteed minimum
    memory: "128Mi"  # 128 MiB — guaranteed minimum
  limits:
    cpu: "500m"      # 0.5 cores — throttled above this
    memory: "256Mi"  # 256 MiB — OOMKilled above this
```

**Behavior:**
- `requests` are used by the scheduler to place the pod on a node with enough capacity
- `limits` are enforced at runtime by the kernel
- CPU above limit → throttled (container slows down)
- Memory above limit → OOMKilled (container is killed and restarted)
- A container with no resource requests can be evicted under memory pressure

### Node Selectors and Tolerations

**Node selectors** restrict which nodes a pod can be scheduled on:

```yaml
spec:
  nodeSelector:
    disk: ssd
    gpu: "true"
```

Nodes must be labeled with matching labels:

```bash
kubectl label nodes node-1 disk=ssd gpu=true
```

**Tolerations** allow a pod to be scheduled on nodes with taints:

```yaml
# Node has a taint:
# kubectl taint nodes node-1 dedicated=gpu:NoSchedule

spec:
  tolerations:
    - key: "dedicated"
      operator: "Equal"
      value: "gpu"
      effect: "NoSchedule"
```

**Taint effects:**
- `NoSchedule` — new pods that don't tolerate the taint won't be scheduled
- `PreferNoSchedule` — Kubernetes tries to avoid scheduling but doesn't guarantee it
- `NoExecute` — existing pods without toleration are evicted

---

## The Production Way

### Multi-Container Pod Patterns

**Ambassador pattern** — proxy container that handles network connections:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: app-with-ambassador
spec:
  containers:
    - name: app
      image: myapp:v1
      env:
        - name: DB_HOST
          value: "localhost"  # Connects to localhost
        - name: DB_PORT
          value: "5432"
    - name: ambassador
      image: envoyproxy/envoy:v1.28
      ports:
        - containerPort: 5432
      # Ambassador proxies to the actual database
```

**Adapter pattern** — transforms output from the main container:

```yaml
containers:
  - name: app
    image: myapp:v1
    volumeMounts:
      - name: metrics
        mountPath: /var/metrics
  - name: metrics-adapter
    image: metrics-adapter:v1
    volumeMounts:
      - name: metrics
        mountPath: /var/metrics
        readOnly: true
    # Transforms app metrics into Prometheus format
```

### Pod Disruption Budget

Protect critical pods from voluntary disruptions (node drain, cluster upgrades):

```yaml
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: app-pdb
spec:
  minAvailable: 2        # At least 2 pods must always be running
  # OR
  # maxUnavailable: 1    # At most 1 pod can be unavailable
  selector:
    matchLabels:
      app: myapp
```

### Pod Priority and Preemption

```yaml
apiVersion: scheduling.k8s.io/v1
kind: PriorityClass
metadata:
  name: high-priority
value: 1000000
globalDefault: false
description: "Priority class for critical workloads"

---
apiVersion: v1
kind: Pod
metadata:
  name: critical-pod
spec:
  priorityClassName: high-priority
  containers:
    - name: app
      image: myapp:v1
```

### Security Context

```yaml
spec:
  securityContext:
    runAsNonRoot: true
    runAsUser: 1000
    runAsGroup: 3000
    fsGroup: 2000
  containers:
    - name: app
      image: myapp:v1
      securityContext:
        allowPrivilegeEscalation: false
        readOnlyRootFilesystem: true
        capabilities:
          drop:
            - ALL
```

---

## Hands-On Lab

### Prerequisites

- A running Kubernetes cluster (minikube, kind, or k3s)
- `kubectl` configured and working

### Exercise 1: Create a Basic Pod

```bash
# Create the pod
kubectl apply -f - <<EOF
apiVersion: v1
kind: Pod
metadata:
  name: nginx-pod
  labels:
    app: nginx
spec:
  containers:
    - name: nginx
      image: nginx:1.25
      ports:
        - containerPort: 80
      resources:
        requests:
          cpu: "100m"
          memory: "64Mi"
        limits:
          cpu: "200m"
          memory: "128Mi"
EOF

# Watch the pod status
kubectl get pods -w

# Describe the pod for detailed info
kubectl describe pod nginx-pod

# Check the logs
kubectl logs nginx-pod

# Execute a command in the pod
kubectl exec -it nginx-pod -- /bin/bash

# Clean up
kubectl delete pod nginx-pod
```

### Exercise 2: Multi-Container Pod (Sidecar)

```bash
kubectl apply -f - <<EOF
apiVersion: v1
kind: Pod
metadata:
  name: multi-container-pod
spec:
  containers:
    - name: writer
      image: busybox:1.36
      command: ['sh', '-c', 'while true; do echo "$(date) - hello from writer" >> /var/log/app.log; sleep 5; done']
      volumeMounts:
        - name: shared-log
          mountPath: /var/log
    - name: reader
      image: busybox:1.36
      command: ['sh', '-c', 'tail -f /var/log/app.log']
      volumeMounts:
        - name: shared-log
          mountPath: /var/log
          readOnly: true
  volumes:
    - name: shared-log
      emptyDir: {}
EOF

# Watch the reader container's output
kubectl logs multi-container-pod -c reader -f

# Clean up
kubectl delete pod multi-container-pod
```

### Exercise 3: Init Container

```bash
kubectl apply -f - <<EOF
apiVersion: v1
kind: Pod
metadata:
  name: init-container-pod
spec:
  initContainers:
    - name: init-myservice
      image: busybox:1.36
      command: ['sh', '-c', 'echo "Initialization complete!" > /work-dir/index.html']
      volumeMounts:
        - name: workdir
          mountPath: /work-dir
  containers:
    - name: nginx
      image: nginx:1.25
      volumeMounts:
        - name: workdir
          mountPath: /usr/share/nginx/html
  volumes:
    - name: workdir
      emptyDir: {}
EOF

# Verify the init container ran
kubectl describe pod init-container-pod | grep -A 5 "Init Containers"

# Port-forward and test
kubectl port-forward init-container-pod 8080:80 &
curl http://localhost:8080

# Clean up
kubectl delete pod init-container-pod
```

### Exercise 4: Pod with Probes

```bash
kubectl apply -f - <<EOF
apiVersion: v1
kind: Pod
metadata:
  name: probed-pod
spec:
  containers:
    - name: app
      image: nginx:1.25
      ports:
        - containerPort: 80
      livenessProbe:
        httpGet:
          path: /
          port: 80
        initialDelaySeconds: 5
        periodSeconds: 10
        failureThreshold: 3
      readinessProbe:
        httpGet:
          path: /
          port: 80
        initialDelaySeconds: 3
        periodSeconds: 5
EOF

# Observe the probe behavior
kubectl describe pod probed-pod | grep -A 10 "Conditions"
kubectl get events --field-selector involvedObject.name=probed-pod

# Clean up
kubectl delete pod probed-pod
```

### Exercise 5: Node Selectors

```bash
# List nodes and label one
kubectl get nodes
kubectl label nodes <node-name> disk=ssd

# Create a pod with nodeSelector
kubectl apply -f - <<EOF
apiVersion: v1
kind: Pod
metadata:
  name: ssd-pod
spec:
  nodeSelector:
    disk: ssd
  containers:
    - name: nginx
      image: nginx:1.25
EOF

# Verify it landed on the labeled node
kubectl get pod ssd-pod -o wide

# Clean up
kubectl delete pod ssd-pod
kubectl label nodes <node-name> disk-
```

---

## Verification Checklist

- [ ] Can explain why pods exist instead of just containers
- [ ] Can write a pod YAML spec from scratch
- [ ] Understand the difference between resource requests and limits
- [ ] Can use init containers for setup tasks
- [ ] Know the pod lifecycle phases
- [ ] Can set up liveness and readiness probes
- [ ] Understand node selectors and tolerations

---

## Limitation

You have learned how to create individual pods. But in production, you never manage pods directly. If a pod crashes, who creates a new one? If you need 5 replicas, do you create 5 YAML files? If you need to update your application, do you delete and recreate pods manually?

**Next:** [Module 27: Deployments & ReplicaSets](../27-deployments-and-replicasets/README.md) — Kubernetes manages pod lifecycle declaratively so you don't have to.
