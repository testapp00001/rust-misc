# Solution 02: Convert Unstructured Prints to Structured JSON Logging

## Complete Solution

### app.py (converted)

```python
from flask import Flask, request, jsonify
import logging
import json
import sys
import time
import random
import uuid
from datetime import datetime, timezone

# --- JSON Formatter ---

class JSONFormatter(logging.Formatter):
    def format(self, record):
        log_entry = {
            'timestamp': datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%S.%fZ'),
            'level': record.levelname,
            'message': record.getMessage(),
            'service': 'user-service',
            'logger': record.name,
        }

        # Merge extra fields from the caller
        # We use a convention: if the record has 'extra_data', merge it in
        if hasattr(record, 'extra_data') and isinstance(record.extra_data, dict):
            log_entry.update(record.extra_data)

        if record.exc_info:
            log_entry['exception'] = self.formatException(record.exc_info)

        return json.dumps(log_entry)


# --- Logger Setup ---

def setup_logger():
    logger = logging.getLogger('user-service')
    logger.setLevel(logging.DEBUG)

    handler = logging.StreamHandler(sys.stdout)
    handler.setFormatter(JSONFormatter())
    logger.addHandler(handler)

    # Prevent duplicate handlers if setup_logger is called multiple times
    logger.propagate = False

    return logger


logger = setup_logger()

# --- Simulated Database ---

users_db = {
    1: {"name": "Alice", "email": "alice@example.com"},
    2: {"name": "Bob", "email": "bob@example.com"},
    3: {"name": "Charlie", "email": "charlie@example.com"},
}

# --- Flask App ---

app = Flask(__name__)

@app.before_request
def before_request():
    request.start_time = time.time()
    request.request_id = str(uuid.uuid4())[:8]
    logger.info("Request started", extra={'extra_data': {
        'request_id': request.request_id,
        'method': request.method,
        'path': request.path,
        'client_ip': request.remote_addr,
    }})

@app.after_request
def after_request(response):
    duration = time.time() - request.start_time
    logger.info("Request completed", extra={'extra_data': {
        'request_id': request.request_id,
        'method': request.method,
        'path': request.path,
        'status_code': response.status_code,
        'duration_seconds': round(duration, 4),
    }})
    return response

@app.route('/users', methods=['GET'])
def list_users():
    logger.info("Listing all users", extra={'extra_data': {
        'request_id': request.request_id,
        'user_count': len(users_db),
    }})
    return jsonify(list(users_db.values()))

@app.route('/users/<int:user_id>', methods=['GET'])
def get_user(user_id):
    logger.debug("Looking up user", extra={'extra_data': {
        'request_id': request.request_id,
        'user_id': user_id,
    }})
    user = users_db.get(user_id)
    if user is None:
        logger.warning("User not found", extra={'extra_data': {
            'request_id': request.request_id,
            'user_id': user_id,
        }})
        return jsonify({"error": "User not found"}), 404
    logger.info("User retrieved", extra={'extra_data': {
        'request_id': request.request_id,
        'user_id': user_id,
    }})
    return jsonify(user)

@app.route('/users', methods=['POST'])
def create_user():
    data = request.get_json()
    if not data or 'name' not in data or 'email' not in data:
        logger.error("Invalid request body", extra={'extra_data': {
            'request_id': request.request_id,
            'payload': str(data),
        }})
        return jsonify({"error": "name and email required"}), 400

    new_id = max(users_db.keys()) + 1
    users_db[new_id] = {"name": data['name'], "email": data['email']}
    logger.info("User created", extra={'extra_data': {
        'request_id': request.request_id,
        'user_id': new_id,
        'user_name': data['name'],
    }})
    return jsonify({"id": new_id, **users_db[new_id]}), 201

@app.route('/health')
def health():
    if random.random() < 0.1:
        logger.critical("Health check failed - database unreachable", extra={'extra_data': {
            'request_id': request.request_id,
            'check': 'database',
            'status': 'unreachable',
        }})
        return jsonify({"status": "unhealthy"}), 503
    logger.debug("Health check passed", extra={'extra_data': {
        'request_id': request.request_id,
    }})
    return jsonify({"status": "healthy"})

if __name__ == '__main__':
    logger.info("Starting user service", extra={'extra_data': {
        'port': 5000,
        'host': '0.0.0.0',
    }})
    app.run(host='0.0.0.0', port=5000)
```

