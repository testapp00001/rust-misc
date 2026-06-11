# Exercise 03: Independent — Traefik with Auto-Discovery

## Type
Independent — minimal guidance, you research and build.

## Objective

Deploy Traefik as a reverse proxy that automatically discovers backend services via Docker labels. No manual configuration files — everything is declared in `docker-compose.yml` labels.

## Prerequisites

- Docker and Docker Compose installed
- Basic understanding of Docker labels and container networking

## Instructions

### Overview

You will deploy:
1. **Traefik** as the reverse proxy (the only service that exposes ports 80 and 443).
2. **A "whoami" service** — a tiny HTTP server that prints request details. Traefik provides an official `traefik/whoami` image for testing.
3. **A second "whoami" service** on a different route, to prove that Traefik auto-discovers new containers.

### Requirements

Your `docker-compose.yml` must achieve the following:

1. **Traefik configuration** (via CLI flags or a static config file):
   - Enable the Docker provider.
   - Set `exposedByDefault` to `false` so only labeled services are routed.
   - Create an entrypoint named `web` on port 80.
   - Enable the Traefik dashboard on port 8080.

2. **Service 1 — `whoami-1`**:
   - Use the image `traefik/whoami`.
   - Add Docker labels so Traefik routes requests with `Host(`whoami.localhost`)` to this service.
   - Do NOT expose any ports directly — Traefik handles all traffic.

3. **Service 2 — `whoami-2`**:
   - Use the image `traefik/whoami`.
   - Add Docker labels so Traefik routes requests with `Host(`whoami2.localhost`)` to this service.
   - Do NOT expose any ports directly.

4. **Traefik dashboard**:
   - Accessible at `http://localhost:8080/dashboard/` (note the trailing slash).
   - Configure via a router label on the Traefik service itself.

### Steps

1. Create a directory `exercise-03/` with a single `docker-compose.yml`.
2. Write the complete Compose file with all services and labels.
3. Deploy with `docker compose up -d`.
4. Test the following:

```bash
# Test whoami-1
curl http://whoami.localhost
# Expected: response showing request details (IP, headers, etc.)

# Test whoami-2
curl http://whoami2.localhost
# Expected: response showing request details from the second container

# Verify they are different containers
curl http://whoami.localhost | grep "Hostname"
curl http://whoami2.localhost | grep "Hostname"
# Expected: different hostnames

# Access the Traefik dashboard
curl http://localhost:8080/dashboard/
# Expected: HTML response (the dashboard UI)
```

5. **Dynamic test**: While the stack is running, add a third service `whoami-3` with `Host(`whoami3.localhost`)` to your Compose file. Run `docker compose up -d` again. Verify that `curl http://whoami3.localhost` works WITHOUT restarting Traefik.

## Success Criteria

- [ ] `curl http://whoami.localhost` returns response details from `whoami-1`.
- [ ] `curl http://whoami2.localhost` returns response details from `whoami-2`.
- [ ] The two services return different hostnames (proving they are separate containers).
- [ ] The Traefik dashboard is accessible at `http://localhost:8080/dashboard/`.
- [ ] Adding a third service does NOT require restarting Traefik.
- [ ] No service exposes ports to the host except Traefik (ports 80 and 8080).

## Hints

<details>
<summary>Hint 1 — Traefik static configuration</summary>
Use the `command` key in the Traefik service to pass CLI flags:
```yaml
command:
  - "--api.dashboard=true"
  - "--providers.docker=true"
  - "--providers.docker.exposedbydefault=false"
  - "--entrypoints.web.address=:80"
```
</details>

<details>
<summary>Hint 2 — Docker labels for routing</summary>
Each service needs three labels minimum:
```yaml
labels:
  - "traefik.enable=true"
  - "traefik.http.routers.MYNAME.rule=Host(`...`)"
  - "traefik.http.services.MYNAME.loadbalancer.server.port=PORT"
```
</details>

<details>
<summary>Hint 3 — Dashboard routing</summary>
The dashboard requires a special label on the Traefik service itself:
```yaml
- "traefik.http.routers.dashboard.rule=PathPrefix(`/dashboard`) || PathPrefix(`/api`)"
- "traefik.http.routers.dashboard.service=api@internal"
```
The `api@internal` is Traefik's built-in API service — you do NOT point it at a container.
</details>

<details>
<summary>Hint 4 — The dashboard shows 404</summary>
You must access the dashboard with a trailing slash: `/dashboard/` not `/dashboard`. Also ensure the dashboard router is attached to the `web` entrypoint.
</details>

<details>
<summary>Hint 5 — `exposedByDefault`</summary>
Setting `exposedByDefault=false` means Traefik ignores any container that does not have `traefik.enable=true`. This is a security best practice — you must explicitly opt in each service.
</details>
