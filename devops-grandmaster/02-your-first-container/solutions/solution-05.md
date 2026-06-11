# Solution 05: Container Lifecycle Management Scenario

## Phase 1: Deploy the Stack

### Step 1.1: Create a Custom Network

```bash
docker network create app-network
```

```
7c2b8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7
```

Verify:

```bash
docker network ls
```

```
NETWORK ID     NAME          DRIVER    SCOPE
...            bridge        bridge    local
...            host          host      local
...            app-network   bridge    local    <-- your custom network
...            none          null      local
```

**Why a custom network?** The default bridge network does not provide DNS
resolution between containers. On a custom network, containers can reach
each other by name (e.g., `http://app:5000` works from the monitor container).

### Step 1.2: Launch the Application

```bash
docker run -d \
  --name app \
  --network app-network \
  -p 5000:5000 \
  python:3.11-slim \
  python -m http.server 5000
```

Verify:

```bash
curl http://localhost:5000
```

```
<!DOCTYPE HTML>
...
Directory listing for /
...
```

The Python HTTP server serves a directory listing. This confirms the
container is running and the port mapping works.

### Step 1.3: Launch a Monitoring Sidecar

```bash
docker run -d \
  --name monitor \
  --network app-network \
  alpine \
  sh -c "while true; do wget -q -O- http://app:5000 > /dev/null && echo \$(date): app is healthy || echo \$(date): app is DOWN; sleep 5; done"
```

Verify:

```bash
docker logs monitor
```

```
Wed Jun 11 04:00:10 UTC 2026: app is healthy
Wed Jun 11 04:00:15 UTC 2026: app is healthy
Wed Jun 11 04:00:20 UTC 2026: app is healthy
```

**Why this works:** The monitor container is on the same custom network as
the app container. It can resolve `app` by name to the app container's IP
address. The `wget` command checks if the HTTP server responds.

## Phase 2: Verify the System

### Step 2.1: Check All Containers

```bash
docker ps --format 'table {{.Names}}\t{{.Image}}\t{{.Status}}\t{{.Ports}}'
```

```
NAMES     IMAGE             STATUS          PORTS
monitor   alpine            Up 2 minutes
app       python:3.11-slim  Up 3 minutes    0.0.0.0:5000->5000/tcp
```

### Step 2.2: Check Network Connectivity

```bash
curl http://localhost:5000
# Returns the directory listing -- app is reachable from host

docker logs --tail 3 monitor
# Shows "app is healthy" -- monitor can reach app by name
```

### Step 2.3: Inspect Resource Usage

```bash
docker stats --no-stream
```

```
CONTAINER ID   NAME      CPU %   MEM USAGE / LIMIT   MEM %
...            monitor   0.00%   1.1MiB / 15.6GiB    0.01%
...            app       0.00%   9.5MiB / 15.6GiB    0.06%
```

The app container uses more memory because Python is a heavier runtime
than Alpine's shell.

### Step 2.4: Document the Running State

```bash
docker ps --format '{{.Names}}: {{.Status}}' > /tmp/docker-state.txt
cat /tmp/docker-state.txt
```

```
monitor: Up 5 minutes
app: Up 6 minutes
```

## Phase 3: Simulate a Failure

### Step 3.1: Kill the Application

```bash
docker kill app
```

```
app
```

**`docker kill` vs `docker stop`:**
- `docker stop` sends SIGTERM, waits 10 seconds for graceful shutdown, then SIGKILL.
- `docker kill` sends SIGKILL immediately. The process has no chance to clean up.

### Step 3.2: Observe the Impact

```bash
# Is the app running?
docker ps
# Only 'monitor' appears -- 'app' is gone
```

```bash
# Check app's exit state
docker inspect -f '{{.State.ExitCode}}' app
# 137 (128 + 9 = SIGKILL)
```

```bash
# Check monitor's perspective
docker logs --tail 10 monitor
```

```
Wed Jun 11 04:05:00 UTC 2026: app is healthy
Wed Jun 11 04:05:05 UTC 2026: app is healthy
Wed Jun 11 04:05:10 UTC 2026: app is DOWN
Wed Jun 11 04:05:15 UTC 2026: app is DOWN
Wed Jun 11 04:05:20 UTC 2026: app is DOWN
```

The monitor detected the failure within one polling interval (5 seconds).

### Step 3.3: Investigate the Failure

