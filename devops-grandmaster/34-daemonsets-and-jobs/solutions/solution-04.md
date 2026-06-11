# Solution 04: Implement a Job Queue with Retry and Backoff

## Complete Solution

### redis-deployment.yaml

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: redis
  namespace: job-queue
spec:
  replicas: 1
  selector:
    matchLabels:
      app: redis
  template:
    metadata:
      labels:
        app: redis
    spec:
      containers:
        - name: redis
          image: redis:7-alpine
          ports:
            - containerPort: 6379
          resources:
            requests:
              memory: "64Mi"
              cpu: "50m"
            limits:
              memory: "128Mi"
              cpu: "100m"
          readinessProbe:
            exec:
              command:
                - redis-cli
                - ping
            initialDelaySeconds: 5
            periodSeconds: 5
          livenessProbe:
            exec:
              command:
                - redis-cli
                - ping
            initialDelaySeconds: 15
            periodSeconds: 10
---
apiVersion: v1
kind: Service
metadata:
  name: redis
  namespace: job-queue
spec:
  selector:
    app: redis
  ports:
    - port: 6379
      targetPort: 6379
```

### worker-script-configmap.yaml

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: worker-script
  namespace: job-queue
data:
  worker.py: |
    #!/usr/bin/env python3
    import redis
    import json
    import time
    import random
    import os
    import sys
    from datetime import datetime

    # Configuration
    REDIS_HOST = os.getenv('REDIS_HOST', 'redis')
    REDIS_PORT = int(os.getenv('REDIS_PORT', '6379'))
    MAX_RETRIES = int(os.getenv('MAX_RETRIES', '3'))
    PROCESSING_TIME = float(os.getenv('PROCESSING_TIME', '2'))

    # Connect to Redis
    r = redis.Redis(host=REDIS_HOST, port=REDIS_PORT, decode_responses=True)

    def process_item(item):
        """Simulate processing with random failures"""
        time.sleep(PROCESSING_TIME)

        # 30% chance of failure for testing
        if random.random() < 0.3:
            raise Exception("Processing failed randomly")

        return True

    def calculate_backoff(retry_count):
        """Exponential backoff: 2^retry_count seconds"""
        return 2 ** retry_count

    def process_queue():
        """Main processing loop"""
        print(f"[{datetime.now()}] Worker started, waiting for items...")

        while True:
            try:
                # Pop item from main queue (blocking)
                _, item_data = r.brpop('job_queue')

                item = json.loads(item_data)
                item_id = item.get('id', 'unknown')
                retries = item.get('retries', 0)

                print(f"[{datetime.now()}] Processing item {item_id} (attempt {retries + 1})")

                try:
                    process_item(item)
                    print(f"[{datetime.now()}] Successfully processed item {item_id}")

                    # Record success metric
                    r.incr('metrics:success')
                    r.lpush('completed_items', json.dumps({
                        'id': item_id,
                        'completed_at': time.time(),
                        'attempts': retries + 1
                    }))

                except Exception as e:
                    print(f"[{datetime.now()}] Failed to process item {item_id}: {e}")

                    if retries >= MAX_RETRIES:
                        # Move to dead letter queue
                        item['failed_at'] = time.time()
                        item['error'] = str(e)
                        item['final_attempt'] = retries + 1
                        r.lpush('dead_letter_queue', json.dumps(item))
                        r.incr('metrics:dead_letter')
                        print(f"[{datetime.now()}] Item {item_id} moved to dead letter queue after {retries + 1} attempts")
                    else:
                        # Calculate backoff and re-queue
                        backoff = calculate_backoff(retries)
                        item['retries'] = retries + 1
                        item['next_retry'] = time.time() + backoff
                        item['last_error'] = str(e)

                        # Use sorted set for delayed processing
                        r.zadd('delayed_queue', {json.dumps(item): time.time() + backoff})
                        r.incr('metrics:retries')
                        print(f"[{datetime.now()}] Item {item_id} will retry in {backoff} seconds (attempt {retries + 1}/{MAX_RETRIES})")

            except json.JSONDecodeError as e:
                print(f"[{datetime.now()}] Invalid item data: {item_data}, error: {e}")
                r.incr('metrics:errors')
            except Exception as e:
                print(f"[{datetime.now()}] Unexpected error: {e}")
                r.incr('metrics:errors')
                time.sleep(1)  # Prevent tight loop on persistent errors

    def process_delayed_queue():
        """Move items from delayed queue to main queue when ready"""
        print(f"[{datetime.now()}] Delayed queue processor started")

        while True:
            try:
                # Get items that are ready to be processed
                now = time.time()
                items = r.zrangebyscore('delayed_queue', 0, now, start=0, num=10)

                for item_data in items:
                    # Move from delayed to main queue
                    r.zrem('delayed_queue', item_data)
                    r.lpush('job_queue', item_data)
                    item = json.loads(item_data)
                    print(f"[{datetime.now()}] Moved item {item.get('id')} from delayed queue to main queue")

                time.sleep(1)

            except Exception as e:
                print(f"[{datetime.now()}] Error in delayed queue processor: {e}")
                time.sleep(5)

    if __name__ == '__main__':
        import threading

        # Start delayed queue processor in background
        delayed_thread = threading.Thread(target=process_delayed_queue, daemon=True)
        delayed_thread.start()

        # Start main processing
        process_queue()
```

