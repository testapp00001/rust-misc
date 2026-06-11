# Module 20: Reverse Proxy

> **Previous module (19):** Load balancing — distributing traffic across identical instances.
> **Limitation:** You have multiple different services (frontend, API, auth) but no single entry point that routes intelligently between them.
> **This module:** Reverse proxy patterns with Nginx and Traefik.

---

## 1. The Problem

Your application is no longer a single service. You have:

- A React frontend (static files, served by Nginx)
- A Python API backend (Flask/FastAPI)
- A Node.js WebSocket server
- An authentication microservice

Each runs in its own container on its own port. Users should access everything through one domain: `https://example.com`.

How do you:
- Route `/` to the frontend
- Route `/api/*` to the Python backend
- Route `/ws` to the WebSocket server
- Route `/auth/*` to the auth service
- Handle SSL in one place
- Apply rate limiting globally

You need a reverse proxy.

---

## 2. The Naive Way — Expose Every Service on Its Own Port

```
https://example.com:3000     → Frontend
https://example.com:5000     → API
https://example.com:8080     → WebSocket
https://example.com:4000     → Auth
```

**Why it fails:**

1. **Firewall nightmares.** You need to open ports 3000, 5000, 8080, 4000. More ports = more attack surface.

2. **CORS hell.** The frontend at `:3000` making requests to `:5000` is a cross-origin request. You need CORS headers on every service, and debugging CORS issues is miserable.

3. **No single entry point.** Clients need to know which port maps to which service. Changing a port means updating all clients.

4. **SSL everywhere.** You need SSL certificates for every service, or you run some on HTTP (insecure).

5. **No centralized rate limiting.** Each service must implement its own rate limiting, or one service can be overwhelmed without others knowing.

6. **Ugly URLs.** `https://example.com:5000/api/users` is not a production URL.

---

## 3. The Right Way — Reverse Proxy

A reverse proxy sits in front of all your services. Clients connect only to the proxy. The proxy routes requests to the correct backend based on rules you define.

```
                          ┌──────────────────────┐
                          │     Reverse Proxy     │
                          │   (Nginx / Traefik)   │
                          │   Port 80 and 443     │
                          └───────────┬───────────┘
                                      │
            ┌─────────────────────────┼─────────────────────────┐
            │                         │                         │
            v                         v                         v
    ┌───────────────┐       ┌───────────────┐       ┌───────────────┐
    │   Frontend    │       │   API Server  │       │  WebSocket    │
    │  (React app)  │       │   (FastAPI)   │       │   Server      │
    │   Port 3000   │       │   Port 5000   │       │   Port 8080   │
    └───────────────┘       └───────────────┘       └───────────────┘

Client sees only: https://example.com
```

### Forward Proxy vs Reverse Proxy

```
FORWARD PROXY (client-side):
  Client → Proxy → Internet

  - Client knows it's using a proxy
  - Proxy hides the client from the internet
  - Use case: corporate firewalls, content filtering, privacy
  - The client configures the proxy

REVERSE PROXY (server-side):
  Client → Proxy → Backend servers

  - Client does NOT know it's a proxy
  - Proxy hides the backend servers from the client
  - Use case: load balancing, SSL termination, routing
  - The server configures the proxy
```

---

## 4. The Production Way

### Nginx Reverse Proxy — Path-Based Routing

```nginx
events {
    worker_connections 1024;
}

http {
    # Frontend upstream
    upstream frontend {
        server frontend:3000;
    }

    # API upstream (could be load balanced)
    upstream api {
        server api:5000;
    }

    # WebSocket upstream
    upstream websocket {
        server ws_server:8080;
    }

    server {
        listen 80;
        server_name example.com;

        # Frontend: serve static files or proxy to frontend server
        location / {
            proxy_pass http://frontend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }

        # API: route /api/* to backend
        location /api/ {
            proxy_pass http://api;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;

            # Strip /api prefix if backend expects /
            # proxy_pass http://api/;
        }

        # WebSocket: route /ws/* to websocket server
        location /ws/ {
            proxy_pass http://websocket;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_set_header Host $host;
            proxy_read_timeout 86400;  # Keep WS connection open
        }
    }
}
```

### Nginx Reverse Proxy — Host-Based Routing

```nginx
http {
    server {
        listen 80;
        server_name api.example.com;

        location / {
            proxy_pass http://api_backend;
        }
    }

    server {
        listen 80;
        server_name web.example.com;

        location / {
            proxy_pass http://frontend;
        }
    }

    server {
        listen 80;
        server_name admin.example.com;

        location / {
            proxy_pass http://admin_panel;
        }
    }
}
```

### Traefik — Auto-Discovery with Docker

Traefik automatically discovers services from Docker labels. No manual configuration files.

**docker-compose.yml:**
```yaml
version: "3.8"

services:
  traefik:
    image: traefik:v3.0
    command:
      - "--providers.docker=true"
      - "--providers.docker.exposedbydefault=false"
      - "--entrypoints.web.address=:80"
      - "--entrypoints.websecure.address=:443"
      - "--api.dashboard=true"
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.dashboard.rule=Host(`traefik.example.com`)"
      - "traefik.http.routers.dashboard.service=api@internal"

  frontend:
    image: myapp-frontend:latest
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.frontend.rule=Host(`example.com`) && PathPrefix(`/`)"
      - "traefik.http.services.frontend.loadbalancer.server.port=3000"

  api:
    image: myapp-api:latest
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.api.rule=Host(`example.com`) && PathPrefix(`/api`)"
      - "traefik.http.services.api.loadbalancer.server.port=5000"

  websocket:
    image: myapp-ws:latest
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.ws.rule=Host(`example.com`) && PathPrefix(`/ws`)"
      - "traefik.http.services.ws.loadbalancer.server.port=8080"
      - "traefik.http.routers.ws.service=ws"
```

