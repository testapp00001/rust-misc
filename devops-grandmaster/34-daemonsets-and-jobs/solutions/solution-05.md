# Solution 05: Design Batch Processing with Jobs and Monitoring with DaemonSets

## Complete Solution

### shared-storage.yaml

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: shared-data
  namespace: batch-processing
spec:
  accessModes:
    - ReadWriteMany
  resources:
    requests:
      storage: 10Gi
```

### redis.yaml

```yaml
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
  namespace: batch-processing
spec:
  selector:
    app: redis
  ports:
    - port: 6379
      targetPort: 6379
```

### batch-worker-configmap.yaml

```yaml
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
            'chunk_id': CHUNK_ID,
            'node': os.environ.get('NODE_NAME', 'unknown')
        }
        return result

    def process_chunk():
        """Process all items in assigned chunk"""
        chunk_file = Path(DATA_DIR) / f'chunk_{CHUNK_ID}.json'
        output_file = Path(DATA_DIR) / f'result_{CHUNK_ID}.json'

        # Check if chunk exists
        if not chunk_file.exists():
            print(f"Chunk file not found: {chunk_file}")
            r.hset(f'chunk:{CHUNK_ID}', mapping={'status': 'error', 'error': 'Chunk file not found'})
            return

        # Load chunk data
        with open(chunk_file, 'r') as f:
            items = json.load(f)

        print(f"Processing chunk {CHUNK_ID} with {len(items)} items")
        r.hset(f'chunk:{CHUNK_ID}', mapping={
            'status': 'processing',
            'total_items': len(items),
            'processed_items': 0,
            'started_at': time.time()
        })

        results = []
        errors = []

        for i, item in enumerate(items):
            try:
                result = process_item(item)
                results.append(result)

                # Update progress
                r.hset(f'chunk:{CHUNK_ID}', 'processed_items', i + 1)
                r.incrby('metrics:total_processed', 1)

                if (i + 1) % 10 == 0:
                    print(f"Chunk {CHUNK_ID}: Processed {i + 1}/{len(items)} items")

            except Exception as e:
                print(f"Error processing item {item['id']}: {e}")
                errors.append({
                    'id': item['id'],
                    'error': str(e)
                })
                r.incrby('metrics:total_errors', 1)

        # Save results
        with open(output_file, 'w') as f:
            json.dump({
                'chunk_id': CHUNK_ID,
                'results': results,
                'errors': errors,
                'completed_at': time.time()
            }, f, indent=2)

        # Update final status
        r.hset(f'chunk:{CHUNK_ID}', mapping={
            'status': 'completed',
            'result_count': len(results),
            'error_count': len(errors),
            'completed_at': time.time()
        })
        r.incrby('metrics:total_chunks_completed', 1)

        print(f"Chunk {CHUNK_ID} completed: {len(results)} successful, {len(errors)} errors")

    if __name__ == '__main__':
        process_chunk()
```

### data-generator-configmap.yaml

```yaml
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

### aggregator-configmap.yaml

```yaml
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
        chunks_found = 0

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
            chunks_found += 1

            print(f"Chunk {chunk_id}: {len(chunk_result['results'])} results, "
                  f"{len(chunk_result['errors'])} errors")

        # Create final report
        report = {
            'summary': {
                'total_items': manifest['total_items'],
                'total_processed': total_processed,
                'successful': len(all_results),
                'failed': len(all_errors),
                'chunks_processed': chunks_found,
                'success_rate': round(len(all_results) / total_processed * 100, 2) if total_processed > 0 else 0
            },
            'results': all_results,
            'errors': all_errors,
            'completed_at': time.time()
        }

        # Save aggregated results
        output_file = data_path / 'aggregated_results.json'
        with open(output_file, 'w') as f:
            json.dump(report, f, indent=2)

        print(f"\n{'='*50}")
        print(f"Aggregation complete!")
        print(f"{'='*50}")
        print(f"Total items: {manifest['total_items']}")
        print(f"Total processed: {total_processed}")
        print(f"Successful: {len(all_results)}")
        print(f"Failed: {len(all_errors)}")
        print(f"Success rate: {report['summary']['success_rate']}%")
        print(f"\nResults saved to {output_file}")

    if __name__ == '__main__':
        aggregate_results()
```

### monitor-configmap.yaml

