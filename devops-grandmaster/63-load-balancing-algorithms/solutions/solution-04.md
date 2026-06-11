# Solution 04: Production Load Balancer Design

## Part A: Nginx Configuration

```nginx
# /etc/nginx/nginx.conf

worker_processes auto;
worker_rlimit_nofile 65535;

events {
    worker_connections 16384;
    multi_accept on;
    use epoll;
}

http {
    # ──────────────────────────────────────────────
    # Rate Limiting
    # ──────────────────────────────────────────────
    # Define rate limit zone: 100 req/s per client IP
    # 10m zone stores ~160,000 IP addresses
    limit_req_zone $binary_remote_addr zone=api_limit:10m rate=100r/s;

    # Log rate-limited requests
    log_format main '$remote_addr - $remote_user [$time_local] '
                    '"$request" $status $body_bytes_sent '
                    '"$http_referer" "$http_user_agent" '
                    'rt=$request_time';

    access_log /var/log/nginx/access.log main;

    # ──────────────────────────────────────────────
    # SSL Configuration
    # ──────────────────────────────────────────────
    ssl_certificate /etc/nginx/ssl/company.com.crt;
    ssl_certificate_key /etc/nginx/ssl/company.com.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;
    ssl_prefer_server_ciphers on;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;

    # ──────────────────────────────────────────────
    # API Backend Pool (round-robin, default)
    # ──────────────────────────────────────────────
    upstream api_backend {
        # Health check parameters:
        # max_fails=3: After 3 failures, mark server as unhealthy
        # fail_timeout=30s: Unhealthy for 30s, then retry
        server 10.0.1.1:8080 max_fails=3 fail_timeout=30s;
        server 10.0.1.2:8080 max_fails=3 fail_timeout=30s;
        server 10.0.1.3:8080 max_fails=3 fail_timeout=30s;
        # api-server-4 is unhealthy -- excluded or marked down
        server 10.0.1.4:8080 max_fails=3 fail_timeout=30s down;
    }

    # ──────────────────────────────────────────────
    # WebSocket Backend Pool (least-connections)
    # ──────────────────────────────────────────────
    upstream ws_backend {
        least_conn;
        server 10.0.1.1:8080 max_fails=3 fail_timeout=30s;
        server 10.0.1.2:8080 max_fails=3 fail_timeout=30s;
        server 10.0.1.3:8080 max_fails=3 fail_timeout=30s;
    }

    # ──────────────────────────────────────────────
    # Server Block
    # ──────────────────────────────────────────────
    server {
        listen 443 ssl http2;
        server_name api.company.com;

        # Security headers
        add_header Strict-Transport-Security "max-age=31536000" always;
        add_header X-Content-Type-Options "nosniff" always;

        # Health check endpoint (exempt from rate limiting)
        location /health {
            limit_req off;
            access_log off;
            return 200 "OK\n";
            add_header Content-Type text/plain;
        }

        # API endpoints (round-robin, rate-limited)
        location /api/ {
            limit_req zone=api_limit burst=200 nodelay;

            proxy_pass http://api_backend;
            proxy_http_version 1.1;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;

            # Timeouts
            proxy_connect_timeout 5s;
            proxy_send_timeout 30s;
            proxy_read_timeout 30s;

            # Connection draining: keepalive to backend
            proxy_next_upstream error timeout http_502 http_503;
            proxy_next_upstream_tries 2;
            proxy_next_upstream_timeout 10s;
        }

        # WebSocket endpoints (least-connections)
        location /ws/ {
            limit_req off;  # No rate limiting for WebSocket

            proxy_pass http://ws_backend;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;

            # WebSocket timeouts (long-lived connections)
            proxy_read_timeout 3600s;
            proxy_send_timeout 3600s;
        }

        # Default: return 404 for unmatched paths
        location / {
            return 404;
        }
    }

    # HTTP to HTTPS redirect
    server {
        listen 80;
        server_name api.company.com;
        return 301 https://$host$request_uri;
    }
}
```

### Why api-server-4 Is Marked Down

