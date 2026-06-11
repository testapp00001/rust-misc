# Module 19: Load Balancing

> **Previous module (18):** Horizontal vs vertical scaling — running multiple instances of your application.
> **Limitation:** Multiple instances need a way to distribute incoming traffic intelligently.
> **This module:** Load balancing algorithms, Nginx, HAProxy, health checks, and production patterns.

---

## 1. The Problem

You have 4 application instances running. A user sends a request. Which instance handles it?

If all requests go to one instance, you have not scaled at all — you just have 3 idle servers. You need a mechanism to spread traffic evenly, detect failures, and route around broken instances.

Without load balancing, horizontal scaling is useless.

---

## 2. The Naive Way — DNS Round Robin

The simplest approach: return multiple IP addresses for your domain.

```
example.com  →  A  10.0.0.1
             →  A  10.0.0.2
             →  A  10.0.0.3
```

DNS rotates which IP is returned. Clients connect to different servers.

**Why it fails:**

1. **No health checks.** If `10.0.0.2` crashes, DNS keeps sending traffic to it. Users see errors for 1 in 3 requests.

2. **DNS caching.** Clients cache DNS responses. A client might keep hitting the same dead server for hours until the TTL expires.

3. **No control over distribution.** DNS has no concept of "this server is overloaded, send less traffic there."

4. **Uneven distribution.** Some clients get the first IP, some get the second. No guarantee of even spread.

5. **No session awareness.** Cannot route related requests to the same backend.

DNS round robin is not load balancing. It is random IP selection with no feedback loop.

---

## 3. The Right Way — A Dedicated Load Balancer

A load balancer sits between clients and your application instances. It receives every request and forwards it to a healthy instance based on a configured algorithm.

```
         ┌──────────┐
         │  Client  │
         └────┬─────┘
              │
              v
      ┌───────────────┐
      │ Load Balancer │
      └───────┬───────┘
              │
    ┌─────────┼─────────┐
    │         │         │
┌───┴───┐ ┌──┴──┐ ┌───┴───┐
│ App 1 │ │App 2│ │ App 3 │
└───────┘ └─────┘ └───────┘
```

### L4 vs L7 Load Balancing

**Layer 4 (Transport Layer):**
```
Client → Load Balancer → Backend

Load balancer sees: IP address, port, TCP/UDP
Load balancer does NOT see: HTTP headers, URL path, cookies

- Faster (less processing)
- Forwards raw TCP connections
- Cannot make routing decisions based on content
- Example: AWS NLB, HAProxy (TCP mode)
```

**Layer 7 (Application Layer):**
```
Client → Load Balancer → Backend

Load balancer sees: HTTP method, URL path, headers, cookies, body
Load balancer CAN: route by path, inspect headers, modify requests

- Slower (must parse HTTP)
- Can do path-based routing (/api → backend, / → frontend)
- Can cache responses
- Can compress/modify content
- Example: Nginx, HAProxy (HTTP mode), AWS ALB
```

### Load Balancing Algorithms

**Round Robin:**
```
Request 1 → App 1
Request 2 → App 2
Request 3 → App 3
Request 4 → App 1  (cycle repeats)

Good for: equal-capacity servers
Bad for: mixed-capacity servers, long-lived connections
```

**Weighted Round Robin:**
```
App 1 (weight 5): gets 5 out of every 10 requests
App 2 (weight 3): gets 3 out of every 10 requests
App 3 (weight 2): gets 2 out of every 10 requests

Good for: mixed-capacity servers
Bad for: doesn't account for current load
```

**Least Connections:**
```
App 1: 5 active connections
App 2: 2 active connections  ← next request goes here
App 3: 5 active connections

Good for: varied request durations
Bad for: requires tracking connections (more overhead)
```

**IP Hash:**
```
hash(client_ip) % num_servers = assigned server

Client A (192.168.1.10) → always goes to App 2
Client B (192.168.1.20) → always goes to App 1

Good for: session affinity without cookies
Bad for: uneven distribution if clients come from few IPs (NAT)
```

### Health Checks

The load balancer must detect unhealthy instances.

```
Active health check:
  LB → GET /health → App 1 → 200 OK   ✓ healthy
  LB → GET /health → App 2 → 200 OK   ✓ healthy
  LB → GET /health → App 3 → timeout   ✗ unhealthy!
  LB stops sending traffic to App 3

Passive health check:
  LB monitors real responses
  App 3 returns 500 errors → mark unhealthy
  No extra traffic needed for checks
```

---

## 4. The Production Way

### Nginx Load Balancer Configuration

```nginx
http {
    upstream app_backend {
        # Algorithm: least connections
        least_conn;

        # Backends with weights
        server app1:5000 weight=3;
        server app2:5000 weight=2;
        server app3:5000 weight=1;

        # Health check parameters
        # (commercial Nginx Plus has active health checks;
        #  open-source uses max_fails and fail_timeout)
        server app1:5000 max_fails=3 fail_timeout=30s;
        server app2:5000 max_fails=3 fail_timeout=30s;
        server app3:5000 max_fails=3 fail_timeout=30s;

        # Backup server (only used when all others are down)
        server app_backup:5000 backup;

        # Keepalive connections to backends
        keepalive 32;
    }

    server {
        listen 80;

        location / {
            proxy_pass http://app_backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;

            # Timeouts
            proxy_connect_timeout 5s;
            proxy_read_timeout 60s;
            proxy_send_timeout 60s;

            # Retry on failure
            proxy_next_upstream error timeout http_502 http_503;
            proxy_next_upstream_tries 3;
        }
    }
}
```

