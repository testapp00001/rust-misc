# Cheatsheet: Logging Strategy

## Key Principles

1. **Log to stdout/stderr** — not to files
2. **Use structured logging** — JSON format
3. **Include context** — request ID, user ID, timestamp
4. **Set log levels** — DEBUG, INFO, WARN, ERROR
5. **Configure rotation** — max-size, max-file

## Docker Logging
```bash
# View logs
docker logs my-app
docker logs -f my-app            # Follow
docker logs --tail 100 my-app    # Last 100 lines
docker logs -t my-app            # With timestamps
docker logs --since 1h my-app    # Last hour

# Log rotation
docker run --log-opt max-size=10m --log-opt max-file=3 my-app
```

## Structured Logging (Python)
```python
import json, logging, sys

class JSONFormatter(logging.Formatter):
    def format(self, record):
        return json.dumps({
            'timestamp': self.formatTime(record),
            'level': record.levelname,
            'message': record.getMessage(),
            'module': record.module,
        })

handler = logging.StreamHandler(sys.stdout)
handler.setFormatter(JSONFormatter())
logger = logging.getLogger(__name__)
logger.addHandler(handler)
```

## Docker Logging Drivers
```bash
# In docker run
docker run --log-driver=json-file my-app
docker run --log-driver=syslog --log-opt syslog-address=tcp://logserver:514 my-app
docker run --log-driver=fluentd my-app
docker run --log-driver=none my-app  # Disable logging
```

## Production Architecture
```
App (stdout) → Log Collector (Fluentd/Filebeat) → Storage (Elasticsearch/Loki) → Viewer (Kibana/Grafana)
```
