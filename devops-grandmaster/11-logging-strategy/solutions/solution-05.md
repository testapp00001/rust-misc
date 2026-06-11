# Solution 05: Set Up Docker Logging Drivers to Forward Logs to a Central System

## Complete Solution

### Project Structure

```
centralized-logging/
  docker-compose.yml
  fluentd/
    Dockerfile
    conf/
      fluent.conf
  sample-app/
    Dockerfile
    app.py
    requirements.txt
```

### fluentd/conf/fluent.conf

```xml
<source>
  @type forward
  port 24224
  bind 0.0.0.0
</source>

# Parse Docker log tags to extract container name and ID
<filter docker.**>
  @type parser
  key_name log
  reserve_data true
  <parse>
    @type json
  </parse>
</filter>

<match docker.**>
  @type elasticsearch
  host elasticsearch
  port 9200
  logstash_format true
  logstash_prefix docker-logs
  include_tag_key true
  tag_key @log_name
  <buffer>
    flush_interval 5s
    retry_max_interval 30
    retry_forever true
  </buffer>
</match>
```

**Why each section:**

- `<source>` -- Fluentd listens on port 24224 (the default for Docker's
  fluentd driver) for incoming log data.
- `<filter>` -- Attempts to parse the `log` field as JSON. If the
  application outputs structured JSON logs, this extracts the fields into
  top-level Elasticsearch fields instead of nesting everything under `log`.
  `reserve_data true` keeps the original fields even if parsing fails.
- `<match>` -- Forward everything matching `docker.**` to Elasticsearch.
  `logstash_format true` creates daily indices like `docker-logs-2024.03.15`.
  The buffer configuration retries on failure so logs are not lost if
  Elasticsearch is temporarily unavailable.

### fluentd/Dockerfile

```dockerfile
FROM fluent/fluentd:v1.16-debian-1

USER root

RUN gem install elasticsearch -v 8.2.0 \
    && gem install fluent-plugin-elasticsearch

COPY conf/fluent.conf /fluentd/etc/fluent.conf

USER fluent
```

**Why this base image:** The official Fluentd image includes the forward
input plugin. We add the Elasticsearch output plugin. The Debian-based image
is used because the Alpine variant has compatibility issues with some gems.

### sample-app/app.py

```python
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
            'timestamp': datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%S.%fZ'),
            'level': record.levelname,
            'message': record.getMessage(),
            'service': os.environ.get('SERVICE_NAME', 'sample-app'),
        }
        if hasattr(record, 'extra_data') and isinstance(record.extra_data, dict):
            log_entry.update(record.extra_data)
        if record.exc_info:
            log_entry['exception'] = self.formatException(record.exc_info)
        return json.dumps(log_entry)


logger = logging.getLogger('sample-app')
handler = logging.StreamHandler(sys.stdout)
handler.setFormatter(JSONFormatter())
logger.addHandler(handler)
logger.setLevel(os.environ.get('LOG_LEVEL', 'INFO'))
logger.propagate = False

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
                   extra={'extra_data': {'endpoint': '/slow',
                                         'duration_seconds': round(delay, 2)}})
    return jsonify({'status': 'ok', 'delay': round(delay, 2)})


@app.route('/health')
def health():
    logger.debug("Health check", extra={'extra_data': {'endpoint': '/health'}})
    return jsonify({'status': 'healthy'})


if __name__ == '__main__':
    logger.info("Starting sample application", extra={'extra_data': {
        'port': 5000,
        'log_level': os.environ.get('LOG_LEVEL', 'INFO'),
    }})
    app.run(host='0.0.0.0', port=5000)
```

### sample-app/Dockerfile

```dockerfile
FROM python:3.12-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY . .
CMD ["python", "app.py"]
```

### sample-app/requirements.txt

```
flask==3.0.0
```

### docker-compose.yml