### producer-script-configmap.yaml

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: producer-script
  namespace: job-queue
data:
  producer.py: |
    #!/usr/bin/env python3
    import redis
    import json
    import time
    import uuid

    REDIS_HOST = 'redis'
    REDIS_PORT = 6379
    NUM_ITEMS = 20

    r = redis.Redis(host=REDIS_HOST, port=REDIS_PORT, decode_responses=True)

    print(f"Producing {NUM_ITEMS} items...")

    for i in range(NUM_ITEMS):
        item = {
            'id': str(uuid.uuid4())[:8],
            'data': f'Item {i}',
            'created_at': time.time(),
            'retries': 0
        }
        r.lpush('job_queue', json.dumps(item))
        print(f"Produced item {item['id']}")

    print(f"\nFinished producing {NUM_ITEMS} items")
    print(f"Queue length: {r.llen('job_queue')}")
```

### monitor-script-configmap.yaml

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: monitor-script
  namespace: job-queue
data:
  monitor.py: |
    #!/usr/bin/env python3
    import redis
    import time
    from datetime import datetime

    REDIS_HOST = 'redis'
    REDIS_PORT = 6379

    r = redis.Redis(host=REDIS_HOST, port=REDIS_PORT, decode_responses=True)

    print("Queue Monitor Started")
    print("=" * 60)

    while True:
        try:
            # Get queue lengths
            main_queue = r.llen('job_queue')
            delayed_queue = r.zcard('delayed_queue')
            dead_letter = r.llen('dead_letter_queue')
            completed = r.llen('completed_items')

            # Get metrics
            success = r.get('metrics:success') or 0
            retries = r.get('metrics:retries') or 0
            errors = r.get('metrics:errors') or 0
            dead = r.get('metrics:dead_letter') or 0

            print(f"\n[{datetime.now().strftime('%H:%M:%S')}] Queue Status:")
            print(f"  Main Queue: {main_queue}")
            print(f"  Delayed Queue: {delayed_queue}")
            print(f"  Dead Letter Queue: {dead_letter}")
            print(f"  Completed Items: {completed}")

            print(f"\nMetrics:")
            print(f"  Successful: {success}")
            print(f"  Retries: {retries}")
            print(f"  Errors: {errors}")
            print(f"  Dead Letter: {dead}")

            # Calculate success rate
            total_processed = int(success) + int(dead)
            if total_processed > 0:
                success_rate = (int(success) / total_processed) * 100
                print(f"\n  Success Rate: {success_rate:.1f}%")

            print("=" * 60)
            time.sleep(5)

        except Exception as e:
            print(f"Error: {e}")
            time.sleep(5)
```

### worker-job.yaml

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: queue-worker
  namespace: job-queue
spec:
  # Run indefinitely (restart on failure)
  backoffLimit: 0
  template:
    metadata:
      labels:
        app: queue-worker
    spec:
      restartPolicy: Never
      containers:
        - name: worker
          image: python:3.11-slim
          command: ["/bin/bash", "-c"]
          args:
            - |
              pip install redis
              python /scripts/worker.py
          env:
            - name: REDIS_HOST
              value: "redis"
            - name: REDIS_PORT
              value: "6379"
            - name: MAX_RETRIES
              value: "3"
            - name: PROCESSING_TIME
              value: "2"
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 500m
              memory: 256Mi
          volumeMounts:
            - name: worker-script
              mountPath: /scripts
      volumes:
        - name: worker-script
          configMap:
            name: worker-script
```

### producer-job.yaml

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: queue-producer
  namespace: job-queue
spec:
  backoffLimit: 3
  ttlSecondsAfterFinished: 3600
  template:
    metadata:
      labels:
        app: queue-producer
    spec:
      restartPolicy: Never
      containers:
        - name: producer
          image: python:3.11-slim
          command: ["/bin/bash", "-c"]
          args:
            - |
              pip install redis
              python /scripts/producer.py
          resources:
            requests:
              cpu: 50m
              memory: 64Mi
            limits:
              cpu: 100m
              memory: 128Mi
          volumeMounts:
            - name: producer-script
              mountPath: /scripts
      volumes:
        - name: producer-script
          configMap:
            name: producer-script
```

