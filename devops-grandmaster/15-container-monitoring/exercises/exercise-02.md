# Exercise 02: Set Up cAdvisor and Prometheus

**Type:** Guided
**Difficulty:** Beginner-Intermediate
**Time:** 45-60 minutes

---

## Objective

Deploy cAdvisor and Prometheus using Docker Compose, configure Prometheus to scrape cAdvisor metrics, and verify that container resource metrics are flowing into Prometheus.

---

## Part A: Create the project structure

1. Create the following directory layout:

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

## Part B: Create a sample application

The application needs to be a simple HTTP server that consumes measurable resources.

Create `app/server.py`:

```python
from flask import Flask, jsonify
import os
import time
import threading

app = Flask(__name__)

# Track active requests for a gauge metric
active_requests = 0
lock = threading.Lock()

@app.route('/')
def index():
    global active_requests
    with lock:
        active_requests += 1
    # Simulate some work
    time.sleep(0.05)
    with lock:
        active_requests -= 1
    return jsonify(status="ok", pid=os.getpid())

@app.route('/health')
def health():
    return jsonify(status="healthy")

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

Create `app/requirements.txt`:

```
flask==3.0.0
```

Create `app/Dockerfile`:

Write a Dockerfile that:
- Uses `python:3.12-slim` as the base image
- Sets a working directory of `/app`
- Copies requirements.txt first (for layer caching)
- Installs the Python dependencies
- Copies the application code
- Exposes port 5000
- Runs the server with `python server.py`

---

## Part C: Configure Prometheus

Create `prometheus.yml` with the following requirements:

- Global scrape interval: 15 seconds
- Global evaluation interval: 15 seconds
- Scrape cAdvisor at `cadvisor:8080` with the job name `cadvisor`
- Scrape Prometheus itself at `localhost:9090` with the job name `prometheus`
- Scrape the sample app at `app:5000` with the job name `sample-app` and metrics path `/`

Note: The sample app in Part B does not expose Prometheus metrics natively. That is intentional -- cAdvisor provides the container-level metrics for it. The `sample-app` scrape job is included for completeness; it will show as "DOWN" in Prometheus targets until the app exposes a `/metrics` endpoint. That is acceptable for this exercise.

---

## Part D: Create Docker Compose

Create `docker-compose.yml` with three services:

**1. `app` service:**
- Build from `./app`
- Port mapping: 5000:5000
- Resource limits: 256MB memory, 0.5 CPU

**2. `cadvisor` service:**
- Image: `gcr.io/cadvisor/cadvisor:v0.47.0`
- Required volume mounts for reading host cgroup data:
  - `/:/rootfs:ro`
  - `/var/run:/var/run:ro`
  - `/sys:/sys:ro`
  - `/var/lib/docker/:/var/lib/docker:ro`
  - `/dev/disk/:/dev/disk:ro`
- Port mapping: 8080:8080
- Must run as privileged (cAdvisor needs access to cgroup files)
- Device: `/dev/kmsg`

**3. `prometheus` service:**
- Image: `prom/prometheus:v2.47.0`
- Mount `prometheus.yml` to `/etc/prometheus/prometheus.yml`
- Named volume `prometheus_data` mounted to `/prometheus`
- Port mapping: 9090:9090
- Command arguments:
  - `--config.file=/etc/prometheus/prometheus.yml`
  - `--storage.tsdb.retention.time=7d`

**4. Define the named volume:**
- `prometheus_data`

---

## Part E: Start and verify

1. Start the stack:

```bash
cd monitoring-setup
docker compose up -d
```

2. Verify all containers are running:

```bash
docker compose ps
```

3. Verify cAdvisor is exposing metrics:

```bash
curl -s http://localhost:8080/metrics | head -20
```

4. Verify Prometheus targets are up:

```bash
# Open in browser or use curl
curl -s http://localhost:9090/api/v1/targets | python3 -m json.tool
```

5. Query Prometheus for a container metric:

```bash
curl -s 'http://localhost:9090/api/v1/query?query=container_memory_usage_bytes{name=~".+"}' | python3 -m json.tool
```

---

## Part F: Explore metrics

Using the Prometheus web UI at `http://localhost:9090`, run these queries:

1. Find all container names that cAdvisor is reporting:

```promql
container_memory_usage_bytes{name=~".+"}
```

2. Get CPU usage per container as a percentage:

```promql
rate(container_cpu_usage_seconds_total{name=~".+"}[5m]) * 100
```

3. Get memory usage in megabytes:

```promql
container_memory_usage_bytes{name=~".+"} / 1024 / 1024
```

4. Get network receive rate in bytes per second:

```promql
rate(container_network_receive_bytes_total{name=~".+"}[5m])
```

5. Check container restart count:

```promql
container_restart_count{name=~".+"}
```

### Questions

- Which container uses the most memory? Why?
- What happens to CPU usage if you run `curl http://localhost:5000/` in a loop?
- Why does cAdvisor itself show up as a container in the metrics?

---

## Success Criteria

- [ ] `docker compose ps` shows three services (app, cadvisor, prometheus) in `running` state.
- [ ] cAdvisor web UI is accessible at `http://localhost:8080`.
- [ ] Prometheus web UI is accessible at `http://localhost:9090`.
- [ ] Prometheus targets page shows `cadvisor` and `prometheus` targets as `UP`.
- [ ] You can query `container_memory_usage_bytes` in Prometheus and see results for your containers.
- [ ] You can explain why `rate(container_cpu_usage_seconds_total[5m]) * 100` gives CPU percentage.

---

## Hints

<details>
<summary>Hint 1: cAdvisor privileged mode</summary>

cAdvisor needs `privileged: true` because it reads cgroup files from `/sys/fs/cgroup/` on the host. Without privileged access, it cannot see other containers' resource usage. This is a legitimate use of privileged mode -- cAdvisor is a read-only observer.

</details>

<details>
<summary>Hint 2: Prometheus config syntax</summary>

The `scrape_configs` section is a list. Each entry has a `job_name` and `static_configs` with a list of `targets`. Targets are strings in `host:port` format. When referencing other Docker Compose services, use the service name as the host (e.g., `cadvisor:8080`).

</details>

<details>
<summary>Hint 3: Dockerfile layer caching</summary>

Copy `requirements.txt` before copying the rest of the application code. Docker caches layers, so if only your Python code changes, `pip install` does not re-run. This is a standard Docker best practice:

```dockerfile
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY . .
```

</details>

<details>
<summary>Hint 4: cAdvisor metric filtering</summary>

When querying in Prometheus, cAdvisor exposes metrics for ALL containers, including itself and Prometheus. Use the `name` label to filter for your specific container. The container name in cAdvisor metrics matches the Docker container name (or the Compose service name with a suffix like `monitoring-setup-app-1`).

</details>

<details>
<summary>Hint 5: Checking Prometheus configuration</summary>

You can validate your Prometheus config without starting the server:

```bash
docker run --rm -v ./prometheus.yml:/etc/prometheus/prometheus.yml \
  prom/prometheus:v2.47.0 \
  promtool check config /etc/prometheus/prometheus.yml
```

</details>
