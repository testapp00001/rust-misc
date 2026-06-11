# Solution 01: DaemonSet vs Deployment vs Job - When to Use Each

## Part 1: Conceptual Understanding - Answers

### 1. What is the primary difference between a DaemonSet and a Deployment with the same number of replicas as nodes?

**Answer:**

The primary difference is **scheduling intent**:

- **Deployment**: You specify "I want N replicas" and Kubernetes decides where to place them using its scheduler. If you have 5 nodes and 3 replicas, Kubernetes will choose which 3 nodes to use (based on resource availability, affinity rules, etc.).

- **DaemonSet**: You specify "I want exactly one pod on every node." Kubernetes automatically places a pod on each node and maintains that count as nodes are added or removed.

**Key distinction**: A Deployment with `replicas: 5` on a 5-node cluster *happens* to have one pod per node, but:
- If you add a 6th node, no new pod is created
- If you remove a node, Kubernetes will reschedule the pod to another node (maintaining 5 replicas)
- Pods can be scheduled on the same node if resources allow

With a DaemonSet:
- Adding a 6th node automatically creates a new pod
- Removing a node removes that node's pod (no rescheduling)
- Guaranteed one pod per node (never two on the same node)

### 2. When would you use a Job instead of a Deployment?

**Answer:**

Use a Job when:

1. **The task has a defined completion state** - e.g., processing a batch of files, running a migration, generating a report
2. **The task should not run continuously** - Jobs run until completion, then stop
3. **You need to ensure completion** - Jobs track success/failure and can retry
4. **The task is idempotent** - Can be safely retried without side effects

Use a Deployment when:

1. **The application should run indefinitely** - Web servers, API endpoints, long-running services
2. **You need to maintain a specific number of replicas** - For high availability or load distribution
3. **The application handles incoming requests** - It waits for work rather than processing a finite set

**Examples:**
- Job: Database migration, image processing, report generation
- Deployment: Web server, API gateway, message consumer

### 3. Can a DaemonSet be used to run a batch processing task? Why or why not?

**Answer:**

**Technically yes, but it's not the right tool.**

A DaemonSet *can* run batch tasks, but it's designed for **long-running, node-level services**, not finite tasks. Here's why it's not ideal:

**Problems with using DaemonSet for batch tasks:**

1. **No completion tracking**: DaemonSets don't have built-in success/failure tracking like Jobs
2. **No parallelism control**: You get one pod per node, regardless of how many you need
3. **No retry logic**: DaemonSets restart failed pods indefinitely, but don't track completion
4. **Resource waste**: If the task completes on a node, the pod stays running (or restarts)
5. **No aggregation**: No built-in way to combine results from all nodes

**When DaemonSets ARE appropriate for "batch-like" work:**

- Log collection (continuous, node-level)
- Monitoring agents (continuous, node-level)
- Cache warming (runs once per node, then serves requests)

**Better approach for batch tasks:**

Use a Job with `completions` and `parallelism` to control how many pods process the batch and how many run concurrently.

### 4. What happens to a DaemonSet pod if a new node is added to the cluster?

**Answer:**

When a new node is added:

1. **Kubernetes detects the new node** via the node controller
2. **DaemonSet controller checks** if a pod should run on this node (based on node selector, tolerations, etc.)
3. **If eligible, a new pod is automatically created** on the new node
4. **No manual intervention required** - this is the key benefit of DaemonSets

This automatic scaling is why DaemonSets are perfect for:
- Logging agents (need logs from every node)
- Monitoring agents (need metrics from every node)
- Network plugins (need to configure networking on every node)
- Storage agents (need to manage storage on every node)

**Example scenario:**
```
Initial state: 3 nodes, 3 DaemonSet pods
Add node 4: DaemonSet automatically creates pod on node 4
Remove node 3: DaemonSet pod on node 3 is deleted (not rescheduled)
```

### 5. How does a CronJob relate to a Job?

**Answer:**

A **CronJob creates Jobs** on a schedule. The relationship is:

```
CronJob (schedule: "0 2 * * *")
  └── Creates Job (at 2:00 AM daily)
       └── Creates Pod(s) (to execute the task)
```