**Why Traefik is powerful:**
- Add a new service, add labels, Traefik picks it up automatically
- Built-in Let's Encrypt integration for automatic SSL
- Built-in dashboard for monitoring
- Supports Docker, Kubernetes, Consul, etcd, and more

### SSL/TLS Termination

```nginx
server {
    listen 443 ssl;
    server_name example.com;

    ssl_certificate /etc/letsencrypt/live/example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/example.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    # HSTS — force browsers to use HTTPS
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;

    location / {
        proxy_pass http://frontend;
    }

    location /api/ {
        proxy_pass http://api;
    }
}

# Redirect all HTTP to HTTPS
server {
    listen 80;
    server_name example.com;
    return 301 https://$host$request_uri;
}
```

### Rate Limiting at Proxy Layer

```nginx
http {
    # Define rate limit zones
    limit_req_zone $binary_remote_addr zone=api_limit:10m rate=10r/s;
    limit_req_zone $binary_remote_addr zone=login_limit:10m rate=1r/s;

    server {
        listen 443 ssl;

        # API: 10 requests per second per IP
        location /api/ {
            limit_req zone=api_limit burst=20 nodelay;
            proxy_pass http://api;
        }

        # Login: 1 request per second per IP (brute force protection)
        location /api/auth/login {
            limit_req zone=login_limit burst=5 nodelay;
            proxy_pass http://api;
        }

        # Frontend: no rate limit (static files)
        location / {
            proxy_pass http://frontend;
        }
    }
}
```

### Caching at Proxy Layer

```nginx
http {
    proxy_cache_path /var/cache/nginx levels=1:2 keys_zone=my_cache:10m
                     max_size=10g inactive=60m use_temp_path=off;

    server {
        listen 80;

        # Cache API responses for 5 minutes
        location /api/products {
            proxy_pass http://api;
            proxy_cache my_cache;
            proxy_cache_valid 200 5m;
            proxy_cache_valid 404 1m;
            add_header X-Cache-Status $upstream_cache_status;
        }

        # Cache static assets for 1 year
        location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg)$ {
            proxy_pass http://frontend;
            proxy_cache my_cache;
            proxy_cache_valid 200 365d;
            expires 1y;
            add_header Cache-Control "public, immutable";
        }

        # Don't cache dynamic content
        location /api/auth {
            proxy_pass http://api;
            proxy_cache off;
        }
    }
}
```

### WebSocket Proxying

WebSocket requires special handling because it upgrades from HTTP to a persistent bidirectional connection.

```nginx
location /ws/ {
    proxy_pass http://websocket_backend;
    proxy_http_version 1.1;

    # These two headers are required for WebSocket upgrade
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";

    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;

    # Long timeout — WebSocket connections can be idle for hours
    proxy_read_timeout 3600s;
    proxy_send_timeout 3600s;
}
```

---

## 5. Hands-On Lab

### Lab: Nginx Reverse Proxy for a Full-Stack App

**docker-compose.yml:**
```yaml
version: "3.8"

services:
  frontend:
    image: nginx:alpine
    volumes:
      - ./frontend:/usr/share/nginx/html:ro

  api:
    build: ./api
    expose:
      - "5000"

  proxy:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      - frontend
      - api
```

**frontend/index.html:**
```html
<!DOCTYPE html>
<html>
<body>
    <h1>Frontend</h1>
    <button onclick="callApi()">Call API</button>
    <pre id="result"></pre>
    <script>
        async function callApi() {
            const res = await fetch('/api/hello');
            const data = await res.json();
            document.getElementById('result').textContent = JSON.stringify(data, null, 2);
        }
    </script>
</body>
</html>
```

**api/app.py:**
```python
from flask import Flask, jsonify
import os

app = Flask(__name__)

@app.route('/api/hello')
def hello():
    return jsonify({
        "message": "Hello from API",
        "hostname": os.environ.get('HOSTNAME', 'unknown')
    })

@app.route('/api/health')
def health():
    return jsonify({"status": "ok"})

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

**api/Dockerfile:**
```dockerfile
FROM python:3.11-slim
WORKDIR /app
RUN pip install flask
COPY app.py .
CMD ["python", "app.py"]
```

**nginx.conf:**
```nginx
events {
    worker_connections 1024;
}

http {
    upstream frontend {
        server frontend:80;
    }

    upstream api {
        server api:5000;
    }

    server {
        listen 80;

        # Frontend serves static files
        location / {
            proxy_pass http://frontend;
        }

        # API routes go to backend
        location /api/ {
            proxy_pass http://api;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }
    }
}
```

Run the lab:

```bash
# Create directories and files as shown above
mkdir -p frontend api

# Start everything
docker compose up -d

# Test frontend
curl http://localhost/
# Returns the HTML page

# Test API through the proxy
curl http://localhost/api/hello
# Returns: {"hostname":"abc123","message":"Hello from API"}

# Test health check
curl http://localhost/api/health
# Returns: {"status":"ok"}

# The client only sees port 80 — all routing happens in the proxy
```

---

## 6. Limitation

The reverse proxy routes traffic to the right service. But load balancing across multiple instances creates a subtle problem: **sessions**.

If a user logs in on App Instance 1, their session data is stored in that instance's memory. The next request might be routed to Instance 2, which has no knowledge of that session. The user appears logged out.

Sticky sessions are a band-aid. The real solution requires rethinking how you store user state.

---

## 7. Next Topic

**Module 21: Session Management** — Sticky sessions, shared state, and designing stateless applications that work with load balancers. [Go to Module 21 →](../21-session-management/README.md)
