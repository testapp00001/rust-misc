# Exercise 05: Integration — API Gateway with Rate Limiting and Authentication

## Type
Integration — combines multiple concepts into a production-like setup.

## Objective

Build a complete API gateway using Nginx that enforces rate limiting, requires API key authentication, and routes to multiple backend services. This exercise simulates a real production API gateway.

## Scenario

You are building the API gateway for "ShopStack", an e-commerce platform. The gateway must:

1. Authenticate all API requests using an API key.
2. Apply different rate limits to different tiers of clients.
3. Route requests to the correct microservice.
4. Provide a public health endpoint (no auth required).
5. Log all requests with the client's API key for auditing.

## Services

| Service | Internal Port | Public Path | Auth Required |
|---------|---------------|-------------|---------------|
| Product Service | 5001 | `/api/v1/products` | Yes |
| Cart Service | 5002 | `/api/v1/cart` | Yes |
| Checkout Service | 5003 | `/api/v1/checkout` | Yes |
| Health | N/A | `/api/health` | No |
| Auth Info | 5004 | `/api/v1/auth/validate` | No (it validates keys) |

## Rate Limiting Tiers

| Tier | Rate | Burst | Identified By |
|------|------|-------|---------------|
| Free | 5 requests/second | 10 | API key starting with `free_` |
| Pro | 20 requests/second | 40 | API key starting with `pro_` |
| Enterprise | 100 requests/second | 200 | API key starting with `ent_` |

## Requirements

### 1. API Key Authentication

- Every request to `/api/v1/*` (except `/api/health`) must include an `X-API-Key` header.
- If the header is missing, return `401 Unauthorized` with JSON: `{"error": "Missing API key", "status": 401}`.
- The gateway validates the key by forwarding the request to the Auth Service at `http://auth:5004/api/v1/auth/validate` with the `X-API-Key` header.
- If the Auth Service returns a non-200 status, the gateway must return `403 Forbidden` with JSON: `{"error": "Invalid API key", "status": 403}`.

Use Nginx's `auth_request` directive to implement this. The auth service acts as a subrequest validator.

### 2. Rate Limiting

- Use `limit_req_zone` with the `$http_x_api_key` variable (not `$remote_addr`) so that rate limits follow the API key, not the IP address.
- Create three zones: `free_tier`, `pro_tier`, `enterprise_tier`.
- Use Nginx's `map` directive to extract the tier from the API key prefix and select the appropriate zone.
- When rate limited, return `429 Too Many Requests` with JSON: `{"error": "Rate limit exceeded", "status": 429, "retry_after": 1}`.

### 3. Request Logging

Configure Nginx to log each request with:
- Timestamp
- Client IP
- API key (redacted — show only first 8 characters followed by `...`)
- Method and URI
- Upstream response status
- Response time

Use a custom `log_format` named `api_gateway`.

### 4. Response Headers

The gateway must add these headers to every response:
- `X-Request-ID` — unique per request (`$request_id`)
- `X-RateLimit-Limit` — the tier's rate limit (e.g., `5`, `20`, `100`)
- `X-RateLimit-Remaining` — remaining requests in the current window (this is tricky with Nginx; use `limit_req_status` or a header from the auth subrequest if possible — if not, document the limitation)

### 5. Mock Backend Services

Use simple Docker services for the backends. Each service should respond with a JSON message identifying itself. Example for the Product Service:

```python
from flask import Flask, jsonify
app = Flask(__name__)

@app.route('/api/v1/products')
def products():
    return jsonify({"service": "products", "items": ["Widget", "Gadget"]})

@app.route('/api/v1/products/<product_id>')
def product(product_id):
    return jsonify({"service": "products", "id": product_id, "name": "Widget"})

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5001)
```

Create similar stubs for Cart, Checkout, and Auth services.

## Deliverables

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

## Testing