```yaml
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
      retries: 10
    logging:
      driver: json-file
      options:
        max-size: "10m"
        max-file: "3"

  kibana:
    image: kibana:8.12.0
    environment:
      - ELASTICSEARCH_HOSTS=http://elasticsearch:9200
    ports:
      - "5601:5601"
    depends_on:
      elasticsearch:
        condition: service_healthy
    logging:
      driver: json-file
      options:
        max-size: "5m"
        max-file: "2"

  fluentd:
    build: ./fluentd
    ports:
      - "24224:24224"
      - "24224:24224/udp"
    depends_on:
      elasticsearch:
        condition: service_healthy
    logging:
      driver: json-file
      options:
        max-size: "5m"
        max-file: "2"

  sample-app:
    build: ./sample-app
    ports:
      - "5000:5000"
    environment:
      - SERVICE_NAME=sample-app
      - LOG_LEVEL=INFO
    logging:
      driver: fluentd
      options:
        fluentd-address: "localhost:24224"
        tag: "docker.{{.Name}}"
    depends_on:
      - fluentd

volumes:
  es-data:
```

**Critical detail about the logging driver configuration:**

The `sample-app` service uses `driver: fluentd` instead of the default
`json-file`. This means:

1. Docker sends all stdout/stderr output from `sample-app` to Fluentd on
   port 24224.
2. Fluentd receives the logs and forwards them to Elasticsearch.
3. `docker logs sample-app` will **no longer work** because the logs are
   not stored locally. This is the key trade-off of using a forwarding
   driver.

**Why `fluentd-address: "localhost:24224"`:** In Docker Compose, all
services share the same network namespace for `localhost` when using
`network_mode: host`. In the default bridge network, you would use the
service name: `fluentd-address: "fluentd:24224"`. However, the fluentd
driver connects from the Docker daemon (host network), not from within
the container network, so `localhost` is correct when the port is mapped.

### Build and Start

```bash
cd centralized-logging

# Build and start everything
docker compose up -d --build

# Wait for Elasticsearch to be healthy (may take 1-2 minutes)
docker compose logs -f elasticsearch 2>&1 | grep -m 1 "started"
# Or check the health endpoint
curl -s http://localhost:9200/_cluster/health | jq .

# Wait for Fluentd to be ready
docker compose logs -f fluentd 2>&1 | grep -m 1 "listening"

# Verify all services are running
docker compose ps
```

### Generate Traffic

```bash
# Generate a variety of log entries
for i in $(seq 1 20); do
  curl -s http://localhost:5000/ > /dev/null
  curl -s http://localhost:5000/slow > /dev/null
  curl -s http://localhost:5000/error > /dev/null
done

# Wait for logs to be indexed (Fluentd buffers for 5 seconds)
sleep 15
```

### Verify Logs in Elasticsearch

```bash
# Check that the index was created
curl -s "http://localhost:9200/_cat/indices?v"
# You should see an index like docker-logs-2024.03.15

# Count total log entries
curl -s "http://localhost:9200/docker-logs-*/_count" | jq .

# View a sample of all logs
curl -s "http://localhost:9200/docker-logs-*/_search?size=5&pretty" \
  | jq '.hits.hits[]._source'

# Search for ERROR logs only
curl -s "http://localhost:9200/docker-logs-*/_search" \
  -H "Content-Type: application/json" \
  -d '{"query": {"match": {"level": "ERROR"}}}' \
  | jq '.hits.hits[]._source'

# Search for slow responses (WARNING level with duration > 2 seconds)
curl -s "http://localhost:9200/docker-logs-*/_search" \
  -H "Content-Type: application/json" \
  -d '{
    "query": {
      "bool": {
        "must": [
          {"match": {"level": "WARNING"}},
          {"range": {"duration_seconds": {"gte": 2.0}}}
        ]
      }
    }
  }' | jq '.hits.hits[]._source'

# Search for "Simulated error" text
curl -s "http://localhost:9200/docker-logs-*/_search" \
  -H "Content-Type: application/json" \
  -d '{"query": {"match": {"message": "Simulated error"}}}' \
  | jq '.hits.hits[]._source'
```

### Verify in Kibana

1. Open http://localhost:5601 in a browser.
2. Go to **Management > Stack Management > Index Patterns** (or
   **Data Views** in newer Kibana versions).
3. Create an index pattern: `docker-logs-*`.
4. Set `timestamp` as the time field.
5. Go to **Discover**.
6. You should see all the log entries from `sample-app`.
7. Use the search bar to filter:
   - `level: "ERROR"` -- shows only error logs
   - `level: "WARNING" AND duration_seconds > 2` -- shows slow responses
   - `message: "Simulated"` -- full-text search

