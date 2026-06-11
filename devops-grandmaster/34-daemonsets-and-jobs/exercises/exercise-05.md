# Exercise 05: Design Batch Processing with Jobs and Monitoring with DaemonSets

## Objective

Design and implement a complete batch processing system that:
1. Uses Jobs to process large datasets in parallel
2. Uses a DaemonSet to monitor job progress and collect metrics
3. Implements job coordination and result aggregation
4. Provides real-time visibility into processing status

## Background

This exercise combines Jobs and DaemonSets to build a production-ready batch processing pipeline. You'll learn how to:
- Split large tasks into parallel workloads
- Monitor job execution across nodes
- Aggregate results from multiple workers
- Handle failures gracefully

## Instructions

### Step 1: Create the Namespace

```bash
kubectl create namespace batch-processing
```

### Step 2: Create a Shared Storage Solution

Create a ConfigMap and PVC for shared data:

```yaml
# shared-storage.yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: shared-data
  namespace: batch-processing
spec:
  accessModes:
    - ReadWriteMany  # Important for multi-node access
  resources:
    requests:
      storage: 10Gi
```

### Step 3: Create the Batch Processing Script

Create a ConfigMap with a worker script that:
1. Reads a chunk of data from shared storage
2. Processes each item in the chunk
3. Writes results back to shared storage
4. Reports progress to a Redis metrics endpoint
5. Handles errors gracefully

```yaml
# batch-worker-configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: batch-worker-script
  namespace: batch-processing
data:
  worker.py: |
    #!/usr/bin/env python3
    import os
    import sys
    import json
    import time
    import redis
    from pathlib import Path

    # Configuration
    CHUNK_ID = os.getenv('CHUNK_ID', '0')
    REDIS_HOST = os.getenv('REDIS_HOST', 'redis')
    REDIS_PORT = int(os.getenv('REDIS_PORT', '6379'))
    DATA_DIR = os.getenv('DATA_DIR', '/data')
    PROCESSING_TIME = float(os.getenv('PROCESSING_TIME', '0.5'))

    # Connect to Redis for metrics
    r = redis.Redis(host=REDIS_HOST, port=REDIS_PORT, decode_responses=True)

    def process_item(item):
        """Process a single item"""
        time.sleep(PROCESSING_TIME)

        # Simulate some computation
        result = {
            'id': item['id'],
            'input': item['data'],
            'output': item['data'].upper(),
            'processed_at': time.time(),
            'chunk_id': CHUNK_ID
        }
        return result

    def process_chunk():
        """Process all items in assigned chunk"""
        chunk_file = Path(DATA_DIR) / f'chunk_{CHUNK_ID}.json'
        output_file = Path(DATA_DIR) / f'result_{CHUNK_ID}.json'

        # Check if chunk exists
        if not chunk_file.exists():
            print(f"Chunk file not found: {chunk_file}")
            r.hset(f'chunk:{CHUNK_ID}', 'status', 'error')
            r.hset(f'chunk:{CHUNK_ID}', 'error', 'Chunk file not found')
            return

        # Load chunk data
        with open(chunk_file, 'r') as f:
            items = json.load(f)

        print(f"Processing chunk {CHUNK_ID} with {len(items)} items")
        r.hset(f'chunk:{CHUNK_ID}', 'status', 'processing')
        r.hset(f'chunk:{CHUNK_ID}', 'total_items', len(items))
        r.hset(f'chunk:{CHUNK_ID}', 'processed_items', 0)

        results = []
        errors = []

        for i, item in enumerate(items):
            try:
                result = process_item(item)
                results.append(result)

                # Update progress
                r.hset(f'chunk:{CHUNK_ID}', 'processed_items', i + 1)
                r.hincrby('metrics:total_processed', 1)

                if (i + 1) % 10 == 0:
                    print(f"Chunk {CHUNK_ID}: Processed {i + 1}/{len(items)} items")

            except Exception as e:
                print(f"Error processing item {item['id']}: {e}")
                errors.append({
                    'id': item['id'],
                    'error': str(e)
                })
                r.hincrby('metrics:total_errors', 1)

        # Save results
        with open(output_file, 'w') as f:
            json.dump({
                'chunk_id': CHUNK_ID,
                'results': results,
                'errors': errors,
                'completed_at': time.time()
            }, f, indent=2)

        # Update final status
        r.hset(f'chunk:{CHUNK_ID}', 'status', 'completed')
        r.hset(f'chunk:{CHUNK_ID}', 'result_count', len(results))
        r.hset(f'chunk:{CHUNK_ID}', 'error_count', len(errors))
        r.hincrby('metrics:total_chunks_completed', 1)

        print(f"Chunk {CHUNK_ID} completed: {len(results)} successful, {len(errors)} errors")

    if __name__ == '__main__':
        process_chunk()
```

