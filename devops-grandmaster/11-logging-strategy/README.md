# Module 11: Logging Strategy

## The Problem: Where Did My Logs Go?

Your container is running. Something goes wrong. You need to see the logs. But...

```bash
# You restart the container
docker restart my-app

# The logs are gone.
docker logs my-app
# Only shows logs from AFTER restart
```

Container logs are ephemeral by default. When a container dies, its logs die with it.

## The Naive Way: Log to Files Inside the Container

```python
# BAD: Logging to a file inside the container
import logging
logging.basicConfig(filename='/app/logs/app.log')
```

**Why this fails:**
- Container filesystem is ephemeral (Module 09)
- Log files grow until disk is full
- Can't search across multiple containers
- Can't set up alerts on log patterns
- Rotating logs inside a container is painful

## The Right Way: Log to stdout/stderr

```python
# GOOD: Log to stdout
import logging
import sys

logging.basicConfig(
    stream=sys.stdout,
    level=logging.INFO,
    format='%(asctime)s %(levelname)s %(message)s'
)

logger = logging.getLogger(__name__)
logger.info("Application started")
logger.error("Something went wrong", exc_info=True)
```

```javascript
// Node.js: console.log goes to stdout
console.log('Application started');
console.error('Something went wrong');
```

```go
// Go: log package writes to stderr by default
log.Println("Application started")
log.Printf("Request processed in %v", duration)
```

**Docker captures stdout/stderr automatically:**
```bash
# View logs
docker logs my-app

# Follow logs in real-time
docker logs -f my-app

# Last 100 lines
docker logs --tail 100 my-app

# With timestamps
docker logs -t my-app

# Since specific time
docker logs --since 2024-01-01T00:00:00 my-app
```

## Structured Logging

Plain text logs are hard to parse. Use structured logging (JSON):

```python
import json
import logging
import sys

class JSONFormatter(logging.Formatter):
    def format(self, record):
        log_entry = {
            'timestamp': self.formatTime(record),
            'level': record.levelname,
            'message': record.getMessage(),
            'module': record.module,
            'function': record.funcName,
            'line': record.lineno,
        }
        if record.exc_info:
            log_entry['exception'] = self.formatException(record.exc_info)
        return json.dumps(log_entry)

handler = logging.StreamHandler(sys.stdout)
handler.setFormatter(JSONFormatter())
logger = logging.getLogger(__name__)
logger.addHandler(handler)
logger.setLevel(logging.INFO)

logger.info("User logged in", extra={'user_id': 123, 'ip': '10.0.0.1'})
```

Output:
```json
{"timestamp": "2024-01-15 10:30:45", "level": "INFO", "message": "User logged in", "module": "app", "function": "login", "line": 42, "user_id": 123, "ip": "10.0.0.1"}
```

## Docker Logging Drivers

Docker can send logs to different destinations:

```bash
# Check current logging driver
docker info --format '{{.LoggingDriver}}'
```

### Configure per container:
```bash
# Send to json-file (default)
docker run --log-driver=json-file my-app

# Send to syslog
docker run --log-driver=syslog --log-opt syslog-address=tcp://logserver:514 my-app

# Send to fluentd
docker run --log-driver=fluentd --log-opt fluentd-address=localhost:24224 my-app

# Send to AWS CloudWatch
docker run --log-driver=awslogs --log-opt awslogs-group=my-app my-app

# Disable logging (for noisy containers)
docker run --log-driver=none my-app
```

### Configure globally (/etc/docker/daemon.json):
```json
{
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "10m",
    "max-file": "3"
  }
}
```

## Log Rotation

Without rotation, logs fill disk:

```bash
# Set max size and number of log files
docker run \
  --log-opt max-size=10m \
  --log-opt max-file=3 \
  my-app

# This keeps 3 files of 10MB each = 30MB max
```

## Production Logging Architecture