```bash
docker compose up -d

# Test without API key — should fail
curl http://localhost/api/v1/products
# Expected: 401 {"error": "Missing API key", "status": 401}

# Test with invalid API key — should fail
curl -H "X-API-Key: invalid_key" http://localhost/api/v1/products
# Expected: 403 {"error": "Invalid API key", "status": 403}

# Test with valid free-tier key
curl -H "X-API-Key: free_abc123" http://localhost/api/v1/products
# Expected: 200 {"service": "products", "items": [...]}

# Test with valid pro-tier key
curl -H "X-API-Key: pro_xyz789" http://localhost/api/v1/cart
# Expected: 200 {"service": "cart", ...}

# Test health endpoint — no auth required
curl http://localhost/api/health
# Expected: 200 {"status": "ok"}

# Test rate limiting (free tier)
for i in $(seq 1 20); do
  curl -s -o /dev/null -w "%{http_code}\n" \
    -H "X-API-Key: free_test123" \
    http://localhost/api/v1/products
done
# Expected: some 200s, then 429s after burst is exhausted
```

## Success Criteria

- [ ] Requests without `X-API-Key` to `/api/v1/*` return 401.
- [ ] Requests with an invalid API key return 403.
- [ ] Requests with a valid API key are forwarded to the correct backend.
- [ ] `/api/health` works without any API key.
- [ ] Free-tier keys are rate limited at 5 req/s with burst of 10.
- [ ] Pro-tier keys are rate limited at 20 req/s with burst of 40.
- [ ] Enterprise-tier keys are rate limited at 100 req/s with burst of 200.
- [ ] Rate-limited requests return 429 with a JSON body.
- [ ] Each backend service returns its own identification in the response.
- [ ] The access log includes the (redacted) API key and response time.

## Hints

<details>
<summary>Hint 1 — auth_request directive</summary>
The `auth_request` directive sends a subrequest to an internal location. If the subrequest returns 2xx, the original request proceeds. If it returns 401 or 403, that status is returned to the client.
```nginx
location /api/v1/ {
    auth_request /auth;
    auth_request_set $auth_status $upstream_status;
    ...
}

location = /auth {
    internal;
    proxy_pass http://auth-service:5004/api/v1/auth/validate;
    proxy_pass_request_body off;
    proxy_set_header Content-Length "";
    proxy_set_header X-API-Key $http_x_api_key;
}
```
</details>

<details>
<summary>Hint 2 — map directive for tier selection</summary>
Use `map` to extract the tier from the API key prefix:
```nginx
map $http_x_api_key $api_tier {
    default         "free";
    ~^free_         "free";
    ~^pro_          "pro";
    ~^ent_          "enterprise";
}
```
Then use another `map` to select the rate limit zone:
```nginx
map $api_tier $rate_limit_zone {
    "free"        free_tier;
    "pro"         pro_tier;
    "enterprise"  enterprise_tier;
}
```
</details>

<details>
<summary>Hint 3 — Custom 429 error page</summary>
Use `limit_req_status 429;` combined with `error_page`:
```nginx
limit_req_status 429;
error_page 429 = @rate_limited;

location @rate_limited {
    default_type application/json;
    return 429 '{"error": "Rate limit exceeded", "status": 429, "retry_after": 1}';
}
```
</details>

<details>
<summary>Hint 4 — Redacting API keys in logs</summary>
You can use Nginx's `map` with a regex capture to extract the first 8 characters:
```nginx
map $http_x_api_key $redacted_key {
    default          "none";
    "~^(?<prefix>.{8})"  "${prefix}...";
}
```
Then use `$redacted_key` in your custom `log_format`.
</details>

<details>
<summary>Hint 5 — X-RateLimit-Remaining is hard</summary>
Nginx does not expose the remaining request count in the current window natively. This is a known limitation. Document this in your solution and explain what a production system would use instead (e.g., a Lua script, OpenResty, or an external rate limiter like Redis).
</details>
