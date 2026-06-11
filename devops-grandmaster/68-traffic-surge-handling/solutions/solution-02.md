# Solution 02: Designing a Queue-Based Ingestion Layer

## Part A: Redesign with a Queue

### New Architecture

```
                                    +-----------------+
                                    |  Status Store   |
                                    |   (Redis)       |
                                    +--------+--------+
                                             |
+--------+    +-------------+    +-----------+-----------+    +-------------+    +----------+
| Client |--->| Ingestion  |--->|     Message Queue     |--->|   Worker    |--->| Database |
|  50K   |    |   Layer     |    |       (Redis List)    |    |   Layer     |    | 3K w/s   |
|  RPS   |<---|  20 pods    |    +-----------+-----------+    |  Auto-scaled|    +----------+
+--------+    +-------------+                |               +-------------+
  |                                            |                      |
  |  POST /orders                              |                      |
  |  -> validate                               +--- BLPOP --->        |
  |  -> generate order_id                             |                |
  |  -> PUSH to queue                                 |                |
  |  -> SET order status = "queued"                   |                |
  |  <- {order_id, status: "queued"}                  |                |
  |                                                   |                |
  |  GET /orders/{id}/status                          |                |
  |  <- {status: "queued"|"processing"|"completed"}   |                |
```

### Component Roles

1. **Ingestion Layer**: Lightweight pods that validate requests, generate order IDs, push to Redis queue, and return immediately. Handles 50,000 RPS with minimal resources.

2. **Redis Queue**: Acts as a buffer between ingestion and processing. At 50,000 RPS ingestion and 3,000/sec processing, the queue grows at 47,000 messages/second during the surge. For a 15-minute sale, that is ~42 million messages.

3. **Worker Layer**: Consumes from the queue at 3,000/second (database limit). Auto-scaled by KEDA based on queue depth.

4. **Status Store**: Redis hash storing order status. Clients poll for updates.

### Why This Works

The queue decouples acceptance from fulfillment. The client gets an immediate response (order ID), and the actual processing happens asynchronously. The system never drops requests -- it buffers them. The trade-off is latency: orders take seconds to minutes to complete, not milliseconds.

---

## Part B: Ingestion Service YAML

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: order-ingestion
  namespace: production
spec:
  replicas: 20
  selector:
    matchLabels:
      app: order-ingestion
  template:
    metadata:
      labels:
        app: order-ingestion
    spec:
      containers:
        - name: ingestion
          image: order-ingestion:latest
          ports:
            - containerPort: 8080
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 500m
              memory: 256Mi
          env:
            - name: REDIS_URL
              value: "redis://redis-queue.production.svc:6379"
            - name: MAX_QUEUE_DEPTH
              value: "1000000"
            - name: BACKPRESSURE_THRESHOLD
              value: "800000"
          readinessProbe:
            httpGet:
              path: /health
              port: 8080
            periodSeconds: 5
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            periodSeconds: 10
---
apiVersion: v1
kind: Service
metadata:
  name: order-ingestion
  namespace: production
spec:
  selector:
    app: order-ingestion
  ports:
    - port: 80
      targetPort: 8080
  type: ClusterIP
```

### Why This Works

- **Low resource requests** (100m CPU, 128Mi memory) allow high pod density per node
- **20 replicas** at 2,500 RPS each handles 50,000 RPS
- **Backpressure threshold** at 800K of 1M queue capacity rejects requests before the queue is completely full
- **Health probes** ensure only ready pods receive traffic

---

## Part C: Worker Service YAML

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: order-worker
  namespace: production
spec:
  replicas: 5
  selector:
    matchLabels:
      app: order-worker
  template:
    metadata:
      labels:
        app: order-worker
    spec:
      containers:
        - name: worker
          image: order-worker:latest
          resources:
            requests:
              cpu: 500m
              memory: 512Mi
            limits:
              cpu: 1
              memory: 1Gi
          env:
            - name: REDIS_URL
              value: "redis://redis-queue.production.svc:6379"
            - name: DATABASE_URL
              value: "postgres://user:pass@postgres:5432/orders"
            - name: CONCURRENCY
              value: "10"
            - name: MAX_RETRIES
              value: "3"
            - name: RETRY_DELAY_MS
              value: "5000"
            - name: PROCESSING_RATE_PER_POD
              value: "600"  # 600/sec per pod, 5 pods = 3000/sec total
---
apiVersion: keda.sh/v1alpha1
kind: ScaledObject
metadata:
  name: order-worker-scaler
  namespace: production
spec:
  scaleTargetRef:
    name: order-worker
  pollingInterval: 15
  cooldownPeriod: 300
  minReplicaCount: 2
  maxReplicaCount: 20
  triggers:
    - type: redis
      metadata:
        address: redis-queue.production.svc:6379
        listName: order_queue
        listLength: "1000"  # 1 worker per 1000 queued messages
```

### Why This Works

- **CONCURRENCY=10** limits each pod to 10 concurrent database operations, preventing connection pool exhaustion
- **PROCESSING_RATE_PER_POD=600** caps throughput per pod to stay within database limits
- **KEDA ScaledObject** auto-scales workers based on queue depth: 1,000 queued messages = 1 additional worker
- **minReplicaCount=2** ensures workers are always running (no cold start delay)
- **Retry logic** handles transient database failures without losing messages

---

## Common Mistakes to Avoid

- **Making the ingestion layer do too much.** The ingestion service should only validate, enqueue, and return. Do not make database calls or payment processing in the ingestion path.

- **Not implementing backpressure.** Without backpressure, the queue grows unbounded until Redis runs out of memory, causing data loss.

- **Processing faster than the database can handle.** Workers must rate-limit themselves. If 20 workers each process at 1,000/sec, you send 20,000 writes/sec to a database that handles 3,000/sec.

- **Not handling worker crashes.** Use Redis `BRPOPLPUSH` or consumer groups to ensure messages are not lost if a worker crashes mid-processing.

- **Forgetting status updates.** Clients need to know their order status. Update the status store at each stage: queued -> processing -> completed/failed.

## Key Takeaway

Queue-based architecture transforms an impossible 50,000 RPS problem into a manageable one: 50,000 RPS ingestion (lightweight), 3,000/sec processing (database-limited), and a growing queue that eventually drains. The key is separating acceptance from fulfillment.

## Relevant README Sections
- [Pre-Warming Strategy](../README.md#pre-warming-strategy)
- [Connection Pooling](../README.md#connection-pooling)
- [The Production Way](../README.md#the-production-way)