```yaml
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
    import os
    from http.server import HTTPServer, BaseHTTPRequestHandler
    import threading
    from datetime import datetime

    REDIS_HOST = os.getenv('REDIS_HOST', 'redis')
    REDIS_PORT = int(os.getenv('REDIS_PORT', '6379'))
    NODE_NAME = os.getenv('NODE_NAME', 'unknown')

    r = redis.Redis(host=REDIS_HOST, port=REDIS_PORT, decode_responses=True)

    class MonitorHandler(BaseHTTPRequestHandler):
        def do_GET(self):
            if self.path == '/health':
                self.send_response(200)
                self.send_header('Content-type', 'application/json')
                self.end_headers()
                self.wfile.write(json.dumps({'status': 'healthy', 'node': NODE_NAME}).encode())

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
            'node': NODE_NAME,
            'total_processed': r.get('metrics:total_processed') or 0,
            'total_errors': r.get('metrics:total_errors') or 0,
            'chunks_completed': r.get('metrics:total_chunks_completed') or 0,
            'timestamp': time.time()
        }

    def get_status():
        """Get status of all chunks"""
        chunk_keys = r.keys('chunk:*')
        chunks = {}

        for key in chunk_keys:
            chunk_id = key.split(':')[1]
            chunks[chunk_id] = r.hgetall(key)

        return {
            'node': NODE_NAME,
            'chunks': chunks,
            'metrics': get_metrics(),
            'timestamp': time.time()
        }

    def run_server():
        """Run HTTP server for metrics"""
        server = HTTPServer(('0.0.0.0', 8080), MonitorHandler)
        print(f"Monitor server started on node {NODE_NAME}, port 8080")
        server.serve_forever()

    def print_status():
        """Print status to stdout periodically"""
        while True:
            try:
                status = get_status()
                print(f"\n{'='*60}")
                print(f"Batch Processing Status - {datetime.now().strftime('%H:%M:%S')} (Node: {NODE_NAME})")
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
            except Exception as e:
                print(f"Error: {e}")

            time.sleep(10)

    if __name__ == '__main__':
        # Start HTTP server in background
        server_thread = threading.Thread(target=run_server, daemon=True)
        server_thread.start()

        # Print status to stdout
        print_status()
```

### orchestrator-configmap.yaml

```yaml
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
    import os
    import redis
    from pathlib import Path

    REDIS_HOST = os.getenv('REDIS_HOST', 'redis')
    REDIS_PORT = int(os.getenv('REDIS_PORT', '6379'))
    DATA_DIR = os.getenv('DATA_DIR', '/data')
    WAIT_TIMEOUT = int(os.getenv('WAIT_TIMEOUT', '300'))

    r = redis.Redis(host=REDIS_HOST, port=REDIS_PORT, decode_responses=True)

    def wait_for_chunks(num_chunks, timeout=WAIT_TIMEOUT):
        """Wait for all chunks to complete"""
        start_time = time.time()

        while time.time() - start_time < timeout:
            completed = 0
            errors = 0

            for chunk_id in range(num_chunks):
                status = r.hget(f'chunk:{chunk_id}', 'status')
                if status == 'completed':
                    completed += 1
                elif status == 'error':
                    errors += 1

            if completed + errors == num_chunks:
                if errors > 0:
                    print(f"Warning: {errors} chunks had errors")
                return completed == num_chunks

            print(f"Waiting for chunks: {completed} completed, {errors} errors, "
                  f"{num_chunks - completed - errors} pending")
            time.sleep(5)

        return False

    def orchestrate():
        """Main orchestration logic"""
        print("Starting batch processing orchestration")

        # Wait for manifest
        manifest_file = Path(DATA_DIR) / 'manifest.json'
        print("Waiting for data manifest...")
        while not manifest_file.exists():
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
            print("All chunks completed successfully!")
        else:
            print("Timeout or errors waiting for chunks!")

        # Run aggregation
        print("\nStarting aggregation...")
        import subprocess
        result = subprocess.run(['python3', '/scripts/aggregator.py'], capture_output=True, text=True)
        print(result.stdout)
        if result.stderr:
            print(f"Errors: {result.stderr}")

        print("\nBatch processing complete!")

    if __name__ == '__main__':
        orchestrate()
```

### data-generator-job.yaml

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: data-generator
  namespace: batch-processing
