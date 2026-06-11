# Solution 03: Grafana Dashboards for Container Metrics

---

## Part A: Add Grafana to the stack

### Grafana provisioning: datasource

`grafana/provisioning/datasources/prometheus.yml`:

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

**Explanation:**
- `access: proxy` means Grafana's backend makes the request to Prometheus (not the browser). This is the correct setting when Grafana and Prometheus are in the same Docker network.
- `isDefault: true` makes this the default datasource for new panels.
- `editable: false` prevents users from changing the datasource configuration through the Grafana UI. This is good practice for provisioned resources.

### Grafana provisioning: dashboard provider

`grafana/provisioning/dashboards/dashboard.yml`:

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

### Updated `docker-compose.yml`

Add the `grafana` service and `grafana_data` volume:

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

**Key points:**
- Port 3001 on the host maps to 3000 in the container (Grafana's default port).
- `grafana_data` volume persists dashboards, settings, and user data.
- The provisioning directory is mounted read-only so Grafana provisions resources from your YAML files on startup.

---

## Part C: Dashboard JSON

`grafana/provisioning/dashboards/container-monitoring.json`:

```json
{
  "dashboard": {
    "id": null,
    "uid": "container-monitoring",
    "title": "Container Monitoring",
    "tags": ["docker", "containers", "cadvisor"],
    "timezone": "browser",
    "schemaVersion": 38,
    "version": 1,
    "refresh": "30s",
    "panels": [
      {
        "id": 1,
        "title": "Container CPU Usage",
        "type": "timeseries",
        "gridPos": { "h": 8, "w": 12, "x": 0, "y": 0 },
        "datasource": { "type": "prometheus", "uid": "PBFA97CFB590B2093" },
        "targets": [
          {
            "expr": "rate(container_cpu_usage_seconds_total{name=~\".+\",name!~\".*cadvisor.*\",name!~\".*prometheus.*\"}[5m]) * 100",
            "legendFormat": "{{name}}",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percent",
            "min": 0,
            "max": 100,
            "custom": {
              "lineWidth": 2,
              "fillOpacity": 15,
              "showPoints": "auto",
              "spanNulls": false
            },
            "thresholds": {
              "mode": "absolute",
              "steps": [
                { "color": "green", "value": null },
                { "color": "yellow", "value": 70 },
                { "color": "red", "value": 90 }
              ]
            }
          }
        },
        "options": {
          "tooltip": { "mode": "multi" },
          "legend": { "displayMode": "list", "placement": "bottom" }
        }
      },
      {
        "id": 2,
        "title": "Container Memory Usage",
        "type": "timeseries",
        "gridPos": { "h": 8, "w": 12, "x": 12, "y": 0 },
        "targets": [
          {
            "expr": "container_memory_usage_bytes{name=~\".+\",name!~\".*cadvisor.*\",name!~\".*prometheus.*\"} / 1024 / 1024",
            "legendFormat": "{{name}}",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "decmbytes",
            "min": 0,
            "custom": {
              "lineWidth": 2,
              "fillOpacity": 15,
              "showPoints": "auto",
              "spanNulls": false
            }
          }
        },
        "options": {
          "tooltip": { "mode": "multi" },
          "legend": { "displayMode": "list", "placement": "bottom" }
        }
      },
      {
        "id": 3,
        "title": "Memory Usage (% of Limit)",
        "type": "gauge",
        "gridPos": { "h": 8, "w": 12, "x": 0, "y": 8 },
        "targets": [
          {
            "expr": "container_memory_usage_bytes{name=~\".+\",name!~\".*cadvisor.*\",name!~\".*prometheus.*\"} / container_spec_memory_limit_bytes{name=~\".+\",name!~\".*cadvisor.*\",name!~\".*prometheus.*\"} * 100",
            "legendFormat": "{{name}}",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percent",
            "min": 0,
            "max": 100,
            "thresholds": {
              "mode": "absolute",
              "steps": [
                { "color": "green", "value": null },
                { "color": "yellow", "value": 70 },
                { "color": "red", "value": 90 }
              ]
            }
          }
        }
      },
      {
        "id": 4,
        "title": "Network Receive Rate",
        "type": "timeseries",
        "gridPos": { "h": 8, "w": 12, "x": 12, "y": 8 },
        "targets": [
          {
            "expr": "rate(container_network_receive_bytes_total{name=~\".+\",name!~\".*cadvisor.*\",name!~\".*prometheus.*\"}[5m])",
            "legendFormat": "{{name}}",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "Bps",
            "min": 0,
            "custom": {
              "lineWidth": 2,
              "fillOpacity": 10
            }
          }
        }
      },
      {
        "id": 5,
        "title": "Network Transmit Rate",
        "type": "timeseries",
        "gridPos": { "h": 8, "w": 12, "x": 0, "y": 16 },
        "targets": [
          {
            "expr": "rate(container_network_transmit_bytes_total{name=~\".+\",name!~\".*cadvisor.*\",name!~\".*prometheus.*\"}[5m])",
            "legendFormat": "{{name}}",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "Bps",
            "min": 0,
            "custom": {
              "lineWidth": 2,
              "fillOpacity": 10
            }
          }
        }
      },
      {
        "id": 6,
        "title": "Container Restarts",
        "type": "stat",
        "gridPos": { "h": 8, "w": 12, "x": 12, "y": 16 },
        "targets": [
          {
            "expr": "container_restart_count{name=~\".+\",name!~\".*cadvisor.*\",name!~\".*prometheus.*\"}",
            "legendFormat": "{{name}}",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "thresholds": {
              "mode": "absolute",
              "steps": [
                { "color": "green", "value": null },
                { "color": "yellow", "value": 1 },
                { "color": "red", "value": 3 }
              ]
            }
          }
        },
        "options": {
          "reduceOptions": {
            "calcs": ["lastNotNull"]
          },
          "colorMode": "background",
          "textMode": "auto"
        }
      }
    ]
  },
  "overwrite": true
}
```

**Note on datasource UID:** The `datasource.uid` field references the UID of the provisioned Prometheus datasource. If you get a "datasource not found" error, either remove the `datasource` field entirely (Grafana will use the default datasource) or check the actual UID by querying:

```bash
curl -s -u admin:admin http://localhost:3001/api/datasources | python3 -m json.tool
```

A simpler approach that avoids the UID issue:

```json
"datasource": { "type": "prometheus", "uid": "${DS_PROMETHEUS}" }
```

Or omit the `datasource` field entirely and Grafana uses the default.

---

## Part D: Verify the dashboard

After restarting Grafana:

```bash
docker compose restart grafana
```

Open `http://localhost:3001`, navigate to Dashboards, and you should see "Container Monitoring" listed.

---

## Part E: Observing load

After running the load generation loop:

1. **CPU Usage panel:** You should see a spike in CPU usage for the `app` container. The spike will smooth out over time as the 5-minute `rate()` window averages.

2. **Memory Usage panel:** Memory usage for the `app` container should show a slight increase (Flask allocates request/response objects). The increase may be modest because the app is lightweight.

3. **Network Receive Rate panel:** A clear spike in bytes/sec should appear during the load test, then return to baseline.

4. **Container Restarts panel:** Should remain at 0 (no restarts during normal load).

---

## Key insights

### Why use `name!~".*cadvisor.*"` filters

cAdvisor reports metrics for ALL containers, including itself and Prometheus. Without these filters, your dashboards would show infrastructure containers alongside your application, making it harder to focus on what matters. The regex filters `name!~` exclude containers whose names match the pattern.

### Gauge vs timeseries panels

- **Timeseries** panels show data over time, useful for trends (CPU, memory, network rates).
- **Gauge** panels show the current value against a range, useful for percentage-of-limit views (memory usage as % of limit).
- **Stat** panels show a single number, useful for counters (restart count).

### Panel layout (gridPos)

Grafana uses a 24-column grid. Each `gridPos` defines:
- `w`: width (12 = half screen, 24 = full screen)
- `h`: height in row units (8 = approximately 240px)
- `x`: horizontal position (0 = left, 12 = center)
- `y`: vertical position (0 = top, 8 = one row down)