**Key points:**

1. **CronJob is a template**: It defines *when* and *how* to create Jobs
2. **Job is the execution**: It defines *what* to run and manages completion
3. **Pod is the worker**: It actually executes the task

**CronJob features:**

- **Schedule**: Cron expression defining when to run
- **Concurrency Policy**: What to do if a new run is due while previous is still running
  - `Allow`: Run concurrently (default)
  - `Forbid`: Skip new run
  - `Replace`: Cancel old run, start new one
- **History Limits**: How many completed/failed Jobs to keep
- **Starting Deadline**: How late a Job can start before being skipped

**Example:**
```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: daily-backup
spec:
  schedule: "0 2 * * *"  # 2:00 AM daily
  concurrencyPolicy: Forbid  # Don't run if previous is still running
  successfulJobsHistoryLimit: 3  # Keep last 3 successful Jobs
  failedJobsHistoryLimit: 1  # Keep last 1 failed Job
  jobTemplate:
    spec:
      template:
        spec:
          containers:
            - name: backup
              image: backup-tool
          restartPolicy: OnFailure
```

---

## Part 2: Scenario Matching - Answers

### Scenario A: Collect logs from every node

**Workload Type: DaemonSet**

**Reasoning:**
- Logs are generated on every node
- Need exactly one collector per node
- Must automatically adapt to node changes
- Continuous operation (not a one-time task)

**Why not other types:**
- Deployment: Would place collectors on arbitrary nodes, missing some
- Job: Would complete and stop, but logs are continuous
- CronJob: Would run periodically, missing logs between runs

### Scenario B: Process 10,000 image files

**Workload Type: Job**

**Reasoning:**
- Finite task with clear completion
- Can be parallelized (multiple pods processing different files)
- Should retry on failure
- Needs to track completion progress

**Configuration:**
```yaml
spec:
  completions: 10000  # Total files to process
  parallelism: 10     # Process 10 at a time
  backoffLimit: 3     # Retry failed files up to 3 times
```

**Why not other types:**
- DaemonSet: Would only process one file per node, no parallelism control
- Deployment: Would run forever, even after all files are processed
- CronJob: No need for scheduling; just run once

### Scenario C: Web application with 3 replicas

**Workload Type: Deployment**

**Reasoning:**
- Long-running application
- Needs specific number of replicas for availability
- Handles incoming requests
- Should be updated/rolled back easily

**Why not other types:**
- DaemonSet: Would run on every node (too many replicas)
- Job: Would complete and stop, but web server should run forever
- CronJob: No scheduling needed; runs continuously

### Scenario D: Daily report at midnight

**Workload Type: CronJob**

**Reasoning:**
- Scheduled task (daily at midnight)
- One-time execution per day
- Should track history (success/failure)
- May need to handle missed runs

**Configuration:**
```yaml
spec:
  schedule: "0 0 * * *"
  concurrencyPolicy: Forbid
  successfulJobsHistoryLimit: 7
  failedJobsHistoryLimit: 3
```

**Why not other types:**
- DaemonSet: Would run on every node, but only need one report
- Deployment: Would run forever, generating reports continuously
- Job: Would need external scheduling; CronJob handles this

### Scenario E: Monitor network traffic on every node

**Workload Type: DaemonSet**

**Reasoning:**
- Need to monitor traffic on every node
- Network traffic is node-specific
- Continuous monitoring (not one-time)
- Must adapt to node changes

**Why not other types:**
- Deployment: Would miss some nodes
- Job: Would complete and stop, but monitoring is continuous
- CronJob: Would miss traffic between runs

### Scenario F: Database migration script

**Workload Type: Job**

**Reasoning:**
- One-time task before deployment
- Must complete successfully
- Should retry on failure
- Clear completion state

**Configuration:**
```yaml
spec:
  backoffLimit: 3
  activeDeadlineSeconds: 3600  # Timeout after 1 hour
  template:
    spec:
      restartPolicy: Never  # Don't restart on failure; let Job handle retries
```