### Step 4: Create a Data Generator Job

Create a Job that generates test data and splits it into chunks:

```yaml
# data-generator-configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: data-generator-script
  namespace: batch-processing
data:
  generator.py: |
    #!/usr/bin/env python3
    import json
    import os
    import uuid
    from pathlib import Path

    NUM_ITEMS = int(os.getenv('NUM_ITEMS', '100'))
    CHUNK_SIZE = int(os.getenv('CHUNK_SIZE', '20'))
    DATA_DIR = os.getenv('DATA_DIR', '/data')

    # Create data directory
    Path(DATA_DIR).mkdir(parents=True, exist_ok=True)

    # Generate items
    items = []
    for i in range(NUM_ITEMS):
        items.append({
            'id': str(uuid.uuid4())[:8],
            'data': f'Item {i}',
            'index': i
        })

    # Split into chunks
    chunks = []
    for i in range(0, len(items), CHUNK_SIZE):
        chunk = items[i:i + CHUNK_SIZE]
        chunk_id = len(chunks)
        chunk_file = Path(DATA_DIR) / f'chunk_{chunk_id}.json'

        with open(chunk_file, 'w') as f:
            json.dump(chunk, f, indent=2)

        chunks.append({
            'chunk_id': chunk_id,
            'file': str(chunk_file),
            'item_count': len(chunk)
        })

        print(f"Created chunk {chunk_id} with {len(chunk)} items")

    # Save manifest
    manifest = {
        'total_items': len(items),
        'chunk_size': CHUNK_SIZE,
        'num_chunks': len(chunks),
        'chunks': chunks
    }

    with open(Path(DATA_DIR) / 'manifest.json', 'w') as f:
        json.dump(manifest, f, indent=2)

    print(f"\nGenerated {len(items)} items in {len(chunks)} chunks")
    print(f"Manifest saved to {DATA_DIR}/manifest.json")
```

### Step 5: Create the Aggregator Job

Create a Job that aggregates results from all chunks:

```yaml
# aggregator-configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: aggregator-script
  namespace: batch-processing
data:
  aggregator.py: |
    #!/usr/bin/env python3
    import json
    import os
    import time
    from pathlib import Path

    DATA_DIR = os.getenv('DATA_DIR', '/data')

    def aggregate_results():
        """Aggregate results from all chunks"""
        data_path = Path(DATA_DIR)
        manifest_file = data_path / 'manifest.json'

        if not manifest_file.exists():
            print("Manifest file not found!")
            return

        with open(manifest_file, 'r') as f:
            manifest = json.load(f)

        num_chunks = manifest['num_chunks']
        print(f"Aggregating results from {num_chunks} chunks")

        all_results = []
        all_errors = []
        total_processed = 0

        for chunk_id in range(num_chunks):
            result_file = data_path / f'result_{chunk_id}.json'

            if not result_file.exists():
                print(f"Warning: Result file for chunk {chunk_id} not found")
                continue

            with open(result_file, 'r') as f:
                chunk_result = json.load(f)

            all_results.extend(chunk_result['results'])
            all_errors.extend(chunk_result['errors'])
            total_processed += len(chunk_result['results']) + len(chunk_result['errors'])

            print(f"Chunk {chunk_id}: {len(chunk_result['results'])} results, "
                  f"{len(chunk_result['errors'])} errors")

        # Create final report
        report = {
            'summary': {
                'total_items': manifest['total_items'],
                'total_processed': total_processed,
                'successful': len(all_results),
                'failed': len(all_errors),
                'success_rate': len(all_results) / total_processed * 100 if total_processed > 0 else 0
            },
            'results': all_results,
            'errors': all_errors,
            'completed_at': time.time()
        }

        # Save aggregated results
        output_file = data_path / 'aggregated_results.json'
        with open(output_file, 'w') as f:
            json.dump(report, f, indent=2)

        print(f"\nAggregation complete!")
        print(f"Total processed: {total_processed}")
        print(f"Successful: {len(all_results)}")
        print(f"Failed: {len(all_errors)}")
        print(f"Success rate: {report['summary']['success_rate']:.1f}%")
        print(f"\nResults saved to {output_file}")

    if __name__ == '__main__':
        aggregate_results()
```

### Step 6: Create a Monitoring DaemonSet

Create a DaemonSet that:
1. Runs on every node
2. Monitors job execution
3. Collects metrics from Redis
4. Provides a dashboard endpoint

