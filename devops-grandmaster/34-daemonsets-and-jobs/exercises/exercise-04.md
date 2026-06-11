# Exercise 04: Implement a Job Queue with Retry and Backoff

## Objective

Create a job processing system using Kubernetes Jobs that:
- Processes items from a queue
- Handles failures with exponential backoff
- Tracks job completion and failure metrics
- Implements dead letter queue for failed items

## Background

Real-world batch processing requires robust error handling. This exercise teaches you to build a resilient job queue using Kubernetes primitives.

## Instructions

### Step 1: Create the Namespace

```bash
kubectl create namespace job-queue
```

### Step 2: Create a Redis Queue Backend

Deploy Redis as a queue backend:

```yaml
# redis-deployment.yaml
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

### Step 3: Create a ConfigMap for Job Processing Script

Create a ConfigMap with a worker script that:
1. Connects to Redis
2. Pops items from the main queue
3. Processes each item (simulate with random failures)
4. On success: logs completion
5. On failure: increments retry count, re-queues with exponential backoff, or moves to dead letter queue after max retries

```yaml
# worker-script-configmap.yaml
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
        print("Worker started, waiting for items...")

        while True:
            # Pop item from main queue (blocking)
            _, item_data = r.brpop('job_queue')

            try:
                item = json.loads(item_data)
                item_id = item.get('id', 'unknown')
                retries = item.get('retries', 0)

                print(f"Processing item {item_id} (attempt {retries + 1})")

                try:
                    process_item(item)
                    print(f"Successfully processed item {item_id}")

                    # Record success metric
                    r.incr('metrics:success')
                    r.lpush('completed_items', json.dumps({
                        'id': item_id,
                        'completed_at': time.time()
                    }))

                except Exception as e:
                    print(f"Failed to process item {item_id}: {e}")

                    if retries >= MAX_RETRIES:
                        # Move to dead letter queue
                        item['failed_at'] = time.time()
                        item['error'] = str(e)
                        r.lpush('dead_letter_queue', json.dumps(item))
                        r.incr('metrics:dead_letter')
                        print(f"Item {item_id} moved to dead letter queue")
                    else:
                        # Calculate backoff and re-queue
                        backoff = calculate_backoff(retries)
                        item['retries'] = retries + 1
                        item['next_retry'] = time.time() + backoff

                        # Use sorted set for delayed processing
                        r.zadd('delayed_queue', {json.dumps(item): time.time() + backoff})
                        r.incr('metrics:retries')
                        print(f"Item {item_id} will retry in {backoff} seconds")

            except json.JSONDecodeError:
                print(f"Invalid item data: {item_data}")
                r.incr('metrics:errors')

    def process_delayed_queue():
        """Move items from delayed queue to main queue when ready"""
        while True:
            # Get items that are ready to be processed
            now = time.time()
            items = r.zrangebyscore('delayed_queue', 0, now, start=0, num=10)

            for item_data in items:
                # Move from delayed to main queue
                r.zrem('delayed_queue', item_data)
                r.lpush('job_queue', item_data)
                print(f"Moved item from delayed queue to main queue")

            time.sleep(1)

    if __name__ == '__main__':
        import threading

        # Start delayed queue processor in background
        delayed_thread = threading.Thread(target=process_delayed_queue, daemon=True)
        delayed_thread.start()

        # Start main processing
        process_queue()
```

### Step 4: Create the Worker Job

Create a Job that runs the worker script:

Your Job should:
- Name: `queue-worker`
- Image: `python:3.11-slim`
- Install redis-py and run the worker script
- Connect to Redis service
- Set appropriate resource limits
- Run indefinitely (this is a long-running job)

### Step 5: Create a Job Producer

Create a script that produces test items for the queue:

```yaml
# producer-script-configmap.yaml
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

    print(f"Finished producing {NUM_ITEMS} items")
    print(f"Queue length: {r.llen('job_queue')}")
```

### Step 6: Create Monitoring Script

Create a script to monitor queue metrics:

```yaml
# monitor-script-configmap.yaml
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

    REDIS_HOST = 'redis'
    REDIS_PORT = 6379

    r = redis.Redis(host=REDIS_HOST, port=REDIS_PORT, decode_responses=True)

    print("Queue Monitor Started")
    print("=" * 50)

    while True:
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

        print(f"\nQueue Status:")
        print(f"  Main Queue: {main_queue}")
        print(f"  Delayed Queue: {delayed_queue}")
        print(f"  Dead Letter Queue: {dead_letter}")
        print(f"  Completed Items: {completed}")

        print(f"\nMetrics:")
        print(f"  Successful: {success}")
        print(f"  Retries: {retries}")
        print(f"  Errors: {errors}")
        print(f"  Dead Letter: {dead}")

        print("=" * 50)
        time.sleep(5)
```

### Step 7: Test the System

1. Deploy Redis
2. Run the producer to add items to the queue
3. Start the worker job
4. Monitor the queue processing
5. Verify failed items go to dead letter queue

## Deliverables

Create the following files:
1. `redis-deployment.yaml` - Redis deployment and service
2. `worker-script-configmap.yaml` - Worker processing script
3. `producer-script-configmap.yaml` - Queue producer script
4. `monitor-script-configmap.yaml` - Queue monitor script
5. `worker-job.yaml` - Worker Job manifest

## Success Criteria

- [ ] Redis is deployed and accessible
- [ ] Worker Job processes items from the queue
- [ ] Failed items are retried with exponential backoff
- [ ] Items exceeding max retries go to dead letter queue
- [ ] Metrics are tracked (success, retries, errors, dead letter)
- [ ] Delayed queue processes items after backoff period

## Hints

<details>
<summary>Hint 1: Running Python in Kubernetes</summary>

Use init containers or a command to install dependencies:

```yaml
containers:
  - name: worker
    image: python:3.11-slim
    command: ["/bin/bash", "-c"]
    args:
      - |
        pip install redis
        python /scripts/worker.py
    volumeMounts:
      - name: worker-script
        mountPath: /scripts
```
</details>

<details>
<summary>Hint 2: Job that Runs Indefinitely</summary>

For a worker that should keep running:

```yaml
spec:
  backoffLimit: 0
  template:
    spec:
      restartPolicy: Never
      containers:
        - name: worker
          # ... container spec
```

Note: This is unusual for Jobs. Normally Jobs complete. Consider using a Deployment for long-running workers.
</details>

<details>
<summary>Hint 3: Exponential Backoff Formula</summary>

```
backoff_time = base * 2^retry_count
```

Common values:
- base = 1 second
- retry 0: 1 second
- retry 1: 2 seconds
- retry 2: 4 seconds
- retry 3: 8 seconds (max retries reached)
</details>

<details>
<summary>Hint 4: Redis Sorted Sets for Delayed Processing</summary>

Use sorted sets with timestamps as scores:

```python
# Add to delayed queue
r.zadd('delayed_queue', {item_data: process_at_timestamp})

# Get items ready for processing
now = time.time()
ready_items = r.zrangebyscore('delayed_queue', 0, now)
```
</details>

<details>
<summary>Hint 5: Testing Failure Scenarios</summary>

To test failure handling:
1. Set `PROCESSING_TIME` to a short value
2. The script has a 30% failure rate built in
3. Watch items move between queues
4. Check the dead letter queue after processing
</details>

## Common Issues

1. **Connection refused**: Ensure Redis service is running and accessible
2. **Module not found**: Install redis-py in the container
3. **Queue not processing**: Check worker logs for errors
4. **Items stuck in delayed queue**: Verify the delayed queue processor is running
