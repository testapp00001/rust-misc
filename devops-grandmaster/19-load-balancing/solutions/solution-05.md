# Solution 05: Multi-Layer Load Balancing Architecture

## Complete Answer

### Architecture Diagram

```
                         INTERNET
                            |
                            v
                   +------------------+
                   |   L7 LB (Nginx)  |
                   |  host port 8080  |
                   |  HTTP, path-aware|
                   +--------+---------+
                            |
              +-------------+-------------+
              |                           |
     +--------v---------+       +--------v---------+
     | L4 LB 1 (HAProxy)|       | L4 LB 2 (HAProxy)|
     |   TCP mode, :80  |       |   TCP mode, :80  |
     |  bal: roundrobin |       |  bal: roundrobin |
     +---+----------+---+       +---+----------+---+
         |          |               |          |
    +----v---+ +----v---+     +----v---+ +----v---+
    | app-1  | | app-2  |     | app-3  | | app-4  |
    | :5678  | | :5678  |     | :5678  | | :5678  |
    +--------+ +--------+     +--------+ +--------+
```

### Project structure

```
~/lb-exercise-05/
+-- docker-compose.yml
+-- nginx-l7.conf
+-- haproxy-l4-1.cfg
+-- haproxy-l4-2.cfg
```

### nginx-l7.conf

```nginx
events {
    worker_connections 1024;
}

http {
    upstream l4_pool {
        server l4-lb-1:80;
        server l4-lb-2:80;
    }

    server {
        listen 80;

        location / {
            proxy_pass http://l4_pool;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-L4-Addr $upstream_addr;
        }
    }
}
```

### haproxy-l4-1.cfg

```haproxy
global
    log stdout format raw local0

defaults
    mode tcp
    timeout connect 5s
    timeout client  30s
    timeout server  30s
    log global

frontend fe_tcp
    bind *:80
    default_backend be_apps

backend be_apps
    balance roundrobin
    option tcp-check
    server app-1 app-1:5678 check inter 2s fall 3 rise 2
    server app-2 app-2:5678 check inter 2s fall 3 rise 2
```

### haproxy-l4-2.cfg

```haproxy
global
    log stdout format raw local0

defaults
    mode tcp
    timeout connect 5s
    timeout client  30s
    timeout server  30s
    log global

frontend fe_tcp
    bind *:80
    default_backend be_apps

backend be_apps
    balance roundrobin
    option tcp-check
    server app-3 app-3:5678 check inter 2s fall 3 rise 2
    server app-4 app-4:5678 check inter 2s fall 3 rise 2
```

### docker-compose.yml

```yaml
services:
  # --- Application Servers ---
  app-1:
    image: hashicorp/http-echo:0.2.3
    command: ["-text=Hello from app-1", "-listen=:5678"]
    networks:
      - lb-net

  app-2:
    image: hashicorp/http-echo:0.2.3
    command: ["-text=Hello from app-2", "-listen=:5678"]
    networks:
      - lb-net

  app-3:
    image: hashicorp/http-echo:0.2.3
    command: ["-text=Hello from app-3", "-listen=:5678"]
    networks:
      - lb-net

  app-4:
    image: hashicorp/http-echo:0.2.3
    command: ["-text=Hello from app-4", "-listen=:5678"]
    networks:
      - lb-net

  # --- Layer 4 Load Balancers ---
  l4-lb-1:
    image: haproxy:2.9-alpine
    volumes:
      - ./haproxy-l4-1.cfg:/usr/local/etc/haproxy/haproxy.cfg:ro
    depends_on:
      - app-1
      - app-2
    networks:
      - lb-net

  l4-lb-2:
    image: haproxy:2.9-alpine
    volumes:
      - ./haproxy-l4-2.cfg:/usr/local/etc/haproxy/haproxy.cfg:ro
    depends_on:
      - app-3
      - app-4
    networks:
      - lb-net

  # --- Layer 7 Load Balancer ---
  l7-lb:
    image: nginx:1.25-alpine
    ports:
      - "8080:80"
    volumes:
      - ./nginx-l7.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      - l4-lb-1
      - l4-lb-2
    networks:
      - lb-net

networks:
  lb-net:
    driver: bridge
```

### Verification output

```bash
$ docker compose up -d

$ for i in $(seq 1 16); do
    echo "Request $i: $(curl -s http://localhost:8080)"
  done
Request 1: Hello from app-1
Request 2: Hello from app-3
Request 3: Hello from app-2
Request 4: Hello from app-4
Request 5: Hello from app-1
Request 6: Hello from app-3
Request 7: Hello from app-2
Request 8: Hello from app-4
Request 9: Hello from app-1
Request 10: Hello from app-3
Request 11: Hello from app-2
Request 12: Hello from app-4
Request 13: Hello from app-1
Request 14: Hello from app-3
Request 15: Hello from app-2
Request 16: Hello from app-4
```

The pattern shows the L7 LB alternating between the two L4 LBs (Round Robin),
and each L4 LB alternating between its two app servers.

### Failure test

```bash
$ docker compose stop app-2

$ for i in $(seq 1 12); do
    echo "Request $i: $(curl -s http://localhost:8080)"
  done
Request 1: Hello from app-1
Request 2: Hello from app-3
Request 3: Hello from app-1
Request 4: Hello from app-4
Request 5: Hello from app-1
Request 6: Hello from app-3
Request 7: Hello from app-1
Request 8: Hello from app-4
Request 9: Hello from app-1
Request 10: Hello from app-3
Request 11: Hello from app-1
Request 12: Hello from app-4
```