The scenario states api-server-4 is unhealthy with 200 active connections.
Marking it as `down` immediately stops new traffic. Existing connections
will complete (connection draining) but no new connections are accepted.
This is better than waiting for health check failures.

## Part B: Health Check Design

### Passive Health Check (Nginx Open Source)

```nginx
upstream api_backend {
    server 10.0.1.1:8080 max_fails=3 fail_timeout=30s;
    server 10.0.1.2:8080 max_fails=3 fail_timeout=30s;
    server 10.0.1.3:8080 max_fails=3 fail_timeout=30s;
}
```

**How it works:**
- Nginx tracks failed requests to each backend
- A request fails if: connection refused, connection timeout, or HTTP 502/503
- After `max_fails` (3) failures within `fail_timeout` (30s), the backend
  is marked as unhealthy
- After `fail_timeout` (30s), Nginx retries the backend
- If the retry succeeds, the backend is marked healthy again

**Limitation:** Passive checks only detect failures when real traffic hits
the backend. If no traffic is sent, a dead backend goes undetected.

### Active Health Check (Nginx Plus)

```nginx
upstream api_backend {
    zone api_backend_zone 64k;
    server 10.0.1.1:8080;
    server 10.0.1.2:8080;
    server 10.0.1.3:8080;
}

server {
    location / {
        proxy_pass http://api_backend;
        health_check interval=5s
                     fails=3
                     passes=2
                     uri=/health
                     match=health_ok;
    }
}

# Define what a "healthy" response looks like
match health_ok {
    status 200;
    body ~ "OK";
}
```

**Configuration:**
- `interval=5s`: Check every 5 seconds
- `fails=3`: 3 consecutive failures to mark unhealthy
- `passes=2`: 2 consecutive successes to mark healthy again
- `uri=/health`: Specific health endpoint
- `match=health_ok`: Validates response status and body

### Health Endpoint Design

```python
# Backend health endpoint implementation

@app.route('/health')
def health():
    """Liveness check: is the process running?"""
    return jsonify({"status": "ok"}), 200

@app.route('/health/ready')
def health_ready():
    """Readiness check: can the process handle requests?"""
    checks = {
        "database": check_database_connection(),
        "redis": check_redis_connection(),
        "disk_space": check_disk_space(),
    }
    all_healthy = all(checks.values())
    status_code = 200 if all_healthy else 503
    return jsonify({"status": "ok" if all_healthy else "degraded", "checks": checks}), status_code

def check_database_connection():
    try:
        db.session.execute(text('SELECT 1'))
        return True
    except Exception:
        return False

def check_redis_connection():
    try:
        redis_client.ping()
        return True
    except Exception:
        return False

def check_disk_space():
    import shutil
    total, used, free = shutil.disk_usage("/")
    return free > 1_000_000_000  # At least 1GB free
```

**Why two endpoints:**
- `/health` (liveness): "Is the process alive?" Used by the OS/container
  runtime to decide whether to restart the process.
- `/health/ready` (readiness): "Can the process serve traffic?" Used by
  the load balancer to decide whether to send requests.

A process can be alive but not ready (e.g., database connection lost).
The load balancer should stop sending traffic, but the process should
not be killed.

## Part C: Connection Draining

### 1. What happens to in-flight requests?

When a backend is marked unhealthy:
- **New requests:** Routed to healthy backends immediately
- **In-flight requests:** Continue to completion on the unhealthy backend
- **Keep-alive connections:** No new requests sent; existing idle connections
  eventually timeout

The load balancer does NOT forcibly close active connections. This allows
requests that are almost complete to finish successfully.

### 2. Nginx Connection Draining Configuration

```nginx
upstream api_backend {
    server 10.0.1.1:8080 max_fails=3 fail_timeout=30s;
    server 10.0.1.2:8080 max_fails=3 fail_timeout=30s;
    server 10.0.1.3:8080 max_fails=3 fail_timeout=30s;
}

server {
    location /api/ {
        proxy_pass http://api_backend;

        # Retry on failure, but only try 2 backends
        proxy_next_upstream error timeout http_502 http_503;
        proxy_next_upstream_tries 2;
        proxy_next_upstream_timeout 10s;

        # Keepalive connections to backends
        proxy_http_version 1.1;
        proxy_set_header Connection "";
    }
}
```

