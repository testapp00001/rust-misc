# Exercise 05: Multi-Layer Load Balancing Architecture (Integration)

## Objective

Design and implement a two-layer load-balancing architecture: an external L7
(HTTP) load balancer in front of an internal L4 (TCP) load balancer, which
distributes traffic across application servers.

## Prerequisites

- Docker and Docker Compose
- Understanding of L4 vs L7 load balancing
- Completion of Exercises 01-04

## Background

In real-world architectures, a single load balancer is rarely enough. A common
pattern is:

```
Internet --> L7 LB (Nginx/HAProxy) --> L4 LB (HAProxy TCP mode) --> App Servers
```

- The **L7 (Layer 7)** load balancer understands HTTP. It can route based on
  URL path, host header, cookies, etc. It terminates TLS.
- The **L4 (Layer 4)** load balancer operates at the TCP level. It is faster
  (no HTTP parsing) and is often used for raw TCP services like databases or
  internal microservice traffic.

## Instructions

### Step 1 -- Design the architecture

Before writing any code, draw (on paper or in a text diagram) the following
architecture:

```
                    +-----------------+
   Client -------->|  L7 LB (Nginx)  |  port 8080 on host
                    +--------+--------+
                             |
              +--------------+--------------+
              |                             |
     +--------v--------+          +--------v--------+
     |  L4 LB (HAProxy)|          |  L4 LB (HAProxy)|
     |  (TCP, port 80) |          |  (TCP, port 80) |
     +--------+--------+          +--------+--------+
              |                             |
     +--------+--------+          +--------+--------+
     | App Server 1    |          | App Server 3    |
     | App Server 2    |          | App Server 4    |
     +-----------------+          +-----------------+
```

Label each component with its purpose and protocol.

### Step 2 -- Create the project

```
mkdir -p ~/lb-exercise-05 && cd ~/lb-exercise-05
```

Create the following files:

```
.
+-- docker-compose.yml
+-- nginx-l7.conf
+-- haproxy-l4-1.cfg
+-- haproxy-l4-2.cfg
```

### Step 3 -- Define the app servers

In `docker-compose.yml`, create four app server containers using
`hashicorp/http-echo:0.2.3`:

| Container     | Response text       |
|---------------|---------------------|
| `app-1`       | "Hello from app-1"  |
| `app-2`       | "Hello from app-2"  |
| `app-3`       | "Hello from app-3"  |
| `app-4`       | "Hello from app-4"  |

All listen on port `5678`.

### Step 4 -- Create the L4 load balancers

Create two HAProxy containers (`l4-lb-1` and `l4-lb-2`) using the image
`haproxy:2.9-alpine`.

**`haproxy-l4-1.cfg`** should:

- Listen on port `80` in TCP mode.
- Balance traffic across `app-1` and `app-2` using the `roundrobin` algorithm.

**`haproxy-l4-2.cfg`** should:

- Listen on port `80` in TCP mode.
- Balance traffic across `app-3` and `app-4` using the `roundrobin` algorithm.

Minimal HAProxy TCP config example:

```haproxy
global
    log stdout format raw local0

defaults
    mode tcp
    timeout connect 5s
    timeout client  30s
    timeout server  30s

frontend fe_tcp
    bind *:80
    default_backend be_apps

backend be_apps
    balance roundrobin
    server app-1 app-1:5678 check
    server app-2 app-2:5678 check
```

### Step 5 -- Create the L7 load balancer

Create `nginx-l7.conf` with:

- An `upstream` block listing the two L4 load balancers (`l4-lb-1:80` and
  `l4-lb-2:80`).
- A `server` block listening on port `80` that proxies to the upstream.
- Add a custom header so you can see which L4 LB handled the request:

```nginx
proxy_set_header X-L4-LB $upstream_addr;
```

### Step 6 -- Wire it all together

In `docker-compose.yml`:

- The L7 Nginx container (`l7-lb`) maps host port `8080` to container port
  `80`.
- The L4 HAProxy containers expose port `80` internally (no host mapping
  needed).
- Define a custom network so all containers can communicate.
- Set `depends_on` so the L7 LB starts after the L4 LBs, and the L4 LBs
  start after the app servers.

### Step 7 -- Verify the full path

```bash
docker compose up -d

# Should return responses from all four app servers
for i in $(seq 1 16); do
  echo "Request $i: $(curl -s http://localhost:8080)"
done
```

You should see responses from `app-1` through `app-4`, confirming that:

1. The L7 LB distributes requests across both L4 LBs.
2. Each L4 LB distributes requests across its two app servers.

### Step 8 -- Observe failure behaviour

Stop `app-2`:

```bash
docker compose stop app-2
```

Send another burst of requests. Verify that:

- `l4-lb-1` stops sending traffic to `app-2` (HAProxy health check fails).
- Traffic still reaches `app-1`, `app-3`, and `app-4`.
- The L7 LB still distributes across both L4 LBs.

### Step 9 -- Write an architecture document

Create `architecture.md` that includes:

1. Your text diagram of the architecture.
2. A description of the role of each layer.
3. What happens when a single app server fails.
4. What happens when an entire L4 LB fails.
5. How you would add TLS termination at the L7 layer.
6. How you would extend this to support path-based routing (e.g., `/api` goes
   to one set of backends, `/static` goes to another).

## Success Criteria

- [ ] All eight containers (4 app servers, 2 L4 LBs, 1 L7 LB) start without
      errors.
- [ ] `curl` requests return responses from all four app servers.
- [ ] Stopping an app server causes traffic to shift to the remaining three.
- [ ] `architecture.md` contains a clear diagram and answers all six questions.
- [ ] The Nginx config uses an `upstream` block with the L4 LBs as members.
- [ ] The HAProxy configs use TCP mode (not HTTP mode).

## Hints

<details>
<summary>Hint 1 -- Docker networking</summary>

Define a custom bridge network in `docker-compose.yml`:

```yaml
networks:
  lb-net:
    driver: bridge
```

Attach every service to this network. Containers can then reach each other by
service name.

</details>

<details>
<summary>Hint 2 -- HAProxy health checks</summary>

The `check` keyword after each `server` line enables active TCP health checks.
HAProxy probes the backend every 2 seconds by default. When a backend fails
three consecutive checks, it is marked down.

</details>

<details>
<summary>Hint 3 -- Adding TLS at L7</summary>

To add TLS termination in Nginx:

```nginx
server {
    listen 443 ssl;
    ssl_certificate     /etc/nginx/cert.pem;
    ssl_certificate_key /etc/nginx/key.pem;
    ...
}
```

You would generate self-signed certs for testing or use Let's Encrypt for
production. Mount the cert files into the Nginx container.

</details>

<details>
<summary>Hint 4 -- Path-based routing</summary>

In Nginx, use multiple `location` blocks, each with its own `proxy_pass`
pointing to a different upstream:

```nginx
upstream api_backends { ... }
upstream static_backends { ... }

server {
    location /api    { proxy_pass http://api_backends; }
    location /static { proxy_pass http://static_backends; }
}
```

</details>
