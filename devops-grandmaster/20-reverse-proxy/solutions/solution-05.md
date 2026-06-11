# Solution 05: API Gateway with Rate Limiting and Authentication

## Complete Solution

### Directory Structure

```
exercise-05/
  docker-compose.yml
  nginx/
    nginx.conf
  services/
    product-service/
      app.py
      Dockerfile
    cart-service/
      app.py
      Dockerfile
    checkout-service/
      app.py
      Dockerfile
    auth-service/
      app.py
      Dockerfile
```

---

### `nginx/nginx.conf`

```nginx
events {
    worker_connections 1024;
}

http {
    # ============================================================
    # Rate limit zones (keyed by API key, not IP)
    # ============================================================
    limit_req_zone $http_x_api_key zone=free_tier:10m rate=5r/s;
    limit_req_zone $http_x_api_key zone=pro_tier:10m rate=20r/s;
    limit_req_zone $http_x_api_key zone=enterprise_tier:10m rate=100r/s;

    # ============================================================
    # Map: extract tier from API key prefix
    # ============================================================
    map $http_x_api_key $api_tier {
        default          "free";
        "~^free_"        "free";
        "~^pro_"         "pro";
        "~^ent_"         "enterprise";
        ""               "none";     # No API key
    }

    # ============================================================
    # Map: select rate limit zone based on tier
    # ============================================================
    map $api_tier $rate_limit_zone {
        "free"         free_tier;
        "pro"          pro_tier;
        "enterprise"   enterprise_tier;
        default        "";          # No zone for missing key (bypasses limit_req)
    }

    # ============================================================
    # Map: select burst based on tier
    # ============================================================
    map $api_tier $rate_limit_burst {
        "free"         10;
        "pro"          40;
        "enterprise"   200;
        default        10;
    }

    # ============================================================
    # Map: rate limit header value
    # ============================================================
    map $api_tier $rate_limit_value {
        "free"         "5";
        "pro"          "20";
        "enterprise"   "100";
        default        "5";
    }

    # ============================================================
    # Map: redact API key for logging (first 8 chars + ...)
    # ============================================================
    map $http_x_api_key $redacted_key {
        default              "none";
        "~^(?<prefix>.{8})"  "${prefix}...";
    }

    # ============================================================
    # Custom log format
    # ============================================================
    log_format api_gateway '$remote_addr - $redacted_key [$time_local] '
                           '"$request" $status $body_bytes_sent '
                           '"$http_referer" "$http_user_agent" '
                           'rt=$request_time';

    access_log /var/log/nginx/access.log api_gateway;

    # ============================================================
    # Upstream definitions
    # ============================================================
    upstream product_service {
        server product-service:5001;
    }

    upstream cart_service {
        server cart-service:5002;
    }

    upstream checkout_service {
        server checkout-service:5003;
    }

    upstream auth_service {
        server auth-service:5004;
    }

    # ============================================================
    # Server block
    # ============================================================
    server {
        listen 80;
        server_name localhost;

        # ----------------------------------------------------------
        # Health endpoint — no auth, no rate limiting
        # ----------------------------------------------------------
        location = /api/health {
            default_type application/json;
            return 200 '{"status": "ok"}';
        }

        # ----------------------------------------------------------
        # Auth subrequest — internal only
        # ----------------------------------------------------------
        location = /auth {
            internal;
            proxy_pass http://auth_service/api/v1/auth/validate;
            proxy_pass_request_body off;
            proxy_set_header Content-Length "";
            proxy_set_header X-API-Key $http_x_api_key;
        }

        # ----------------------------------------------------------
        # Product service
        # ----------------------------------------------------------
        location /api/v1/products {
            # Auth check
            auth_request /auth;
            auth_request_set $auth_status $upstream_status;

            # Rate limiting
            limit_req zone=$rate_limit_zone burst=$rate_limit_burst nodelay;
            limit_req_status 429;

            # Add custom headers
            add_header X-Request-ID $request_id always;
            add_header X-RateLimit-Limit $rate_limit_value always;

            # Proxy to backend
            proxy_pass http://product_service;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_set_header X-Request-ID $request_id;
            proxy_set_header X-API-Key $http_x_api_key;
        }

        # ----------------------------------------------------------
        # Cart service
        # ----------------------------------------------------------
        location /api/v1/cart {
            auth_request /auth;
            auth_request_set $auth_status $upstream_status;

            limit_req zone=$rate_limit_zone burst=$rate_limit_burst nodelay;
            limit_req_status 429;

            add_header X-Request-ID $request_id always;
            add_header X-RateLimit-Limit $rate_limit_value always;

            proxy_pass http://cart_service;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_set_header X-Request-ID $request_id;
            proxy_set_header X-API-Key $http_x_api_key;
        }

        # ----------------------------------------------------------
        # Checkout service
        # ----------------------------------------------------------
        location /api/v1/checkout {
            auth_request /auth;
            auth_request_set $auth_status $upstream_status;

            limit_req zone=$rate_limit_zone burst=$rate_limit_burst nodelay;
            limit_req_status 429;

            add_header X-Request-ID $request_id always;
            add_header X-RateLimit-Limit $rate_limit_value always;

            proxy_pass http://checkout_service;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_set_header X-Request-ID $request_id;
            proxy_set_header X-API-Key $http_x_api_key;
        }

        # ----------------------------------------------------------
        # Custom error responses
        # ----------------------------------------------------------
        error_page 401 = @unauthorized;
        error_page 403 = @forbidden;
        error_page 429 = @rate_limited;
        error_page 502 503 = @service_unavailable;

        location @unauthorized {
            default_type application/json;
            return 401 '{"error": "Missing API key", "status": 401}';
        }

        location @forbidden {
            default_type application/json;
            return 403 '{"error": "Invalid API key", "status": 403}';
        }

        location @rate_limited {
            default_type application/json;
            return 429 '{"error": "Rate limit exceeded", "status": 429, "retry_after": 1}';
        }

        location @service_unavailable {
            default_type application/json;
            return 503 '{"error": "Service Unavailable", "status": 503}';
        }
    }
}
```

