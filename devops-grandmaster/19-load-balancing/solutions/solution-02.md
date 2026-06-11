# Solution 02: Nginx as a Load Balancer

## Complete Answer

### nginx.conf

```nginx
events {
    worker_connections 1024;
}

http {
    upstream backend_pool {
        server backend-1:5678;
        server backend-2:5678;
        server backend-3:5678;
    }

    server {
        listen 80;

        location / {
            proxy_pass http://backend_pool;
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

### Verification

```bash
docker compose up -d

# First request
$ curl http://localhost:8080
Hello from backend-1

# Second request
$ curl http://localhost:8080
Hello from backend-2

# Third request
$ curl http://localhost:8080
Hello from backend-3

# Fourth request -- cycles back
$ curl http://localhost:8080
Hello from backend-1
```

### Switching to Least Connections

Add `least_conn;` inside the `upstream` block:

```nginx
upstream backend_pool {
    least_conn;
    server backend-1:5678;
    server backend-2:5678;
    server backend-3:5678;
}
```

Reload:

```bash
docker compose exec load-balancer nginx -s reload
```

Verify it still works with `curl`.

## Why It Works

1. **Docker DNS**: When containers are in the same Docker Compose project,
   Docker creates a network where each container can reach others by their
   service name. Nginx resolves `backend-1`, `backend-2`, `backend-3` via
   Docker's built-in DNS.

2. **upstream block**: This tells Nginx about a group of servers that share
   the load. Nginx maintains a list of these servers and applies the chosen
   algorithm to select one for each request.

3. **proxy_pass**: This directive tells Nginx to forward the client's request
   to the named upstream. Nginx acts as a reverse proxy -- the client never
   talks directly to the backends.

4. **Round Robin (default)**: Without an explicit algorithm directive, Nginx
   uses Round Robin. It cycles through the servers in the order they are
   listed.

5. **least_conn**: When added, Nginx tracks active connections to each
   backend and always picks the one with the fewest. The reload command tells
   the running Nginx master process to re-read its configuration without
   dropping connections.

## Common Mistakes

### 1. Using `localhost` instead of service names

```nginx
# WRONG -- localhost refers to the Nginx container itself
server localhost:5678;

# CORRECT -- use the Docker Compose service name
server backend-1:5678;
```

### 2. Forgetting the `events` block

Nginx requires an `events` block even if you do not customise it. Without it,
Nginx will refuse to start with a cryptic error.

### 3. Mounting nginx.conf to the wrong path

The default Nginx config file is at `/etc/nginx/nginx.conf`. If you mount to
a different path without telling Nginx to use it (`nginx -c /path/to/conf`),
Nginx will use its default config and ignore yours.

### 4. Not using `:ro` for the volume mount

Without `:ro`, the container could accidentally modify your host file. This
is a minor issue in development but a good habit.

### 5. Forgetting `depends_on`

Without `depends_on`, Docker Compose may start Nginx before the backends are
ready. Nginx will fail to resolve the upstream hostnames on the first request.
`depends_on` ensures the backends start first (though it does not wait for
them to be *ready* -- that requires health checks).

### 6. Mixing up host and container ports

The mapping `8080:80` means "host port 8080 maps to container port 80." Nginx
listens on port 80 inside the container. The client connects to port 8080 on
the host. Reversing these is a common source of "connection refused" errors.