spec:
  backoffLimit: 3
  ttlSecondsAfterFinished: 3600
  template:
    metadata:
      labels:
        app: data-generator
    spec:
      restartPolicy: Never
      containers:
        - name: generator
          image: python:3.11-slim
          command: ["/bin/bash", "-c"]
          args:
            - |
              python /scripts/generator.py
          env:
            - name: NUM_ITEMS
              value: "100"
            - name: CHUNK_SIZE
              value: "20"
            - name: DATA_DIR
              value: "/data"
          resources:
            requests:
              cpu: 50m
              memory: 64Mi
            limits:
              cpu: 100m
              memory: 128Mi
          volumeMounts:
            - name: generator-script
              mountPath: /scripts
            - name: shared-data
              mountPath: /data
      volumes:
        - name: generator-script
          configMap:
            name: data-generator-script
        - name: shared-data
          persistentVolumeClaim:
            claimName: shared-data
```

### batch-worker-job.yaml

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: batch-worker
  namespace: batch-processing
spec:
  completions: 5
  parallelism: 3
  backoffLimit: 3
  completionMode: Indexed
  template:
    metadata:
      labels:
        app: batch-worker
    spec:
      restartPolicy: OnFailure
      containers:
        - name: worker
          image: python:3.11-slim
          command: ["/bin/bash", "-c"]
          args:
            - |
              pip install redis
              python /scripts/worker.py
          env:
            - name: CHUNK_ID
              valueFrom:
                fieldRef:
                  fieldPath: metadata.annotations['batch.kubernetes.io/job-completion-index']
            - name: REDIS_HOST
              value: "redis"
            - name: REDIS_PORT
              value: "6379"
            - name: DATA_DIR
              value: "/data"
            - name: NODE_NAME
              valueFrom:
                fieldRef:
                  fieldPath: spec.nodeName
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
            - name: shared-data
              mountPath: /data
      volumes:
        - name: worker-script
          configMap:
            name: batch-worker-script
        - name: shared-data
          persistentVolumeClaim:
            claimName: shared-data
```

### orchestrator-job.yaml

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: orchestrator
  namespace: batch-processing
spec:
  backoffLimit: 1
  ttlSecondsAfterFinished: 3600
  template:
    metadata:
      labels:
        app: orchestrator
    spec:
      restartPolicy: Never
      containers:
        - name: orchestrator
          image: python:3.11-slim
          command: ["/bin/bash", "-c"]
          args:
            - |
              pip install redis
              python /scripts/orchestrator.py
          env:
            - name: REDIS_HOST
              value: "redis"
            - name: REDIS_PORT
              value: "6379"
            - name: DATA_DIR
              value: "/data"
            - name: WAIT_TIMEOUT
              value: "300"
          resources:
            requests:
              cpu: 50m
              memory: 64Mi
            limits:
              cpu: 100m
              memory: 128Mi
          volumeMounts:
            - name: orchestrator-script
              mountPath: /scripts
            - name: aggregator-script
              mountPath: /scripts/aggregator.py
              subPath: aggregator.py
            - name: shared-data
              mountPath: /data
      volumes:
        - name: orchestrator-script
          configMap:
            name: orchestrator-script
        - name: aggregator-script
          configMap:
            name: aggregator-script
        - name: shared-data
          persistentVolumeClaim:
            claimName: shared-data
```

### monitor-daemonset.yaml

```yaml
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: batch-monitor
  namespace: batch-processing
  labels:
    app: batch-monitor
spec:
  selector:
    matchLabels:
      app: batch-monitor
  template:
    metadata:
      labels:
        app: batch-monitor
    spec:
      tolerations:
        - key: node-role.kubernetes.io/control-plane
          operator: Exists
          effect: NoSchedule
        - key: node-role.kubernetes.io/master
          operator: Exists
          effect: NoSchedule
      containers:
        - name: monitor
          image: python:3.11-slim
          command: ["/bin/bash", "-c"]
          args:
            - |
              pip install redis
              python /scripts/monitor.py
          env:
            - name: REDIS_HOST
              value: "redis"
            - name: REDIS_PORT
              value: "6379"
            - name: NODE_NAME
              valueFrom:
                fieldRef:
                  fieldPath: spec.nodeName
          ports:
            - containerPort: 8080
              name: http
          resources:
            requests:
              cpu: 50m
              memory: 64Mi
            limits:
              cpu: 100m
              memory: 128Mi
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 10
            periodSeconds: 30
          readinessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 10
          volumeMounts:
            - name: monitor-script
              mountPath: /scripts
      volumes:
        - name: monitor-script
          configMap:
            name: monitor-script
