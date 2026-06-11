# Exercise 03: Grafana Dashboards for Container Metrics

**Type:** Independent
**Difficulty:** Intermediate
**Time:** 60-90 minutes

---

## Objective

Add Grafana to your monitoring stack, provision a Prometheus datasource automatically, and build a container monitoring dashboard with panels for CPU, memory, network, and disk I/O using PromQL queries.

---

## Prerequisites

- Completed Exercise 02 (cAdvisor + Prometheus running)
- Your `monitoring-setup/` directory from Exercise 02

---

## Part A: Add Grafana to the stack

1. Create Grafana provisioning directories:

```bash
mkdir -p grafana/provisioning/datasources
mkdir -p grafana/provisioning/dashboards
```

2. Create `grafana/provisioning/datasources/prometheus.yml`:

Write a Grafana datasource provisioning file that:
- Defines a datasource named `Prometheus`
- Type: `prometheus`
- Access mode: `proxy`
- URL: `http://prometheus:9090`
- Sets it as the default datasource
- Sets `editable: false`

3. Create `grafana/provisioning/dashboards/dashboard.yml`:

Write a Grafana dashboard provider provisioning file that:
- Uses provider name `default`
- Organization ID: 1
- Type: `file`
- Dashboard path: `/etc/grafana/provisioning/dashboards`
- Allows deletion and editing

4. Update `docker-compose.yml` to add a `grafana` service:

- Image: `grafana/grafana:10.1.0`
- Mount `grafana_data` volume to `/var/lib/grafana`
- Mount `./grafana/provisioning` to `/etc/grafana/provisioning`
- Port mapping: 3001:3000 (map host 3001 to container 3000)
- Environment variables:
  - `GF_SECURITY_ADMIN_PASSWORD=admin`
  - `GF_USERS_ALLOW_SIGN_UP=false`
- Define the `grafana_data` named volume

5. Restart the stack:

```bash
docker compose up -d
```

6. Verify Grafana is running:

```bash
curl -s http://localhost:3001/api/health
```

Expected output: `{"commit":"...","database":"ok","version":"..."}`

---

## Part B: Verify the Prometheus datasource

1. Open Grafana at `http://localhost:3001`
2. Login with `admin` / `admin` (change password if prompted)
3. Navigate to Configuration > Data Sources
4. Verify that `Prometheus` appears as a datasource
5. Click on it and verify the URL is `http://prometheus:9090`
6. Click "Test" -- it should say "Data source is working"

---

## Part C: Create a dashboard with JSON provisioning

Instead of clicking through the Grafana UI, create a dashboard JSON file that Grafana provisions automatically.

Create `grafana/provisioning/dashboards/container-monitoring.json` with the following requirements:

### Panel 1: CPU Usage (%)

- Title: "Container CPU Usage"
- Type: `timeseries`
- Unit: percent (0-100)
- Query:

```promql
rate(container_cpu_usage_seconds_total{name=~".+",name!~".*cadvisor.*",name!~".*prometheus.*"}[5m]) * 100
```

- Legend: `{{name}}`
- Threshold: warning at 70%, critical at 90%

### Panel 2: Memory Usage (MB)

- Title: "Container Memory Usage"
- Type: `timeseries`
- Unit: megabytes (MB)
- Query:

```promql
container_memory_usage_bytes{name=~".+",name!~".*cadvisor.*",name!~".*prometheus.*"} / 1024 / 1024
```

- Legend: `{{name}}`

### Panel 3: Memory Usage as % of Limit

- Title: "Memory Usage (% of Limit)"
- Type: `gauge`
- Unit: percent (0-100)
- Query:

```promql
container_memory_usage_bytes{name=~".+",name!~".*cadvisor.*",name!~".*prometheus.*"} / container_spec_memory_limit_bytes{name=~".+",name!~".*cadvisor.*",name!~".*prometheus.*"} * 100
```

- Thresholds: green at 0, yellow at 70, red at 90

### Panel 4: Network Receive Rate

- Title: "Network Receive Rate"
- Type: `timeseries`
- Unit: bytes/sec (Bps)
- Query:

```promql
rate(container_network_receive_bytes_total{name=~".+",name!~".*cadvisor.*",name!~".*prometheus.*"}[5m])
```