---

### `services/auth-service/app.py`

```python
from flask import Flask, request, jsonify

app = Flask(__name__)

# Simulated valid API keys
VALID_KEYS = {
    "free_abc123": {"tier": "free", "name": "Free User"},
    "pro_xyz789": {"tier": "pro", "name": "Pro User"},
    "ent_master": {"tier": "enterprise", "name": "Enterprise User"},
    "free_test123": {"tier": "free", "name": "Test Free"},
}

@app.route('/api/v1/auth/validate', methods=['GET'])
def validate():
    api_key = request.headers.get('X-API-Key', '')
    if not api_key:
        return jsonify({"error": "No API key provided"}), 401
    if api_key not in VALID_KEYS:
        return jsonify({"error": "Invalid API key"}), 403
    return jsonify({"valid": True, "tier": VALID_KEYS[api_key]["tier"]}), 200

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5004)
```

### `services/auth-service/Dockerfile`

```dockerfile
FROM python:3.11-slim
WORKDIR /app
RUN pip install flask
COPY app.py .
CMD ["python", "app.py"]
```

### `services/product-service/app.py`

```python
from flask import Flask, jsonify

app = Flask(__name__)

@app.route('/api/v1/products')
def products():
    return jsonify({"service": "products", "items": ["Widget", "Gadget", "Doohickey"]})

@app.route('/api/v1/products/<product_id>')
def product(product_id):
    return jsonify({"service": "products", "id": product_id, "name": "Widget"})

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5001)
```

### `services/product-service/Dockerfile`

```dockerfile
FROM python:3.11-slim
WORKDIR /app
RUN pip install flask
COPY app.py .
CMD ["python", "app.py"]
```

### `services/cart-service/app.py`

```python
from flask import Flask, jsonify

app = Flask(__name__)

@app.route('/api/v1/cart')
def cart():
    return jsonify({"service": "cart", "items": [], "total": 0})

@app.route('/api/v1/cart/add', methods=['POST'])
def add_to_cart():
    return jsonify({"service": "cart", "message": "Item added"})

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5002)
```

### `services/cart-service/Dockerfile`

```dockerfile
FROM python:3.11-slim
WORKDIR /app
RUN pip install flask
COPY app.py .
CMD ["python", "app.py"]
```

### `services/checkout-service/app.py`

```python
from flask import Flask, jsonify

app = Flask(__name__)

@app.route('/api/v1/checkout')
def checkout():
    return jsonify({"service": "checkout", "status": "ready"})

@app.route('/api/v1/checkout/complete', methods=['POST'])
def complete():
    return jsonify({"service": "checkout", "order_id": "ORD-12345", "status": "confirmed"})

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5003)
```

### `services/checkout-service/Dockerfile`

```dockerfile
FROM python:3.11-slim
WORKDIR /app
RUN pip install flask
COPY app.py .
CMD ["python", "app.py"]
```

---

### `docker-compose.yml`

```yaml
version: "3.8"

services:
  proxy:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      - product-service
      - cart-service
      - checkout-service
      - auth-service

  product-service:
    build: ./services/product-service
    expose:
      - "5001"

  cart-service:
    build: ./services/cart-service
    expose:
      - "5002"

  checkout-service:
    build: ./services/checkout-service
    expose:
      - "5003"

  auth-service:
    build: ./services/auth-service
    expose:
      - "5004"
```

---

## Why It Works

### auth_request for API key validation

```nginx
auth_request /auth;
```

This directive tells Nginx to send a subrequest to the `/auth` internal location before forwarding the original request. The subrequest is a lightweight HTTP request to the auth service.

The internal location:

```nginx
location = /auth {
    internal;
    proxy_pass http://auth_service/api/v1/auth/validate;
    proxy_pass_request_body off;
    proxy_set_header Content-Length "";
    proxy_set_header X-API-Key $http_x_api_key;
}
```

