# Solution 05: Horizontal Scaling with Docker Compose

## Part A: Create the Application

### app.py

The application is provided in the exercise. No changes needed.

### Dockerfile

```dockerfile
FROM python:3.11-slim

WORKDIR /app

RUN pip install --no-cache-dir flask

COPY app.py .

EXPOSE 5000

CMD ["python", "app.py"]
```

**Why this works:**
- `python:3.11-slim` is a small base image that includes Python without unnecessary system packages.
- `pip install --no-cache-dir flask` installs Flask without caching pip's download files, keeping the image small.
- `EXPOSE 5000` documents the port but does not actually publish it -- that is done in Docker Compose.
- The app binds to `0.0.0.0` (all interfaces) so it is accessible from outside the container.

---

## Part B: Configure Nginx as a Load Balancer

### nginx.conf

```nginx
events {
    worker_connections 1024;
}

http {
    upstream app_servers {
        server app:5000;
        # Docker Compose DNS resolves 'app' to all replicas
        # Nginx automatically load-balances across them
    }

    server {
        listen 80;

        # Health check endpoint for Nginx itself
        location /nginx-health {
            access_log off;
            return 200 '{"status": "nginx healthy"}';
            add_header Content-Type application/json;
        }

        # Proxy all other requests to the app upstream
        location / {
            proxy_pass http://app_servers;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }
    }
}
```

**Why this works:**
- The `upstream app_servers` block uses the Docker Compose service name `app`. Docker's embedded DNS resolves this to the IP addresses of all replicas.
- Nginx uses round-robin by default to distribute requests across all IPs in the upstream.
- `proxy_set_header` forwards the original client information so the application knows who made the request.
- The `/nginx-health` endpoint returns a 200 directly from Nginx without proxying to the app. This lets you health-check the load balancer independently.

---

## Part C: Write the Docker Compose File

### docker-compose.yml

```yaml
version: "3.8"

services:
  app:
    build: .
    deploy:
      replicas: 4
      resources:
        limits:
          cpus: "0.25"
          memory: 128M
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:5000/health"]
      interval: 10s
      timeout: 5s
      retries: 3
      start_period: 10s
    networks:
      - app-network

  nginx:
    image: nginx:alpine
    ports:
      - "8080:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      app:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost/nginx-health"]
      interval: 10s
      timeout: 5s
      retries: 3
    networks:
      - app-network

networks:
  app-network:
    driver: bridge
```

**Key decisions explained:**

1. **`deploy.replicas: 4`**: Runs 4 identical copies of the app container. Docker Compose handles naming (app-1, app-2, etc.) and DNS registration automatically.

2. **`deploy.resources.limits`**: Each replica gets 0.25 CPU cores and 128MB RAM. With 4 replicas, the total is 1 CPU and 512MB -- leaving resources for Nginx and the host system.

3. **Health check on app**: Uses `curl` to hit the `/health` endpoint. The `start_period: 10s` gives the Flask app time to start before health checks begin. Without this, containers might be marked unhealthy before the app is ready.

4. **`depends_on` with `condition: service_healthy`**: Nginx does not start until the app's health check passes. This prevents Nginx from trying to proxy to app instances that are not ready.

5. **Custom bridge network**: Both services share `app-network`. This is required for Docker DNS to resolve the service name `app` to the replica IPs.

6. **Port 8080 on host**: Maps to port 80 inside the Nginx container. Using 8080 avoids conflicts with other services that might use port 80.

---

## Part D: Verify Horizontal Scaling

### Step 1: Start the services

```bash
cd /path/to/exercise-05
docker compose up -d --build
```

Wait for all health checks to pass:

```bash
docker compose ps
```

You should see `app` with 4 replicas and `nginx` with 1 instance, all healthy.

### Step 2: Send 12 requests and record instances

```bash
for i in $(seq 1 12); do
  echo -n "Request $i: "
  curl -s http://localhost:8080/ | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['instance'])"
done
```

**Expected output (instance names will vary):**

```
Request 1: exercise-05-app-1
Request 2: exercise-05-app-3
Request 3: exercise-05-app-2
Request 4: exercise-05-app-4
Request 5: exercise-05-app-1
Request 6: exercise-05-app-3
Request 7: exercise-05-app-2
Request 8: exercise-05-app-4
Request 9: exercise-05-app-1
Request 10: exercise-05-app-3
Request 11: exercise-05-app-2
Request 12: exercise-05-app-4
```

All 4 instances respond in round-robin order. This confirms Nginx is distributing traffic.

