# Exercise 05: Container Lifecycle Management Scenario

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Apply everything you have learned about Docker container management in a
realistic scenario. You will create, manage, monitor, troubleshoot, and
clean up containers as part of a simulated production workflow.

## Scenario

You are the on-call engineer for a small startup. It is 2 AM and you need
to:

1. Deploy a monitoring stack (Prometheus and a sample app)
2. Verify everything is working
3. Simulate a failure and diagnose it
4. Recover the service
5. Clean up after yourself

This exercise walks you through the entire lifecycle.

## Tasks

### Phase 1: Deploy the Stack

#### Step 1.1: Create a Custom Network

Containers on the default bridge network cannot resolve each other by name.
Create a custom network:

```bash
docker network create app-network
```

Verify it exists:

```bash
docker network ls
```

#### Step 1.2: Launch the Application

Run a simple Python HTTP server that serves a health check endpoint.
Use the `python:3.11-slim` image:

```bash
docker run -d \
  --name app \
  --network app-network \
  -p 5000:5000 \
  python:3.11-slim \
  python -m http.server 5000
```

Verify it works:

```bash
curl http://localhost:5000
```

<details>
<summary>Hint</summary>

If port 5000 is in use (common on macOS for AirPlay), use a different port:
change both port mappings to 5001:5000.

</details>

#### Step 1.3: Launch a Monitoring Sidecar

Run an Alpine container on the same network that continuously checks the
application's health:

```bash
docker run -d \
  --name monitor \
  --network app-network \
  alpine \
  sh -c "while true; do wget -q -O- http://app:5000 > /dev/null && echo \$(date): app is healthy || echo \$(date): app is DOWN; sleep 5; done"
```

Verify the monitor is running and logging:

```bash
docker logs monitor
```

You should see lines like: `Wed Jun 11 04:00:00 UTC 2026: app is healthy`

<details>
<summary>Hint</summary>

Alpine uses `wget` by default, not `curl`. The `-q` flag suppresses
progress output and `-O-` sends the response to stdout.

</details>

### Phase 2: Verify the System

#### Step 2.1: Check All Containers

```bash
docker ps --format 'table {{.Names}}\t{{.Image}}\t{{.Status}}\t{{.Ports}}'
```

You should see both `app` and `monitor` running.

#### Step 2.2: Check Network Connectivity

Verify the containers can reach each other:

```bash
# From the host, check both services
curl http://localhost:5000

# Check monitor logs for successful health checks
docker logs --tail 5 monitor
```

#### Step 2.3: Inspect Resource Usage

```bash
docker stats --no-stream
```

Record the memory usage of each container.

#### Step 2.4: Document the Running State

Save the current state for reference:

```bash
docker ps --format '{{.Names}}: {{.Status}}' > /tmp/docker-state.txt
cat /tmp/docker-state.txt
```

### Phase 3: Simulate a Failure

#### Step 3.1: Kill the Application

Simulate a crash by killing the app container:

```bash
docker kill app
```

#### Step 3.2: Observe the Impact

Check what happened:

```bash
# Is the app still running?
docker ps

# What does the app's exit status show?
docker inspect -f '{{.State.ExitCode}}' app

# What does the monitor see?
docker logs --tail 10 monitor
```

The monitor should now be logging "app is DOWN" messages.

<details>
<summary>Hint</summary>

`docker kill` sends SIGKILL (immediate termination). Compare this with
`docker stop` which sends SIGTERM first and waits for a graceful shutdown.

</details>

#### Step 3.3: Investigate the Failure

Practice your debugging workflow:

```bash
# Check the app container's final state
docker inspect -f '{{.State.Status}}' app

# Check why it exited
docker inspect -f '{{.State.Error}}' app

# Check the app's last logs
docker logs --tail 20 app
```

### Phase 4: Recover the Service

#### Step 4.1: Restart the Application

```bash
docker start app
```

#### Step 4.2: Verify Recovery

```bash
# Is the app running again?
docker ps

# Can you reach it?
curl http://localhost:5000

# Is the monitor detecting recovery?
docker logs --tail 5 monitor
```

The monitor should now show "app is healthy" again.

#### Step 4.3: Explain the Recovery

Answer these questions:
1. Why did `docker start` work instead of `docker run`?
2. What is preserved across a stop/start cycle?
3. What would be different if you used `docker run` instead?

<details>
<summary>Hint</summary>

`docker start` reuses the existing container (with its configuration,
network settings, and name). `docker run` creates a brand new container.
Since the container was killed (not removed), it still exists in the
"Exited" state and can be restarted.

</details>

### Phase 5: Clean Up

#### Step 5.1: Stop All Containers

```bash
docker stop monitor app
```

#### Step 5.2: Remove All Containers

```bash
docker rm monitor app
```

#### Step 5.3: Remove the Network

```bash
docker network rm app-network
```

#### Step 5.4: Verify Complete Cleanup

```bash
# No containers
docker ps -a

# No custom networks (only default ones remain)
docker network ls

# Check disk usage
docker system df
```

#### Step 5.5: Nuclear Option (if needed)

If you want to remove everything Docker-related that is unused:

```bash
docker system prune -f
```

Warning: This removes ALL stopped containers, unused networks, dangling
images, and build cache.

## Success Criteria

- [ ] You created a custom Docker network
- [ ] You launched two containers that communicate by name
- [ ] You verified the system with `docker ps`, `docker logs`, `docker stats`
- [ ] You simulated a failure with `docker kill`
- [ ] You observed the failure impact through logs and inspection
- [ ] You recovered the service with `docker start`
- [ ] You cleaned up all containers and the network
- [ ] You can explain the difference between `docker stop`, `docker kill`, `docker start`, and `docker rm`
- [ ] You can explain why `docker start` works for recovery but `docker run` would create a new container

## Key Concepts Practiced

- **Container lifecycle**: create, start, stop, kill, restart, remove
- **Custom networks**: enabling container-to-container communication by name
- **Health monitoring**: using sidecar containers to watch application health
- **Failure simulation**: understanding how containers behave when killed
- **Recovery**: restarting stopped containers vs creating new ones
- **Cleanup**: proper teardown of multi-container setups

## Next Steps

This exercise used pre-existing images. The natural next question is:
how do you package YOUR application into a container image?

Continue to [Module 03: Writing Dockerfiles](../03-writing-dockerfiles/) to learn how.
