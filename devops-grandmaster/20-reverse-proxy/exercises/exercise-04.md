# Exercise 04: Challenge — Path-Based Routing for Microservices

## Type
Challenge — no starter code, you design and build from scratch.

## Objective

Design and implement an Nginx reverse proxy configuration that routes traffic to five microservices based on URL path prefixes. You must handle path rewriting, WebSocket upgrades, and custom error pages.

## Scenario

You are building a platform with the following services:

| Service | Internal Port | Public Path | Notes |
|---------|---------------|-------------|-------|
| Frontend | 3000 | `/` | React SPA, serves static files |
| User API | 5001 | `/api/users` | REST API, expects paths without the prefix |
| Product API | 5002 | `/api/products` | REST API, expects paths without the prefix |
| Order API | 5003 | `/api/orders` | REST API, expects paths without the prefix |
| WebSocket | 8080 | `/ws` | Real-time notifications |

## Requirements

### 1. Path Rewriting

The API services expect clean paths. When a request arrives at `/api/users/123`, the User API should receive `/users/123` (with `/api` stripped). Implement this for all three API services.

### 2. WebSocket Support

The `/ws` path must support WebSocket upgrades. Your Nginx config must include the required headers (`Upgrade`, `Connection`) and appropriate timeouts for long-lived connections.

### 3. Custom Error Pages

- If any upstream service is down, Nginx should return a custom JSON error response instead of the default Nginx error page.
- The error response format must be: `{"error": "Service Unavailable", "message": "The requested service is temporarily unavailable", "status": 503}`
- This applies to `502` and `503` errors.

### 4. Request Headers

Every proxied request must include:
- `Host` — the original host
- `X-Real-IP` — the client's IP address
- `X-Forwarded-For` — the chain of proxy IPs
- `X-Forwarded-Proto` — `http` or `https`
- `X-Request-ID` — a unique ID for tracing (use Nginx's `$request_id` variable)

### 5. Health Check Endpoint

Nginx itself should serve a health check at `/health` that returns `{"status": "ok"}` without proxying to any backend. Use Nginx's `return` directive or `default_type` with `return 200`.

### 6. Docker Compose

Create a `docker-compose.yml` with all 6 containers (5 services + 1 Nginx proxy). Use simple mock services (e.g., `traefik/whoami` or a one-line Python HTTP server) — the focus is on the Nginx configuration, not the backends.

## Deliverables

```
exercise-04/
  docker-compose.yml
  nginx/
    nginx.conf
    error.json          # The custom error response
```

## Testing

```bash
docker compose up -d

# Test frontend
curl http://localhost/

# Test path rewriting
curl http://localhost/api/users/123
# The User API should receive /users/123

curl http://localhost/api/products/search?q=laptop
# The Product API should receive /products/search?q=laptop

# Test health endpoint
curl http://localhost/health
# Expected: {"status": "ok"}

# Test WebSocket (install wscat: npm i -g wscat)
wscat -c ws://localhost/ws
# Expected: successful WebSocket connection

# Simulate a backend being down
docker compose stop user-api
curl http://localhost/api/users/123
# Expected: {"error": "Service Unavailable", "message": "...", "status": 503}
```

## Success Criteria

- [ ] `/api/users/*` requests are rewritten and forwarded to User API (prefix `/api` stripped).
- [ ] `/api/products/*` requests are rewritten and forwarded to Product API (prefix `/api` stripped).
- [ ] `/api/orders/*` requests are rewritten and forwarded to Order API (prefix `/api` stripped).
- [ ] WebSocket connections at `/ws` are properly upgraded.
- [ ] `/health` returns `{"status": "ok"}` directly from Nginx.
- [ ] When a backend is down, the client receives a JSON error, not an Nginx HTML error page.
- [ ] All five required headers are set on every proxied request.
- [ ] The `X-Request-ID` header contains a unique value per request.

## Hints

<details>
<summary>Hint 1 — Path rewriting with proxy_pass</summary>
When `proxy_pass` includes a URI (even just `/`), Nginx replaces the matched `location` prefix with that URI. Example:
```
location /api/users/ {
    proxy_pass http://user-api:5001/;
}
```
The trailing `/` on both `location` and `proxy_pass` strips `/api/users` and passes the remainder.
</details>

<details>
<summary>Hint 2 — Custom error pages as JSON</summary>
Use `error_page` to redirect errors to a named location:
```nginx
error_page 502 503 = @error_fallback;

location @error_fallback {
    default_type application/json;
    return 503 '{"error": "Service Unavailable", "message": "The requested service is temporarily unavailable", "status": 503}';
}
```
</details>

<details>
<summary>Hint 3 — X-Request-ID</summary>
Nginx has a built-in `$request_id` variable (available since Nginx 1.11.0). Add this header:
```
proxy_set_header X-Request-ID $request_id;
```
</details>

<details>
<summary>Hint 4 — WebSocket location block</summary>
WebSocket requires three specific settings:
```nginx
proxy_http_version 1.1;
proxy_set_header Upgrade $http_upgrade;
proxy_set_header Connection "upgrade";
```
Also set `proxy_read_timeout` to a high value (e.g., 3600s) to prevent idle disconnection.
</details>

<details>
<summary>Hint 5 — Health check without proxying</summary>
Add a dedicated location that returns a static response:
```nginx
location /health {
    default_type application/json;
    return 200 '{"status": "ok"}';
}
```
</details>
