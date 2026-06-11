# Solution 03: Spot Instance Strategy

## Problem Statement

Design a complete spot instance strategy for a Kubernetes cluster using
Karpenter. The solution must include node pool configuration, workload
tolerations, graceful shutdown handling, a checkpoint/resume pattern for
long-running jobs, and a cost comparison.

## Complete Solution

### Part 1: Karpenter NodePool for Spot Instances

```yaml
apiVersion: karpenter.sh/v1beta1
kind: NodePool
metadata:
  name: spot-general
spec:
  template:
    metadata:
      labels:
        capacity-type: spot
        cost-team: platform
    spec:
      # Taint spot nodes so only tolerant pods schedule here
      taints:
        - key: karpenter.sh/capacity-type
          value: spot
          effect: NoSchedule

      requirements:
        # Allow multiple instance families for availability
        - key: karpenter.k8s.aws/instance-family
          operator: In
          values:
            - m5
            - m5a
            - m6i
            - m6a
            - m7i
            - c5
            - c6i
            - r5
            - r6i
        - key: karpenter.k8s.aws/instance-size
          operator: In
          values:
            - xlarge
            - 2xlarge
            - 4xlarge
        # Spot only
        - key: karpenter.sh/capacity-type
          operator: In
          values:
            - spot
        # Spread across AZs
        - key: topology.kubernetes.io/zone
          operator: In
          values:
            - us-east-1a
            - us-east-1b
            - us-east-1c
        - key: kubernetes.io/arch
          operator: In
          values:
            - amd64

      # Kubelet configuration optimized for cost
      kubelet:
        maxPods: 110
        evictionHard:
          memory.available: "200Mi"
          nodefs.available: "10%"
        evictionSoft:
          memory.available: "500Mi"
          nodefs.available: "15%"
        evictionSoftGracePeriod:
          memory.available: "1m"
          nodefs.available: "1m"

  # Disruption policy -- consolidate underutilized spot nodes
  disruption:
    consolidationPolicy: WhenUnderutilized
    expireAfter: 720h  # 30 days max node lifetime

  # Limits to prevent runaway provisioning
  limits:
    cpu: "500"
    memory: "1000Gi"

  # Weight: lower than on-demand pool so spot is tried first
  # but falls back to on-demand if spot is unavailable
  weight: 10
---
# On-demand fallback pool
apiVersion: karpenter.sh/v1beta1
kind: NodePool
metadata:
  name: on-demand-fallback
spec:
  template:
    metadata:
      labels:
        capacity-type: on-demand
    spec:
      requirements:
        - key: karpenter.k8s.aws/instance-family
          operator: In
          values:
            - m5
            - m6i
        - key: karpenter.k8s.aws/instance-size
          operator: In
          values:
            - xlarge
            - 2xlarge
        - key: karpenter.sh/capacity-type
          operator: In
          values:
            - on-demand
        - key: topology.kubernetes.io/zone
          operator: In
          values:
            - us-east-1a
            - us-east-1b
            - us-east-1c

  disruption:
    consolidationPolicy: WhenUnderutilized

  limits:
    cpu: "200"
    memory: "400Gi"

  # Higher weight = lower priority (spot preferred via lower weight)
  weight: 50
```

### Part 2: Deployment with Spot Tolerations and Affinity

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-worker
  namespace: production
  labels:
    app: api-worker
    cost-tier: spot-eligible
