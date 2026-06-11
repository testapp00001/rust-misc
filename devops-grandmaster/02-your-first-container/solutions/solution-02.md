# Solution 02: Run, Inspect, Stop, Remove

## Step 1: Run a Background Container

```bash
docker run -d --name web-server -p 8080:80 nginx
```

Expected output (a long container ID):

```
a1b2c3d4e5f6...
```

Verify:

```bash
docker ps
```

```
CONTAINER ID   IMAGE   COMMAND                  STATUS         PORTS                  NAMES
a1b2c3d4e5f6   nginx   "/docker-entrypoint.…"   Up 5 seconds   0.0.0.0:8080->80/tcp   web-server
```

**What happened:**
- Docker pulled the `nginx:latest` image (if not cached)
- Created a container named `web-server`
- Started Nginx in detached mode (`-d`)
- Mapped host port 8080 to container port 80 (`-p 8080:80`)

## Step 2: Test the Container

```bash
curl http://localhost:8080
```

Expected output (the Nginx welcome page HTML):

```html
<!DOCTYPE html>
<html>
<head>
<title>Welcome to nginx!</title>
...
```

If you see this, the container is serving traffic correctly.

**Why this works:** The `-p 8080:80` flag created a port forwarding rule.
Any request to `localhost:8080` on the host is forwarded to port 80 inside
the container, where Nginx is listening.

## Step 3: Inspect the Container

### docker top -- See running processes

```bash
docker top web-server
```

```
UID   PID   PPID   C   STIME   TTY   TIME      CMD
root  1234  1200   0   04:00   ?     00:00:00   nginx: master process nginx -g daemon off;
33    1289  1234   0   04:00   ?     00:00:00   nginx: worker process
```

This shows the actual OS processes running inside the container. Nginx runs
a master process and one or more worker processes.

### docker stats -- See resource usage

```bash
docker stats --no-stream web-server
```

```
CONTAINER ID   NAME        CPU %   MEM USAGE / LIMIT   MEM %   NET I/O       BLOCK I/O
a1b2c3d4e5f6   web-server  0.00%   2.5MiB / 15.6GiB    0.02%   1.2kB / 0B    0B / 0B
```

Nginx is very lightweight -- typically uses only 2-5 MB of memory.

### docker inspect -- Full configuration

```bash
docker inspect web-server
```

This produces a large JSON blob. The most useful sections are:
- `State` -- running status, start time, PID
- `NetworkSettings` -- IP address, ports, networks
- `HostConfig` -- port bindings, resource limits, restart policy

### Extract specific fields

```bash
# IP address
docker inspect -f '{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}' web-server
# Expected: 172.17.0.2 (or similar)

# State
docker inspect -f '{{.State.Status}}' web-server
# Expected: running

# Image
docker inspect -f '{{.Config.Image}}' web-server
# Expected: nginx
```

### docker logs -- View output

```bash
docker logs web-server
```

```
172.17.0.1 - - [11/Jun/2026:04:00:00 +0000] "GET / HTTP/1.1" 200 615 ...
```

The logs show the HTTP requests Nginx has received (including your `curl`).

## Step 4: Observe What Changes Across Stop/Start

```bash
docker stop web-server
docker logs web-server   # Logs are still there
docker start web-server
docker logs web-server   # Old logs + new logs
```

**Answer: Yes, logs persist across stop/start cycles.** The container's
writable layer (where logs are stored) survives stops. Only `docker rm`
deletes the container and its logs.

## Step 5: Remove the Container

```bash
docker stop web-server
docker rm web-server
```

Or in one command:

```bash
docker rm -f web-server
```

**Answer: No, removing a container does NOT delete the image.** The image
is a read-only template shared by all containers created from it. Removing
a container only deletes its writable layer.

```bash
docker images | grep nginx
# nginx   latest   ...   ...   187MB
```

The image is still there, ready to create new containers.

## Key Takeaways

| Action | Affects Container | Affects Image |
|--------|------------------|---------------|
| `docker stop` | Stops the process | No effect |
| `docker start` | Restarts the process | No effect |
| `docker rm` | Deletes the container | No effect |
| `docker rmi` | Cannot remove if containers use it | Deletes the image |

## Common Mistakes to Avoid

- **`docker rm` on a running container.** Always stop first, or use `docker rm -f`.
- **Confusing `docker stop` with `docker rm`.** Stop pauses; remove deletes.
- **Expecting `docker rm` to free image space.** It does not. Use `docker rmi` for images.
- **Forgetting `docker ps -a`.** `docker ps` only shows running containers. Stopped
  containers are hidden unless you use `-a`.
