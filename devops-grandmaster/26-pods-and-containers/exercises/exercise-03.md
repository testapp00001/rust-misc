# Exercise 03: Sidecar Logging Pattern

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective

Implement the sidecar container pattern by creating a Pod where a main application container writes logs to a shared volume and a sidecar container reads and forwards those logs. You will design the Pod spec from scratch, handle the shared volume mechanism, and verify that the sidecar is processing logs correctly.

## Scenario

Your application writes structured log entries to a file at `/var/log/app/application.log`. The operations team needs these logs streamed to stdout so that standard log collection tools (like Fluentd or the kubelet's built-in log capture) can pick them up. Rather than modifying the application code, you will deploy a sidecar container that tails the log file and outputs it to stdout.

## Tasks

### Part A: Design the Pod Specification

Create a file named `sidecar-logging.yaml` containing a Pod with:

**Main container: `app`**
- Image: `busybox:1.36`
- Command: A shell script that writes a timestamped log line to `/var/log/app/application.log` every 5 seconds
- The script should run in an infinite loop
- Mount a shared volume at `/var/log/app`

**Sidecar container: `log-shipper`**
- Image: `busybox:1.36`
- Command: Tail the log file at `/var/log/app/application.log` and output each line to stdout
- Mount the same shared volume at `/var/log/app`

**Shared volume:**
- Use an `emptyDir` volume that both containers mount

Build the entire manifest from scratch. Do not look at the solution until you have a complete YAML file.

<details>
<summary>Hint -- Shared Volume</summary>
Define a volume in `spec.volumes` with a name and `emptyDir: {}`. Then reference that volume name in both containers' `volumeMounts` with their respective mount paths. Both containers mount the same volume name but can use different sub-paths or the same path.
</details>

<details>
<summary>Hint -- The Tailing Command</summary>
The sidecar needs to tail a file that may not exist yet when the container starts. Use `tail -f` which will wait for the file to appear. The command would be something like `['sh', '-c', 'tail -f /var/log/app/application.log']`.
</details>

### Part B: Resource Configuration

Set appropriate resource requests and limits for both containers:

- The main app container is CPU-intensive: request 200m CPU, limit 500m CPU, request 128Mi memory, limit 256Mi memory
- The sidecar is lightweight: request 50m CPU, limit 100m CPU, request 64Mi memory, limit 128Mi memory

<details>
<summary>Hint -- Resource Asymmetry</summary>
Sidecar containers typically need far fewer resources than the main application. This is one reason to keep sidecar logic simple -- complex processing should happen in dedicated services, not in sidecar containers within the same Pod.
</details>

### Part C: Deploy and Verify

Apply your manifest and verify the sidecar is working:

```bash
# Deploy the Pod
kubectl apply -f sidecar-logging.yaml

# Wait for it to be ready
kubectl wait --for=condition=Ready pod/sidecar-logging --timeout=60s

# Check that both containers are running
kubectl get pod sidecar-logging

# View logs from the sidecar container -- you should see the app's log lines
kubectl logs sidecar-logging -c log-shipper

# Verify the main container is producing logs
kubectl logs sidecar-logging -c app
```

Answer these questions:

1. Can you see the application's log output through the sidecar container's logs?
2. What would happen if the main container writes logs faster than the sidecar can read them? Is this a concern with `emptyDir`?
3. Why use a shared volume instead of having the sidecar read from the same stdout stream?

<details>
<summary>Hint -- Why Not Just stdout?</summary>
Some applications write to files by design (log rotation, structured formats, compliance requirements). The sidecar pattern decouples log production from log collection. The application does not need to know about the log infrastructure, and the sidecar can be swapped without changing the application.
</details>

### Part D: Cleanup and Reflection

Delete the Pod after verification:

```bash
kubectl delete -f sidecar-logging.yaml
```

Then answer: In a production environment, what additional concerns would you have for this sidecar pattern? Consider log rotation, volume size, crash recovery, and log format transformation.

## Success Criteria

- [ ] Your YAML manifest defines a Pod with two containers sharing an emptyDir volume
- [ ] The main container writes logs to the shared volume
- [ ] The sidecar container reads and forwards those logs
- [ ] Both containers have independent resource requests and limits
- [ ] `kubectl logs` on the sidecar shows the application's log output
- [ ] You can explain when and why to use the sidecar logging pattern

## What You Should Understand After This Exercise

The sidecar pattern separates concerns within a Pod. The main container focuses on its primary function, while the sidecar handles cross-cutting concerns like log forwarding. This pattern extends to other use cases: configuration watchers, TLS certificate rotation, health monitoring, and network proxies. The key mechanism is the shared volume, which enables communication between containers without network overhead.