spec:
  replicas: 6
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 2
      maxSurge: 2
  selector:
    matchLabels:
      app: api-worker
  template:
    metadata:
      labels:
        app: api-worker
        cost-tier: spot-eligible
    spec:
      # Tolerate the spot taint
      tolerations:
        - key: karpenter.sh/capacity-type
          value: spot
          effect: NoSchedule

      # Prefer spot, but allow on-demand if needed
      affinity:
        nodeAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
            - weight: 100
              preference:
                matchExpressions:
                  - key: karpenter.sh/capacity-type
                    operator: In
                    values:
                      - spot
            - weight: 50
              preference:
                matchExpressions:
                  - key: kubernetes.io/arch
                    operator: In
                    values:
                      - amd64
        # Spread pods across nodes and zones
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
            - weight: 100
              podAffinityTerm:
                labelSelector:
                  matchExpressions:
                    - key: app
                      operator: In
                      values:
                        - api-worker
                topologyKey: topology.kubernetes.io/zone

      topologySpreadConstraints:
        - maxSkew: 1
          topologyKey: topology.kubernetes.io/zone
          whenUnsatisfiable: ScheduleAnyway
          labelSelector:
            matchLabels:
              app: api-worker

      terminationGracePeriodSeconds: 120

      containers:
        - name: worker
          image: registry.example.com/api-worker:v2.4.1
          ports:
            - containerPort: 8080

          resources:
            requests:
              cpu: 500m
              memory: 512Mi
            limits:
              cpu: 1
              memory: 1Gi

          lifecycle:
            preStop:
              exec:
                # Drain in-flight requests before termination
                command:
                  - /bin/sh
                  - -c
                  - |
                    echo "Received SIGTERM -- starting graceful shutdown"
                    
                    # 1. Stop accepting new connections
                    /app/drain-connections --timeout 30
                    
                    # 2. Flush in-progress work to the queue
                    /app/checkpoint-queue --output /tmp/queue-state.json
                    
                    # 3. Upload checkpoint to S3
                    aws s3 cp /tmp/queue-state.json \
                      s3://my-checkpoints/api-worker/${HOSTNAME}/queue-state.json
                    
                    # 4. Deregister from service discovery
                    curl -X POST http://consul:8500/v1/agent/service/deregister/${HOSTNAME}
                    
                    echo "Graceful shutdown complete"

          env:
            - name: NODE_NAME
              valueFrom:
                fieldRef:
                  fieldPath: spec.nodeName
            - name: POD_NAME
              valueFrom:
                fieldRef:
                  fieldPath: metadata.name

          readinessProbe:
            httpGet:
              path: /healthz
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 5

          livenessProbe:
            httpGet:
              path: /healthz
              port: 8080
            initialDelaySeconds: 15
            periodSeconds: 10
```

### Part 3: Graceful Shutdown Lifecycle Hooks

Spot instances receive a 2-minute warning before termination. The pod must
handle this signal within `terminationGracePeriodSeconds`.

```
Spot Termination Timeline
==========================

T+0:00  AWS sends interruption notice (2 min warning)
        Karpenter detects via EC2 Spot Instance Advisor / SQS

T+0:05  Karpenter marks node as Unschedulable (cordoned)
        K8s starts evicting pods

T+0:10  Pod receives SIGTERM
        preStop hook begins executing
        Readiness probe starts failing (stop new traffic)

T+0:30  preStop hook: in-flight requests drained
        preStop hook: state checkpointed to S3

T+0:45  Pod is removed from Service endpoints
        No new traffic reaches this pod

T+1:00  preStop hook completes
        Container receives final shutdown signal

T+1:30  Replacement pod scheduled on a new node
        Replacement pod passes readiness probe

T+2:00  AWS terminates the spot instance
        Node removed from cluster

        TARGET: Zero dropped requests (replacement ready before termination)
```

**Application-level graceful shutdown (Go example):**

```go
package main

import (
    "context"
    "log"
    "net/http"
    "os"
    "os/signal"
    "syscall"
    "time"
)

