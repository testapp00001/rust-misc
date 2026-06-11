# Solution 01: Pod Lifecycle Phases

## Part A: Phase Definitions

| Phase | Definition | Conditions That Trigger It | Example Workload |
|-------|------------|---------------------------|------------------|
| Pending | The Pod has been accepted by the cluster, but one or more containers have not yet been created. This includes time spent scheduling, pulling images, and waiting for init containers. | 1. The Pod has not yet been scheduled to a node (no suitable node found, or scheduler has not processed it yet). 2. The Pod is scheduled but container images are still being pulled. 3. Init containers are still running. 4. Resource requests cannot be satisfied by any node. | A machine learning training job submitted to a cluster that is currently at capacity, waiting for a node with enough GPU resources to become available. |
| Running | The Pod has been bound to a node, all containers have been created, and at least one container is in a running or starting/restarting state. | 1. All init containers completed successfully and the main container(s) have started. 2. A container crashed but the restartPolicy allows restart, so it is in the process of restarting. | A web server handling HTTP traffic continuously. |
| Succeeded | All containers in the Pod have terminated with exit code 0, and no container will be restarted. | 1. A batch job completed successfully. 2. A cron job finished its work. 3. A data migration script ran to completion. | A nightly ETL job that processes a fixed dataset, transforms it, loads it into a data warehouse, and exits cleanly. |
| Failed | All containers in the Pod have terminated, and at least one container exited with a non-zero status or was terminated by the system. | 1. An application crashed due to a bug (exit code 1). 2. A container was OOM-killed (exit code 137). 3. A container exceeded its resource limits and was terminated by the kubelet. | A data processing script that encounters a corrupted input file, logs the error, and exits with code 1. |

## Part B: Transition Scenarios

**Scenario 1: Pod requesting 8 CPU cores, all nodes have only 4 cores available.**

The Pod will remain in **Pending** phase. The scheduler cannot find a node whose allocatable CPU resources meet the Pod's request of 8 cores. The Pod will stay Pending indefinitely until either a node with sufficient resources joins the cluster or the resource request is reduced. You can verify this with `kubectl describe pod` which will show "FailedScheduling" events with a message like "Insufficient cpu."

**Scenario 2: Batch job processing 1000 records, all containers exit with code 0.**

The Pod will enter **Succeeded** phase. When all containers in a Pod terminate with exit code 0, Kubernetes marks the Pod as Succeeded. The Pod object remains in the API server (it is not deleted), but no containers are running and no restarts occur (assuming restartPolicy is not Always, or the Pod is part of a Job).

**Scenario 3: Web server running indefinitely.**

The Pod will be in **Running** phase. At least one container is actively running (the web server), which is the definition of the Running phase. The Pod will stay in Running as long as the container continues to operate normally and the node remains healthy.

**Scenario 4: Script exits with code 1.**

The Pod will be in **Failed** phase (if restartPolicy is Never) or will cycle through **Running** and **Pending** states before settling in **CrashLoopBackOff** (if restartPolicy is Always). With restartPolicy: Always, Kubernetes restarts the container, it fails again, and exponential backoff kicks in. Eventually the Pod status shows CrashLoopBackOff but the phase is technically Running (since the container keeps restarting). With restartPolicy: Never or OnFailure with a Job, the Pod goes to Failed.

**Scenario 5: Node loses network connectivity.**

The Pod will be in **Unknown** phase. The kubelet on the node is responsible for reporting Pod status to the API server. When the control plane cannot reach the kubelet, it marks the Pod as Unknown. The Pod may still be running on the node, but the cluster cannot confirm its state. After a configurable timeout, the node controller will mark the node as NotReady and may evict the Pods to reschedule them on healthy nodes.

## Part C: Conditions vs Phases

**1. Pod conditions:**

The four standard Pod conditions are:
- **PodScheduled** -- The Pod has been scheduled to a node.
- **Initialized** -- All init containers have completed successfully.
- **ContainersReady** -- All containers in the Pod are ready.
- **Ready** -- The Pod can serve traffic. This is the aggregate condition that considers both readiness probes and container state.

Each condition has a status of True, False, or Unknown.

**2. Running phase with a problem condition:**

Yes. A Pod in the Running phase can have a Ready condition of False. This happens when a container is running but its readiness probe is failing. For example, a web server container is up but its `/ready` endpoint returns 503 because it has not finished loading its cache. The Pod phase is Running (at least one container is running), but the Ready condition is False, so the Pod will not receive traffic from Services.

**3. Why both phases and conditions exist:**

Phases provide a high-level summary of where the Pod is in its lifecycle. They are simple, easy to understand, and sufficient for basic monitoring. Conditions provide granular, machine-readable information about specific aspects of the Pod's state. They enable precise decision-making:

- A deployment controller checks the Ready condition to know when a Pod is available to serve traffic.
- The scheduler checks PodScheduled to know if it needs to act.
- Monitoring systems check ContainersReady to detect partially degraded Pods.

Phases are for humans. Conditions are for controllers and automation. Both are necessary because they serve different audiences and different levels of precision.

### Common Mistakes to Avoid

- **Confusing Running with Ready.** A Pod can be Running but not Ready. Readiness probes control traffic routing, not Pod phase.
- **Assuming Succeeded means healthy.** A Pod can reach Succeeded even if the process had errors that it handled internally without a non-zero exit code.
- **Forgetting about Unknown.** The Unknown phase is not a failure -- it means the control plane has lost contact with the node. The Pod may still be running fine on the node itself.
- **Treating Failed as permanent.** With restartPolicy: Always, a Pod that crashes will be restarted. Its phase may cycle between Running and a failed state. The Pod only stays in Failed permanently if restartPolicy is Never.

## Key Takeaway

Pod phases are a coarse-grained lifecycle indicator. They tell you the overall state but not the details. For debugging and automation, always check both the phase and the conditions. The real skill in Kubernetes operations is knowing which state transitions are normal and which indicate a problem.
