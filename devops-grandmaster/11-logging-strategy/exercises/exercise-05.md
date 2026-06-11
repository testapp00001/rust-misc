# Exercise 05: Set Up Docker Logging Drivers to Forward Logs to a Central System

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Configure Docker to forward container logs to a centralized logging system
using logging drivers. You will set up a local ELK (Elasticsearch, Logstash,
Kibana) or EFK (Elasticsearch, Fluentd, Kibana) stack, configure Docker's
logging drivers to forward logs, and verify that logs from multiple containers
appear in a single searchable interface.

## Background

`docker logs` works fine for a single host with a few containers. In
production, you have dozens of containers across multiple hosts. You need a
central place to search, filter, and alert on logs from all services.
Docker logging drivers forward logs from containers to external systems
without changing your application code.

---

## Instructions

### Step 1: Set Up the Central Logging Stack

Create a `docker-compose.yml` that runs the logging infrastructure:

**Option A (Recommended): EFK Stack**

```yaml
# docker-compose.yml
version: "3.8"

services:
  elasticsearch:
    image: elasticsearch:8.12.0
    environment:
      - discovery.type=single-node
      - xpack.security.enabled=false
      - "ES_JAVA_OPTS=-Xms512m -Xmx512m"
    ports:
      - "9200:9200"
    volumes:
      - es-data:/usr/share/elasticsearch/data
    healthcheck:
      test: ["CMD-SHELL", "curl -f http://localhost:9200/_cluster/health || exit 1"]
      interval: 10s
      timeout: 5s
      retries: 5

  kibana:
    image: kibana:8.12.0
    environment:
      - ELASTICSEARCH_HOSTS=http://elasticsearch:9200
    ports:
      - "5601:5601"
    depends_on:
      elasticsearch:
        condition: service_healthy

volumes:
  es-data:
```

**Option B: Loki + Grafana**

```yaml
# docker-compose.yml
version: "3.8"

services:
  loki:
    image: grafana/loki:2.9.0
    ports:
      - "3100:3100"
    command: -config.file=/etc/loki/local-config.yaml

  grafana:
    image: grafana/grafana:10.2.0
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    depends_on:
      - loki
```

Choose one option and get it running. Verify the logging system is healthy
before proceeding.

<details>
<summary>Hint</summary>

For the EFK stack, Elasticsearch needs at least 1-2 minutes to start. Use
`docker compose logs -f elasticsearch` to watch for the "started" message.
Do not proceed until the health check passes.

For Loki, verify with: `curl http://localhost:3100/ready`

</details>

### Step 2: Create a Sample Application

Create a simple application that generates different types of log output.
This simulates a real service that the team would run alongside the logging
infrastructure.

```python
# sample-app/app.py
from flask import Flask, jsonify
import logging
import json
import sys
import os
import random
import time
from datetime import datetime, timezone

class JSONFormatter(logging.Formatter):
    def format(self, record):
        log_entry = {
            'timestamp': datetime.now(timezone.utc).isoformat(),
            'level': record.levelname,
            'message': record.getMessage(),
            'service': os.environ.get('SERVICE_NAME', 'sample-app'),
        }
        if hasattr(record, 'extra_data'):
            log_entry.update(record.extra_data)
        if record.exc_info:
            log_entry['exception'] = self.formatException(record.exc_info)
        return json.dumps(log_entry)

logger = logging.getLogger('app')
handler = logging.StreamHandler(sys.stdout)
handler.setFormatter(JSONFormatter())
logger.addHandler(handler)
logger.setLevel(os.environ.get('LOG_LEVEL', 'INFO'))

app = Flask(__name__)

@app.route('/')
def index():
    logger.info("Home page accessed", extra={'extra_data': {'endpoint': '/'}})
    return jsonify({'status': 'ok'})

@app.route('/error')
def trigger_error():
    try:
        raise ValueError("Simulated error for testing")
    except Exception:
        logger.error("Unhandled exception", exc_info=True,
                     extra={'extra_data': {'endpoint': '/error'}})
        return jsonify({'error': 'Internal error'}), 500

@app.route('/slow')
def slow_endpoint():
    delay = random.uniform(0.5, 3.0)
    time.sleep(delay)
    logger.warning("Slow response detected",
                   extra={'extra_data': {'endpoint': '/slow', 'duration_seconds': round(delay, 2)}})
    return jsonify({'status': 'ok', 'delay': round(delay, 2)})

if __name__ == '__main__':
    logger.info("Starting sample application")
    app.run(host='0.0.0.0', port=5000)
```

Add it to your `docker-compose.yml` as a service.