```
┌──────────┐     ┌──────────┐     ┌──────────┐
│  App A   │     │  App B   │     │  App C   │
│ (stdout) │     │ (stdout) │     │ (stdout) │
└────┬─────┘     └────┬─────┘     └────┬─────┘
     │                │                │
     └────────────────┼────────────────┘
                      │
              ┌───────▼────────┐
              │  Log Collector  │
              │ (Fluentd/Filebeat)│
              └───────┬────────┘
                      │
              ┌───────▼────────┐
              │  Log Storage    │
              │ (Elasticsearch/ │
              │  Loki/CloudWatch)│
              └───────┬────────┘
                      │
              ┌───────▼────────┐
              │  Log Viewer     │
              │ (Kibana/Grafana)│
              └────────────────┘
```

## Hands-On Exercise

### Exercise 1: Structured Logging in Python

Create `app.py`:
```python
from flask import Flask, request, jsonify
import logging
import json
import sys
import uuid
from datetime import datetime

class JSONFormatter(logging.Formatter):
    def format(self, record):
        log_entry = {
            'timestamp': datetime.utcnow().isoformat(),
            'level': record.levelname,
            'message': record.getMessage(),
            'module': record.module,
        }
        if hasattr(record, 'extra_data'):
            log_entry.update(record.extra_data)
        return json.dumps(log_entry)

logger = logging.getLogger('app')
handler = logging.StreamHandler(sys.stdout)
handler.setFormatter(JSONFormatter())
logger.addHandler(handler)
logger.setLevel(logging.INFO)

app = Flask(__name__)

@app.before_request
def before_request():
    request.request_id = str(uuid.uuid4())
    logger.info('Request started', extra={'extra_data': {
        'request_id': request.request_id,
        'method': request.method,
        'path': request.path,
        'ip': request.remote_addr
    }})

@app.route('/')
def index():
    logger.info('Home page accessed', extra={'extra_data': {
        'request_id': request.request_id
    }})
    return jsonify({'status': 'ok'})

@app.route('/error')
def error():
    try:
        raise ValueError('Something broke!')
    except Exception:
        logger.error('Error occurred', exc_info=True, extra={'extra_data': {
            'request_id': request.request_id
        }})
        return jsonify({'error': 'Internal error'}), 500

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

Dockerfile:
```dockerfile
FROM python:3.11-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install -r requirements.txt
COPY . .
CMD ["python", "app.py"]
```

requirements.txt:
```
flask==3.0.0
```

Build and test:
```bash
docker build -t logging-demo .
docker run -d --name logging-demo -p 5000:5000 logging-demo

# Generate some logs
curl http://localhost:5000/
curl http://localhost:5000/error

# View structured logs
docker logs logging-demo

# Filter JSON logs
docker logs logging-demo 2>&1 | jq '.level'
docker logs logging-demo 2>&1 | jq 'select(.level == "ERROR")'
```

### Exercise 2: Log Rotation
```bash
# Run with log rotation
docker run -d --name rotated-logs \
  --log-opt max-size=1m \
  --log-opt max-file=3 \
  -p 5001:5000 logging-demo

# Generate lots of logs
for i in $(seq 1 1000); do curl http://localhost:5001/; done

# Check log file sizes
docker inspect rotated-logs --format='{{.LogPath}}'
ls -lh $(docker inspect rotated-logs --format='{{.LogPath}}')
```

## Limitation: Logs Tell You WHAT Happened, Not IF It's Working

You can see logs, but how do you know if your app is actually healthy **right now**?

**Next problem:** How do you detect when a container is unhealthy before users notice?

→ **Next module:** [12-health-checks](../12-health-checks/) — Knowing if your app is actually alive

## Checklist

- [ ] I log to stdout/stderr, not to files
- [ ] I use structured logging (JSON)
- [ ] I configure log rotation (max-size, max-file)
- [ ] I understand Docker logging drivers
- [ ] I can search and filter logs
- [ ] I know the production logging architecture (collector → storage → viewer)
