# Solution 03: Configure Kubernetes Probes for a Stateful App

## Complete Solution

### Project Structure

```
k8s-healthcheck/
  api/
    Dockerfile
    app.py
    requirements.txt
  k8s/
    namespace.yaml
    postgres-statefulset.yaml
    api-deployment.yaml
```

### k8s/namespace.yaml

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: healthcheck-demo
```

### k8s/postgres-statefulset.yaml

```yaml
apiVersion: v1
kind: Service
metadata:
  name: postgres
  namespace: healthcheck-demo
spec:
  clusterIP: None
  selector:
    app: postgres
  ports:
    - port: 5432
---
apiVersion: v1
kind: Service
metadata:
  name: postgres-client
  namespace: healthcheck-demo
spec:
  selector:
    app: postgres
  ports:
    - port: 5432
---
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: postgres
  namespace: healthcheck-demo
spec:
  serviceName: postgres
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
        - name: postgres
          image: postgres:16-alpine
          env:
            - name: POSTGRES_PASSWORD
              value: postgres
            - name: POSTGRES_DB
              value: appdb
          ports:
            - containerPort: 5432

          # Startup probe: give PostgreSQL up to 5 minutes to start.
          # On first run, PostgreSQL initializes the data directory and
          # creates the database. On subsequent runs, it may replay WAL
          # entries. Both can take a long time.
          startupProbe:
            exec:
              command: ["pg_isready", "-U", "postgres"]
            failureThreshold: 30
            periodSeconds: 10

          # Liveness probe: is PostgreSQL stuck?
          # pg_isready checks if the server is accepting connections.
          # This is sufficient for liveness -- if PostgreSQL is stuck,
          # it will stop accepting connections.
          livenessProbe:
            exec:
              command: ["pg_isready", "-U", "postgres"]
            periodSeconds: 10
            failureThreshold: 3

          # Readiness probe: can PostgreSQL execute queries?
          # pg_isready is NOT sufficient here. During WAL replay,
          # PostgreSQL accepts connections but cannot execute queries.
          # A real query (SELECT 1) confirms the server is truly ready.
          readinessProbe:
            exec:
              command:
                - sh
                - -c
                - "PGPASSWORD=postgres psql -U postgres -d appdb -c 'SELECT 1'"
            periodSeconds: 5
            failureThreshold: 3

          volumeMounts:
            - name: postgres-data
              mountPath: /var/lib/postgresql/data
  volumeClaimTemplates:
    - metadata:
        name: postgres-data
      spec:
        accessModes: ["ReadWriteOnce"]
        resources:
          requests:
            storage: 1Gi
```

### k8s/api-deployment.yaml

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api
  namespace: healthcheck-demo
spec:
  replicas: 2
  selector:
    matchLabels:
      app: api
  template:
    metadata:
      labels:
        app: api
    spec:
      # Init container: wait for PostgreSQL to be accepting connections.
      # This is simpler and more reliable than relying on the readiness
      # probe to handle dependency ordering. The init container blocks
      # until PostgreSQL is ready, so the API container never starts
      # without a database.
      initContainers:
        - name: wait-for-postgres
          image: postgres:16-alpine
          command:
            - sh
            - -c
            - |
              echo "Waiting for PostgreSQL..."
              until pg_isready -h postgres-client -U postgres; do
                echo "PostgreSQL is not ready yet. Sleeping..."
                sleep 2
              done
              echo "PostgreSQL is ready!"

      containers:
        - name: api
          image: healthcheck-api:latest
          imagePullPolicy: IfNotPresent
          env:
            - name: DB_HOST
              value: postgres-client
            - name: DB_PASSWORD
              value: postgres
          ports:
            - containerPort: 5000

          # Startup probe: give the API up to 60 seconds to start.
          # The API imports dependencies and connects to the database
          # on startup. With the init container, the database is already
          # accepting connections, but the API still needs time to
          # initialize its Flask application.
          startupProbe:
            httpGet:
              path: /healthz
              port: 5000
            failureThreshold: 30
            periodSeconds: 2

          # Liveness probe: is the API process alive and responsive?
          # Uses /healthz which does NOT check the database.
          # If the database goes down, the API should not restart.
          livenessProbe:
            httpGet:
              path: /healthz
              port: 5000
            periodSeconds: 10
            failureThreshold: 3

          # Readiness probe: can the API serve traffic?
          # Uses /readyz which checks the database connection.
          # If the database is unreachable, the API is removed from
          # the Service endpoints but NOT restarted.
          readinessProbe:
            httpGet:
              path: /readyz
              port: 5000
            periodSeconds: 5
            failureThreshold: 3
```

### api/app.py

```python
from flask import Flask, jsonify
import psycopg2
import os
import time

app = Flask(__name__)

DB_CONFIG = {
    'host': os.environ.get('DB_HOST', 'postgres'),
    'port': os.environ.get('DB_PORT', '5432'),
    'user': os.environ.get('DB_USER', 'postgres'),
    'password': os.environ.get('DB_PASSWORD', 'postgres'),
    'dbname': os.environ.get('DB_NAME', 'appdb'),
}


@app.route('/healthz')
def healthz():
    """Liveness: is the process alive?"""
    return jsonify({'status': 'alive'})


@app.route('/readyz')
def readyz():
    """Readiness: can we serve traffic?"""
    checks = {}
    try:
        conn = psycopg2.connect(**DB_CONFIG, connect_timeout=3)
        cur = conn.cursor()
        cur.execute('SELECT 1')
        cur.close()
        conn.close()
        checks['database'] = 'ok'
    except Exception as e:
        checks['database'] = str(e)

    all_ok = all(v == 'ok' for v in checks.values())
    return jsonify({
        'status': 'ready' if all_ok else 'not_ready',
        'checks': checks,
    }), 200 if all_ok else 503


@app.route('/')
def index():
    return jsonify({'message': 'API is running'})


if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

### api/Dockerfile

```dockerfile
FROM python:3.12-slim
RUN apt-get update && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY . .
EXPOSE 5000
CMD ["python", "app.py"]
```

### api/requirements.txt

```
flask==3.0.0
psycopg2-binary==2.9.9
```

### Build and Deploy

```bash
# Build the API image
docker build -t healthcheck-api:latest ./api/

