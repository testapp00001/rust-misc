# Exercise 02: Init Containers and Resource Limits

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Create a Pod specification that uses init containers to perform setup work before the main application starts, and configure appropriate resource requests and limits for all containers. You will follow step-by-step instructions to build the manifest, apply it to a cluster, and verify the behavior.

## Scenario

You are deploying a web application that depends on two external services: a database and a message queue. Before the main application container starts, you need init containers that wait for both dependencies to be available. The application also needs defined resource boundaries so the scheduler can place it correctly and the cluster remains stable.

## Tasks

### Step 1: Create the Base Pod Manifest

Create a file named `init-pod.yaml` with the following structure:

- Pod name: `webapp-with-init`
- Namespace: `default`
- Labels: `app: webapp`, `tier: frontend`
- The main container should use the `nginx:1.25` image and expose container port 80

```yaml
# Write your base Pod spec here
```

<details>
<summary>Hint -- Basic Pod Structure</summary>
A minimal Pod spec needs apiVersion, kind, metadata (name, labels), and spec.containers (name, image, ports). The ports field uses a list with a containerPort entry.
</details>

### Step 2: Add Init Containers

Add two init containers to the Pod spec:

**Init Container 1: `wait-for-db`**
- Image: `busybox:1.36`
- Command: `['sh', '-c', 'until nc -z postgres-service 5432; do echo waiting for database; sleep 2; done']`

**Init Container 2: `wait-for-mq`**
- Image: `busybox:1.36`
- Command: `['sh', '-c', 'until nc -z rabbitmq-service 5672; do echo waiting for message queue; sleep 2; done']`

<details>
<summary>Hint -- Init Container Syntax</summary>
Init containers go in `spec.initContainers` (not `spec.containers`). The syntax is identical to regular containers -- name, image, command. Kubernetes runs them in order, and each must complete successfully before the next one starts.
</details>

### Step 3: Add Resource Requests and Limits

Configure resource constraints for all containers:

| Container | CPU Request | CPU Limit | Memory Request | Memory Limit |
|-----------|-------------|-----------|----------------|--------------|
| wait-for-db | 50m | 100m | 32Mi | 64Mi |
| wait-for-mq | 50m | 100m | 32Mi | 64Mi |
| webapp | 250m | 500m | 128Mi | 256Mi |

<details>
<summary>Hint -- Resource Syntax</summary>
Resources go under `spec.containers[].resources` (and `spec.initContainers[].resources`). The structure has two keys: `requests` and `limits`. Each key takes `cpu` and `memory` values. CPU is measured in millicores (m), memory in binary units (Mi, Gi).
</details>

### Step 4: Apply and Verify

Run the following commands and record the output:

```bash
# Apply the manifest
kubectl apply -f init-pod.yaml

# Watch the Pod status (init containers run first)
kubectl get pod webapp-with-init -w

# Describe the Pod to see init container details
kubectl describe pod webapp-with-init

# Check init container logs (they complete and stop, but logs remain)
kubectl logs webapp-with-init -c wait-for-db
kubectl logs webapp-with-init -c wait-for-mq
```

Answer these questions based on the output:

1. What Pod phase was the Pod in while init containers were running?
2. In what order did the init containers execute?
3. How can you tell from `kubectl describe` that init containers completed successfully?
4. What happened to resource requests on the node after the init containers completed?

<details>
<summary>Hint -- Reading Describe Output</summary>
In `kubectl describe`, look at the "Init Containers" section. Each init container will show its State as "Terminated" with Reason "Completed" and Exit Code 0. The "Conditions" section shows whether Initialized is True. The "Events" section at the bottom shows the sequence of events.
</details>

### Step 5: Test Failure Behavior

Delete the Pod, then modify the first init container's command to connect to a non-existent host:

```yaml
command: ['sh', '-c', 'until nc -z nonexistent-host 9999; do echo waiting; sleep 2; done']
```

Apply the modified manifest and observe:

1. What phase does the Pod stay in?
2. What do the events show?
3. How does this affect the main container?

After observing, delete the Pod and restore the original command.

<details>
<summary>Hint -- Init Container Failure</summary>
If an init container fails, Kubernetes restarts the Pod (not just the init container). The Pod stays in Pending or Init phase. Check `kubectl describe` events and `kubectl get pod` status for the Init:0/2 indicator.
</details>

## Success Criteria

- [ ] Your YAML manifest has a valid Pod spec with one main container and two init containers
- [ ] Resource requests and limits are set on all three containers
- [ ] The Pod runs successfully in your cluster with both init containers completing
- [ ] You can explain what happens when an init container fails
- [ ] You can read `kubectl describe` output to determine init container status

## What You Should Understand After This Exercise

Init containers are a powerful mechanism for separating setup logic from application logic. They run to completion before the main containers start, they run sequentially, and they have their own resource definitions. Resource requests and limits are not just best practice -- they directly affect scheduling decisions and runtime behavior.