```yaml
# monitor-configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: monitor-script
  namespace: batch-processing
data:
  monitor.py: |
    #!/usr/bin/env python3
    import redis
    import time
    import json
    from http.server import HTTPServer, BaseHTTPRequestHandler
    import threading

    REDIS_HOST = 'redis'
    REDIS_PORT = 6379

    r = redis.Redis(host=REDIS_HOST, port=REDIS_PORT, decode_responses=True)

    class MonitorHandler(BaseHTTPRequestHandler):
        def do_GET(self):
            if self.path == '/health':
                self.send_response(200)
                self.send_header('Content-type', 'application/json')
                self.end_headers()
                self.wfile.write(json.dumps({'status': 'healthy'}).encode())

            elif self.path == '/metrics':
                metrics = get_metrics()
                self.send_response(200)
                self.send_header('Content-type', 'application/json')
                self.end_headers()
                self.wfile.write(json.dumps(metrics, indent=2).encode())

            elif self.path == '/status':
                status = get_status()
                self.send_response(200)
                self.send_header('Content-type', 'application/json')
                self.end_headers()
                self.wfile.write(json.dumps(status, indent=2).encode())

            else:
                self.send_response(404)
                self.end_headers()

        def log_message(self, format, *args):
            pass  # Suppress default logging

    def get_metrics():
        """Get current metrics from Redis"""
        return {
            'total_processed': r.get('metrics:total_processed') or 0,
            'total_errors': r.get('metrics:total_errors') or 0,
            'chunks_completed': r.get('metrics:total_chunks_completed') or 0,
            'timestamp': time.time()
        }

    def get_status():
        """Get status of all chunks"""
        # Get all chunk keys
        chunk_keys = r.keys('chunk:*')
        chunks = {}

        for key in chunk_keys:
            chunk_id = key.split(':')[1]
            chunks[chunk_id] = r.hgetall(key)

        return {
            'chunks': chunks,
            'metrics': get_metrics(),
            'timestamp': time.time()
        }

    def run_server():
        """Run HTTP server for metrics"""
        server = HTTPServer(('0.0.0.0', 8080), MonitorHandler)
        print("Monitor server started on port 8080")
        server.serve_forever()

    def print_status():
        """Print status to stdout periodically"""
        while True:
            status = get_status()
            print(f"\n{'='*60}")
            print(f"Batch Processing Status - {time.strftime('%H:%M:%S')}")
            print(f"{'='*60}")
            print(f"Metrics:")
            print(f"  Total Processed: {status['metrics']['total_processed']}")
            print(f"  Total Errors: {status['metrics']['total_errors']}")
            print(f"  Chunks Completed: {status['metrics']['chunks_completed']}")
            print(f"\nChunks:")
            for chunk_id, chunk_status in sorted(status['chunks'].items()):
                print(f"  Chunk {chunk_id}: {chunk_status.get('status', 'unknown')} "
                      f"({chunk_status.get('processed_items', 0)}/{chunk_status.get('total_items', '?')})")
            print(f"{'='*60}")
            time.sleep(5)

    if __name__ == '__main__':
        # Start HTTP server in background
        server_thread = threading.Thread(target=run_server, daemon=True)
        server_thread.start()

        # Print status to stdout
        print_status()
```

### Step 7: Create the Job Orchestrator

Create a script that orchestrates the entire workflow:

```yaml
# orchestrator-configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: orchestrator-script
  namespace: batch-processing
data:
  orchestrator.py: |
    #!/usr/bin/env python3
    import json
    import time
    import redis
    from pathlib import Path

    REDIS_HOST = 'redis'
    REDIS_PORT = 6379
    DATA_DIR = '/data'

    r = redis.Redis(host=REDIS_HOST, port=REDIS_PORT, decode_responses=True)

    def wait_for_chunks(num_chunks, timeout=300):
        """Wait for all chunks to complete"""
        start_time = time.time()

        while time.time() - start_time < timeout:
            completed = 0

            for chunk_id in range(num_chunks):
                status = r.hget(f'chunk:{chunk_id}', 'status')
                if status == 'completed':
                    completed += 1

            if completed == num_chunks:
                return True

            print(f"Waiting for chunks: {completed}/{num_chunks} completed")
            time.sleep(5)

        return False

    def orchestrate():
        """Main orchestration logic"""
        print("Starting batch processing orchestration")

        # Wait for manifest
        manifest_file = Path(DATA_DIR) / 'manifest.json'
        while not manifest_file.exists():
            print("Waiting for data manifest...")
            time.sleep(2)

        with open(manifest_file, 'r') as f:
            manifest = json.load(f)

        num_chunks = manifest['num_chunks']
        print(f"Manifest loaded: {manifest['total_items']} items in {num_chunks} chunks")

        # Reset metrics
        r.set('metrics:total_processed', 0)
        r.set('metrics:total_errors', 0)
        r.set('metrics:total_chunks_completed', 0)

        # Wait for all chunks to complete
        print("Waiting for all chunks to complete...")
        if wait_for_chunks(num_chunks):
            print("All chunks completed!")
        else:
            print("Timeout waiting for chunks!")
            return

        # Run aggregation
        print("\nStarting aggregation...")
        import subprocess
        subprocess.run(['python3', '/scripts/aggregator.py'])

        print("\nBatch processing complete!")

    if __name__ == '__main__':
        orchestrate()
```

