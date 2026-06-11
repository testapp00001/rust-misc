# Exercise 02: Set Up Prometheus and Grafana with node_exporter

**Type:** Guided
**Estimated time:** 35 minutes

## Objective

Deploy a working Prometheus, Grafana, and node_exporter stack using Docker
Compose. Verify that Prometheus is scraping targets, explore the Prometheus
UI, and build a basic dashboard in Grafana.

## Prerequisites

- Docker and Docker Compose installed
- Ports 9090, 3000, and 9100 available
- A web browser

## Instructions

### Step 1 -- Create the Project Structure

```bash
mkdir -p prometheus-lab/prometheus prometheus-lab/grafana/provisioning/datasources
cd prometheus-lab
```

### Step 2 -- Write the Prometheus Configuration

Create `prometheus/prometheus.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']
```

### Step 3 -- Write the Grafana Datasource Provisioning

Create `grafana/provisioning/datasources/prometheus.yml`:

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

### Step 4 -- Write the Docker Compose File

Create `docker-compose.yml` with three services:

1. **prometheus** -- uses `prom/prometheus:latest`, maps port 9090, mounts the
   config file at `/etc/prometheus/prometheus.yml`.
2. **grafana** -- uses `grafana/grafana:latest`, maps port 3000, sets the
   admin password to `admin`, mounts the provisioning directory.
3. **node-exporter** -- uses `prom/node-exporter:latest`, maps port 9100.

Make sure prometheus uses the command flag `--config.file=/etc/prometheus/prometheus.yml`
and `--web.enable-lifecycle`.

### Step 5 -- Start the Stack

```bash
docker compose up -d
```

Verify all three containers are running:

```bash
docker compose ps
```

### Step 6 -- Explore Prometheus

Open `http://localhost:9090` in your browser.

1. Go to **Status > Targets**. Confirm both targets (`prometheus` and
   `node-exporter`) are `UP`.
2. Go to **Graph** and try these queries:
   - `up` -- should show both targets with value 1.
   - `node_load1` -- system load average.
   - `rate(node_cpu_seconds_total{mode="idle"}[5m])` -- CPU idle rate.

### Step 7 -- Explore Grafana

Open `http://localhost:3000` in your browser. Log in with `admin` / `admin`.

1. Confirm the Prometheus data source is configured (Settings > Data Sources).
2. Create a new dashboard with four panels:
   - **CPU Usage** (Stat panel): `100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)`
   - **Memory Usage** (Gauge panel): `(1 - node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes) * 100`
   - **Disk Usage** (Gauge panel): `(1 - node_filesystem_avail_bytes{fstype!="tmpfs",mountpoint="/"} / node_filesystem_size_bytes{fstype!="tmpfs",mountpoint="/"}) * 100`
   - **Network Received** (Time series panel): `rate(node_network_receive_bytes_total{device="eth0"}[5m])`

### Step 8 -- Generate Load and Observe

Open a terminal and generate some CPU and disk activity:

```bash
# CPU load
dd if=/dev/urandom bs=1M count=100 | md5sum

# Disk write
dd if=/dev/zero of=/tmp/testfile bs=1M count=500
rm /tmp/testfile
```

Watch the Grafana panels update in real time.

## Success Criteria

- [ ] All three containers are running (`docker compose ps`).
- [ ] Both Prometheus targets show as `UP` in the targets page.
- [ ] The `up` query in Prometheus returns `1` for both targets.
- [ ] You can query `node_load1` and see real data in Prometheus.
- [ ] Grafana is accessible at `http://localhost:3000` with the default
      credentials.
- [ ] The Prometheus data source is auto-configured in Grafana.
- [ ] Your Grafana dashboard shows four panels with live data.
- [ ] The panels respond to the load you generated in Step 8.

## Hints

<details>
<summary>Hint 1 -- Docker Compose networking</summary>
Docker Compose creates a shared network for all services. Use the service
name (e.g. `node-exporter`) as the hostname in Prometheus configuration.
</details>

<details>
<summary>Hint 2 -- Grafana provisioning path</summary>
Grafana reads provisioning files from `/etc/grafana/provisioning/` inside the
container. Mount your local `grafana/provisioning` directory to that path.
</details>

<details>
<summary>Hint 3 -- Prometheus config reload</summary>
You can reload Prometheus configuration without restarting by sending a
POST request: `curl -X POST http://localhost:9090/-/reload`. This requires
the `--web.enable-lifecycle` flag.
</details>

<details>
<summary>Hint 4 -- node-exporter device names</summary>
The network device name might not be `eth0`. Check available devices by
querying `node_network_receive_bytes_total` in Prometheus and looking at
the `device` label values.
</details>
