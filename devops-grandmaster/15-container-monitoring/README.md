# Module 15: Container Monitoring — CPU, Memory, Network, Disk Metrics

> **Previous Module (14):** Secrets Management
> **Previous Limitation:** You have configured and secured your containers, but you have zero visibility into what they are actually doing at runtime.
> **This Module Solves That:** Real-time observability into container resource usage, health, and performance.

---

## 1. The Problem

Your application is running in production. Users report it is slow. You log into the server and see 10 containers running. Which one is the problem?

- Is the web app consuming all available memory?
- Is the database disk I/O saturated?
- Is the Redis cache container restarting repeatedly?
- Is the nginx proxy dropping connections?

Without monitoring, you are debugging blind. You cannot:

- Detect problems before users report them
- Identify which container is the bottleneck
- Make informed decisions about resource limits
- Plan capacity for growth
- Correlate application performance with infrastructure metrics

**The core problem:** Running containers without monitoring is like driving a car with no dashboard. You only find out something is wrong when the engine seizes.

---

## 2. The Naive Way

### Manually checking with `docker stats`

```bash
docker stats
```

Output:

```
CONTAINER ID   NAME          CPU %   MEM USAGE / LIMIT   MEM %   NET I/O       BLOCK I/O     PIDS
a1b2c3d4e5f6   web-app       2.34%   128MiB / 512MiB     25.0%   1.2kB / 0B    0B / 0B       15
f6e5d4c3b2a1   postgres      0.56%   256MiB / 1GiB       25.0%   0B / 0B       12MB / 4MB    8
b2a1f6e5d4c3   redis         0.12%   32MiB / 256MiB      12.5%   0B / 0B       0B / 0B       4
```

### Why it fails

1. **No history.** `docker stats` shows the current moment. You cannot see what happened 5 minutes ago, let alone last week.
2. **No alerting.** You have to be staring at the terminal when the problem occurs.
3. **No correlation.** You cannot see CPU and memory trends side by side.
4. **No persistence.** Close the terminal, lose all data.
5. **Manual process.** Requires a human to run it and interpret the output.

---

## 3. The Right Way

### The monitoring stack: Prometheus + Grafana

The industry standard for container monitoring is:

1. **cAdvisor** — Collects container metrics (CPU, memory, network, disk)
2. **Prometheus** — Scrapes and stores metrics time-series data
3. **Grafana** — Visualizes metrics in dashboards

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Container   │────>│   cAdvisor   │────>│  Prometheus  │
│  (your app)  │     │  (metrics)   │     │  (storage)   │
└──────────────┘     └──────────────┘     └──────┬───────┘
                                                  │
                                                  v
                                          ┌──────────────┐
                                          │   Grafana    │
                                          │ (dashboards) │
                                          └──────────────┘
```

### Key metrics to monitor

**The USE Method** (for every resource, check Utilization, Saturation, Errors):

| Resource | Utilization | Saturation | Errors |
|----------|-------------|------------|--------|
| CPU | `container_cpu_usage_seconds_total` | `container_cpu_cfs_throttled_seconds_total` | — |
| Memory | `container_memory_usage_bytes` | `container_memory_working_set_bytes` | OOM kills |
| Disk I/O | `container_fs_io_time_seconds_total` | `container_fs_io_current` | I/O errors |
| Network | `container_network_receive_bytes_total` | `container_network_transmit_packets_dropped_total` | Packet errors |

**The RED Method** (for every service, check Rate, Errors, Duration):

| Metric | Description |
|--------|-------------|
| Rate | Requests per second |
| Errors | Error rate (4xx, 5xx) |
| Duration | Latency (p50, p95, p99) |

### Container resource limits

Always set resource limits. Without them, one container can starve the entire host:

```bash
# Limit memory to 512MB, CPU to 1.5 cores
docker run -d \
  --memory=512m \
  --memory-swap=512m \
  --cpus=1.5 \
  --name myapp \
  myapp:1.0.0
