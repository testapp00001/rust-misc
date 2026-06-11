# Solution 03: Implement Log Levels and Filtering for a Multi-Service App

## Complete Solution

### Project Structure

```
multi-service-logging/
  docker-compose.yml
  docker-compose.dev.yml
  docker-compose.staging.yml
  docker-compose.prod.yml
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

### Shared Logging Setup (used by all services)

Each service contains the same logging setup, parameterized by service name
and log level. Here is the pattern (inline in each `app.py`):

```python
import os
import json
import logging
import sys
from datetime import datetime, timezone


class JSONFormatter(logging.Formatter):
    def format(self, record):
        log_entry = {
            'timestamp': datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%S.%fZ'),
            'level': record.levelname,
            'message': record.getMessage(),
            'service': record.name,
            'logger': record.name,
        }
        if hasattr(record, 'extra_data') and isinstance(record.extra_data, dict):
            log_entry.update(record.extra_data)
        if record.exc_info:
            log_entry['exception'] = self.formatException(record.exc_info)
        return json.dumps(log_entry)


def setup_logger(service_name: str) -> logging.Logger:
    level = os.environ.get('LOG_LEVEL', 'INFO').upper()
    logger = logging.getLogger(service_name)
    logger.setLevel(getattr(logging, level, logging.INFO))

    handler = logging.StreamHandler(sys.stdout)
    handler.setFormatter(JSONFormatter())
    logger.addHandler(handler)
    logger.propagate = False

    return logger
```

### api-gateway/app.py

```python
from flask import Flask, request, jsonify
import logging
import json
import sys
import os
import time
import uuid
import requests
from datetime import datetime, timezone

# (Insert JSONFormatter and setup_logger from above)

logger = setup_logger('api-gateway')

app = Flask(__name__)

ORDER_SERVICE_URL = os.environ.get('ORDER_SERVICE_URL', 'http://order-service:8001')
INVENTORY_SERVICE_URL = os.environ.get('INVENTORY_SERVICE_URL', 'http://inventory-service:8002')


@app.before_request
def before_request():
    request.start_time = time.time()
    request.request_id = str(uuid.uuid4())[:8]
    logger.debug("Incoming request details", extra={'extra_data': {
        'request_id': request.request_id,
        'method': request.method,
        'path': request.path,
        'headers': dict(request.headers),
        'client_ip': request.remote_addr,
    }})
    logger.info("Request received", extra={'extra_data': {
        'request_id': request.request_id,
        'method': request.method,
        'path': request.path,
    }})


@app.after_request
def after_request(response):
    duration = time.time() - request.start_time
    logger.info("Response sent", extra={'extra_data': {
        'request_id': request.request_id,
        'method': request.method,
        'path': request.path,
        'status_code': response.status_code,
        'duration_seconds': round(duration, 4),
    }})
    return response


@app.route('/order', methods=['POST'])
def create_order():
    data = request.get_json()
    logger.debug("Forwarding to order-service", extra={'extra_data': {
        'request_id': request.request_id,
        'upstream': 'order-service',
        'payload': data,
    }})

    try:
        resp = requests.post(f"{ORDER_SERVICE_URL}/orders", json=data, timeout=5)

        if resp.status_code >= 500:
            logger.error("Upstream server error", extra={'extra_data': {
                'request_id': request.request_id,
                'upstream': 'order-service',
                'upstream_status': resp.status_code,
            }})
            return jsonify({"error": "Order service unavailable"}), 502

        if resp.status_code >= 400:
            logger.warning("Upstream client error", extra={'extra_data': {
                'request_id': request.request_id,
                'upstream': 'order-service',
                'upstream_status': resp.status_code,
            }})
            return resp.json(), resp.status_code

        logger.info("Order created successfully", extra={'extra_data': {
            'request_id': request.request_id,
            'order_response': resp.json(),
        }})
        return jsonify(resp.json()), 201

    except requests.exceptions.ConnectionError:
        logger.error("Upstream service unreachable", extra={'extra_data': {
            'request_id': request.request_id,
            'upstream': 'order-service',
            'error_type': 'connection_error',
        }})
        return jsonify({"error": "Service unavailable"}), 503

    except requests.exceptions.Timeout:
        logger.error("Upstream service timeout", extra={'extra_data': {
            'request_id': request.request_id,
            'upstream': 'order-service',
            'error_type': 'timeout',
        }})
        return jsonify({"error": "Service timeout"}), 504


