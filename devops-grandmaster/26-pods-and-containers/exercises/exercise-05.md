# Exercise 05: Multi-Container Application Design

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Design a complete set of Pod specifications for a multi-container application that integrates init containers, sidecar containers, resource management, and health checking. You will apply everything learned in this module and connect it to concepts from earlier modules on container images, logging, and graceful shutdown.

## Scenario

You are deploying an API gateway that consists of three functional components within a single Pod:

1. **Configuration Loader** (init container) -- fetches the latest API routing configuration from a ConfigMap and writes it to a shared volume
2. **API Gateway** (main container) -- reads the configuration and routes incoming HTTP requests to backend services
3. **Metrics Exporter** (sidecar container) -- scrapes metrics from the gateway's metrics endpoint and exposes them in Prometheus format on a separate port

The application has specific requirements:
- The gateway must not start until configuration is loaded and validated
- The metrics exporter must be available for scraping at all times the gateway is running
- Resource usage must be predictable and bounded
- The Pod must handle graceful shutdown properly
- Health checks must accurately reflect the actual state of each component

## Tasks

### Part A: Architecture Diagram

Before writing any YAML, draw an ASCII diagram showing:

- All containers in the Pod and their roles
- The shared volumes and which containers mount them
- The port mappings (gateway on 8080, metrics on 9090)
- The startup sequence (which container starts first)
- How the containers communicate (volumes, localhost, or both)

Create this diagram in a file named `architecture.txt`.

<details>
<summary>Hint -- Communication Patterns</summary>
Containers in the same Pod share the network namespace, so they can reach each other via localhost. They can also share volumes for file-based communication. The init container uses a volume to pass data to the main container. The sidecar uses localhost to scrape metrics from the main container.
</details>

### Part B: Pod Specification

Create a file named `api-gateway.yaml` with the complete Pod manifest.

**Init Container: `config-loader`**
- Image: `busybox:1.36`
- Command: Simulate fetching configuration by writing a JSON routing config to `/config/routes.json`
- Generate a config that maps `/api/v1/users` to `http://user-service:8080` and `/api/v1/orders` to `http://order-service:8080`
- Resource requests: 50m CPU, 32Mi memory
- Resource limits: 100m CPU, 64Mi memory

**Main Container: `gateway`**
- Image: `nginx:1.25`
- Use the configuration from the shared volume by mounting it into nginx's configuration directory
- Expose container port 8080
- Resource requests: 200m CPU, 256Mi memory
- Resource limits: 500m CPU, 512Mi memory
- Configure a liveness probe (httpGet on /healthz, port 8080, initial delay 10 seconds)
- Configure a readiness probe (httpGet on /healthz, port 8080, initial delay 5 seconds)
- Set a terminationGracePeriodSeconds of 30

**Sidecar Container: `metrics-exporter`**
- Image: `busybox:1.36`
- Command: Simulate metrics collection by running a loop that curls the gateway's metrics endpoint on localhost and outputs the result
- Expose container port 9090
- Resource requests: 50m CPU, 64Mi memory
- Resource limits: 100m CPU, 128Mi memory
- Configure a liveness probe (tcpSocket on port 9090)

**Volumes:**
- `config-volume` (emptyDir) -- shared between init container and main container
- `logs-volume` (emptyDir) -- shared between main container and sidecar container

<details>
<summary>Hint -- Volume Sharing Strategy</summary>
Each volume can be mounted by multiple containers. The config volume is written to by the init container and read by the main container. The logs volume is written to by the main container (if needed) and read by the sidecar. Use different mount paths in each container to avoid conflicts.
</details>

<details>
<summary>Hint -- Probe Configuration</summary>
For the gateway, use httpGet probes. For the metrics exporter, since it is a simple loop script, use a tcpSocket probe to verify the port is listening. The readiness probe determines whether traffic is sent to the Pod; the liveness probe determines whether the container is restarted.
</details>

### Part C: Resource Budget

The cluster node has 2 CPU cores and 4Gi memory available. Fill in the resource budget table and verify your Pod can be scheduled:

| Container | CPU Request | CPU Limit | Memory Request | Memory Limit |
|-----------|-------------|-----------|----------------|--------------|
| config-loader | | | | |
| gateway | | | | |
| metrics-exporter | | | | |
| **Total** | | | | |

Calculate: Does the total CPU request fit within 2000m? Does the total memory request fit within 4Gi?

<details>
<summary>Hint -- Requests vs Limits</summary>
Requests are what the scheduler uses to decide if the Pod fits on a node. Limits are enforced at runtime. Your total requests must be less than the node's allocatable resources. Limits can exceed requests (overcommit), but if all containers hit their limits simultaneously, the node may become unstable.
</details>

### Part D: Deployment and Testing

Deploy and test the complete Pod:

```bash
# Deploy
kubectl apply -f api-gateway.yaml

# Watch startup sequence
kubectl get pod api-gateway -w

# Verify all containers are running
kubectl get pod api-gateway -o wide

# Check init container completed
kubectl describe pod api-gateway | grep -A 3 "Init Containers"

# Verify configuration was loaded
kubectl exec api-gateway -c gateway -- cat /config/routes.json

# Check gateway health
kubectl exec api-gateway -c gateway -- curl -s http://localhost:8080/healthz

# Verify metrics exporter is running
kubectl logs api-gateway -c metrics-exporter --tail=5

# Check resource usage
kubectl top pod api-gateway
```

Record your observations for each command.

### Part E: Failure Simulation

Test the resilience of your design:

1. Delete the configuration file from the running container and observe what happens to the gateway
2. Describe the behavior of the metrics exporter if the gateway container crashes
3. Explain what happens to the Pod if the node runs out of memory

```bash
# Simulate configuration loss
kubectl exec api-gateway -c gateway -- rm /config/routes.json

# Observe behavior
kubectl describe pod api-gateway
kubectl logs api-gateway -c gateway --tail=20
```

<details>
<summary>Hint -- Container Failure Semantics</summary>
When a container in a Pod crashes, only that container restarts (not the entire Pod, unless restartPolicy is set to Always). Other containers in the Pod continue running. If a liveness probe fails, Kubernetes restarts just that container. If the node runs out of memory, the kubelet may evict Pods based on QoS class.
</details>

## Success Criteria

- [ ] Your architecture diagram clearly shows all containers, volumes, ports, and communication paths
- [ ] The YAML manifest defines all three containers with correct images, commands, and ports
- [ ] Init containers complete before the main container starts
- [ ] Resource requests and limits are set on all containers and total under the node capacity
- [ ] Liveness and readiness probes are configured appropriately for each container
- [ ] The Pod deploys successfully and all containers are healthy
- [ ] You can explain the failure behavior of each component

## What You Should Understand After This Exercise

Real Kubernetes applications rarely consist of a single container. Init containers handle setup, sidecars handle cross-cutting concerns, and the main container focuses on business logic. Designing a multi-container Pod requires thinking about startup ordering, resource budgets, health checking, and failure modes for each container independently and for the Pod as a whole. This exercise integrates all concepts from the module and connects them to broader application architecture concerns.
