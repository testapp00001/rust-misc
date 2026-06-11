# Cheatsheet: Centralized Logging

## ELK Stack

| Component | Role |
|-----------|------|
| Elasticsearch | Storage and search |
| Logstash | Processing and transformation |
| Kibana | Visualization |

## EFK Stack

| Component | Role |
|-----------|------|
| Elasticsearch | Storage and search |
| Fluentd/Fluent Bit | Log collection |
| Kibana | Visualization |

## Loki + Grafana
```yaml
# docker-compose.yml
services:
  loki:
    image: grafana/loki:latest
    ports:
      - "3100:3100"

  promtail:
    image: grafana/promtail:latest
    volumes:
      - /var/log:/var/log
    command: -config.file=/etc/promtail/config.yml
```

## Log Query Examples

### Elasticsearch (KQL)
```
level:ERROR AND service:api
message:"database connection" AND NOT status:200
```

### Loki (LogQL)
```
{job="api"} |= "error"
{service="api"} | json | status >= 500
{job="api"} |= "error" | logfmt | duration > 1s
```

## Log Retention
```
Hot storage:  7 days  (fast SSD)
Warm storage: 30 days (standard SSD)
Cold storage: 90 days (HDD/S3)
Delete:       After 90 days
```
