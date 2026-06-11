# Exercise 03: Implement Log Levels and Filtering for a Multi-Service App

**Type:** Independent
**Time:** 45 minutes
**Difficulty:** Medium

## Objective

Build a multi-service application using Docker Compose where each service
emits structured JSON logs with appropriate log levels. Implement a log
level filtering mechanism so that each environment (development, staging,
production) sees only the logs it needs.

## Background

In production you run many services simultaneously. Not every log line from
every service is equally important. DEBUG-level logs are invaluable during
development but create noise (and cost) in production. You need a strategy
to control log verbosity per service and per environment.

---

## Instructions

### Step 1: Create the Project Structure

```
multi-service-logging/
  docker-compose.yml
  api-gateway/
    Dockerfile
    app.py
    requirements.txt
  order-service/
    Dockerfile
    app.py
    requirements.txt
  inventory-service/
    Dockerfile
    app.py
    requirements.txt
```

### Step 2: Implement the Shared Logging Setup

Create a Python module (or inline it in each service) that provides a
`setup_logger()` function. This function must:

1. Create a logger that outputs JSON to stdout
2. Accept a `service_name` parameter
3. Accept a `log_level` parameter (default: `INFO`)
4. Include these fields in every log entry:
   - `timestamp` (ISO 8601)
   - `level`
   - `message`
   - `service` (the service name)
   - `logger` (the Python logger name)
5. Support a `context` dictionary for extra fields (e.g., `order_id`,
   `product_sku`)

<details>
<summary>Hint</summary>

Use `os.environ.get('LOG_LEVEL', 'INFO')` to read the log level from an
environment variable. This lets you change verbosity without modifying code.

```python
import os
import json
import logging
import sys
from datetime import datetime, timezone

def setup_logger(service_name: str, log_level: str = None) -> logging.Logger:
    level = log_level or os.environ.get('LOG_LEVEL', 'INFO')
    logger = logging.getLogger(service_name)
    logger.setLevel(getattr(logging, level.upper(), logging.INFO))
    # ... add handler with JSON formatter
    return logger
```

</details>

### Step 3: Implement the Services

**api-gateway** (port 8000): Accepts HTTP requests and calls the other two
services. Log at these levels:
- DEBUG: incoming headers, upstream request details
- INFO: request received, response sent (with status code and duration)
- WARNING: upstream service returned 4xx
- ERROR: upstream service returned 5xx or was unreachable

**order-service** (port 8001): Manages orders. Log at these levels:
- DEBUG: database query details, validation steps
- INFO: order created, order status changed
- WARNING: duplicate order attempt, stock low
- ERROR: payment processing failed, database connection lost

**inventory-service** (port 8002): Manages inventory. Log at these levels:
- DEBUG: cache hit/miss, item lookup details
- INFO: stock updated, item reserved
- WARNING: stock below threshold, approaching capacity
- ERROR: inventory inconsistency detected

Each service should be a simple Flask app with 2-3 endpoints. The exact
business logic does not matter -- what matters is that each service emits
logs at all five levels (DEBUG, INFO, WARNING, ERROR, CRITICAL) under
realistic conditions.

<details>
<summary>Hint</summary>

You can simulate different conditions by adding query parameters:

```python
@app.route('/orders', methods=['POST'])
def create_order():
    logger.debug("Validating order payload", extra={'payload': request.get_json()})
    # ...
    if quantity > stock:
        logger.warning("Insufficient stock", extra={'requested': quantity, 'available': stock})
        return jsonify({"error": "Insufficient stock"}), 409
    logger.info("Order created", extra={'order_id': order_id, 'total': total})
    return jsonify({"order_id": order_id}), 201
```

</details>

### Step 4: Configure Docker Compose

Create a `docker-compose.yml` that defines all three services. Each service
must:

- Use the `LOG_LEVEL` environment variable (default: `INFO`)
- Set log rotation: `max-size: 5m`, `max-file: 3`
- Use the `json-file` logging driver
- Expose its port for testing

Create three compose override files (or profiles) for different environments:

**docker-compose.dev.yml** -- all services at `DEBUG` level
**docker-compose.staging.yml** -- all services at `INFO` level
**docker-compose.prod.yml** -- all services at `WARNING` level

Run with:
```bash
# Development (all DEBUG logs)
docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d

# Staging (INFO and above)
docker compose -f docker-compose.yml -f docker-compose.staging.yml up -d

# Production (WARNING and above)
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

### Step 5: Test and Verify

After starting the services, generate traffic and verify log output:

```bash
# Generate traffic
curl http://localhost:8000/order -X POST \
  -H "Content-Type: application/json" \
  -d '{"product": "widget", "quantity": 5}'
curl http://localhost:8000/inventory?product=widget

# Check logs per service
docker compose logs api-gateway
docker compose logs order-service
docker compose logs inventory-service

# Verify filtering works
docker compose logs api-gateway 2>&1 | jq 'select(.level == "DEBUG")'
# In production mode, this should return nothing
docker compose logs api-gateway 2>&1 | jq 'select(.level == "WARNING")'
# In production mode, this should return results
```

---

## Success Criteria

- [ ] All three services emit valid JSON logs to stdout
- [ ] Each service uses appropriate log levels for its messages
- [ ] The `LOG_LEVEL` environment variable controls which logs are emitted
- [ ] In `dev` mode, DEBUG logs are visible for all services
- [ ] In `prod` mode, only WARNING and above are visible
- [ ] Log rotation is configured for all containers (`max-size: 5m`,
      `max-file: 3`)
- [ ] You can filter logs by service name and by level using `jq`
- [ ] Each log entry includes the `service` field identifying its source

## What You Should Understand After This Exercise

Log levels are not just labels -- they are a cost and signal control
mechanism. In development, you want maximum verbosity. In production, you
want only the signals that require human attention. Environment variables
let you change this without rebuilding images. Structured logs make it
possible to filter by level, by service, or by any other field using
standard tools like `jq`.
