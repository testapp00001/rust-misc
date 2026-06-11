# Exercise 03: Manage a Multi-Container Application

**Type:** Independent
**Time:** 35 minutes
**Difficulty:** Medium

## Objective

Manage multiple containers simultaneously. This exercise simulates a real
scenario where you need to run several services, inspect their relationships,
monitor them, and clean them up properly.

## Scenario

You are setting up a local development environment that requires three services:

1. **Nginx** -- a web server (public-facing)
2. **Redis** -- an in-memory data store (backend)
3. **Alpine** -- a lightweight container running a loop that pings Redis

These three containers must run at the same time, be identifiable by name,
and be cleaned up when you are done.

## Tasks

### Part A: Launch Three Containers

Start all three containers in the background.

1. Start Nginx with the name `web` on port 8080:

```bash
docker run -d --name web -p 8080:80 nginx
```

2. Start Redis with the name `cache`:

```bash
docker run -d --name cache redis:7-alpine
```

3. Start an Alpine container named `monitor` that runs a loop.
   This container should stay running (not exit immediately):

```bash
docker run -d --name monitor alpine sleep 3600
```

Verify all three are running:

```bash
docker ps
```

You should see three containers: `web`, `cache`, and `monitor`.

<details>
<summary>Hint</summary>

Alpine is a minimal image -- it does not have most tools installed.
The `sleep 3600` command keeps it alive for an hour so you can work with it.

</details>

### Part B: Inspect the Running System

Without stopping any container, gather the following information:

1. List all running containers showing only their names and images:

```bash
docker ps --format '{{.Names}}\t{{.Image}}'
```

2. View the logs of the `web` container:

```bash
docker logs web
```

3. Check what processes are running inside the `monitor` container:

```bash
docker top monitor
```

4. Get the IP address of the `cache` container:

```bash
docker inspect -f '{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}' cache
```

Record the IP address -- you will need it in Part C.

<details>
<summary>Hint</summary>

`docker ps --format` lets you customize the output columns.
Common fields: `{{.Names}}`, `{{.Image}}`, `{{.Status}}`, `{{.Ports}}`.

</details>

### Part C: Execute Commands Inside Running Containers

Use `docker exec` to interact with running containers without restarting them.

1. Install `curl` inside the `monitor` container and use it to test the `web` server:

```bash
docker exec monitor sh -c "apk add --no-cache curl && curl -s http://web:80"
```

Wait -- the `monitor` container cannot resolve `web` by name because they
are on the default bridge network. Use the IP address you found instead,
or use `--link` (not recommended for production, but fine for learning).

Instead, exec into `monitor` and ping the `cache` container by IP:

```bash
docker exec monitor sh -c "ping -c 2 <CACHE_IP_ADDRESS>"
```

Replace `<CACHE_IP_ADDRESS>` with the actual IP from Part B.

2. Open an interactive shell inside the `web` container:

```bash
docker exec -it web bash
```

Inside the container, explore:
- `cat /etc/nginx/nginx.conf`
- `ls /usr/share/nginx/html/`
- `exit` to leave

3. Use the Redis CLI to check the cache server is alive:

```bash
docker exec cache redis-cli ping
```

You should see `PONG`.

<details>
<summary>Hint</summary>

`docker exec` runs a command inside a *running* container. The `-it` flags
give you an interactive terminal. Without `-it`, the command runs and exits.

</details>

### Part D: Monitor All Containers

Watch the resource usage of all containers at once:

```bash
docker stats
```

Press `Ctrl+C` after observing for about 10 seconds.

Answer these questions:
- Which container uses the least memory?
- Which container uses the most memory?
- Why do you think that is?

### Part E: Stop Containers in Order

In a real application, you often need to stop services in a specific order.
Practice this:

1. Stop the `monitor` container first (it depends on the others):

```bash
docker stop monitor
```

2. Stop the `cache` container:

```bash
docker stop cache
```

3. Stop the `web` container last:

```bash
docker stop web
```

4. Confirm all containers are stopped (not removed):

```bash
docker ps -a
```

All three should show status `Exited`.

### Part F: Clean Up Everything

Remove all three containers:

```bash
docker rm monitor cache web
```

Verify:

```bash
docker ps -a
```

The list should be empty (or show no containers from this exercise).

<details>
<summary>Hint</summary>

You can remove multiple containers in one command by listing their names.
Alternatively, `docker rm $(docker ps -a -q)` removes ALL stopped containers.

</details>

## Success Criteria

- [ ] You launched three containers with descriptive names
- [ ] You inspected containers with `docker ps`, `docker logs`, `docker top`, and `docker inspect`
- [ ] You executed commands inside running containers with `docker exec`
- [ ] You monitored resource usage with `docker stats`
- [ ] You stopped containers in a specific order
- [ ] You cleaned up all containers and verified they are gone
- [ ] You can explain why naming containers matters for multi-container setups