- `internal` prevents external clients from hitting this endpoint directly.
- `proxy_pass_request_body off` sends a GET-like subrequest (no body), reducing overhead.
- `proxy_set_header Content-Length ""` is required when disabling the body to avoid a mismatch.
- `proxy_set_header X-API-Key $http_x_api_key` forwards the client's API key to the auth service.

If the auth service returns 2xx, the original request proceeds. If it returns 401 or 403, Nginx returns that status to the client. The `error_page` directives then serve the JSON error bodies.

### map directive for tier-based rate limiting

The `map` directive is Nginx's equivalent of a switch statement. It evaluates a variable and maps it to another value:

```nginx
map $http_x_api_key $api_tier {
    default          "free";
    "~^free_"        "free";
    "~^pro_"         "pro";
    "~^ent_"         "enterprise";
}
```

This extracts the tier from the API key prefix using regex matching. The `~` prefix indicates a regex pattern. `^free_` matches keys starting with `free_`.

A second `map` selects the rate limit zone:

```nginx
map $api_tier $rate_limit_zone {
    "free"         free_tier;
    "pro"          pro_tier;
    "enterprise"   enterprise_tier;
    default        "";
}
```

This allows a single `limit_req` directive in each location block to dynamically select the correct zone based on the client's tier. Without `map`, you would need duplicate location blocks for each tier.

### Dynamic rate limit zone selection

```nginx
limit_req zone=$rate_limit_zone burst=$rate_limit_burst nodelay;
```

The `zone` parameter accepts variables, so Nginx selects the correct zone at request time. The `nodelay` flag means rate-limited requests are immediately rejected (503/429) rather than queued.

**Important caveat:** When `$rate_limit_zone` is empty (no API key), `limit_req` with an empty zone is ignored. This means unauthenticated requests bypass rate limiting — but they are caught by `auth_request` first, so they never reach the backend.

### Redacted API keys in logs

```nginx
map $http_x_api_key $redacted_key {
    default              "none";
    "~^(?<prefix>.{8})"  "${prefix}...";
}
```

The named capture group `(?<prefix>.{8})` grabs the first 8 characters. The `${prefix}...` substitution produces something like `free_abc...`. This allows debugging (you can identify which tier was used) without exposing the full key in logs.

### Custom error pages for each failure mode

Each error has its own named location with a specific JSON body:

- `@unauthorized` (401) — missing API key
- `@forbidden` (403) — invalid API key
- `@rate_limited` (429) — rate limit exceeded
- `@service_unavailable` (502/503) — backend is down

This provides clear, actionable error messages to API consumers. A well-designed API always returns structured errors.

---

## Common Mistakes

### 1. Using `$remote_addr` for rate limiting instead of `$http_x_api_key`

**Symptom:** All requests from the same IP share the same rate limit, regardless of API key.

**Cause:** `limit_req_zone $remote_addr ...` keys on the client IP. If multiple API consumers share an IP (corporate NAT, mobile carriers), they interfere with each other.

**Fix:** Use `limit_req_zone $http_x_api_key ...` to key on the API key.

### 2. Missing `proxy_pass_request_body off` in auth subrequest

**Symptom:** POST requests to the API hang or fail.

**Cause:** The `auth_request` subrequest includes the original request body by default. The auth service does not expect a body on a validation endpoint, so it may hang waiting for the body to arrive or reject the request.

### 3. auth_request returns 401 but client sees 500

**Symptom:** Instead of a 401 JSON error, the client gets a 500 Internal Server Error.

**Cause:** Missing `error_page 401 = @unauthorized;` or the named location is missing. Nginx passes through the auth service's status code but does not automatically serve a custom body.

### 4. Rate limit zone is empty for missing API keys

**Symptom:** Requests without an API key are not rate limited — they get a 401 from auth_request but are not counted.

**Cause:** When `$rate_limit_zone` is empty, `limit_req` is effectively skipped. This is usually acceptable (auth_request rejects them first), but if you want to rate-limit even unauthenticated requests, map the empty case to a default zone.

### 5. `add_header` does not work on error responses

**Symptom:** 401 and 429 responses do not include `X-Request-ID` or `X-RateLimit-Limit` headers.

**Cause:** Nginx's `add_header` directive only applies to successful (2xx, 3xx) responses by default. On error responses, the error page location block generates a fresh response.

**Fix:** Add `always` to `add_header` directives: `add_header X-Request-ID $request_id always;`. This ensures the header is present on all responses, including errors.

### 6. X-RateLimit-Remaining is not implemented

**Symptom:** The header is missing or always shows the same value.

**Cause:** Nginx does not expose the remaining request count in a rate limit zone. The `limit_req` module only knows when a request is over the limit — it does not expose the current count.

**Production alternatives:**
- Use OpenResty (Nginx + Lua) with `lua-resty-limit-req`, which exposes the remaining count.
- Use an external rate limiter (Redis + a rate limiting library) and return the count from the auth subresponse.
- Use a dedicated API gateway (Kong, Ambassador, AWS API Gateway) that tracks this natively.

This is a documented limitation of vanilla Nginx and is an expected gap in the exercise.