```bash
# Container status
docker inspect -f '{{.State.Status}}' app
# exited

# Exit code
docker inspect -f '{{.State.ExitCode}}' app
# 137

# Error field (usually empty for kill)
docker inspect -f '{{.State.Error}}' app
# (empty)

# Last logs
docker logs --tail 20 app
```

```
172.18.0.3 - - [11/Jun/2026:04:00:15 +0000] "GET / HTTP/1.1" 200 -
172.18.0.3 - - [11/Jun/2026:04:00:20 +0000] "GET / HTTP/1.1" 200 -
...
```

Exit code 137 means the process was killed by SIGKILL (signal 9).
This matches our `docker kill` command.

## Phase 4: Recover the Service

### Step 4.1: Restart the Application

```bash
docker start app
```

```
app
```

### Step 4.2: Verify Recovery

```bash
docker ps
# Both 'app' and 'monitor' are running
```

```bash
curl http://localhost:5000
# Directory listing appears -- app is responding again
```

```bash
docker logs --tail 5 monitor
```

```
Wed Jun 11 04:05:30 UTC 2026: app is DOWN
Wed Jun 11 04:05:35 UTC 2026: app is DOWN
Wed Jun 11 04:05:40 UTC 2026: app is healthy
Wed Jun 11 04:05:45 UTC 2026: app is healthy
```

The monitor detected recovery within one polling interval.

### Step 4.3: Explain the Recovery

**Why did `docker start` work instead of `docker run`?**

`docker kill` terminates the process but does NOT remove the container.
The container still exists in the "Exited" state with all its configuration
(name, port mappings, network settings, environment variables). `docker start`
reuses this existing container, restarting the same process with the same
configuration.

**What is preserved across a stop/start cycle?**
- Container name and ID
- Port mappings
- Network configuration
- Environment variables
- Volumes (if any)
- Logs from previous runs
- The container's writable layer (files created/modified inside)

**What would be different with `docker run`?**

`docker run` creates a BRAND NEW container. This means:
- You would get an error if the name `app` is still taken by the stopped container
- You would need to specify all options again (`--name`, `--network`, `-p`, etc.)
- A new container ID would be assigned
- Previous logs would be lost (they belong to the old container)

To use `docker run` instead, you would first need to `docker rm app`,
then run the full command again. This is wasteful and error-prone for
simple recovery scenarios.

## Phase 5: Clean Up

### Step 5.1: Stop All Containers

```bash
docker stop monitor app
```

```
monitor
app
```

### Step 5.2: Remove All Containers

```bash
docker rm monitor app
```

```
monitor
app
```

### Step 5.3: Remove the Network

```bash
docker network rm app-network
```

```
app-network
```

### Step 5.4: Verify Complete Cleanup

```bash
docker ps -a
# No containers from this exercise

docker network ls
# Only default networks remain (bridge, host, none)

docker system df
# Shows disk usage by images, containers, volumes, build cache
```

### Step 5.5: Nuclear Option

```bash
docker system prune -f
```

This removes:
- All stopped containers
- All unused networks
- All dangling images (untagged layers)
- All build cache

It does NOT remove:
- Running containers
- Images tagged and in use
- Volumes (use `docker system prune --volumes -f` for those)

## Key Concepts Practiced

| Concept | What You Learned |
|---------|-----------------|
| Custom networks | Containers on the same custom network can resolve each other by name |
| Sidecar pattern | A lightweight container that monitors the main application |
| `docker kill` | Sends SIGKILL for immediate termination (exit code 137) |
| `docker stop` | Sends SIGTERM, waits, then SIGKILL (graceful shutdown) |
| `docker start` | Restarts an existing stopped container with the same configuration |
| `docker run` | Creates a new container (not the same as restart!) |
| Failure detection | Monitoring containers can detect failures within their polling interval |
| Ordered cleanup | Stop services, remove containers, remove networks |

## Common Mistakes to Avoid

- **Using `docker run` to "restart" a service.** Use `docker start` on the
  existing container. `docker run` creates a new one and requires re-specifying
  all options.

- **Forgetting to remove containers before removing images.** Containers hold
  a reference to their image. You cannot remove an image while containers
  that use it exist.

- **Not using custom networks.** The default bridge network does not provide
  DNS resolution. Containers cannot reach each other by name. Always use
  custom networks for multi-container setups.

- **Confusing `docker stop` and `docker kill`.** Use `docker stop` for
  graceful shutdown (gives the process time to clean up). Use `docker kill`
  only when the process is hung and will not respond to SIGTERM.

- **Not cleaning up networks.** Custom networks persist even after all
  containers are removed. Always `docker network rm` when done.
