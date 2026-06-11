# Solution 02: Set Up cAdvisor and Prometheus

---

## Part A: Project structure

```
monitoring-setup/
  docker-compose.yml
  prometheus.yml
  app/
    Dockerfile
    server.py
    requirements.txt
```

---

## Part B: Sample application

### `app/server.py`

The provided server.py from the exercise is correct as-is. No changes needed.

### `app/requirements.txt`

```
flask==3.0.0
```

### `app/Dockerfile`

```dockerfile
FROM python:3.12-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY server.py .

EXPOSE 5000

CMD ["python", "server.py"]
```

**Explanation:** `requirements.txt` is copied before `server.py` so Docker caches the `pip install` layer. If only `server.py` changes, the dependency installation step is skipped on rebuild.

---

## Part C: Prometheus configuration

### `prometheus.yml`

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'cadvisor'
    static_configs:
      - targets: ['cadvisor:8080']

  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'sample-app'
    static_configs:
      - targets: ['app:5000']
    metrics_path: /
```

**Explanation:**
- `cadvisor` job scrapes the cAdvisor container metrics exporter.
- `prometheus` job scrapes Prometheus itself for self-monitoring (scrape duration, TSDB stats, etc.).
- `sample-app` job is configured to scrape the app, but since the Flask app does not expose `/metrics`, this target will show as `DOWN` in Prometheus. This is expected and acceptable for this exercise -- cAdvisor provides the container-level metrics for the app.

---

## Part D: Docker Compose

### `docker-compose.yml`

```yaml
services:
  app:
    build: ./app
    ports:
      - "5000:5000"
    deploy:
      resources:
        limits:
          memory: 256M
          cpus: '0.5'

  cadvisor:
    image: gcr.io/cadvisor/cadvisor:v0.47.0
    volumes:
      - /:/rootfs:ro
      - /var/run:/var/run:ro
      - /sys:/sys:ro
      - /var/lib/docker/:/var/lib/docker:ro
      - /dev/disk/:/dev/disk:ro
    ports:
      - "8080:8080"
    privileged: true
    devices:
      - /dev/kmsg

  prometheus:
    image: prom/prometheus:v2.47.0
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    ports:
      - "9090:9090"
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.retention.time=7d'

volumes:
  prometheus_data:
```

**Key points:**
- The `app` service has `deploy.resources.limits` set to 256MB memory and 0.5 CPU. These limits are enforced by Docker via cgroups and are what cAdvisor reports.
- cAdvisor requires `privileged: true` and specific volume mounts to read the host's cgroup files. Without these, it cannot see other containers' metrics.
- The `prometheus_data` named volume persists Prometheus data across container restarts.

---

## Part E: Verification

### Starting the stack

```bash
cd monitoring-setup
docker compose up -d
```

Expected output: Three services start successfully.

### Verify containers are running

```bash
docker compose ps
```

Expected: `app`, `cadvisor`, and `prometheus` all show `Up` status.

### Verify cAdvisor exposes metrics

```bash
curl -s http://localhost:8080/metrics | head -20
```

Expected: Prometheus-formatted metrics output starting with `# HELP` and `# TYPE` lines.

### Verify Prometheus targets

```bash
curl -s http://localhost:9090/api/v1/targets | python3 -m json.tool
```

Expected output (abbreviated):

```json
{
  "data": {
    "activeTargets": [
      {
        "labels": { "job": "cadvisor" },
        "health": "up",
        ...
      },
      {
        "labels": { "job": "prometheus" },
        "health": "up",
        ...
      },
      {
        "labels": { "job": "sample-app" },
        "health": "down",
        ...
      }
    ]
  }
}
```

Note: `sample-app` is `DOWN` because the Flask app does not expose a `/metrics` endpoint. This is expected.

### Query container metrics

```bash
curl -s 'http://localhost:9090/api/v1/query?query=container_memory_usage_bytes{name=~".+"}' | python3 -m json.tool
```

Expected: JSON response with memory usage values for each container (app, cadvisor, prometheus).

---

## Part F: Exploring metrics -- Answers

### Q: Which container uses the most memory? Why?

A: Typically **Prometheus** uses the most memory. Prometheus maintains an in-memory database for recent metrics, a query engine, and internal caches. cAdvisor uses moderate memory because it collects metrics for all containers. The sample app uses minimal memory (the Flask server is lightweight).

The exact ranking depends on the system, but Prometheus is almost always the heaviest because it stores time-series data in memory before flushing to disk.

### Q: What happens to CPU usage if you run `curl http://localhost:5000/` in a loop?

A: The `app` container's CPU usage increases. The `rate(container_cpu_usage_seconds_total{name=~".*app.*"}[5m]) * 100` query will show a rising CPU percentage. The sleep(0.05) in the Flask handler means each request consumes a small amount of CPU, and many concurrent requests add up.

However, the increase may be modest because:
1. The `time.sleep(0.05)` releases the GIL and does not consume CPU.
2. The actual work (Flask request parsing, JSON serialization) is lightweight.
3. The 0.5 CPU limit means Docker throttles the container if it tries to use more.

### Q: Why does cAdvisor itself show up as a container in the metrics?

A: cAdvisor runs as a Docker container itself. The cgroup files for the cAdvisor container are visible on the host filesystem just like any other container. cAdvisor reports metrics for ALL containers it can see, including itself. This is why PromQL queries often include filters like `name!~".*cadvisor.*"` to exclude cAdvisor from application-focused dashboards.

---

## Troubleshooting

### cAdvisor shows no metrics for other containers

- Verify `privileged: true` is set.
- Verify all required volume mounts are present.
- Check cAdvisor logs: `docker compose logs cadvisor`

### Prometheus targets show "connection refused"

- Verify the service names match in `prometheus.yml` targets (use Docker Compose service names, not `localhost`).
- Wait 15-30 seconds after starting -- Prometheus needs time to discover targets.

### App container OOM kills

- The 256MB limit is tight for Python + Flask. If the app crashes, increase to 512M.
- Check: `docker inspect monitoring-setup-app-1 | grep OOMKilled`