### monitor-deployment.yaml

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: queue-monitor
  namespace: job-queue
spec:
  replicas: 1
  selector:
    matchLabels:
      app: queue-monitor
  template:
    metadata:
      labels:
        app: queue-monitor
    spec:
      containers:
        - name: monitor
          image: python:3.11-slim
          command: ["/bin/bash", "-c"]
          args:
            - |
              pip install redis
              python /scripts/monitor.py
          resources:
            requests:
              cpu: 50m
              memory: 64Mi
            limits:
              cpu: 100m
              memory: 128Mi
          volumeMounts:
            - name: monitor-script
              mountPath: /scripts
      volumes:
        - name: monitor-script
          configMap:
            name: monitor-script
```

## Why This Solution Works

### 1. Redis as a Queue Backend

Redis is an excellent choice for job queues because:

- **Atomic operations**: `BRPOP` ensures only one worker processes each item
- **Sorted sets**: Enable delayed processing with timestamps
- **Persistence**: Can be configured for durability
- **Performance**: In-memory storage for fast operations
- **Pub/Sub**: Can be extended for real-time notifications

### 2. Exponential Backoff Strategy

The backoff formula `2^retry_count` provides:

- **Retry 0**: 1 second delay
- **Retry 1**: 2 seconds delay
- **Retry 2**: 4 seconds delay
- **Retry 3**: 8 seconds delay (max retries reached)

This prevents:
- Overwhelming downstream services with rapid retries
- Resource exhaustion from constant retry attempts
- Thundering herd problems

### 3. Dead Letter Queue for Failed Items

Failed items that exceed max retries are moved to a dead letter queue:

- **Preserves failed items** for debugging
- **Prevents infinite retry loops**
- **Enables manual inspection** and reprocessing
- **Tracks failure reasons** for analysis

### 4. Delayed Queue for Backoff

Using Redis sorted sets for delayed processing:

```python
# Add to delayed queue with timestamp as score
r.zadd('delayed_queue', {item_data: process_at_timestamp})

# Get items ready for processing
ready_items = r.zrangebyscore('delayed_queue', 0, now)
```

This ensures:
- Items are processed after their backoff period
- Multiple workers can coordinate safely
- No polling or sleeping in the main loop

### 5. Metrics Tracking

Comprehensive metrics for monitoring:

- `metrics:success`: Successful completions
- `metrics:retries`: Retry attempts
- `metrics:errors`: Processing errors
- `metrics:dead_letter`: Items moved to dead letter queue

These enable:
- Success rate calculation
- Performance monitoring
- Alerting on failures
- Capacity planning

## Verification Steps

### 1. Deploy Redis

```bash
kubectl apply -f redis-deployment.yaml

# Wait for Redis to be ready
kubectl wait --for=condition=ready pod -l app=redis -n job-queue --timeout=60s
```

### 2. Start the Worker

```bash
kubectl apply -f worker-job.yaml

# Check worker logs
kubectl logs -f job/queue-worker -n job-queue
```

### 3. Produce Test Items

```bash
kubectl apply -f producer-job.yaml

# Wait for producer to complete
kubectl wait --for=condition=complete job/queue-producer -n job-queue

# Check producer logs
kubectl logs job/queue-producer -n job-queue
```

### 4. Monitor Processing

```bash
kubectl apply -f monitor-deployment.yaml

# Watch monitor logs
kubectl logs -f deployment/queue-monitor -n job-queue
```

### 5. Check Results

```bash
# Check metrics
kubectl exec -it deployment/redis -n job-queue -- redis-cli \
  GET metrics:success
kubectl exec -it deployment/redis -n job-queue -- redis-cli \
  GET metrics:dead_letter

# Check completed items
kubectl exec -it deployment/redis -n job-queue -- redis-cli \
  LLEN completed_items

# Check dead letter queue
kubectl exec -it deployment/redis -n job-queue -- redis-cli \
  LLEN dead_letter_queue
```

## Common Mistakes to Avoid

### 1. Using BRPOP Without Timeout

**Mistake:** Using `BRPOP` with timeout 0 (block forever).

**Problem:** Worker hangs if queue is empty and never shuts down gracefully.

**Solution:** Use a reasonable timeout or handle signals:
```python
# Option 1: Use timeout
_, item_data = r.brpop('job_queue', timeout=30)
if item_data is None:
    continue  # Timeout, check for shutdown signal

# Option 2: Use BLPOP with signal handling
import signal
running = True
def handler(signum, frame):
    global running
    running = False