# Load into your cluster
# For kind:
kind load docker-image healthcheck-api:latest
# For minikube:
minikube image load healthcheck-api:latest

# Deploy
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/postgres-statefulset.yaml
kubectl apply -f k8s/api-deployment.yaml
```

### Verification

```bash
# Watch pods start
kubectl -n healthcheck-demo get pods -w

# Expected progression:
# postgres-0   0/1   Init:0/1   0          5s
# postgres-0   0/1   PodInitializing   0     10s    (startup probe running)
# postgres-0   1/1   Running   0               30s   (ready)
# api-xxx      0/1   Init:0/1   0              5s    (waiting for postgres)
# api-xxx      0/1   ContainerCreating   0     15s
# api-xxx      0/1   Running   0               20s   (startup probe running)
# api-xxx      1/1   Running   0               25s   (ready)

# Check probe events
kubectl -n healthcheck-demo describe pod postgres-0 | grep -A5 "Events"
kubectl -n healthcheck-demo describe pod <api-pod> | grep -A5 "Events"

# Test the API
kubectl -n healthcheck-demo port-forward svc/postgres-client 5432:5432 &
kubectl -n healthcheck-demo port-forward deployment/api 5000:5000

curl http://localhost:5000/healthz
curl http://localhost:5000/readyz

# Simulate database failure
kubectl -n healthcheck-demo exec postgres-0 -- pg_ctl stop -D /var/lib/postgresql/data -m fast

# Watch what happens
kubectl -n healthcheck-demo get pods -w
# Expected:
# - postgres-0: liveness probe fails, container restarts
# - api pods: readiness probe fails (503 from /readyz), removed from Service
# - api pods: liveness probe PASSES (200 from /healthz), NOT restarted
# - When postgres recovers, api readiness probe passes, traffic resumes

# Verify API pod status
kubectl -n healthcheck-demo describe pod <api-pod> | grep "Ready"
# Should show: Ready: False (because readiness probe is failing)
# But the pod should NOT be restarting

# Check events
kubectl -n healthcheck-demo get events --sort-by='.lastTimestamp'
# Should see: "Readiness probe failed" for API pods
# Should NOT see: "Liveness probe failed" for API pods
```

---

## Why This Works

1. **Init containers handle dependency ordering.** The API container never
   starts until PostgreSQL is accepting connections. This is simpler and
   more reliable than trying to handle "database not ready" in the
   application startup code.

2. **The startup probe protects PostgreSQL during WAL replay.** After a
   crash or restart, PostgreSQL replays its write-ahead log. During this
   time, `pg_isready` returns success (the server accepts connections) but
   queries fail. The startup probe uses `pg_isready` with generous timing
   (30 retries * 10 seconds = 5 minutes). The readiness probe uses a real
   query (`SELECT 1`) to catch this state.

3. **The liveness probe does not check the database.** If the database
   goes down, the API pods are removed from the Service (readiness fails)
   but not restarted (liveness passes). When the database recovers, the
   readiness probe passes and traffic resumes automatically.

4. **Separate headless and client Services.** The headless Service
   (`clusterIP: None`) provides stable DNS names for the StatefulSet pods
   (`postgres-0.postgres.healthcheck-demo.svc`). The client Service
   provides a stable endpoint for applications to connect to.

5. **`imagePullPolicy: IfNotPresent`** for the locally built API image.
   Without this, Kubernetes tries to pull `healthcheck-api:latest` from a
   registry and fails.

## Common Mistakes

### Mistake 1: Using pg_isready for readiness

```yaml
readinessProbe:
  exec:
    command: ["pg_isready", "-U", "postgres"]
```

`pg_isready` only checks if the server accepts connections. During WAL
replay, it returns success but the server cannot execute queries. The API
gets marked as ready, starts sending queries, and fails.

### Mistake 2: No startup probe on PostgreSQL

```yaml
# No startupProbe
livenessProbe:
  exec:
    command: ["pg_isready", "-U", "postgres"]
  periodSeconds: 10
  failureThreshold: 3
```

With `failureThreshold: 3` and `periodSeconds: 10`, PostgreSQL has 30
seconds to start. On first run (data directory initialization), it may need
60+ seconds. The liveness probe kills it before it finishes.

### Mistake 3: Checking database in API liveness probe

```yaml
# API liveness probe
livenessProbe:
  httpGet:
    path: /readyz  # WRONG -- this checks the database
    port: 5000
```

A database outage restarts all API pods. Restarting the API does not fix
the database. Use `/healthz` for liveness.

### Mistake 4: Not using init containers

Without the init container, the API starts immediately and tries to connect
to PostgreSQL. If PostgreSQL is not ready, the connection fails. The API
enters a crash loop until PostgreSQL happens to be ready. With the init
container, the API waits cleanly and starts only when its dependency is
available.