### Step 8: Deploy Redis

Deploy Redis for metrics storage:

```yaml
# redis.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: redis
  namespace: batch-processing
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
---
apiVersion: v1
kind: Service
metadata:
  name: redis
  namespace: batch-processing
spec:
  selector:
    app: redis
  ports:
    - port: 6379
      targetPort: 6379
```

### Step 9: Create the Monitoring DaemonSet

Create a DaemonSet that runs the monitor on every node:

```yaml
# monitor-daemonset.yaml
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: batch-monitor
  namespace: batch-processing
spec:
  selector:
    matchLabels:
      app: batch-monitor
  template:
    metadata:
      labels:
        app: batch-monitor
    spec:
      containers:
        - name: monitor
          image: python:3.11-slim
          command: ["/bin/bash", "-c"]
          args:
            - |
              pip install redis
              python /scripts/monitor.py
          ports:
            - containerPort: 8080
          volumeMounts:
            - name: monitor-script
              mountPath: /scripts
          resources:
            requests:
              memory: "64Mi"
              cpu: "50m"
            limits:
              memory: "128Mi"
              cpu: "100m"
      volumes:
        - name: monitor-script
          configMap:
            name: monitor-script
```

## Deliverables

Create the following files:
1. `shared-storage.yaml` - PVC for shared data
2. `redis.yaml` - Redis deployment and service
3. `batch-worker-configmap.yaml` - Worker processing script
4. `data-generator-configmap.yaml` - Data generator script
5. `aggregator-configmap.yaml` - Result aggregator script
6. `monitor-configmap.yaml` - Monitoring script
7. `orchestrator-configmap.yaml` - Orchestration script
8. `monitor-daemonset.yaml` - Monitoring DaemonSet
9. `data-generator-job.yaml` - Job to generate test data
10. `batch-worker-job.yaml` - Job to process chunks
11. `orchestrator-job.yaml` - Job to orchestrate the workflow

## Success Criteria

- [ ] Data generator creates chunks in shared storage
- [ ] Worker Jobs process all chunks
- [ ] Results are aggregated correctly
- [ ] Monitoring DaemonSet runs on all nodes
- [ ] Metrics are collected and accessible
- [ ] System handles failures gracefully
- [ ] Dashboard shows real-time status

## Hints

<details>
<summary>Hint 1: Parallel Job Execution</summary>

To run multiple worker Jobs in parallel, create a Job for each chunk:

```yaml
# Use a loop or template to create multiple Jobs
for chunk_id in range(num_chunks):
    # Create Job with CHUNK_ID environment variable
```

Or use a single Job with multiple pods:

```yaml
spec:
  completions: 5
  parallelism: 3
```
</details>

<details>
<summary>Hint 2: DaemonSet Monitoring</summary>

The DaemonSet monitor provides:
- Per-node metrics collection
- Health check endpoint (/health)
- Metrics endpoint (/metrics)
- Status dashboard (/status)

Access via: `kubectl port-forward daemonset/batch-monitor 8080:8080 -n batch-processing`
</details>

<details>
<summary>Hint 3: Shared Storage Access</summary>

For ReadWriteMany PVC:
- Use NFS, CephFS, or cloud-based shared storage
- All pods can read and write simultaneously
- Coordinate access with Redis or file locks if needed
</details>

<details>
<summary>Hint 4: Job Coordination</summary>

Use Redis to coordinate Jobs:
- Each worker updates its chunk status
- Orchestrator monitors all chunks
- Aggregator waits for all chunks to complete
</details>

<details>
<summary>Hint 5: Testing the Complete Pipeline</summary>

1. Deploy Redis and shared storage
2. Run data generator Job
3. Run orchestrator Job (it will create worker Jobs)
4. Monitor progress via DaemonSet dashboard
5. Check aggregated results in shared storage
</details>

## Common Issues

1. **PVC not bound**: Ensure you have a StorageClass that supports ReadWriteMany
2. **Redis connection refused**: Verify Redis service is running
3. **Chunks not found**: Check shared storage mount paths
4. **Metrics not updating**: Verify Redis connection in worker scripts
5. **Monitor not accessible**: Check DaemonSet pod status and port forwarding