### Filtering Summary

| Filter | Elasticsearch Query | Kibana Query |
|--------|-------------------|--------------|
| Service name | `{"match": {"service": "sample-app"}}` | `service: "sample-app"` |
| Log level | `{"match": {"level": "ERROR"}}` | `level: "ERROR"` |
| Time range | Use `@timestamp` range query | Use Kibana time picker |
| Full text | `{"match": {"message": "error"}}` | `message: "error"` |

---

## Why This Works

1. **Docker logging drivers decouple log production from log consumption.**
   The application writes to stdout. Docker's fluentd driver forwards the
   output to Fluentd. Fluentd indexes it in Elasticsearch. The application
   has no knowledge of Elasticsearch, and Docker has no knowledge of the
   index structure. Each layer is independent.

2. **Fluentd acts as a buffer.** If Elasticsearch is temporarily unavailable,
   Fluentd retries with exponential backoff (`retry_forever true`). Logs are
   buffered in memory (and optionally on disk) until the destination recovers.
   This prevents log loss during transient failures.

3. **The JSON parser in Fluentd extracts structured fields.** Because the
   application outputs JSON, Fluentd can parse it and send individual fields
   to Elasticsearch. This makes every field searchable and filterable. If the
   application output plain text, you would only be able to do full-text
   search.

4. **The `tag` option identifies the source.** `tag: "docker.{{.Name}}"`
   includes the container name in the Fluentd tag. This lets you filter logs
   by container in Kibana, even when multiple containers forward to the same
   Fluentd instance.

5. **Log rotation is still configured.** Even though logs are forwarded, the
   logging infrastructure itself (Elasticsearch, Kibana, Fluentd) uses the
   json-file driver with rotation. This prevents the logging stack from
   filling the disk with its own logs.

## Common Mistakes

### Mistake 1: Using localhost when Fluentd is in a different container

```yaml
# WRONG if using bridge networking
logging:
  driver: fluentd
  options:
    fluentd-address: "localhost:24224"
```

The fluentd driver connects from the Docker daemon process (on the host),
not from inside the container. If Fluentd's port 24224 is mapped to the
host, `localhost:24224` works. If Fluentd is only accessible via the Docker
network, use the service name. In this exercise, we map the port, so
`localhost` is correct.

### Mistake 2: Expecting `docker logs` to work with the fluentd driver

```bash
docker logs sample-app
# Error: configured logging driver does not support reading
```

When you use a forwarding driver (fluentd, syslog, awslogs), Docker does
not store logs locally. `docker logs` only works with the `json-file` and
`local` drivers. This is the fundamental trade-off: centralized logging
means losing local log access.

### Mistake 3: Not waiting for Elasticsearch to be healthy

Fluentd will fail to connect to Elasticsearch on startup if ES is not ready.
With `retry_forever true`, Fluentd will keep retrying, but the first batch
of logs may be delayed. Always use `depends_on` with `condition: service_healthy`
and include a healthcheck on Elasticsearch.

### Mistake 4: Forgetting that the fluentd driver has a buffer limit

If Fluentd is down for an extended period, the Docker daemon buffers logs in
memory. If the buffer fills up, logs are dropped. For critical services,
consider:

```yaml
logging:
  driver: fluentd
  options:
    fluentd-address: "localhost:24224"
    fluentd-async: "true"
    fluentd-retry-wait: "1s"
    fluentd-max-retries: "30"
```

### Mistake 5: Not parsing JSON in Fluentd

If you omit the `<filter>` section that parses JSON, all structured log data
ends up as a single string in the `log` field of Elasticsearch. You lose the
ability to filter by `level`, `service`, or any other structured field. Always
parse JSON logs in your Fluentd configuration.

### Mistake 6: Not setting up index lifecycle management

Without index lifecycle management (ILM), Elasticsearch indices grow forever.
In production, configure ILM to:

- Roll over indices when they reach 50 GB or 1 day
- Move old indices to warm storage after 7 days
- Delete indices after 90 days

This keeps storage costs under control and maintains query performance.