### HAProxy Load Balancer Configuration

```
global
    maxconn 4096

defaults
    mode http
    timeout connect 5s
    timeout client 30s
    timeout server 30s
    option httplog
    option dontlognull

frontend http_front
    bind *:80
    default_backend app_servers

backend app_servers
    balance leastconn

    # Active health checks
    option httpchk GET /health
    http-check expect status 200

    server app1 app1:5000 check inter 5s fall 3 rise 2 weight 3
    server app2 app2:5000 check inter 5s fall 3 rise 2 weight 2
    server app3 app3:5000 check inter 5s fall 3 rise 2 weight 1

    # Connection draining on failure
    option redispatch
    retries 3
```

**Key HAProxy features:**
- `check inter 5s` — health check every 5 seconds
- `fall 3` — mark unhealthy after 3 consecutive failures
- `rise 2` — mark healthy after 2 consecutive successes
- `weight` — traffic distribution ratio
- `option redispatch` — retry on a different server if one fails

### Sticky Sessions

Sometimes you need the same client to hit the same backend.

```
# Nginx sticky sessions (commercial/Plus)
upstream app_backend {
    sticky cookie srv_id expires=1h domain=.example.com path=/;
    server app1:5000;
    server app2:5000;
}

# HAProxy sticky sessions (open source)
backend app_servers
    balance roundrobin
    cookie SERVERID insert indirect nocache
    server app1 app1:5000 check cookie app1
    server app2 app2:5000 check cookie app2
```

**Warning:** Sticky sessions are a band-aid for stateful applications. The real fix is making your app stateless (covered in Module 21).

### SSL Termination at Load Balancer

```
Client ──HTTPS──> Load Balancer ──HTTP──> App 1
                  (terminates SSL)       (plain HTTP)
                  ──HTTP──> App 2
                  ──HTTP──> App 3
```

Benefits:
- SSL certificate management in one place
- Offloads CPU-intensive encryption from app servers
- App servers only need to handle HTTP
- One certificate covers all backends

```nginx
server {
    listen 443 ssl;
    server_name example.com;

    ssl_certificate /etc/ssl/certs/example.com.pem;
    ssl_certificate_key /etc/ssl/private/example.com.key;
    ssl_protocols TLSv1.2 TLSv1.3;

    location / {
        proxy_pass http://app_backend;  # Plain HTTP to backends
        proxy_set_header X-Forwarded-Proto https;
    }
}

# Redirect HTTP to HTTPS
server {
    listen 80;
    server_name example.com;
    return 301 https://$host$request_uri;
}
```

---

## 5. Hands-On Lab

### Lab: Nginx Load Balancing 3 App Instances

**docker-compose.yml:**
```yaml
version: "3.8"

services:
  app1:
    image: nginx:alpine
    volumes:
      - ./app1-index.html:/usr/share/nginx/html/index.html:ro

  app2:
    image: nginx:alpine
    volumes:
      - ./app2-index.html:/usr/share/nginx/html/index.html:ro

  app3:
    image: nginx:alpine
    volumes:
      - ./app3-index.html:/usr/share/nginx/html/index.html:ro

  loadbalancer:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx-lb.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      - app1
      - app2
      - app3
```

**app1-index.html:**
```html
<h1>Handled by App 1</h1>
```

**app2-index.html:**
```html
<h1>Handled by App 2</h1>
```

**app3-index.html:**
```html
<h1>Handled by App 3</h1>
```

**nginx-lb.conf:**
```nginx
events {
    worker_connections 1024;
}

http {
    upstream app_servers {
        least_conn;
        server app1:80;
        server app2:80;
        server app3:80;
    }

    server {
        listen 80;

        location / {
            proxy_pass http://app_servers;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }
    }
}
```

Run the lab:

```bash
# Start all services
docker compose up -d

# Send requests and see which instance handles each
for i in $(seq 1 9); do
  echo "Request $i: $(curl -s http://localhost)"
done
```

Expected output:
```
Request 1: Handled by App 1
Request 2: Handled by App 2
Request 3: Handled by App 3
Request 4: Handled by App 1
Request 5: Handled by App 2
Request 6: Handled by App 3
Request 7: Handled by App 1
Request 8: Handled by App 2
Request 9: Handled by App 3
```

### Test Health Check Behavior

```bash
# Stop one backend
docker compose stop app2

# Send requests — traffic routes to app1 and app3 only
for i in $(seq 1 6); do
  echo "Request $i: $(curl -s http://localhost)"
done

# Restart app2
docker compose start app2

# Traffic returns to all three
for i in $(seq 1 6); do
  echo "Request $i: $(curl -s http://localhost)"
done
```

---

## 6. Limitation

Load balancing distributes traffic across multiple instances of the same service. But real applications are made of multiple services: a frontend, an API, a database, maybe a microservice or two.

How do you route `/api/users` to the user service and `/` to the frontend, all through one entry point? How do you handle SSL, rate limiting, and caching in one place?

The load balancer is just the beginning. You need a reverse proxy that can route to different services.

---

## 7. Next Topic

**Module 20: Reverse Proxy** — Nginx, Traefik, and the gateway pattern. Single entry point that routes to multiple different services. [Go to Module 20 →](../20-reverse-proxy/README.md)