```

```yaml
# Docker Compose
services:
  app:
    image: myapp:1.0.0
    deploy:
      resources:
        limits:
          cpus: '1.5'
          memory: 512M
        reservations:
          cpus: '0.5'
          memory: 256M
```

### OOM Kills (Out of Memory)

When a container exceeds its memory limit, the Linux kernel kills it (OOM killer). You can detect this:

```bash
# Check if a container was OOM killed
docker inspect myapp | grep OOMKilled
# "OOMKilled": true

# Check kernel logs
dmesg | grep -i "oom\|killed"

# Check container events
docker events --filter event=oom
```

**Preventing OOM kills:**
1. Set appropriate memory limits (not too low, not too high)
2. Monitor memory usage trends
3. Set `--memory-swap` equal to `--memory` (prevent swap usage)
4. Use `--oom-kill-disable` only for critical containers with proper limits

---

## 4. The Production Way

### Complete monitoring stack with Docker Compose

```yaml
version: '3.8'

services:
  # --- Your Application ---
  app:
    image: myapp:1.0.0
    ports:
      - "3000:3000"
    deploy:
      resources:
        limits:
          memory: 512M
          cpus: '1.0'

  # --- cAdvisor: Container metrics exporter ---
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

  # --- Prometheus: Metrics storage ---
  prometheus:
    image: prom/prometheus:v2.47.0
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    ports:
      - "9090:9090"
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--storage.tsdb.retention.time=30d'

  # --- Grafana: Visualization ---
  grafana:
    image: grafana/grafana:10.1.0
    volumes:
      - grafana_data:/var/lib/grafana
      - ./grafana/provisioning:/etc/grafana/provisioning
    ports:
      - "3001:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_USERS_ALLOW_SIGN_UP=false

volumes:
  prometheus_data:
  grafana_data:
```

### Prometheus configuration

**prometheus.yml:**

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  # Scrape cAdvisor metrics
  - job_name: 'cadvisor'
    static_configs:
      - targets: ['cadvisor:8080']
    metrics_path: /metrics

  # Scrape Prometheus itself
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  # Scrape your application (if it exposes /metrics)
  - job_name: 'app'
    static_configs:
      - targets: ['app:3000']
    metrics_path: /metrics
```

### Grafana provisioning

**grafana/provisioning/datasources/prometheus.yml:**

```yaml
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: false
```

**grafana/provisioning/dashboards/dashboard.yml:**

```yaml
apiVersion: 1

providers:
  - name: 'default'
    orgId: 1
    folder: ''
    type: file
    disableDeletion: false
    editable: true
    options:
      path: /etc/grafana/provisioning/dashboards
      foldersFromFilesStructure: false
```

### Essential Grafana dashboard queries

**CPU usage per container:**

```promql
rate(container_cpu_usage_seconds_total{name=~".+"}[5m]) * 100
```

**Memory usage per container:**

```promql
container_memory_usage_bytes{name=~".+"} / 1024 / 1024
```

**Memory usage as percentage of limit:**

```promql
container_memory_usage_bytes{name=~".+"} / container_spec_memory_limit_bytes{name=~".+"} * 100
```

**Network receive rate (bytes/sec):**

```promql
rate(container_network_receive_bytes_total{name=~".+"}[5m])
```

**Container restart count:**

```promql
container_restart_count{name=~".+"}
```

**Disk I/O rate:**

```promql
rate(container_fs_io_time_seconds_total{name=~".+"}[5m])
```

### Alerting rules

**prometheus.yml (with alerting):**

```yaml
global:
  scrape_interval: 15s

rule_files:
  - /etc/prometheus/alerts.yml

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']

scrape_configs:
  - job_name: 'cadvisor'
    static_configs:
      - targets: ['cadvisor:8080']
```

**alerts.yml:**