```

## Why This Solution Works

### 1. Separation of Concerns

The architecture separates responsibilities:

| Component | Responsibility |
|-----------|---------------|
| Data Generator | Creates test data and splits into chunks |
| Batch Workers | Process individual chunks in parallel |
| Orchestrator | Coordinates workflow and waits for completion |
| Aggregator | Combines results from all chunks |
| Monitor | Provides real-time visibility (DaemonSet) |

This separation enables:
- Independent scaling of each component
- Clear failure isolation
- Easier debugging and maintenance

### 2. Indexed Job for Parallel Processing

Using `completionMode: Indexed` enables:

```yaml
completions: 5
parallelism: 3
completionMode: Indexed
```

Each pod gets a unique index via the annotation `batch.kubernetes.io/job-completion-index`. This allows:
- Deterministic chunk assignment (pod 0 processes chunk 0)
- Safe parallel processing without coordination
- Easy result collection by index

### 3. DaemonSet for Cluster-Wide Monitoring

The DaemonSet ensures monitoring runs on every node:

- **Local metrics collection**: Each monitor can collect node-specific data
- **Health endpoints**: `/health`, `/metrics`, `/status` on every node
- **No single point of failure**: Multiple monitor instances
- **Automatic adaptation**: Monitors appear on new nodes

### 4. Redis for Coordination

Redis provides:

- **Progress tracking**: Hash per chunk with status and counts
- **Metrics storage**: Atomic counters for processed items, errors
- **Coordination**: Orchestrator waits for all chunks via Redis

### 5. Shared Storage for Data Pipeline

ReadWriteMany PVC enables:

- Generator writes chunks
- Workers read chunks and write results
- Aggregator reads all results
- All components access the same data

## Verification Steps

### 1. Deploy Infrastructure

```bash
# Create namespace
kubectl create namespace batch-processing

# Deploy Redis and shared storage
kubectl apply -f redis.yaml
kubectl apply -f shared-storage.yaml

# Wait for Redis
kubectl wait --for=condition=ready pod -l app=redis -n batch-processing --timeout=60s
```

### 2. Deploy ConfigMaps

```bash
kubectl apply -f batch-worker-configmap.yaml
kubectl apply -f data-generator-configmap.yaml
kubectl apply -f aggregator-configmap.yaml
kubectl apply -f monitor-configmap.yaml
kubectl apply -f orchestrator-configmap.yaml
```

### 3. Generate Test Data

```bash
kubectl apply -f data-generator-job.yaml

# Wait for completion
kubectl wait --for=condition=complete job/data-generator -n batch-processing --timeout=120s

# Verify data
kubectl logs job/data-generator -n batch-processing
```

### 4. Deploy Monitoring DaemonSet

```bash
kubectl apply -f monitor-daemonset.yaml

# Verify monitors are running on all nodes
kubectl get pods -n batch-processing -l app=batch-monitor -o wide
```

### 5. Run Batch Processing

```bash
# Start workers
kubectl apply -f batch-worker-job.yaml

# Watch progress
kubectl get jobs -n batch-processing -w
```

### 6. Run Orchestrator

```bash
kubectl apply -f orchestrator-job.yaml

# Watch orchestration
kubectl logs -f job/orchestrator -n batch-processing
```

### 7. Check Results

```bash
# View aggregated results
kubectl exec -it job/orchestrator -n batch-processing -- cat /data/aggregated_results.json

# Check metrics
kubectl exec -it deployment/redis -n batch-processing -- redis-cli GET metrics:total_processed
kubectl exec -it deployment/redis -n batch-processing -- redis-cli GET metrics:total_errors
```

### 8. Access Monitor Dashboard

```bash
# Port forward to a monitor pod
kubectl port-forward daemonset/batch-monitor 8080:8080 -n batch-processing

# Access endpoints
curl http://localhost:8080/health
curl http://localhost:8080/metrics
curl http://localhost:8080/status
```

## Common Mistakes to Avoid

### 1. Using ReadWriteOnce for Shared Storage

**Mistake:** Using `ReadWriteOnce` PVC for multi-pod access.

**Problem:** Only one node can mount the PVC; workers on other nodes fail.

**Solution:** Use `ReadWriteMany` for shared access:
```yaml
accessModes:
  - ReadWriteMany