- Legend: `{{name}}`

### Panel 5: Network Transmit Rate

- Title: "Network Transmit Rate"
- Type: `timeseries`
- Unit: bytes/sec (Bps)
- Query:

```promql
rate(container_network_transmit_bytes_total{name=~".+",name!~".*cadvisor.*",name!~".*prometheus.*"}[5m])
```

- Legend: `{{name}}`

### Panel 6: Container Restarts

- Title: "Container Restarts"
- Type: `stat`
- Unit: none
- Query:

```promql
container_restart_count{name=~".+",name!~".*cadvisor.*",name!~".*prometheus.*"}
```

- Legend: `{{name}}`
- Thresholds: green at 0, yellow at 1, red at 3

---

## Part D: Verify the dashboard

1. Restart Grafana to pick up the new dashboard file:

```bash
docker compose restart grafana
```

2. Open Grafana at `http://localhost:3001`
3. Navigate to Dashboards
4. Open the "Container Monitoring" dashboard
5. Verify all 6 panels show data

---

## Part E: Generate load and observe

1. Generate some traffic to your sample app:

```bash
for i in $(seq 1 200); do
  curl -s http://localhost:5000/ > /dev/null &
done
wait
```

2. Observe the dashboard panels:
   - Does CPU usage increase?
   - Does memory usage change?
   - Does the network receive rate spike?

3. Leave the dashboard open for 5 minutes and observe how the graphs smooth out over time.

---

## Success Criteria

- [ ] Grafana is running and accessible at `http://localhost:3001`.
- [ ] Prometheus datasource is provisioned automatically (not added manually).
- [ ] Dashboard has at least 6 panels showing CPU, memory, network, and restart metrics.
- [ ] All panels show data from your sample application container.
- [ ] You can explain why the PromQL queries use `name!~".*cadvisor.*"` filters.
- [ ] The gauge panel shows memory usage as a percentage of the configured limit.

---

## Hints

<details>
<summary>Hint 1: Dashboard JSON structure</summary>

A Grafana dashboard JSON file has this top-level structure:

```json
{
  "dashboard": {
    "id": null,
    "uid": "container-monitoring",
    "title": "Container Monitoring",
    "tags": ["docker", "containers"],
    "timezone": "browser",
    "panels": [ ... ],
    "schemaVersion": 38,
    "version": 1
  },
  "overwrite": true
}
```

Each panel in the `panels` array needs: `id`, `title`, `type`, `gridPos` (for layout), `targets` (for queries), and `fieldConfig` (for units and thresholds).

</details>

<details>
<summary>Hint 2: Panel gridPos layout</summary>

Grafana uses a 24-column grid. Each panel needs a `gridPos` object:

```json
"gridPos": { "h": 8, "w": 12, "x": 0, "y": 0 }
```

- `h`: height in grid units (each unit = 30px)
- `w`: width (12 = half screen, 24 = full screen)
- `x`: column position (0-23)
- `y`: row position (0, 8, 16, ...)

For a 2x3 grid of panels:
```
Panel1 (x:0,y:0)   Panel2 (x:12,y:0)
Panel3 (x:0,y:8)   Panel4 (x:12,y:8)
Panel5 (x:0,y:16)  Panel6 (x:12,y:16)
```

</details>

<details>
<summary>Hint 3: Provisioning file location</summary>

Grafana provisioning files must be in the correct directory. The dashboard JSON file goes in the same directory referenced by the dashboard provider's `path` option. In this exercise, that is `/etc/grafana/provisioning/dashboards/`, which maps to `./grafana/provisioning/dashboards/` on your host.

</details>

<details>
<summary>Hint 4: Filtering out infrastructure containers</summary>

The `name!~".*cadvisor.*"` regex filter excludes cAdvisor containers from the results. Without this filter, your dashboard would show cAdvisor and Prometheus alongside your application containers, which clutters the view. The `=~` operator matches regex, and `!~` excludes regex matches.

</details>

<details>
<summary>Hint 5: Reloading dashboards</summary>

Grafana watches provisioned dashboard files for changes. If you modify the JSON file, Grafana picks up the change automatically within a few seconds. You can also force a reload by restarting the Grafana container:

```bash
docker compose restart grafana
```

</details>
