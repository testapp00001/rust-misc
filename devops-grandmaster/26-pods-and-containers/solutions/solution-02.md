# Solution 02: Init Containers and Resource Limits

## Step 1: Base Pod Manifest

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: webapp-with-init
  labels:
    app: webapp
    tier: frontend
spec:
  containers:
    - name: webapp
      image: nginx:1.25
      ports:
        - containerPort: 80
```

This is the minimal valid Pod spec. The labels `app` and `tier` are conventions that help with Service selectors, monitoring, and organizational queries.

## Step 2: Adding Init Containers

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: webapp-with-init
  labels:
    app: webapp
    tier: frontend
spec:
  initContainers:
    - name: wait-for-db
      image: busybox:1.36
      command: ['sh', '-c', 'until nc -z postgres-service 5432; do echo waiting for database; sleep 2; done']
    - name: wait-for-mq
      image: busybox:1.36
      command: ['sh', '-c', 'until nc -z rabbitmq-service 5672; do echo waiting for message queue; sleep 2; done']
  containers:
    - name: webapp
      image: nginx:1.25
      ports:
        - containerPort: 80
```

Key points about init containers:
- They are defined under `spec.initContainers`, not `spec.containers`.
- They run in order: `wait-for-db` runs first. Only after it exits with code 0 does `wait-for-mq` start.
- Only after both init containers complete successfully does the main container (`webapp`) start.
- If any init container fails, the Pod restarts (the init container sequence starts over from the beginning).
- Init containers can use different images than the main container, which is useful for using minimal images like busybox for setup tasks.

## Step 3: Adding Resource Requests and Limits

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: webapp-with-init
  labels:
    app: webapp
    tier: frontend
spec:
  initContainers:
    - name: wait-for-db
      image: busybox:1.36
      command: ['sh', '-c', 'until nc -z postgres-service 5432; do echo waiting for database; sleep 2; done']
      resources:
        requests:
          cpu: 50m
          memory: 32Mi
        limits:
          cpu: 100m
          memory: 64Mi
    - name: wait-for-mq
      image: busybox:1.36
      command: ['sh', '-c', 'until nc -z rabbitmq-service 5672; do echo waiting for message queue; sleep 2; done']
      resources:
        requests:
          cpu: 50m
          memory: 32Mi
        limits:
          cpu: 100m
          memory: 64Mi
  containers:
    - name: webapp
      image: nginx:1.25
      ports:
        - containerPort: 80
      resources:
        requests:
          cpu: 250m
          memory: 128Mi
        limits:
          cpu: 500m
          memory: 256Mi
```

Resource field explanation:
- **requests**: The minimum amount the container needs. The scheduler uses requests to find a node with enough available resources. 50m means 50 millicores, which is 5% of one CPU core.
- **limits**: The maximum amount the container is allowed to use. If a container exceeds its memory limit, it is OOM-killed. If it exceeds its CPU limit, it is throttled (not killed).

## Step 4: Verification Answers

When you deploy and inspect the Pod:

**1. Pod phase during init container execution:**

The Pod is in **Pending** phase. While init containers are running, the Pod has not fully started. The `Initialized` condition is False. You can see this in `kubectl get pod` where the status shows "Init:0/2", "Init:1/2" as each init container progresses.

**2. Order of execution:**

The init containers run in the order they are listed in the YAML. `wait-for-db` runs first (connecting to postgres-service on port 5432), and only after it exits successfully does `wait-for-mq` start (connecting to rabbitmq-service on port 5672). This is a guaranteed sequential order.

**3. How to tell init containers completed successfully:**

In `kubectl describe pod`, under the "Init Containers" section, each init container shows:
- State: Terminated
- Reason: Completed
- Exit Code: 0

The "Conditions" section shows `Initialized: True`. The "Events" section shows "Started container wait-for-db", "Completed container wait-for-db", etc.

**4. Resource requests after init containers complete:**

Init container resource requests are only counted while the init containers are running. The scheduler uses the maximum of:
- The sum of all regular container requests
- The maximum single init container request

Once init containers complete and terminate, they no longer consume CPU or memory on the node. Their resource requests are released. This means init containers are efficient -- they use resources only during startup.

## Step 5: Failure Behavior Answers

With the modified init container connecting to a nonexistent host:

**1. Pod phase:**

The Pod stays in **Pending** phase. The status shows "Init:0/2" or "Init:CrashLoopBackOff". The `Initialized` condition remains False.

**2. Events:**

The events show repeated "Back-off restarting failed container" messages. You will see "Started container wait-for-db" followed by "Back-off restarting failed container" as the init container fails, the Pod restarts, and the backoff interval increases (10s, 20s, 40s, ...).

**3. Effect on main container:**

The main container (webapp/nginx) never starts. It will not be created at all until all init containers complete successfully. The Pod remains in its initialization phase indefinitely (or until the init container eventually connects, which will not happen in this case).

### Common Mistakes to Avoid

- **Putting init containers in `spec.containers`.** This is the most common syntax error. Init containers go in `spec.initContainers`. If you put them in `spec.containers`, they run as regular containers in parallel with your main container, defeating the purpose.

- **Not setting resource requests on init containers.** Without requests, the scheduler does not account for init container resource usage. If an init container needs significant resources, it could be scheduled on a node that does not have enough capacity, causing the init container to be OOM-killed or CPU-throttled.

- **Assuming init containers run once per Pod lifecycle.** If the Pod restarts (due to a liveness probe failure, for example), init containers run again. They run every time the Pod starts, not just the first time. Design init containers to be idempotent.

- **Using heavy images for init containers.** Init containers run briefly. Using large images wastes time downloading them. Use minimal images like busybox or alpine for init containers whenever possible.

- **Not handling the case where the dependency never becomes available.** The `until nc -z` pattern will loop forever if the service never comes up. In production, consider adding a timeout or maximum retry count to avoid stuck Pods.

## Key Takeaway

Init containers enforce startup ordering and separate setup logic from application logic. Resource requests and limits are not optional annotations -- they directly control scheduling decisions and runtime resource management. A Pod without resource requests is a Pod that the scheduler cannot plan for, which leads to unpredictable behavior under load.