func main() {
    mux := http.NewServeMux()
    mux.HandleFunc("/healthz", healthHandler)
    mux.HandleFunc("/api/work", workHandler)

    server := &http.Server{
        Addr:    ":8080",
        Handler: mux,
    }

    // Channel to receive OS signals
    stop := make(chan os.Signal, 1)
    signal.Notify(stop, syscall.SIGTERM, syscall.SIGINT)

    // Start server in goroutine
    go func() {
        log.Println("Server starting on :8080")
        if err := server.ListenAndServe(); err != http.ErrServerClosed {
            log.Fatalf("Server error: %v", err)
        }
    }()

    // Wait for termination signal
    sig := <-stop
    log.Printf("Received signal %v -- beginning graceful shutdown", sig)

    // Mark as not-ready (remove from load balancer)
    // This is handled by the readiness probe failing
    healthy = false

    // Give in-flight requests up to 60 seconds to complete
    ctx, cancel := context.WithTimeout(context.Background(), 60*time.Second)
    defer cancel()

    // Stop accepting new connections, drain existing
    if err := server.Shutdown(ctx); err != nil {
        log.Printf("Shutdown error: %v", err)
    }

    // Checkpoint any pending work to external storage
    if err := checkpointPendingWork(ctx); err != nil {
        log.Printf("Checkpoint error: %v", err)
    }

    log.Println("Graceful shutdown complete")
}
```

### Part 4: Checkpoint/Resume Pattern for Long-Running Jobs

For jobs that process large datasets and cannot finish within the typical
spot interruption window:

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: data-processor
  namespace: production
spec:
  backoffLimit: 10        # Allow retries on spot interruption
  activeDeadlineSeconds: 86400  # 24h max
  ttlSecondsAfterFinished: 3600
  template:
    metadata:
      labels:
        app: data-processor
    spec:
      restartPolicy: OnFailure
      tolerations:
        - key: karpenter.sh/capacity-type
          value: spot
          effect: NoSchedule

      initContainers:
        # Restore checkpoint if it exists
        - name: restore-checkpoint
          image: registry.example.com/data-processor:v1.0.0
          command:
            - /bin/sh
            - -c
            - |
              CHECKPOINT_PATH="s3://my-checkpoints/data-processor/${JOB_NAME}/checkpoint.json"
              
              if aws s3 ls "$CHECKPOINT_PATH" 2>/dev/null; then
                echo "Found checkpoint -- resuming from last position"
                aws s3 cp "$CHECKPOINT_PATH" /data/checkpoint.json
                echo "RESTORE=true" >> /data/state.env
              else
                echo "No checkpoint found -- starting from beginning"
                echo "RESTORE=false" >> /data/state.env
                echo '{"offset": 0, "processed": 0, "failed": 0}' > /data/checkpoint.json
              fi
          volumeMounts:
            - name: workdir
              mountPath: /data
          env:
            - name: JOB_NAME
              valueFrom:
                fieldRef:
                  fieldPath: metadata.name

      containers:
        - name: processor
          image: registry.example.com/data-processor:v1.0.0
          command:
            - /bin/sh
            - -c
            - |
              # Load state
              source /data/state.env
              
              # Trap SIGTERM for checkpoint
              trap 'save_checkpoint; exit 0' SIGTERM SIGINT
              
              save_checkpoint() {
                echo "Interrupted -- saving checkpoint"
                # Write current progress to checkpoint file
                /app/checkpoint --output /data/checkpoint.json
                aws s3 cp /data/checkpoint.json \
                  "s3://my-checkpoints/data-processor/${JOB_NAME}/checkpoint.json"
                echo "Checkpoint saved"
              }
              
              # Periodic checkpoint (every 5 minutes)
              while true; do
                save_checkpoint
                sleep 300
              done &
              CHECKPOINT_PID=$!
              
              # Run the main processing
              if [ "$RESTORE" = "true" ]; then
                /app/process --resume --checkpoint /data/checkpoint.json
              else
                /app/process --input s3://my-data/input/
              fi
              
              EXIT_CODE=$?
              
              # Final checkpoint
              save_checkpoint
              
              exit $EXIT_CODE
          resources:
            requests:
              cpu: 2
              memory: 4Gi
            limits:
              cpu: 4
              memory: 8Gi
          volumeMounts:
            - name: workdir
              mountPath: /data

      volumes:
        - name: workdir
          emptyDir:
            sizeLimit: 1Gi
```