```

### 2. Not Setting NODE_NAME in Workers

**Mistake:** Not passing node information to workers.

**Problem:** Can't track which node processed which items.

**Solution:** Use downward API:
```yaml
env:
  - name: NODE_NAME
    valueFrom:
      fieldRef:
        fieldPath: spec.nodeName
```

### 3. Missing Index in Indexed Job

**Mistake:** Not using the job completion index for chunk assignment.

**Problem:** Workers may process the same chunk or skip chunks.

**Solution:** Use the annotation:
```yaml
env:
  - name: CHUNK_ID
    valueFrom:
      fieldRef:
        fieldPath: metadata.annotations['batch.kubernetes.io/job-completion-index']
```

### 4. No Health Checks on DaemonSet

**Mistake:** No liveness/readiness probes on monitor pods.

**Problem:** Unhealthy monitors remain in rotation; no automatic recovery.

**Solution:** Add probes:
```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 8080
readinessProbe:
  httpGet:
    path: /health
    port: 8080
```

### 5. Not Handling Missing Chunks

**Mistake:** Aggregator assumes all result files exist.

**Problem:** Aggregator fails if any worker failed.

**Solution:** Check for missing files and handle gracefully:
```python
if not result_file.exists():
    print(f"Warning: Result file for chunk {chunk_id} not found")
    continue
```

### 6. Tight Orchestration Loop

**Mistake:** Orchestrator polls Redis in a tight loop.

**Problem:** Excessive CPU usage and Redis load.

**Solution:** Add sleep between polls:
```python
time.sleep(5)  # Poll every 5 seconds
```

## Production Considerations

### 1. Storage Backend

For production, use a distributed storage system:

- **CephFS**: Open source, highly available
- **NFS**: Simple, widely supported
- **Amazon EFS**: Managed NFS for AWS
- **Azure Files**: Managed file share for Azure
- **Google Filestore**: Managed NFS for GCP

### 2. Monitoring Integration

Export metrics to Prometheus:

```python
from prometheus_client import Counter, Gauge, start_http_server

# Define metrics
items_processed = Counter('batch_items_processed_total', 'Total items processed', ['chunk_id'])
items_errors = Counter('batch_items_errors_total', 'Total errors', ['chunk_id'])
chunks_completed = Gauge('batch_chunks_completed', 'Chunks completed')

# Update in worker
items_processed.labels(chunk_id=CHUNK_ID).inc()
```

### 3. Distributed Tracing

Add OpenTelemetry for tracing:

```python
from opentelemetry import trace

tracer = trace.get_tracer("batch-worker")

with tracer.start_as_current_span("process-item") as span:
    span.set_attribute("item.id", item['id'])
    span.set_attribute("chunk.id", CHUNK_ID)
    result = process_item(item)
```

### 4. Workflow Orchestration

For complex workflows, consider:

- **Argo Workflows**: Kubernetes-native workflow engine
- **Apache Airflow**: Workflow orchestration platform
- **Tekton**: Kubernetes-native CI/CD pipelines

### 5. Auto-Scaling Workers

Use Kubernetes auto-scaling:

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: batch-worker
spec:
  # Dynamic parallelism based on queue size
  parallelism: 3
  completions: 5
```

Or use KEDA for event-driven scaling:

```yaml
apiVersion: keda.sh/v1alpha1
kind: ScaledJob
metadata:
  name: batch-worker
spec:
  jobTargetRef:
    template:
      spec:
        containers:
          - name: worker
            image: worker:latest
  pollingInterval: 30
  minReplicaCount: 0
  maxReplicaCount: 10
  triggers:
    - type: redis
      metadata:
        address: redis:6379
        listName: job_queue
        listLength: "5"
```

### 6. Error Recovery

Implement checkpointing for long-running jobs:

```python
def process_chunk():
    # Load checkpoint if exists
    checkpoint_file = Path(DATA_DIR) / f'checkpoint_{CHUNK_ID}.json'
    if checkpoint_file.exists():
        with open(checkpoint_file, 'r') as f:
            checkpoint = json.load(f)
        start_index = checkpoint['last_processed'] + 1
    else:
        start_index = 0

    # Process from checkpoint
    for i in range(start_index, len(items)):
        process_item(items[i])

        # Save checkpoint every 10 items
        if (i + 1) % 10 == 0:
            with open(checkpoint_file, 'w') as f:
                json.dump({'last_processed': i}, f)
```