**Why not other types:**
- DaemonSet: Would run migration on every node (wrong)
- Deployment: Would run forever (migrations are one-time)
- CronJob: No scheduling needed; run once

### Scenario G: Cache server on every node

**Workload Type: DaemonSet**

**Reasoning:**
- Need cache on every node for low latency
- Each node has its own cache instance
- Continuous operation
- Must adapt to node changes

**Why not other types:**
- Deployment: Would place cache on arbitrary nodes
- Job: Cache should run forever, not complete
- CronJob: No scheduling needed; runs continuously

---

## Part 3: Resource Behavior - Answers

### 1. Deployment with 3 replicas on 5 nodes

**Answer:**
- **3 pods run** (one per replica)
- **Kubernetes decides placement** based on:
  - Resource availability
  - Node affinity rules
  - Pod anti-affinity rules
  - Taints and tolerations
- **Possible distributions:**
  - 3 pods on 3 different nodes (most common)
  - 2 pods on one node, 1 on another (if resources allow)
  - All 3 on same node (unlikely, but possible if other nodes are full)

**Key insight:** Deployments don't guarantee node coverage; they guarantee replica count.

### 2. DaemonSet on 5 nodes

**Answer:**
- **5 pods run** (one per node)
- **Guaranteed placement:** One pod on each node
- **No choice:** Kubernetes must place a pod on every node (unless filtered by node selector/tolerations)

**Key insight:** DaemonSets guarantee node coverage, not replica count.

### 3. Job with completions: 5, parallelism: 2

**Answer:**

Kubernetes executes this as:

```
Time 0: Start pod 1, pod 2 (parallelism = 2)
Time 1: Pod 1 completes → Start pod 3
Time 2: Pod 2 completes → Start pod 4
Time 3: Pod 3 completes → Start pod 5
Time 4: Pod 4 completes
Time 5: Pod 5 completes → Job marked as complete
```

**Behavior:**
- **At most 2 pods run simultaneously** (parallelism)
- **Total of 5 pods must complete successfully** (completions)
- **Failed pods are retried** (up to backoffLimit)
- **Job completes when 5 pods succeed**

**Key insight:** `completions` is the total work; `parallelism` is the concurrency limit.

### 4. CronJob with schedule: "0 0 * * *"

**Answer:**

**When it runs:**
- **Daily at midnight UTC**
- Uses standard cron syntax: `minute hour day month weekday`
- `0 0 * * *` = minute 0, hour 0, every day, every month, every weekday

**If previous run hasn't finished:**

Depends on `concurrencyPolicy`:

| Policy | Behavior |
|--------|----------|
| `Allow` (default) | New Job starts concurrently with running Job |
| `Forbid` | New Job is skipped; running Job continues |
| `Replace` | Running Job is deleted; new Job starts |

**Example with Forbid policy:**
```
Day 1, 00:00: Job 1 starts
Day 1, 00:30: Job 1 still running
Day 2, 00:00: Job 2 should start, but Job 1 is running → Job 2 is skipped
Day 2, 00:45: Job 1 completes
Day 3, 00:00: Job 3 starts (no running Job)
```

**Key insight:** CronJobs use the `concurrencyPolicy` to handle overlapping runs.

---

## Common Mistakes to Avoid

1. **Using Deployment for node-level agents**
   - Mistake: `Deployment with replicas: <node-count>`
   - Problem: Doesn't adapt to node changes
   - Solution: Use DaemonSet

2. **Using DaemonSet for batch tasks**
   - Mistake: DaemonSet that runs a task and exits
   - Problem: Pod restarts indefinitely
   - Solution: Use Job with completions

3. **Using Job for long-running services**
   - Mistake: Job that runs a web server
   - Problem: Job completes when pod exits
   - Solution: Use Deployment

4. **Not setting backoffLimit on Jobs**
   - Mistake: Default backoffLimit (6) may be too high
   - Problem: Excessive retries on non-recoverable errors
   - Solution: Set appropriate backoffLimit

5. **Not setting concurrencyPolicy on CronJobs**
   - Mistake: Default `Allow` policy
   - Problem: Multiple instances running simultaneously
   - Solution: Use `Forbid` or `Replace` as needed
