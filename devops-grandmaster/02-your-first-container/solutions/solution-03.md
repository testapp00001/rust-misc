# Solution 03: Manage a Multi-Container Application

## Part A: Launch Three Containers

```bash
# 1. Web server
docker run -d --name web -p 8080:80 nginx

# 2. Redis cache
docker run -d --name cache redis:7-alpine

# 3. Monitoring container (stays alive with sleep)
docker run -d --name monitor alpine sleep 3600
```

Verify all three are running:

```bash
docker ps
```

```
CONTAINER ID   IMAGE             COMMAND                  STATUS         PORTS                  NAMES
e5f6a7b8c9d0   alpine            "sleep 3600"             Up 10 seconds                         monitor
c3d4e5f6a7b8   redis:7-alpine    "docker-entrypoint.s…"   Up 15 seconds   6379/tcp              cache
a1b2c3d4e5f6   nginx             "/docker-entrypoint.…"   Up 20 seconds   0.0.0.0:8080->80/tcp   web
```

**Why `sleep 3600`?** Alpine does not have a long-running service by default.
Without `sleep`, the container would exit immediately. The `sleep 3600` command
keeps it alive for one hour so you can exec into it.

## Part B: Inspect the Running System

### List containers with custom format

```bash
docker ps --format '{{.Names}}\t{{.Image}}'
```

```
monitor   alpine
cache     redis:7-alpine
web       nginx
```

### View web logs

```bash
docker logs web
```

Initially empty (no requests yet). After visiting `http://localhost:8080` or
running `curl http://localhost:8080`, you will see access logs.

### Check monitor processes

```bash
docker top monitor
```

```
UID   PID   PPID   C   STIME   TTY   TIME      CMD
root  ...   ...    0   ...     ?     00:00:00   sleep 3600
```

### Get cache IP

```bash
docker inspect -f '{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}' cache
```

```
172.17.0.3
```

(The actual IP will vary. Record it for Part C.)

## Part C: Execute Commands Inside Running Containers

### Test connectivity from monitor to cache

```bash
# First, exec into monitor and install curl
docker exec monitor sh -c "apk add --no-cache curl"
```

Since containers on the default bridge network cannot resolve names, use
the IP address from Part B:

```bash
docker exec monitor sh -c "ping -c 2 172.17.0.3"
```

```
PING 172.17.0.3 (172.17.0.3): 56 data bytes
64 bytes from 172.17.0.3: seq=0 ttl=64 time=0.089 ms
64 bytes from 172.17.0.3: seq=1 ttl=64 time=0.058 ms
```

**Why not by name?** The default bridge network does not provide DNS resolution.
Only custom networks (created with `docker network create`) allow containers
to find each other by name. Exercise 05 covers custom networks.

### Explore the web container

```bash
docker exec -it web bash
```

Inside the container:

```bash
cat /etc/nginx/nginx.conf
# Shows the Nginx configuration

ls /usr/share/nginx/html/
# index.html  50x.html

exit
```

### Test Redis

```bash
docker exec cache redis-cli ping
```

```
PONG
```

This confirms Redis is running and accepting commands.

## Part D: Monitor All Containers

```bash
docker stats --no-stream
```

```
CONTAINER ID   NAME      CPU %   MEM USAGE / LIMIT   MEM %   NET I/O         BLOCK I/O
e5f6a7b8c9d0   monitor   0.00%   1.1MiB / 15.6GiB    0.01%   648B / 0B       0B / 0B
c3d4e5f6a7b8   cache     0.07%   3.5MiB / 15.6GiB    0.02%   2.1kB / 0B      0B / 0B
a1b2c3d4e5f6   web       0.00%   2.5MiB / 15.6GiB    0.02%   1.5kB / 615B    0B / 0B
```

**Answers:**
- **Least memory:** `monitor` (~1.1 MB) -- it is just running `sleep 3600`, which is nearly zero work.
- **Most memory:** `cache` (~3.5 MB) -- Redis loads its data structures and runs a background process.
- **Why:** Redis maintains an in-memory data store with internal structures (hash tables, lists, etc.).
  Nginx is efficient but needs worker processes. Alpine with `sleep` does almost nothing.

## Part E: Stop Containers in Order

```bash
docker stop monitor   # No dependencies
docker stop cache     # Backend service
docker stop web       # Frontend service (last)
```

Verify:

```bash
docker ps -a
```

All three show `Exited` status.

**Why order matters:** In a real application, you might have a web server that
depends on a database. If you stop the database first, the web server will
encounter errors. Stopping in dependency order (frontend first, then backend)
allows graceful shutdown.

## Part F: Clean Up Everything

```bash
docker rm monitor cache web
```

Verify:

```bash
docker ps -a
```

Empty (or no containers from this exercise).

**Alternative cleanup approaches:**

```bash
# Remove all stopped containers at once
docker rm $(docker ps -a -q)

# Nuclear option: remove everything unused
docker system prune -f
```

## Key Takeaways

- **Naming matters.** `--name` lets you refer to containers by human-readable
  names instead of random IDs.
- **Default bridge has limitations.** Containers cannot resolve each other by name.
  Custom networks (covered in Exercise 05) solve this.
- **`docker exec` is your debugging tool.** It lets you run commands inside a
  running container without restarting it.
- **Memory usage depends on the workload.** A `sleep` command uses nearly zero
  memory; a database uses more.
- **Orderly shutdown prevents cascading errors.** Stop dependent services first.