After `app-2` stops, HAProxy health checks detect the failure within a few
seconds. L4-LB-1 stops sending traffic to `app-2` and routes all requests to
`app-1`. The L7 LB continues distributing across both L4 LBs. No requests
fail.

## Why It Works

### Layer separation

1. **L7 LB (Nginx)** understands HTTP. It can inspect headers, URLs, and
   cookies. It is the entry point where TLS termination, path-based routing,
   rate limiting, and request rewriting happen. It distributes traffic across
   multiple L4 LBs for horizontal scalability.

2. **L4 LB (HAProxy in TCP mode)** does not parse HTTP. It forwards raw TCP
   packets, making it faster per request than L7. It is ideal for the
   internal hop where you only need simple distribution. HAProxy's TCP health
   checks (`option tcp-check`) verify that the backend port is accepting
   connections.

3. **App servers** are the actual application. They are unaware of the load
   balancers -- they just receive TCP connections and respond.

### Why two L4 LBs instead of one?

A single L4 LB is a single point of failure and a throughput bottleneck. Two
L4 LBs provide:

- **Redundancy**: If one fails, the L7 LB routes around it.
- **Capacity**: Traffic is split, doubling the internal throughput.
- **Isolation**: Each L4 LB can serve a different subset of backends,
  enabling blue-green or canary deployments at the L4 layer.

### HAProxy health checks

The `check inter 2s fall 3 rise 2` directive means:

- Probe the backend every 2 seconds.
- After 3 consecutive failures, mark the backend as DOWN.
- After 2 consecutive successes (once it comes back), mark it as UP.

This is an active health check -- HAProxy sends TCP probes independent of
client traffic.

## Common Mistakes

### 1. Running HAProxy in HTTP mode instead of TCP mode

```haproxy
# WRONG -- this parses HTTP headers, adding overhead for a TCP pass-through
defaults
    mode http

# CORRECT -- raw TCP forwarding
defaults
    mode tcp
```

### 2. Not using a custom Docker network

Without a custom network, containers may end up on different default networks
and cannot resolve each other by service name. Always define an explicit
network.

### 3. Putting port mappings on internal services

The L4 LBs and app servers do not need `ports:` mappings on the host. Only
the L7 LB (the entry point) needs `ports: ["8080:80"]`. Exposing internal
ports on the host wastes port numbers and is a security risk.

### 4. Forgetting health checks on HAProxy backends

Without the `check` keyword, HAProxy sends traffic to all listed servers
regardless of their state. If an app server crashes, HAProxy will keep trying
to connect and clients will see errors. Always add `check` after each server
line.

### 5. Circular dependency in depends_on

Ensure the dependency chain is acyclic: L7 depends on L4 LBs, L4 LBs depend
on app servers. Never create a cycle (e.g., L7 depending on app servers
directly while also depending on L4 LBs that depend on the same app servers).

### 6. Missing `option tcp-check` in HAProxy

By default, HAProxy performs a TCP connect check (just opens a socket). For
more robust checking, add `option tcp-check` which sends additional probes.
For HTTP backends, use `option httpchk GET /healthz` instead.

## Architecture Document Answers (architecture.md)

### 1. Role of each layer

- **L7 (Nginx)**: HTTP-aware entry point. Terminates TLS, inspects request
  headers/paths, applies routing rules, adds headers, and distributes across
  L4 LBs.
- **L4 (HAProxy)**: Fast TCP-level distribution. Forwards raw connections to
  app servers with active health checks. No HTTP parsing overhead.
- **App servers**: Stateless application instances that process requests.

### 2. Single app server failure

HAProxy detects the failure via TCP health check (within 6 seconds: 3 checks
at 2s intervals). It removes the failed server from rotation. Traffic
continues to flow to the remaining app servers behind that L4 LB. The L7 LB
is unaffected.

### 3. Entire L4 LB failure

If `l4-lb-1` crashes, Nginx detects the failure on the next request attempt.
Its passive health check (`max_fails`/`fail_timeout`) marks `l4-lb-1` as
unavailable. Nginx routes all traffic to `l4-lb-2`, which now serves all four
app servers (assuming its config is updated, or in this exercise, it still
serves `app-3` and `app-4`). The system degrades gracefully: capacity is
halved but service continues.

### 4. Adding TLS termination at L7

```nginx
server {
    listen 443 ssl;
    ssl_certificate     /etc/nginx/cert.pem;
    ssl_certificate_key /etc/nginx/key.pem;

    location / {
        proxy_pass http://l4_pool;
    }
}

server {
    listen 80;
    return 301 https://$host$request_uri;
}
```

Generate self-signed certs for testing:

```bash
openssl req -x509 -nodes -days 365 \
  -newkey rsa:2048 \
  -keyout key.pem \
  -out cert.pem \
  -subj "/CN=localhost"
```

Mount `cert.pem` and `key.pem` into the Nginx container.

### 5. Path-based routing

```nginx
upstream api_pool {
    server l4-lb-1:80;
}

upstream static_pool {
    server l4-lb-2:80;
}

server {
    listen 80;

    location /api {
        proxy_pass http://api_pool;
    }

    location /static {
        proxy_pass http://static_pool;
    }
}
```

This lets you route different URL paths to different backend pools. In
practice, `/api` might go to application servers while `/static` goes to a
CDN or object storage.