### Step 3: Concurrent request comparison

**With 4 replicas:**

```bash
echo "=== 4 replicas, 4 concurrent requests ==="
time (
  for i in $(seq 1 4); do
    curl -s http://localhost:8080/heavy &
  done
  wait
)
```

**Expected:** All 4 requests complete in roughly the time of a single request (~1-2 seconds) because each runs on a different replica.

**With 1 replica (for comparison):**

```bash
docker compose up -d --scale app=1
echo "=== 1 replica, 4 concurrent requests ==="
time (
  for i in $(seq 1 4); do
    curl -s http://localhost:8080/heavy &
  done
  wait
)
```

**Expected:** The 4 requests take roughly 4x as long (~4-8 seconds) because they all queue on the single replica.

### Results table

| Request | Instance (4 replicas) | Time (4 replicas) | Time (1 replica) |
|---------|----------------------|-------------------|-------------------|
| 1 | app-1 | ~1.2s | ~1.2s |
| 2 | app-2 | ~1.3s | ~2.5s |
| 3 | app-3 | ~1.2s | ~3.8s |
| 4 | app-4 | ~1.1s | ~5.1s |
| **Total** | | **~1.3s (parallel)** | **~5.1s (sequential)** |

The key insight: with horizontal scaling, concurrent requests are processed in parallel across replicas. The total wall-clock time is roughly the time of a single request, not the sum of all requests.

---

## Part E: Scale Up and Down

### Scale up to 8 replicas

```bash
docker compose up -d --scale app=8
```

Verify 8 replicas are running:

```bash
docker compose ps | grep app | wc -l
# Should output: 8
```

Send 16 requests:

```bash
for i in $(seq 1 16); do
  echo -n "Request $i: "
  curl -s http://localhost:8080/ | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['instance'])"
done
```

**Expected:** All 8 unique instance names appear, each serving 2 requests (16 / 8 = 2).

### Scale down to 2 replicas

```bash
docker compose up -d --scale app=2
```

Verify 2 replicas:

```bash
docker compose ps | grep app | wc -l
# Should output: 2
```

Send 10 requests:

```bash
for i in $(seq 1 10); do
  echo -n "Request $i: "
  curl -s http://localhost:8080/ | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['instance'])"
done
```

**Expected:** Only 2 unique instance names appear.

### Did any requests fail during scale-down?

**No.** Docker Compose sends SIGTERM to the containers being removed. The Flask app receives the signal and can finish processing any in-flight requests before shutting down. Nginx detects that the upstream server is gone (via health check failure or connection close) and routes future requests to the remaining replicas.

In a production environment, you would configure:
1. A `stop_grace_period` in Docker Compose to give the app time to finish in-flight requests.
2. Connection draining in the load balancer to stop sending new requests before the container stops.
3. Health checks with a short interval so the load balancer detects the removed instance quickly.

---

## Common Mistakes

1. **Not installing curl in the container for health checks.** The `python:3.11-slim` image does not include curl. Install it in the Dockerfile: `RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*`. Without curl, the health check fails and containers are perpetually unhealthy.

2. **Forgetting `EXPOSE` and `ports` in the right places.** The app's Dockerfile uses `EXPOSE 5000` (documentation only). The Nginx service uses `ports: "8080:80"` to publish to the host. Do not publish the app's port to the host -- only Nginx should be accessible from outside.

3. **Not using a custom network.** Without an explicit network, Docker Compose creates a default network. This works, but an explicit network makes the architecture clear and allows you to add network-level isolation later.

4. **Scaling the Nginx service instead of the app service.** `docker compose up -d --scale nginx=3` would run 3 Nginx instances, all binding to port 80 on the host -- which would fail with a port conflict. Scale the app service, not the load balancer.

5. **Not checking that Docker DNS resolves to all replicas.** If only one instance appears in responses, check that all replicas are healthy. Docker DNS only includes healthy containers in the resolution. Run `docker compose ps` to verify all replicas show "healthy" status.

6. **Using `links` instead of relying on Docker DNS.** The `links` directive is legacy. In Docker Compose with a shared network, services can reach each other by service name automatically. No `links` needed.

---

## Relevant README Sections

- [Lab B: Horizontal Scaling with Docker Compose](../README.md#lab-b-horizontal-scaling-with-docker-compose)
- [The Right Way -- Scale Horizontally](../README.md#3-the-right-way--scale-horizontally-scale-out)
- [Stateful vs Stateless Applications](../README.md#stateful-vs-stateless-applications)
