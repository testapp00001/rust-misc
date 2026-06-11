# Exercise 02: Run, Inspect, Stop, Remove

**Type:** Guided
**Time:** 25 minutes
**Difficulty:** Easy

## Objective

Practice the core container lifecycle commands hands-on: `docker run`,
`docker ps`, `docker inspect`, `docker stop`, `docker start`, and `docker rm`.
By the end you will have moved a container through every state in its lifecycle.

## Setup

Make sure Docker is running:

```bash
docker --version
# Docker version 24.x.x or later
```

## Tasks

### Step 1: Run a Background Container

Start an Nginx web server in the background with the name `web-server`:

```bash
docker run -d --name web-server -p 8080:80 nginx
```

Verify it is running:

```bash
docker ps
```

You should see `web-server` in the output with status `Up`.

<details>
<summary>Hint</summary>

If port 8080 is already in use, pick another port like 8081:
`docker run -d --name web-server -p 8081:80 nginx`

</details>

### Step 2: Test the Container

Confirm the web server is responding. Try one of these:

```bash
curl http://localhost:8080
```

Or open `http://localhost:8080` in your browser.

You should see the "Welcome to nginx!" page.

### Step 3: Inspect the Container

Run each of these commands and observe the output:

```bash
# See the container's processes
docker top web-server

# See resource usage (press Ctrl+C to stop)
docker stats web-server

# See the full configuration (it is long!)
docker inspect web-server

# Extract just the IP address
docker inspect -f '{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}' web-server

# View the container's logs
docker logs web-server
```

Record the following information from your inspection:
1. The container's IP address
2. The process running inside the container
3. The image used to create it

<details>
<summary>Hint</summary>

`docker inspect` outputs JSON. The `-f` flag uses Go template syntax to
extract specific fields. The container IP is usually in the `172.17.x.x` range.

</details>

### Step 4: Stop and Start the Container

```bash
# Stop the container
docker stop web-server

# Confirm it is stopped
docker ps
# web-server should NOT appear

# But it still exists!
docker ps -a
# web-server should appear with status "Exited"

# Start it again
docker start web-server

# Confirm it is running again
docker ps
# web-server should be back with status "Up"
```

### Step 5: Observe What Changes Across Stop/Start

```bash
# Stop the container
docker stop web-server

# Check the logs -- do they survive a restart?
docker logs web-server

# Start it again
docker start web-server

# Check the logs again -- old logs are still there
docker logs web-server
```

Answer this question: **Do logs persist across stop/start cycles?**

### Step 6: Remove the Container

```bash
# You cannot remove a running container
docker rm web-server
# Error: You cannot remove a running container

# Stop it first, then remove
docker stop web-server
docker rm web-server

# Confirm it is gone
docker ps -a
# web-server should NOT appear in the list
```

Alternatively, force-remove a running container in one step:

```bash
# (Only if web-server is still running)
docker rm -f web-server
```

### Step 7: Verify Cleanup

```bash
# The container is gone
docker ps -a

# But the image is still cached
docker images | grep nginx
```

Answer this question: **Does removing a container delete the image?**

<details>
<summary>Hint</summary>

Containers and images are separate things. Removing a container only
removes the writable layer, not the read-only image underneath.

</details>

## Success Criteria

- [ ] You ran an Nginx container in the background and verified it serves traffic
- [ ] You used `docker top`, `docker stats`, `docker inspect`, and `docker logs`
- [ ] You stopped and restarted a container, confirming state persists
- [ ] You removed a container and confirmed the image still exists
- [ ] You can explain the difference between `docker stop` and `docker rm`
- [ ] You can explain why `docker rm` fails on a running container

## Common Mistakes to Avoid

- Trying to `docker rm` a running container (stop it first, or use `-f`)
- Confusing `docker stop` (graceful) with `docker kill` (immediate)
- Forgetting that `docker ps` only shows running containers (use `-a` for all)
- Expecting `docker rm` to delete the image (it does not)