@app.route('/inventory', methods=['GET'])
def check_inventory():
    product = request.args.get('product')
    logger.debug("Forwarding to inventory-service", extra={'extra_data': {
        'request_id': request.request_id,
        'upstream': 'inventory-service',
        'product': product,
    }})

    try:
        resp = requests.get(f"{INVENTORY_SERVICE_URL}/inventory/{product}", timeout=5)

        if resp.status_code >= 500:
            logger.error("Upstream server error", extra={'extra_data': {
                'request_id': request.request_id,
                'upstream': 'inventory-service',
                'upstream_status': resp.status_code,
            }})
            return jsonify({"error": "Inventory service unavailable"}), 502

        if resp.status_code == 404:
            logger.warning("Product not found in inventory", extra={'extra_data': {
                'request_id': request.request_id,
                'product': product,
            }})
            return jsonify({"error": "Product not found"}), 404

        logger.info("Inventory check completed", extra={'extra_data': {
            'request_id': request.request_id,
            'product': product,
        }})
        return jsonify(resp.json())

    except requests.exceptions.ConnectionError:
        logger.error("Upstream service unreachable", extra={'extra_data': {
            'request_id': request.request_id,
            'upstream': 'inventory-service',
        }})
        return jsonify({"error": "Service unavailable"}), 503


@app.route('/health')
def health():
    logger.debug("Health check requested", extra={'extra_data': {
        'request_id': request.request_id,
    }})
    return jsonify({"status": "healthy", "service": "api-gateway"})


if __name__ == '__main__':
    logger.info("Starting api-gateway", extra={'extra_data': {
        'port': 8000,
        'log_level': os.environ.get('LOG_LEVEL', 'INFO'),
    }})
    app.run(host='0.0.0.0', port=8000)
```

### order-service/app.py

```python
from flask import Flask, request, jsonify
import logging
import json
import sys
import os
import time
import uuid
from datetime import datetime, timezone

# (Insert JSONFormatter and setup_logger from above)

logger = setup_logger('order-service')

app = Flask(__name__)

orders_db = {}
STOCK_LOW_THRESHOLD = 10


@app.route('/orders', methods=['POST'])
def create_order():
    data = request.get_json()
    order_id = str(uuid.uuid4())[:8]

    logger.debug("Validating order payload", extra={'extra_data': {
        'order_id': order_id,
        'payload': data,
    }})

    if not data or 'product' not in data or 'quantity' not in data:
        logger.error("Invalid order payload", extra={'extra_data': {
            'order_id': order_id,
            'payload': str(data),
        }})
        return jsonify({"error": "product and quantity required"}), 400

    quantity = data['quantity']
    product = data['product']

    # Check for duplicate
    for existing in orders_db.values():
        if existing['product'] == product and existing['status'] == 'pending':
            logger.warning("Duplicate order attempt detected", extra={'extra_data': {
                'order_id': order_id,
                'product': product,
                'existing_order_id': existing['id'],
            }})

    # Simulate stock check
    simulated_stock = 50
    if quantity > simulated_stock:
        logger.warning("Insufficient stock for order", extra={'extra_data': {
            'order_id': order_id,
            'product': product,
            'requested': quantity,
            'available': simulated_stock,
        }})
        return jsonify({"error": "Insufficient stock"}), 409

    # Simulate payment processing
    if data.get('simulate_payment_failure'):
        logger.error("Payment processing failed", extra={'extra_data': {
            'order_id': order_id,
            'product': product,
            'amount': quantity * 10,
            'error_type': 'payment_gateway_timeout',
        }})
        return jsonify({"error": "Payment failed"}), 500

    orders_db[order_id] = {
        'id': order_id,
        'product': product,
        'quantity': quantity,
        'status': 'pending',
    }

    logger.info("Order created", extra={'extra_data': {
        'order_id': order_id,
        'product': product,
        'quantity': quantity,
        'status': 'pending',
    }})

    # Check if stock is getting low
    remaining = simulated_stock - quantity
    if remaining < STOCK_LOW_THRESHOLD:
        logger.warning("Stock below threshold after order", extra={'extra_data': {
            'product': product,
            'remaining_stock': remaining,
            'threshold': STOCK_LOW_THRESHOLD,
        }})

    return jsonify(orders_db[order_id]), 201


@app.route('/orders/<order_id>', methods=['GET'])
def get_order(order_id):
    logger.debug("Looking up order", extra={'extra_data': {
        'order_id': order_id,
    }})

    order = orders_db.get(order_id)
    if order is None:
        logger.warning("Order not found", extra={'extra_data': {
            'order_id': order_id,
        }})
        return jsonify({"error": "Order not found"}), 404

    logger.info("Order retrieved", extra={'extra_data': {
        'order_id': order_id,
        'status': order['status'],
    }})
    return jsonify(order)