signal.signal(signal.SIGTERM, handler)
```

### 2. Not Handling JSON Decode Errors

**Mistake:** Assuming all queue items are valid JSON.

**Problem:** Malformed items crash the worker.

**Solution:** Always wrap JSON parsing in try/except:
```python
try:
    item = json.loads(item_data)
except json.JSONDecodeError:
    r.incr('metrics:errors')
    continue
```

### 3. Race Conditions in Delayed Queue

**Mistake:** Multiple workers moving items from delayed queue simultaneously.

**Problem:** Same item processed multiple times.

**Solution:** Use Redis transactions or Lua scripts:
```python
# Use pipeline for atomic operations
pipe = r.pipeline()
pipe.zrangebyscore('delayed_queue', 0, now)
pipe.zremrangebyscore('delayed_queue', 0, now)
results = pipe.execute()
```

### 4. Not Setting MAX_RETRIES

**Mistake:** Using default or unlimited retries.

**Problem:** Items retry indefinitely, wasting resources.

**Solution:** Always set a reasonable max retries:
```python
MAX_RETRIES = int(os.getenv('MAX_RETRIES', '3'))
```

### 5. Ignoring Backoff Calculation

**Mistake:** Using fixed delay instead of exponential backoff.

**Problem:** Overwhelms downstream services during failures.

**Solution:** Use exponential backoff:
```python
def calculate_backoff(retry_count):
    return 2 ** retry_count  # 1, 2, 4, 8, 16, ...
```

### 6. Not Tracking Metrics

**Mistake:** No visibility into queue processing.

**Problem:** Can't detect failures or performance issues.

**Solution:** Track comprehensive metrics:
```python
r.incr('metrics:success')
r.incr('metrics:retries')
r.incr('metrics:errors')
r.incr('metrics:dead_letter')
```

## Production Considerations

### 1. Redis Persistence

Configure Redis for durability:

```yaml
containers:
  - name: redis
    image: redis:7-alpine
    command: ["redis-server", "--appendonly", "yes"]
    volumeMounts:
      - name: redis-data
        mountPath: /data
volumes:
  - name: redis-data
    persistentVolumeClaim:
      claimName: redis-data
```

### 2. Redis Sentinel for High Availability

Deploy Redis Sentinel for automatic failover:

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: redis
spec:
  replicas: 3
  serviceName: redis
  selector:
    matchLabels:
      app: redis
  template:
    spec:
      containers:
        - name: redis
          image: redis:7-alpine
          command: ["redis-server", "--sentinel"]
```

### 3. Queue Monitoring with Prometheus

Export metrics for Prometheus:

```python
from prometheus_client import Counter, Gauge, start_http_server

# Define metrics
items_processed = Counter('items_processed_total', 'Total items processed')
queue_length = Gauge('queue_length', 'Current queue length')

# Update metrics
items_processed.inc()
queue_length.set(r.llen('job_queue'))

# Start metrics server
start_http_server(8000)
```

### 4. Dead Letter Queue Processing

Implement dead letter queue reprocessing:

```python
def reprocess_dead_letter():
    """Manually reprocess items from dead letter queue"""
    while True:
        item_data = r.rpop('dead_letter_queue')
        if item_data is None:
            break

        item = json.loads(item_data)
        item['retries'] = 0  # Reset retries
        r.lpush('job_queue', json.dumps(item))
        print(f"Requeued item {item['id']} from dead letter queue")
```

### 5. Circuit Breaker Pattern

Implement circuit breaker for downstream services:

```python
class CircuitBreaker:
    def __init__(self, failure_threshold=5, reset_timeout=60):
        self.failure_count = 0
        self.failure_threshold = failure_threshold
        self.reset_timeout = reset_timeout
        self.last_failure_time = None
        self.state = 'closed'  # closed, open, half-open

    def call(self, func, *args, **kwargs):
        if self.state == 'open':
            if time.time() - self.last_failure_time > self.reset_timeout:
                self.state = 'half-open'
            else:
                raise Exception("Circuit breaker is open")

        try:
            result = func(*args, **kwargs)
            if self.state == 'half-open':
                self.state = 'closed'
                self.failure_count = 0
            return result
        except Exception as e:
            self.failure_count += 1
            self.last_failure_time = time.time()
            if self.failure_count >= self.failure_threshold:
                self.state = 'open'
            raise
```

### 6. Graceful Shutdown

Handle SIGTERM for graceful shutdown:

```python
import signal
import sys

def shutdown_handler(signum, frame):
    print("Received shutdown signal, finishing current item...")
    global running
    running = False

signal.signal(signal.SIGTERM, shutdown_handler)
signal.signal(signal.SIGINT, shutdown_handler)

running = True
while running:
    _, item_data = r.brpop('job_queue', timeout=1)
    if item_data:
        process_item(json.loads(item_data))

print("Worker shutdown complete")
sys.exit(0)
```
