# Solution 04: Path-Based Routing for Microservices

## Complete Solution

### Directory Structure

```
exercise-04/
  docker-compose.yml
  nginx/
    nginx.conf
    error.json
```

### `nginx/error.json`

```json
{
    "error": "Service Unavailable",
    "message": "The requested service is temporarily unavailable",
    "status": 503
}
```

### `nginx/nginx.conf`

```nginx
events {
    worker_connections 1024;
}

http {
    # Upstream definitions
    upstream frontend {
        server frontend:3000;
    }

    upstream user_api {
        server user-api:5001;
    }

    upstream product_api {
        server product-api:5002;
    }

    upstream order_api {
        server order-api:5003;
    }

    upstream websocket {
        server ws-server:8080;
    }

    server {
        listen 80;
        server_name localhost;

        # Health check — served directly by Nginx, no proxy
        location /health {
            default_type application/json;
            return 200 '{"status": "ok"}';
        }

        # Frontend: catch-all for root
        location / {
            proxy_pass http://frontend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_set_header X-Request-ID $request_id;
        }

        # User API: rewrite /api/users/* -> /users/*
        location /api/users/ {
            proxy_pass http://user_api/;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_set_header X-Request-ID $request_id;
        }

        # Product API: rewrite /api/products/* -> /products/*
        location /api/products/ {
            proxy_pass http://product_api/;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_set_header X-Request-ID $request_id;
        }

        # Order API: rewrite /api/orders/* -> /orders/*
        location /api/orders/ {
            proxy_pass http://order_api/;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_set_header X-Request-ID $request_id;
        }

        # WebSocket: upgrade support
        location /ws {
            proxy_pass http://websocket;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_set_header X-Request-ID $request_id;
            proxy_read_timeout 3600s;
            proxy_send_timeout 3600s;
        }

        # Custom error handling for 502 and 503
        error_page 502 503 = @error_fallback;

        location @error_fallback {
            default_type application/json;
            return 503 '{"error": "Service Unavailable", "message": "The requested service is temporarily unavailable", "status": 503}';
        }
    }
}
```

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
      - frontend
      - user-api
      - product-api
      - order-api
      - ws-server

  frontend:
    image: nginx:alpine
    volumes:
      - ./frontend:/usr/share/nginx/html:ro
    expose:
      - "3000"
    # Override default nginx port to 3000
    command: >
      sh -c "sed -i 's/listen\s*80/listen 3000/' /etc/nginx/conf.d/default.conf &&
             nginx -g 'daemon off;'"

  user-api:
    image: traefik/whoami
    command: ["--port", "5001"]
    expose:
      - "5001"

  product-api:
    image: traefik/whoami
    command: ["--port", "5002"]
    expose:
      - "5002"

  order-api:
    image: traefik/whoami
    command: ["--port", "5003"]
    expose:
      - "5003"

  ws-server:
    image: traefik/whoami
    command: ["--port", "8080"]
    expose:
      - "8080"
```

---

## Why It Works

### Path rewriting with trailing slashes

The key mechanism is the interaction between the `location` prefix and the `proxy_pass` URI:

```nginx
location /api/users/ {
    proxy_pass http://user_api/;
}
```

When Nginx sees a `proxy_pass` with a URI (even just `/`), it **replaces** the matched `location` prefix with that URI. So:

| Incoming request | Matched location | proxy_pass URI | Backend receives |
|---|---|---|---|
| `/api/users/123` | `/api/users/` | `http://user_api/` | `/123` |
| `/api/users/search?q=john` | `/api/users/` | `http://user_api/` | `/search?q=john` |
| `/api/products/456` | `/api/products/` | `http://product_api/` | `/456` |

The trailing `/` on both `location` and `proxy_pass` is critical. If you omit the trailing `/` on `proxy_pass`, Nginx forwards the full original URI without stripping the prefix.

### WebSocket upgrade

WebSocket connections start as HTTP requests with special headers:

```
GET /ws HTTP/1.1
Upgrade: websocket
Connection: Upgrade
```