@app.route('/health')
def health():
    return jsonify({"status": "healthy", "service": "order-service"})


if __name__ == '__main__':
    logger.info("Starting order-service", extra={'extra_data': {
        'port': 8001,
        'log_level': os.environ.get('LOG_LEVEL', 'INFO'),
    }})
    app.run(host='0.0.0.0', port=8001)
```

### inventory-service/app.py

```python
from flask import Flask, request, jsonify
import logging
import json
import sys
import os
from datetime import datetime, timezone

# (Insert JSONFormatter and setup_logger from above)

logger = setup_logger('inventory-service')

app = Flask(__name__)

inventory_db = {
    'widget': {'stock': 150, 'capacity': 500},
    'gadget': {'stock': 8, 'capacity': 200},
    'doohickey': {'stock': 300, 'capacity': 1000},
}

STOCK_THRESHOLD = 10
CAPACITY_THRESHOLD = 0.9


@app.route('/inventory/<product>', methods=['GET'])
def get_inventory(product):
    logger.debug("Cache lookup for product", extra={'extra_data': {
        'product': product,
        'cache': 'miss',
    }})

    item = inventory_db.get(product)
    if item is None:
        logger.warning("Product not found in inventory", extra={'extra_data': {
            'product': product,
        }})
        return jsonify({"error": "Product not found"}), 404

    logger.info("Inventory retrieved", extra={'extra_data': {
        'product': product,
        'stock': item['stock'],
    }})
    return jsonify({"product": product, **item})


@app.route('/inventory/<product>', methods=['PUT'])
def update_inventory(product):
    data = request.get_json()
    quantity = data.get('quantity', 0)

    logger.debug("Processing inventory update", extra={'extra_data': {
        'product': product,
        'quantity_change': quantity,
        'operation': 'reserve' if quantity < 0 else 'restock',
    }})

    if product not in inventory_db:
        logger.warning("Attempted update on unknown product", extra={'extra_data': {
            'product': product,
        }})
        return jsonify({"error": "Product not found"}), 404

    item = inventory_db[product]
    new_stock = item['stock'] + quantity

    if new_stock < 0:
        logger.error("Inventory inconsistency detected", extra={'extra_data': {
            'product': product,
            'current_stock': item['stock'],
            'requested_change': quantity,
            'resulting_stock': new_stock,
        }})
        return jsonify({"error": "Insufficient stock"}), 409

    if new_stock > item['capacity']:
        logger.warning("Inventory approaching capacity", extra={'extra_data': {
            'product': product,
            'current_stock': new_stock,
            'capacity': item['capacity'],
            'utilization': round(new_stock / item['capacity'], 2),
        }})

    item['stock'] = new_stock

    logger.info("Stock updated", extra={'extra_data': {
        'product': product,
        'previous_stock': item['stock'] - quantity,
        'new_stock': new_stock,
        'change': quantity,
    }})

    if new_stock < STOCK_THRESHOLD:
        logger.warning("Stock below threshold", extra={'extra_data': {
            'product': product,
            'stock': new_stock,
            'threshold': STOCK_THRESHOLD,
        }})

    return jsonify({"product": product, **item})


@app.route('/health')
def health():
    return jsonify({"status": "healthy", "service": "inventory-service"})


if __name__ == '__main__':
    logger.info("Starting inventory-service", extra={'extra_data': {
        'port': 8002,
        'log_level': os.environ.get('LOG_LEVEL', 'INFO'),
    }})
    app.run(host='0.0.0.0', port=8002)
```

### Dockerfiles (identical for all three services)

```dockerfile
FROM python:3.12-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY . .
CMD ["python", "app.py"]
```

### requirements.txt (all services)

```
flask==3.0.0
requests==2.31.0
```

### docker-compose.yml (base)

```yaml
version: "3.8"