```yaml
groups:
  - name: container_alerts
    rules:
      - alert: ContainerHighCPU
        expr: rate(container_cpu_usage_seconds_total{name=~".+"}[5m]) * 100 > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Container {{ $labels.name }} CPU > 80%"
          description: "Container {{ $labels.name }} has been above 80% CPU for 5 minutes."

      - alert: ContainerHighMemory
        expr: container_memory_usage_bytes{name=~".+"} / container_spec_memory_limit_bytes{name=~".+"} * 100 > 90
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Container {{ $labels.name }} memory > 90% of limit"
          description: "Container {{ $labels.name }} is at risk of OOM kill."

      - alert: ContainerRestarting
        expr: increase(container_restart_count{name=~".+"}[15m]) > 3
        labels:
          severity: critical
        annotations:
          summary: "Container {{ $labels.name }} restarting frequently"
          description: "Container {{ $labels.name }} has restarted {{ $value }} times in 15 minutes."
```

---

## 5. Hands-On Lab

### Lab: Set up complete container monitoring

**Objective:** Deploy Prometheus, Grafana, and cAdvisor to monitor a multi-container application.

#### Step 1: Create the project

```bash
mkdir -p monitoring-lab/grafana/provisioning/{datasources,dashboards} && cd monitoring-lab
```

#### Step 2: Create a sample application with custom metrics

**app.js:**

```javascript
const express = require('express');
const client = require('prom-client');

const app = express();
const register = client.register;

// Collect default metrics (CPU, memory, event loop, etc.)
client.collectDefaultMetrics({ prefix: 'myapp_' });

// Custom metrics
const httpRequestDuration = new client.Histogram({
  name: 'myapp_http_request_duration_seconds',
  help: 'Duration of HTTP requests in seconds',
  labelNames: ['method', 'route', 'status'],
  buckets: [0.01, 0.05, 0.1, 0.5, 1, 2, 5],
});

const httpRequestsTotal = new client.Counter({
  name: 'myapp_http_requests_total',
  help: 'Total number of HTTP requests',
  labelNames: ['method', 'route', 'status'],
});

const activeConnections = new client.Gauge({
  name: 'myapp_active_connections',
  help: 'Number of active connections',
});

// Middleware to track requests
app.use((req, res, next) => {
  const start = Date.now();
  activeConnections.inc();

  res.on('finish', () => {
    const duration = (Date.now() - start) / 1000;
    httpRequestDuration.observe(
      { method: req.method, route: req.path, status: res.statusCode },
      duration
    );
    httpRequestsTotal.inc(
      { method: req.method, route: req.path, status: res.statusCode }
    );
    activeConnections.dec();
  });
  next();
});

// Simulate varying load
app.get('/', (req, res) => {
  const delay = Math.random() * 100;
  setTimeout(() => res.json({ message: 'Hello', delay }), delay);
});

app.get('/heavy', (req, res) => {
  // Simulate CPU-intensive work
  let sum = 0;
  for (let i = 0; i < 1000000; i++) sum += Math.random();
  res.json({ message: 'Heavy computation', result: sum });
});

app.get('/memory', (req, res) => {
  // Allocate some memory temporarily
  const arr = new Array(1000000).fill('x');
  res.json({ message: 'Memory allocated', size: arr.length });
});

// Prometheus metrics endpoint
app.get('/metrics', async (req, res) => {
  res.set('Content-Type', register.contentType);
  res.end(await register.metrics());
});

app.listen(3000, () => console.log('App running on port 3000'));
```

**package.json:**

```json
{
  "name": "monitoring-lab",
  "version": "1.0.0",
  "dependencies": {
    "express": "^4.18.2",
    "prom-client": "^15.0.0"
  }
}
```

**Dockerfile:**

```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm install --production
COPY app.js .
EXPOSE 3000
CMD ["node", "app.js"]
```

#### Step 3: Create Prometheus configuration

**prometheus.yml:**

```yaml
global:
  scrape_interval: 10s
  evaluation_interval: 10s

scrape_configs:
  - job_name: 'cadvisor'
    static_configs:
      - targets: ['cadvisor:8080']

  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'myapp'
    static_configs:
      - targets: ['app:3000']
    metrics_path: /metrics
```