Nginx must forward these headers to the backend. The three required directives are:

1. `proxy_http_version 1.1;` — WebSocket requires HTTP/1.1, not 1.0.
2. `proxy_set_header Upgrade $http_upgrade;` — forwards the client's `Upgrade: websocket` header.
3. `proxy_set_header Connection "upgrade";` — tells the backend to switch protocols.

Without all three, the WebSocket handshake fails with a 400 or the connection silently falls back to HTTP polling.

The `proxy_read_timeout 3600s;` prevents Nginx from closing idle WebSocket connections. Default Nginx timeout is 60 seconds, which would kill any WebSocket that is idle for a minute.

### Health check without proxying

```nginx
location /health {
    default_type application/json;
    return 200 '{"status": "ok"}';
}
```

The `return` directive sends a response directly from Nginx without contacting any backend. This is useful for load balancer health checks — they can verify Nginx is running without depending on any backend service. The `default_type` ensures the `Content-Type` header is `application/json`.

### Custom error pages

```nginx
error_page 502 503 = @error_fallback;

location @error_fallback {
    default_type application/json;
    return 503 '{"error": "Service Unavailable", ...}';
}
```

When a backend is down, Nginx receives a connection refused and would normally serve its default HTML error page. The `error_page` directive intercepts 502 (Bad Gateway — backend unreachable) and 503 (Service Unavailable) errors and redirects to a named location. The named location returns a JSON response with the correct status code.

The `= @error_fallback` syntax preserves the original error status code. If the upstream returned 502, the client sees 502. If it returned 503, the client sees 503.

### X-Request-ID for tracing

```nginx
proxy_set_header X-Request-ID $request_id;
```

Nginx's `$request_id` variable generates a unique 32-character hexadecimal string for each request. This ID is invaluable for debugging: when a request passes through multiple services, you can grep logs across all services using the same request ID to trace the full request path.

---

## Common Mistakes

### 1. Missing trailing slash on `proxy_pass`

**Symptom:** Requests to `/api/users/123` reach the backend as `/api/users/123` instead of `/123`.

**Cause:** `proxy_pass http://user_api;` (no trailing `/`) does not strip the location prefix. The full original URI is forwarded.

**Fix:** Always add a trailing `/` to both `location` and `proxy_pass` when you want prefix stripping:
```
location /api/users/ {
    proxy_pass http://user_api/;
}
```

### 2. `location` and `proxy_pass` trailing slash mismatch

**Symptom:** 404 errors or unexpected paths on the backend.

**Cause:** If `location` has a trailing slash but `proxy_pass` does not (or vice versa), the path transformation is not what you expect.

**Rule:** Both must have trailing slashes, or neither must. For prefix stripping, both need them.

### 3. WebSocket connections fail silently

**Symptom:** The client falls back to HTTP long-polling instead of using WebSocket.

**Cause:** Missing one of the three WebSocket headers. The most commonly forgotten one is `proxy_http_version 1.1;`.

### 4. Error page returns HTML instead of JSON

**Symptom:** When a backend is down, the client receives `<html><body><h1>502 Bad Gateway</h1>...` instead of JSON.

**Cause:** The `error_page` directive is missing, or the named location does not have `default_type application/json;`.

### 5. Catch-all `location /` matches everything first

**Symptom:** All requests go to the frontend, including `/api/*`.

**Cause:** Nginx processes `location` blocks by specificity. A `location /` prefix match has the lowest priority, but if the more specific `/api/` locations are missing or have typos, the catch-all wins.

**Fix:** Ensure your API location blocks are present and have correct paths. Nginx selects the longest matching prefix, so `/api/users/` beats `/` for requests starting with `/api/users/`.

### 6. Health endpoint proxied to a backend

**Symptom:** `/health` returns an error when backends are down.

**Cause:** The `/health` location is using `proxy_pass` instead of `return`. A health endpoint should be independent of backend availability.

**Fix:** Use `return 200 ...` instead of `proxy_pass`.
