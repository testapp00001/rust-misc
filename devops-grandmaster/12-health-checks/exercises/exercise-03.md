# Exercise 03: Configure Kubernetes Probes for a Stateful App

**Type:** Independent
**Time:** 45 minutes
**Difficulty:** Medium

## Objective

Deploy a stateful application (PostgreSQL + a Python API) to Kubernetes and
configure liveness, readiness, and startup probes for both the database and
the API. Use `initContainers` for dependency ordering and configure probe
timing so that slow startups do not trigger false restarts.

## Background

Stateful applications are harder to health-check than stateless ones. A
PostgreSQL database has a recovery period after a crash where it replays
write-ahead logs. During recovery, it accepts connections but cannot serve
queries. A naive readiness probe that only checks "can I open a TCP
connection" will mark the database as ready too early, causing the API to
fail when it tries to query.

---

## Instructions

### Step 1: Create the Project Structure

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

### Step 2: Create the API Application

```python
# api/app.py
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
        'checks': checks
    }), 200 if all_ok else 503


@app.route('/')
def index():
    return jsonify({'message': 'API is running'})


if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

```text
# api/requirements.txt
flask==3.0.0
psycopg2-binary==2.9.9
```

```dockerfile
# api/Dockerfile
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

### Step 3: Create the Kubernetes Manifests

**namespace.yaml:**
```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: healthcheck-demo
```

**postgres-statefulset.yaml:**

Create a StatefulSet for PostgreSQL with:

- A headless Service for stable DNS names
- A regular Service for client connections
- Liveness probe using `pg_isready`
- Readiness probe using a real query (`SELECT 1`)
- Startup probe with generous timing (PostgreSQL can take 30-60 seconds
  to initialize on first run)

Fill in the missing probe configurations:

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
          # TODO: Configure startupProbe
          # - Use: pg_isready -U postgres
          # - failureThreshold: 30
          # - periodSeconds: 10
          # This gives PostgreSQL up to 5 minutes to start

          # TODO: Configure livenessProbe
          # - Use: pg_isready -U postgres
          # - periodSeconds: 10
          # - failureThreshold: 3

          # TODO: Configure readinessProbe
          # - Use: psql -U postgres -d appdb -c "SELECT 1"
          # - periodSeconds: 5
          # - failureThreshold: 3
          # NOTE: psql needs the PGPASSWORD env var or a .pgpass file

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

<details>
<summary>Hint</summary>

For the readiness probe that runs a real query, you need to set the
`PGPASSWORD` environment variable. Use an `exec` probe, not `httpGet`:

```yaml
readinessProbe:
  exec:
    command:
      - sh
      - -c
      - "PGPASSWORD=postgres psql -U postgres -d appdb -c 'SELECT 1'"
  periodSeconds: 5
  failureThreshold: 3
```

For the startup probe, use `pg_isready` which checks if the server is
accepting connections:

```yaml
startupProbe:
  exec:
    command: ["pg_isready", "-U", "postgres"]
  failureThreshold: 30
  periodSeconds: 10
```

</details>

**api-deployment.yaml:**

Create a Deployment for the API with:

- An init container that waits for PostgreSQL to be ready
- Liveness probe on `/healthz`
- Readiness probe on `/readyz`
- Startup probe with moderate timing

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
      # TODO: Add initContainers section
      # - Use a busybox or postgres image
      # - Wait for postgres-client:5432 to accept TCP connections
      # - Command: until pg_isready -h postgres-client -U postgres; do sleep 2; done
      #   (or use a TCP check with nc/sh)

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

          # TODO: Configure startupProbe
          # - httpGet /healthz on port 5000
          # - failureThreshold: 30, periodSeconds: 2
          # - This gives the API 60 seconds to start

          # TODO: Configure livenessProbe
          # - httpGet /healthz on port 5000
          # - periodSeconds: 10, failureThreshold: 3

          # TODO: Configure readinessProbe
          # - httpGet /readyz on port 5000
          # - periodSeconds: 5, failureThreshold: 3
```

<details>
<summary>Hint</summary>

The init container approach is simpler than relying on readiness probes for
dependency ordering. The init container blocks until PostgreSQL is ready,
so the API container never starts without a database.

```yaml
initContainers:
  - name: wait-for-postgres
    image: postgres:16-alpine
    command:
      - sh
      - -c
      - |
        until pg_isready -h postgres-client -U postgres; do
          echo "Waiting for postgres..."
          sleep 2
        done
```

</details>

### Step 4: Build and Deploy

```bash
# Build the API image (into your local cluster)
docker build -t healthcheck-api:latest ./api/

# If using kind:
kind load docker-image healthcheck-api:latest

# If using minikube:
minikube image load healthcheck-api:latest

# Deploy
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/postgres-statefulset.yaml
kubectl apply -f k8s/api-deployment.yaml
```

### Step 5: Verify Probe Behavior

```bash
# Watch pods start up
kubectl -n healthcheck-demo get pods -w

# Check probe status
kubectl -n healthcheck-demo describe pod <postgres-pod-name>
kubectl -n healthcheck-demo describe pod <api-pod-name>

# Check that the API is serving traffic
kubectl -n healthcheck-demo port-forward svc/api 5000:5000
curl http://localhost:5000/healthz
curl http://localhost:5000/readyz

# Simulate a database failure
kubectl -n healthcheck-demo exec <postgres-pod-name> -- pg_ctl stop -D /var/lib/postgresql/data

# Watch what happens to the API pods
kubectl -n healthcheck-demo get pods -w
# The API readiness probe should fail (503 from /readyz)
# The API liveness probe should still pass (200 from /healthz)
# The API pods should NOT restart -- they should just stop receiving traffic
```

---

## Success Criteria

- [ ] PostgreSQL StatefulSet starts and becomes ready within 2 minutes
- [ ] The API Deployment pods wait for PostgreSQL via init containers
- [ ] The startup probe prevents liveness probe interference during
      PostgreSQL WAL replay and API initialization
- [ ] When PostgreSQL is stopped, the API readiness probe fails but the
      liveness probe continues to pass -- pods are not restarted
- [ ] When PostgreSQL recovers, the API readiness probe passes again and
      pods resume receiving traffic
- [ ] You can explain why `pg_isready` alone is insufficient for a readiness
      check (it only checks connection acceptance, not query capability)

## Common Mistakes to Avoid

- Using `pg_isready` for both liveness and readiness -- `pg_isready` only
  checks if the server accepts connections, not if it can execute queries
- Not setting a startup probe on PostgreSQL -- during WAL replay, the server
  accepts TCP connections but cannot execute queries, causing false readiness
- Using `imagePullPolicy: Always` for locally built images -- use
  `IfNotPresent` or the image will not be found
- Setting probe timeouts too aggressively -- network calls to a database
  can take longer under load

## What You Should Understand After This Exercise

Kubernetes gives you three independent probes for a reason. The startup
probe protects slow-starting processes from premature liveness failures.
The liveness probe catches deadlocks and unrecoverable states. The readiness
probe controls traffic routing based on real capability. For stateful apps,
the key insight is that "accepting connections" and "ready to serve queries"
are different states, and your readiness probe must check the latter.
