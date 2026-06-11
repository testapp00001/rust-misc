# Solution 03: Health-Check-Aware Load Balancing

## Complete Answer

### nginx.conf (with passive health checks)

```nginx
events {
    worker_connections 1024;
}

http {
    upstream backend_pool {
        server backend-1:5678 max_fails=2 fail_timeout=15s;
        server backend-2:5678 max_fails=2 fail_timeout=15s;
        server backend-3:5678 max_fails=2 fail_timeout=15s;
    }

    server {
        listen 80;

        location / {
            proxy_pass http://backend_pool;
            proxy_next_upstream error timeout http_502 http_503;
        }
    }
}
```

### docker-compose.yml

```yaml
services:
  backend-1:
    image: hashicorp/http-echo:0.2.3
    command: ["-text=Hello from backend-1", "-listen=:5678"]

  backend-2:
    image: hashicorp/http-echo:0.2.3
    command: ["-text=Hello from backend-2", "-listen=:5678"]

  backend-3:
    image: hashicorp/http-echo:0.2.3
    command: ["-text=Hello from backend-3", "-listen=:5678"]

  load-balancer:
    image: nginx:1.25-alpine
    ports:
      - "8080:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      - backend-1
      - backend-2
      - backend-3
```

### Step 3 -- Normal operation output

```
$ for i in $(seq 1 12); do curl -s http://localhost:8080; echo; done
Hello from backend-1
Hello from backend-2
Hello from backend-3
Hello from backend-1
Hello from backend-2
Hello from backend-3
Hello from backend-1
Hello from backend-2
Hello from backend-3
Hello from backend-1
Hello from backend-2
Hello from backend-3
```

All three backends receive traffic in a round-robin pattern.

### Step 4 -- After stopping backend-2

```
$ docker compose stop backend-2

$ for i in $(seq 1 12); do curl -s http://localhost:8080; echo; done
Hello from backend-1
Hello from backend-3
Hello from backend-1
Hello from backend-3
Hello from backend-1
Hello from backend-3
Hello from backend-1
Hello from backend-3
Hello from backend-1
Hello from backend-3
Hello from backend-1
Hello from backend-3
```

Nginx detected that `backend-2` is unreachable. After 2 failed attempts
(`max_fails=2`), it marks the server as unavailable. The first 1-2 requests
to `backend-2` will fail or return an error, but Nginx's `proxy_next_upstream`
directive retries them on the next available server. After that, Nginx routes
directly to `backend-1` and `backend-3` only.

### Step 5 -- After restarting backend-2

```
$ docker compose start backend-2

# Wait 15 seconds (the fail_timeout period)

$ for i in $(seq 1 12); do curl -s http://localhost:8080; echo; done
Hello from backend-1
Hello from backend-2
Hello from backend-3
Hello from backend-1
Hello from backend-2
Hello from backend-3
Hello from backend-1
Hello from backend-2
Hello from backend-3
Hello from backend-1
Hello from backend-2
Hello from backend-3
```

After the `fail_timeout` window (15 seconds) expires, Nginx considers
`backend-2` eligible again. It probes the server with a real client request.
If it responds successfully, it is re-added to the pool. Traffic distribution
returns to normal.

### Passive vs Active Health Checks

**Passive health checks** (what we configured) are reactive: Nginx detects a
failure only when a real client request fails. There is no separate probing
traffic. The `max_fails` parameter sets the failure threshold, and
`fail_timeout` sets both the window for counting failures and the duration
a failed server stays marked as down. The advantage is zero overhead from
probes. The disadvantage is that at least one real client request must fail
before the problem is detected.

**Active health checks** are proactive: the load balancer sends periodic HTTP
(or TCP) requests to a dedicated health endpoint (e.g., `/healthz`) on each
backend, independent of client traffic. If the health check fails, the backend
is removed from the pool *before* any client request is affected. This requires
Nginx Plus (commercial) or the third-party `nginx_upstream_check_module`. The
advantage is faster detection and zero impact on clients. The disadvantage is
slightly more network traffic and the need to maintain a health endpoint on
each backend.

## Why It Works

1. **`max_fails=2`**: Nginx counts failures (connection refused, timeout, or
   HTTP 502/503) on real requests. Once 2 failures accumulate within the
   `fail_timeout` window, the server is marked as unavailable.

2. **`fail_timeout=15s`**: This serves a dual purpose. First, it sets the
   time window in which `max_fails` failures must occur to trigger a
   downtime marking. Second, it sets how long the server stays marked as
   down before Nginx retries it. After 15 seconds, Nginx will tentatively
   send a request to the recovered server.

3. **`proxy_next_upstream error timeout http_502 http_503`**: When a request
   to one backend fails, Nginx automatically retries the same request on the
   next available backend. This means the client does not see the failure
   (as long as at least one backend is healthy). Without this directive,
   Nginx would return the error directly to the client.

4. **Docker Compose `stop`/`start`**: When you stop a container, Docker
   kills the process and closes the port. Nginx's next attempt to connect
   to that port gets "connection refused," which counts as a failure.

## Common Mistakes

### 1. Confusing `fail_timeout` with a request timeout

`fail_timeout` is NOT a request timeout. It is the window for counting
failures and the downtime period. A request timeout is configured separately
with `proxy_read_timeout` and `proxy_connect_timeout`.

### 2. Setting `max_fails` too low

With `max_fails=1`, a single slow request or a brief network hiccup will take
a server out of rotation. This causes unnecessary churn. `max_fails=2` or
`max_fails=3` is more resilient.

### 3. Not configuring `proxy_next_upstream`

Without this directive, when a request hits a failed backend, Nginx returns
the error to the client. The client sees a 502 Bad Gateway even though
healthy backends exist. `proxy_next_upstream` tells Nginx to silently retry
on the next backend.

### 4. Expecting immediate recovery

After `fail_timeout` expires, Nginx does not instantly flood the recovered
server with traffic. It sends a single tentative request. If that succeeds,
the server is re-added. This is gentle, but it means recovery is not
instantaneous -- there is a short period where the server is underutilised.

### 5. Confusing passive with active health checks

Passive checks only detect failures on real traffic. If a backend is returning
HTTP 500 errors but is still reachable, passive checks will not detect the
problem unless you add `http_500` to `proxy_next_upstream`. Active checks would
catch this because they inspect the response status code from a dedicated
health endpoint.

### 6. Forgetting that Nginx OSS does not support active health checks

Many tutorials reference `check` or `health_check` directives. These are
Nginx Plus features. Open-source Nginx only supports passive checks via
`max_fails`/`fail_timeout`. For active checks in open-source Nginx, you need
a third-party module or a sidecar approach.
