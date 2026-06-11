# Exercise 02: Implement Blue-Green with Docker Compose

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Implement a working blue-green deployment using Docker Compose and nginx
as the traffic router. This exercise walks you through building the
infrastructure that enables instant traffic switching and rollback.

## Scenario

You have a simple web service that returns its version number. You need
to deploy new versions with zero downtime using blue-green deployment.

The service responds to `GET /health` with:
```json
{"status": "healthy", "version": "1.0.0", "color": "blue"}
```

## Tasks

### Part A: Create the Docker Compose File

Write a `docker-compose.yml` that defines:
- `app-blue` container running `myapp:v1`
- `app-green` container running `myapp:v2`
- `nginx` container that routes traffic to the active environment

<details>
<summary>Hint</summary>

All three containers should be on the same Docker network. The nginx
container needs a volume mount for its configuration file. Both app
containers should have health checks.

</details>

### Part B: Write the nginx Configuration

Create an `nginx.conf` that:
- Listens on port 80
- Proxies all traffic to the active environment
- Uses an include file for the upstream server so it can be swapped

<details>
<summary>Hint</summary>

Use an `include` directive for the upstream configuration. The active
environment config file contains a single `server` line pointing to
either `app-blue:8080` or `app-green:8080`.

```nginx
upstream active {
    include /etc/nginx/conf.d/active-env.conf;
}
```

</details>

### Part C: Write the Deployment Script

Create a `deploy.sh` script that:
1. Detects which environment is currently active
2. Deploys the new version to the idle environment
3. Waits for the idle environment to pass health checks
4. Switches nginx to point to the new environment
5. Reloads nginx
6. Keeps the old environment running for rollback

<details>
<summary>Hint 1</summary>

Track the active environment in a file (e.g., `/tmp/active-env`). On the
first run, default to `blue`.

</details>

<details>
<summary>Hint 2</summary>

To switch environments, overwrite the `active-env.conf` file and reload
nginx. The reload is graceful -- no dropped connections.

```bash
echo "server app-green:8080;" > active-env.conf
docker compose exec nginx nginx -s reload
```

</details>

### Part D: Write the Rollback Script

Create a `rollback.sh` script that switches back to the previous
environment instantly.

<details>
<summary>Hint</summary>

Rollback is just the reverse of the switch step. Read the current active
environment, determine the previous one, update the config file, and
reload nginx. The previous environment is still running, so no deployment
is needed.

</details>

## Success Criteria

- [ ] Docker Compose defines blue, green, and nginx containers on the same network
- [ ] nginx routes traffic based on the active-env.conf include file
- [ ] Deployment script detects current environment and deploys to idle
- [ ] Deployment script waits for health checks before switching
- [ ] Rollback script switches back to the previous environment in under 5 seconds

## What You Should Understand After This Exercise

Blue-green deployment is fundamentally about maintaining two identical
environments and routing traffic between them. The traffic router (nginx
in this case) is the control plane -- switching environments is just
changing which upstream nginx points to. Rollback is instant because the
old environment is still running.