services:
  api-gateway:
    build: ./api-gateway
    ports:
      - "8000:8000"
    environment:
      - LOG_LEVEL=${LOG_LEVEL:-INFO}
      - ORDER_SERVICE_URL=http://order-service:8001
      - INVENTORY_SERVICE_URL=http://inventory-service:8002
    logging:
      driver: json-file
      options:
        max-size: "5m"
        max-file: "3"
    depends_on:
      - order-service
      - inventory-service

  order-service:
    build: ./order-service
    ports:
      - "8001:8001"
    environment:
      - LOG_LEVEL=${LOG_LEVEL:-INFO}
    logging:
      driver: json-file
      options:
        max-size: "5m"
        max-file: "3"

  inventory-service:
    build: ./inventory-service
    ports:
      - "8002:8002"
    environment:
      - LOG_LEVEL=${LOG_LEVEL:-INFO}
    logging:
      driver: json-file
      options:
        max-size: "5m"
        max-file: "3"
```

### docker-compose.dev.yml

```yaml
version: "3.8"

services:
  api-gateway:
    environment:
      - LOG_LEVEL=DEBUG

  order-service:
    environment:
      - LOG_LEVEL=DEBUG

  inventory-service:
    environment:
      - LOG_LEVEL=DEBUG
```

### docker-compose.staging.yml

```yaml
version: "3.8"

services:
  api-gateway:
    environment:
      - LOG_LEVEL=INFO

  order-service:
    environment:
      - LOG_LEVEL=INFO

  inventory-service:
    environment:
      - LOG_LEVEL=INFO
```

### docker-compose.prod.yml

```yaml
version: "3.8"

services:
  api-gateway:
    environment:
      - LOG_LEVEL=WARNING

  order-service:
    environment:
      - LOG_LEVEL=WARNING

  inventory-service:
    environment:
      - LOG_LEVEL=WARNING
```

### Running and Testing

```bash
# Development -- all DEBUG logs visible
docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d --build

# Generate traffic
curl -X POST http://localhost:8000/order \
  -H "Content-Type: application/json" \
  -d '{"product": "widget", "quantity": 5}'
curl http://localhost:8000/inventory?product=widget
curl http://localhost:8000/inventory?product=nonexistent

# Check that DEBUG logs appear in dev mode
docker compose logs api-gateway 2>&1 | jq 'select(.level == "DEBUG")' | head -5

# Switch to production -- only WARNING and above
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# Generate the same traffic
curl -X POST http://localhost:8000/order \
  -H "Content-Type: application/json" \
  -d '{"product": "widget", "quantity": 5}

# Verify DEBUG logs are NOT present
docker compose logs api-gateway 2>&1 | jq 'select(.level == "DEBUG")'
# Should return nothing

# Verify WARNING logs ARE present (e.g., 404 on nonexistent product)
curl http://localhost:8000/inventory?product=nonexistent
docker compose logs inventory-service 2>&1 | jq 'select(.level == "WARNING")'
```

---

## Why This Works

1. **Environment variable controls log level.** `os.environ.get('LOG_LEVEL', 'INFO')`
   reads the level at startup. Docker Compose override files set different
   values per environment. No code change needed to switch verbosity.

2. **Python's logging module handles filtering.** When you set
   `logger.setLevel(logging.WARNING)`, the logger silently discards DEBUG
   and INFO messages before they reach the formatter. This is efficient --
   the JSON serialization never happens for suppressed messages.

3. **Compose override files are additive.** The base `docker-compose.yml`
   defines the services and their defaults. The override files only specify
   the `environment` section, which merges with (and overrides) the base.
   This is the standard Docker Compose pattern for environment-specific
   configuration.

4. **Log rotation is in the base file.** Every container gets rotation
   regardless of environment. This prevents any single container from
   filling the disk, even in development.

5. **Per-service log levels are possible.** If `order-service` is noisy and
   `inventory-service` is quiet, you can set different levels per service
   in the override file. The `LOG_LEVEL` environment variable is per-service.

## Common Mistakes

### Mistake 1: Setting log level in the Dockerfile

```dockerfile
# BAD -- baked into the image, cannot change without rebuild
ENV LOG_LEVEL=INFO
```

The level should be set via `docker-compose.yml` environment, not in the
image. The image should contain the code; the runtime configuration
should be external.

### Mistake 2: Using print() for some services and logging for others

All services must use the same logging mechanism. Mixing `print()` and
`logging` produces inconsistent output that breaks `jq` parsing.

### Mistake 3: Not testing that filtering actually works

Setting `LOG_LEVEL=WARNING` in the compose file is not enough. You must
verify that DEBUG logs do not appear in the output. Test by generating
traffic that would produce DEBUG logs and confirming they are absent.

### Mistake 4: Forgetting that Python's logging level is set at startup

If you change the `LOG_LEVEL` environment variable, you must restart the
container. The logger reads the level once during initialization. Some
frameworks support runtime reconfiguration, but the simple approach used
here requires a restart.