### Dockerfile

```dockerfile
FROM python:3.12-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY . .
CMD ["python", "app.py"]
```

### requirements.txt

```
flask==3.0.0
```

### Build and Test

```bash
docker build -t user-service .
docker run -d --name user-service -p 5000:5000 user-service

# Generate traffic
curl http://localhost:5000/users
curl http://localhost:5000/users/1
curl http://localhost:5000/users/999
curl -X POST http://localhost:5000/users \
  -H "Content-Type: application/json" \
  -d '{"name": "Dave", "email": "dave@example.com"}'
curl http://localhost:5000/health

# Verify all output is valid JSON
docker logs user-service 2>&1 | python -m json.tool --no-ensure-ascii

# Filter by level
docker logs user-service 2>&1 | jq 'select(.level == "WARNING")'
docker logs user-service 2>&1 | jq 'select(.level == "ERROR")'

# Trace a request (pick a request_id from the output)
docker logs user-service 2>&1 | jq 'select(.request_id == "REPLACE_ME")'
```

---

## Why This Works

1. **JSONFormatter as a single point of truth.** Every log record passes
   through this formatter. It guarantees that every output line is valid
   JSON with the required fields. You never have to remember to add
   timestamps or service names -- the formatter handles it.

2. **extra_data convention.** Python's `logging` module has a quirk: if you
   pass `extra={'message': '...'}`, it will raise an error because `message`
   is a reserved attribute on `LogRecord`. By using a nested `extra_data`
   dictionary, we avoid all naming collisions.

3. **request_id in before_request.** By generating the request ID in
   `before_request` and storing it on the Flask `request` object, every
   handler has access to it. We pass it in `extra_data` on every log call,
   so you can trace all log lines for a single request.

4. **Correct log levels.** INFO for normal operations (list, get, create).
   WARNING for expected failures (404 user not found). ERROR for unexpected
   failures (bad request body). CRITICAL for system-level problems
   (database unreachable). DEBUG for diagnostic detail (lookup parameters).

5. **stdout, not a file.** `StreamHandler(sys.stdout)` ensures Docker
   captures all output via `docker logs`.

## Common Mistakes

### Mistake 1: Embedding data in the message string

```python
# BAD
logger.info(f"User {user_id} logged in from {ip}")

# GOOD
logger.info("User logged in", extra={'extra_data': {'user_id': user_id, 'ip': ip}})
```

The bad version makes the data unqueryable. You cannot write
`jq 'select(.user_id == 123)'` because user_id is not a field -- it is
part of the message string.

### Mistake 2: Forgetting to handle extra attribute conflicts

```python
# BAD -- will raise KeyError if 'message' or 'args' is in extra
logger.info("test", extra={'message': 'conflict'})

# GOOD -- nest extra data in a dedicated key
logger.info("test", extra={'extra_data': {'detail': 'no conflict'}})
```

### Mistake 3: Not setting propagate to False

```python
# Without this, the root logger may also handle the record,
# producing duplicate output
logger.propagate = False
```

### Mistake 4: Using print() alongside logging

Some developers leave `print()` calls for "quick debugging" and use logging
for "real" output. This produces a mix of JSON and plain text in `docker logs`,
which breaks `jq` parsing. Pick one method and be consistent.

### Mistake 5: Forgetting exc_info for exceptions

```python
# BAD -- no stack trace
try:
    do_something()
except Exception as e:
    logger.error(f"Failed: {e}")

# GOOD -- includes full stack trace in the 'exception' field
try:
    do_something()
except Exception:
    logger.error("Operation failed", exc_info=True)
```

`exc_info=True` tells the formatter to include the full traceback. In our
JSONFormatter, it goes into the `exception` field as a string.