### Step 3: Configure the Fluentd Logging Driver

Create a Fluentd configuration that receives logs from Docker and forwards
them to Elasticsearch.

```xml
<!-- fluentd/conf/fluent.conf -->
<source>
  @type forward
  port 24224
  bind 0.0.0.0
</source>

<match docker.**>
  @type elasticsearch
  host elasticsearch
  port 9200
  logstash_format true
  logstash_prefix docker-logs
  <buffer>
    flush_interval 5s
  </buffer>
</match>
```

Add Fluentd to your `docker-compose.yml`:

```yaml
  fluentd:
    build: ./fluentd
    ports:
      - "24224:24224"
      - "24224:24224/udp"
    depends_on:
      elasticsearch:
        condition: service_healthy
```

Create a `fluentd/Dockerfile`:

```dockerfile
FROM fluent/fluentd:v1.16
USER root
RUN gem install elasticsearch -v 8.2.0 \
    && gem install fluent-plugin-elasticsearch
COPY conf/fluent.conf /fluentd/etc/
USER fluent
```

### Step 4: Configure Docker to Use the Fluentd Driver

Update your sample application service in `docker-compose.yml` to use the
Fluentd logging driver:

```yaml
  sample-app:
    build: ./sample-app
    ports:
      - "5000:5000"
    logging:
      driver: fluentd
      options:
        fluentd-address: "localhost:24224"
        tag: "docker.{{.Name}}"
    depends_on:
      - fluentd
```

### Step 5: Generate Traffic and Verify

```bash
# Start everything
docker compose up -d

# Wait for services to be healthy
docker compose ps

# Generate traffic
for i in $(seq 1 20); do
  curl -s http://localhost:5000/ > /dev/null
  curl -s http://localhost:5000/slow > /dev/null
  curl -s http://localhost:5000/error > /dev/null
done

# Wait a few seconds for logs to be indexed
sleep 10
```

**For EFK (Elasticsearch + Kibana):**

```bash
# Check if logs arrived in Elasticsearch
curl -s "http://localhost:9200/docker-logs-*/_count" | jq .

# Search for ERROR logs
curl -s "http://localhost:9200/docker-logs-*/_search?q=level:ERROR&pretty" | jq '.hits.hits[]._source'

# Open Kibana at http://localhost:5601
# 1. Go to Management > Stack Management > Index Patterns
# 2. Create index pattern: docker-logs-*
# 3. Go to Discover and search for logs
```

**For Loki + Grafana:**

If you chose the Loki option, you will need a Docker Loki driver plugin:

```bash
docker plugin install grafana/loki-docker-driver:latest --alias loki
```

Then configure the logging driver:

```yaml
  sample-app:
    logging:
      driver: loki
      options:
        loki-url: "http://localhost:3100/loki/api/v1/push"
```

### Step 6: Demonstrate Filtering

Prove that you can filter logs by:

1. **Service name** -- show logs only from `sample-app`
2. **Log level** -- show only ERROR-level logs
3. **Time range** -- show logs from the last 5 minutes
4. **Full-text search** -- find all logs containing "Simulated error"

Document the queries you used for each filter.

---

## Success Criteria

- [ ] The logging infrastructure (Elasticsearch + Kibana or Loki + Grafana)
      is running and healthy
- [ ] The sample application is configured to use the Fluentd (or Loki)
      Docker logging driver
- [ ] Logs from the sample application appear in the centralized logging
      system within 30 seconds
- [ ] You can filter logs by service name, log level, time range, and
      full-text search
- [ ] You can explain the difference between the `json-file` driver (local
      storage) and the `fluentd` driver (forwarding) and when to use each
- [ ] You understand the trade-off: forwarding logs adds network overhead
      and a dependency on the logging infrastructure being available

## Common Mistakes to Avoid

- Not waiting for Elasticsearch to be fully healthy before starting Fluentd
- Using `fluentd-address: localhost:24224` when Fluentd is in a different
  container (use the service name, not localhost, in Docker Compose)
- Forgetting that when you use the `fluentd` driver, `docker logs` no longer
  shows container output (logs are forwarded, not stored locally)
- Not setting a log rotation policy even when forwarding (the forwarding
  buffer can grow if the destination is unreachable)

## What You Should Understand After This Exercise

Docker logging drivers are the bridge between container-local logging and
centralized log management. The `fluentd` driver forwards logs to a collector
which indexes them in a searchable store. This is the standard production
architecture. The key trade-off is complexity and dependency: you now depend
on the logging infrastructure being available, and you lose the simplicity
of `docker logs`. For most production systems, this trade-off is worth it.