**Connection draining behavior:**
1. Backend is marked unhealthy (after 3 failures)
2. Nginx stops sending new requests to that backend
3. Existing requests on that backend complete normally
4. After `fail_timeout` (30s), Nginx retries the backend
5. If the backend responds, it is marked healthy and receives traffic again

### 3. `down` vs Removing from Upstream

| Action | Behavior | Use Case |
|--------|----------|----------|
| `server ... down;` | Permanently removes from pool. No traffic, no retries. Server remains in config. | Planned maintenance, known unhealthy server |
| Remove from config | Server is gone. Requires config reload. | Permanent decommission |
| `max_fails` removal | Temporary removal. Automatically retried after `fail_timeout`. | Transient failures, auto-recovery |

```nginx
# Planned maintenance: mark as down
upstream api_backend {
    server 10.0.1.1:8080;
    server 10.0.1.2:8080 down;  # In maintenance
    server 10.0.1.3:8080;
}

# After maintenance: remove "down" and reload
# nginx -s reload
```

## Part D: Rate Limiting

```nginx
http {
    # Define rate limit zone
    # $binary_remote_addr: client IP (binary, 16 bytes for IPv4)
    # zone=api_limit:10m: shared memory zone, 10MB (~160K IPs)
    # rate=100r/s: 100 requests per second per IP
    limit_req_zone $binary_remote_addr zone=api_limit:10m rate=100r/s;

    # Define response for rate-limited requests
    limit_req_status 429;

    server {
        listen 443 ssl;
        server_name api.company.com;

        # Health check: exempt from rate limiting
        location /health {
            limit_req off;
            return 200 "OK\n";
        }

        # API: rate-limited
        location /api/ {
            # burst=200: allow burst of 200 requests above the rate
            # nodelay: process burst requests immediately (don't queue)
            limit_req zone=api_limit burst=200 nodelay;

            # Custom 429 response
            error_page 429 = @rate_limited;

            proxy_pass http://api_backend;
        }

        location @rate_limited {
            default_type application/json;
            return 429 '{"error": "rate_limit_exceeded", "retry_after": 1}';
        }
    }
}
```

### Rate Limiting Behavior

```
Normal traffic (below 100 req/s):
  All requests pass through immediately.

Burst traffic (100-300 req/s):
  First 100 req/s: pass through (within rate limit)
  Next 200 req: pass through (burst allowance, nodelay)
  Beyond 300: rejected with HTTP 429

Sustained overload (>100 req/s for extended period):
  Burst tokens are consumed and not replenished fast enough
  Requests beyond 100 req/s are rejected with HTTP 429
```

### Common Mistakes to Avoid

- **Not exempting health checks from rate limiting.** If health checks
  count toward the rate limit, a health checker polling every second
  consumes 1 req/s of the client's budget. Always exempt `/health`.
- **Using `limit_req` without `burst`.** Without burst, any request
  above the rate limit is immediately rejected. This causes failures
  during normal traffic spikes. Always set a burst value.
- **Setting `max_fails` too low.** A single timeout (e.g., slow query)
  should not mark a server as unhealthy. Use `max_fails=3` to require
  multiple consecutive failures.
- **Forgetting about keep-alive connections.** Nginx reuses connections
  to backends via keepalive. If a backend goes down, existing keepalive
  connections may fail. Use `proxy_next_upstream` to retry on failure.
- **Not monitoring rate-limited requests.** If legitimate users are being
  rate-limited, you need to know. Log 429 responses and set up alerts.

## Key Takeaway

A production load balancer is a critical control point that handles far more
than traffic distribution. SSL termination offloads crypto from backends.
Health checks detect and remove failed servers. Connection draining allows
graceful removal. Rate limiting protects backends from overload. Each feature
requires careful configuration -- wrong thresholds cause either false positives
(unhealthy servers removed unnecessarily) or false negatives (actually
unhealthy servers still receiving traffic).