#### Step 4: Create Grafana datasource provisioning

**grafana/provisioning/datasources/prometheus.yml:**

```yaml
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: true
```

#### Step 5: Create docker-compose.yml

```yaml
version: '3.8'

services:
  app:
    build: .
    ports:
      - "3000:3000"
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

  grafana:
    image: grafana/grafana:10.1.0
    volumes:
      - grafana_data:/var/lib/grafana
      - ./grafana/provisioning:/etc/grafana/provisioning
    ports:
      - "3001:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_USERS_ALLOW_SIGN_UP=false

volumes:
  prometheus_data:
  grafana_data:
```

#### Step 6: Start the stack

```bash
docker compose up -d
```

#### Step 7: Generate some load

```bash
# Install a load testing tool
pip install locust

# Or use a simple loop
for i in $(seq 1 1000); do
  curl -s http://localhost:3000/ > /dev/null &
  curl -s http://localhost:3000/heavy > /dev/null &
  curl -s http://localhost:3000/memory > /dev/null &
  if [ $((i % 100)) -eq 0 ]; then
    echo "Sent $i batches of requests"
    sleep 1
  fi
done
wait
```

#### Step 8: Explore the monitoring stack

**cAdvisor (raw container metrics):**
```bash
open http://localhost:8080
```

**Prometheus (query and explore metrics):**
```bash
open http://localhost:9090

# Try these queries:
# CPU usage: rate(container_cpu_usage_seconds_total{name="monitoring-lab-app-1"}[5m])
# Memory: container_memory_usage_bytes{name="monitoring-lab-app-1"}
# Custom: myapp_http_request_duration_seconds_count
```

**Grafana (dashboards):**
```bash
open http://localhost:3001
# Login: admin / admin
# Add Prometheus datasource: http://prometheus:9090
```

#### Step 9: Create a Grafana dashboard

1. Go to Grafana at `http://localhost:3001`
2. Login with `admin`/`admin`
3. Go to Dashboards > New Dashboard
4. Add panels with these queries:

**Panel 1: CPU Usage**
```promql
rate(container_cpu_usage_seconds_total{name=~".+"}[5m]) * 100
```

**Panel 2: Memory Usage**
```promql
container_memory_usage_bytes{name=~".+"} / 1024 / 1024
```

**Panel 3: Request Rate**
```promql
rate(myapp_http_requests_total[5m])
```

**Panel 4: Request Latency (p95)**
```promql
histogram_quantile(0.95, rate(myapp_http_request_duration_seconds_bucket[5m]))
```

**Panel 5: Container Restarts**
```promql
container_restart_count{name=~".+"}
```

#### Step 10: Observe OOM behavior

```bash
# Watch memory usage
docker stats monitoring-lab-app-1

# Stress the app with memory requests
for i in $(seq 1 100); do
  curl -s http://localhost:3000/memory &
done
wait

# Check if container was OOM killed
docker inspect monitoring-lab-app-1 | grep OOMKilled

# Check container events
docker events --filter container=monitoring-lab-app-1 --filter event=oom
```

#### Cleanup

```bash
docker compose down -v
```

---

## 6. Limitation

You now have full visibility into your containers. You can see CPU spikes, memory growth, network traffic, and container restarts in real time.

But building and deploying your application is still a manual process:

1. You write code locally
2. You manually run `docker build`
3. You manually run `docker push`
4. You manually run `docker compose up`
5. You manually verify it works

This is error-prone and slow. When you push code to git, nothing happens automatically. There is no automated testing, no automated deployment, no automated rollback.

**You need to automate the entire build-test-deploy pipeline.**

---

## 7. Next Topic

**Module 16: CI/CD Pipelines** — We will learn how to automate the entire software delivery process using GitHub Actions, GitLab CI, and Jenkins. Every push to git will automatically build, test, scan, and deploy your containers.