### Part 5: Cost Comparison Table

```
Cost Comparison: On-Demand vs Spot Strategy
=============================================

Component          On-Demand      Spot          Savings
                    ($/month)      ($/month)     ($/month)
-----------------------------------------------------------+
Compute Nodes

  10x m5.2xlarge    $ 5,548       $ 1,664       $ 3,884
  8x  c5.4xlarge    $ 6,998       $ 2,099       $ 4,899
  6x  r5.2xlarge    $ 5,582       $ 1,675       $ 3,907
  4x  m5.xlarge     $ 1,387       $   416       $   971
                    ---------     ---------     ---------
  Subtotal:         $19,515       $ 5,854       $13,661

On-Demand Fallback
  2x  m5.2xlarge    $ 1,110       $   --        $   --
  (reserved for                                     
   critical svc)                                    
                    ---------     ---------     ---------
  Subtotal:         $ 1,110       $ 1,110       $    --

Additional Costs
  Checkpoint S3     $    --       $    45       $   -45
  (storage for                                      
   state)                                           
  Monitoring        $    --       $    20       $   -20
  (interruption                                     
   handler)                                         
                    ---------     ---------     ---------
  Subtotal:         $    --       $    65       $   -65

-----------------------------------------------------------+
TOTAL:              $20,625       $ 7,029       $13,596/mo
ANNUAL:             $247,500      $ 84,348      $163,152/yr
SAVINGS:                                      65.9% reduction
-----------------------------------------------------------+

Risk-Adjusted Analysis
=======================

Scenario               Spot Savings   Downtime Cost   Net Savings
-----------------------------------------------------------------+
Best case (no int.)    $163,152       $        0      $163,152
Typical (2 int./mo)    $163,152       $     2,400     $160,752
Worst case (8 int./mo) $163,152       $    12,000     $151,152
-----------------------------------------------------------------+

* Downtime cost assumes 5 min recovery per interruption
  at effective rate of $150/hr lost revenue per service

Break-Even: Spot remains cheaper even with 40+ interruptions/month.
```

## Why This Works

1. **Multiple instance families.** Karpenter selects from 9 instance families,
   increasing the pool of available spot capacity. AWS recommends diversifying
   to reduce interruption rates.

2. **Weight-based priority.** Setting `weight: 10` for spot and `weight: 50`
   for on-demand means Karpenter tries spot first. If spot capacity is
   unavailable, it falls back to on-demand automatically.

3. **Graceful shutdown within 2 minutes.** The preStop hook runs within the
   termination grace period, checkpointing state and draining connections
   before the instance is reclaimed.

4. **Checkpoint/resume for batch jobs.** Long-running jobs save progress
   every 5 minutes and on SIGTERM. If interrupted, the init container
   restores from the last checkpoint and resumes processing.

5. **Pod anti-affinity and topology spread.** Distributing pods across
   availability zones prevents correlated failures when an entire AZ loses
   spot capacity.

## Common Mistakes to Avoid

- **Running stateful workloads on spot without checkpoints.** Databases,
   message brokers, and other stateful services should not run on spot
   unless they have robust replication and failover.

- **Using a single instance type.** If you only request m5.xlarge spot
   instances, a capacity shortage for that specific type will evict all
   your nodes simultaneously.

- **Setting terminationGracePeriodSeconds too low.** If the grace period
   is shorter than the time needed to drain connections and checkpoint,
   the kubelet will SIGKILL the container, losing in-flight work.

- **Not testing interruption handling.** Use `aws ec2
   send-spot-instance-interruptions` in a test environment to verify
   your shutdown logic actually works before relying on it in production.

- **Ignoring spot pricing history.** Some instance types have much higher
   interruption rates than others. Check the AWS Spot Instance Advisor
   before selecting your instance pool.
