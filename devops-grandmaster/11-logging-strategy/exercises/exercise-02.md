# Exercise 02: Convert Unstructured Prints to Structured JSON Logging

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Take a working Python application that uses `print()` statements for all
output and convert it to use structured JSON logging with proper log levels.
By the end, every log line emitted by the application should be valid JSON
that can be parsed by `jq`.

## Background

The application below is a simple HTTP API built with Flask. It works, but
its logging is a mess: it uses `print()` for everything, timestamps are
inconsistent, and there is no way to filter by severity. Your job is to fix it.

---

## Starting Code

Create a file called `app.py` with the following content:

```python
from flask import Flask, request, jsonify
from datetime import datetime
import time
import random
import uuid

app = Flask(__name__)

# Simulated database
users_db = {
    1: {"name": "Alice", "email": "alice@example.com"},
    2: {"name": "Bob", "email": "bob@example.com"},
    3: {"name": "Charlie", "email": "charlie@example.com"},
}

@app.before_request
def before_request():
    request.start_time = time.time()
    request.request_id = str(uuid.uuid4())[:8]
    print(f"[{datetime.now()}] --> {request.method} {request.path} from {request.remote_addr}")

@app.after_request
def after_request(response):
    duration = time.time() - request.start_time
    print(f"[{datetime.now()}] <-- {request.method} {request.path} {response.status_code} in {duration:.3f}s")
    return response

@app.route('/users', methods=['GET'])
def list_users():
    print("Fetching all users")
    return jsonify(list(users_db.values()))

@app.route('/users/<int:user_id>', methods=['GET'])
def get_user(user_id):
    print(f"Looking up user {user_id}")
    user = users_db.get(user_id)
    if user is None:
        print(f"WARNING: User {user_id} not found")
        return jsonify({"error": "User not found"}), 404
    return jsonify(user)

@app.route('/users', methods=['POST'])
def create_user():
    data = request.get_json()
    if not data or 'name' not in data or 'email' not in data:
        print(f"ERROR: Invalid request body: {data}")
        return jsonify({"error": "name and email required"}), 400

    new_id = max(users_db.keys()) + 1
    users_db[new_id] = {"name": data['name'], "email": data['email']}
    print(f"Created user {new_id}: {data['name']} ({data['email']})")
    return jsonify({"id": new_id, **users_db[new_id]}), 201

@app.route('/health')
def health():
    # Sometimes simulate a failure
    if random.random() < 0.1:
        print("CRITICAL: Health check failed! Database unreachable!")
        return jsonify({"status": "unhealthy"}), 503
    print("Health check passed")
    return jsonify({"status": "healthy"})

if __name__ == '__main__':
    print("Starting user service on port 5000")
    app.run(host='0.0.0.0', port=5000)
```

Create a `requirements.txt`:
```
flask==3.0.0
```

Create a `Dockerfile`:
```dockerfile
FROM python:3.12-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY . .
CMD ["python", "app.py"]
```

---

## Instructions

### Step 1: Add a JSON Formatter

Create a custom `logging.Formatter` subclass called `JSONFormatter` that
outputs each log record as a single line of valid JSON. Every log entry must
include at least these fields:

- `timestamp` -- ISO 8601 format (e.g., `2024-03-15T10:30:45.123Z`)
- `level` -- the log level name (DEBUG, INFO, WARNING, ERROR, CRITICAL)
- `message` -- the log message
- `service` -- a fixed string identifying the service (e.g., `"user-service"`)
- `request_id` -- the short UUID generated in `before_request`

<details>
<summary>Hint</summary>

Use `logging.StreamHandler(sys.stdout)` to send all logs to stdout. The
formatter's `format()` method receives a `logging.LogRecord` and must return
a string. Use `json.dumps()` to serialize a dictionary.

To include `request_id`, you can use a `logging.LoggerAdapter`, a
`logging.Filter`, or pass it via the `extra` parameter on each log call.

</details>

### Step 2: Replace Every `print()` with a Logger Call

Replace each `print()` call with the appropriate log level:

| Original pattern | Correct level |
|------------------|---------------|
| `print("Fetching all users")` | `logger.info(...)` |
| `print(f"WARNING: User {user_id} not found")` | `logger.warning(...)` |
| `print(f"ERROR: Invalid request body: {data}")` | `logger.error(...)` |
| `print("CRITICAL: Health check failed! ...")` | `logger.critical(...)` |
| `print("Starting user service on port 5000")` | `logger.info(...)` |

For request/response logging in `before_request` and `after_request`, use
`logger.info()` and include the request metadata as structured fields, not
embedded in the message string.

<details>
<summary>Hint</summary>

Instead of putting data in the message string like this:
```python
logger.info(f"--> {request.method} {request.path}")
```

Pass structured data as extra fields:
```python
logger.info("Request started", extra={
    'method': request.method,
    'path': request.path,
    'client_ip': request.remote_addr
})
```

Your JSON formatter should merge these extra fields into the JSON output.

</details>

### Step 3: Add a Correlation ID

The `request_id` should appear in every log line generated during a single
HTTP request. There are several ways to achieve this:

- Pass `extra={'request_id': request.request_id}` on every log call
- Use a `logging.Filter` that reads the request ID from Flask's `g` or
  `request` object
- Use a `logging.LoggerAdapter` with a default `extra` dict

Choose one approach and implement it.

### Step 4: Build, Run, and Verify

```bash
docker build -t user-service .
docker run -d --name user-service -p 5000:5000 user-service

# Generate traffic
curl http://localhost:5000/users
curl http://localhost:5000/users/1
curl http://localhost:5000/users/999
curl -X POST http://localhost:5000/users -H "Content-Type: application/json" \
  -d '{"name": "Dave", "email": "dave@example.com"}'
curl http://localhost:5000/health
```

Verify that every line of output is valid JSON:

```bash
docker logs user-service 2>&1 | python -m json.tool --no-ensure-ascii
```

Verify you can filter by level:

```bash
docker logs user-service 2>&1 | jq 'select(.level == "ERROR")'
docker logs user-service 2>&1 | jq 'select(.level == "WARNING")'
```

Verify you can trace a request:

```bash
# Pick a request_id from one log line, then filter
docker logs user-service 2>&1 | jq 'select(.request_id == "abcd1234")'
```

---

## Success Criteria

- [ ] Every line of `docker logs` output is valid, parseable JSON
- [ ] Each log entry contains: timestamp, level, message, service, request_id
- [ ] Log levels are used correctly (info for normal operations, warning for
      expected failures like 404, error for unexpected failures, critical for
      system-level problems)
- [ ] Request metadata (method, path, status code, duration) appears as
      structured fields, not embedded in the message string
- [ ] You can filter logs by level using `jq`
- [ ] You can trace all log lines for a single request using the request_id

## Common Mistakes to Avoid

- Embedding structured data inside the message string instead of as separate
  JSON fields
- Using `print()` alongside `logging` -- pick one and be consistent
- Forgetting to send logs to stdout (they must go to `sys.stdout`, not a file)
- Not handling the case where `extra` fields might conflict with built-in
  `LogRecord` attributes
